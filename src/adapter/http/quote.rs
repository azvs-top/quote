use axum::extract::{Path, Query, State};
use axum::Json;

use crate::app::AppState;
use crate::quote::{GetQuoteById, GetQuoteRandom, ListQuotes, Quote, QuoteQuery, QuoteQueryFilter};

use super::dto::{ListQuotesQuery, RandomQuoteQuery};
use super::error::HttpError;

pub async fn get_quote_random(
    State(state): State<AppState>,
    Query(params): Query<RandomQuoteQuery>,
) -> Result<Json<Quote>, HttpError> {
    let mut builder = QuoteQuery::builder().with_active(params.active.or(Some(true)));

    if !state.config.quote.inline_langs.is_empty() {
        builder = builder.filter(QuoteQueryFilter::HasInlineAllLang(
            state.config.quote.inline_langs.clone(),
        ));
    }

    let quote = GetQuoteRandom::new(state.quote_port.as_ref())
        .execute(builder.build())
        .await?;

    Ok(Json(quote))
}

pub async fn get_quote_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Quote>, HttpError> {
    let quote = GetQuoteById::new(state.quote_port.as_ref())
        .execute(id)
        .await?;
    Ok(Json(quote))
}

pub async fn list_quotes(
    State(state): State<AppState>,
    Query(params): Query<ListQuotesQuery>,
) -> Result<Json<Vec<Quote>>, HttpError> {
    let limit = params.limit.map(|v| v as i64);
    let offset = params
        .page
        .map(|page| (page.saturating_sub(1) * params.limit.unwrap_or(10)) as i64);

    let query = QuoteQuery::builder()
        .with_active(params.active)
        .with_limit(limit)
        .with_offset(offset)
        .build();

    let quotes = ListQuotes::new(state.quote_port.as_ref()).execute(query).await?;
    Ok(Json(quotes))
}
