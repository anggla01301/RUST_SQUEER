use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::ranking::service::RankingService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MyStatusQuery {
    pub r#type: String,
}

// 랭킹 API 핸들러다.

pub async fn get_weekly_ranking(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: RankingService = state.ranking_service;
    match service.get_ranking_board("WEEKLY", user_id).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_monthly_ranking(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: RankingService = state.ranking_service;
    match service.get_ranking_board("MONTHLY", user_id).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_my_status(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Query(query): Query<MyStatusQuery>,
) -> impl IntoResponse {
    if query.r#type != "WEEKLY" && query.r#type != "MONTHLY" {
        return (
            StatusCode::BAD_REQUEST,
            "type은 WEEKLY 또는 MONTHLY만 허용됩니다.",
        )
            .into_response();
    }

    let service: RankingService = state.ranking_service;
    match service.get_my_only_status(&query.r#type, user_id).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
