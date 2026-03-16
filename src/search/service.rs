use sqlx::PgPool;

use crate::search::model::{MissionSearchResult, StoreSearchResult};

// 통합 검색 서비스다.
#[derive(Clone)]
pub struct SearchService {
    pool: PgPool,
}

impl SearchService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search_by_mission_title(&self, keyword: &str) -> Vec<MissionSearchResult> {
        sqlx::query_as::<_, MissionSearchResult>(
            "SELECT
                MISSION_ID as mission_id,
                MISSION_TITLE as mission_title,
                STORE_ID as store_id
            FROM MISSION
            WHERE MISSION_TITLE ILIKE '%' || $1 || '%'
            ORDER BY MISSION_ID DESC",
        )
        .bind(keyword)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn search_by_store_name(&self, keyword: &str) -> Vec<StoreSearchResult> {
        sqlx::query_as::<_, StoreSearchResult>(
            "SELECT
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id
            FROM STORE_
            WHERE STORE_NAME ILIKE '%' || $1 || '%'
            ORDER BY STORE_ID DESC",
        )
        .bind(keyword)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn search_by_category(&self, category: &str) -> Vec<StoreSearchResult> {
        sqlx::query_as::<_, StoreSearchResult>(
            "SELECT
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id
            FROM STORE_
            WHERE STORE_CATEGORY = $1
            ORDER BY STORE_ID DESC",
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}
