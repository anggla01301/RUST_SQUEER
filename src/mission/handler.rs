use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::mission::model::{AuthenticateRequestDto, Mission, MissionRequestDto};
use crate::mission::service::MissionService;
use crate::state::AppState;

// 미션 API 핸들러다.

pub async fn create_mission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(dto): Json<MissionRequestDto>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    match service.create_mission(user_id, dto).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn get_missions(State(state): State<AppState>) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    (StatusCode::OK, Json(service.get_missions().await)).into_response()
}

pub async fn get_my_missions(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    (
        StatusCode::OK,
        Json(service.get_my_missions(user_id).await),
    )
        .into_response()
}

pub async fn get_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    match service.get_mission(mission_id).await {
        Some(mission) => (StatusCode::OK, Json(mission)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn update_mission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(mission_id): Path<i64>,
    Json(mission): Json<Mission>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    match service.update_mission(mission_id, user_id, mission).await {
        Some(updated) => (StatusCode::OK, Json(updated)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn delete_mission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(mission_id): Path<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    if service.delete_mission(mission_id, user_id).await {
        StatusCode::OK.into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn get_missions_by_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    (
        StatusCode::OK,
        Json(service.get_missions_by_category(&category).await),
    )
        .into_response()
}

pub async fn participate(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(mission_id): Path<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    match service.participate(mission_id, user_id).await {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn authenticate(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(participate_id): Path<i64>,
    Json(dto): Json<AuthenticateRequestDto>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    match service.authenticate(participate_id, user_id, dto).await {
        Some(message) => (StatusCode::OK, message).into_response(),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn add_bookmark(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(mission_id): Path<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    if service.add_bookmark(mission_id, user_id).await {
        (StatusCode::OK, "찜 추가 완료").into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn remove_bookmark(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(mission_id): Path<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    if service.remove_bookmark(mission_id, user_id).await {
        (StatusCode::OK, "찜 취소 완료").into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn get_my_bookmarks(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let service: MissionService = state.mission_service;
    (
        StatusCode::OK,
        Json(service.get_my_bookmarks(user_id).await),
    )
        .into_response()
}
