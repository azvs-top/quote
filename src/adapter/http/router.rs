use axum::routing::get;
use axum::Router;

use crate::app::AppState;

use super::hello::hello;
use super::quote::{get_quote_by_id, get_quote_random, list_quotes};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hello", get(hello))
        .route("/quotes", get(list_quotes))
        .route("/quote/{id}", get(get_quote_by_id))
        .route("/quote/random", get(get_quote_random))
}
