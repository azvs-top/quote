use crate::application::ApplicationError;
use crate::application::storage::{StoragePayload, StoragePort};
use crate::domain::value::ObjectKey;

/// 上传单个对象的服务。
///
/// 功能：
/// - 校验上传入参（path/bytes/content_type）。
/// - 调用 `StoragePort::upload` 返回对象 key。
pub struct UploadObjectService<'a> {
    port: &'a (dyn StoragePort + Send + Sync),
}

impl<'a> UploadObjectService<'a> {
    pub fn new(port: &'a (dyn StoragePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        path: &str,
        payload: StoragePayload,
        content_type: &str,
    ) -> Result<ObjectKey, ApplicationError> {
        validate_upload_input(path, &payload, content_type)?;
        self.port.upload(path, payload, content_type).await
    }
}

pub(super) fn validate_upload_input(
    path: &str,
    payload: &StoragePayload,
    content_type: &str,
) -> Result<(), ApplicationError> {
    let normalized_path = path.trim();
    if normalized_path.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "path must not be empty".to_string(),
        ));
    }

    if normalized_path.starts_with('/')
        || normalized_path.ends_with('/')
        || normalized_path.contains('\\')
        || normalized_path.contains("..")
        || normalized_path.contains("//")
    {
        return Err(ApplicationError::InvalidInput(format!(
            "invalid path: {path}"
        )));
    }

    if payload.bytes.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "payload.bytes must not be empty".to_string(),
        ));
    }

    let mime = content_type.trim();
    if mime.is_empty() || !mime.contains('/') {
        return Err(ApplicationError::InvalidInput(format!(
            "invalid content_type: {content_type}"
        )));
    }

    Ok(())
}
