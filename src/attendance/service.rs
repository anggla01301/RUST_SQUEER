use chrono::Local;
use sqlx::PgPool;

use crate::attendance::model::AttendanceResponseDto;
use crate::auth::model::UserInfo;

// 출석 체크 비즈니스 로직을 담당한다.
#[derive(Clone)]
pub struct AttendanceService {
    pool: PgPool,
}

impl AttendanceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 출석 체크는 오늘 중복 여부, 연속 출석, 누적 출석, 보상을 한 번에 계산한다.
    pub async fn perform_check_in(&self, user_id: i64) -> AttendanceResponseDto {
        let user_info = self.find_user_info(user_id).await;
        let mut user_info = match user_info {
            Some(info) => info,
            None => {
                return AttendanceResponseDto {
                    status: "ERROR".to_string(),
                    user_info_attend_straight: 0,
                    user_info_attend: 0,
                    user_info_attend_max: 0,
                    reward_point: 0,
                    user_info_point: 0,
                    message: "사용자 정보를 찾을 수 없습니다.".to_string(),
                }
            }
        };

        // 서버 로컬 날짜를 기준으로 "오늘"을 판정한다.
        let today = Local::now().date_naive();
        if user_info.last_attend_date == Some(today) {
            return self.build_response(
                &user_info,
                "DONE",
                0,
                "오늘은 이미 출석체크를 완료했습니다.",
            );
        }

        let mut current_streak = user_info.user_info_attend_straight;
        let total_count = user_info.user_info_attend;
        let mut max_streak = user_info.user_info_attend_max;

        // 마지막 출석일이 어제가 아니면 연속 출석이 끊긴 것으로 본다.
        if let Some(last_date) = user_info.last_attend_date {
            if last_date.succ_opt() != Some(today) {
                current_streak = 0;
            }
        }

        let next_streak = current_streak + 1;
        let next_total_count = total_count + 1;
        user_info.user_info_attend_straight = next_streak;
        user_info.user_info_attend = next_total_count;
        user_info.last_attend_date = Some(today);

        if next_streak > max_streak {
            max_streak = next_streak;
            user_info.user_info_attend_max = max_streak;
        }

        // 보상 구간은 7일 / 14일만 있으므로 14일까지만 의미가 있다.
        let reward_streak = next_streak.min(14);
        let reward_point = if reward_streak == 7 {
            1000
        } else if reward_streak == 14 {
            2000
        } else {
            0
        };
        user_info.user_info_point = user_info.user_info_point.saturating_add(reward_point);

        // 계산 결과를 USER_INFO에 즉시 반영한다.
        sqlx::query(
            "UPDATE USER_INFO
            SET USER_INFO_ATTEND_STRAIGHT = $1,
                USER_INFO_ATTEND = $2,
                USER_INFO_ATTEND_MAX = $3,
                LAST_ATTEND_DATE = $4,
                USER_INFO_POINT = $5
            WHERE USER_ID = $6",
        )
        .bind(user_info.user_info_attend_straight)
        .bind(user_info.user_info_attend)
        .bind(user_info.user_info_attend_max)
        .bind(user_info.last_attend_date)
        .bind(user_info.user_info_point)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .ok();

        let success_msg = if next_streak == max_streak && next_streak > 1 {
            "역대 최고 기록 경신!"
        } else {
            "출석 체크 완료!"
        };

        self.build_response(&user_info, "DONE", reward_point, success_msg)
    }

    // 상태 조회는 값을 바꾸지 않고 READY / DONE 여부만 계산한다.
    pub async fn get_attendance_status(&self, user_id: i64) -> AttendanceResponseDto {
        let user_info = match self.find_user_info(user_id).await {
            Some(info) => info,
            None => {
                return AttendanceResponseDto {
                    status: "ERROR".to_string(),
                    user_info_attend_straight: 0,
                    user_info_attend: 0,
                    user_info_attend_max: 0,
                    reward_point: 0,
                    user_info_point: 0,
                    message: "사용자 정보 없음".to_string(),
                }
            }
        };

        let is_checked = user_info.last_attend_date == Some(Local::now().date_naive());
        self.build_response(
            &user_info,
            if is_checked { "DONE" } else { "READY" },
            0,
            if is_checked {
                "내일 또 오세요!"
            } else {
                "출석 도장을 찍어주세요!"
            },
        )
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

    fn build_response(
        &self,
        ui: &UserInfo,
        status: &str,
        reward: i32,
        msg: &str,
    ) -> AttendanceResponseDto {
        AttendanceResponseDto {
            status: status.to_string(),
            user_info_attend_straight: ui.user_info_attend_straight,
            user_info_attend: ui.user_info_attend,
            user_info_attend_max: ui.user_info_attend_max,
            reward_point: reward,
            user_info_point: ui.user_info_point,
            message: msg.to_string(),
        }
    }
}
