use crate::application::ApplicationError;
use config::{Config, Environment, File};
use directories::BaseDirs;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_ENV_KEY: &str = "AZVS_QUOTE_CONFIG";

/// 解析配置文件路径。
///
/// 路径优先级：
/// 1. 环境变量 `AZVS_QUOTE_CONFIG`（显式指定）
/// 2. 用户配置目录下的默认路径：
///    - Linux: `~/.config/azvs/quote.toml`
///    - macOS: `~/Library/Application Support/azvs/quote.toml`
///    - Windows: `%APPDATA%\\azvs\\quote.toml`
pub fn resolve_config_file() -> Result<PathBuf, ApplicationError> {
    if let Ok(path) = std::env::var(CONFIG_ENV_KEY) {
        let path = path.trim();
        if path.is_empty() {
            return Err(ApplicationError::InvalidInput(format!(
                "{CONFIG_ENV_KEY} is empty"
            )));
        }
        return Ok(PathBuf::from(path));
    }

    let path = BaseDirs::new()
        .ok_or(ApplicationError::ConfigDirNotFound)?
        .config_dir()
        .join("azvs")
        .join("quote.toml");
    Ok(path)
}

/// 从配置文件 + 环境变量加载配置结构体。
///
/// - 如果配置文件不存在，将只使用环境变量。
/// - 环境变量使用 `__` 作为层级分隔符。
pub fn load_config<T>() -> Result<T, ApplicationError>
where
    T: DeserializeOwned,
{
    let config_file = resolve_config_file()?;
    let mut builder = Config::builder();

    if config_file.exists() {
        builder = builder.add_source(File::from(config_file));
    }

    builder = builder.add_source(Environment::default().separator("__"));
    let settings = builder.build()?;
    Ok(settings.try_deserialize()?)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApplicationConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub cli: CliConfig,
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CliConfig {
    #[serde(default)]
    pub format: CliFormatConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CliFormatConfig {
    pub default_get: Option<String>,
    pub default_list: Option<String>,
    pub image_mode: CliImageMode,
    pub presets: HashMap<String, String>,
}

impl Default for CliFormatConfig {
    fn default() -> Self {
        Self {
            default_get: None,
            default_list: None,
            image_mode: CliImageMode::Meta,
            presets: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CliImageMode {
    #[default]
    Meta,
    Ascii,
    View,
}
