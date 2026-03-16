use sqlx::PgPool;

use crate::auth::model::User;
use crate::ranking::model::{RankItemDto, RankingResponseDto};

// 랭킹 조회 서비스다.
#[derive(Clone)]
pub struct RankingService {
    pool: PgPool,
}

impl RankingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_ranking_board(
        &self,
        ranking_type: &str,
        user_id: i64,
    ) -> Option<RankingResponseDto> {
        let user = self.find_user(user_id).await?;
        let ranking_list = sqlx::query_as::<_, RankItemDto>(
            "SELECT
                r.RANK_NO as rank_no,
                u.USER_ID as user_id,
                COALESCE(r.SETTLED_NICKNAME, u.USER_NICKNAME) as nickname,
                COALESCE(r.SETTLED_AVATAR, u.USER_AVATAR) as avatar,
                r.EXP_SCORE as season_exp
            FROM RANKING r
            JOIN USER_ u ON u.USER_ID = r.USER_ID
            WHERE r.TYPE = $1
            ORDER BY r.RANK_NO ASC",
        )
        .bind(ranking_type)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let my_ranking = if user.user_type == "USER" {
            self.build_my_status(ranking_type, &user).await
        } else {
            None
        };

        Some(RankingResponseDto {
            r#type: ranking_type.to_string(),
            my_ranking,
            ranking_list,
        })
    }

    pub async fn get_my_only_status(
        &self,
        ranking_type: &str,
        user_id: i64,
    ) -> Option<RankItemDto> {
        let user = self.find_user(user_id).await?;
        if user.user_type == "STORE" {
            return None;
        }
        self.build_my_status(ranking_type, &user).await
    }

    async fn build_my_status(&self, ranking_type: &str, user: &User) -> Option<RankItemDto> {
        let entity = sqlx::query_as::<_, RankItemDto>(
            "SELECT
                r.RANK_NO as rank_no,
                u.USER_ID as user_id,
                COALESCE(r.SETTLED_NICKNAME, u.USER_NICKNAME) as nickname,
                COALESCE(r.SETTLED_AVATAR, u.USER_AVATAR) as avatar,
                r.EXP_SCORE as season_exp
            FROM RANKING r
            JOIN USER_ u ON u.USER_ID = r.USER_ID
            WHERE r.TYPE = $1 AND r.USER_ID = $2",
        )
        .bind(ranking_type)
        .bind(user.user_id.unwrap_or_default())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        entity.or_else(|| {
            Some(RankItemDto {
                rank_no: 1,
                user_id: user.user_id.unwrap_or_default(),
                nickname: user.user_nickname.clone(),
                avatar: user.user_avatar.clone(),
                season_exp: 0,
            })
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
}
