use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Creator {
    System,
    User,
}
impl Creator {
    pub fn is_system(self) -> bool {
        matches!(self, Creator::System)
    }

    pub fn is_user(self) -> bool {
        matches!(self, Creator::User)
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DictType {
    pub type_id: i64,
    pub type_key: String,
    pub type_name: Option<String>,
    pub type_active: bool,
    pub type_creator: String,
    pub type_remark: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Dict {
    pub type_id: i64,
    pub type_key: String,
    pub type_name: Option<String>,
    pub type_active: bool,
    pub type_creator: String,
    pub type_remark: Option<String>,

    pub item_id: i64,
    pub item_key: String,
    pub item_value: Option<String>,
    pub is_default: bool,
    pub item_active: bool,
    pub item_creator: String,
    pub item_remark: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct DictQuery {
    type_id: Option<i64>,
    type_key: Option<String>,
    type_creator: Option<String>,
    type_active: Option<bool>,

    item_id: Option<i64>,
    item_key: Option<String>,
    item_creator: Option<String>,
    item_active: Option<bool>,

    is_default: Option<bool>,

    limit: Option<i64>,
    offset: Option<i64>,

    // 传给 f_dict(langs) 的参数
    langs: Option<Vec<String>>,
}

impl DictQuery {
    pub fn builder() -> DictQueryBuilder {
        DictQueryBuilder::default()
    }

    pub fn type_id(&self) -> Option<i64> {
        self.type_id
    }

    pub fn type_key(&self) -> Option<&str> {
        self.type_key.as_deref()
    }

    pub fn type_creator(&self) -> Option<&str> {
        self.type_creator.as_deref()
    }

    pub fn type_active(&self) -> Option<bool> {
        self.type_active
    }

    pub fn item_id(&self) -> Option<i64> {
        self.item_id
    }

    pub fn item_key(&self) -> Option<&str> {
        self.item_key.as_deref()
    }

    pub fn item_creator(&self) -> Option<&str> {
        self.item_creator.as_deref()
    }

    pub fn item_active(&self) -> Option<bool> {
        self.item_active
    }

    pub fn is_default(&self) -> Option<bool> {
        self.is_default
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    pub fn offset(&self) -> Option<i64> {
        self.offset
    }

    pub fn langs(&self) -> Option<&[String]> {
        self.langs.as_deref()
    }
}

#[derive(Default)]
pub struct DictQueryBuilder {
    inner: DictQuery,
}

impl DictQueryBuilder {
    pub fn type_id(mut self, type_id: i64) -> Self {
        self.inner.type_id = Some(type_id);
        self
    }

    pub fn with_type_id(mut self, type_id: Option<i64>) -> Self {
        self.inner.type_id = type_id;
        self
    }

    pub fn type_key(mut self, type_key: impl Into<String>) -> Self {
        self.inner.type_key = Some(type_key.into());
        self
    }

    pub fn with_type_key(mut self, type_key: Option<String>) -> Self {
        self.inner.type_key = type_key;
        self
    }

    pub fn type_creator(mut self, type_creator: impl Into<String>) -> Self {
        self.inner.type_creator = Some(type_creator.into());
        self
    }

    pub fn with_type_creator(mut self, type_creator: Option<String>) -> Self {
        self.inner.type_creator = type_creator;
        self
    }

    pub fn type_active(mut self, type_active: bool) -> Self {
        self.inner.type_active = Some(type_active);
        self
    }

    pub fn with_type_active(mut self, type_active: Option<bool>) -> Self {
        self.inner.type_active = type_active;
        self
    }

    pub fn item_id(mut self, item_id: i64) -> Self {
        self.inner.item_id = Some(item_id);
        self
    }

    pub fn with_item_id(mut self, item_id: Option<i64>) -> Self {
        self.inner.item_id = item_id;
        self
    }

    pub fn item_key(mut self, item_key: impl Into<String>) -> Self {
        self.inner.item_key = Some(item_key.into());
        self
    }

    pub fn with_item_key(mut self, item_key: Option<String>) -> Self {
        self.inner.item_key = item_key;
        self
    }

    pub fn item_creator(mut self, item_creator: impl Into<String>) -> Self {
        self.inner.item_creator = Some(item_creator.into());
        self
    }

    pub fn with_item_creator(mut self, item_creator: Option<String>) -> Self {
        self.inner.item_creator = item_creator;
        self
    }

    pub fn item_active(mut self, item_active: bool) -> Self {
        self.inner.item_active = Some(item_active);
        self
    }

    pub fn with_item_active(mut self, item_active: Option<bool>) -> Self {
        self.inner.item_active = item_active;
        self
    }

    pub fn is_default(mut self, is_default: bool) -> Self {
        self.inner.is_default = Some(is_default);
        self
    }

    pub fn with_is_default(mut self, is_default: Option<bool>) -> Self {
        self.inner.is_default = is_default;
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.inner.limit = Some(limit);
        self
    }

    pub fn with_limit(mut self, limit: Option<i64>) -> Self {
        self.inner.limit = limit;
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.inner.offset = Some(offset);
        self
    }

    pub fn with_offset(mut self, offset: Option<i64>) -> Self {
        self.inner.offset = offset;
        self
    }

    pub fn langs(mut self, langs: Vec<String>) -> Self {
        self.inner.langs = Some(langs);
        self
    }

    pub fn with_langs(mut self, langs: Option<Vec<String>>) -> Self {
        self.inner.langs = langs;
        self
    }

    pub fn build(self) -> DictQuery {
        self.inner
    }
}
