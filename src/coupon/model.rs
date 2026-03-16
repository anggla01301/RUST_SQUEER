//! 쿠폰 엔티티와 요청/응답 DTO를 정의할 파일이다.
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponReceiveDetail {
    #[serde(rename = "receiveId")]
    pub receive_id: i64,
    pub used: i32,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "receivePath")]
    pub receive_path: Option<String>,
    #[serde(rename = "issuedAt")]
    pub issued_at: Option<NaiveDateTime>,
    #[serde(rename = "expiredAt")]
    pub expired_at: NaiveDateTime,
    #[serde(rename = "couponId")]
    pub coupon_id: i64,
    #[serde(rename = "couponName")]
    pub coupon_name: String,
    #[serde(rename = "userId")]
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponResponseDto {
    #[serde(rename = "receiveId")]
    pub receive_id: i64,
    #[serde(rename = "couponName")]
    pub coupon_name: String,
    #[serde(rename = "expiredAt")]
    pub expired_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouponUseResponseDto {
    pub status: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "couponName")]
    pub coupon_name: String,
    #[serde(rename = "missionPeople")]
    pub mission_people: i32,
    #[serde(rename = "expMultiplier")]
    pub exp_multiplier: i32,
    #[serde(rename = "canPullUp")]
    pub can_pull_up: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponHistoryResponseDto {
    #[serde(rename = "receiveNo")]
    pub receive_no: i64,
    #[serde(rename = "couponName")]
    pub coupon_name: String,
    pub used: i32,
    #[serde(rename = "issuedAt")]
    pub issued_at: Option<NaiveDateTime>,
    #[serde(rename = "expiredAt")]
    pub expired_at: NaiveDateTime,
}
