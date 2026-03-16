use crate::application::ApplicationError;
use crate::application::quote::{QuoteFilter, QuotePort, QuoteQuery};
use crate::domain::entity::Quote;

/// 获取随机 Quote 的服务。
///
/// 功能：
/// - 接收可选 `QuoteFilter` 作为筛选条件。
/// - 通过 `QuotePort::get` 返回一条匹配记录（由实现层决定随机策略）。
pub struct GetRandomQuoteService<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> GetRandomQuoteService<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, filter: Option<QuoteFilter>) -> Result<Quote, ApplicationError> {
        let query = QuoteQuery::builder().with_filter(filter).build();
        self.port.get(query).await
    }
}
