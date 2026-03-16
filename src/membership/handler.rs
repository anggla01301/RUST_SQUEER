//! 멤버십 API 핸들러가 들어갈 파일이다.
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::membership::service::MembershipService;
use crate::state::AppState;

pub async fn get_products(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: MembershipService = state.membership_service;
    match service.get_membership_products(user_id).await {
        Some(products) => (StatusCode::OK, Json(products)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn purchase(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(membership_no): Path<i64>,
) -> impl IntoResponse {
    let service: MembershipService = state.membership_service;
    match service.purchase(user_id, membership_no).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn get_history(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: MembershipService = state.membership_service;
    (
        StatusCode::OK,
        Json(service.get_history(user_id).await.unwrap_or_default()),
    )
        .into_response()
}
