// Axum에서 컨트롤러 함수에 자주 쓰는 타입들이다.
//
// State:
//   Router::with_state(AppState) 로 등록한 전역 상태를 꺼낼 때 사용한다.
//   예: AuthService, UserRepository 같은 공용 의존성
//
// Extension:
//   여기서는 JWT 미들웨어가 넣어준 요청 단위 데이터(user_id)를 꺼낼 때만 사용한다.
//
// Json:
//   요청 바디 JSON을 DTO로 역직렬화할 때 사용한다.
//
// HeaderMap:
//   요청 헤더/응답 헤더를 다룰 때 사용한다.
//
// StatusCode:
//   HTTP 상태 코드 (200, 401, 404, 500 등)
//
// IntoResponse / Response:
//   컨트롤러 함수의 반환값을 실제 HTTP 응답으로 변환할 때 사용한다.
use axum::{
    extract::{Extension, Json, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

// 환경변수 읽을 때 사용한다.
// 여기서는 JWT_REFRESH_EXPIRATION 같은 값을 읽는다.
use std::env;

// refresh token 쿠키를 만들고/삭제하는 공통 헬퍼 함수들이다.
// build_refresh_cookie: refreshToken 쿠키 문자열 생성
// clear_refresh_cookie: refreshToken 쿠키 삭제용 Set-Cookie 문자열 생성
use crate::auth::cookie::{build_refresh_cookie, clear_refresh_cookie};

// JWT 생성/검증 유틸리티다.
// handler에서는 refresh 로직에서 직접 사용한다.
use crate::auth::jwt::JwtUtil;

// 인증 관련 요청/응답 DTO들이다.
//
// LoginRequestDto: 로그인 요청 body
// RefreshResponse: refresh 응답 body (새 access token)
// SetUserRoleRequest: 역할 설정 요청 body
// StoreSignUpRequestDto: 점주 회원가입 요청 body
// TotalSignUpRequestDto: 통합 회원가입 요청 body
// UserProfileUpdateDto: 프로필 수정 요청 body
// UserSignUpRequestDto: 일반 유저 회원가입 요청 body
use crate::auth::model::{
    LoginRequestDto, RefreshResponse, SetUserRoleRequest, StoreSignUpRequestDto,
    TotalSignUpRequestDto, UserProfileUpdateDto, UserSignUpRequestDto,
};

use crate::auth::service::{AuthError, AuthService};
use crate::state::AppState;

// 인증 API 엔드포인트를 정의한다.
// 즉, routes.rs 에서 연결된 실제 컨트롤러 함수들이 이 파일에 있다.

// AuthError를 HTTP 응답으로 바꾸는 규칙을 정의한다.
//
// 이걸 구현해두면 handler에서 Err(e) 발생 시
// e.into_response() 로 바로 HTTP 응답 변환이 가능하다.
//
// 즉 서비스 에러를 컨트롤러에서 상태코드 + 메시지로 매핑하는 공통 규칙이다.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        // 각 AuthError 종류를 HTTP 상태코드와 사용자용 메시지로 변환한다.
        let (status, message) = match self {
            AuthError::EmailDuplicated => (StatusCode::CONFLICT, "이메일 중복"),
            AuthError::SocialEmailUpDuplicated => (StatusCode::CONFLICT, "소셜 이메일 중복"),
            AuthError::EmailNotFound => (StatusCode::NOT_FOUND, "이메일 없음"),
            AuthError::PasswordNotMatch => (StatusCode::UNAUTHORIZED, "비밀번호 불일치"),
            AuthError::AccountWithdraw => (StatusCode::FORBIDDEN, "탈퇴한 계정"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "유저 없음"),
            AuthError::UserInfoNotFound => (StatusCode::NOT_FOUND, "유저 정보 없음"),
            AuthError::SignupFailed => {
                (StatusCode::BAD_REQUEST, "회원가입 정보가 올바르지 않습니다")
            }
            AuthError::InvalidUserType => {
                (StatusCode::BAD_REQUEST, "유효하지 않은 사용자 타입입니다")
            }
        };

        // (StatusCode, &str) 튜플을 실제 HTTP 응답으로 변환한다.
        (status, message).into_response()
    }
}

// utoipa(OpenAPI/Swagger) 문서용 매크로다.
// 이 함수가 POST /api/auth/signup 라우트라는 것,
// 요청 body 타입과 응답 정보를 문서에 등록한다.
#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = TotalSignUpRequestDto,
    responses((status = 200, description = "Sign up success", body = String))
)]
// 통합 회원가입 컨트롤러다.
//
// State(state):
//   AppState 안에 모아둔 공용 서비스들을 꺼낸다.
//
// Json(dto):
//   요청 body JSON을 TotalSignUpRequestDto 로 파싱한다.
pub async fn sign_up(
    State(state): State<AppState>,
    Json(dto): Json<TotalSignUpRequestDto>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.sign_up(dto).await {
        // 성공 시 200 OK + "회원가입 성공"
        Ok(_) => (StatusCode::OK, "회원가입 성공").into_response(),

        // 실패 시 AuthError -> IntoResponse 규칙으로 HTTP 응답 변환
        Err(e) => e.into_response(),
    }
}

// 일반 사용자 회원가입 API 문서 정의
#[utoipa::path(
    post,
    path = "/api/auth/signup/user",
    request_body = UserSignUpRequestDto,
    responses((status = 200, description = "Sign up success", body = String))
)]
// 일반 유저 회원가입 컨트롤러다.
pub async fn sign_up_user(
    State(state): State<AppState>,
    Json(dto): Json<UserSignUpRequestDto>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.sign_up_user(dto).await {
        Ok(_) => (StatusCode::OK, "회원가입 성공").into_response(),
        Err(e) => e.into_response(),
    }
}

// 점주/가게 회원가입 API 문서 정의
#[utoipa::path(
    post,
    path = "/api/auth/signup/store",
    request_body = StoreSignUpRequestDto,
    responses((status = 200, description = "Sign up success", body = String))
)]
// 점주 회원가입 컨트롤러다.
pub async fn sign_up_store(
    State(state): State<AppState>,
    Json(dto): Json<StoreSignUpRequestDto>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.sign_up_store(dto).await {
        Ok(_) => (StatusCode::OK, "회원가입 성공").into_response(),
        Err(e) => e.into_response(),
    }
}

// 로그인 API 문서 정의
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequestDto,
    responses((status = 200, description = "Login success"))
)]
// 로그인 컨트롤러다.
//
// 로그인 성공 시 보통:
// - access token 은 응답 JSON body 에 포함
// - refresh token 은 HttpOnly cookie 로 내려주는 구조
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginRequestDto>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.login(dto).await {
        // service.login 성공 시 response 안에는 토큰 정보 등이 들어있다고 볼 수 있다.
        Ok(response) => {
            // refresh token 쿠키 만료시간을 환경변수에서 읽는다.
            //
            // JWT_REFRESH_EXPIRATION 이 없으면 기본값 604800000(밀리초, 약 7일)을 사용한다.
            // parse 실패 시에도 동일한 기본값 사용.
            let refresh_expiration = env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or("604800000".to_string())
                .parse::<i64>()
                .unwrap_or(604800000);

            // 응답 헤더를 담을 맵 생성
            let mut headers = HeaderMap::new();

            // Set-Cookie 헤더에 refresh token 쿠키를 넣는다.
            //
            // build_refresh_cookie(...) 는
            // "refreshToken=...; HttpOnly; Secure; SameSite=..." 같은 문자열을 만들어줄 가능성이 크다.
            //
            // refresh_expiration / 1000:
            // 환경변수가 ms 단위라면 쿠키 max-age 는 초 단위로 맞추기 위해 1000으로 나눈다.
            headers.insert(
                SET_COOKIE,
                build_refresh_cookie(&response.refresh_token, refresh_expiration / 1000)
                    .parse()
                    .unwrap(),
            );

            // 최종 응답:
            // - 상태코드 200
            // - Set-Cookie 헤더 포함
            // - JSON body 포함
            //
            // 주의:
            // 현재 설명에 따르면 refresh token 은 응답 JSON에서 숨겨지고
            // access token 중심 응답만 내려가도록 바뀌었을 수 있다.
            (StatusCode::OK, headers, Json(response)).into_response()
        }

        // 로그인 실패 시 AuthError -> HTTP 응답 변환
        Err(e) => e.into_response(),
    }
}

// 프로필 수정 API 문서 정의
#[utoipa::path(
    patch,
    path = "/api/auth/profile",
    request_body = UserProfileUpdateDto,
    responses((status = 200, description = "Profile updated"))
)]
// 프로필 수정 컨트롤러다.
//
// Extension(user_id): JWT 미들웨어가 request.extensions 에 넣어둔 user_id 를 꺼낸다.
// 즉 이 함수는 "인증된 사용자"를 전제로 동작한다.
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(dto): Json<UserProfileUpdateDto>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.update_profile(user_id, dto).await {
        // 성공 시 수정된 응답 데이터를 JSON으로 반환
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => e.into_response(),
    }
}

// 역할 변경 API 문서 정의
#[utoipa::path(
    patch,
    path = "/api/auth/role",
    request_body = SetUserRoleRequest,
    responses((status = 200, description = "Role set", body = String))
)]
// 사용자 역할 변경 컨트롤러다.
// 현재 로그인한 user_id와 요청 body의 user_type 값을 service에 전달한다.
pub async fn set_user_role(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(request): Json<SetUserRoleRequest>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service
        .update_user_type(user_id, &request.user_type)
        .await
    {
        Ok(_) => (StatusCode::OK, "역할 설정 완료").into_response(),
        Err(e) => e.into_response(),
    }
}

// 로그아웃 API 문서 정의
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses((status = 200, description = "Logout success", body = String))
)]
// 로그아웃 컨트롤러다.
//
// 보통 로그아웃 시:
// - DB에 저장된 refresh token 제거
// - 브라우저 쿠키의 refresh token 제거(Set-Cookie로 만료 쿠키 전송)
pub async fn logout(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> impl IntoResponse {
    let auth_service: AuthService = state.auth_service;
    match auth_service.logout(user_id).await {
        Ok(_) => {
            // 응답 헤더 생성
            let mut headers = HeaderMap::new();

            // refresh token 삭제용 Set-Cookie 헤더 추가
            // clear_refresh_cookie() 는 만료된 refreshToken 쿠키 문자열을 만들어줄 가능성이 크다.
            headers.insert(SET_COOKIE, clear_refresh_cookie().parse().unwrap());

            // 200 OK + 쿠키 삭제 헤더 + 성공 메시지 반환
            (StatusCode::OK, headers, "로그아웃 성공").into_response()
        }
        Err(e) => e.into_response(),
    }
}

// 토큰 재발급(refresh) API 문서 정의
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses((status = 200, description = "Token refreshed", body = RefreshResponse))
)]
// refresh token 으로 새 access token 을 발급하는 컨트롤러다.
//
// 특징:
// - Authorization 헤더가 아니라 cookie 헤더에서 refreshToken 을 찾는다.
// - refresh token 검증 후 user_id 추출
// - DB에 저장된 refresh token 과 비교
// - 새 access token 생성
// - 새 refresh token 도 생성해서 DB와 cookie를 갱신 (rotation)
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_repository = state.user_repository;
    // JwtUtil 생성
    let jwt_util = match JwtUtil::new() {
        Ok(jwt_util) => jwt_util,
        Err(err) => {
            tracing::error!("JWT 설정 오류: {err:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "JWT 설정 오류").into_response();
        }
    };

    // 요청 헤더에서 cookie 헤더를 읽고,
    // 그 안에서 refreshToken=... 값을 파싱해 꺼낸다.
    //
    // 흐름:
    // headers.get("cookie")
    //   -> cookie 헤더 가져오기
    // and_then(|v| v.to_str().ok())
    //   -> 문자열로 변환
    // and_then(|cookies| { ... })
    //   -> ; 로 나눠 각 쿠키를 순회하면서 refreshToken= 접두사 찾기
    // map(|value| value.to_string())
    //   -> 최종적으로 Option<String>
    let refresh_token = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let cookie = cookie.trim();
                cookie.strip_prefix("refreshToken=")
            })
        })
        .map(|value| value.to_string());

    // refresh_token 이 존재하고, JWT 유효성도 통과해야 한다.
    //
    // 둘 중 하나라도 실패하면 401 반환
    let refresh_token = match refresh_token {
        Some(token) if jwt_util.validate_refresh_token(&token) => token,
        _ => return (StatusCode::UNAUTHORIZED, "유효하지 않은 리프레시 토큰").into_response(),
    };

    // refresh token 안에서 user_id 추출
    let user_id = match jwt_util.get_refresh_user_id(&refresh_token) {
        Some(user_id) => user_id,
        None => return (StatusCode::UNAUTHORIZED, "유효하지 않은 토큰").into_response(),
    };

    // DB에서 해당 user_id 의 유저 조회
    let user = match user_repository.find_by_id(user_id).await {
        Some(user) => user,
        None => return (StatusCode::NOT_FOUND, "유저 없음").into_response(),
    };

    // DB에 저장된 refresh_token 과 현재 요청의 refresh_token 이 일치하는지 확인
    //
    // 이 검사는 중요한 보안 포인트다.
    // 단순히 JWT 형식이 유효한 것만으로 끝내지 않고,
    // 서버(DB)가 현재 인정하는 refresh token 과 같은지도 확인한다.
    if user.refresh_token.as_deref() != Some(refresh_token.as_str()) {
        return (StatusCode::UNAUTHORIZED, "토큰 불일치").into_response();
    }

    // 새 access token 생성
    let new_access_token = jwt_util.generate_token(
        user_id,
        user.user_email.as_deref().unwrap_or(""),
        &user.user_type,
    );

    // 새 refresh token 생성 (rotation)
    let new_refresh_token = jwt_util.generate_refresh_token(user_id);

    // DB에 새 refresh token 저장
    user_repository
        .update_refresh_token(user_id, &new_refresh_token)
        .await;

    // refresh token 쿠키 만료시간 읽기
    let refresh_expiration = env::var("JWT_REFRESH_EXPIRATION")
        .unwrap_or("604800000".to_string())
        .parse::<i64>()
        .unwrap_or(604800000);

    // 응답 헤더 생성
    let mut headers = HeaderMap::new();

    // 새 refresh token 을 Set-Cookie 헤더로 내려준다.
    headers.insert(
        SET_COOKIE,
        build_refresh_cookie(&new_refresh_token, refresh_expiration / 1000)
            .parse()
            .unwrap(),
    );

    // 최종 응답:
    // - 200 OK
    // - 새 refresh token 쿠키
    // - JSON body 에는 새 access token 만 포함
    (
        StatusCode::OK,
        headers,
        Json(RefreshResponse {
            access_token: new_access_token,
        }),
    )
        .into_response()
}
