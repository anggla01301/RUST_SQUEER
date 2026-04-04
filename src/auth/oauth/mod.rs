//! 소셜 로그인 전용 하위 모듈이다.
//! 공급자별 사용자 정보 조회와 로그인 성공 후 토큰 발급 흐름을 다룬다.

pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod service;
pub(crate) mod dto;
