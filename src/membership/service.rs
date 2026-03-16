//! 멤버십 규칙과 혜택 계산 로직을 구현할 서비스 파일이다.
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use sqlx::PgPool;

use crate::auth::model::{User, UserInfo};
use crate::membership::model::{
    Membership, MembershipHistoryDto, MembershipPayment, MembershipPaymentDto,
};

#[derive(Clone)]
pub struct MembershipService {
    pool: PgPool,
}

impl MembershipService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_membership_products(&self, user_id: i64) -> Option<Vec<Membership>> {
        let user = self.find_user(user_id).await?;

        Some(
            sqlx::query_as::<_, Membership>(
                "SELECT
                    MEMBERSHIPID as membership_id,
                    MEMBERSHIPNAME as membership_name,
                    PRICE as price,
                    BENEFITS as benefits
                FROM MEMBERSHIP
                WHERE MEMBERSHIPNAME LIKE '%' || $1 || '%'
                ORDER BY MEMBERSHIPID",
            )
            .bind(&user.user_type)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default(),
        )
    }

    pub async fn purchase(&self, user_id: i64, membership_no: i64) -> Option<MembershipPaymentDto> {
        let user = self.find_user(user_id).await?;
        let user_info = self.find_user_info(user_id).await?;
        let membership = self.find_membership(membership_no).await?;

        if !membership.membership_name.contains(&user.user_type) {
            return None;
        }
        if user_info.user_info_point < membership.price {
            return None;
        }

        let now = Local::now().naive_local();
        let month_start = beginning_of_month(now)?;
        let next_month_start = next_month_start(now)?;
        let already_purchased: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM MEMBERSHIP_PAYMENT
            WHERE USER_ID = $1
              AND STATUS = 'COMPLETE'
              AND PAIDAT >= $2
              AND PAIDAT < $3",
        )
        .bind(user_id)
        .bind(month_start)
        .bind(next_month_start)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        if already_purchased > 0 {
            return None;
        }

        let (mission_make, mission_do) =
            updated_membership_counts(&user.user_type, &membership.membership_name, &user_info)?;
        let current_point = user_info.user_info_point - membership.price;

        let mut tx = self.pool.begin().await.ok()?;

        sqlx::query(
            "UPDATE USER_INFO
            SET USER_INFO_POINT = $1,
                USER_INFO_MISSION_MAKE = $2,
                USER_INFO_MISSION_DO = $3
            WHERE USER_ID = $4",
        )
        .bind(current_point)
        .bind(mission_make)
        .bind(mission_do)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .ok()?;

        let payment = sqlx::query_as::<_, MembershipPayment>(
            "INSERT INTO MEMBERSHIP_PAYMENT (
                AMOUNT, PAIDAT, STATUS, MEMBERSHIP_ID, USER_ID
            )
            VALUES ($1, $2, 'COMPLETE', $3, $4)
            RETURNING PAYMENTID as payment_id",
        )
        .bind(membership.price)
        .bind(now)
        .bind(membership.membership_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .ok()?;

        tx.commit().await.ok()?;

        let success_message = if user.user_type == "STORE" {
            format!("축하합니다! 이제 총 {mission_make}회 미션 생성이 가능합니다.")
        } else {
            format!("축하합니다! 이제 총 {mission_do}회 미션 수행이 가능합니다.")
        };

        Some(MembershipPaymentDto {
            payment_id: payment.payment_id,
            membership_name: membership.membership_name,
            mission_make,
            mission_do,
            current_point,
            success_message,
        })
    }

    pub async fn get_history(&self, user_id: i64) -> Option<Vec<MembershipHistoryDto>> {
        self.find_user(user_id).await?;
        Some(
            sqlx::query_as::<_, MembershipHistoryDto>(
                "SELECT
                    mp.PAYMENTID as payment_id,
                    m.MEMBERSHIPNAME as membership_name,
                    mp.AMOUNT as amount,
                    mp.PAIDAT as paid_at,
                    mp.STATUS as status
                FROM MEMBERSHIP_PAYMENT mp
                JOIN MEMBERSHIP m ON m.MEMBERSHIPID = mp.MEMBERSHIP_ID
                WHERE mp.USER_ID = $1
                ORDER BY mp.PAIDAT DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default(),
        )
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

    async fn find_membership(&self, membership_id: i64) -> Option<Membership> {
        sqlx::query_as::<_, Membership>(
            "SELECT
                MEMBERSHIPID as membership_id,
                MEMBERSHIPNAME as membership_name,
                PRICE as price,
                BENEFITS as benefits
            FROM MEMBERSHIP
            WHERE MEMBERSHIPID = $1",
        )
        .bind(membership_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }
}

fn beginning_of_month(now: NaiveDateTime) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(now.year(), now.month(), 1)?.and_hms_opt(0, 0, 0)
}

fn next_month_start(now: NaiveDateTime) -> Option<NaiveDateTime> {
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)?.and_hms_opt(0, 0, 0)
}

fn updated_membership_counts(
    user_type: &str,
    membership_name: &str,
    user_info: &UserInfo,
) -> Option<(i32, i32)> {
    let mut mission_make = user_info.user_info_mission_make;
    let mut mission_do = user_info.user_info_mission_do;

    if user_type == "STORE" && membership_name.contains("STORE") {
        if membership_name.contains("STARTER") {
            mission_make += 2;
        } else if membership_name.contains("GROWTH") {
            mission_make += 4;
        } else if membership_name.contains("PRO") {
            mission_make += 7;
        } else {
            return None;
        }
    } else if user_type == "USER" && membership_name.contains("USER") {
        if membership_name.contains("STARTER") {
            mission_do += 10;
        } else if membership_name.contains("GROWTH") {
            mission_do += 15;
        } else if membership_name.contains("PRO") {
            mission_do += 20;
        } else {
            return None;
        }
    } else {
        return None;
    }

    Some((mission_make, mission_do))
}
