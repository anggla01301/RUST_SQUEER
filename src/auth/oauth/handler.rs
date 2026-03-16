use super::model::SocialLoginRequest;
use super::service::{OAuthService, OAuthSuccessResponse};
use crate::auth::cookie::build_refresh_cookie;
use crate::auth::model::LoginResponseDto;

use axum::{
    extract::{Query, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use crate::state::AppState;

// OAuth 로그인 엔드포인트를 정의한다.
// 공급자 access token 또는 authorization code를 받아 내부 로그인 흐름으로 연결한다.

// 소셜 로그인 성공 응답은 세 공급자가 모두 동일하므로 공통 함수로 묶었다.
fn build_login_response(result: OAuthSuccessResponse) -> Response {
    let OAuthSuccessResponse {
        access_token,
        refresh_token,
        max_age,
        user_id,
        user_name,
        user_nickname,
        user_email,
        user_type,
        ..
    } = result;
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&refresh_token, max_age)
            .parse()
            .unwrap(),
    );
    (
        StatusCode::OK,
        headers,
        Json(LoginResponseDto {
            token: access_token,
            refresh_token,
            is_new_user: user_type == "PENDING",
            user_id,
            user_name,
            user_nickname,
            user_email,
            user_type,
            user_info_level: 0,
            user_info_point: 0,
        }),
    )
        .into_response()
}

// 브라우저 기반 OAuth 인가 코드 플로우의 콜백 엔드포인트다.
#[utoipa::path(
    get,
    path = "/api/auth/oauth/callback",
    params(
        ("code" = String, Query, description = "Authorization code"),
        ("provider" = String, Query, description = "Provider id (kakao|naver|google)")
    ),
    responses(
        (status = 302, description = "Redirect with access token", headers(("Location" = String, description = "Redirect URL"))),
        (status = 400, description = "Missing code/provider", body = String),
        (status = 500, description = "OAuth error", body = String)
    )
)]
pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let oauth_service: OAuthService = state.oauth_service;
    // provider와 code가 모두 있어야 공급자 토큰으로 교환할 수 있다.
    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => return (StatusCode::BAD_REQUEST, "code 없음").into_response(),
    };

    let registration_id = match params.get("provider") {
        Some(p) => p.clone(),
        None => return (StatusCode::BAD_REQUEST, "provider 없음").into_response(),
    };

    // 인가 코드를 공급자 access token으로 바꾼다.
    let access_token = match oauth_service.exchange_code(&registration_id, &code).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "토큰 발급 실패").into_response(),
    };

    // 공급자 API에서 사용자 프로필을 가져와 내부 User 모델로 매핑한다.
    let principal = match oauth_service
        .load_user(&registration_id, &access_token, None)
        .await
    {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "유저 로드 실패").into_response(),
    };

    // 내부 로그인 성공 처리 후 프런트 리다이렉트 URL을 조립한다.
    let result = match oauth_service.handle_oauth_success(principal.user).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "토큰 생성 실패").into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&result.refresh_token, result.max_age)
            .parse()
            .unwrap(),
    );

    let redirect_url = format!("{}#token={}", result.redirect_uri, result.access_token);
    headers.insert(axum::http::header::LOCATION, redirect_url.parse().unwrap());

    // 브라우저는 쿠키를 저장한 뒤 프런트엔드 주소로 이동한다.
    (StatusCode::FOUND, headers).into_response()
}

// 카카오 access token을 이미 가진 모바일/SPA 클라이언트를 위한 엔드포인트다.
#[utoipa::path(
    post,
    path = "/api/auth/kakao",
    request_body = SocialLoginRequest,
    responses(
        (status = 200, description = "Login success", body = LoginResponseDto, headers(("Set-Cookie" = String, description = "Refresh token cookie"))),
        (status = 500, description = "OAuth error", body = String)
    )
)]
pub async fn kakao_login(
    State(state): State<AppState>,
    Json(request): Json<SocialLoginRequest>,
) -> impl IntoResponse {
    let oauth_service: OAuthService = state.oauth_service;
    let principal = match oauth_service
        .load_user("kakao", &request.access_token, None)
        .await
    {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "카카오 로그인 실패").into_response(),
    };

    match oauth_service.handle_oauth_success(principal.user).await {
        Ok(result) => build_login_response(result),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "토큰 생성 실패").into_response(),
    }
}

// 네이버 access token 기반 로그인 엔드포인트다.
#[utoipa::path(
    post,
    path = "/api/auth/naver",
    request_body = SocialLoginRequest,
    responses(
        (status = 200, description = "Login success", body = LoginResponseDto, headers(("Set-Cookie" = String, description = "Refresh token cookie"))),
        (status = 500, description = "OAuth error", body = String)
    )
)]
pub async fn naver_login(
    State(state): State<AppState>,
    Json(request): Json<SocialLoginRequest>,
) -> impl IntoResponse {
    let oauth_service: OAuthService = state.oauth_service;
    let principal = match oauth_service
        .load_user("naver", &request.access_token, None)
        .await
    {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "네이버 로그인 실패").into_response(),
    };

    match oauth_service.handle_oauth_success(principal.user).await {
        Ok(result) => build_login_response(result),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "토큰 생성 실패").into_response(),
    }
}

// 구글 access token 기반 로그인 엔드포인트다.
#[utoipa::path(
    post,
    path = "/api/auth/google",
    request_body = SocialLoginRequest,
    responses(
        (status = 200, description = "Login success", body = LoginResponseDto, headers(("Set-Cookie" = String, description = "Refresh token cookie"))),
        (status = 500, description = "OAuth error", body = String)
    )
)]
pub async fn google_login(
    State(state): State<AppState>,
    Json(request): Json<SocialLoginRequest>,
) -> impl IntoResponse {
    let oauth_service: OAuthService = state.oauth_service;
    let principal = match oauth_service
        .load_user("google", &request.access_token, None)
        .await
    {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "구글 로그인 실패").into_response(),
    };

    match oauth_service.handle_oauth_success(principal.user).await {
        Ok(result) => build_login_response(result),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "토큰 생성 실패").into_response(),
    }
}
