use crate::domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("could not resolve user config directory")]
    ConfigDirNotFound,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("resource conflict: {0}")]
    Conflict(String),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("dependency error: {0}")]
    Dependency(String),

    #[error(transparent)]
    Domain(#[from] DomainError),
}
