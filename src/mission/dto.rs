// ============================================================
// mission/dto.rs — 요청/응답 DTO 구조체
// DB 엔티티는 model.rs 에 있다.
// ============================================================

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

// ── 미션 생성 요청 ───────────────────────────────────────────
// MissionRequestDTO.java 대응
#[derive(Deserialize, Debug)]
pub struct MissionRequestDto{
    pub mission_title: String,
    pub mission_info: String,
}

// ── 미션 생성 응답 ───────────────────────────────────────────
// MissionCreateResponseDTO.java 대응
#[derive(Debug, Serialize)]
pub struct MissionCreateResponseDto{
    pub mission_id: i64,
    pub mission_code: String,
    pub mission_start: NaiveDate,
    pub mission_end: NaiveDate,
}

// ── 미션 목록 응답 ───────────────────────────────────────────
// MissionListResponseDTO.java 대응
// 목록/찜/내 미션 조회 모두 이 DTO 로 통일한다.
#[derive(Debug, Serialize,Clone)]
pub struct MissionListResponseDto{
    pub mission_id: i64,
    pub user_id: i64,          // 점주 user_id
    pub mission_title: Option<String>,
    pub mission_info: Option<String>,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_people: i32,
    pub mission_image: Option<String>, // presigned URL 로 변환된 값
    pub store_name: Option<String>,
    pub store_category: Option<String>,
    pub store_latitude: Option<BigDecimal>,
    pub store_longitude: Option<BigDecimal>,
    pub is_pull_up: i32,
}

// ── 미션 인증 코드 응답 ──────────────────────────────────────
// MissionCodeResponseDTO.java 대응
#[derive(Debug, Serialize)]
pub struct MissionCodeResponseDto {
    pub mission_code: String,
}

// ── 미션 참여 응답 ───────────────────────────────────────────
// MissionParticipateResponseDTO.java 대응
#[derive(Debug, Serialize)]
pub struct MissionParticipateResponseDto {
    pub participate_id: i64,
    pub mission_id: i64,
    pub mission_title: Option<String>,
    pub mission_participate_status: Option<String>,
    pub mission_participate_start_date: Option<NaiveDate>,
    pub store_name: Option<String>,
}

// ── 인증 요청 DTO ────────────────────────────────────────────
// AuthenticateRequestDTO.java 대응
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequestDto {
    pub input_code: String,   // 사용자가 입력한 인증 코드
    pub user_lat: f64,        // 인증 시점 사용자 위도
    pub user_lng: f64,        // 인증 시점 사용자 경도
}

// ── 인증 응답 DTO ────────────────────────────────────────────
// AuthenticateResponseDTO.java 대응
// 인증 처리 직후 최신 USER_INFO 스냅샷까지 같이 내려준다.
#[derive(Debug, Serialize)]
pub struct AuthenticateResponseDto {
    pub status: String,            // "SUCCESS", "FAILED", "ALREADY_COMPLETED"
    pub message: String,
    pub fail_count: i64,
    pub completed: bool,
    // 인증 처리 직후 최신 USER_INFO 스냅샷
    pub user_info_level: Option<i32>,
    pub user_info_exp: Option<i32>,
    pub user_info_point: Option<i32>,
    pub user_info_mission_do: Option<i32>,
    pub user_info_mission_make: Option<i32>,
    pub user_info_attend: Option<i32>,
    pub user_info_attend_straight: Option<i32>,
    pub user_info_attend_max: Option<i32>,
    pub last_attend_date: Option<NaiveDate>,
    pub temp_mission_people: Option<i32>,
    pub temp_exp_multiplier: Option<i32>,
    // 경험치 진행도 계산값
    pub current_level_min_exp: Option<i32>,
    pub next_level_exp: Option<i32>,
    pub current_level_progress_exp: Option<i32>,
    pub required_exp_for_next_level: Option<i32>,
    pub remaining_exp_to_next_level: Option<i32>,
    pub exp_progress_percent: Option<i32>,
}

// ── 미션 수정 요청 ───────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct MissionUpdateDto {
    pub mission_title: Option<String>,
    pub mission_info: Option<String>,
    pub mission_start: Option<NaiveDate>,
    pub mission_end: Option<NaiveDate>,
    pub mission_people: Option<i32>,
}