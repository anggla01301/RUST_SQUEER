//! 멤버십 등급/혜택 모델을 정의할 파일이다.
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Membership {
    #[serde(rename = "membershipId")]
    pub membership_id: i64,
    #[serde(rename = "membershipName")]
    pub membership_name: String,
    pub price: i32,
    pub benefits: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipPaymentDto {
    #[serde(rename = "paymentId")]
    pub payment_id: i64,
    #[serde(rename = "membershipName")]
    pub membership_name: String,
    #[serde(rename = "missionMake")]
    pub mission_make: i32,
    #[serde(rename = "missionDo")]
    pub mission_do: i32,
    #[serde(rename = "currentPoint")]
    pub current_point: i32,
    #[serde(rename = "successMessage")]
    pub success_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MembershipHistoryDto {
    #[serde(rename = "paymentId")]
    pub payment_id: i64,
    #[serde(rename = "membershipName")]
    pub membership_name: String,
    pub amount: i32,
    #[serde(rename = "paidAt")]
    pub paid_at: NaiveDateTime,
    pub status: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct MembershipPayment {
    pub payment_id: i64,
}
