use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::config::service::ConfigService;
use crate::state::AppState;

// 런타임 설정 조회 API다.
pub async fn get_runtime_config(State(state): State<AppState>) -> impl IntoResponse {
    let service: ConfigService = state.config_service;
    (StatusCode::OK, Json(service.get_runtime_config())).into_response()
}
