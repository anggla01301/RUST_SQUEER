use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 사용자 알림 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserNotification {
    pub notification_id: i64,
    pub notification_title: String,
    pub notification_content: String,
    pub created_at: Option<NaiveDateTime>,
    pub notification_is_read: Option<i32>,
    pub notification_receiver_id: Option<i64>,
    pub notification_sender_id: Option<i64>,
}
