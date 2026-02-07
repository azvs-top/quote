use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {

    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    

    #[error("could not find user config directory")]
    ConfigDirNotFound,

    #[error("missing pgsql config while backend is pgsql")]
    MissingPgsqlConfig,

    #[error("missing file config while backend is file")]
    MissingFileConfig,

    #[error("empty json condition list")]
    EmptyJsonCondition,

    #[error("Invalid JSON path segment: {0}")]
    InvalidJsonPath(String),

    #[error("Invalid Filter: {0}")]
    InvalidFilter(String),
    
    
    // ##### Quote ##### //
    #[error("content is missing")]
    QuoteMissingContent,

    #[error("invalid content")]
    QuoteInvalidContent,

    #[error("quote not found")]
    QuoteNotFound,
}