//! 위치 정보 모델과 좌표 DTO를 정의할 파일이다.
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NearbyMissionResponseDto {
    #[serde(rename = "missionId")]
    pub mission_id: i64,
    #[serde(rename = "missionTitle")]
    pub mission_title: String,
    #[serde(rename = "missionInfo")]
    pub mission_info: Option<String>,
    #[serde(rename = "missionStart")]
    pub mission_start: Option<NaiveDate>,
    #[serde(rename = "missionEnd")]
    pub mission_end: Option<NaiveDate>,
    #[serde(rename = "missionPeople")]
    pub mission_people: i32,
    #[serde(rename = "storeName")]
    pub store_name: String,
    #[serde(rename = "storeCategory")]
    pub store_category: String,
    #[serde(rename = "storeLatitude")]
    pub store_latitude: f64,
    #[serde(rename = "storeLongitude")]
    pub store_longitude: f64,
    #[serde(rename = "missionImage")]
    pub mission_image: Option<String>,
    pub distance: f64,
}
