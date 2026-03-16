//! 위치 기반 API 핸들러가 들어갈 파일이다.
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::location::service::LocationService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct NearbyMissionQuery {
    pub lat: f64,
    pub lng: f64,
    pub radius: Option<f64>,
}

pub async fn get_nearby_missions(
    State(state): State<AppState>,
    Query(query): Query<NearbyMissionQuery>,
) -> impl IntoResponse {
    let service: LocationService = state.location_service;
    (
        StatusCode::OK,
        Json(
            service
                .get_nearby_missions(query.lat, query.lng, query.radius.unwrap_or(1.0))
                .await,
        ),
    )
        .into_response()
}
