use crate::domain::DomainError;
use crate::domain::value::{Lang, ObjectKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MultiLangText = HashMap<Lang, String>;
pub type MultiLangObject = HashMap<Lang, ObjectKey>;

#[derive(Debug, Clone, Serialize)]
pub struct Quote {
    id: i64,
    inline: MultiLangText,
    external: MultiLangObject,
    markdown: MultiLangObject,
    image: Vec<ObjectKey>,
    remark: Option<String>,
}

impl Quote {
    pub fn new(
        id: i64,
        inline: MultiLangText,
        external: MultiLangObject,
        markdown: MultiLangObject,
        image: Vec<ObjectKey>,
        remark: Option<String>,
    ) -> Result<Self, DomainError> {
        if id <= 0 {
            return Err(DomainError::InvalidQuoteId(id));
        }

        Self::validate_inline_text_map(&inline)?;
        Self::validate_lang_map(&inline)?;
        Self::validate_lang_map(&external)?;
        Self::validate_lang_map(&markdown)?;
        Self::validate_image_keys(&image)?;

        if inline.is_empty() && external.is_empty() && markdown.is_empty() && image.is_empty() {
            return Err(DomainError::QuoteMissingContent);
        }

        Ok(Self {
            id,
            inline,
            external,
            markdown,
            image,
            remark,
        })
    }

    pub fn has_content(&self) -> bool {
        !self.inline.is_empty()
            || !self.external.is_empty()
            || !self.markdown.is_empty()
            || !self.image.is_empty()
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn inline(&self) -> &MultiLangText {
        &self.inline
    }

    pub fn external(&self) -> &MultiLangObject {
        &self.external
    }

    pub fn markdown(&self) -> &MultiLangObject {
        &self.markdown
    }

    pub fn image(&self) -> &[ObjectKey] {
        &self.image
    }

    pub fn remark(&self) -> Option<&str> {
        self.remark.as_deref()
    }

    pub fn get_inline_texts_by_langs(&self, langs: &[Lang]) -> Result<Vec<String>, DomainError> {
        if langs.is_empty() {
            return Err(DomainError::QuoteMissingContent);
        }

        let mut texts = Vec::with_capacity(langs.len());
        for lang in langs {
            let text = self
                .inline
                .get(lang)
                .ok_or(DomainError::QuoteMissingContent)?;
            if text.trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
            texts.push(text.clone());
        }

        Ok(texts)
    }

    fn validate_lang_map<T>(map: &HashMap<Lang, T>) -> Result<(), DomainError> {
        for key in map.keys() {
            if key.as_str().trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }

    fn validate_inline_text_map(map: &MultiLangText) -> Result<(), DomainError> {
        for value in map.values() {
            if value.trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }

    fn validate_image_keys(image: &[ObjectKey]) -> Result<(), DomainError> {
        for key in image {
            if key.as_str().trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }
}

/// 持久化/传输层使用的 Quote 数据结构。
///
/// 注意：
/// - 该类型允许反序列化。
/// - 转换为领域实体必须通过 `TryFrom<QuoteDto> for Quote`，以执行领域校验。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDto {
    pub id: i64,
    pub inline: MultiLangText,
    pub external: MultiLangObject,
    pub markdown: MultiLangObject,
    pub image: Vec<ObjectKey>,
    pub remark: Option<String>,
}

impl TryFrom<QuoteDto> for Quote {
    type Error = DomainError;

    fn try_from(value: QuoteDto) -> Result<Self, Self::Error> {
        Quote::new(
            value.id,
            value.inline,
            value.external,
            value.markdown,
            value.image,
            value.remark,
        )
    }
}

impl From<Quote> for QuoteDto {
    fn from(value: Quote) -> Self {
        Self {
            id: value.id,
            inline: value.inline,
            external: value.external,
            markdown: value.markdown,
            image: value.image,
            remark: value.remark,
        }
    }
}
