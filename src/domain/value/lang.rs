use crate::domain::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// 语言编码值对象。
///
/// 校验规则：
/// - 非空，长度不超过 16。
/// - 仅允许 `a-z`、`0-9`、`-`。
/// - 不允许以 `-` 开头或结尾。
/// - 不允许出现连续 `--`。
///
/// 示例：`en`、`zh`、`pt-br`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Lang(String);

impl Lang {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let value = raw.into();
        validate_lang(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Lang {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Lang {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Lang::new(s)
    }
}

impl TryFrom<String> for Lang {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Lang::new(value)
    }
}

impl From<Lang> for String {
    fn from(value: Lang) -> Self {
        value.0
    }
}

fn validate_lang(value: &str) -> Result<(), DomainError> {
    if value.is_empty() || value.len() > 16 {
        return Err(DomainError::InvalidLang(value.to_string()));
    }

    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(DomainError::InvalidLang(value.to_string()));
    }

    if value.starts_with('-') || value.ends_with('-') || value.contains("--") {
        return Err(DomainError::InvalidLang(value.to_string()));
    }

    Ok(())
}
