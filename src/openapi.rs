use utoipa::openapi::security::{
    ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme,
};
use utoipa::openapi::Components;
use utoipa::{Modify, OpenApi};

use crate::auth::model::{
    LoginRequestDto, LoginResponseDto, RefreshResponse, SetUserRoleRequest, StoreSignUpRequestDto,
    TotalSignUpRequestDto, UserProfileUpdateDto, UserProfileUpdateResponseDto,
    UserSignUpRequestDto,
};
use crate::auth::oauth::model::SocialLoginRequest;

// Swagger/OpenAPI 문서에 노출할 엔드포인트와 스키마를 모아 둔 설정 파일이다.

const BEARER_AUTH_SCHEME: &str = "bearer_auth";
const USER_TYPE_AUTH_SCHEME: &str = "user_type_auth";

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.add_security_scheme(
            BEARER_AUTH_SCHEME,
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "로그인 후 발급받은 access token을 Bearer 형식으로 입력합니다.",
                    ))
                    .build(),
            ),
        );
        components.add_security_scheme(
            USER_TYPE_AUTH_SCHEME,
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "X-User-Type",
                "테스트할 권한 값을 입력합니다. 예: USER, STORE, PENDING",
            ))),
        );

        let security = SecurityRequirement::new(BEARER_AUTH_SCHEME, Vec::<String>::new())
            .add(USER_TYPE_AUTH_SCHEME, Vec::<String>::new());

        for target in ["/api/auth/profile", "/api/auth/role", "/api/auth/logout"] {
            if let Some(path_item) = openapi.paths.paths.get_mut(target) {
                for operation in [
                    path_item.get.as_mut(),
                    path_item.put.as_mut(),
                    path_item.post.as_mut(),
                    path_item.delete.as_mut(),
                    path_item.options.as_mut(),
                    path_item.head.as_mut(),
                    path_item.patch.as_mut(),
                    path_item.trace.as_mut(),
                ]
                .into_iter()
                .flatten()
                {
                    operation.security = Some(vec![security.clone()]);
                }
            }
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::handler::sign_up,
        crate::auth::handler::sign_up_user,
        crate::auth::handler::sign_up_store,
        crate::auth::handler::login,
        crate::auth::handler::update_profile,
        crate::auth::handler::set_user_role,
        crate::auth::handler::logout,
        crate::auth::handler::refresh,
        crate::auth::oauth::handler::oauth_callback,
        crate::auth::oauth::handler::kakao_login,
        crate::auth::oauth::handler::naver_login,
        crate::auth::oauth::handler::google_login
    ),
    components(schemas(
        LoginRequestDto,
        LoginResponseDto,
        UserSignUpRequestDto,
        StoreSignUpRequestDto,
        TotalSignUpRequestDto,
        UserProfileUpdateDto,
        UserProfileUpdateResponseDto,
        SetUserRoleRequest,
        RefreshResponse,
        SocialLoginRequest
    )),
    modifiers(&SecurityAddon),
    info(
        title = "Squeer API",
        version = "1.0.0",
        description = "API documentation"
    )
)]
pub struct ApiDoc;
