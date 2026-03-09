//state.rs
use sqlx::PgPool;
use crate::auth::jwt::JwtUtil;
use crate::auth::repository::UserRepository;
use crate::auth::service::AuthService;
//Spring의 ApplicationContext 대응
#[derive(Clone)]
pub struct AppState{
    pub auth_service: AuthService,
    pub user_repository: UserRepository,
}

impl AppState{
    pub async fn new(pool: PgPool)->Self{
        let user_repository=UserRepository::new(pool.clone());
        let auth_service=AuthService::new(UserRepository::new(pool),JwtUtil::new());
        Self{auth_service,user_repository}
    }
}

