use uuid::Uuid;

use crate::common::model::AppError;
use crate::config::model::AppConfig;

// 공통 파일 경로/URL 생성 서비스다.
#[derive(Clone)]
pub struct OciStorageService {
    config: AppConfig,
}

impl OciStorageService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    // 실제 업로드 전, 저장될 객체 경로를 안전하게 생성한다.
    pub fn build_object_name(&self, original_name: &str, folder: &str) -> Result<String, AppError> {
        // 확장자를 유지해야 브라우저와 스토리지가 파일 유형을 쉽게 추론할 수 있다.
        let ext = original_name
            .rsplit('.')
            .next()
            .filter(|ext| !ext.is_empty() && *ext != original_name)
            .ok_or(AppError::InvalidInput)?;
        Ok(format!("{folder}/{}.{}", Uuid::new_v4(), ext))
    }

    // OCI 공개 URL 형식을 그대로 맞춘다.
    pub fn build_object_url(&self, object_name: &str) -> Result<String, AppError> {
        // 이 함수는 실제 업로드를 수행하지 않고, DB에 저장할 최종 접근 URL만 조립한다.
        let oci = self.config.oci.as_ref().ok_or(AppError::InvalidInput)?;
        Ok(format!(
            "https://objectstorage.{}.oraclecloud.com/n/{}/b/{}/o/{}",
            oci.region, oci.namespace, oci.bucket, object_name
        ))
    }

    // 업로드 후 조회용 URL 생성 함수다.
    pub fn generate_presigned_url(&self, image_path: &str) -> Result<String, AppError> {
        // 이미 전체 URL이 저장된 경우 그대로 반환하고,
        // 객체 키만 저장된 경우에만 URL을 다시 조립한다.
        if image_path.is_empty() {
            return Err(AppError::InvalidInput);
        }
        if image_path.contains("/o/") {
            return Ok(image_path.to_string());
        }
        self.build_object_url(image_path)
    }
}
