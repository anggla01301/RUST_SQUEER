use crate::auth::model::User;
use std::collections::HashMap;


// OAuth 로그인 과정에서 쓰는 값 객체들이다.

// 공급자별 응답 JSON을 통일된 형태로 변환한 사용자 정보다.
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_id: String,
    pub email: Option<String>,
    pub nickname: Option<String>,
}

// 지원하지 않는 provider 문자열이 들어왔을 때 쓰는 단순 에러 타입이다.
#[derive(Debug)]
pub struct UnsupportedProviderError;

impl OAuthUserInfo {
    // 공급자별 JSON 구조 차이를 여기서 흡수해 공통 형태로 바꾼다.
    pub fn of(
        registration_id: &str,
        attrs: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, UnsupportedProviderError> {
        match registration_id {
            "kakao" => {
                // 카카오는 최상위 `id`, `kakao_account`, `properties`로 값이 나뉜다.
                let provider_id = attrs.get("id").map(|v| v.to_string()).unwrap_or_default();

                let account = attrs.get("kakao_account").and_then(|v| v.as_object());

                let props = attrs.get("properties").and_then(|v| v.as_object());

                let email = account
                    .and_then(|a| a.get("email"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let nickname = props
                    .and_then(|p| p.get("nickname"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Ok(Self {
                    provider: "KAKAO".to_string(),
                    provider_id,
                    email,
                    nickname,
                })
            }

            "naver" => {
                // 네이버는 실제 사용자 정보가 `response` 아래에 들어 있다.
                let resp = attrs.get("response").and_then(|v| v.as_object());

                let provider_id = resp
                    .and_then(|r| r.get("id"))
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                let email = resp
                    .and_then(|r| r.get("email"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let nickname = resp
                    .and_then(|r| r.get("name"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(Self {
                    provider: "NAVER".to_string(),
                    provider_id,
                    email,
                    nickname,
                })
            }

            "google" => {
                // 구글은 표준 OIDC 형태라 필드가 비교적 평평하다.
                let provider_id = attrs.get("sub").map(|v| v.to_string()).unwrap_or_default();

                let email = attrs
                    .get("email")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let nickname = attrs
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(Self {
                    provider: "GOOGLE".to_string(),
                    provider_id,
                    email,
                    nickname,
                })
            }
            _ => Err(UnsupportedProviderError),
        }
    }
}

// 인증 완료 후 애플리케이션 내부에서 들고 다니는 principal이다.
#[derive(Debug, Clone)]
pub struct OAuthPrincipal {
    pub user: User,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl OAuthPrincipal {
    pub fn new(user: User, attributes: HashMap<String, serde_json::Value>) -> Self {
        Self { user, attributes }
    }

    // Spring Security의 authority 문자열과 비슷한 형태를 만든다.
    pub fn get_role(&self) -> String {
        format!("ROLE_{}", self.user.user_type)
    }

    // 공급자 내부에서 쓰는 사용자 식별자를 이름처럼 노출한다.
    pub fn get_name(&self) -> Option<&str> {
        self.user.provider_id.as_deref()
    }
}


