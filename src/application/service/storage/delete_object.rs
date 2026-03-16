use crate::application::ApplicationError;
use crate::application::storage::StoragePort;
use crate::domain::value::ObjectKey;

/// 删除单个对象的服务。
pub struct DeleteObjectService<'a> {
    port: &'a (dyn StoragePort + Send + Sync),
}

impl<'a> DeleteObjectService<'a> {
    pub fn new(port: &'a (dyn StoragePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, key: &ObjectKey) -> Result<(), ApplicationError> {
        match self.port.delete(key).await {
            Ok(()) => Ok(()),
            // 删除语义幂等：对象不存在视为已删除成功。
            Err(ApplicationError::NotFound(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
