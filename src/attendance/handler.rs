use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::attendance::service::AttendanceService;
use crate::state::AppState;

// 출석 API 핸들러 모음이다.

pub async fn check_in(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: AttendanceService = state.attendance_service;
    let result = service.perform_check_in(user_id).await;
    if result.status == "FAIL" || result.status == "ERROR" {
        return (StatusCode::BAD_REQUEST, Json(result)).into_response();
    }
    (StatusCode::OK, Json(result)).into_response()
}

pub async fn get_status(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: AttendanceService = state.attendance_service;
    let result = service.get_attendance_status(user_id).await;
    if result.status == "FAIL" || result.status == "ERROR" {
        return (StatusCode::BAD_REQUEST, Json(result)).into_response();
    }
    (StatusCode::OK, Json(result)).into_response()
}
