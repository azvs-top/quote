use crate::app::app_error::AppError;
use crate::dict::{DictPort, DictQuery, DictType};

pub struct GetTypeByKey<'a> {
    port: &'a dyn DictPort,
}

impl<'a> GetTypeByKey<'a> {
    pub fn new(port: &'a dyn DictPort) -> Self {
        Self { port }
    }

    pub async fn execute(&self, key: impl Into<String>) -> Result<DictType, AppError> {
        let query = DictQuery::builder()
            .type_key(key)
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
    use crate::dict::{DictType, GetTypeByKey, MockDictPort};

    #[tokio::test]
    async fn get_type_by_key_success() {
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

        let dict_type = GetTypeByKey::new(&mock).execute("status").await.unwrap();
        assert_eq!(dict_type.type_key, "status");
    }

    #[tokio::test]
    async fn get_type_by_key_failure() {
        let mut mock = MockDictPort::new();
        mock.expect_get_type()
            .returning(|_| Err(AppError::DictNotFound));

        let err = GetTypeByKey::new(&mock).execute("missing").await.unwrap_err();
        assert!(matches!(err, AppError::DictNotFound));
    }
}
