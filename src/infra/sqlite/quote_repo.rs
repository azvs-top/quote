use crate::application::ApplicationError;
use crate::application::quote::{QuoteCreate, QuoteFilter, QuotePort, QuoteQuery, QuoteUpdate};
use crate::domain::entity::{MultiLangObject, MultiLangText, Quote};
use crate::domain::value::ObjectKey;
use async_trait::async_trait;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

pub struct SqliteQuoteRepo {
    pool: SqlitePool,
}

impl SqliteQuoteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn map_sqlx_error(err: sqlx::Error, op: &str) -> ApplicationError {
        match err {
            sqlx::Error::Database(db_err) => {
                let code = db_err
                    .code()
                    .map(|code| code.into_owned())
                    .unwrap_or_else(|| "unknown".to_string());
                let message = db_err.message();
                Self::map_sqlite_db_error(op, &code, message)
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

    fn map_sqlite_db_error(op: &str, code: &str, message: &str) -> ApplicationError {
        // SQLite constraint family:
        // - base: 19
        // - extended examples:
        //   CHECK=275, FOREIGNKEY=787, NOTNULL=1299, PRIMARYKEY=1555,
        //   TRIGGER=1811, UNIQUE=2067, ROWID=2579
        match code {
            // conflict-like constraints
            "1555" | "2067" | "2579" => {
                ApplicationError::Conflict(format!("{op} failed ({code}): {message}"))
            }
            // input-invalid constraints
            "19" | "275" | "787" | "1299" | "1811" | "1043" | "2323" | "2835" | "3091" => {
                ApplicationError::InvalidInput(format!("{op} failed ({code}): {message}"))
            }
            _ => {
                // Some drivers may only surface generic "constraint failed" text.
                if message.to_ascii_lowercase().contains("constraint") {
                    ApplicationError::InvalidInput(format!("{op} failed ({code}): {message}"))
                } else {
                    ApplicationError::Dependency(format!("{op} failed ({code}): {message}"))
                }
            }
        }
    }

    fn serialize_json_text<T: serde::Serialize>(
        value: &T,
        field: &str,
    ) -> Result<String, ApplicationError> {
        serde_json::to_string(value)
            .map_err(|err| ApplicationError::Dependency(format!("serialize {field} failed: {err}")))
    }

    fn deserialize_json_text<T: serde::de::DeserializeOwned>(
        value: String,
        field: &str,
    ) -> Result<T, ApplicationError> {
        serde_json::from_str(&value).map_err(|err| {
            ApplicationError::Dependency(format!("deserialize {field} failed: {err}"))
        })
    }

    fn map_row_to_quote(row: QuoteRow) -> Result<Quote, ApplicationError> {
        let inline: MultiLangText = Self::deserialize_json_text(row.inline, "inline")?;
        let external: MultiLangObject = Self::deserialize_json_text(row.external, "external")?;
        let markdown: MultiLangObject = Self::deserialize_json_text(row.markdown, "markdown")?;
        let image: Vec<ObjectKey> = Self::deserialize_json_text(row.image, "image")?;

        Quote::new(row.id, inline, external, markdown, image, row.remark)
            .map_err(ApplicationError::from)
    }

    fn is_empty_filter(filter: &QuoteFilter) -> bool {
        filter.all_of.is_empty()
            && filter.any_of.is_empty()
            && filter.not.is_none()
            && filter.inline_all.is_empty()
            && filter.inline_any.is_empty()
            && filter.external_all.is_empty()
            && filter.external_any.is_empty()
            && filter.markdown_all.is_empty()
            && filter.markdown_any.is_empty()
            && filter.image_exists.is_none()
    }

    fn push_lang_all(
        qb: &mut QueryBuilder<'_, Sqlite>,
        col: &str,
        langs: &[crate::domain::value::Lang],
    ) {
        qb.push("(");
        for (idx, lang) in langs.iter().enumerate() {
            if idx > 0 {
                qb.push(" AND ");
            }
            Self::push_lang_exists_expr(qb, col, lang);
        }
        qb.push(")");
    }

    fn push_lang_any(
        qb: &mut QueryBuilder<'_, Sqlite>,
        col: &str,
        langs: &[crate::domain::value::Lang],
    ) {
        qb.push("(");
        for (idx, lang) in langs.iter().enumerate() {
            if idx > 0 {
                qb.push(" OR ");
            }
            Self::push_lang_exists_expr(qb, col, lang);
        }
        qb.push(")");
    }

    fn push_lang_exists_expr(
        qb: &mut QueryBuilder<'_, Sqlite>,
        col: &str,
        lang: &crate::domain::value::Lang,
    ) {
        // Use one stable expression form so future expression indexes can be added
        // without changing repository code.
        qb.push("json_extract(");
        qb.push(col);
        qb.push(", '$.");
        qb.push(lang.as_str());
        qb.push("') IS NOT NULL");
    }

    fn push_filter_expr(
        qb: &mut QueryBuilder<'_, Sqlite>,
        filter: &QuoteFilter,
    ) -> Result<(), ApplicationError> {
        if Self::is_empty_filter(filter) {
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
                qb.push("json_array_length(image) > 0");
            } else {
                qb.push("json_array_length(image) = 0");
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

#[cfg(test)]
mod tests {
    use super::SqliteQuoteRepo;
    use crate::application::ApplicationError;

    #[test]
    fn sqlite_constraint_codes_are_mapped_to_user_facing_errors() {
        let conflict =
            SqliteQuoteRepo::map_sqlite_db_error("op", "2067", "UNIQUE constraint failed");
        assert!(matches!(conflict, ApplicationError::Conflict(_)));

        let invalid_input =
            SqliteQuoteRepo::map_sqlite_db_error("op", "787", "FOREIGN KEY constraint failed");
        assert!(matches!(invalid_input, ApplicationError::InvalidInput(_)));

        let generic = SqliteQuoteRepo::map_sqlite_db_error("op", "19", "constraint failed");
        assert!(matches!(generic, ApplicationError::InvalidInput(_)));
    }
}

#[derive(Debug, FromRow)]
struct QuoteRow {
    id: i64,
    inline: String,
    external: String,
    markdown: String,
    image: String,
    remark: Option<String>,
}

#[async_trait]
impl QuotePort for SqliteQuoteRepo {
    async fn create(&self, create: QuoteCreate) -> Result<Quote, ApplicationError> {
        let inline = Self::serialize_json_text(&create.inline, "inline")?;
        let external = Self::serialize_json_text(&create.external, "external")?;
        let markdown = Self::serialize_json_text(&create.markdown, "markdown")?;
        let image = Self::serialize_json_text(&create.image, "image")?;

        let row = sqlx::query_as::<_, QuoteRow>(
            r#"
            INSERT INTO quote (inline, external, markdown, image, remark)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, inline, external, markdown, image, remark
            "#,
        )
        .bind(inline)
        .bind(external)
        .bind(markdown)
        .bind(image)
        .bind(create.remark)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| Self::map_sqlx_error(err, "insert quote"))?;

        Self::map_row_to_quote(row)
    }

    async fn get(&self, query: QuoteQuery) -> Result<Quote, ApplicationError> {
        let mut qb = QueryBuilder::<Sqlite>::new(
            "SELECT id, inline, external, markdown, image, remark FROM quote WHERE 1=1",
        );

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
            qb.push(" LIMIT 1");
        } else {
            if !Self::is_empty_filter(query.filter()) {
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
        let mut qb = QueryBuilder::<Sqlite>::new(
            "SELECT id, inline, external, markdown, image, remark FROM quote WHERE 1=1",
        );

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        }

        if !Self::is_empty_filter(query.filter()) {
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
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT COUNT(1) FROM quote WHERE 1=1");

        if let Some(id) = query.id() {
            qb.push(" AND id = ");
            qb.push_bind(id);
        }

        if !Self::is_empty_filter(query.filter()) {
            qb.push(" AND (");
            Self::push_filter_expr(&mut qb, query.filter())?;
            qb.push(")");
        }

        qb.build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(|err| Self::map_sqlx_error(err, "count quote"))
    }

    async fn update(&self, update: QuoteUpdate) -> Result<Quote, ApplicationError> {
        let mut qb = QueryBuilder::<Sqlite>::new("UPDATE quote SET ");
        let mut has_set = false;

        if let Some(inline) = update.inline {
            if has_set {
                qb.push(", ");
            }
            qb.push("inline = ");
            qb.push_bind(Self::serialize_json_text(&inline, "inline")?);
            has_set = true;
        }

        if let Some(external) = update.external {
            if has_set {
                qb.push(", ");
            }
            qb.push("external = ");
            qb.push_bind(Self::serialize_json_text(&external, "external")?);
            has_set = true;
        }

        if let Some(markdown) = update.markdown {
            if has_set {
                qb.push(", ");
            }
            qb.push("markdown = ");
            qb.push_bind(Self::serialize_json_text(&markdown, "markdown")?);
            has_set = true;
        }

        if let Some(image) = update.image {
            if has_set {
                qb.push(", ");
            }
            qb.push("image = ");
            qb.push_bind(Self::serialize_json_text(&image, "image")?);
            has_set = true;
        }

        if let Some(remark) = update.remark {
            if has_set {
                qb.push(", ");
            }
            qb.push("remark = ");
            qb.push_bind(remark);
            has_set = true;
        }

        if !has_set {
            return Err(ApplicationError::InvalidInput(
                "no fields to update".to_string(),
            ));
        }

        qb.push(" WHERE id = ");
        qb.push_bind(update.id);
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
        let result = sqlx::query("DELETE FROM quote WHERE id = ?")
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
