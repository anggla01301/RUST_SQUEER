use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::store::model::Store;

// 검색 결과에서 사용하는 미션 요약 DTO다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionSearchResult {
    pub mission_id: i64,
    pub mission_title: String,
    pub store_id: Option<i64>,
}

// 가게 검색 결과는 기존 Store 엔티티를 재사용한다.
pub type StoreSearchResult = Store;
