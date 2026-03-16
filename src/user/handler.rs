use axum::{extract::{Extension, State}, http::StatusCode, response::IntoResponse, Json};
use crate::state::AppState;
use crate::user::service::UserService;

// 현재 로그인 사용자 조회 API다.
pub async fn get_me(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: UserService = state.user_service;
    match service.get_me(user_id).await {
        Some(me) => (StatusCode::OK, Json(me)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
