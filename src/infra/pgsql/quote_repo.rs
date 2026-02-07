use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery, QuoteQueryFilter};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
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
        let pg = self.build_pg_json(filter)?;
         pg.sql(sql, "content")?;
        Ok(())
    }

    fn build_pg_json(&self, filter: &QuoteQueryFilter) -> Result<PgJson, AppError> {
        use QuoteQueryFilter::*;
        match filter {
            And(filters) => {
                if filters.is_empty() {
                    return Err(AppError::InvalidFilter(String::from("And filter cannot be empty")));
                }
                Ok(PgJson::And(
                    filters.iter()
                        .map(|f| self.build_pg_json(f))
                        .collect::<Result<Vec<_>, _>>()?,
                ))

            }
            Or(filters) => {
                if filters.is_empty() {
                    return Err(AppError::InvalidFilter(String::from("Or filter cannot be empty")));
                }
                Ok(PgJson::Or(
                    filters.iter()
                        .map(|f| self.build_pg_json(f))
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }

            HasInline => Ok(json_exists!("inline")),
            HasExternal => Ok(json_exists!("external")),
            HasMarkdown => Ok(json_exists!("markdown")),
            HasImage => Ok(json_exists!("image")),
            HasAudio => Ok(json_exists!("audio")),

            HasInlineAllLang(lang) => Self::build_lang_all("inline", lang),
            HasInlineAnyLang(lang) => Self::build_lang_any("inline", lang),
            HasExternalAllLang(lang) => Self::build_lang_any("external", lang),
            HasExternalAnyLang(lang) => Self::build_lang_any("external", lang),
            HasMarkdownAllLang(lang) => Self::build_lang_all("markdown", lang),
            HasMarkdownAnyLang(lang) => Self::build_lang_all("markdown", lang),
        }
    }

    fn build_lang_all(root: &str, langs: &[String]) -> Result<PgJson, AppError> {
        if langs.is_empty() {
            return Err(AppError::InvalidFilter(String::from("langs cannot be empty")));
        }
        let conditions = langs.iter()
            .map(|lang| {
                PgJson::Exists(vec![root.to_owned(), lang.to_owned()])
            }).collect::<Vec<_>>();
        Ok(PgJson::And(conditions))
    }

    fn build_lang_any(root: &str, langs: &[String]) -> Result<PgJson, AppError> {
        if langs.is_empty() {
            return Err(AppError::InvalidFilter(String::from("langs cannot be empty")));
        }
        let conditions = langs.iter()
        .map(|lang| {
            PgJson::Exists(vec![root.to_owned(), lang.to_owned()])
        })
        .collect::<Vec<_>>();
        Ok(PgJson::Or(conditions))
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

        sql.push(" ORDER BY id ASC");

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
