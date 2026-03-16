use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::state::AppState;
use crate::status::service::StatusService;

// 종합 상태 API 핸들러다.

pub async fn get_total_status(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: StatusService = state.status_service;
    match service.get_user_total_status(user_id).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
