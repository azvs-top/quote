use crate::application::config::{ApplicationConfig, DatabaseBackend, StorageBackend};
use crate::application::quote::QuotePort;
use crate::application::storage::StoragePort;
use crate::application::ApplicationError;
use crate::infra::{MinioStorageRepo, NoneStorageRepo, PostgresQuoteRepo};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApplicationState {
    pub quote_port: Arc<dyn QuotePort + Send + Sync>,
    pub storage_port: Arc<dyn StoragePort + Send + Sync>,
    pub config: ApplicationConfig,
}

impl ApplicationState {
    /// 以已注入端口与已加载配置构建应用状态。
    fn from_parts(
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

    /// 根据配置自动装配应用状态（默认入口）。
    ///
    /// 当前支持：
    /// - 数据库：`postgres`
    /// - 存储：`none` / `minio`
    pub async fn new() -> Result<Self, ApplicationError> {
        let config = ApplicationConfig::load()?;

        let quote_port: Arc<dyn QuotePort + Send + Sync> = match config.database.backend {
            DatabaseBackend::Postgres => {
                let pg = config.database.postgres.as_ref().ok_or_else(|| {
                    ApplicationError::InvalidInput(
                        "database.backend=postgres requires [database.postgres]".to_string(),
                    )
                })?;

                let pool = PgPoolOptions::new()
                    .max_connections(pg.max_connections.unwrap_or(10))
                    .min_connections(pg.min_connections.unwrap_or(0))
                    .connect(&pg.url)
                    .await
                    .map_err(|err| {
                        ApplicationError::Dependency(format!("connect postgres failed: {err}"))
                    })?;

                Arc::new(PostgresQuoteRepo::new(pool))
            }
            DatabaseBackend::Mysql => {
                return Err(ApplicationError::InvalidInput(
                    "database.backend=mysql is not implemented yet".to_string(),
                ));
            }
        };

        let storage_port: Arc<dyn StoragePort + Send + Sync> = match config.storage.backend {
            StorageBackend::None => Arc::new(NoneStorageRepo::new()),
            StorageBackend::Minio => {
                let minio = config.storage.minio.as_ref().ok_or_else(|| {
                    ApplicationError::InvalidInput(
                        "storage.backend=minio requires [storage.minio]".to_string(),
                    )
                })?;
                Arc::new(MinioStorageRepo::new(minio).await?)
            }
            StorageBackend::File => {
                return Err(ApplicationError::InvalidInput(
                    "storage.backend=file is not implemented yet".to_string(),
                ));
            }
        };

        Ok(Self::from_parts(quote_port, storage_port, config))
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
        Ok(Self::from_parts(
            Arc::new(quote_port),
            Arc::new(storage_port),
            config,
        ))
    }
}
