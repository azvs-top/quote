use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub async fn hello() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
