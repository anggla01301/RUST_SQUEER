use sqlx::PgPool;

use crate::auth::model::User;
use crate::user::model::MeResponse;

// 사용자 조회 서비스다.
#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // /api/user/me 는 JWT에서 꺼낸 user_id 하나만으로
    // 현재 로그인 사용자의 요약 정보를 만드는 API다.
    pub async fn get_me(&self, user_id: i64) -> Option<MeResponse> {
        let user = sqlx::query_as::<_, User>(
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
        .unwrap_or(None)?;

        // DB 엔티티 전체를 그대로 내보내지 않고,
        // 마이페이지에서 바로 쓸 필드만 DTO로 추려 반환한다.
        Some(MeResponse {
            user_id: user.user_id.unwrap_or_default(),
            user_name: user.user_name,
            user_nickname: user.user_nickname,
            user_email: user.user_email.unwrap_or_default(),
            user_type: user.user_type,
            user_avatar: user.user_avatar,
            user_is_active: user.user_is_active,
            user_joindate: user.user_joindate,
        })
    }
}
