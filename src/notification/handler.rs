use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::notification::service::NotificationService;
use crate::state::AppState;

// 알림 API 핸들러다.

pub async fn get_my_notifications(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: NotificationService = state.notification_service;
    (
        StatusCode::OK,
        Json(service.get_my_notifications(user_id).await),
    )
        .into_response()
}

pub async fn mark_read(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(notification_id): Path<i64>,
) -> impl IntoResponse {
    let service: NotificationService = state.notification_service;
    if service.mark_read(user_id, notification_id).await {
        StatusCode::OK.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
