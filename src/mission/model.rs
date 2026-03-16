use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// 미션 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mission {
    pub mission_id: i64,
    pub mission_title: String,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_info: Option<String>,
    pub mission_people: i32,
    pub mission_code: String,
    pub mission_image: Option<String>,
    pub store_id: i64,
    pub is_pull_up: Option<i32>,
    pub mission_created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MissionRequestDto {
    pub mission_title: String,
    pub mission_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCreateResponseDto {
    pub mission_id: i64,
    pub mission_code: String,
    pub mission_start: NaiveDate,
    pub mission_end: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionListResponseDto {
    pub mission_id: i64,
    pub user_id: i64,
    pub mission_title: String,
    pub mission_info: Option<String>,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_people: i32,
    pub mission_image: Option<String>,
    pub store_name: String,
    pub store_category: String,
    pub store_latitude: f64,
    pub store_longitude: f64,
    pub is_pull_up: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionParticipateResponseDto {
    pub participate_id: i64,
    pub mission_id: i64,
    pub mission_title: String,
    pub mission_participate_status: String,
    pub mission_participate_start_date: NaiveDate,
    pub store_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthenticateRequestDto {
    pub input_code: String,
    pub user_lat: f64,
    pub user_lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionBookmark {
    pub bookmark_id: i64,
    pub user_id: i64,
    pub mission_id: i64,
}
