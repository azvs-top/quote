use crate::app::{AppError, MinioConfig};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

/// # Minio中的概念
/// + Object：存储的实际内容
/// + Bucket：资源组织和权限隔离
/// + Key：对象的唯一标识符
/// + Policy：权限控制，决定读写删
/// + Presigned URL: 安全地分享私有文件
pub struct Minio {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
}

impl Minio {
    pub async fn new(cfg: &MinioConfig) -> Result<Self, AppError> {
        let region = Region::new(cfg.region.clone());
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(&cfg.endpoint)
            .credentials_provider(Credentials::new(
                &cfg.access_key,
                &cfg.secret_key,
                None,
                None,
                "minio",
            ))
            .region(region)
            .load()
            .await;

        // NOTE: Virtual-hosted style -> bucket.endpoint/key
        // NOTE: Path-style -> endpoint/bucket/key (minio默认使用)
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true) // 启用 path-style
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        client
            .head_bucket()
            .bucket(&cfg.bucket)
            .send()
            .await
            .map_err(map_s3_error)?;

        Ok(Self {
            client,
            bucket: cfg.bucket.clone(),
        })
    }

    pub async fn get_text(&self, key: &str) -> Result<String, AppError> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = resp
            .body
            .collect()
            .await
            .map_err(|_| AppError::ExternalStorageError)?;
        let bytes = data.into_bytes();

        Ok(String::from_utf8(bytes.to_vec())?)
    }

    pub async fn put_text(&self, path: &str, txt: &str) -> Result<String, AppError> {
        // 随机生成文件名并
        let key = format!("{}/{}", path.trim_end_matches("/"), Uuid::new_v4());
        let body = ByteStream::from(txt.as_bytes().to_vec());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .content_type("text/plain; charset=utf-8")
            .send()
            .await
            .map_err(map_s3_error)?;

        Ok(key)
    }

    pub async fn put_text_file(&self, path: &str, body: ByteStream) -> Result<String, AppError> {
        let key = format!("{}/{}", path.trim_end_matches("/"), Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            // NOTE: content_type是我自称的类型，并无法保证类型正确。
            .content_type("text/plain; charset=utf-8")
            .send()
            .await
            .map_err(map_s3_error)?;

        Ok(key)
    }
}

fn map_s3_error<E>(err: SdkError<E>) -> AppError {
    match err {
        SdkError::ServiceError(service_err) => {
            let status = service_err.raw().status().as_u16();
            match status {
                403 => AppError::MinioAccessDenied,
                404 => AppError::MinioBucketNotFound,
                _ => AppError::ExternalStorageError,
            }
        }
        SdkError::TimeoutError(_) | SdkError::DispatchFailure(_) => AppError::MinioUnavailable,
        _ => AppError::ExternalStorageError,
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppState;

    #[tokio::test]
    async fn minio_read_text_demo() -> anyhow::Result<()> {
        let state = AppState::new().await?;
        let minio = state.minio.as_ref().expect("Minio is not configured.");
        let key = "test/ykkhds.txt";

        let content = minio.get_text(key).await?;
        println!("{}", content);

        Ok(())
    }
}
