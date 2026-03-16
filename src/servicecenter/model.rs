use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// 고객센터 문의 엔티티다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceCenter {
    pub service_center_id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
    pub created_date: NaiveDate,
    pub response_content: Option<String>,
    pub response_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInquiryRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
    pub created_date: NaiveDate,
    pub response_content: Option<String>,
    pub response_date: Option<NaiveDate>,
}
