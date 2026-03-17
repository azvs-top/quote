use super::{MultiLangObject, MultiLangText, Quote, QuoteDraft};
use crate::domain::DomainError;
use crate::domain::value::ObjectKey;

impl QuoteDraft {
    /// 构造一个用于创建 Quote 的领域草稿。
    ///
    /// 与 `Quote` 的区别是：
    /// - 不包含持久化 id
    /// - 只表达“待创建内容是否合法”
    pub fn new(
        inline: MultiLangText,
        external: MultiLangObject,
        markdown: MultiLangObject,
        image: Vec<ObjectKey>,
        remark: Option<String>,
    ) -> Result<Self, DomainError> {
        Quote::validate_parts(&inline, &external, &markdown, &image)?;

        Ok(Self {
            inline,
            external,
            markdown,
            image,
            remark,
        })
    }

    /// 返回草稿中的 inline 文本内容。
    pub fn inline(&self) -> &MultiLangText {
        &self.inline
    }

    /// 返回草稿中的 external 对象引用。
    pub fn external(&self) -> &MultiLangObject {
        &self.external
    }

    /// 返回草稿中的 markdown 对象引用。
    pub fn markdown(&self) -> &MultiLangObject {
        &self.markdown
    }

    /// 返回草稿中的图片对象引用列表。
    pub fn image(&self) -> &[ObjectKey] {
        &self.image
    }

    /// 返回草稿中的备注。
    pub fn remark(&self) -> Option<&str> {
        self.remark.as_deref()
    }
}
