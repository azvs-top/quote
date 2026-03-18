use crate::application::ApplicationError;

/// 将 Postgres/sqlx 错误映射为应用层错误语义。
///
/// 该模块负责把数据库和连接池层面的异常转换为：
/// - 资源冲突
/// - 输入不合法
/// - 依赖故障
pub fn map_sqlx_error(err: sqlx::Error, op: &str) -> ApplicationError {
    // `op` 用于构造可定位的错误文案，例如 "insert quote" / "update quote"。
    match err {
        sqlx::Error::Database(db_err) => {
            let code = db_err
                .code()
                .map(|code| code.into_owned())
                .unwrap_or_else(|| "unknown".to_string());
            let message = db_err.message();
            // 这里按 SQLSTATE 归类，不绑定具体 SQL 文本。
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
        | sqlx::Error::Protocol(_) => ApplicationError::Dependency(format!("{op} failed: {err}")),
        _ => ApplicationError::Dependency(format!("{op} failed: {err}")),
    }
}
