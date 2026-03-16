use anyhow::Result;
use sqlx::PgPool;

use crate::achievement::service::AchievementService;
use crate::attendance::service::AttendanceService;
use crate::auth::jwt::JwtUtil;
use crate::auth::oauth::service::OAuthService;
use crate::auth::repository::{UserInfoRepository, UserRepository};
use crate::auth::service::AuthService;
use crate::coupon::service::CouponService;
use crate::event::service::EventService;
use crate::common::service::OciStorageService;
use crate::config::service::ConfigService;
use crate::location::service::LocationService;
use crate::membership::service::MembershipService;
use crate::mission::service::MissionService;
use crate::notification::service::NotificationService;
use crate::ranking::service::RankingService;
use crate::search::service::SearchService;
use crate::servicecenter::service::ServiceCenterService;
use crate::status::service::StatusService;
use crate::store::service::StoreRepository;
use crate::store::service::StoreService;
use crate::user::service::UserService;

#[derive(Clone)]
pub struct AppState {
    // 컨트롤러가 직접 써야 하는 "공용 의존성의 최종 형태"만 둔다.
    // 즉, 전역에서 재사용할 서비스/리포지토리를 이 구조체에 모은다.
    pub achievement_service: AchievementService,
    pub attendance_service: AttendanceService,
    pub auth_service: AuthService,
    pub coupon_service: CouponService,
    pub config_service: ConfigService,
    pub event_service: EventService,
    pub location_service: LocationService,
    pub membership_service: MembershipService,
    pub mission_service: MissionService,
    pub notification_service: NotificationService,
    pub oauth_service: OAuthService,
    pub oci_storage_service: OciStorageService,
    pub ranking_service: RankingService,
    pub search_service: SearchService,
    pub service_center_service: ServiceCenterService,
    pub status_service: StatusService,
    pub store_service: StoreService,
    pub user_service: UserService,
    pub user_repository: UserRepository,
}

impl AppState {
    // 조립 책임은 한 곳(AppState::new)에 모은다.
    // main.rs는 "무엇을 만든다"보다 "서버를 어떻게 시작한다"에 집중하게 된다.
    pub fn new(pool: PgPool) -> Result<Self> {
        let jwt_util = JwtUtil::new()?;
        let user_repository = UserRepository::new(pool.clone());
        let user_info_repository = UserInfoRepository::new(pool.clone());
        let store_repository = StoreRepository::new(pool.clone());
        let achievement_service = AchievementService::new(pool.clone());
        let attendance_service = AttendanceService::new(pool.clone());
        let coupon_service = CouponService::new(pool.clone());
        let config_service = ConfigService::from_env()?;
        let event_service = EventService::new(pool.clone());
        let location_service = LocationService::new(pool.clone());
        let membership_service = MembershipService::new(pool.clone());
        let mission_service = MissionService::new(pool.clone());
        let notification_service = NotificationService::new(pool.clone());
        let oci_storage_service = OciStorageService::new(config_service.config.clone());
        let ranking_service = RankingService::new(pool.clone());
        let search_service = SearchService::new(pool.clone());
        let service_center_service = ServiceCenterService::new(pool.clone());
        let status_service = StatusService::new(pool.clone());
        let store_service = StoreService::new(pool.clone());
        let user_service = UserService::new(pool);

        let auth_service = AuthService::new(
            user_repository.clone(),
            user_info_repository.clone(),
            store_repository,
            jwt_util,
        );
        let oauth_service = OAuthService::new(user_repository.clone(), user_info_repository);

        Ok(Self {
            achievement_service,
            attendance_service,
            auth_service,
            coupon_service,
            config_service,
            event_service,
            location_service,
            membership_service,
            mission_service,
            notification_service,
            oauth_service,
            oci_storage_service,
            ranking_service,
            search_service,
            service_center_service,
            status_service,
            store_service,
            user_service,
            user_repository,
        })
    }
}
