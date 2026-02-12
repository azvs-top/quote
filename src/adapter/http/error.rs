use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppError;

use super::dto::ErrorBody;

pub struct HttpError(pub AppError);

impl From<AppError> for HttpError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::QuoteNotFound | AppError::DictNotFound | AppError::MinioObjectNotFound => {
                StatusCode::NOT_FOUND
            }
            AppError::InvalidFilter(_)
            | AppError::InvalidJsonPath(_)
            | AppError::QuoteInvalidContent
            | AppError::QuoteMissingContent
            | AppError::InvalidUtf8Content
            | AppError::EmptyJsonCondition => StatusCode::BAD_REQUEST,
            AppError::MinioUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorBody {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}
