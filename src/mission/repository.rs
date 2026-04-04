// ============================================================
// mission/repository.rs — DB 쿼리 전담
// MissionRepository + MissionParticipateRepository
// + MissionBookmarkRepository 를 하나로 합쳤다.
// ============================================================

use sqlx::PgPool;

use super::model::{
    Mission,MissionBookmark,MissionParticipate,
    MissionParticipateWithDetail, MissionWithStore,
};

#[derive(Clone)]
pub struct MissionRepository {
    pool: PgPool, //데이터베이스에 연결하는 통로(pool)를 가지고 있다는 뜻
}

impl MissionRepository {
    pub fn new(pool: PgPool) -> Self{ //자바의 인자생성자와 같음
        Self {pool}
    }

    // ── Mission 기본 CRUD ────────────────────────────────────

    // missionRepository.findById()
    pub async fn find_by_id(&self, mission_id: i64) -> Option<Mission> {
        sqlx::query_as::<_, Mission>(
            "SELECT mission_id, mission_title, mission_start, mission_end,
       mission_info, mission_people, mission_code, mission_image, store_id, is_pull_up,
       store_reward_given_yn, mission_created_at
        FROM mission
        WHERE mission_id = $1", //$1은 파라미터 자리

        )
            .bind(mission_id)//위에서 만든 $1에 이 값이 들어감
            .fetch_optional(&self.pool)
            .await
            .ok().flatten()
    }
}