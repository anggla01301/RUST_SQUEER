use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// 저장된 랭킹 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ranking {
    pub ranking_id: i64,
    pub r#type: String,
    pub user_id: i64,
    pub rank_no: i32,
    pub exp_score: i32,
    pub reward_point: i32,
    pub period_start: NaiveDateTime,
    pub period_end: NaiveDateTime,
    pub settled_nickname: Option<String>,
    pub settled_avatar: Option<String>,
    pub settled_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct RankItemDto {
    pub rank_no: i32,
    pub user_id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
    pub season_exp: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RankingResponseDto {
    pub r#type: String,
    pub my_ranking: Option<RankItemDto>,
    pub ranking_list: Vec<RankItemDto>,
}
