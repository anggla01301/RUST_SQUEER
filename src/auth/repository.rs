use chrono::NaiveDateTime;
// NaiveDateTime:
//   날짜 + 시간을 표현하는 chrono 타입이다.
//   timezone(시간대) 정보는 없다.
//   여기서는 "탈퇴일이 특정 시점보다 이전인지" 비교할 때 사용한다.

use sqlx::PgPool;
// PgPool:
//   PostgreSQL 연결 풀이다.
//   DB 연결을 매번 새로 만들지 않고 재사용하기 위한 객체다.
//   자바로 치면 DataSource/HikariCP 비슷한 감각으로 보면 된다.

use crate::auth::model::{User, UserInfo};
// User:
//   USER_ 테이블 한 행(row)을 Rust 구조체로 표현한 타입
//
// UserInfo:
//   USER_INFO 테이블 한 행(row)을 Rust 구조체로 표현한 타입

// 인증 도메인의 DB 접근 계층이다.
// 서비스 계층이 "무슨 작업을 할지" 결정한다면,
// Repository는 "DB에 어떤 SQL을 날릴지"를 담당한다.

#[derive(Clone)]
pub struct UserRepository {
    // 실제 DB 질의에 사용할 PostgreSQL 연결 풀
    pool: PgPool,
}

impl UserRepository {
    // UserRepository 생성자
    // main.rs 같은 곳에서 만들어둔 pool을 주입받아 저장한다.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, user_id: i64) -> Option<User> {
        sqlx::query_as::<_, User>(
            // query_as::<_, User>(...):
            //   SQL 조회 결과를 User 구조체로 매핑한다.
            //
            // SELECT 절에서 "컬럼명 as 필드명" 형태를 쓰는 이유:
            //   DB 컬럼명(USER_ID)과 Rust 필드명(user_id)이 다르기 때문이다.
            "SELECT
                USER_ID             as user_id,
                USER_NAME           as user_name,
                USER_NICKNAME       as user_nickname,
                USER_EMAIL          as user_email,
                USER_PASSWORD       as user_password,
                USER_NUMBER         as user_number,
                USER_AVATAR         as user_avatar,
                USER_IS_ACTIVE      as user_is_active,
                USER_JOINDATE       as user_joindate,
                USER_UPDATEDATE     as user_updatedate,
                USER_TYPE           as user_type,
                USER_WITHDRAW_DATE  as user_withdraw_date,
                PROVIDER            as provider,
                PROVIDER_ID         as provider_id,
                REFRESH_TOKEN       as refresh_token
            FROM USER_
            WHERE USER_ID = $1",
            // $1:
            //   PostgreSQL 바인딩 파라미터 자리다.
            //   아래 .bind(user_id)가 이 자리에 들어간다.
        )
            .bind(user_id)
            // bind(user_id):
            //   SQL의 $1 자리에 user_id 값을 넣는다.
            //   문자열 더하기로 SQL을 만드는 게 아니라,
            //   바인딩 방식으로 안전하게 값을 전달한다.
            .fetch_optional(&self.pool)
            // fetch_optional(...):
            //   결과가 0개 또는 1개일 때 쓰는 메서드다.
            //   - 유저를 찾으면 Some(User)
            //   - 못 찾으면 None
            .await
            // 비동기 DB 작업 완료까지 기다린다.
            .unwrap_or(None)
        // sqlx 결과는 원래 Result<Option<User>, sqlx::Error> 형태인데,
        // - 성공하면 내부 Option<User> 사용
        // - 실패하면 None 반환
        // 즉 "유저 없음"과 "DB 에러"를 둘 다 None으로 처리한다.
    }

    pub async fn exists_by_user_email(&self, email: &str) -> bool {
        let result = sqlx::query_scalar::<_, i64>(
            // query_scalar::<_, i64>(...):
            //   구조체가 아니라 스칼라 값 하나만 조회할 때 사용한다.
            //   COUNT(*) 결과는 숫자 하나이므로 i64로 받는다.
            "SELECT COUNT(*) FROM USER_ WHERE USER_EMAIL = $1",
        )
            .bind(email)
            // $1 자리에 email을 바인딩한다.
            .fetch_one(&self.pool)
            // fetch_one(...):
            //   결과가 정확히 1행이라고 기대할 때 사용한다.
            //   COUNT(*)는 조건에 맞는 행이 0개여도
            //   결과 자체는 항상 1행이므로 fetch_one이 맞다.
            .await
            .unwrap_or(0);
        // 쿼리 실패 시 기본값 0 사용

        result > 0
        // COUNT(*)가 1 이상이면 이메일이 존재하는 것이다.
        // 반환값:
        //   true  → 이미 존재
        //   false → 없음 또는 쿼리 실패
    }

    pub async fn find_by_user_email(&self, email: &str) -> Option<User> {
        sqlx::query_as::<_, User>(
            // 이메일로 USER_ 테이블에서 유저 한 명을 조회한다.
            // 로그인 시 가장 자주 쓰이는 조회다.
            "SELECT
                USER_ID             as user_id,
                USER_NAME           as user_name,
                USER_NICKNAME       as user_nickname,
                USER_EMAIL          as user_email,
                USER_PASSWORD       as user_password,
                USER_NUMBER         as user_number,
                USER_AVATAR         as user_avatar,
                USER_IS_ACTIVE      as user_is_active,
                USER_JOINDATE       as user_joindate,
                USER_UPDATEDATE     as user_updatedate,
                USER_TYPE           as user_type,
                USER_WITHDRAW_DATE  as user_withdraw_date,
                PROVIDER            as provider,
                PROVIDER_ID         as provider_id,
                REFRESH_TOKEN       as refresh_token
            FROM USER_
            WHERE USER_EMAIL = $1",
        )
            .bind(email)
            // $1 자리에 email을 넣는다.
            .fetch_optional(&self.pool)
            // 결과: Some(User) 또는 None
            .await
            .unwrap_or(None)
        // 쿼리 실패도 None으로 처리
    }

    pub async fn find_by_active_and_withdraw_before(
        &self,
        is_active: &str,
        date: NaiveDateTime,
    ) -> Vec<User> {
        sqlx::query_as::<_, User>(
            // 특정 활성 상태이면서,
            // 탈퇴일(USER_WITHDRAW_DATE)이 특정 시점보다 이전인 유저들을 모두 조회한다.
            //
            // 예:
            //   USER_IS_ACTIVE   = 'N'
            //   USER_WITHDRAW_DATE < 기준 시각
            //
            // 오래된 탈퇴 계정 정리 같은 배치 작업에서 사용한다.
            "SELECT
                USER_ID             as user_id,
                USER_NAME           as user_name,
                USER_NICKNAME       as user_nickname,
                USER_EMAIL          as user_email,
                USER_PASSWORD       as user_password,
                USER_NUMBER         as user_number,
                USER_AVATAR         as user_avatar,
                USER_IS_ACTIVE      as user_is_active,
                USER_JOINDATE       as user_joindate,
                USER_UPDATEDATE     as user_updatedate,
                USER_TYPE           as user_type,
                USER_WITHDRAW_DATE  as user_withdraw_date,
                PROVIDER            as provider,
                PROVIDER_ID         as provider_id,
                REFRESH_TOKEN       as refresh_token
            FROM USER_
            WHERE USER_IS_ACTIVE = $1
              AND USER_WITHDRAW_DATE < $2",
        )
            .bind(is_active)    // $1 <- is_active
            .bind(date)         // $2 <- date
            .fetch_all(&self.pool)
            // fetch_all(...):
            //   결과 여러 행을 전부 가져와 Vec<User>로 만든다.
            .await
            .unwrap_or(vec![])
        // 실패 시 빈 벡터 반환
    }

    pub async fn find_by_provider_and_provider_id(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> Option<User> {
        sqlx::query_as::<_, User>(
            // 소셜 로그인 계정을 provider + provider_id 조합으로 조회한다.
            //
            // 예:
            //   provider    = "google"
            //   provider_id = "123456789"
            //
            // 같은 provider 안에서 provider_id가 유저 식별자 역할을 한다.
            "SELECT
                USER_ID             as user_id,
                USER_NAME           as user_name,
                USER_NICKNAME       as user_nickname,
                USER_EMAIL          as user_email,
                USER_PASSWORD       as user_password,
                USER_NUMBER         as user_number,
                USER_AVATAR         as user_avatar,
                USER_IS_ACTIVE      as user_is_active,
                USER_JOINDATE       as user_joindate,
                USER_UPDATEDATE     as user_updatedate,
                USER_TYPE           as user_type,
                USER_WITHDRAW_DATE  as user_withdraw_date,
                PROVIDER            as provider,
                PROVIDER_ID         as provider_id,
                REFRESH_TOKEN       as refresh_token
            FROM USER_
            WHERE PROVIDER = $1
              AND PROVIDER_ID = $2",
        )
            .bind(provider)     // $1 <- provider
            .bind(provider_id)  // $2 <- provider_id
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None)
    }

    pub async fn exists_by_email_and_provider_not_null(&self, email: &str) -> bool {
        let result = sqlx::query_scalar::<_, i64>(
            // 같은 이메일이 존재하면서,
            // PROVIDER가 NULL이 아닌 계정이 있는지 검사한다.
            // 즉 "소셜 계정으로 가입된 이메일인가?"를 확인하는 용도다.
            "SELECT COUNT(*) FROM USER_
            WHERE USER_EMAIL = $1
              AND PROVIDER IS NOT NULL",
        )
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        result > 0
        // COUNT > 0 이면 true
    }

    pub async fn save(&self, user: &User) -> Option<User> {
        sqlx::query_as::<_, User>(
            // USER_ 테이블에 새 row를 INSERT 한다.
            //
            // INSERT 후 RETURNING 절을 통해
            // 저장된 최종 row를 다시 받아 User 구조체로 매핑한다.
            //
            // 저장 성공 시 반환값:
            //   - DB가 생성한 USER_ID
            //   - 저장된 전체 컬럼값을 포함한 User
            "INSERT INTO USER_ (
                USER_NAME, USER_NICKNAME, USER_EMAIL, USER_PASSWORD,
                USER_NUMBER, USER_AVATAR, USER_IS_ACTIVE, USER_JOINDATE,
                USER_UPDATEDATE, USER_TYPE, USER_WITHDRAW_DATE,
                PROVIDER, PROVIDER_ID, REFRESH_TOKEN
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14
            )
            RETURNING
                USER_ID             as user_id,
                USER_NAME           as user_name,
                USER_NICKNAME       as user_nickname,
                USER_EMAIL          as user_email,
                USER_PASSWORD       as user_password,
                USER_NUMBER         as user_number,
                USER_AVATAR         as user_avatar,
                USER_IS_ACTIVE      as user_is_active,
                USER_JOINDATE       as user_joindate,
                USER_UPDATEDATE     as user_updatedate,
                USER_TYPE           as user_type,
                USER_WITHDRAW_DATE  as user_withdraw_date,
                PROVIDER            as provider,
                PROVIDER_ID         as provider_id,
                REFRESH_TOKEN       as refresh_token",
        )
            .bind(&user.user_name)          // $1  <- user_name
            .bind(&user.user_nickname)      // $2  <- user_nickname
            .bind(&user.user_email)         // $3  <- user_email (Option<String>, None이면 NULL)
            .bind(&user.user_password)      // $4  <- user_password
            .bind(&user.user_number)        // $5  <- user_number
            .bind(&user.user_avatar)        // $6  <- user_avatar
            .bind(&user.user_is_active)     // $7  <- user_is_active
            .bind(user.user_joindate)       // $8  <- user_joindate
            .bind(user.user_updatedate)     // $9  <- user_updatedate
            .bind(&user.user_type)          // $10 <- user_type
            .bind(user.user_withdraw_date)  // $11 <- user_withdraw_date
            .bind(&user.provider)           // $12 <- provider
            .bind(&user.provider_id)        // $13 <- provider_id
            .bind(&user.refresh_token)      // $14 <- refresh_token
            .fetch_optional(&self.pool)
            // INSERT ... RETURNING 결과를 0개 또는 1개 row로 받는다.
            // 정상이라면 보통 1행이 돌아온다.
            .await
            .unwrap_or(None)
        // 실패 시 None 반환
    }

    pub async fn update_refresh_token(&self, user_id: i64, refresh_token: &str) {
        sqlx::query(
            // 특정 유저의 REFRESH_TOKEN 값을 새 토큰으로 갱신한다.
            // 동시에 USER_UPDATEDATE도 NOW()로 갱신한다.
            "UPDATE USER_
            SET REFRESH_TOKEN   = $1,
                USER_UPDATEDATE = NOW()
            WHERE USER_ID = $2",
        )
            .bind(refresh_token)    // $1 <- 새 refresh token
            .bind(user_id)          // $2 <- 대상 user_id
            .execute(&self.pool)
            // execute(...):
            //   UPDATE / DELETE / INSERT처럼
            //   row를 구조체로 받을 필요 없는 실행용 쿼리에서 사용한다.
            .await
            .ok();
        // Result를 Option으로 바꾼 뒤 버린다.
        // - 성공해도 그냥 끝
        // - 실패해도 호출자에게 에러를 올리지 않음
        // 현재 코드는 "실패를 무시하는" 스타일이다.
    }

    pub async fn clear_refresh_token(&self, user_id: i64) {
        sqlx::query(
            // 특정 유저의 REFRESH_TOKEN을 NULL로 만든다.
            // 로그아웃 시 사용된다.
            "UPDATE USER_
            SET REFRESH_TOKEN   = NULL,
                USER_UPDATEDATE = NOW()
            WHERE USER_ID = $1",
        )
            .bind(user_id)  // $1 <- 대상 user_id
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn update_withdraw(&self, user_id: i64) {
        sqlx::query(
            // 회원 탈퇴(소프트 삭제)를 수행한다.
            //
            // 실제 row를 지우는 DELETE가 아니라 UPDATE 방식이다.
            //
            // 변경 내용:
            //   - USER_IS_ACTIVE     = 'N'
            //   - USER_WITHDRAW_DATE = NOW()
            //   - REFRESH_TOKEN      = NULL
            //   - USER_UPDATEDATE    = NOW()
            "UPDATE USER_
            SET USER_IS_ACTIVE      = 'N',
                USER_WITHDRAW_DATE  = NOW(),
                REFRESH_TOKEN       = NULL,
                USER_UPDATEDATE     = NOW()
            WHERE USER_ID = $1",
        )
            .bind(user_id)  // $1 <- 대상 user_id
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn update_user_type(&self, user_id: i64, user_type: &str) {
        sqlx::query(
            // 특정 유저의 USER_TYPE을 변경한다.
            //
            // 예:
            //   PENDING -> USER
            //   PENDING -> STORE
            //   USER    -> STORE
            "UPDATE USER_
            SET USER_TYPE       = $1,
                USER_UPDATEDATE = NOW()
            WHERE USER_ID = $2",
        )
            .bind(user_type)    // $1 <- 새 user_type
            .bind(user_id)      // $2 <- 대상 user_id
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn update_profile(&self, user_id: i64, user_nickname: &str, user_type: &str) {
        sqlx::query(
            // 유저 프로필 수정 쿼리다.
            //
            // 수정 대상:
            //   - USER_NICKNAME
            //   - USER_TYPE
            //   - USER_UPDATEDATE
            //
            // 이름, 이메일, 비밀번호는 건드리지 않는다.
            "UPDATE USER_
            SET USER_NICKNAME   = $1,
                USER_TYPE       = $2,
                USER_UPDATEDATE = NOW()
            WHERE USER_ID = $3",
        )
            .bind(user_nickname)    // $1 <- 새 닉네임
            .bind(user_type)        // $2 <- 새 user_type
            .bind(user_id)          // $3 <- 대상 user_id
            .execute(&self.pool)
            .await
            .ok();
    }
}

#[derive(Clone)]
pub struct UserInfoRepository {
    // USER_INFO 테이블 접근용 PostgreSQL 연결 풀
    pool: PgPool,
}

impl UserInfoRepository {
    // UserInfoRepository 생성자
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, user_id: i64) -> Option<UserInfo> {
        sqlx::query_as::<_, UserInfo>(
            // USER_ID로 USER_INFO 한 건을 조회한다.
            // 결과가 있으면 UserInfo 구조체로 매핑한다.
            "SELECT
                USER_ID                     as user_id,
                USER_INFO_LEVEL             as user_info_level,
                USER_INFO_EXP               as user_info_exp,
                USER_INFO_POINT             as user_info_point,
                USER_INFO_MISSION_DO        as user_info_mission_do,
                USER_INFO_MISSION_MAKE      as user_info_mission_make,
                USER_INFO_ATTEND            as user_info_attend,
                USER_INFO_ATTEND_STRAIGHT   as user_info_attend_straight,
                USER_INFO_ATTEND_MAX        as user_info_attend_max,
                LAST_ATTEND_DATE            as last_attend_date,
                TEMP_MISSION_PEOPLE         as temp_mission_people,
                TEMP_EXP_MULTIPLIER         as temp_exp_multiplier
            FROM USER_INFO
            WHERE USER_ID = $1",
        )
            .bind(user_id)  // $1 <- 대상 user_id
            .fetch_optional(&self.pool)
            // 결과: Some(UserInfo) 또는 None
            .await
            .unwrap_or(None)
    }

    pub async fn save(&self, user_info: UserInfo) -> Option<UserInfo> {
        sqlx::query_as::<_, UserInfo>(
            // USER_INFO 저장 쿼리다.
            //
            // 특징: INSERT + ON CONFLICT (USER_ID) DO UPDATE
            // 즉:
            //   - USER_INFO가 없으면 새로 INSERT
            //   - 이미 있으면 UPDATE
            // 이런 패턴을 upsert라고 부른다.
            //
            // EXCLUDED.xxx:
            //   INSERT VALUES로 들어오려던 "새 값"을 뜻한다.
            //   즉 "충돌나면 새 값으로 덮어써라"는 의미다.
            "INSERT INTO USER_INFO (
                USER_ID,
                USER_INFO_LEVEL, USER_INFO_EXP, USER_INFO_POINT,
                USER_INFO_MISSION_DO, USER_INFO_MISSION_MAKE,
                USER_INFO_ATTEND, USER_INFO_ATTEND_STRAIGHT, USER_INFO_ATTEND_MAX,
                LAST_ATTEND_DATE, TEMP_MISSION_PEOPLE, TEMP_EXP_MULTIPLIER
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
            ON CONFLICT (USER_ID) DO UPDATE SET
                USER_INFO_LEVEL             = EXCLUDED.USER_INFO_LEVEL,
                USER_INFO_EXP               = EXCLUDED.USER_INFO_EXP,
                USER_INFO_POINT             = EXCLUDED.USER_INFO_POINT,
                USER_INFO_MISSION_DO        = EXCLUDED.USER_INFO_MISSION_DO,
                USER_INFO_MISSION_MAKE      = EXCLUDED.USER_INFO_MISSION_MAKE,
                USER_INFO_ATTEND            = EXCLUDED.USER_INFO_ATTEND,
                USER_INFO_ATTEND_STRAIGHT   = EXCLUDED.USER_INFO_ATTEND_STRAIGHT,
                USER_INFO_ATTEND_MAX        = EXCLUDED.USER_INFO_ATTEND_MAX,
                LAST_ATTEND_DATE            = EXCLUDED.LAST_ATTEND_DATE,
                TEMP_MISSION_PEOPLE         = EXCLUDED.TEMP_MISSION_PEOPLE,
                TEMP_EXP_MULTIPLIER         = EXCLUDED.TEMP_EXP_MULTIPLIER
            RETURNING
                USER_ID                     as user_id,
                USER_INFO_LEVEL             as user_info_level,
                USER_INFO_EXP               as user_info_exp,
                USER_INFO_POINT             as user_info_point,
                USER_INFO_MISSION_DO        as user_info_mission_do,
                USER_INFO_MISSION_MAKE      as user_info_mission_make,
                USER_INFO_ATTEND            as user_info_attend,
                USER_INFO_ATTEND_STRAIGHT   as user_info_attend_straight,
                USER_INFO_ATTEND_MAX        as user_info_attend_max,
                LAST_ATTEND_DATE            as last_attend_date,
                TEMP_MISSION_PEOPLE         as temp_mission_people,
                TEMP_EXP_MULTIPLIER         as temp_exp_multiplier",
        )
            .bind(user_info.user_id)                    // $1  <- user_id
            .bind(user_info.user_info_level)            // $2  <- level
            .bind(user_info.user_info_exp)              // $3  <- exp
            .bind(user_info.user_info_point)            // $4  <- point
            .bind(user_info.user_info_mission_do)       // $5  <- mission_do
            .bind(user_info.user_info_mission_make)     // $6  <- mission_make
            .bind(user_info.user_info_attend)           // $7  <- attend
            .bind(user_info.user_info_attend_straight)  // $8  <- attend_straight
            .bind(user_info.user_info_attend_max)       // $9  <- attend_max
            .bind(user_info.last_attend_date)           // $10 <- last_attend_date
            .bind(user_info.temp_mission_people)        // $11 <- temp_mission_people
            .bind(user_info.temp_exp_multiplier)        // $12 <- temp_exp_multiplier
            .fetch_optional(&self.pool)
            // 저장 후 RETURNING 절로 최종 row를 다시 받아온다.
            .await
            .unwrap_or(None)
        // 실패 시 None
    }
}