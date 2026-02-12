use crate::app::app_error::AppError;
use crate::dict::{DictPort, DictQuery, DictType};

const DEFAULT_PAGE_SIZE: i64 = 10;
const DEFAULT_OFFSET: i64 = 0;

pub struct ListType<'a> {
    port: &'a (dyn DictPort + Send + Sync),
}

impl<'a> ListType<'a> {
    pub fn new(port: &'a (dyn DictPort + Send + Sync)) -> Self {
        Self { port }
    }

    /// 列出字典类型（支持 active 与分页）
    ///
    /// # Behavior
    /// - 默认分页：第一页 10 条（`limit=10`, `offset=0`）。
    /// - 若传入 `type_active`，仅返回匹配 active 状态的类型。
    /// - `limit<=0` 或 `offset<0` 时回退到默认分页值。
    pub async fn execute(&self, query: DictQuery) -> Result<Vec<DictType>, AppError> {
        let limit = match query.limit() {
            Some(limit) if limit > 0 => limit,
            _ => DEFAULT_PAGE_SIZE,
        };
        let offset = match query.offset() {
            Some(offset) if offset >= 0 => offset,
            _ => DEFAULT_OFFSET,
        };

        // list_type 用例只关注 active + 分页；默认返回所有 type。
        let new_query = DictQuery::builder()
            .with_type_active(query.type_active())
            .limit(limit)
            .offset(offset)
            .with_langs(query.langs().map(|langs| langs.to_vec()))
            .build();

        self.port.list_type(new_query).await
    }
}

#[cfg(test)]
mod tests {
    use crate::app::app_error::AppError;
    use crate::dict::{DictQuery, DictType, ListType, MockDictPort};

    #[tokio::test]
    async fn list_type_default_first_page() {
        let mut mock = MockDictPort::new();
        mock.expect_list_type().returning(|query| {
            assert_eq!(query.type_active(), None);
            assert_eq!(query.limit(), Some(10));
            assert_eq!(query.offset(), Some(0));
            Ok(vec![])
        });

        let items = ListType::new(&mock)
            .execute(DictQuery::builder().build())
            .await
            .unwrap();

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn list_type_with_active_and_pagination() {
        let mut mock = MockDictPort::new();
        mock.expect_list_type().returning(|query| {
            assert_eq!(query.type_active(), Some(true));
            assert_eq!(query.limit(), Some(20));
            assert_eq!(query.offset(), Some(40));
            Ok(vec![DictType {
                type_id: 1,
                type_key: "status".to_string(),
                type_name: Some("Status".to_string()),
                type_active: true,
                type_creator: "system".to_string(),
                type_remark: None,
            }])
        });

        let items = ListType::new(&mock)
            .execute(
                DictQuery::builder()
                    .type_active(true)
                    .limit(20)
                    .offset(40)
                    .build(),
            )
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].type_key, "status");
    }

    #[tokio::test]
    async fn list_type_invalid_pagination_fallback_to_default() {
        let mut mock = MockDictPort::new();
        mock.expect_list_type().returning(|query| {
            assert_eq!(query.limit(), Some(10));
            assert_eq!(query.offset(), Some(0));
            Err(AppError::DictNotFound)
        });

        let err = ListType::new(&mock)
            .execute(DictQuery::builder().limit(-1).offset(-1).build())
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::DictNotFound));
    }
}
