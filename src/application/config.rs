use crate::application::ApplicationError;
use config::{Config, Environment, File};
use directories::BaseDirs;
use serde::de::DeserializeOwned;
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