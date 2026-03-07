use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use validator::Validate;

//@Entity+@Table(name="USER_")
//@Getter @Setter #NoArgsConstructor @AllArgsConstructor @Builder->derive로 대체
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]

pub struct User {
    //@Id @GeneratedValue(SEQUENCE)->DB가 자동 생성, Option<i64>로 선언
    pub user_id: Option<i64>,

    //@Column(nullable=false, unique=true, length=20)
    pub user_nickname: String,

    //@Column(unique=true, length=100)
    pub user_email: Option<String>,

    //@Column(nullable=false, length=100)
    pub user_password: String,

    //@Column(unique=true, length=13)
    pub user_number: Option<String>,

    //@Column(length=500)
    pub user_avatar: Option<String>,

    //@Builder.default="Y"
    pub user_is_active: String,

    //@Column(updatable=false)->@Prepersist에서 설정
    pub user_joindate: Option<NaiveDateTime>,

    pub user_updatedate: Option<NaiveDateTime>,

    //@Column(updatable=false, length=10)
    pub user_type: String,

    pub user_withdraw_date: Option<NaiveDateTime>,

    //OAuth2
    pub provider: Option<String>,
    pub provider_id: Option<String>,

    pub refresh_token: Option<String>,
    //@OnetoOne은 나중에 연관관계 구현할 때 추가
    //pub user_info:: Option<UserInfo>,
}

impl User {
    //@NoArgsConstructor + @Builder.Default 대응
    pub fn new(user_nickname: String, user_password: String, user_type: String) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            user_id: None,
            user_nickname,
            user_email: None,
            user_password,
            user_number: None,
            user_avatar: None,
            user_is_active: "Y".to_string(), //@Builder.Default
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

//UserInfo
#[derive(Debug, Clone, Serialize, Deserialize,sqlx::FromRow)]
pub struct UserInfo {
    //@Id @MapsId->User의 PK를 그대로 공유
    pub user_id: i64,

    //@OneToOne->일단 주석 처리(연관관계는 나중에)
    //pub user: User,
    pub user_info_level: i32,
    pub user_info_exp: i32,
    pub user_info_point: i32,
    pub user_info_mission_do: i32,
    pub user_info_mission_make: i32,

    //@Builder.Default=0
    pub user_info_attend: i32,
    pub user_info_attend_straight: i32,
    pub user_info_attend_max: i32,

    pub last_attend_date: Option<NaiveDate>,

    //@Builder.Default=15
    pub temp_mission_people: i32,
    //@Builder.Default=1
    pub temp_exp_multiplier: i32,
}

impl UserInfo {
    //createForNormalUser 팩토리 메소드 대응
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

    //createForStoreUser 팩토리 메소드 대응
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
    //createForOAuthUser 팩토리 메소드 대응
    pub fn for_oauth_user(user_id: i64) -> Self {
        Self::for_normal_user(user_id) //일반 유저랑 동일
    }
}
//LoginRequestDTO
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequestDto {
    pub user_email: String,
    pub user_password: String,
}

//LoginResponseDTO
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponseDto {
    pub token: String,
    pub user_id: i64,
    pub user_nickname: String,
    pub user_email: String,
    pub user_type: String,
    pub user_info_level: i32,
    pub user_info_point: i32,

    //@JsonIgnore->skip_serializing
    //Json 응답에서 제외됨
    #[serde(skip_serializing)]
    pub refresh_token: String,
}

//UserSignUpRequestDTO
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserSignUpRequestDto {
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    #[validate(length(min = 1, message = "이메일은 필수입니다"))]
    pub user_email: String,

    //@Pattern->regex
    #[validate(regex(
        path="*PASSWORD_REGEX",
        message="비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다."
    ))]
    pub user_password: String,

    //@Size(min=2, max=20)
    #[validate(length(
        min = 2,
        max = 20,
        message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"
    ))]
    pub user_nickname: String,

    #[validate(regex(
    path="*PHONE_REGEX",
    message="올바른 전화번호 형식이 아닙니다"
    ))]
    pub user_number: String,
}

//StoreSignUpRequestDto
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoreSignUpRequestDto {
    #[validate(email(message = "이메일 형식이 아닙니다"))]
    pub user_email: String,

    #[validate(regex(
        path = "*PASSWORD_REGEX",
        message = "비밀번호는 8자 이상, 영문 대문자/숫자/특수문자를 각각 최소 한개를 포함해야 합니다"
    ))]
    pub user_password: String,

    #[validate(length(min = 2, max = 20, message = "닉네임은 최소 2자 이상, 최대 20자 이하로 입력해주세요"))]
    pub user_nickname: String,

    #[validate(regex(
        path = "*PHONE_REGEX",
        message = "올바른 전화번호 형식이 아닙니다"

    ))]
    pub user_number: String,

    #[validate(length(min = 1, message = "가게 이름은 필수입니다"))]
    pub store_name: String,

    #[validate(length(min = 1, message = "가게 카테고리는 필수입니다"))]
    pub store_category: String,

    //BigDecimal->f64
    pub store_latitude: f64,
    pub store_longitude: f64,

}


//정규식 상수
//@Pattern regexp 대응
use once_cell::sync::Lazy;
use regex::Regex;

static PASSWORD_REGEX: Lazy<Regex> = Lazy::new(||{
    Regex::new(r"^(?=.*[A-Z])(?=.*\d)(?=.*[!@#$%^&*]).{8,}$").unwrap()
});

static PHONE_REGEX: Lazy<Regex>=Lazy::new(||{
    Regex::new(r"^01[0-9]-\d{3,4}-\d{4}$").unwrap()
});








