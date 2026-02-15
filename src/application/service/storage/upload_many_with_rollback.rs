use crate::application::storage::{StoragePayload, StoragePort};
use crate::application::ApplicationError;
use crate::domain::value::ObjectKey;
use super::upload_object::validate_upload_input;

/// 批量上传项。
#[derive(Debug, Clone)]
pub struct UploadObjectItem {
    pub path: String,
    pub payload: StoragePayload,
    pub content_type: String,
}

/// 批量上传并在失败时回滚已上传对象的服务。
pub struct UploadManyWithRollbackService<'a> {
    port: &'a (dyn StoragePort + Send + Sync),
}

impl<'a> UploadManyWithRollbackService<'a> {
    pub fn new(port: &'a (dyn StoragePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        items: Vec<UploadObjectItem>,
    ) -> Result<Vec<ObjectKey>, ApplicationError> {
        let mut uploaded = Vec::with_capacity(items.len());

        for (idx, item) in items.into_iter().enumerate() {
            validate_upload_input(&item.path, &item.payload, &item.content_type)?;

            match self
                .port
                .upload(&item.path, item.payload, &item.content_type)
                .await
            {
                Ok(key) => uploaded.push(key),
                Err(upload_err) => {
                    let rollback_errors = self.rollback_uploaded(&uploaded).await;
                    if rollback_errors.is_empty() {
                        return Err(upload_err);
                    }

                    return Err(ApplicationError::Dependency(format!(
                        "upload failed at item {idx}: {upload_err}; rollback failed: {}",
                        rollback_errors.join("; ")
                    )));
                }
            }
        }

        Ok(uploaded)
    }

    async fn rollback_uploaded(&self, uploaded: &[ObjectKey]) -> Vec<String> {
        let mut errors = Vec::new();
        for key in uploaded.iter().rev() {
            if let Err(err) = self.port.delete(key).await {
                // 回滚删除采用幂等语义：对象不存在不算失败。
                if matches!(err, ApplicationError::NotFound(_)) {
                    continue;
                }
                errors.push(format!("{} => {}", key.as_str(), err));
            }
        }
        errors
    }
}
