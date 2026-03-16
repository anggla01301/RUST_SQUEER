use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// OCI Object Storage 설정이다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    pub user: Option<String>,
    pub fingerprint: Option<String>,
    pub tenancy: Option<String>,
    pub region: String,
    pub key_file: Option<String>,
    pub namespace: String,
    pub bucket: String,
}

// PortOne 결제 설정이다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortOneConfig {
    pub api_secret: Option<String>,
    pub base_url: Option<String>,
    pub store_id: Option<String>,
    pub channel_key: Option<String>,
}

// 앱 전역 설정 묶음이다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub cors_allowed_origin: Option<String>,
    pub oci: Option<OciConfig>,
    pub portone: Option<PortOneConfig>,
}

// 외부 노출용 안전한 설정 요약이다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RuntimeConfigResponse {
    pub cors_allowed_origin: Option<String>,
    pub oci_region: Option<String>,
    pub oci_bucket: Option<String>,
    pub portone_base_url: Option<String>,
    pub portone_store_id: Option<String>,
}
