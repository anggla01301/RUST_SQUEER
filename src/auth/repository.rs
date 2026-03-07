use crate::auth::model::User;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct UserRepository {
    pool: PgPool, //DB 커넥션 풀(JpaRepository 대응)
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    //findById()대응
    pub async fn find_by_id(&self, user_id: i64) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM USER_ WHERE USER_ID = $1"
        )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None)
    }

    //existsByUserEmail() 대응
    pub async fn exists_by_user_email(&self, email: &str) -> bool {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM USER_ WHERE USER_EMAIL= $1")
                .bind(email)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        result > 0
    }

    //findByUserEmail() 대응
    pub async fn find_by_user_email(&self, email: &str) -> Option<User> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM USER_ WHERE EMAIL= $1"
        )
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None)
    }

    //findByUserIsActiveAndUserWithdrawDateBefore() 대응
    pub async fn find_by_active_and_withdraw_before(
        &self,
        is_active: &str,
        date: NaiveDateTime,
    )->Vec<User>{
        sqlx::query_as::<_,User>(
            "SELECT * FROM USER_ WHERE USER_ID_ACTIVE= $1 AND USER_WITHDRAW_DATE<$2"
        ).bind(is_active).bind(date).fetch_all(&self.pool).await.unwrap_or(vec![])
    }

    //findByProviderAndProviderId() 대응
    pub async fn find_by_provider_and_provider_id(
        &self,
        provider: &str,
        provider_id: &str,
    )->Option<User>{
        sqlx::query_as::<_,User>(
            "SELECT * FROM USER_ WHERE PROVIDER= $1 AND PROVIDER_ID= $2"
        ).bind(provider).bind(provider_id).fetch_optional(&self.pool).await.unwrap_or(None)
    }

    //existsByUserEmailAndProviderIsNotNull() 대응
    pub async fn exists_by_email_and_provider_not_null(&self,email: &str)->bool{
        let result=sqlx::query_scalar::<_,i64>(
            "SELECT COUNT(*) FROM USER_ WHRERE USER_EMAIL=$1 AND PROVIDER IS NOT NULL"
        ).bind(email).fetch_one(&self.pool).await.unwrap_or(0);
        result>0
    }

    //save() 대응
    pub async fn save(&self, user: &User)->Option<User>{
        sqlx::query_as::<_,User>(
            "INSERT INTO USER_ (USER_NICKNAME, USER_EMAIL, USER_PASSWORD, USER_NUMBER,
                   USER_IS_ACTIVE,USER_JOINDATE, USER_UPDATEDATE,USER_TYPE,PROVIDER, PROVIDER_ID)
            VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"

        )
            .bind(&user.user_nickname).bind(&user.user_email).bind(&user.user_password).bind(&user.user_number)
            .bind(&user.user_is_active).bind(user.user_joindate).bind(user.user_updatedate).bind(&user.user_type)
            .bind(&user.provider).bind(&user.provider_id).fetch_optional(&self.pool).await.unwrap_or(None)
    }

    //refreshToken 업데이트
    pub async fn update_refresh_token(&self, user_id:i64,refresh_token:&str){
        sqlx::query(
            "UPDATE USER_ SET REFRESH_TOKEN = $1 WHERE USER_ID=$2"
        ).bind(refresh_token).bind(user_id).execute(&self.pool).await.ok();
    }

    //탈퇴 처리
    pub async fn update_withdraw(&self, user_id: i64){
        sqlx::query(
            "UPDATE USER_ SET USER_IS_ACTIVE='N', REFRESH_TOKEN=NULL WHERE USER_ID=$1"
        ).bind(user_id).execute(&self.pool).await.ok();
    }

    //userType 업데이트
    pub async fn update_user_type(&self, user_id: i64, user_type: &str){
        sqlx::query(
            "UPDATE USER_ SET USER_TYPE=$1 WHERE USER_ID=$2"
        ).bind(user_type).bind(user_id).execute(&self.pool).await.ok();

    }
}
