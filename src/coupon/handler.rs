//! 쿠폰 관련 HTTP 핸들러가 들어갈 파일이다.
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::coupon::service::CouponService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CouponUseQuery {
    #[serde(rename = "missionId")]
    pub mission_id: Option<i64>,
}

pub async fn get_my_coupons(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: CouponService = state.coupon_service;
    (
        StatusCode::OK,
        Json(service.get_my_available_coupons(user_id).await),
    )
        .into_response()
}

pub async fn use_coupon(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(receive_no): Path<i64>,
    Query(query): Query<CouponUseQuery>,
) -> impl IntoResponse {
    let service: CouponService = state.coupon_service;
    match service
        .use_coupon_and_extract(user_id, receive_no, query.mission_id)
        .await
    {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn delete_coupon(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(receive_no): Path<i64>,
) -> impl IntoResponse {
    let service: CouponService = state.coupon_service;
    if service.delete_coupon(user_id, receive_no).await {
        (StatusCode::OK, "쿠폰이 성공적으로 삭제되었습니다.").into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn get_coupon_history(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: CouponService = state.coupon_service;
    (
        StatusCode::OK,
        Json(service.get_all_coupon_history(user_id).await),
    )
        .into_response()
}
