use chrono::Local;
use sqlx::PgPool;

use crate::servicecenter::model::{CreateInquiryRequest, InquiryResponse, ServiceCenter};

// 고객센터 문의 서비스다.
#[derive(Clone)]
pub struct ServiceCenterService {
    pool: PgPool,
}

impl ServiceCenterService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: i64, req: CreateInquiryRequest) -> Option<InquiryResponse> {
        let inquiry = sqlx::query_as::<_, ServiceCenter>(
            "INSERT INTO SERVICE_CENTER (
                USER_ID,
                SERVICE_CENTER_TITLE,
                SERVICE_CENTER_CONTENT,
                SERVICE_CENTER_STATUS,
                SERVICE_CENTER_DATE
            )
            VALUES ($1, $2, $3, 'OPEN', $4)
            RETURNING
                SERVICE_CENTER_ID as service_center_id,
                USER_ID as user_id,
                SERVICE_CENTER_TITLE as title,
                SERVICE_CENTER_CONTENT as content,
                SERVICE_CENTER_STATUS as status,
                SERVICE_CENTER_DATE as created_date,
                SERVICE_CENTER_RESPONSE_CONTENT as response_content,
                SERVICE_CENTER_RESPONSE_DATE as response_date",
        )
        .bind(user_id)
        .bind(req.title)
        .bind(req.content)
        .bind(Local::now().date_naive())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;
        Some(self.to_response(inquiry))
    }

    pub async fn my(&self, user_id: i64) -> Vec<InquiryResponse> {
        sqlx::query_as::<_, ServiceCenter>(
            "SELECT
                SERVICE_CENTER_ID as service_center_id,
                USER_ID as user_id,
                SERVICE_CENTER_TITLE as title,
                SERVICE_CENTER_CONTENT as content,
                SERVICE_CENTER_STATUS as status,
                SERVICE_CENTER_DATE as created_date,
                SERVICE_CENTER_RESPONSE_CONTENT as response_content,
                SERVICE_CENTER_RESPONSE_DATE as response_date
            FROM SERVICE_CENTER
            WHERE USER_ID = $1
            ORDER BY SERVICE_CENTER_DATE DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| self.to_response(row))
        .collect()
    }

    pub async fn answer(&self, inquiry_id: i64, answer: String) -> Option<InquiryResponse> {
        let updated = sqlx::query_as::<_, ServiceCenter>(
            "UPDATE SERVICE_CENTER
            SET SERVICE_CENTER_RESPONSE_CONTENT = $1,
                SERVICE_CENTER_RESPONSE_DATE = $2,
                SERVICE_CENTER_STATUS = 'DONE'
            WHERE SERVICE_CENTER_ID = $3
            RETURNING
                SERVICE_CENTER_ID as service_center_id,
                USER_ID as user_id,
                SERVICE_CENTER_TITLE as title,
                SERVICE_CENTER_CONTENT as content,
                SERVICE_CENTER_STATUS as status,
                SERVICE_CENTER_DATE as created_date,
                SERVICE_CENTER_RESPONSE_CONTENT as response_content,
                SERVICE_CENTER_RESPONSE_DATE as response_date",
        )
        .bind(answer)
        .bind(Local::now().date_naive())
        .bind(inquiry_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;
        Some(self.to_response(updated))
    }

    fn to_response(&self, sc: ServiceCenter) -> InquiryResponse {
        InquiryResponse {
            id: sc.service_center_id,
            title: sc.title,
            content: sc.content,
            status: sc.status,
            created_date: sc.created_date,
            response_content: sc.response_content,
            response_date: sc.response_date,
        }
    }
}
