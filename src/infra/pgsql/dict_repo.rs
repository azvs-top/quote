use crate::app::app_error::AppError;
use crate::dict::{Dict, DictPort, DictQuery, DictType};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

pub struct DictRepoPgsql {
    pool: PgPool,
}

impl DictRepoPgsql {
    pub fn new(pool: PgPool) -> Self {
        DictRepoPgsql { pool }
    }
}

#[async_trait]
impl DictPort for DictRepoPgsql {
    async fn get_type(&self, query: DictQuery) -> Result<DictType, AppError> {
        let type_id = query.type_id();
        let type_key = query.type_key();

        if type_id.is_none() && type_key.is_none() {
            return Err(AppError::DictNotFound);
        }

        let mut sql = QueryBuilder::<Postgres>::new(
            r#"SELECT DISTINCT
            type_id, type_key, type_name, type_active, type_creator, type_remark
            FROM "#
        );

        if let Some(langs) = query.langs() {
            sql.push("quote.f_dict(");
            sql.push_bind(langs);
            sql.push(")");
        } else {
            sql.push("quote.f_dict()");
        }

        sql.push(" WHERE 1=1");

        if let Some(type_id) = type_id {
            sql.push(" AND type_id = ");
            sql.push_bind(type_id);
        } else if let Some(type_key) = type_key {
            sql.push(" AND type_key = ");
            sql.push_bind(type_key);
        }

        sql.push(" ORDER BY type_id ASC LIMIT 1");

        let row = sql
            .build_query_as::<DictType>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::DictNotFound)?;

        Ok(row)
    }

    async fn list_type(&self, query: DictQuery) -> Result<Vec<DictType>, AppError> {
        let mut sql = QueryBuilder::<Postgres>::new(
            r#"SELECT DISTINCT
            type_id, type_key, type_name, type_active, type_creator, type_remark
            FROM "#
        );

        if let Some(langs) = query.langs() {
            sql.push("quote.f_dict(");
            sql.push_bind(langs);
            sql.push(")");
        } else {
            sql.push("quote.f_dict()");
        }

        sql.push(" WHERE 1=1");

        if let Some(type_id) = query.type_id() {
            sql.push(" AND type_id = ");
            sql.push_bind(type_id);
        }
        if let Some(type_key) = query.type_key() {
            sql.push(" AND type_key = ");
            sql.push_bind(type_key);
        }
        if let Some(type_creator) = query.type_creator() {
            sql.push(" AND type_creator = ");
            sql.push_bind(type_creator);
        }
        if let Some(type_active) = query.type_active() {
            sql.push(" AND type_active = ");
            sql.push_bind(type_active);
        }

        sql.push(" ORDER BY type_id ASC");

        if let Some(limit) = query.limit() {
            sql.push(" LIMIT ");
            sql.push_bind(limit);
        }
        if let Some(offset) = query.offset() {
            sql.push(" OFFSET ");
            sql.push_bind(offset);
        }


        let rows = sql
            .build_query_as::<DictType>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn list_item(&self, query: DictQuery) -> Result<Vec<Dict>, AppError> {
        let type_id = query.type_id();
        let type_key = query.type_key();

        let mut sql = QueryBuilder::<Postgres>::new(
            r#"SELECT
            type_id, type_key, type_name, type_active, type_creator, type_remark,
            item_id, item_key, item_value, is_default, item_active, item_creator, item_remark
            FROM "#
        );

        if let Some(langs) = query.langs() {
            sql.push("quote.f_dict(");
            sql.push_bind(langs);
            sql.push(")");
        } else {
            sql.push("quote.f_dict()");
        }

        sql.push(" WHERE 1=1");

        if let Some(type_id) = type_id {
            sql.push(" AND type_id = ");
            sql.push_bind(type_id);
        } else if let Some(type_key) = type_key {
            sql.push(" AND type_key = ");
            sql.push_bind(type_key);
        }

        if let Some(item_creator) = query.item_creator() {
            sql.push(" AND item_creator = ");
            sql.push_bind(item_creator);
        }

        if let Some(is_default) = query.is_default() {
            sql.push(" AND is_default = ");
            sql.push_bind(is_default);
        }

        if let Some(item_active) = query.item_active() {
            sql.push(" AND item_active = ");
            sql.push_bind(item_active);
        }

        sql.push(" ORDER BY type_id ASC, item_id ASC");

        if let Some(limit) = query.limit() {
            sql.push(" LIMIT ");
            sql.push_bind(limit);
        }
        if let Some(offset) = query.offset() {
            sql.push(" OFFSET ");
            sql.push_bind(offset);
        }

        let rows = sql
            .build_query_as::<Dict>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}
