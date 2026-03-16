use chrono::Local;
use sqlx::PgPool;

use crate::notification::model::UserNotification;

// 알림 서비스다.
#[derive(Clone)]
pub struct NotificationService {
    pool: PgPool,
}

impl NotificationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn send(
        &self,
        receiver_id: i64,
        sender_id: Option<i64>,
        title: &str,
        content: &str,
    ) -> Option<UserNotification> {
        sqlx::query_as::<_, UserNotification>(
            "INSERT INTO USER_NOTIFICATION (
                NOTIFICATION_RECEIVER_ID,
                NOTIFICATION_SENDER_ID,
                NOTIFICATION_TITLE,
                NOTIFICATION_CONTENT,
                NOTIFICATION_CREATEDAT,
                NOTIFICATION_IS_READ
            )
            VALUES ($1, $2, $3, $4, $5, 0)
            RETURNING
                NOTIFICATION_ID as notification_id,
                NOTIFICATION_TITLE as notification_title,
                NOTIFICATION_CONTENT as notification_content,
                NOTIFICATION_CREATEDAT as created_at,
                NOTIFICATION_IS_READ as notification_is_read,
                NOTIFICATION_RECEIVER_ID as notification_receiver_id,
                NOTIFICATION_SENDER_ID as notification_sender_id",
        )
        .bind(receiver_id)
        .bind(sender_id)
        .bind(title)
        .bind(content)
        .bind(Local::now().naive_local())
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn get_my_notifications(&self, receiver_id: i64) -> Vec<UserNotification> {
        sqlx::query_as::<_, UserNotification>(
            "SELECT
                NOTIFICATION_ID as notification_id,
                NOTIFICATION_TITLE as notification_title,
                NOTIFICATION_CONTENT as notification_content,
                NOTIFICATION_CREATEDAT as created_at,
                NOTIFICATION_IS_READ as notification_is_read,
                NOTIFICATION_RECEIVER_ID as notification_receiver_id,
                NOTIFICATION_SENDER_ID as notification_sender_id
            FROM USER_NOTIFICATION
            WHERE NOTIFICATION_RECEIVER_ID = $1
            ORDER BY NOTIFICATION_CREATEDAT DESC",
        )
        .bind(receiver_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn mark_read(&self, receiver_id: i64, notification_id: i64) -> bool {
        sqlx::query(
            "UPDATE USER_NOTIFICATION
            SET NOTIFICATION_IS_READ = 1
            WHERE NOTIFICATION_ID = $1 AND NOTIFICATION_RECEIVER_ID = $2",
        )
        .bind(notification_id)
        .bind(receiver_id)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false)
    }
}
