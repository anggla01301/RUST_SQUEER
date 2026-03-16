use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

// 공통 에러 응답 바디다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

// 공통 성공 응답 바디다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

// 스프링 ErrorCode를 러스트 enum으로 옮긴 공통 예외 타입이다.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("입력값이 올바르지 않습니다")]
    InvalidInput,
    #[error("이미 사용중인 이메일입니다")]
    EmailDuplicated,
    #[error("이메일 또는 비밀번호가 일치하지 않습니다")]
    EmailNotFound,
    #[error("이미 탈퇴한 계정입니다")]
    AccountWithdraw,
    #[error("유효하지 않은 토큰입니다")]
    InvalidToken,
    #[error("사용자를 찾을 수 없습니다")]
    UserNotFound,
    #[error("유저 정보를 찾을 수 없습니다")]
    UserInfoNotFound,
    #[error("이미 가입된 이메일입니다")]
    SocialEmailDuplicated,
    #[error("회원가입에 실패했습니다")]
    SignupFailed,
    #[error("가게를 찾을 수 없습니다")]
    StoreNotFound,
    #[error("미션을 찾을 수 없습니다")]
    MissionNotFound,
    #[error("참여 정보를 찾을 수 없습니다")]
    ParticipateNotFound,
    #[error("미션 생성 가능 횟수가 부족합니다")]
    MissionCreateLimit,
    #[error("이미 참여한 미션입니다")]
    MissionAlreadyParticipated,
    #[error("이미 찜한 미션입니다.")]
    MissionAlreadyBookmarked,
    #[error("찜한 미션을 찾을 수 없습니다.")]
    MissionBookmarkNotFound,
    #[error("알림을 찾을 수 없습니다")]
    NotificationNotFound,
    #[error("문의를 찾을 수 없습니다")]
    InquiryNotFound,
    #[error("이미 처리된 결제입니다.")]
    PaymentAlreadyProcessed,
    #[error("서버 오류가 발생했습니다")]
    Internal,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidInput => StatusCode::BAD_REQUEST,
            Self::EmailDuplicated => StatusCode::BAD_REQUEST,
            Self::EmailNotFound => StatusCode::NOT_FOUND,
            Self::AccountWithdraw => StatusCode::BAD_REQUEST,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::UserInfoNotFound => StatusCode::NOT_FOUND,
            Self::SocialEmailDuplicated => StatusCode::BAD_REQUEST,
            Self::SignupFailed => StatusCode::BAD_REQUEST,
            Self::StoreNotFound => StatusCode::NOT_FOUND,
            Self::MissionNotFound => StatusCode::NOT_FOUND,
            Self::ParticipateNotFound => StatusCode::NOT_FOUND,
            Self::MissionCreateLimit => StatusCode::BAD_REQUEST,
            Self::MissionAlreadyParticipated => StatusCode::BAD_REQUEST,
            Self::MissionAlreadyBookmarked => StatusCode::BAD_REQUEST,
            Self::MissionBookmarkNotFound => StatusCode::NOT_FOUND,
            Self::NotificationNotFound => StatusCode::NOT_FOUND,
            Self::InquiryNotFound => StatusCode::NOT_FOUND,
            Self::PaymentAlreadyProcessed => StatusCode::CONFLICT,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status_code(),
            Json(ErrorResponse {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
