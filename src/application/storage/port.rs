use crate::application::ApplicationError;
use crate::application::storage::StoragePayload;
use crate::domain::value::ObjectKey;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StoragePort {
    /// 上传对象并返回对象 key。
    ///
    /// # Parameters
    /// - `path`：对象前缀路径（如 `text/en`、`markdown/zh`、`image`），不能为空。
    /// - `payload.filename`：可选原始文件名，用于推断后缀或排障；可为 `None`。
    /// - `payload.bytes`：对象二进制内容；通常要求非空。
    /// - `content_type`：MIME 类型（如 `text/plain; charset=utf-8`、`image/png`），不能为空。
    ///
    /// # Returns
    /// - 返回已持久化的 `ObjectKey`，必须可用于后续 `exists/delete/download`。
    ///
    /// # Errors
    /// - 参数不合法、网络错误、鉴权失败或写入失败时返回实现层映射后的错误。
    async fn upload(
        &self,
        path: &str,
        payload: StoragePayload,
        content_type: &str,
    ) -> Result<ObjectKey, ApplicationError>;

    /// 删除指定对象。
    ///
    /// # Parameters
    /// - `key`：目标对象 key，必须是合法 `ObjectKey`。
    ///
    /// # Errors
    /// - 对象不存在、无权限或删除失败时返回实现层映射后的错误。
    async fn delete(&self, key: &ObjectKey) -> Result<(), ApplicationError>;

    /// 检查对象是否存在。
    ///
    /// # Parameters
    /// - `key`：目标对象 key，必须是合法 `ObjectKey`。
    ///
    /// # Returns
    /// - `true`：对象存在。
    /// - `false`：对象不存在。
    ///
    /// # Errors
    /// - 网络错误、鉴权失败等异常返回实现层映射后的错误。
    async fn exists(&self, key: &ObjectKey) -> Result<bool, ApplicationError>;

    /// 下载对象的完整二进制内容。
    ///
    /// # Parameters
    /// - `key`：目标对象 key，必须是合法 `ObjectKey`。
    ///
    /// # Returns
    /// - 对象字节数组。
    ///
    /// # Errors
    /// - 对象不存在、无权限、读取失败时返回实现层映射后的错误。
    async fn download(&self, key: &ObjectKey) -> Result<Vec<u8>, ApplicationError>;
}
