use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    // ===== 基础设施 =====
    #[error("MinIO unavailable")]
    MinioUnavailable,

    #[error("MinIO access denied")]
    MinioAccessDenied,

    #[error("MinIO object not found")]
    MinioObjectNotFound,

    #[error("MinIO bucket not found")]
    MinioBucketNotFound,

    #[error("Object content is not valid UTF-8")]
    InvalidUtf8Content,

    #[error("External storage error")]
    ExternalStorageError,

    // ===== Config =====
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

    // ===== Quote =====
    #[error("content is missing")]
    QuoteMissingContent,

    #[error("invalid content")]
    QuoteInvalidContent,

    #[error("quote not found")]
    QuoteNotFound,
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        AppError::InvalidUtf8Content
    }
}

impl From<SdkError<GetObjectError>> for AppError {
    fn from(err: SdkError<GetObjectError>) -> Self {
        match err {
            // 匹配具体的服务错误
            SdkError::ServiceError(service_err) => {
                match service_err.err() {
                    GetObjectError::NoSuchKey(_) => AppError::MinioObjectNotFound,
                    GetObjectError::InvalidObjectState(_) => AppError::ExternalStorageError,
                    _ => AppError::ExternalStorageError,
                }
            }

            // 网络相关错误
            SdkError::TimeoutError(_) => AppError::MinioUnavailable,
            SdkError::DispatchFailure(_) => AppError::MinioUnavailable,

            // 其他错误
            _ => AppError::ExternalStorageError,
        }
    }
}
