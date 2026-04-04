use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize,Serialize};

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

//UserInfo 테이블 엔터니
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

