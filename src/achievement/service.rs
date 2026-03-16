use chrono::Local;
use sqlx::PgPool;

use crate::achievement::model::{Achievement, UserAchievement};
use crate::auth::model::UserInfo;

// 업적 서비스다.
#[derive(Clone)]
pub struct AchievementService {
    pool: PgPool,
}

impl AchievementService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_achievements(&self) -> Vec<Achievement> {
        sqlx::query_as::<_, Achievement>(
            "SELECT
                ACHIEVEMENT_ID as achievement_id,
                ACHIEVEMENT_NAME as achievement_name,
                CONDITION_DESC as condition_desc,
                CONDITION_TYPE as condition_type,
                CONDITION_VALUE as condition_value,
                REWARD_POINT as reward_point,
                REWARD_EXP as reward_exp
            FROM ACHIEVEMENT
            ORDER BY ACHIEVEMENT_ID",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn get_user_achievements(&self, user_id: i64) -> Vec<UserAchievement> {
        sqlx::query_as::<_, UserAchievement>(
            "SELECT
                ua.USER_ACHIEVEMENT_ID as user_achievement_id,
                ua.USER_ID as user_id,
                ua.ACHIEVEMENT_ID as achievement_id,
                ua.ACHIEVED_AT as achieved_at,
                a.ACHIEVEMENT_NAME as achievement_name
            FROM USER_ACHIEVEMENT ua
            JOIN ACHIEVEMENT a ON a.ACHIEVEMENT_ID = ua.ACHIEVEMENT_ID
            WHERE ua.USER_ID = $1
            ORDER BY ua.ACHIEVED_AT DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn check_and_achieve(&self, user_id: i64) {
        let user_info = match self.find_user_info(user_id).await {
            Some(info) => info,
            None => return,
        };

        let achievements = self.get_achievements().await;
        for achievement in achievements {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM USER_ACHIEVEMENT WHERE USER_ID = $1 AND ACHIEVEMENT_ID = $2",
            )
            .bind(user_id)
            .bind(achievement.achievement_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
            if exists > 0 {
                continue;
            }

            let achieved = match achievement.condition_type.as_deref() {
                Some("LEVEL") => {
                    user_info.user_info_level >= achievement.condition_value.unwrap_or(0)
                }
                Some("MISSION_DO") => {
                    user_info.user_info_mission_do >= achievement.condition_value.unwrap_or(0)
                }
                Some("MISSION_MAKE") => {
                    user_info.user_info_mission_make >= achievement.condition_value.unwrap_or(0)
                }
                Some("ATTEND") => {
                    user_info.user_info_attend >= achievement.condition_value.unwrap_or(0)
                }
                _ => false,
            };

            if achieved {
                sqlx::query(
                    "INSERT INTO USER_ACHIEVEMENT (USER_ID, ACHIEVEMENT_ID, ACHIEVED_AT)
                    VALUES ($1, $2, $3)",
                )
                .bind(user_id)
                .bind(achievement.achievement_id)
                .bind(Local::now().naive_local())
                .execute(&self.pool)
                .await
                .ok();

                sqlx::query(
                    "UPDATE USER_INFO
                    SET USER_INFO_POINT = COALESCE(USER_INFO_POINT, 0) + $1,
                        USER_INFO_EXP = COALESCE(USER_INFO_EXP, 0) + $2
                    WHERE USER_ID = $3",
                )
                .bind(achievement.reward_point.unwrap_or(0))
                .bind(achievement.reward_exp.unwrap_or(0))
                .bind(user_id)
                .execute(&self.pool)
                .await
                .ok();
            }
        }
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
