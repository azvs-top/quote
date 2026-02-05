use crate::app::AppError;
use sqlx::{Postgres, QueryBuilder};

/// PgSQL的专属JSON条件判断拼接
#[derive(Debug, Clone)]
pub enum PgJson {
    // 逻辑非
    Not(Box<PgJson>),
    // 逻辑与
    And(Vec<PgJson>),
    // 逻辑或
    Or(Vec<PgJson>),
    // 路径存在
    Exists(Vec<String>),
    // 路径不存在
    NotExists(Vec<String>),
}

impl PgJson {
    /// 生成 SQL 片段
    pub fn sql<'a>(
        &self,
        sql: &mut QueryBuilder<'a, Postgres>,
        field: &str,
    ) -> Result<(), AppError> {
        match self {
            PgJson::Not(condition) => {
                sql.push("NOT (");
                condition.sql(sql, field)?;
                sql.push(")");
            }
            PgJson::And(list) => {
                if list.is_empty() {
                    return Err(AppError::EmptyJsonCondition);
                }
                sql.push("(");
                for (i, c) in list.iter().enumerate() {
                    if i > 0 {
                        sql.push(" AND ");
                    }
                    c.sql(sql, field)?;
                }
                sql.push(")");
            }
            PgJson::Or(list) => {
                if list.is_empty() {
                    return Err(AppError::EmptyJsonCondition);
                }
                sql.push("(");
                for (i, c) in list.iter().enumerate() {
                    if i > 0 {
                        sql.push(" OR ");
                    }
                    c.sql(sql, field)?;
                }
                sql.push(")");
            }
            PgJson::Exists(path) => {
                sql.push(field)
                    .push(" #> '{")
                    .push(&join_path(path)?)
                    .push("}' IS NOT NULL");
            }
            PgJson::NotExists(path) => {
                sql.push(field)
                    .push(" #> '{")
                    .push(&join_path(path)?)
                    .push("}' IS NULL");
            }
        }
        Ok(())
    }
}

fn join_path(path: &[String]) -> Result<String, AppError> {
    for p in path {
        if !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(AppError::InvalidJsonPath(p.clone()));
        }
    }
    Ok(path.join(","))
}

// ======================= 宏 =======================

#[macro_export]
macro_rules! json_exists {
    ($($p:expr),+ $(,)?) => {
        $crate::PgJson::Exists(vec![$($p.to_string()),+])
    };
}

#[macro_export]
macro_rules! json_not_exists {
    ($($p:expr),+ $(,)?) => {
        $crate::PgJson::NotExists(vec![$($p.to_string()),+])
    };
}

#[macro_export]
macro_rules! json_and {
    ($($c:expr),+ $(,)?) => {
        $crate::PgJson::And(vec![$($c),+])
    };
}

#[macro_export]
macro_rules! json_or {
    ($($c:expr),+ $(,)?) => {
        $crate::PgJson::Or(vec![$($c),+])
    };
}

#[macro_export]
macro_rules! json_not {
    ($c:expr $(,)?) => {
        $crate::PgJson::Not(Box::new($c))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Postgres, QueryBuilder};

    fn build_sql(cond: &PgJson, field: &str) -> Result<String, AppError> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM t WHERE ");
        cond.sql(&mut qb, field)?;
        Ok(qb.sql().to_string())
    }

    #[test]
    fn exists_path() -> Result<(), AppError> {
        // let cond = PgJson::Exists(vec!["user".into(), "name".into()]);
        let cond = json_exists!("user", "name");

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE content #> '{user,name}' IS NOT NULL"
        );
        Ok(())
    }

    #[test]
    fn not_exists_path() -> Result<(), AppError> {
        // let cond = PgJson::NotExists(vec!["meta".into(), "deleted".into()]);
        let cond = json_not_exists!("meta", "deleted");

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE content #> '{meta,deleted}' IS NULL"
        );
        Ok(())
    }

    #[test]
    fn and_condition() -> Result<(), AppError> {
        // let cond = PgJson::And(vec![
        //     PgJson::Exists(vec!["a".into()]),
        //     PgJson::NotExists(vec!["b".into()]),
        // ]);
        let cond = json_and!(
            json_exists!("a"),
            json_not_exists!("b"),
        );

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE (content #> '{a}' IS NOT NULL AND content #> '{b}' IS NULL)"
        );
        Ok(())
    }

    #[test]
    fn or_condition() -> Result<(), AppError> {
        // let cond = PgJson::Or(vec![
        //     PgJson::Exists(vec!["x".into()]),
        //     PgJson::Exists(vec!["y".into()]),
        // ]);
        let cond = json_or!(
            json_exists!("x"),
            json_exists!("y"),
        );

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE (content #> '{x}' IS NOT NULL OR content #> '{y}' IS NOT NULL)"
        );
        Ok(())
    }

    #[test]
    fn not_condition() -> Result<(), AppError> {
        // let cond = PgJson::Not(Box::new(
        //     PgJson::Exists(vec!["secret".into()])
        // ));
        let cond = json_not!(json_exists!("secret"));

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE NOT (content #> '{secret}' IS NOT NULL)"
        );
        Ok(())
    }

    #[test]
    fn nested_condition() -> Result<(), AppError> {
        // let cond = PgJson::And(vec![
        //     PgJson::Exists(vec!["a".into()]),
        //     PgJson::Not(Box::new(
        //         PgJson::Or(vec![
        //             PgJson::Exists(vec!["b".into()]),
        //             PgJson::Exists(vec!["c".into()]),
        //         ])
        //     )),
        // ]);
        let cond = json_and!(
            json_exists!("a"),
            json_not!(
                json_or!(
                    json_exists!("b"),
                    json_exists!("c"),
                )
            )
        );

        let sql = build_sql(&cond, "content")?;

        assert_eq!(
            sql,
            "SELECT * FROM t WHERE (content #> '{a}' IS NOT NULL AND NOT ((content #> '{b}' IS NOT NULL OR content #> '{c}' IS NOT NULL)))"
        );
        Ok(())
    }

    #[test]
    fn empty_and_should_error() {
        let cond = PgJson::And(vec![]);
        // let cond = json_and!(); // 使用宏，在编译期就会报错

        let err = build_sql(&cond, "content").unwrap_err();

        assert!(matches!(err, AppError::EmptyJsonCondition));
    }

    #[test]
    fn invalid_path_should_error() {
        // let cond = PgJson::Exists(vec!["a".into(), "bad-key".into()]);
        let cond = json_exists!("a", "bad-key");

        let err = build_sql(&cond, "content").unwrap_err();

        assert!(matches!(err, AppError::InvalidJsonPath(_)));
    }
}