// ============================================================
// auth/oauth/dto.rs — 프론트와 주고받는 OAuth DTO 만 모아둔다.
// 내부 처리용 구조체는 model.rs 에 있다.
// ============================================================

use serde::{Deserialize,Serialize};
use utoipa::ToSchema;

// 프론트가 공급자 access token을 직접 넘기는 요청 DTO
#[derive(Debug,Serialize,Deserialize,ToSchema)]
pub struct SocialLoginRequest{
    pub access_token: String,
}
