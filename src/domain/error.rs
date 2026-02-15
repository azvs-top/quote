use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("quote content is missing")]
    QuoteMissingContent,

    #[error("quote content is invalid")]
    QuoteInvalidContent,

    #[error("invalid quote id: {0}")]
    InvalidQuoteId(i64),

    #[error("quote not found")]
    QuoteNotFound,

    #[error("invalid language code: {0}")]
    InvalidLang(String),

    #[error("invalid object key: {0}")]
    InvalidObjectKey(String),
}
