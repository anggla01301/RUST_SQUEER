use super::model::{OAuthPrincipal, OAuthUserInfo};
use crate::auth::jwt::JwtUtil;
use crate::auth::model::{User, UserInfo};
use crate::auth::repository::{UserInfoRepository, UserRepository};
use anyhow::Context;
use reqwest::Client;
use std::collections::HashMap;
use std::env;

// OAuth 비즈니스 로직을 담당한다.
// 공급자 사용자 정보 조회, 최초 가입 처리, 내부 JWT 발급 흐름이 여기에 모여 있다.

#[derive(Clone)]
pub struct OAuthService {
    user_repository: UserRepository,
    user_info_repository: UserInfoRepository,
    http_client: Client,
}

#[derive(Debug)]
pub enum OAuthError {
    UnsupportedProvider,
    ApiCallFailed,
    DbError,
    InvalidRedirectUri,
}

// OAuth 로그인 성공 후 컨트롤러가 바로 응답으로 바꿀 데이터 묶음이다.
pub struct OAuthSuccessResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub redirect_uri: String,
    pub max_age: i64,
    // 응답 바디에 넣을 최소 사용자 정보다.
    pub user_id: i64,
    pub user_name: String,
    pub user_nickname: String,
    pub user_email: String,
    pub user_type: String,
}

impl OAuthService {
    pub fn new(user_repository: UserRepository, user_info_repository: UserInfoRepository) -> Self {
        Self {
            user_repository,
            user_info_repository,
            http_client: Client::new(),
        }
    }

    // 공급자 access token으로 사용자 정보를 읽고, 필요하면 내부 계정을 생성한다.
    pub async fn load_user(
        &self,
        registration_id: &str,
        access_token: &str,
        user_type: Option<String>,
    ) -> Result<OAuthPrincipal, OAuthError> {
        // 공급자 API 응답 JSON을 먼저 가져온다.
        let attrs = self.fetch_user_info(registration_id, access_token).await?;

        // 공급자마다 다른 JSON 구조를 공통 사용자 정보로 정규화한다.
        let info = OAuthUserInfo::of(registration_id, &attrs)
            .map_err(|_| OAuthError::UnsupportedProvider)?;

        let user_type = user_type.unwrap_or_else(|| "USER".to_string());

        // 이미 가입된 소셜 계정이면 그대로 사용하고, 없으면 신규 가입시킨다.
        let user = match self
            .user_repository
            .find_by_provider_and_provider_id(&info.provider, &info.provider_id)
            .await
        {
            Some(existing_user) => existing_user,

            None => {
                let now = chrono::Local::now().naive_local();
                // 일반 비밀번호 로그인 계정이 아니므로 패스워드는 더미 문자열로 채운다.
                let new_user = User {
                    user_id: None,
                    user_name: info
                        .nickname
                        .clone()
                        .unwrap_or_else(|| "OAuth User".to_string()),
                    provider: Some(info.provider.clone()),
                    provider_id: Some(info.provider_id.clone()),
                    user_email: info.email.clone(),
                    user_nickname: info.nickname.clone().unwrap_or_default(),
                    user_type: user_type.clone(),
                    user_password: "OAUTH_USER".to_string(),
                    user_number: None,
                    user_avatar: None,
                    user_is_active: "Y".to_string(),
                    user_joindate: Some(now),
                    user_updatedate: Some(now),
                    user_withdraw_date: None,
                    refresh_token: None,
                };

                let saved_user = self
                    .user_repository
                    .save(&new_user)
                    .await
                    .ok_or(OAuthError::DbError)?;

                let saved_user_id = saved_user.user_id.ok_or(OAuthError::DbError)?;

                // OAuth 계정도 내부 로직상 USER_INFO는 반드시 있어야 한다.
                let user_info = if user_type == "STORE" {
                    UserInfo::for_store_user(saved_user_id)
                } else {
                    UserInfo::for_oauth_user(saved_user_id)
                };

                self.user_info_repository
                    .save(user_info)
                    .await
                    .ok_or(OAuthError::DbError)?;

                saved_user
            }
        };

        Ok(OAuthPrincipal::new(user, attrs))
    }

    // 공급자 API 호출만 담당하는 내부 helper다.
    async fn fetch_user_info(
        &self,
        registration_id: &str,
        access_token: &str,
    ) -> Result<HashMap<String, serde_json::Value>, OAuthError> {
        // provider 문자열에 따라 사용자 정보 endpoint가 달라진다.
        let url = match registration_id {
            "kakao" => "https://kapi.kakao.com/v2/user/me",
            "naver" => "https://openapi.naver.com/v1/nid/me",
            "google" => "https://www.googleapis.com/oauth2/v3/userinfo",
            _ => return Err(OAuthError::UnsupportedProvider),
        };

        let response = self
            .http_client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| OAuthError::ApiCallFailed)?
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .map_err(|_| OAuthError::ApiCallFailed)?;

        Ok(response)
    }

    // 내부 User를 기준으로 우리 서비스의 JWT와 리다이렉트 정보를 만든다.
    pub async fn handle_oauth_success(
        &self,
        user: User,
    ) -> Result<OAuthSuccessResponse, OAuthError> {
        // OAuth 로그인도 최종적으로는 일반 로그인과 같은 JWT 체계를 사용한다.
        let jwt_util = JwtUtil::new().map_err(|err| {
            tracing::error!("JWT 설정 오류: {err:#}");
            OAuthError::DbError
        })?;

        let user_id = user.user_id.ok_or(OAuthError::DbError)?;
        let user_email = user.user_email.as_deref().unwrap_or("");
        let user_type = &user.user_type;

        let access_token = jwt_util.generate_token(user_id, user_email, user_type);
        let refresh_token = jwt_util.generate_refresh_token(user_id);

        self.user_repository
            .update_refresh_token(user_id, &refresh_token)
            .await;

        // 리다이렉트 허용 범위를 제한해 오픈 리다이렉트 실수를 막는다.
        let redirect_uri = env::var("OAUTH2_REDIRECT_URI").map_err(|err| {
            tracing::error!("OAUTH2_REDIRECT_URI 없음: {err}");
            OAuthError::InvalidRedirectUri
        })?;

        if !redirect_uri.starts_with("https://") && !redirect_uri.starts_with("http://localhost") {
            return Err(OAuthError::InvalidRedirectUri);
        }

        let refresh_expiration = env::var("JWT_REFRESH_EXPIRATION")
            .unwrap_or("604800000".to_string())
            .parse::<i64>()
            .unwrap_or(604800000);

        let max_age = refresh_expiration / 1000;

        Ok(OAuthSuccessResponse {
            access_token,
            refresh_token,
            redirect_uri,
            max_age,
            user_id,
            user_name: user.user_name.clone(),
            user_nickname: user.user_nickname.clone(),
            user_email: user.user_email.clone().unwrap_or_default(),
            user_type: user.user_type.clone(),
        })
    }

    // 인가 코드를 공급자 access token으로 교환한다.
    pub async fn exchange_code(
        &self,
        registration_id: &str,
        code: &str,
    ) -> Result<String, OAuthError> {
        // 공급자마다 토큰 endpoint와 환경 변수 키가 다르다.
        let (token_url, client_id_key, client_secret_key, redirect_uri_key) = match registration_id
        {
            "kakao" => (
                "https://kauth.kakao.com/oauth/token",
                "KAKAO_CLIENT_ID",
                "KAKAO_CLIENT_SECRET",
                "KAKAO_REDIRECT_URI",
            ),
            "naver" => (
                "https://nid.naver.com/oauth2.0/token",
                "NAVER_CLIENT_ID",
                "NAVER_CLIENT_SECRET",
                "NAVER_REDIRECT_URI",
            ),
            "google" => (
                "https://oauth2.googleapis.com/token",
                "GOOGLE_CLIENT_ID",
                "GOOGLE_CLIENT_SECRET",
                "GOOGLE_REDIRECT_URI",
            ),
            _ => return Err(OAuthError::UnsupportedProvider),
        };

        // 설정 누락은 런타임 오류로 남기기보다 명확한 에러 로그를 남긴다.
        let client_id = env::var(client_id_key)
            .with_context(|| format!("{client_id_key} 없음"))
            .map_err(|err| {
                tracing::error!("{err:#}");
                OAuthError::ApiCallFailed
            })?;
        let client_secret = env::var(client_secret_key)
            .with_context(|| format!("{client_secret_key} 없음"))
            .map_err(|err| {
                tracing::error!("{err:#}");
                OAuthError::ApiCallFailed
            })?;
        let redirect_uri = env::var(redirect_uri_key)
            .with_context(|| format!("{redirect_uri_key} 없음"))
            .map_err(|err| {
                tracing::error!("{err:#}");
                OAuthError::ApiCallFailed
            })?;

        // OAuth 표준 폼 파라미터로 토큰 교환 요청을 보낸다.
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
        ];

        let response: serde_json::Value = self
            .http_client
            .post(token_url)
            .form(&params)
            .send()
            .await
            .map_err(|_| OAuthError::ApiCallFailed)?
            .json::<serde_json::Value>()
            .await
            .map_err(|_| OAuthError::ApiCallFailed)?;

        // 각 공급자가 내려주는 JSON에서 access_token 문자열만 추출한다.
        response["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(OAuthError::ApiCallFailed)
    }
}
