use crate::application::ApplicationError;

/// 将 SQLite/sqlx 错误映射为应用层错误语义。
pub fn map_sqlx_error(err: sqlx::Error, op: &str) -> ApplicationError {
    // `op` 用于构造可定位的错误文案，例如 "insert quote" / "delete quote"。
    match err {
        sqlx::Error::Database(db_err) => {
            let code = db_err
                .code()
                .map(|code| code.into_owned())
                .unwrap_or_else(|| "unknown".to_string());
            let message = db_err.message();
            map_sqlite_db_error(op, &code, message)
        }
        sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::Protocol(_) => ApplicationError::Dependency(format!("{op} failed: {err}")),
        _ => ApplicationError::Dependency(format!("{op} failed: {err}")),
    }
}

pub fn map_sqlite_db_error(op: &str, code: &str, message: &str) -> ApplicationError {
    // SQLite 常见约束码归类：
    // - 冲突类：主键/唯一约束
    // - 输入类：通用约束、外键、非空等
    match code {
        "1555" | "2067" | "2579" => {
            ApplicationError::Conflict(format!("{op} failed ({code}): {message}"))
        }
        "19" | "275" | "787" | "1299" | "1811" | "1043" | "2323" | "2835" | "3091" => {
            ApplicationError::InvalidInput(format!("{op} failed ({code}): {message}"))
        }
        _ => {
            // 某些驱动只给出模糊错误码，但 message 中会出现 constraint 关键字。
            if message.to_ascii_lowercase().contains("constraint") {
                ApplicationError::InvalidInput(format!("{op} failed ({code}): {message}"))
            } else {
                ApplicationError::Dependency(format!("{op} failed ({code}): {message}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::map_sqlite_db_error;
    use crate::application::ApplicationError;

    #[test]
    fn sqlite_constraint_codes_are_mapped_to_user_facing_errors() {
        let conflict = map_sqlite_db_error("op", "2067", "UNIQUE constraint failed");
        assert!(matches!(conflict, ApplicationError::Conflict(_)));

        let invalid_input = map_sqlite_db_error("op", "787", "FOREIGN KEY constraint failed");
        assert!(matches!(invalid_input, ApplicationError::InvalidInput(_)));

        let generic = map_sqlite_db_error("op", "19", "constraint failed");
        assert!(matches!(generic, ApplicationError::InvalidInput(_)));
    }
}
