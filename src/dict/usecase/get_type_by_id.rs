use crate::app::app_error::AppError;
use crate::dict::{DictPort, DictQuery, DictType};

pub struct GetTypeById<'a> {
    port: &'a (dyn DictPort + Send + Sync),
}

impl<'a> GetTypeById<'a> {
    pub fn new(port: &'a (dyn DictPort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, id: i64) -> Result<DictType, AppError> {
        let query = DictQuery::builder()
            .type_id(id)
            .build();

        self.port
            .get_type(query)
            .await
            .map_err(|_| AppError::DictNotFound)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::app_error::AppError;
    use crate::dict::{DictType, GetTypeById, MockDictPort};

    #[tokio::test]
    async fn get_type_by_id_success() {
        let mut mock = MockDictPort::new();
        mock.expect_get_type().returning(|_| {
            Ok(DictType {
                type_id: 1,
                type_key: "status".to_string(),
                type_name: Some("Status".to_string()),
                type_active: true,
                type_creator: "system".to_string(),
                type_remark: Some("The state of the dictionary.".to_string()),
            })
        });

        let dict_type = GetTypeById::new(&mock).execute(1).await.unwrap();
        assert_eq!(dict_type.type_id, 1);
    }

    #[tokio::test]
    async fn get_type_by_id_failure() {
        let mut mock = MockDictPort::new();
        mock.expect_get_type()
            .returning(|_| Err(AppError::DictNotFound));

        let err = GetTypeById::new(&mock).execute(1).await.unwrap_err();
        assert!(matches!(err, AppError::DictNotFound));
    }
}
