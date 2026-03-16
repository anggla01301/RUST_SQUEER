use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// 출석 체크 응답 DTO다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttendanceResponseDto {
    pub status: String,
    pub user_info_attend_straight: i32,
    pub user_info_attend: i32,
    pub user_info_attend_max: i32,
    pub reward_point: i32,
    pub user_info_point: i32,
    pub message: String,
}
