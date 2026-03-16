//! 위치 계산과 지역 조회 로직을 구현할 서비스 파일이다.
use sqlx::PgPool;

use crate::common::service::OciStorageService;
use crate::config::service::ConfigService;
use crate::location::model::NearbyMissionResponseDto;

#[derive(Clone)]
pub struct LocationService {
    pool: PgPool,
}

impl LocationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_nearby_missions(
        &self,
        lat: f64,
        lng: f64,
        radius_km: f64,
    ) -> Vec<NearbyMissionResponseDto> {
        let storage = ConfigService::from_env()
            .ok()
            .map(|config| OciStorageService::new(config.config));

        let mut missions = sqlx::query_as::<_, NearbyMissionResponseDto>(
            "SELECT
                m.MISSION_ID as mission_id,
                m.MISSION_TITLE as mission_title,
                m.MISSION_INFO as mission_info,
                m.MISSION_START as mission_start,
                m.MISSION_END as mission_end,
                m.MISSION_PEOPLE as mission_people,
                s.STORE_NAME as store_name,
                s.STORE_CATEGORY as store_category,
                s.STORE_LATITUDE as store_latitude,
                s.STORE_LONGITUDE as store_longitude,
                m.MISSION_IMAGE as mission_image,
                (6371 * ACOS(
                    COS(RADIANS($1))
                    * COS(RADIANS(s.STORE_LATITUDE))
                    * COS(RADIANS(s.STORE_LONGITUDE) - RADIANS($2))
                    + SIN(RADIANS($1))
                    * SIN(RADIANS(s.STORE_LATITUDE))
                )) as distance
            FROM MISSION m
            JOIN STORE_ s ON s.STORE_ID = m.STORE_ID
            WHERE m.MISSION_START <= CURRENT_DATE
              AND m.MISSION_END >= CURRENT_DATE
            ORDER BY distance
            LIMIT 50",
        )
        .bind(lat)
        .bind(lng)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for mission in &mut missions {
            if let (Some(image_path), Some(storage)) = (&mission.mission_image, &storage) {
                mission.mission_image = storage.generate_presigned_url(image_path).ok();
            }
        }

        missions
            .into_iter()
            .filter(|mission| mission.distance <= radius_km)
            .collect()
    }
}
