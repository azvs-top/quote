use crate::application::ApplicationError;
use crate::application::storage::{StoragePayload, StoragePort};
use crate::domain::value::ObjectKey;
use async_trait::async_trait;

/// 未配置对象存储时使用的占位实现。
///
/// 行为：
/// - 允许应用在 `storage.backend=none` 时正常启动。
/// - 上传/下载返回明确错误，提示用户启用对象存储。
/// - 删除/存在性检查采用宽松语义，避免清理流程阻塞主业务。
#[derive(Debug, Default)]
pub struct NoneStorageRepo;

impl NoneStorageRepo {
    pub fn new() -> Self {
        Self
    }

    fn disabled_error(action: &str) -> ApplicationError {
        ApplicationError::Dependency(format!(
            "storage backend is disabled (storage.backend=none); cannot {action};"
        ))
    }
}

#[async_trait]
impl StoragePort for NoneStorageRepo {
    async fn upload(
        &self,
        _path: &str,
        _payload: StoragePayload,
        _content_type: &str,
    ) -> Result<ObjectKey, ApplicationError> {
        Err(Self::disabled_error("upload objects"))
    }

    async fn delete(&self, _key: &ObjectKey) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn exists(&self, _key: &ObjectKey) -> Result<bool, ApplicationError> {
        Ok(false)
    }

    async fn download(&self, _key: &ObjectKey) -> Result<Vec<u8>, ApplicationError> {
        Err(Self::disabled_error("download objects"))
    }
}
