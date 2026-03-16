use chrono::{NaiveDate, NaiveDateTime};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// 인증 도메인에서 사용하는 엔티티와 DTO 모음이다.

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: Option<i64>,
    pub user_name: String,
    pub user_nickname: String,
    pub user_email: Option<String>,
    pub user_password: String,
    pub user_number: Option<String>,
    pub user_avatar: Option<String>,
    pub user_is_active: String,
    pub user_joindate: Option<NaiveDateTime>,
    pub user_updatedate: Option<NaiveDateTime>,
    pub user_type: String,
    pub user_withdraw_date: Option<NaiveDateTime>,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
    pub refresh_token: Option<String>,
}

impl User {
    // 일반/스토어 계정 생성 시 공통 기본값을 채운다.
    pub fn new(
        user_name: String,
        user_nickname: String,
        user_password: String,
        user_type: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            user_id: None,
            user_name,
            user_nickname,
            user_email: None,
            user_password,
            user_number: None,
            user_avatar: None,
            user_is_active: "Y".to_string(),
            user_joindate: Some(now),
            user_updatedate: Some(now),
            user_type,
            user_withdraw_date: None,
            provider: None,
            provider_id: None,
            refresh_token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInfo {
    pub user_id: i64,
    pub user_info_level: i32,
    pub user_info_exp: i32,
    pub user_info_point: i32,
    pub user_info_mission_do: i32,
    pub user_info_mission_make: i32,
    pub user_info_attend: i32,
    pub user_info_attend_straight: i32,
    pub user_info_attend_max: i32,
    pub last_attend_date: Option<NaiveDate>,
    pub temp_mission_people: i32,
    pub temp_exp_multiplier: i32,
}

impl UserInfo {
    pub fn for_normal_user(user_id: i64) -> Self {
        Self {
            user_id,
            user_info_level: 1,
            user_info_exp: 0,
            user_info_point: 0,
            user_info_mission_do: 10,
            user_info_mission_make: 0,
            user_info_attend: 0,
            user_info_attend_straight: 0,
            user_info_attend_max: 0,
            last_attend_date: None,
            temp_mission_people: 15,
            temp_exp_multiplier: 1,
        }
    }

    pub fn for_store_user(user_id: i64) -> Self {
        Self {
            user_id,
            user_info_level: 1,
            user_info_exp: 0,
            user_info_point: 0,
            user_info_mission_do: 0,
            user_info_mission_make: 3,
            user_info_attend: 0,
            user_info_attend_straight: 0,
            user_info_attend_max: 0,
            last_attend_date: None,
            temp_mission_people: 15,
            temp_exp_multiplier: 1,
        }
    }

    pub fn for_oauth_user(user_id: i64) -> Self {
        Self::for_normal_user(user_id)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequestDto {
    #[serde(rename = "userEmail")]
    pub user_email: String,
    #[serde(rename = "userPassword")]
    pub user_password: String,
}

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

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserSignUpRequestDto {
    #[serde(rename = "userEmail")]
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    pub user_email: String,
    #[serde(rename = "userName")]
    #[validate(length(
        min = 2,
        max = 50,
        message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"
    ))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(
        path = "*PASSWORD_REGEX",
        message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다."
    ))]
    pub user_password: String,
    #[serde(rename = "userNickname")]
    #[validate(length(
        min = 2,
        max = 20,
        message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"
    ))]
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
    #[validate(length(
        min = 2,
        max = 50,
        message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"
    ))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(
        path = "*PASSWORD_REGEX",
        message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다"
    ))]
    pub user_password: String,
    #[serde(rename = "userNickname")]
    #[validate(length(
        min = 2,
        max = 20,
        message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"
    ))]
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
    #[validate(length(
        min = 2,
        max = 50,
        message = "이름은 최소 2자 이상, 최대 50자 이하로 입력해주세요"
    ))]
    pub user_name: String,
    #[serde(rename = "userPassword")]
    #[validate(regex(
        path = "*PASSWORD_REGEX",
        message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다"
    ))]
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

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserProfileUpdateDto {
    #[serde(rename = "userNickname")]
    #[validate(length(
        min = 2,
        max = 20,
        message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"
    ))]
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

static PASSWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?=.*[A-Z])(?=.*\d)(?=.*[!@#$%^&*]).{8,}$").unwrap());

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^01[0-9]-\d{3,4}-\d{4}$").unwrap());
