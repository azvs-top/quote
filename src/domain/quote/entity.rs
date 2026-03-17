use super::{MultiLangObject, MultiLangText, Quote, QuoteDraft, QuotePatch};
use crate::domain::DomainError;
use crate::domain::value::{Lang, ObjectKey};
use std::collections::HashMap;

impl Quote {
    /// 用完整字段构造一个已持久化的 `Quote`。
    ///
    /// 适用于：
    /// - 仓储层从数据库记录恢复领域实体
    /// - 领域内部在已知 `id` 的前提下重建聚合
    ///
    /// 会校验：
    /// - `id > 0`
    /// - 内容字段满足 Quote 不变量
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

        Self::validate_parts(&inline, &external, &markdown, &image)?;

        Ok(Self {
            id,
            inline,
            external,
            markdown,
            image,
            remark,
        })
    }

    /// 基于草稿和持久化 id 构造 `Quote`。
    ///
    /// 适用于创建成功后，仓储层将数据库分配的 id 与领域草稿合并。
    pub fn from_draft(id: i64, draft: QuoteDraft) -> Result<Self, DomainError> {
        Self::new(
            id,
            draft.inline,
            draft.external,
            draft.markdown,
            draft.image,
            draft.remark,
        )
    }

    pub(super) fn validate_parts(
        inline: &MultiLangText,
        external: &MultiLangObject,
        markdown: &MultiLangObject,
        image: &[ObjectKey],
    ) -> Result<(), DomainError> {
        Self::validate_inline_text_map(inline)?;
        Self::validate_lang_map(inline)?;
        Self::validate_lang_map(external)?;
        Self::validate_lang_map(markdown)?;
        Self::validate_image_keys(image)?;

        if inline.is_empty() && external.is_empty() && markdown.is_empty() && image.is_empty() {
            return Err(DomainError::QuoteMissingContent);
        }

        Ok(())
    }

    /// 判断当前 Quote 是否仍包含任意一种内容。
    ///
    /// 这是一个领域语义判断，不区分内容来自 inline、对象存储还是图片。
    pub fn has_content(&self) -> bool {
        !self.inline.is_empty()
            || !self.external.is_empty()
            || !self.markdown.is_empty()
            || !self.image.is_empty()
    }

    /// 返回 Quote 的持久化主键。
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 返回多语言内联文本。
    ///
    /// key 为语言，value 为对应语言下直接存储的文本内容。
    pub fn inline(&self) -> &MultiLangText {
        &self.inline
    }

    /// 返回多语言 external 对象引用。
    ///
    /// value 为对象存储中的 `ObjectKey`，通常指向纯文本附件。
    pub fn external(&self) -> &MultiLangObject {
        &self.external
    }

    /// 返回多语言 markdown 对象引用。
    pub fn markdown(&self) -> &MultiLangObject {
        &self.markdown
    }

    /// 返回图片对象引用列表。
    pub fn image(&self) -> &[ObjectKey] {
        &self.image
    }

    /// 返回备注文本。
    pub fn remark(&self) -> Option<&str> {
        self.remark.as_deref()
    }

    /// 按语言顺序提取 inline 文本。
    ///
    /// 常用于模板渲染、拼接输出等需要按指定语言顺序读取文本的场景。
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

    /// 将一个领域补丁应用到当前 Quote，返回更新后的新实体。
    ///
    /// 该方法会：
    /// - 先按补丁语义执行增量修改
    /// - 再统一校验更新后的 Quote 是否仍满足不变量
    ///
    /// 若更新结果非法，会返回领域错误而不是产生半合法状态。
    pub fn apply(&self, patch: QuotePatch) -> Result<Self, DomainError> {
        let mut inline = self.inline.clone();
        if patch.clear_inline {
            inline.clear();
        } else {
            for lang in patch.remove_inline {
                inline.remove(&lang);
            }
        }
        if let Some(values) = patch.upsert_inline {
            inline.extend(values);
        }

        let mut external = self.external.clone();
        if patch.clear_external {
            external.clear();
        } else {
            for lang in patch.remove_external {
                external.remove(&lang);
            }
        }
        if let Some(values) = patch.upsert_external {
            external.extend(values);
        }

        let mut markdown = self.markdown.clone();
        if patch.clear_markdown {
            markdown.clear();
        } else {
            for lang in patch.remove_markdown {
                markdown.remove(&lang);
            }
        }
        if let Some(values) = patch.upsert_markdown {
            markdown.extend(values);
        }

        let mut image = self.image.clone();
        if patch.clear_image {
            image.clear();
        } else if !patch.remove_image.is_empty() {
            let remove: std::collections::HashSet<&str> =
                patch.remove_image.iter().map(|key| key.as_str()).collect();
            image.retain(|key| !remove.contains(key.as_str()));
        }
        if let Some(values) = patch.append_image {
            image.extend(values);
        }

        let remark = patch.remark.unwrap_or_else(|| self.remark.clone());

        Self::new(self.id, inline, external, markdown, image, remark)
    }

    pub(super) fn validate_lang_map<T>(map: &HashMap<Lang, T>) -> Result<(), DomainError> {
        for key in map.keys() {
            if key.as_str().trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }

    pub(super) fn validate_inline_text_map(map: &MultiLangText) -> Result<(), DomainError> {
        for value in map.values() {
            if value.trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }

    pub(super) fn validate_image_keys(image: &[ObjectKey]) -> Result<(), DomainError> {
        for key in image {
            if key.as_str().trim().is_empty() {
                return Err(DomainError::QuoteInvalidContent);
            }
        }
        Ok(())
    }
}
