use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::achievement::service::AchievementService;
use crate::state::AppState;

// 업적 관련 API 핸들러다.

pub async fn get_achievements(State(state): State<AppState>) -> impl IntoResponse {
    let service: AchievementService = state.achievement_service;
    (StatusCode::OK, Json(service.get_achievements().await)).into_response()
}

pub async fn get_my_achievements(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: AchievementService = state.achievement_service;
    (
        StatusCode::OK,
        Json(service.get_user_achievements(user_id).await),
    )
        .into_response()
}

pub async fn check_achievements(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: AchievementService = state.achievement_service;
    service.check_and_achieve(user_id).await;
    (StatusCode::OK, "업적 체크 완료").into_response()
}
