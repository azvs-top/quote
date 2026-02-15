use crate::domain::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// 对象存储 key 值对象。
///
/// 校验规则：
/// - 非空，长度不超过 1024。
/// - 不允许以 `/` 开头或结尾。
/// - 不允许包含 `..`、`\\`、`//`。
/// - 不允许包含控制字符。
///
/// 示例：`image/2026/02/abc.png`、`markdown/zh/123.md`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ObjectKey(String);

impl ObjectKey {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let value = raw.into();
        validate_object_key(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ObjectKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ObjectKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for ObjectKey {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectKey::new(s)
    }
}

impl TryFrom<String> for ObjectKey {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ObjectKey::new(value)
    }
}

impl From<ObjectKey> for String {
    fn from(value: ObjectKey) -> Self {
        value.0
    }
}

fn validate_object_key(value: &str) -> Result<(), DomainError> {
    if value.is_empty() || value.len() > 1024 {
        return Err(DomainError::InvalidObjectKey(value.to_string()));
    }

    if value.starts_with('/') || value.ends_with('/') {
        return Err(DomainError::InvalidObjectKey(value.to_string()));
    }

    if value.contains("..") || value.contains('\\') || value.contains("//") {
        return Err(DomainError::InvalidObjectKey(value.to_string()));
    }

    if value.chars().any(|ch| ch.is_control()) {
        return Err(DomainError::InvalidObjectKey(value.to_string()));
    }

    Ok(())
}
