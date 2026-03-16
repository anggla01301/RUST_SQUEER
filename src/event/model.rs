use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 이벤트 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub event_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}
