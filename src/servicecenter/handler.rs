use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::servicecenter::model::CreateInquiryRequest;
use crate::servicecenter::service::ServiceCenterService;
use crate::state::AppState;

// 고객센터 API 핸들러다.

pub async fn create_inquiry(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<CreateInquiryRequest>,
) -> impl IntoResponse {
    let service: ServiceCenterService = state.service_center_service;
    match service.create(user_id, req).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn get_my_inquiries(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: ServiceCenterService = state.service_center_service;
    (StatusCode::OK, Json(service.my(user_id).await)).into_response()
}

pub async fn answer_inquiry(
    State(state): State<AppState>,
    Path(inquiry_id): Path<i64>,
    body: String,
) -> impl IntoResponse {
    let service: ServiceCenterService = state.service_center_service;
    match service.answer(inquiry_id, body).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
