use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// 마이페이지 응답 DTO다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub user_id: i64,
    pub user_name: String,
    pub user_nickname: String,
    pub user_email: String,
    pub user_type: String,
    pub user_avatar: Option<String>,
    pub user_is_active: String,
    pub user_joindate: Option<NaiveDateTime>,
}
