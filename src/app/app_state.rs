use crate::app::app_error::AppError;
use crate::dict::DictPort;
use crate::infra::{DictRepoFile, DictRepoPgsql, Minio, QuoteRepoFile, QuoteRepoPgsql};
use crate::quote::QuotePort;
use config::{Config, Environment, File};
use directories::BaseDirs;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub quote_port: Arc<dyn QuotePort + Send + Sync>,
    pub dict_port: Arc<dyn DictPort + Send + Sync>,

    pub config: Arc<AppConfig>,

    pub minio: Option<Arc<Minio>>,
}

impl AppState {
    pub async fn new() -> Result<Self, AppError> {
        // 加载配置文件并校验语义
        let config = AppConfig::load().await?;

        let (quote_port, dict_port): (
            Arc<dyn QuotePort + Send + Sync>,
            Arc<dyn DictPort + Send + Sync>,
        ) = match config.storage.backend {
            StorageBackend::Pgsql => {
                let pgsql = config.storage.pgsql.as_ref().unwrap();
                let pool = PgPoolOptions::new()
                    .max_connections(pgsql.max_connections.unwrap_or(10))
                    .min_connections(pgsql.min_connections.unwrap_or(0))
                    .connect(&pgsql.url)
                    .await?;
                (
                    Arc::new(QuoteRepoPgsql::new(pool.clone())),
                    Arc::new(DictRepoPgsql::new(pool)),
                )
            }
            StorageBackend::File => {
                let file = config.storage.file.as_ref().unwrap();
                let path = file.resolve_path()?;
                (
                    Arc::new(QuoteRepoFile::new(path.clone())),
                    Arc::new(DictRepoFile::new(path)),
                )
            }
        };

        let minio = match config.minio.as_ref() {
            Some(cfg) => Some(Arc::new(Minio::new(cfg).await?)),
            None => None,
        };

        Ok(Self {
            quote_port,
            dict_port,
            config: Arc::new(config),
            minio,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub storage: StorageConfig,
    pub minio: Option<MinioConfig>,
    pub quote: QuoteConfig,
}

impl AppConfig {
    pub async fn load() -> Result<Self, AppError> {
        // 获取配置文件路径
        let user_config_file = BaseDirs::new()
            .ok_or(AppError::ConfigDirNotFound)?
            .config_dir()
            .join("azvs")
            .join("quote.toml");

        // 允许全局变量覆盖配置文件中的变量
        let mut builder = Config::builder();
        if user_config_file.exists() {
            builder = builder.add_source(File::from(user_config_file));
        }
        builder = builder.add_source(Environment::default().separator("__"));

        let config: AppConfig = builder.build()?.try_deserialize()?;
        config.storage.validate_semantics()?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,

    #[serde(default)]
    pub pgsql: Option<PgsqlConfig>,
    #[serde(default)]
    pub file: Option<FileConfig>,
}

impl StorageConfig {
    // 校验配置文件中的storage模块
    // TODO: 这个检查语义的操作，是否能直接放在AppConfig中，或让AppConfig仅暴露一个方法
    pub fn validate_semantics(&self) -> Result<(), AppError> {
        match self.backend {
            StorageBackend::Pgsql => {
                if self.pgsql.is_none() {
                    return Err(AppError::MissingPgsqlConfig);
                }
            }
            StorageBackend::File => {
                let file = self.file.as_ref().ok_or(AppError::MissingFileConfig)?;

                // 验证 path 能否被解析（包括默认路径）
                let _ = file.resolve_path()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Pgsql,
    File,
}

#[derive(Debug, Deserialize)]
pub struct PgsqlConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default)]
    pub secure: bool,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

#[derive(Debug, Deserialize)]
pub struct QuoteConfig {
    #[serde(default = "default_inline_langs")]
    pub inline_langs: Vec<String>,
}

fn default_inline_langs() -> Vec<String> {
    vec!["en".to_string()]
}

impl FileConfig {
    // 如果backend选择了file但是没有配置path，给定默认值
    pub fn resolve_path(&self) -> Result<std::path::PathBuf, AppError> {
        if let Some(path) = &self.path {
            return Ok(path.into());
        }

        let base = BaseDirs::new()
            .ok_or(AppError::ConfigDirNotFound)?
            .config_dir()
            .join("azvs")
            .join("quote.json");

        Ok(base)
    }
}
