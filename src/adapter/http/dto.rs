use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub struct RandomQuoteQuery {
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuotesQuery {
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}
