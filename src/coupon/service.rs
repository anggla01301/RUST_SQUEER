//! 쿠폰 발급과 사용 처리 로직을 구현할 서비스 파일이다.
use chrono::{Local, TimeDelta};
use sqlx::PgPool;

use crate::auth::model::UserInfo;
use crate::coupon::model::{
    CouponHistoryResponseDto, CouponReceiveDetail, CouponResponseDto, CouponUseResponseDto,
};

#[derive(Clone)]
pub struct CouponService {
    pool: PgPool,
}

impl CouponService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_my_available_coupons(&self, user_id: i64) -> Vec<CouponResponseDto> {
        sqlx::query_as::<_, CouponResponseDto>(
            "SELECT
                cr.RECEIVEID as receive_id,
                c.COUPON_NAME as coupon_name,
                cr.EXPIREDAT as expired_at
            FROM COUPON_RECEIVE cr
            JOIN COUPON c ON c.COUPON_ID = cr.COUPON_ID
            WHERE cr.USER_ID = $1
              AND cr.USED = 0
              AND cr.EXPIREDAT > $2
            ORDER BY cr.EXPIREDAT ASC, cr.ISSUEDAT DESC",
        )
        .bind(user_id)
        .bind(Local::now().naive_local() + TimeDelta::minutes(1))
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn use_coupon_and_extract(
        &self,
        user_id: i64,
        receive_no: i64,
        mission_id: Option<i64>,
    ) -> Option<CouponUseResponseDto> {
        let coupon = self.find_valid_coupon(user_id, receive_no).await?;
        let user_type = self.find_user_type(user_id).await?;
        let user_info = self.find_user_info(user_id).await?;
        let value = extract_number(&coupon.coupon_name);

        let mut mission_people = user_info.temp_mission_people;
        let mut exp_multiplier = user_info.temp_exp_multiplier;
        let mut can_pull_up = 0;

        let mut tx = self.pool.begin().await.ok()?;

        if user_type == "STORE" {
            if coupon.coupon_name.contains("확장") {
                if user_info.temp_mission_people > 15 || value <= 0 {
                    return None;
                }
                mission_people = value;
                sqlx::query("UPDATE USER_INFO SET TEMP_MISSION_PEOPLE = $1 WHERE USER_ID = $2")
                    .bind(mission_people)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await
                    .ok()?;
            } else if coupon.coupon_name.contains("끌올") {
                let mission_id = mission_id?;
                let owned: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*)
                    FROM MISSION m
                    JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
                    WHERE m.MISSION_ID = $1 AND s.USER_ID = $2",
                )
                .bind(mission_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .unwrap_or(0);
                if owned == 0 {
                    return None;
                }
                sqlx::query("UPDATE MISSION SET IS_PULL_UP = 1 WHERE MISSION_ID = $1")
                    .bind(mission_id)
                    .execute(&mut *tx)
                    .await
                    .ok()?;
                can_pull_up = 1;
            } else {
                return None;
            }
        } else if user_type == "USER" {
            if !coupon.coupon_name.contains("경험치")
                || user_info.temp_exp_multiplier > 1
                || value <= 0
            {
                return None;
            }
            exp_multiplier = value;
            sqlx::query("UPDATE USER_INFO SET TEMP_EXP_MULTIPLIER = $1 WHERE USER_ID = $2")
                .bind(exp_multiplier)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .ok()?;
        } else {
            return None;
        }

        sqlx::query(
            "UPDATE COUPON_RECEIVE
            SET USED = 1
            WHERE RECEIVEID = $1 AND USER_ID = $2",
        )
        .bind(receive_no)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .ok()?;

        tx.commit().await.ok()?;

        Some(CouponUseResponseDto {
            status: "SUCCESS".to_string(),
            user_type,
            coupon_name: coupon.coupon_name,
            mission_people,
            exp_multiplier,
            can_pull_up,
        })
    }

    pub async fn delete_coupon(&self, user_id: i64, receive_no: i64) -> bool {
        if self.find_valid_coupon(user_id, receive_no).await.is_none() {
            return false;
        }

        sqlx::query(
            "UPDATE COUPON_RECEIVE
            SET USED = 2
            WHERE RECEIVEID = $1 AND USER_ID = $2",
        )
        .bind(receive_no)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() > 0)
        .unwrap_or(false)
    }

    pub async fn get_all_coupon_history(&self, user_id: i64) -> Vec<CouponHistoryResponseDto> {
        sqlx::query_as::<_, CouponHistoryResponseDto>(
            "SELECT
                cr.RECEIVEID as receive_no,
                c.COUPON_NAME as coupon_name,
                cr.USED as used,
                cr.ISSUEDAT as issued_at,
                cr.EXPIREDAT as expired_at
            FROM COUPON_RECEIVE cr
            JOIN COUPON c ON c.COUPON_ID = cr.COUPON_ID
            WHERE cr.USER_ID = $1
            ORDER BY cr.ISSUEDAT DESC NULLS LAST",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    async fn find_valid_coupon(
        &self,
        user_id: i64,
        receive_no: i64,
    ) -> Option<CouponReceiveDetail> {
        let coupon = sqlx::query_as::<_, CouponReceiveDetail>(
            "SELECT
                cr.RECEIVEID as receive_id,
                cr.USED as used,
                cr.DISPLAY_NAME as display_name,
                cr.RECEIVEPATH as receive_path,
                cr.ISSUEDAT as issued_at,
                cr.EXPIREDAT as expired_at,
                c.COUPON_ID as coupon_id,
                c.COUPON_NAME as coupon_name,
                cr.USER_ID as user_id
            FROM COUPON_RECEIVE cr
            JOIN COUPON c ON c.COUPON_ID = cr.COUPON_ID
            WHERE cr.RECEIVEID = $1 AND cr.USER_ID = $2",
        )
        .bind(receive_no)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;

        if coupon.used != 0 || coupon.expired_at <= Local::now().naive_local() {
            return None;
        }

        Some(coupon)
    }

    async fn find_user_type(&self, user_id: i64) -> Option<String> {
        sqlx::query_scalar("SELECT USER_TYPE FROM USER_ WHERE USER_ID = $1")
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

fn extract_number(text: &str) -> i32 {
    let digits = text
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse::<i32>().unwrap_or(0)
}
