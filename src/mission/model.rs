// ============================================================
// mission/model.rs — DB 테이블과 매핑되는 엔티티 구조체
// DTO 는 dto.rs 에 있다.
// ============================================================

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

// ── Mission — MISSION 테이블 엔티티 ─────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mission {
    pub mission_id: i64,
    pub mission_title: Option<String>,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_info: Option<String>,
    pub mission_people: i32,
    pub mission_code: Option<String>,
    pub mission_image: Option<String>,
    pub store_id: Option<i64>,
    pub is_pull_up: i32,
    pub store_reward_given_yn: Option<String>,
    pub mission_created_at: Option<NaiveDateTime>,
}

// ── MissionBookmark — MISSION_BOOKMARK 테이블 엔티티 ─────────
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionBookmark {
    pub bookmark_id: i64,
    pub user_id: i64,
    pub mission_id: i64,
}

// ── MissionParticipate — MISSION_PARTICIPATE 테이블 엔티티 ───
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionParticipate {
    pub participate_id: i64,
    pub mission_id: i64,
    pub user_id: i64,
    pub mission_participate_code: Option<String>,
    pub mission_participate_status: Option<String>,
    pub mission_participate_start_date: Option<NaiveDate>,
    pub mission_participate_complete_date: Option<NaiveDate>,
    pub mission_participate_given_yn: Option<String>,
    pub mission_participate_reward_exp: Option<i64>,
    pub mission_participate_authenticated_try: Option<i64>,
    pub mission_participate_locked_at: Option<NaiveDateTime>,
    pub mission_participate_locked_until: Option<NaiveDateTime>,
    pub coupon_receive_id: Option<i64>,
}

// ── MissionReview — MISSION_REVIEW 테이블 엔티티 ────────────
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionReview {
    pub mission_review_id: i64,
    pub participate_id: i64,
    pub mission_review_rating: f64,
    pub mission_review_content: String,
    pub mission_review_image_path: Option<String>,
    pub mission_review_date: NaiveDate,
    pub mission_review_like_count: Option<i64>,
}

// ── MissionReviewLike — MISSION_REVIEW_LIKE 테이블 엔티티 ────
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissionReviewLike {
    pub mission_review_like_id: i64,
    pub user_id: i64,
    pub mission_review_id: i64,
    pub mission_review_like_date: NaiveDate,
}

// ── MissionWithStore — JOIN 결과를 담는 뷰 구조체 ────────────
// JPA 의 fetch join 대신 JOIN 쿼리 결과를 직접 받는 구조체.
// service 에서 DTO 변환 시 사용한다.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MissionWithStore {
    // mission 컬럼들
    pub mission_id: i64,
    pub mission_title: Option<String>,
    pub mission_info: Option<String>,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_people: i32,
    pub mission_image: Option<String>,
    pub is_pull_up: i32,
    pub mission_code: Option<String>,
    pub store_reward_given_yn: Option<String>,
    pub mission_created_at: Option<NaiveDateTime>,
    // store 컬럼들 (JOIN)
    pub store_id: i64,
    pub store_owner_user_id: i64,
    pub store_name: Option<String>,
    pub store_category: Option<String>,
    pub store_latitude: Option<BigDecimal>,
    pub store_longitude: Option<BigDecimal>,
}

// ── MissionParticipateWithDetail — 인증 처리 시 JOIN 결과 ────
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MissionParticipateWithDetail {
    // participate 컬럼들
    pub participate_id: i64,
    pub mission_id: i64,
    pub user_id: i64,
    pub mission_participate_status: Option<String>,
    pub mission_participate_start_date: Option<NaiveDate>,
    pub mission_participate_complete_date: Option<NaiveDate>,
    pub mission_participate_given_yn: Option<String>,
    pub mission_participate_reward_exp: Option<i64>,
    pub mission_participate_authenticated_try: Option<i64>,
    pub mission_participate_locked_at: Option<NaiveDateTime>,
    pub mission_participate_locked_until: Option<NaiveDateTime>,
    // mission 컬럼들 (JOIN)
    pub mission_title: Option<String>,
    pub mission_code: Option<String>,
    pub mission_people: i32,
    pub store_reward_given_yn: Option<String>,
    // store 컬럼들 (JOIN)
    pub store_id: i64,
    pub store_owner_user_id: i64,
    pub store_name: Option<String>,
    pub store_latitude: Option<BigDecimal>,
    pub store_longitude: Option<BigDecimal>,
}
