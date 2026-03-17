use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery};
use crate::domain::quote::{MultiLangObject, MultiLangText, Quote, QuoteDraft, QuotePatch, QuoteFilter};
use crate::domain::value::ObjectKey;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

pub struct PostgresQuoteRepo {
    pool: PgPool,
}

impl PostgresQuoteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 细粒度映射 sqlx 错误，保留业务语义（冲突/输入错误/依赖故障）。
    fn map_sqlx_error(err: sqlx::Error, op: &str) -> ApplicationError {
        match err {
            sqlx::Error::Database(db_err) => {
                let code = db_err
                    .code()
                    .map(|code| code.into_owned())
                    .unwrap_or_else(|| "unknown".to_string());
                let message = db_err.message();
                match code.as_str() {
                    // postgres unique_violation / mysql duplicate entry(sqlstate)
                    "23505" | "23000" => {
                        ApplicationError::Conflict(format!("{op} failed ({code}): {message}"))
                    }
                    // postgres not_null_violation / check_violation
                    "23502" | "23514" => {
                        ApplicationError::InvalidInput(format!("{op} failed ({code}): {message}"))
                    }
                    _ => ApplicationError::Dependency(format!("{op} failed ({code}): {message}")),
                }
            }
            sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::Io(_)
            | sqlx::Error::Tls(_)
            | sqlx::Error::Protocol(_) => {
                ApplicationError::Dependency(format!("{op} failed: {err}"))
            }
            _ => ApplicationError::Dependency(format!("{op} failed: {err}")),
        }
    }

    /// 将领域/应用结构序列化为 JSON 值，供 JSONB 字段写入。
    fn serialize_json_value<T: serde::Serialize + ?Sized>(
        value: &T,
        field: &str,
    ) -> Result<Value, ApplicationError> {
        serde_json::to_value(value)
            .map_err(|err| ApplicationError::Dependency(format!("serialize {field} failed: {err}")))
    }

    /// 从 JSON 值反序列化为目标结构，并附带字段名便于定位错误。
    fn deserialize_json_value<T: serde::de::DeserializeOwned>(
        value: Value,
        field: &str,
    ) -> Result<T, ApplicationError> {
        serde_json::from_value(value).map_err(|err| {
            ApplicationError::Dependency(format!("deserialize {field} failed: {err}"))
        })
    }

    /// 将数据库行转换为领域实体，并触发领域构造校验。
    fn map_row_to_quote(row: QuoteRow) -> Result<Quote, ApplicationError> {
        let inline: MultiLangText = Self::deserialize_json_value(row.inline, "inline")?;
        let external: MultiLangObject = Self::deserialize_json_value(row.external, "external")?;
        let markdown: MultiLangObject = Self::deserialize_json_value(row.markdown, "markdown")?;
        let image: Vec<ObjectKey> = Self::deserialize_json_value(row.image, "image")?;

        Quote::new(row.id, inline, external, markdown, image, row.remark)
            .map_err(ApplicationError::from)
    }

    /// 追加“语言全集匹配”条件：同一列必须同时包含所有语言 key。
    fn push_lang_all(
        qb: &mut QueryBuilder<'_, Postgres>,
        col: &str,
        langs: &[crate::domain::value::Lang],
    ) {
        qb.push("(");
        for (idx, lang) in langs.iter().enumerate() {
            if idx > 0 {
                qb.push(" AND ");
            }
            qb.push(col);
            qb.push(" ? ");
            qb.push_bind(lang.as_str().to_string());
        }
        qb.push(")");
    }

    /// 追加“语言任一匹配”条件：同一列包含任意一个语言 key 即可。
    fn push_lang_any(
        qb: &mut QueryBuilder<'_, Postgres>,
        col: &str,
        langs: &[crate::domain::value::Lang],
    ) {
        qb.push("(");
        for (idx, lang) in langs.iter().enumerate() {
            if idx > 0 {
                qb.push(" OR ");
            }
            qb.push(col);
            qb.push(" ? ");
            qb.push_bind(lang.as_str().to_string());
        }
        qb.push(")");
    }

    /// 递归构建过滤器 SQL 片段（支持 all_of / any_of / not 组合）。
    fn push_filter_expr(
        qb: &mut QueryBuilder<'_, Postgres>,
        filter: &QuoteFilter,
    ) -> Result<(), ApplicationError> {
        if filter.is_empty() {
            qb.push("TRUE");
            return Ok(());
        }

        let mut has_clause = false;

        if !filter.inline_all.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_all(qb, "inline", &filter.inline_all);
            has_clause = true;
        }
        if !filter.inline_any.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_any(qb, "inline", &filter.inline_any);
            has_clause = true;
        }
        if !filter.external_all.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_all(qb, "external", &filter.external_all);
            has_clause = true;
        }
        if !filter.external_any.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_any(qb, "external", &filter.external_any);
            has_clause = true;
        }
        if !filter.markdown_all.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_all(qb, "markdown", &filter.markdown_all);
            has_clause = true;
        }
        if !filter.markdown_any.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            Self::push_lang_any(qb, "markdown", &filter.markdown_any);
            has_clause = true;
        }
        if let Some(image_exists) = filter.image_exists {
            if has_clause {
                qb.push(" AND ");
            }
            if image_exists {
                qb.push("jsonb_array_length(image) > 0");
            } else {
                qb.push("jsonb_array_length(image) = 0");
            }
            has_clause = true;
        }

        if !filter.all_of.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            qb.push("(");
            for (idx, child) in filter.all_of.iter().enumerate() {
                if idx > 0 {
                    qb.push(" AND ");
                }
                qb.push("(");
                Self::push_filter_expr(qb, child)?;
                qb.push(")");
            }
            qb.push(")");
            has_clause = true;
        }

        if !filter.any_of.is_empty() {
            if has_clause {
                qb.push(" AND ");
            }
            qb.push("(");
            for (idx, child) in filter.any_of.iter().enumerate() {
                if idx > 0 {
                    qb.push(" OR ");
                }
                qb.push("(");
                Self::push_filter_expr(qb, child)?;
                qb.push(")");
            }
            qb.push(")");
            has_clause = true;
        }

        if let Some(not) = &filter.not {
            if has_clause {
                qb.push(" AND ");
            }
            qb.push("NOT (");
            Self::push_filter_expr(qb, not)?;
            qb.push(")");
            has_clause = true;
        }

        if !has_clause {
            qb.push("TRUE");
        }

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct QuoteRow {
    id: i64,
    inline: Value,
    external: Value,
    markdown: Value,
    image: Value,
    remark: Option<String>,
}

#[async_trait]
impl QuotePort for PostgresQuoteRepo {
    async fn create(&self, draft: QuoteDraft) -> Result<Quote, ApplicationError> {
        let inline = Self::serialize_json_value(draft.inline(), "inline")?;
        let external = Self::serialize_json_value(draft.external(), "external")?;
        let markdown = Self::serialize_json_value(draft.markdown(), "markdown")?;
        let image = Self::serialize_json_value(draft.image(), "image")?;
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
        .bind(draft.remark())
        .fetch_one(&self.pool)
        .await
        .map_err(|err| Self::map_sqlx_error(err, "insert quote"))?;

        let quote = Self::map_row_to_quote(row)?;

        Ok(quote)
    }

    async fn get(&self, query: QuoteQuery) -> Result<Quote, ApplicationError> {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT id, inline, external, markdown, image, remark FROM quote.quote WHERE 1=1",
        );

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
            qb.push(" LIMIT 1");
        } else {
            if !query.filter().is_empty() {
                qb.push(" AND (");
                Self::push_filter_expr(&mut qb, query.filter())?;
                qb.push(")");
            }
            qb.push(" ORDER BY random() LIMIT 1");
        }

        let row = qb
            .build_query_as::<QuoteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| Self::map_sqlx_error(err, "get quote"))?
            .ok_or_else(|| ApplicationError::NotFound("quote not found".to_string()))?;

        Self::map_row_to_quote(row)
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
            Self::push_filter_expr(&mut qb, query.filter())?;
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
            .map_err(|err| Self::map_sqlx_error(err, "list quote"))?;

        rows.into_iter().map(Self::map_row_to_quote).collect()
    }

    async fn count(&self, query: QuoteQuery) -> Result<i64, ApplicationError> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT COUNT(1) FROM quote.quote WHERE 1=1");

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        }

        if !query.filter().is_empty() {
            qb.push(" AND (");
            Self::push_filter_expr(&mut qb, query.filter())?;
            qb.push(")");
        }

        qb.build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(|err| Self::map_sqlx_error(err, "count quote"))
    }

    async fn update(&self, id: i64, patch: QuotePatch) -> Result<Quote, ApplicationError> {
        if patch.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "no fields to update".to_string(),
            ));
        }

        let current = self.get(QuoteQuery::builder().id(id).build()).await?;
        let next = current.apply(patch).map_err(ApplicationError::from)?;

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE quote.quote SET ");
        qb.push("inline = ");
        qb.push_bind(Self::serialize_json_value(next.inline(), "inline")?);
        qb.push(", external = ");
        qb.push_bind(Self::serialize_json_value(next.external(), "external")?);
        qb.push(", markdown = ");
        qb.push_bind(Self::serialize_json_value(next.markdown(), "markdown")?);
        qb.push(", image = ");
        qb.push_bind(Self::serialize_json_value(next.image(), "image")?);
        qb.push(", remark = ");
        qb.push_bind(next.remark());
        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.push(" RETURNING id, inline, external, markdown, image, remark");

        let row = qb
            .build_query_as::<QuoteRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| Self::map_sqlx_error(err, "update quote"))?
            .ok_or_else(|| ApplicationError::NotFound("quote not found".to_string()))?;

        Self::map_row_to_quote(row)
    }

    async fn delete(&self, id: i64) -> Result<(), ApplicationError> {
        let result = sqlx::query("DELETE FROM quote.quote WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|err| Self::map_sqlx_error(err, "delete quote"))?;

        if result == 0 {
            return Err(ApplicationError::NotFound("quote not found".to_string()));
        }
        Ok(())
    }
}
