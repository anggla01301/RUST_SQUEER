use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::common::model::MessageResponse;
use crate::common::service::OciStorageService;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ImagePathRequest {
    pub original_name: String,
    pub folder: String,
}

// 공통 이미지 경로 생성 API다.
pub async fn create_image_path(
    State(state): State<AppState>,
    Json(request): Json<ImagePathRequest>,
) -> impl IntoResponse {
    let service: OciStorageService = state.oci_storage_service;
    match service.build_object_name(&request.original_name, &request.folder) {
        Ok(path) => (StatusCode::OK, Json(MessageResponse { message: path })).into_response(),
        Err(err) => err.into_response(),
    }
}
