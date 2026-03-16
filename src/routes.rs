use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

// OpenAPI 문서를 생성할 때 쓰는 trait/import다.
use utoipa::OpenApi;

// Swagger UI 화면 설정용 타입들이다.
use utoipa_swagger_ui::Config;
use utoipa_swagger_ui::SwaggerUi;

// 업적 관련 컨트롤러 함수들
use crate::achievement::handler::{check_achievements, get_achievements, get_my_achievements};

// 출석 관련 컨트롤러 함수들
use crate::attendance::handler::{check_in, get_status};

// auth/handler.rs 안의 공개된 함수들을 전부 가져온다.
// sign_up, login, refresh, logout, update_profile 등 인증 관련 핸들러가 여기에 있을 가능성이 크다.
use crate::auth::handler::*;

// JWT 인증 검사 미들웨어
use crate::auth::middleware::jwt_middleware;

// OAuth(카카오/네이버/구글) 로그인 관련 컨트롤러 함수들
use crate::auth::oauth::handler::{google_login, kakao_login, naver_login, oauth_callback};

// 공통 기능 - 이미지 경로 생성 API
use crate::common::handler::create_image_path;

// 런타임 설정값 조회 API
use crate::config::handler::get_runtime_config;
use crate::coupon::handler::{delete_coupon, get_coupon_history, get_my_coupons, use_coupon};

// 이벤트 관련 조회 API
use crate::event::handler::{get_active_events, get_expired_events};
use crate::location::handler::get_nearby_missions;
use crate::membership::handler::{get_history, get_products, purchase};

// 미션 관련 컨트롤러 함수들
use crate::mission::handler::{
    add_bookmark, authenticate, create_mission, delete_mission, get_mission, get_missions,
    get_missions_by_category, get_my_bookmarks, get_my_missions, participate, remove_bookmark,
    update_mission,
};

// 알림 관련 컨트롤러 함수들
use crate::notification::handler::{get_my_notifications, mark_read};

// utoipa로 만든 OpenAPI 문서 정의체
use crate::openapi::ApiDoc;

// 랭킹 관련 컨트롤러 함수들
use crate::ranking::handler::{get_monthly_ranking, get_my_status, get_weekly_ranking};

// 검색 관련 컨트롤러 함수들
use crate::search::handler::{search_category, search_mission, search_store};

// 고객센터 관련 컨트롤러 함수들
use crate::servicecenter::handler::{answer_inquiry, create_inquiry, get_my_inquiries};

// 전체 현황 조회 API
use crate::status::handler::get_total_status;

// 가게 관련 컨트롤러 함수들
use crate::store::handler::{
    create_store, delete_store, get_my_store, get_store, get_store_by_category, update_store,
};

// 내 정보 조회 API
use crate::user::handler::get_me;
use crate::state::AppState;

// 전역 HTTP 라우팅을 구성한다.
// 즉, 어떤 URL + HTTP 메서드가 어떤 핸들러 함수로 연결될지 여기서 정한다.

pub fn create_router() -> Router<AppState> {
    // 인증 없이 접근 가능한(public) API 라우터 묶음이다.
    let public_routes = Router::new()
        // 회원가입 공통 엔드포인트
        .route("/api/auth/signup", post(sign_up))
        // 일반 사용자 회원가입
        .route("/api/auth/signup/user", post(sign_up_user))
        // 점주/가게 회원가입
        .route("/api/auth/signup/store", post(sign_up_store))
        // 로그인
        .route("/api/auth/login", post(login))
        // 리프레시 토큰으로 액세스 토큰 재발급
        .route("/api/auth/refresh", post(refresh))
        // OAuth 콜백 처리
        .route("/api/auth/oauth/callback", get(oauth_callback))
        // 카카오 로그인
        .route("/api/auth/kakao", post(kakao_login))
        // 네이버 로그인
        .route("/api/auth/naver", post(naver_login))
        // 구글 로그인
        .route("/api/auth/google", post(google_login))
        // 진행 중인 이벤트 목록 조회
        .route("/api/events/active", get(get_active_events))
        // 종료된 이벤트 목록 조회
        .route("/api/events/expired", get(get_expired_events))
        // 런타임 설정값 조회
        .route("/api/config/runtime", get(get_runtime_config))
        // 이미지 경로 생성
        .route("/api/image/path", post(create_image_path))
        // 전체 미션 목록 조회
        .route("/api/mission", get(get_missions))
        // 특정 미션 상세 조회
        .route("/api/mission/{mission_id}", get(get_mission))
        // 카테고리별 미션 조회
        .route(
            "/api/mission/category/{category}",
            get(get_missions_by_category),
        )
        // 미션 검색
        .route("/api/search/mission", get(search_mission))
        // 가게 검색
        .route("/api/search/store", get(search_store))
        // 카테고리 검색
        .route("/api/search/category", get(search_category))
        // 특정 가게 조회
        .route("/api/store/{store_id}", get(get_store))
        // 카테고리별 가게 조회
        .route("/api/store/category/{category}", get(get_store_by_category))
        // 위치 기반 주변 미션 조회
        .route("/api/location/missions", get(get_nearby_missions));

    // JWT 인증이 필요한(private) API 라우터 묶음이다.
    let private_routes = Router::new()
        // 프로필 수정
        .route("/api/auth/profile", patch(update_profile))
        // 로그아웃
        .route("/api/auth/logout", post(logout))
        // 사용자 역할 변경
        .route("/api/auth/role", patch(set_user_role))
        // 업적 목록 조회
        .route("/api/achievement", get(get_achievements))
        // 내 업적 조회
        .route("/api/achievement/my", get(get_my_achievements))
        // 업적 달성 체크
        .route("/api/achievement/check", post(check_achievements))
        // 출석 체크
        .route("/api/attendance/check", post(check_in))
        // 출석 상태 조회
        .route("/api/attendance/status", get(get_status))
        // 미션 생성
        .route("/api/mission", post(create_mission))
        // 내가 만든/참여한 미션 조회
        .route("/api/mission/my", get(get_my_missions))
        // 미션 수정
        .route("/api/mission/{mission_id}", patch(update_mission))
        // 미션 삭제
        .route("/api/mission/{mission_id}", delete(delete_mission))
        // 미션 참여
        .route("/api/mission/{mission_id}/participate", post(participate))
        // 참여 인증 처리
        .route(
            "/api/mission/participate/{participate_id}/authenticate",
            post(authenticate),
        )
        // 미션 북마크 추가
        .route("/api/mission/bookmarks/{mission_id}", post(add_bookmark))
        // 미션 북마크 삭제
        .route(
            "/api/mission/bookmarks/{mission_id}",
            delete(remove_bookmark),
        )
        // 내 북마크 목록 조회
        .route("/api/mission/bookmarks", get(get_my_bookmarks))
        // 내 알림 조회
        .route("/api/notification", get(get_my_notifications))
        // 특정 알림 읽음 처리
        .route("/api/notification/{notification_id}/read", post(mark_read))
        // 가게 생성
        .route("/api/store", post(create_store))
        // 가게 수정
        .route("/api/store/{store_id}", patch(update_store))
        // 가게 삭제
        .route("/api/store/{store_id}", delete(delete_store))
        // 내 가게 조회
        .route("/api/store/my", get(get_my_store))
        // 내 사용자 정보 조회
        .route("/api/user/me", get(get_me))
        // 멤버십 상품 목록 조회
        .route("/api/memberships/products", get(get_products))
        // 멤버십 구매
        .route("/api/memberships/purchase/{membership_no}", post(purchase))
        // 멤버십 구매 이력 조회
        .route("/api/memberships/history", get(get_history))
        // 내 쿠폰함 조회
        .route("/api/coupons/myCoupons", get(get_my_coupons))
        // 쿠폰 사용
        .route("/api/coupons/use/{receive_no}", post(use_coupon))
        // 쿠폰 삭제
        .route("/api/coupons/delete/{receive_no}", delete(delete_coupon))
        // 쿠폰 전체 이력 조회
        .route("/api/coupons/history", get(get_coupon_history))
        // 문의 생성
        .route("/api/service-center", post(create_inquiry))
        // 내 문의 목록 조회
        .route("/api/service-center/me", get(get_my_inquiries))
        // 특정 문의에 답변 등록
        .route(
            "/api/service-center/{inquiry_id}/answer",
            post(answer_inquiry),
        )
        // 전체 상태 조회
        .route("/api/status/total", get(get_total_status))
        // 주간 랭킹 조회
        .route("/api/rankings/weekly", get(get_weekly_ranking))
        // 월간 랭킹 조회
        .route("/api/rankings/monthly", get(get_monthly_ranking))
        // 내 랭킹 상태 조회
        .route("/api/rankings/my-status", get(get_my_status))
        // 위에 정의한 private_routes 전체에 JWT 미들웨어를 적용한다.
        // 즉, 이 라우트들은 요청이 들어오면 먼저 jwt_middleware를 통과해야 한다.
        .layer(middleware::from_fn(jwt_middleware));

    // 최종 라우터를 만든다.
    Router::new()
        // 공개 라우트를 합친다.
        .merge(public_routes)
        // 인증 라우트를 합친다.
        .merge(private_routes)
        // Swagger UI 라우트도 합친다.
        .merge(
            // /swagger-ui 경로로 Swagger UI 화면 제공
            SwaggerUi::new("/swagger-ui")
                // OpenAPI json 문서 경로 연결
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                // Swagger UI 설정:
                // 인증 토큰 등을 브라우저에 유지하게 하는 옵션
                .config(Config::from("/api-docs/openapi.json").persist_authorization(true)),
        )
}
