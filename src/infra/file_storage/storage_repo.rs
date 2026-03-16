use crate::application::ApplicationError;
use crate::application::storage::{StoragePayload, StoragePort};
use crate::domain::value::ObjectKey;
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileStorageRepo {
    root: PathBuf,
}

impl FileStorageRepo {
    pub fn new(root: PathBuf) -> Result<Self, ApplicationError> {
        fs::create_dir_all(&root).map_err(|err| {
            ApplicationError::Dependency(format!(
                "create file storage root failed ({}): {err}",
                root.display()
            ))
        })?;
        Ok(Self { root })
    }

    fn key_to_path(&self, key: &ObjectKey) -> PathBuf {
        self.root.join(key.as_str())
    }

    fn build_object_key(
        path: &str,
        _filename: Option<&str>,
    ) -> Result<ObjectKey, ApplicationError> {
        let path = path.trim_matches('/');
        if path.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "storage path cannot be empty".to_string(),
            ));
        }

        ObjectKey::new(format!("{path}/{}", Uuid::new_v4())).map_err(ApplicationError::from)
    }
}

#[async_trait]
impl StoragePort for FileStorageRepo {
    async fn upload(
        &self,
        path: &str,
        payload: StoragePayload,
        _content_type: &str,
    ) -> Result<ObjectKey, ApplicationError> {
        let object_key = Self::build_object_key(path, payload.filename.as_deref())?;
        let file_path = self.key_to_path(&object_key);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                ApplicationError::Dependency(format!(
                    "create object parent directory failed ({}): {err}",
                    parent.display()
                ))
            })?;
        }

        fs::write(&file_path, payload.bytes).map_err(|err| {
            ApplicationError::Dependency(format!(
                "write object failed ({}): {err}",
                file_path.display()
            ))
        })?;

        Ok(object_key)
    }

    async fn delete(&self, key: &ObjectKey) -> Result<(), ApplicationError> {
        let file_path = self.key_to_path(key);
        match fs::remove_file(&file_path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(ApplicationError::Dependency(format!(
                "delete object failed ({}): {err}",
                file_path.display()
            ))),
        }
    }

    async fn exists(&self, key: &ObjectKey) -> Result<bool, ApplicationError> {
        Ok(self.key_to_path(key).is_file())
    }

    async fn download(&self, key: &ObjectKey) -> Result<Vec<u8>, ApplicationError> {
        let file_path = self.key_to_path(key);
        fs::read(&file_path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => {
                ApplicationError::NotFound(format!("object not found: {}", key.as_str()))
            }
            _ => ApplicationError::Dependency(format!(
                "read object failed ({}): {err}",
                file_path.display()
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FileStorageRepo;
    use crate::application::storage::{StoragePayload, StoragePort};
    use std::fs;

    #[tokio::test]
    async fn file_storage_repo_supports_upload_download_delete() {
        let root =
            std::env::temp_dir().join(format!("azvs-quote-storage-{}", uuid::Uuid::new_v4()));
        let repo = FileStorageRepo::new(root.clone()).expect("create repo");

        let key = repo
            .upload(
                "text/en",
                StoragePayload {
                    filename: Some("hello.txt".to_string()),
                    bytes: b"hello".to_vec(),
                },
                "text/plain; charset=utf-8",
            )
            .await
            .expect("upload object");

        assert!(repo.exists(&key).await.expect("exists"));
        assert_eq!(repo.download(&key).await.expect("download"), b"hello");
        assert!(!key.as_str().ends_with(".txt"));

        repo.delete(&key).await.expect("delete");
        assert!(!repo.exists(&key).await.expect("exists after delete"));

        let _ = fs::remove_dir_all(&root);
    }
}
