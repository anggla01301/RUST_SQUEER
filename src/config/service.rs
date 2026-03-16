use std::env;

use crate::common::model::AppError;
use crate::config::model::{AppConfig, OciConfig, PortOneConfig, RuntimeConfigResponse};

// 런타임 설정 로딩 서비스다.
#[derive(Clone)]
pub struct ConfigService {
    pub config: AppConfig,
}

impl ConfigService {
    pub fn from_env() -> Result<Self, AppError> {
        // Spring의 @Value / @Configuration 역할을 단순한 환경 변수 로더로 치환한 형태다.
        let oci_region = env::var("OCI_CONFIG_REGION").ok();
        let oci_namespace = env::var("OCI_OBJECT_STORAGE_NAMESPACE").ok();
        let oci_bucket = env::var("OCI_OBJECT_STORAGE_BUCKET").ok();

        // region / namespace / bucket이 모두 있어야 OCI 설정이 유효하다고 본다.
        let oci = match (oci_region, oci_namespace, oci_bucket) {
            (Some(region), Some(namespace), Some(bucket)) => Some(OciConfig {
                user: env::var("OCI_CONFIG_USER").ok(),
                fingerprint: env::var("OCI_CONFIG_FINGERPRINT").ok(),
                tenancy: env::var("OCI_CONFIG_TENANCY").ok(),
                region,
                key_file: env::var("OCI_CONFIG_KEY_FILE").ok(),
                namespace,
                bucket,
            }),
            _ => None,
        };

        // PortOne은 최소한 base URL 또는 store ID가 있어야 설정이 존재한다고 판단한다.
        let portone =
            if env::var("PORTONE_BASE_URL").is_ok() || env::var("PORTONE_STORE_ID").is_ok() {
                Some(PortOneConfig {
                    api_secret: env::var("PORTONE_API_SECRET").ok(),
                    base_url: env::var("PORTONE_BASE_URL").ok(),
                    store_id: env::var("PORTONE_STORE_ID").ok(),
                    channel_key: env::var("PORTONE_CHANNEL_KEY").ok(),
                })
            } else {
                None
            };

        Ok(Self {
            config: AppConfig {
                cors_allowed_origin: env::var("CORS_ALLOWED_ORIGIN").ok(),
                oci,
                portone,
            },
        })
    }

    pub fn get_runtime_config(&self) -> RuntimeConfigResponse {
        // 외부 노출용 응답에서는 민감값을 빼고 운영 확인에 필요한 필드만 남긴다.
        RuntimeConfigResponse {
            cors_allowed_origin: self.config.cors_allowed_origin.clone(),
            oci_region: self.config.oci.as_ref().map(|oci| oci.region.clone()),
            oci_bucket: self.config.oci.as_ref().map(|oci| oci.bucket.clone()),
            portone_base_url: self
                .config
                .portone
                .as_ref()
                .and_then(|portone| portone.base_url.clone()),
            portone_store_id: self
                .config
                .portone
                .as_ref()
                .and_then(|portone| portone.store_id.clone()),
        }
    }
}
