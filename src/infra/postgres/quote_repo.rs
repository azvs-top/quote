use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery};
use crate::domain::quote::{Quote, QuoteDraft, QuotePatch};
use crate::infra::postgres::error_mapper::map_sqlx_error;
use crate::infra::postgres::quote_mapper::{
    QuoteRow, draft_to_row_values, map_row_to_quote, quote_to_row_values,
};
use crate::infra::postgres::quote_filter_sql::push_filter_expr;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

pub struct PostgresQuoteRepo {
    pool: PgPool,
}

impl PostgresQuoteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuotePort for PostgresQuoteRepo {
    async fn create(&self, draft: QuoteDraft) -> Result<Quote, ApplicationError> {
        let (inline, external, markdown, image, remark) = draft_to_row_values(&draft)?;
        let row = sqlx::query_as::<_, QuoteRow>(
            r#"
            INSERT INTO quote.quote (inline, external, markdown, image, remark)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, inline, external, markdown, image, remark
            "#,
        )
        .bind(inline)
        .bind(external)
        .bind(markdown)
        .bind(image)
        .bind(remark)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_sqlx_error(err, "insert quote"))?;

        let quote = map_row_to_quote(row)?;

        Ok(quote)
    }

    async fn get(&self, query: QuoteQuery) -> Result<Quote, ApplicationError> {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT id, inline, external, markdown, image, remark FROM quote.quote WHERE 1=1",
        );

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        } else {
            if !query.filter().is_empty() {
                qb.push(" AND (");
                push_filter_expr(&mut qb, query.filter())?;
                qb.push(")");
            }
            qb.push(" ORDER BY random() LIMIT 1");
        }

        let row = qb
            .build_query_as::<QuoteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| map_sqlx_error(err, "get quote"))?
            .ok_or_else(|| ApplicationError::NotFound("quote not found".to_string()))?;

        map_row_to_quote(row)
    }

    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, ApplicationError> {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT id, inline, external, markdown, image, remark FROM quote.quote WHERE 1=1",
        );

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        }

        if !query.filter().is_empty() {
            qb.push(" AND (");
            push_filter_expr(&mut qb, query.filter())?;
            qb.push(")");
        }

        qb.push(" ORDER BY id ASC");

        if let Some(limit) = query.limit() {
            qb.push(" LIMIT ");
            qb.push_bind(limit);
        }
        if let Some(offset) = query.offset() {
            qb.push(" OFFSET ");
            qb.push_bind(offset);
        }

        let rows = qb
            .build_query_as::<QuoteRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|err| map_sqlx_error(err, "list quote"))?;

        rows.into_iter().map(map_row_to_quote).collect()
    }

    async fn count(&self, query: QuoteQuery) -> Result<i64, ApplicationError> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT COUNT(1) FROM quote.quote WHERE 1=1");

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        }

        if !query.filter().is_empty() {
            qb.push(" AND (");
            push_filter_expr(&mut qb, query.filter())?;
            qb.push(")");
        }

        qb.build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(|err| map_sqlx_error(err, "count quote"))
    }

    async fn update(&self, id: i64, patch: QuotePatch) -> Result<Quote, ApplicationError> {
        if patch.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "no fields to update".to_string(),
            ));
        }

        let current = self.get(QuoteQuery::builder().id(id).build()).await?;
        let next = current.apply(patch).map_err(ApplicationError::from)?;
        let (inline, external, markdown, image, remark) = quote_to_row_values(&next)?;

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE quote.quote SET ");
        qb.push("inline = ");
        qb.push_bind(inline);
        qb.push(", external = ");
        qb.push_bind(external);
        qb.push(", markdown = ");
        qb.push_bind(markdown);
        qb.push(", image = ");
        qb.push_bind(image);
        qb.push(", remark = ");
        qb.push_bind(remark);
        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.push(" RETURNING id, inline, external, markdown, image, remark");

        let row = qb
            .build_query_as::<QuoteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| map_sqlx_error(err, "update quote"))?
            .ok_or_else(|| ApplicationError::NotFound("quote not found".to_string()))?;

        map_row_to_quote(row)
    }

    async fn delete(&self, id: i64) -> Result<(), ApplicationError> {
        let result = sqlx::query("DELETE FROM quote.quote WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|err| map_sqlx_error(err, "delete quote"))?;

        if result == 0 {
            return Err(ApplicationError::NotFound("quote not found".to_string()));
        }
        Ok(())
    }
}
