use sqlx::PgPool;

use crate::store::model::{Store, StoreRequestDto};

// 매장 도메인의 DB 접근과 서비스 로직을 함께 담당한다.

#[derive(Clone)]
pub struct StoreRepository {
    pool: PgPool,
}

impl StoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // INSERT 후 RETURNING으로 DB가 만든 STORE_ID까지 다시 받는다.
    pub async fn save(&self, store: &Store) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "INSERT INTO STORE_ (
                STORE_NAME, STORE_ADDRESS, STORE_CATEGORY, STORE_LATITUDE, STORE_LONGITUDE, USER_ID
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id",
        )
        .bind(&store.store_name)
        .bind(&store.store_address)
        .bind(&store.store_category)
        .bind(store.store_latitude)
        .bind(store.store_longitude)
        .bind(store.user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    // 스프링의 findByUser_UserId 와 같은 의미다.
    pub async fn find_by_user_id(&self, user_id: i64) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "SELECT
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id
            FROM STORE_
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn find_by_id(&self, store_id: i64) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "SELECT
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id
            FROM STORE_
            WHERE STORE_ID = $1",
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn find_by_category(&self, category: &str) -> Vec<Store> {
        sqlx::query_as::<_, Store>(
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

    // WHERE 절에 user_id를 함께 넣어 소유권 검사를 SQL에서도 수행한다.
    pub async fn update(&self, store_id: i64, user_id: i64, updated: &Store) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "UPDATE STORE_
            SET STORE_NAME = $1,
                STORE_CATEGORY = $2,
                STORE_ADDRESS = $3,
                STORE_LATITUDE = $4,
                STORE_LONGITUDE = $5
            WHERE STORE_ID = $6 AND USER_ID = $7
            RETURNING
                STORE_ID as store_id,
                STORE_NAME as store_name,
                STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category,
                STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude,
                USER_ID as user_id",
        )
        .bind(&updated.store_name)
        .bind(&updated.store_category)
        .bind(&updated.store_address)
        .bind(updated.store_latitude)
        .bind(updated.store_longitude)
        .bind(store_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn delete(&self, store_id: i64, user_id: i64) -> bool {
        sqlx::query("DELETE FROM STORE_ WHERE STORE_ID = $1 AND USER_ID = $2")
            .bind(store_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct StoreService {
    repository: StoreRepository,
}

impl StoreService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: StoreRepository::new(pool),
        }
    }

    // 컨트롤러 DTO를 DB 엔티티 형태로 변환한 뒤 저장한다.
    pub async fn create_store(&self, user_id: i64, dto: StoreRequestDto) -> Option<Store> {
        self.repository
            .save(&Store {
                store_id: None,
                store_name: dto.store_name,
                store_address: dto.store_address,
                store_category: dto.store_category,
                store_latitude: dto.store_latitude,
                store_longitude: dto.store_longitude,
                user_id,
            })
            .await
    }

    pub async fn update_store(&self, store_id: i64, user_id: i64, updated: Store) -> Option<Store> {
        self.repository.update(store_id, user_id, &updated).await
    }

    pub async fn get_store(&self, store_id: i64) -> Option<Store> {
        self.repository.find_by_id(store_id).await
    }

    pub async fn get_store_by_user_id(&self, user_id: i64) -> Option<Store> {
        self.repository.find_by_user_id(user_id).await
    }

    pub async fn get_store_by_category(&self, category: &str) -> Vec<Store> {
        self.repository.find_by_category(category).await
    }

    // 성공 여부를 bool로 반환해 컨트롤러가 HTTP 상태코드를 쉽게 고르도록 한다.
    pub async fn delete_store(&self, store_id: i64, user_id: i64) -> bool {
        self.repository.delete(store_id, user_id).await
    }
}
