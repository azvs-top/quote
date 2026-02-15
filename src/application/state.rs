use crate::application::config::load_config;
use crate::application::quote::QuotePort;
use crate::application::storage::StoragePort;
use crate::application::ApplicationError;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApplicationConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

impl ApplicationConfig {
    /// 从配置文件和环境变量加载 `ApplicationConfig`，并执行语义校验。
    pub fn load() -> Result<Self, ApplicationError> {
        let config = load_config::<Self>()?;
        config.validate_semantics()?;
        Ok(config)
    }

    /// 校验配置项组合是否完整且自洽。
    /// 例如 backend 选项必须与对应子配置块同时出现。
    fn validate_semantics(&self) -> Result<(), ApplicationError> {
        self.database.validate_semantics()?;
        self.storage.validate_semantics()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    Postgres,
    Mysql,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        Self::Postgres
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub backend: DatabaseBackend,
    #[serde(default)]
    pub postgres: Option<PostgresConfig>,
    #[serde(default)]
    pub mysql: Option<MysqlConfig>,
}

impl DatabaseConfig {
    /// 校验数据库 backend 与子配置块的匹配关系。
    /// - `postgres` 需要 `[database.postgres]`
    /// - `mysql` 需要 `[database.mysql]`
    fn validate_semantics(&self) -> Result<(), ApplicationError> {
        match self.backend {
            DatabaseBackend::Postgres => {
                if self.postgres.is_none() {
                    return Err(ApplicationError::InvalidInput(
                        "database.backend=postgres requires [database.postgres]".to_string(),
                    ));
                }
            }
            DatabaseBackend::Mysql => {
                if self.mysql.is_none() {
                    return Err(ApplicationError::InvalidInput(
                        "database.backend=mysql requires [database.mysql]".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub min_connections: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MysqlConfig {
    pub url: String,
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub min_connections: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Minio,
    File,
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::File
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StorageConfig {
    #[serde(default)]
    pub backend: StorageBackend,
    #[serde(default)]
    pub minio: Option<MinioConfig>,
    #[serde(default)]
    pub file: Option<FileStorageConfig>,
}

impl StorageConfig {
    /// 校验存储 backend 与子配置块的匹配关系。
    /// - `minio` 需要 `[storage.minio]`
    /// - `file` 需要 `[storage.file]`
    fn validate_semantics(&self) -> Result<(), ApplicationError> {
        match self.backend {
            StorageBackend::Minio => {
                if self.minio.is_none() {
                    return Err(ApplicationError::InvalidInput(
                        "storage.backend=minio requires [storage.minio]".to_string(),
                    ));
                }
            }
            StorageBackend::File => {
                if self.file.is_none() {
                    return Err(ApplicationError::InvalidInput(
                        "storage.backend=file requires [storage.file]".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileStorageConfig {
    #[serde(default)]
    pub root: Option<String>,
}

#[derive(Clone)]
pub struct ApplicationState {
    pub quote_port: Arc<dyn QuotePort + Send + Sync>,
    pub storage_port: Arc<dyn StoragePort + Send + Sync>,
    pub config: ApplicationConfig,
}

impl ApplicationState {
    /// 以已注入端口与已加载配置构建应用状态。
    fn new(
        quote_port: Arc<dyn QuotePort + Send + Sync>,
        storage_port: Arc<dyn StoragePort + Send + Sync>,
        config: ApplicationConfig,
    ) -> Self {
        Self {
            quote_port,
            storage_port,
            config,
        }
    }

    /// 构建 `ApplicationState`，并完成配置加载。
    ///
    /// # Parameters
    /// - `quote_port`: `QuotePort` 的具体实现（如 pgsql | mysql）。
    /// - `storage_port`: `StoragePort` 的具体实现（如 minio | file）。
    ///
    /// # Behavior
    /// - 自动从 `application::config` 加载本地配置与环境变量覆盖。
    /// - 校验 `database.backend` 与 `storage.backend` 对应配置块是否存在。
    pub fn builder<Q, S>(quote_port: Q, storage_port: S) -> Result<Self, ApplicationError>
    where
        Q: QuotePort + Send + Sync + 'static,
        S: StoragePort + Send + Sync + 'static,
    {
        let config = ApplicationConfig::load()?;
        Ok(Self::new(
            Arc::new(quote_port),
            Arc::new(storage_port),
            config,
        ))
    }
}
