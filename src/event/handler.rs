use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::event::service::EventService;
use crate::state::AppState;

// 이벤트 API 핸들러다.

pub async fn get_active_events(State(state): State<AppState>) -> impl IntoResponse {
    let service: EventService = state.event_service;
    (StatusCode::OK, Json(service.get_running_events().await)).into_response()
}

pub async fn get_expired_events(State(state): State<AppState>) -> impl IntoResponse {
    let service: EventService = state.event_service;
    (StatusCode::OK, Json(service.get_closed_events().await)).into_response()
}
