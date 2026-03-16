use chrono::Local;
use sqlx::PgPool;

use crate::auth::model::{User, UserInfo};
use crate::status::model::TotalStatusResponseDto;

// 메인 상태 조회 서비스다.
#[derive(Clone)]
pub struct StatusService {
    pool: PgPool,
}

impl StatusService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_user_total_status(&self, user_id: i64) -> Option<TotalStatusResponseDto> {
        let user = self.find_user(user_id).await?;
        let ui = self.find_user_info(user_id).await?;
        let is_attended = ui.last_attend_date == Some(Local::now().date_naive());

        let has_pull_up: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM MISSION m
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE s.USER_ID = $1 AND COALESCE(m.IS_PULL_UP, 'N') = 'Y'",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Some(TotalStatusResponseDto {
            user_type: user.user_type,
            user_info_point: ui.user_info_point,
            user_info_mission_make: ui.user_info_mission_make,
            user_info_mission_do: ui.user_info_mission_do,
            temp_mission_people: ui.temp_mission_people,
            temp_exp_multiplier: ui.temp_exp_multiplier,
            user_info_attend_straight: ui.user_info_attend_straight,
            user_info_attend: ui.user_info_attend,
            user_info_attend_max: ui.user_info_attend_max,
            attendance_status: if is_attended { "DONE" } else { "READY" }.to_string(),
            is_pull_up_active: if has_pull_up > 0 { 1 } else { 0 },
        })
    }

    async fn find_user(&self, user_id: i64) -> Option<User> {
        sqlx::query_as::<_, User>(
            "SELECT
                USER_ID as user_id,
                USER_NAME as user_name,
                USER_NICKNAME as user_nickname,
                USER_EMAIL as user_email,
                USER_PASSWORD as user_password,
                USER_NUMBER as user_number,
                USER_AVATAR as user_avatar,
                USER_IS_ACTIVE as user_is_active,
                USER_JOINDATE as user_joindate,
                USER_UPDATEDATE as user_updatedate,
                USER_TYPE as user_type,
                USER_WITHDRAW_DATE as user_withdraw_date,
                PROVIDER as provider,
                PROVIDER_ID as provider_id,
                REFRESH_TOKEN as refresh_token
            FROM USER_
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_user_info(&self, user_id: i64) -> Option<UserInfo> {
        sqlx::query_as::<_, UserInfo>(
            "SELECT
                USER_ID as user_id,
                USER_INFO_LEVEL as user_info_level,
                USER_INFO_EXP as user_info_exp,
                USER_INFO_POINT as user_info_point,
                USER_INFO_MISSION_DO as user_info_mission_do,
                USER_INFO_MISSION_MAKE as user_info_mission_make,
                USER_INFO_ATTEND as user_info_attend,
                USER_INFO_ATTEND_STRAIGHT as user_info_attend_straight,
                USER_INFO_ATTEND_MAX as user_info_attend_max,
                LAST_ATTEND_DATE as last_attend_date,
                TEMP_MISSION_PEOPLE as temp_mission_people,
                TEMP_EXP_MULTIPLIER as temp_exp_multiplier
            FROM USER_INFO
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }
}
