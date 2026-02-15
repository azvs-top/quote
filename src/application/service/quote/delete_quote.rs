use crate::application::quote::{QuotePort, QuoteQuery};
use crate::application::service::storage::DeleteManyService;
use crate::application::storage::StoragePort;
use crate::application::ApplicationError;
use crate::domain::value::ObjectKey;
use std::collections::HashSet;

/// 按 id 删除 Quote 的服务。
///
/// 功能：
/// - 校验 id 参数合法性（必须大于 0）。
/// - 先读取旧 Quote，收集 external/markdown/image 的对象 key。
/// - 执行 `QuotePort::delete` 删除主记录。
/// - 删除成功后批量清理对象存储，避免孤儿文件。
///
/// 错误语义：
/// - 若主记录删除失败，直接返回删除错误，不触发存储清理。
/// - 若主记录删除成功但存储清理失败，返回依赖错误并保留“主记录已删除”的事实。
pub struct DeleteQuoteService<'a> {
    quote_port: &'a (dyn QuotePort + Send + Sync),
    delete_many_service: DeleteManyService<'a>,
}

impl<'a> DeleteQuoteService<'a> {
    pub fn new(
        quote_port: &'a (dyn QuotePort + Send + Sync),
        storage_port: &'a (dyn StoragePort + Send + Sync),
    ) -> Self {
        Self {
            quote_port,
            delete_many_service: DeleteManyService::new(storage_port),
        }
    }

    pub async fn execute(&self, id: i64) -> Result<(), ApplicationError> {
        if id <= 0 {
            return Err(ApplicationError::InvalidInput(
                "id must be greater than 0".to_string(),
            ));
        }

        let existing = self.quote_port.get(QuoteQuery::builder().id(id).build()).await?;
        let keys = collect_storage_keys(&existing);

        self.quote_port.delete(id).await?;

        if keys.is_empty() {
            return Ok(());
        }

        if let Err(err) = self.delete_many_service.execute(&keys).await {
            return Err(ApplicationError::Dependency(format!(
                "quote deleted, but storage cleanup failed: {err}"
            )));
        }

        Ok(())
    }
}

fn collect_storage_keys(quote: &crate::domain::entity::Quote) -> Vec<ObjectKey> {
    // 某些对象 key 可能被多个字段复用，先去重后再删，避免重复调用底层 delete。
    let mut dedup = HashSet::new();
    let mut keys = Vec::new();

    for key in quote.external().values() {
        if dedup.insert(key.as_str().to_string()) {
            keys.push(key.clone());
        }
    }
    for key in quote.markdown().values() {
        if dedup.insert(key.as_str().to_string()) {
            keys.push(key.clone());
        }
    }
    for key in quote.image() {
        if dedup.insert(key.as_str().to_string()) {
            keys.push(key.clone());
        }
    }

    keys
}
