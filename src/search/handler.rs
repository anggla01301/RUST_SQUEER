use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::search::service::SearchService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct KeywordQuery {
    pub keyword: Option<String>,
    pub category: Option<String>,
}

// 검색 API 핸들러다.

pub async fn search_mission(
    State(state): State<AppState>,
    Query(query): Query<KeywordQuery>,
) -> impl IntoResponse {
    let service: SearchService = state.search_service;
    let keyword = query.keyword.unwrap_or_default();
    (
        StatusCode::OK,
        Json(service.search_by_mission_title(&keyword).await),
    )
        .into_response()
}

pub async fn search_store(
    State(state): State<AppState>,
    Query(query): Query<KeywordQuery>,
) -> impl IntoResponse {
    let service: SearchService = state.search_service;
    let keyword = query.keyword.unwrap_or_default();
    (
        StatusCode::OK,
        Json(service.search_by_store_name(&keyword).await),
    )
        .into_response()
}

pub async fn search_category(
    State(state): State<AppState>,
    Query(query): Query<KeywordQuery>,
) -> impl IntoResponse {
    let service: SearchService = state.search_service;
    let category = query.category.unwrap_or_default();
    (
        StatusCode::OK,
        Json(service.search_by_category(&category).await),
    )
        .into_response()
}
