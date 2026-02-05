use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};

pub struct QuoteRepoPgsql {
    pool: PgPool,
}
impl QuoteRepoPgsql {
    pub fn new(pool: PgPool) -> Self {
        QuoteRepoPgsql { pool }
    }
}

#[async_trait]
impl QuotePort for QuoteRepoPgsql {
    async fn find_by_id(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        let id = query.id().ok_or(AppError::QuoteNotFound)?;
        let quote = sqlx::query_as::<_, Quote>(
            r#"
            SELECT id, content, active, remark
            FROM quote.quote
            WHERE id = $1
            "#
            ).bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::QuoteNotFound)?;

        Ok(quote)
    }

    async fn random_find_by_content_key(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(
            "SELECT id, content, active, remark FROM quote.quote WHERE 1=1"
        );
        if let Some(cond) = &query.cond() {
            sql.push(" AND ");
            cond.sql(&mut sql, "content")?;
        }
        sql.push(" ORDER BY random() DESC LIMIT 1");

        let result = sql.build_query_as::<Quote>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::QuoteNotFound)?;

        Ok(result)
    }
}