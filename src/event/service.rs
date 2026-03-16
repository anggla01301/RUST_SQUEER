use chrono::Local;
use sqlx::PgPool;

use crate::event::model::Event;

// 이벤트 조회 서비스다.
#[derive(Clone)]
pub struct EventService {
    pool: PgPool,
}

impl EventService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_running_events(&self) -> Vec<Event> {
        sqlx::query_as::<_, Event>(
            "SELECT
                EVENTID as event_id,
                TITLE as title,
                CONTENT as content,
                STARTDATE as start_date,
                ENDDATE as end_date
            FROM EVENT
            WHERE STARTDATE <= $1 AND ENDDATE >= $1
            ORDER BY STARTDATE DESC",
        )
        .bind(Local::now().naive_local())
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn get_closed_events(&self) -> Vec<Event> {
        sqlx::query_as::<_, Event>(
            "SELECT
                EVENTID as event_id,
                TITLE as title,
                CONTENT as content,
                STARTDATE as start_date,
                ENDDATE as end_date
            FROM EVENT
            WHERE ENDDATE < $1
            ORDER BY ENDDATE DESC",
        )
        .bind(Local::now().naive_local())
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}
