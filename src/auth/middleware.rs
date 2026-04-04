// "Authorization" HTTP 헤더 이름 상수다.
// 예: Authorization: Bearer <jwt-token>
use axum::http::header::AUTHORIZATION;

// Axum에서 미들웨어를 만들 때 자주 쓰는 타입들이다.
// Request: 현재 들어온 HTTP 요청
// StatusCode: HTTP 상태 코드 (200, 401, 500 등)
// Next: 다음 단계(다음 미들웨어 또는 실제 핸들러)로 요청을 넘기는 객체
// Response: 최종 HTTP 응답 타입
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

// JWT 생성/검증 유틸리티다.
// 여기서는 주로
// - validate_token(&token): 토큰 유효성 검사
// - get_user_id(&token): 토큰 안의 user_id 추출
// 에 사용한다.
use crate::auth::jwt::JwtUtil;

// 보호(private) 라우트 앞단에서 JWT를 검사하는 Axum 미들웨어다.
//
// 이 미들웨어의 전체 역할:
// 1. Authorization 헤더가 있는지 확인
// 2. "Bearer <token>" 형식인지 확인
// 3. JWT 유효성 검사
// 4. 토큰 안에서 user_id 추출
// 5. request extension에 user_id를 넣어 다음 핸들러가 사용하게 함
//
// 성공하면 다음 핸들러로 넘기고,
// 실패하면 여기서 바로 401 / 500 응답을 반환한다.
pub async fn jwt_middleware(
    // 현재 들어온 HTTP 요청이다.
    //
    // mut 인 이유:
    // 아래에서 request.extensions_mut().insert(user_id) 로
    // 요청 내부에 user_id를 심어 넣어야 하므로 수정 가능해야 한다.
    mut request: Request,

    // 다음 단계(다음 미들웨어 또는 실제 핸들러)로 요청을 넘기는 객체다.
    //
    // next.run(request).await 를 호출하면
    // "이 미들웨어의 다음 단계"가 실행된다.
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // 요청 헤더에서 Authorization 헤더를 꺼낸다.
    // request.headers().get(AUTHORIZATION) 의 반환값은 Option<&HeaderValue> 다.
    // 즉:
    // - Some(...) = 헤더가 있음
    // - None = 헤더가 없음
    //
    // ok_or(...) 는 Option을 Result로 바꾸는 역할이다.
    // 헤더가 없으면 즉시 Err((401, "Authorization 헤더 없음")) 으로 바꾼다.
    //
    // 뒤의 ? 때문에 헤더가 없으면
    // 이 함수는 바로 종료되고 401 에러를 반환한다.
    let header = request.headers().get(AUTHORIZATION).ok_or((
        StatusCode::UNAUTHORIZED,
        "Authorization 헤더 없음".to_string(),
    ))?;

    // Authorization 헤더 값을 문자열로 바꾼다.
    //
    // header.to_str() 는 Result<&str, _> 를 반환한다.
    // 즉:
    // - 성공하면 헤더를 문자열로 읽음
    // - 실패하면 헤더 형식이 잘못된 것
    //
    // map_err(...) 는 에러 타입을 우리가 원하는 형태로 바꾸는 역할이다.
    // 여기서는 "Authorization 헤더 형식 오류" 라는 401로 변환한다.
    //
    // 뒤의 ? 때문에 변환 실패 시 즉시 함수 종료.
    let value = header.to_str().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "Authorization 헤더 형식 오류".to_string(),
        )
    })?;

    // Authorization 헤더에서 실제 Bearer 토큰 문자열만 추출한다.
    //
    // 이 부분의 흐름:
    //
    // 1. strip_prefix("Bearer ")
    //    - 문자열이 "Bearer " 로 시작하면 그 뒤 부분만 반환
    //    - 아니면 None 반환
    //
    // 2. map(str::trim)
    //    - 앞뒤 공백 제거
    //
    // 3. filter(|token| !token.is_empty())
    //    - 공백 제거 후 빈 문자열이면 버림
    //
    // 4. ok_or(...)
    //    - 위 과정에서 실패하면 401 "Bearer 토큰 없음"
    //
    // 5. to_string()
    //    - 최종적으로 String 으로 소유권 있는 문자열 생성
    //
    // 예:
    // "Bearer abc.def.ghi" -> "abc.def.ghi"
    let token = value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "Bearer 토큰 없음".to_string()))?
        .to_string();

    // 디버그 로그 출력
    // 실제 운영에서는 로그 레벨 설정에 따라 보일 수도 있고 안 보일 수도 있다.
    tracing::debug!("JwtFilter Authorization header received.");

    // JwtUtil 객체를 생성한다.
    //
    // 내부적으로 secret key 로딩, 환경변수 확인 등 설정 검증이 있을 수 있어서
    // 실패 가능성이 있다.
    let jwt_util = match JwtUtil::new() {
        // 정상 생성 시 jwt_util 사용
        Ok(jwt_util) => jwt_util,

        // JwtUtil 생성 실패는 클라이언트 잘못이 아니라
        // 서버 설정 문제에 가깝다.
        // 그래서 500 Internal Server Error 를 반환한다.
        Err(err) => {
            tracing::error!("JWT 설정 오류: {err:#}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT 설정 오류".to_string(),
            ));
        }
    };

    // JWT 유효성 검사
    //
    // validate_token(&token) 이 false 를 반환하면:
    // - 위조 토큰
    // - 만료 토큰
    // - 형식이 잘못된 토큰
    // 등일 가능성이 있다.
    //
    // 이런 경우 즉시 401 Unauthorized 반환
    if !jwt_util.validate_token(&token) {
        tracing::debug!("JwtFilter 유효하지 않은 토큰");
        return Err((StatusCode::UNAUTHORIZED, "유효하지 않은 토큰".to_string()));
    }

    // 토큰이 유효하면 user_id 를 꺼낸다.
    //
    // get_user_id(&token) 은 Option<...> 을 반환한다고 가정할 수 있다.
    // 즉:
    // - Some(id) = 토큰 안에 user_id 있음
    // - None = 토큰은 형식상 통과했지만 user_id 클레임이 없음
    //
    // user_id 가 없으면 인증 실패로 간주하고 401 반환
    let user_id = match jwt_util.get_user_id(&token) {
        // user_id 추출 성공
        Some(id) => id,

        // user_id 가 없으면 인증 대상 사용자를 특정할 수 없으므로 실패 처리
        None => return Err((StatusCode::UNAUTHORIZED, "유저 없음".to_string())),
    };

    let user_type = jwt_util.get_user_type(&token).unwrap_or_default();

    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(user_type);

    // request extension 에 user_id 를 저장한다.
    //
    // "이 요청은 인증된 요청이고, 그 사용자 ID는 이 값이다"
    // 라는 정보를 request 내부에 심어 두는 것이다.
    //
    // 이후 handler 에서
    // - request extension
    // - 또는 Extension 추출 방식
    // 으로 꺼내 쓸 수 있다.
    request.extensions_mut().insert(user_id);

    // 모든 JWT 검사를 통과했으므로
    // 다음 단계(다음 미들웨어 또는 실제 핸들러)로 요청을 넘긴다.
    //
    // 이때 user_id 가 request 안에 들어간 상태로 전달된다.
    Ok(next.run(request).await)
}
