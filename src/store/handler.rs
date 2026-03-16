use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::state::AppState;
use crate::store::model::{Store, StoreRequestDto};
use crate::store::service::StoreService;

// 가게 API 핸들러다.

pub async fn create_store(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(dto): Json<StoreRequestDto>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    match service.create_store(user_id, dto).await {
        Some(store) => (StatusCode::OK, Json(store)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn update_store(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(store_id): Path<i64>,
    Json(store): Json<Store>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    match service.update_store(store_id, user_id, store).await {
        Some(store) => (StatusCode::OK, Json(store)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn get_store(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    match service.get_store(store_id).await {
        Some(store) => (StatusCode::OK, Json(store)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_my_store(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    match service.get_store_by_user_id(user_id).await {
        Some(store) => (StatusCode::OK, Json(store)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_store_by_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    (
        StatusCode::OK,
        Json(service.get_store_by_category(&category).await),
    )
        .into_response()
}

pub async fn delete_store(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(store_id): Path<i64>,
) -> impl IntoResponse {
    let service: StoreService = state.store_service;
    if service.delete_store(store_id, user_id).await {
        StatusCode::OK.into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}
