use crate::application::ApplicationError;
use crate::application::storage::StoragePort;
use crate::domain::value::ObjectKey;
use std::collections::HashSet;

/// 批量删除对象的服务。
///
/// 语义：
/// - 尽力删除所有对象。
/// - 若存在失败，返回汇总错误。
pub struct DeleteManyService<'a> {
    port: &'a (dyn StoragePort + Send + Sync),
}

impl<'a> DeleteManyService<'a> {
    pub fn new(port: &'a (dyn StoragePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, keys: &[ObjectKey]) -> Result<(), ApplicationError> {
        let mut errors = Vec::new();
        let mut seen = HashSet::new();

        for key in keys {
            // 幂等删除：同一批次内重复 key 仅删除一次。
            if !seen.insert(key.as_str().to_string()) {
                continue;
            }

            if let Err(err) = self.port.delete(key).await {
                // 删除语义幂等：对象不存在视为已删除成功。
                if matches!(err, ApplicationError::NotFound(_)) {
                    continue;
                }
                errors.push(format!("{} => {}", key.as_str(), err));
            }
        }

        if errors.is_empty() {
            return Ok(());
        }

        Err(ApplicationError::Dependency(format!(
            "failed to delete {} object(s): {}",
            errors.len(),
            errors.join("; ")
        )))
    }
}
