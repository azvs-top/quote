use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery};
use crate::domain::quote::Quote;

/// 分页查询 Quote 列表的服务。
///
/// 功能：
/// - 校验分页参数（`limit > 0`、`offset >= 0`）。
/// - 通过 `QuotePort::list` 返回满足条件的 Quote 集合。
pub struct ListQuoteService<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> ListQuoteService<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, query: QuoteQuery) -> Result<Vec<Quote>, ApplicationError> {
        if let Some(limit) = query.limit() {
            if limit <= 0 {
                return Err(ApplicationError::InvalidInput(
                    "limit must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(offset) = query.offset() {
            if offset < 0 {
                return Err(ApplicationError::InvalidInput(
                    "offset must be greater than or equal to 0".to_string(),
                ));
            }
        }

        self.port.list(query).await
    }
}
