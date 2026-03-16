use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// STORE_ 테이블과 매핑되는 매장 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Store {
    pub store_id: Option<i64>,
    pub store_name: String,
    pub store_address: Option<String>,
    pub store_category: String,
    pub store_latitude: f64,
    pub store_longitude: f64,
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoreRequestDto {
    pub store_name: String,
    pub store_category: String,
    pub store_address: Option<String>,
    pub store_latitude: f64,
    pub store_longitude: f64,
}

impl Store {
    // 점주 회원가입 또는 프로필 확정 단계에서 기본 매장 레코드를 만든다.
    pub fn new(
        store_name: String,
        store_category: String,
        store_latitude: f64,
        store_longitude: f64,
        user_id: i64,
    ) -> Self {
        Self {
            store_id: None,
            store_name,
            store_address: None,
            store_category,
            store_latitude,
            store_longitude,
            user_id,
        }
    }
}
