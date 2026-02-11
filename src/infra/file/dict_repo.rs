use crate::app::app_error::AppError;
use crate::dict::{Dict, DictPort, DictQuery, DictType};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct DictRepoFile {
    path: PathBuf,
}

impl DictRepoFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl DictPort for DictRepoFile {
    async fn get_type(&self, query: DictQuery) -> Result<DictType, AppError> {
        todo!()
    }

    async fn list_type(&self, query: DictQuery) -> Result<Vec<DictType>, AppError> {
        todo!()
    }

    async fn list_item(&self, query: DictQuery) -> Result<Vec<Dict>, AppError> {
        todo!()
    }

    async fn get_item(&self, query: DictQuery) -> Result<Dict, AppError> {
        todo!()
    }
}
