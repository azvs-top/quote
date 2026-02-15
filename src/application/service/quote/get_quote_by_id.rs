use crate::application::quote::{QuotePort, QuoteQuery};
use crate::application::ApplicationError;
use crate::domain::entity::Quote;

/// 按 id 获取单条 Quote 的服务。
///
/// 功能：
/// - 校验 id 参数合法性（必须大于 0）。
/// - 通过 `QuotePort::get` 返回目标 Quote。
pub struct GetQuoteByIdService<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> GetQuoteByIdService<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, id: i64) -> Result<Quote, ApplicationError> {
        if id <= 0 {
            return Err(ApplicationError::InvalidInput(
                "id must be greater than 0".to_string(),
            ));
        }

        self.port.get(QuoteQuery::builder().id(id).build()).await
    }
}
