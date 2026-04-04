// ============================================================
// auth/dto.rs — 요청/응답 DTO 구조체만 모아둔다.
// DB 엔티티는 model.rs 에 있다.
// ============================================================

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize,Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ── 로그인 ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequestDto{
    #[serde(rename = "userEmail")]
    pub user_email: String,
    #[serde(rename = "userPassword")]
    pub user_password: String,
}

/*
// Java (Jackson 사용 시)
public class AuthResponse {
    @JsonProperty("accessToken")
    public String token;
}
*/

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponseDto {
    #[serde(rename = "accessToken")]
    pub token: String,
    #[serde(skip_serializing)]
    pub refresh_token: String,
    #[serde(rename = "isNewUser")]
    pub is_new_user: bool,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: String,
    #[serde(rename = "userEmail")]
    pub user_email: String,
    #[serde(rename = "userInfoLevel")]
    pub user_info_level: i32,
    #[serde(rename = "userInfoPoint")]
    pub user_info_point: i32,
}

// ── 회원가입 ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserSignUpRequestDto {
    #[serde(rename = "userEmail")]
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    pub user_email: String,
    #[serde(rename = "userName")]
    #[validate(length(min = 2, max = 50, message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(path = "*PASSWORD_REGEX", message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다."))]
    pub user_password: String,
    #[serde(rename = "userNickname")]
    #[validate(length(min = 2, max = 20, message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"))]
    pub user_nickname: String,
    #[serde(rename = "userNumber")]
    #[validate(regex(path = "*PHONE_REGEX", message = "올바른 전화번호 형식이 아닙니다"))]
    pub user_number: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct StoreSignUpRequestDto {
    #[serde(rename = "userEmail")]
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    pub user_email: String,
    #[serde(rename = "userName")]
    #[validate(length(min = 2, max = 50, message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(path = "*PASSWORD_REGEX", message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다"))]
    pub user_password: String,
    #[serde(rename = "userNickname")]
    #[validate(length(min = 2, max = 20, message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"))]
    pub user_nickname: String,
    #[serde(rename = "userNumber")]
    #[validate(regex(path = "*PHONE_REGEX", message = "올바른 전화번호 형식이 아닙니다"))]
    pub user_number: String,
    #[serde(rename = "storeName")]
    #[validate(length(min = 1, message = "가게 이름은 필수입니다"))]
    pub store_name: String,
    #[serde(rename = "storeCategory")]
    #[validate(length(min = 1, message = "가게 카테고리는 필수입니다"))]
    pub store_category: String,
    #[serde(rename = "storeLatitude")]
    pub store_latitude: f64,
    #[serde(rename = "storeLongitude")]
    pub store_longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct TotalSignUpRequestDto {
    #[serde(rename = "userEmail")]
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    pub user_email: String,
    #[serde(rename = "userName")]
    #[validate(length(min = 2, max = 50, message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(path = "*PASSWORD_REGEX", message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다"))]
    pub user_password: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: Option<String>,
    #[serde(rename = "userNumber")]
    #[validate(regex(path = "*PHONE_REGEX", message = "올바른 전화번호 형식이 아닙니다"))]
    pub user_number: String,
    #[serde(rename = "userType")]
    pub user_type: Option<String>,
    #[serde(rename = "storeName")]
    pub store_name: Option<String>,
    #[serde(rename = "storeCategory")]
    pub store_category: Option<String>,
    #[serde(rename = "storeLatitude")]
    pub store_latitude: Option<f64>,
    #[serde(rename = "storeLongitude")]
    pub store_longitude: Option<f64>,
}

// ── 프로필 수정 ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserProfileUpdateDto {
    #[serde(rename = "userNickname")]
    #[validate(length(min = 2, max = 20, message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"))]
    pub user_nickname: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "storeName")]
    pub store_name: Option<String>,
    #[serde(rename = "storeCategory")]
    pub store_category: Option<String>,
    #[serde(rename = "storeLatitude")]
    pub store_latitude: Option<f64>,
    #[serde(rename = "storeLongitude")]
    pub store_longitude: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfileUpdateResponseDto {
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

// ── 기타 ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetUserRoleRequest {
    #[serde(rename = "userType")]
    pub user_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

// ── 정규식 (DTO 유효성 검사에서만 쓰이므로 dto.rs 에 둔다) ───
pub static PASSWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?=.*[A-Z])(?=.*\d)(?=.*[!@#$%^&*]).{8,}$").unwrap());

pub static PHONE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^01[0-9]-\d{3,4}-\d{4}$").unwrap());