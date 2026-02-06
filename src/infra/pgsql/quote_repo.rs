use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery, QuoteQueryFilter};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use sqlx::query::Query;
use crate::{json_exists, PgJson};

pub struct QuoteRepoPgsql {
    pool: PgPool,
}
impl QuoteRepoPgsql {
    pub fn new(pool: PgPool) -> Self {
        QuoteRepoPgsql { pool }
    }

    fn apply_filter(
        &self,
        filter: &QuoteQueryFilter,
        sql: & mut QueryBuilder<Postgres>,
    ) -> Result<(), AppError> {
        match filter {
            QuoteQueryFilter::AllLangs(langs) => {
                let cond = langs.iter()
                    .map(|l| json_exists!("inline", l))
                    .collect();
                PgJson::And(cond).sql(sql, "content")?;
            }
            QuoteQueryFilter::AnyLang(langs) => {
                let cond = langs.iter()
                .map(|l| json_exists!("inline", l))
                .collect();
                PgJson::Or(cond).sql(sql, "content")?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl QuotePort for QuoteRepoPgsql {
    async fn get(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(
            "SELECT id, content, active, remark FROM quote.quote WHERE 1=1"
        );

        if let Some(active) = query.active() {
            sql.push(" AND active = ");
            sql.push_bind(active);
        }

        match query.id() {
            Some(id) => {
                sql.push(" AND id = ");
                sql.push_bind(id);
            }
            None => {
                if let Some(filter) = query.filter() {
                    sql.push(" AND ");
                    self.apply_filter(filter, &mut sql)?;
                }
                sql.push(" ORDER BY random() LIMIT 1");
            }
        }

        let quote = sql.build_query_as::<Quote>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::QuoteNotFound)?;
        Ok(quote)
    }

    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(
            "SELECT id, content, active, remark FROM quote.quote WHERE 1=1"
        );

        if let Some(id) = query.id() {
            sql.push(" AND id = ");
            sql.push_bind(id);
        }

        if let Some(active) = query.active() {
            sql.push(" AND active = ");
            sql.push_bind(active);
        }

        if let Some(filter) = query.filter() {
            sql.push(" AND ");
            self.apply_filter(filter, &mut sql)?;
        }

        sql.push(" ORDER BY id DESC");

        if let Some(limit) = query.limit() {
            sql.push(" LIMIT ");
            sql.push_bind(limit);
        }

        if let Some(offset) = query.offset() {
            sql.push(" OFFSET ");
            sql.push_bind(offset);
        }

        let quotes = sql.build_query_as::<Quote>()
            .fetch_all(&self.pool)
            .await?;

        Ok(quotes)
    }
}
