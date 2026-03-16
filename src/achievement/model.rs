use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 업적 정의 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub achievement_id: i64,
    pub achievement_name: String,
    pub condition_desc: Option<String>,
    pub condition_type: Option<String>,
    pub condition_value: Option<i32>,
    pub reward_point: Option<i32>,
    pub reward_exp: Option<i32>,
}

// 유저 업적 달성 기록이다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAchievement {
    pub user_achievement_id: i64,
    pub user_id: i64,
    pub achievement_id: i64,
    pub achieved_at: Option<NaiveDateTime>,
    pub achievement_name: Option<String>,
}
