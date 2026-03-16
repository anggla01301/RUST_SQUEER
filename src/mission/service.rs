use chrono::{Local, NaiveDateTime, TimeDelta};
use sqlx::PgPool;

use crate::mission::model::{
    AuthenticateRequestDto, Mission, MissionBookmark, MissionCreateResponseDto,
    MissionListResponseDto, MissionParticipate, MissionParticipateResponseDto, MissionRequestDto,
};
use crate::notification::service::NotificationService;
use crate::store::model::Store;

// 미션 비즈니스 로직을 담당한다.
#[derive(Clone)]
pub struct MissionService {
    pool: PgPool,
}

impl MissionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 미션 생성:
    // 1. STORE 계정인지 검사
    // 2. 월간 생성 가능 횟수 확인
    // 3. 점포 조회
    // 4. 기간/코드 생성
    // 5. MISSION 저장
    // 6. 버프/횟수 차감 반영
    pub async fn create_mission(
        &self,
        user_id: i64,
        dto: MissionRequestDto,
    ) -> Option<MissionCreateResponseDto> {
        let user = self.find_user(user_id).await?;
        if user.user_type != "STORE" {
            return None;
        }

        let user_info = self.find_user_info(user_id).await?;
        if user_info.user_info_mission_make <= 0 {
            return None;
        }

        let store = self.find_store_by_user_id(user_id).await?;
        let start = Local::now().date_naive();
        let end = start + chrono::Days::new(14);
        // UUID 일부를 잘라 6자리 미션 코드를 만든다.
        let mission_code = uuid::Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(6)
            .collect::<String>()
            .to_uppercase();

        let mission = sqlx::query_as::<_, Mission>(
            "INSERT INTO MISSION (
                MISSION_TITLE, MISSION_START, MISSION_END, MISSION_INFO,
                MISSION_PEOPLE, MISSION_CODE, MISSION_IMAGE, STORE_ID, IS_PULL_UP, MISSION_CREATED_AT
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, 0, $8)
            RETURNING
                MISSION_ID as mission_id,
                MISSION_TITLE as mission_title,
                MISSION_START as mission_start,
                MISSION_END as mission_end,
                MISSION_INFO as mission_info,
                MISSION_PEOPLE as mission_people,
                MISSION_CODE as mission_code,
                MISSION_IMAGE as mission_image,
                STORE_ID as store_id,
                IS_PULL_UP as is_pull_up,
                MISSION_CREATED_AT as mission_created_at",
        )
        .bind(dto.mission_title)
        .bind(start)
        .bind(end)
        .bind(dto.mission_info)
        .bind(user_info.temp_mission_people)
        .bind(&mission_code)
        .bind(store.store_id?)
        .bind(Local::now().naive_local())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;

        // 미션 생성에 사용한 인원 확장 버프는 1회성으로 보고 초기화한다.
        sqlx::query(
            "UPDATE USER_INFO
            SET TEMP_MISSION_PEOPLE = 15,
                USER_INFO_MISSION_MAKE = USER_INFO_MISSION_MAKE - 1
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .ok();

        Some(MissionCreateResponseDto {
            mission_id: mission.mission_id,
            mission_code,
            mission_start: start,
            mission_end: end,
        })
    }

    // 목록 조회는 STORE_를 조인해서 가게 정보까지 함께 내린다.
    pub async fn get_missions(&self) -> Vec<MissionListResponseDto> {
        self.query_mission_list(
            "SELECT
                m.MISSION_ID as mission_id,
                s.USER_ID as user_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                m.MISSION_IMAGE as mission_image,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.IS_PULL_UP as is_pull_up
            FROM MISSION m
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            ORDER BY m.MISSION_ID DESC",
        )
        .await
    }

    // "내 미션"은 사용자 타입에 따라 다르게 해석한다.
    // STORE면 내가 만든 미션, USER면 내가 참여한 미션 목록을 반환한다.
    pub async fn get_my_missions(&self, user_id: i64) -> Vec<MissionListResponseDto> {
        let Some(user) = self.find_user(user_id).await else {
            return Vec::new();
        };

        let sql = if user.user_type == "USER" {
            "SELECT DISTINCT
                m.MISSION_ID as mission_id,
                s.USER_ID as user_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                m.MISSION_IMAGE as mission_image,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.IS_PULL_UP as is_pull_up
            FROM MISSION_PARTICIPATE p
            JOIN MISSION m ON m.MISSION_ID = p.MISSION_ID
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE p.USER_ID = $1
            ORDER BY m.MISSION_ID DESC"
        } else {
            "SELECT
                m.MISSION_ID as mission_id,
                s.USER_ID as user_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                m.MISSION_IMAGE as mission_image,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.IS_PULL_UP as is_pull_up
            FROM MISSION m
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE s.USER_ID = $1
            ORDER BY m.MISSION_ID DESC"
        };

        sqlx::query_as::<_, MissionListResponseDto>(sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
    }

    pub async fn get_mission(&self, mission_id: i64) -> Option<Mission> {
        sqlx::query_as::<_, Mission>(
            "SELECT
                MISSION_ID as mission_id,
                MISSION_TITLE as mission_title,
                MISSION_START as mission_start,
                MISSION_END as mission_end,
                MISSION_INFO as mission_info,
                MISSION_PEOPLE as mission_people,
                MISSION_CODE as mission_code,
                MISSION_IMAGE as mission_image,
                STORE_ID as store_id,
                IS_PULL_UP as is_pull_up,
                MISSION_CREATED_AT as mission_created_at
            FROM MISSION
            WHERE MISSION_ID = $1",
        )
        .bind(mission_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    // 수정 전에 소유권 검사를 수행해 남의 가게 미션 수정 시도를 막는다.
    pub async fn update_mission(
        &self,
        mission_id: i64,
        user_id: i64,
        update: Mission,
    ) -> Option<Mission> {
        let current = self.get_mission(mission_id).await?;
        let store = self.find_store_by_id(current.store_id).await?;
        if store.user_id != user_id {
            return None;
        }

        sqlx::query_as::<_, Mission>(
            "UPDATE MISSION
            SET MISSION_TITLE = $1,
                MISSION_INFO = $2,
                MISSION_START = $3,
                MISSION_END = $4,
                MISSION_PEOPLE = $5
            WHERE MISSION_ID = $6
            RETURNING
                MISSION_ID as mission_id,
                MISSION_TITLE as mission_title,
                MISSION_START as mission_start,
                MISSION_END as mission_end,
                MISSION_INFO as mission_info,
                MISSION_PEOPLE as mission_people,
                MISSION_CODE as mission_code,
                MISSION_IMAGE as mission_image,
                STORE_ID as store_id,
                IS_PULL_UP as is_pull_up,
                MISSION_CREATED_AT as mission_created_at",
        )
        .bind(update.mission_title)
        .bind(update.mission_info)
        .bind(update.mission_start)
        .bind(update.mission_end)
        .bind(update.mission_people)
        .bind(mission_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn delete_mission(&self, mission_id: i64, user_id: i64) -> bool {
        let current = match self.get_mission(mission_id).await {
            Some(current) => current,
            None => return false,
        };
        let store = match self.find_store_by_id(current.store_id).await {
            Some(store) => store,
            None => return false,
        };
        if store.user_id != user_id {
            return false;
        }

        sqlx::query("DELETE FROM MISSION WHERE MISSION_ID = $1")
            .bind(mission_id)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .unwrap_or(false)
    }

    pub async fn get_missions_by_category(&self, category: &str) -> Vec<MissionListResponseDto> {
        sqlx::query_as::<_, MissionListResponseDto>(
            "SELECT
                m.MISSION_ID as mission_id,
                s.USER_ID as user_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                m.MISSION_IMAGE as mission_image,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.IS_PULL_UP as is_pull_up
            FROM MISSION m
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE s.STORE_CATEGORY = $1
            ORDER BY m.MISSION_ID DESC",
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    // 미션 참여는 USER_ID + MISSION_ID 조합 중복을 막아야 한다.
    pub async fn participate(
        &self,
        mission_id: i64,
        user_id: i64,
    ) -> Option<MissionParticipateResponseDto> {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM MISSION_PARTICIPATE WHERE MISSION_ID = $1 AND USER_ID = $2",
        )
        .bind(mission_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        if exists > 0 {
            return None;
        }

        let _ = self.get_mission(mission_id).await?;
        // 참여 시작 상태는 "진행중", 보상 지급 여부는 'N'으로 기록한다.
        let saved = sqlx::query_as::<_, MissionParticipate>(
            "INSERT INTO MISSION_PARTICIPATE (
                MISSION_ID, USER_ID, MISSION_PARTICIPATE_STATUS, MISSION_PARTICIPATE_START_DATE,
                MISSION_PARTICIPATE_GIVEN_YN, MISSION_PARTICIPATE_AUTHENTICATED_TRY, MISSION_PARTICIPATE_REWARD_EXP
            )
            VALUES ($1, $2, '진행중', $3, 'N', 0, 200)
            RETURNING
                PARTICIPATE_ID as participate_id,
                MISSION_ID as mission_id,
                USER_ID as user_id,
                MISSION_PARTICIPATE_CODE as mission_participate_code,
                MISSION_PARTICIPATE_STATUS as mission_participate_status,
                MISSION_PARTICIPATE_START_DATE as mission_participate_start_date,
                MISSION_PARTICIPATE_COMPLETE_DATE as mission_participate_complete_date,
                MISSION_PARTICIPATE_GIVEN_YN as mission_participate_given_yn,
                MISSION_PARTICIPATE_REWARD_EXP as mission_participate_reward_exp,
                MISSION_PARTICIPATE_AUTHENTICATED_TRY as mission_participate_authenticated_try,
                MISSION_PARTICIPATE_LOCKED_AT as mission_participate_locked_at,
                MISSION_PARTICIPATE_LOCKED_UNTIL as mission_participate_locked_until,
                COUPON_RECEIVE_ID as coupon_receive_id",
        )
        .bind(mission_id)
        .bind(user_id)
        .bind(Local::now().date_naive())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;

        self.find_participation_response(saved.participate_id).await
    }

    // 인증은 소유권, 좌표 범위, 잠금 상태, 거리, 코드 일치 여부를 순서대로 검사한다.
    pub async fn authenticate(
        &self,
        participate_id: i64,
        user_id: i64,
        dto: AuthenticateRequestDto,
    ) -> Option<String> {
        let participate = self.find_participation(participate_id).await?;
        if participate.user_id != user_id {
            return None;
        }

        if !(-90.0..=90.0).contains(&dto.user_lat) || !(-180.0..=180.0).contains(&dto.user_lng) {
            return None;
        }

        // 3회 이상 실패 시 1시간 잠금되므로 먼저 잠금 여부를 확인한다.
        if let Some(until) = participate.mission_participate_locked_until {
            if Local::now().naive_local() < until {
                return Some("잠금 상태입니다".to_string());
            }
        }

        let mission = self.get_mission(participate.mission_id).await?;
        let store = self.find_store_by_id(mission.store_id).await?;
        // 실제 방문 미션이므로 가게 반경 200m 이내만 허용한다.
        if calc_distance(
            dto.user_lat,
            dto.user_lng,
            store.store_latitude,
            store.store_longitude,
        ) > 200.0
        {
            return Some("가게와 너무 멉니다".to_string());
        }

        if mission.mission_code == dto.input_code {
            // 성공 시 참여 상태를 완료로 바꾸고 exp / point를 즉시 반영한다.
            sqlx::query(
                "UPDATE MISSION_PARTICIPATE
                SET MISSION_PARTICIPATE_STATUS = '완료',
                    MISSION_PARTICIPATE_COMPLETE_DATE = $1,
                    MISSION_PARTICIPATE_GIVEN_YN = 'Y'
                WHERE PARTICIPATE_ID = $2",
            )
            .bind(Local::now().date_naive())
            .bind(participate_id)
            .execute(&self.pool)
            .await
            .ok();

            let exp = participate.mission_participate_reward_exp.unwrap_or(200) as i32;
            sqlx::query(
                "UPDATE USER_INFO
                SET USER_INFO_EXP = COALESCE(USER_INFO_EXP, 0) + $1
                WHERE USER_ID = $2",
            )
            .bind(exp)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .ok();

            let notification_service = NotificationService::new(self.pool.clone());
            let _ = notification_service
                .send(
                    user_id,
                    None,
                    "MISSION_COMPLETE",
                    &format!("미션 완료! EXP +{exp}"),
                )
                .await;
            return Some("인증 성공".to_string());
        }

        // 실패 횟수를 누적하고 3회째부터 잠금 시각을 기록한다.
        let fail = participate
            .mission_participate_authenticated_try
            .unwrap_or(0)
            + 1;
        let now = Local::now().naive_local();
        let (locked_at, locked_until): (Option<NaiveDateTime>, Option<NaiveDateTime>) = if fail >= 3
        {
            (Some(now), Some(now + TimeDelta::hours(1)))
        } else {
            (
                participate.mission_participate_locked_at,
                participate.mission_participate_locked_until,
            )
        };

        sqlx::query(
            "UPDATE MISSION_PARTICIPATE
            SET MISSION_PARTICIPATE_AUTHENTICATED_TRY = $1,
                MISSION_PARTICIPATE_LOCKED_AT = $2,
                MISSION_PARTICIPATE_LOCKED_UNTIL = $3
            WHERE PARTICIPATE_ID = $4",
        )
        .bind(fail)
        .bind(locked_at)
        .bind(locked_until)
        .bind(participate_id)
        .execute(&self.pool)
        .await
        .ok();

        Some(format!("인증 실패 ({fail}/3)"))
    }

    // 북마크도 같은 미션을 여러 번 담지 못하도록 중복 검사 후 저장한다.
    pub async fn add_bookmark(&self, mission_id: i64, user_id: i64) -> bool {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM MISSION_BOOKMARK WHERE USER_ID = $1 AND MISSION_ID = $2",
        )
        .bind(user_id)
        .bind(mission_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        if exists > 0 {
            return false;
        }

        sqlx::query_as::<_, MissionBookmark>(
            "INSERT INTO MISSION_BOOKMARK (USER_ID, MISSION_ID)
            VALUES ($1, $2)
            RETURNING BOOKMARK_ID as bookmark_id, USER_ID as user_id, MISSION_ID as mission_id",
        )
        .bind(user_id)
        .bind(mission_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
        .is_some()
    }

    pub async fn remove_bookmark(&self, mission_id: i64, user_id: i64) -> bool {
        sqlx::query("DELETE FROM MISSION_BOOKMARK WHERE USER_ID = $1 AND MISSION_ID = $2")
            .bind(user_id)
            .bind(mission_id)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .unwrap_or(false)
    }

    pub async fn get_my_bookmarks(&self, user_id: i64) -> Vec<MissionListResponseDto> {
        sqlx::query_as::<_, MissionListResponseDto>(
            "SELECT
                m.MISSION_ID as mission_id,
                s.USER_ID as user_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                m.MISSION_IMAGE as mission_image,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.IS_PULL_UP as is_pull_up
            FROM MISSION_BOOKMARK b
            JOIN MISSION m ON m.MISSION_ID = b.MISSION_ID
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE b.USER_ID = $1
            ORDER BY b.BOOKMARK_ID DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    // 같은 DTO를 쓰는 여러 목록 API가 공통으로 사용하는 실행부다.
    async fn query_mission_list(&self, sql: &str) -> Vec<MissionListResponseDto> {
        sqlx::query_as::<_, MissionListResponseDto>(sql)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
    }

    async fn find_participation_response(
        &self,
        participate_id: i64,
    ) -> Option<MissionParticipateResponseDto> {
        sqlx::query_as::<_, MissionParticipateResponseDto>(
            "SELECT
                p.PARTICIPATE_ID as participate_id,
                m.MISSION_ID as mission_id,
                m.MISSION_TITLE as mission_title,
                p.MISSION_PARTICIPATE_STATUS as mission_participate_status,
                p.MISSION_PARTICIPATE_START_DATE as mission_participate_start_date,
                s.STORE_NAME as store_name
            FROM MISSION_PARTICIPATE p
            JOIN MISSION m ON m.MISSION_ID = p.MISSION_ID
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE p.PARTICIPATE_ID = $1",
        )
        .bind(participate_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_participation(&self, participate_id: i64) -> Option<MissionParticipate> {
        sqlx::query_as::<_, MissionParticipate>(
            "SELECT
                PARTICIPATE_ID as participate_id,
                MISSION_ID as mission_id,
                USER_ID as user_id,
                MISSION_PARTICIPATE_CODE as mission_participate_code,
                MISSION_PARTICIPATE_STATUS as mission_participate_status,
                MISSION_PARTICIPATE_START_DATE as mission_participate_start_date,
                MISSION_PARTICIPATE_COMPLETE_DATE as mission_participate_complete_date,
                MISSION_PARTICIPATE_GIVEN_YN as mission_participate_given_yn,
                MISSION_PARTICIPATE_REWARD_EXP as mission_participate_reward_exp,
                MISSION_PARTICIPATE_AUTHENTICATED_TRY as mission_participate_authenticated_try,
                MISSION_PARTICIPATE_LOCKED_AT as mission_participate_locked_at,
                MISSION_PARTICIPATE_LOCKED_UNTIL as mission_participate_locked_until,
                COUPON_RECEIVE_ID as coupon_receive_id
            FROM MISSION_PARTICIPATE
            WHERE PARTICIPATE_ID = $1",
        )
        .bind(participate_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_user(&self, user_id: i64) -> Option<crate::auth::model::User> {
        sqlx::query_as::<_, crate::auth::model::User>(
            "SELECT
                USER_ID as user_id, USER_NAME as user_name, USER_NICKNAME as user_nickname,
                USER_EMAIL as user_email, USER_PASSWORD as user_password, USER_NUMBER as user_number,
                USER_AVATAR as user_avatar, USER_IS_ACTIVE as user_is_active, USER_JOINDATE as user_joindate,
                USER_UPDATEDATE as user_updatedate, USER_TYPE as user_type, USER_WITHDRAW_DATE as user_withdraw_date,
                PROVIDER as provider, PROVIDER_ID as provider_id, REFRESH_TOKEN as refresh_token
            FROM USER_
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_user_info(&self, user_id: i64) -> Option<crate::auth::model::UserInfo> {
        sqlx::query_as::<_, crate::auth::model::UserInfo>(
            "SELECT
                USER_ID as user_id, USER_INFO_LEVEL as user_info_level, USER_INFO_EXP as user_info_exp,
                USER_INFO_POINT as user_info_point, USER_INFO_MISSION_DO as user_info_mission_do,
                USER_INFO_MISSION_MAKE as user_info_mission_make, USER_INFO_ATTEND as user_info_attend,
                USER_INFO_ATTEND_STRAIGHT as user_info_attend_straight, USER_INFO_ATTEND_MAX as user_info_attend_max,
                LAST_ATTEND_DATE as last_attend_date, TEMP_MISSION_PEOPLE as temp_mission_people,
                TEMP_EXP_MULTIPLIER as temp_exp_multiplier
            FROM USER_INFO
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_store_by_user_id(&self, user_id: i64) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "SELECT
                STORE_ID as store_id, STORE_NAME as store_name, STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category, STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude, USER_ID as user_id
            FROM STORE_
            WHERE USER_ID = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    async fn find_store_by_id(&self, store_id: i64) -> Option<Store> {
        sqlx::query_as::<_, Store>(
            "SELECT
                STORE_ID as store_id, STORE_NAME as store_name, STORE_ADDRESS as store_address,
                STORE_CATEGORY as store_category, STORE_LATITUDE as store_latitude,
                STORE_LONGITUDE as store_longitude, USER_ID as user_id
            FROM STORE_
            WHERE STORE_ID = $1",
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }
}

fn calc_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    // Haversine 공식으로 두 좌표 사이의 구면 거리를 미터 단위로 계산한다.
    let r = 6_371_000.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}
