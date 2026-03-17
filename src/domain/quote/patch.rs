use super::{MultiLangObject, MultiLangText, Quote, QuotePatch};
use crate::domain::DomainError;
use crate::domain::value::{Lang, ObjectKey};

impl QuotePatch {
    /// 构造一个增量更新补丁。
    ///
    /// 参数按“新增/覆盖、清空、删除、备注更新”分组，分别表达不同更新意图。
    /// 该构造函数只校验补丁本身的局部合法性，不校验应用后的整体结果。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        upsert_inline: Option<MultiLangText>,
        clear_inline: bool,
        remove_inline: Vec<Lang>,
        upsert_external: Option<MultiLangObject>,
        clear_external: bool,
        remove_external: Vec<Lang>,
        upsert_markdown: Option<MultiLangObject>,
        clear_markdown: bool,
        remove_markdown: Vec<Lang>,
        append_image: Option<Vec<ObjectKey>>,
        clear_image: bool,
        remove_image: Vec<ObjectKey>,
        remark: Option<Option<String>>,
    ) -> Result<Self, DomainError> {
        if let Some(ref value) = upsert_inline {
            Quote::validate_inline_text_map(value)?;
            Quote::validate_lang_map(value)?;
        }
        if let Some(ref value) = upsert_external {
            Quote::validate_lang_map(value)?;
        }
        if let Some(ref value) = upsert_markdown {
            Quote::validate_lang_map(value)?;
        }
        if let Some(ref value) = append_image {
            Quote::validate_image_keys(value)?;
        }
        if !remove_image.is_empty() {
            Quote::validate_image_keys(&remove_image)?;
        }

        Ok(Self {
            upsert_inline,
            clear_inline,
            remove_inline,
            upsert_external,
            clear_external,
            remove_external,
            upsert_markdown,
            clear_markdown,
            remove_markdown,
            append_image,
            clear_image,
            remove_image,
            remark,
        })
    }

    /// 返回需要新增或覆盖的 inline 文本。
    pub fn upsert_inline(&self) -> Option<&MultiLangText> {
        self.upsert_inline.as_ref()
    }

    /// 是否清空全部 inline 文本。
    pub fn clear_inline(&self) -> bool {
        self.clear_inline
    }

    /// 返回需要删除的 inline 语言列表。
    pub fn remove_inline(&self) -> &[Lang] {
        &self.remove_inline
    }

    /// 返回需要新增或覆盖的 external 对象引用。
    pub fn upsert_external(&self) -> Option<&MultiLangObject> {
        self.upsert_external.as_ref()
    }

    /// 是否清空全部 external 内容。
    pub fn clear_external(&self) -> bool {
        self.clear_external
    }

    /// 返回需要删除的 external 语言列表。
    pub fn remove_external(&self) -> &[Lang] {
        &self.remove_external
    }

    /// 返回需要新增或覆盖的 markdown 对象引用。
    pub fn upsert_markdown(&self) -> Option<&MultiLangObject> {
        self.upsert_markdown.as_ref()
    }

    /// 是否清空全部 markdown 内容。
    pub fn clear_markdown(&self) -> bool {
        self.clear_markdown
    }

    /// 返回需要删除的 markdown 语言列表。
    pub fn remove_markdown(&self) -> &[Lang] {
        &self.remove_markdown
    }

    /// 返回需要追加的图片对象。
    pub fn append_image(&self) -> Option<&[ObjectKey]> {
        self.append_image.as_deref()
    }

    /// 是否清空全部图片。
    pub fn clear_image(&self) -> bool {
        self.clear_image
    }

    /// 返回需要删除的图片对象列表。
    pub fn remove_image(&self) -> &[ObjectKey] {
        &self.remove_image
    }

    /// 返回备注更新语义。
    ///
    /// - `None`：不修改备注
    /// - `Some(None)`：清空备注
    /// - `Some(Some(v))`：更新为新值
    pub fn remark(&self) -> Option<Option<&str>> {
        self.remark.as_ref().map(|value| value.as_deref())
    }

    /// 判断该补丁是否不包含任何实际变更。
    pub fn is_empty(&self) -> bool {
        self.upsert_inline
            .as_ref()
            .map_or(true, std::collections::HashMap::is_empty)
            && !self.clear_inline
            && self.remove_inline.is_empty()
            && self
                .upsert_external
                .as_ref()
                .map_or(true, std::collections::HashMap::is_empty)
            && !self.clear_external
            && self.remove_external.is_empty()
            && self
                .upsert_markdown
                .as_ref()
                .map_or(true, std::collections::HashMap::is_empty)
            && !self.clear_markdown
            && self.remove_markdown.is_empty()
            && self.append_image.as_ref().map_or(true, Vec::is_empty)
            && !self.clear_image
            && self.remove_image.is_empty()
            && self.remark.is_none()
    }
}
