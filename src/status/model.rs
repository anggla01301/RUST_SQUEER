use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// 앱 메인 상태 응답 DTO다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotalStatusResponseDto {
    pub user_type: String,
    pub user_info_point: i32,
    pub user_info_mission_make: i32,
    pub user_info_mission_do: i32,
    pub temp_mission_people: i32,
    pub temp_exp_multiplier: i32,
    pub user_info_attend_straight: i32,
    pub user_info_attend: i32,
    pub user_info_attend_max: i32,
    pub attendance_status: String,
    pub is_pull_up_active: i32,
}
