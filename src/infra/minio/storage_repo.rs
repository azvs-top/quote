use crate::application::storage::{StoragePayload, StoragePort};
use crate::application::{ApplicationError, MinioConfig};
use crate::domain::value::ObjectKey;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use uuid::Uuid;

pub struct MinioStorageRepo {
    client: Client,
    bucket: String,
}

impl MinioStorageRepo {
    pub async fn new(cfg: &MinioConfig) -> Result<Self, ApplicationError> {
        let credentials = Credentials::new(
            cfg.access_key.clone(),
            cfg.secret_key.clone(),
            None,
            None,
            "static",
        );
        let region = Region::new(cfg.region.clone());

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config =
            Config::builder()
                .behavior_version(BehaviorVersion::latest())
                .endpoint_url(cfg.endpoint.clone())
                .credentials_provider(shared_config.credentials_provider().ok_or_else(|| {
                    ApplicationError::Dependency("missing aws credentials provider".to_string())
                })?)
                .region(shared_config.region().cloned().ok_or_else(|| {
                    ApplicationError::Dependency("missing aws region".to_string())
                })?)
                .force_path_style(true)
                .build();

        Ok(Self {
            client: Client::from_conf(s3_config),
            bucket: cfg.bucket.clone(),
        })
    }

    /// 将底层 SDK 错误包装为统一依赖错误，附带上下文说明。
    fn map_dependency_error(context: &str, err: impl std::fmt::Display) -> ApplicationError {
        ApplicationError::Dependency(format!("{context}: {err}"))
    }

    /// 判断 S3/MinIO 错误是否属于“对象不存在”。
    fn is_not_found<E, R>(err: &aws_sdk_s3::error::SdkError<E, R>) -> bool
    where
        E: ProvideErrorMetadata,
    {
        err.as_service_error()
            .and_then(|e| e.code())
            .map(|code| code.eq_ignore_ascii_case("notfound") || code == "NoSuchKey")
            .unwrap_or(false)
    }

    /// 生成对象 key：`{path}/{uuid}`。
    fn build_object_key(
        path: &str,
        _filename: Option<&str>,
    ) -> Result<ObjectKey, ApplicationError> {
        let path = path.trim_matches('/');
        let key = format!("{path}/{}", Uuid::new_v4());
        ObjectKey::new(key).map_err(ApplicationError::from)
    }
}

#[async_trait]
impl StoragePort for MinioStorageRepo {
    async fn upload(
        &self,
        path: &str,
        payload: StoragePayload,
        content_type: &str,
    ) -> Result<ObjectKey, ApplicationError> {
        let object_key = Self::build_object_key(path, payload.filename.as_deref())?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key.as_str())
            .body(ByteStream::from(payload.bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|err| Self::map_dependency_error("minio put_object failed", err))?;

        Ok(object_key)
    }

    async fn delete(&self, key: &ObjectKey) -> Result<(), ApplicationError> {
        let result = self
            .client
            .delete_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) if Self::is_not_found(&err) => Ok(()),
            Err(err) => Err(Self::map_dependency_error(
                "minio delete_object failed",
                err,
            )),
        }
    }

    async fn exists(&self, key: &ObjectKey) -> Result<bool, ApplicationError> {
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(err) if Self::is_not_found(&err) => Ok(false),
            Err(err) => Err(Self::map_dependency_error("minio head_object failed", err)),
        }
    }

    async fn download(&self, key: &ObjectKey) -> Result<Vec<u8>, ApplicationError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .send()
            .await
            .map_err(|err| {
                if Self::is_not_found(&err) {
                    ApplicationError::NotFound(format!("object not found: {}", key.as_str()))
                } else {
                    Self::map_dependency_error("minio get_object failed", err)
                }
            })?;

        let data = output
            .body
            .collect()
            .await
            .map_err(|err| Self::map_dependency_error("minio stream collect failed", err))?;
        Ok(data.into_bytes().to_vec())
    }
}
