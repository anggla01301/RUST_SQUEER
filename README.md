# RUST_SQUEER

> Spring Boot/Java 기반 위치기반 미션 플랫폼 SQUEER를 Rust/Axum으로 포팅한 프로젝트입니다.

## 소개

부트캠프 중간 프로젝트로 개발중인 [SQUEER](https://github.com/anggla01301/SQUEER)의 백엔드를 Rust/Axum으로 직접 포팅하는 개인 학습 프로젝트입니다.

Java/Spring의 동작 원리를 Rust의 소유권 모델과 타입 시스템으로 재해석하며, 두 언어/프레임워크의 아키텍처 차이를 깊이 이해하는 것을 목표로 합니다.

현재 코드 리팩토링 진행 중입니다.



## 원본 프로젝트 (SQUEER)

| 항목 | 내용 |
|------|------|
| 분류 | 부트캠프 중간 프로젝트 |
| 개요 | GPS 기반 미션 인증 및 보상 플랫폼 |
| 팀 구성 | 6인 팀 / 부팀장 |
| 담당 | 인증, 미션 로직, OCI 스토리지, 클라우드 인프라 |

---

## 기술 스택

### 원본 (Java/Spring Boot)
- Spring Boot, Spring Security
- Oracle Cloud ATP, OCI Object Storage
- JWT + Refresh Token, OAuth2 (Kakao/Naver/Google)
- PortOne V2, Haversine GPS 인증
- Android (Java)

### 포팅 (Rust)
- **Runtime**: Tokio (비동기)
- **Web Framework**: Axum
- **ORM**: SQLx
- **DB**: PostgreSQL (Supabase)
- **직렬화**: Serde
- **인증**: JWT

---

## 포팅 구조

```
src/
├── main.rs         # 서버 진입점, 라우터 구성
├── state.rs        # 앱 상태 (DB 커넥션 등)
├── routes.rs       # 라우트 정의
├── middleware.rs   # JWT 인증 미들웨어
├── controller.rs   # 요청/응답 처리
├── service.rs      # 비즈니스 로직
└── repository.rs   # DB 접근 레이어
```

---

## Java → Rust 주요 변환 포인트

| Java/Spring | Rust/Axum |
|-------------|-----------|
| `@RestController` | `async fn handler()` + Router |
| `@Service` | `impl ServiceName` |
| `@Repository` | SQLx query functions |
| Spring Security Filter | Axum Middleware (tower) |
| `@Autowired` | `Extension<Arc<AppState>>` |
| `Optional<T>` | `Option<T>` |
| `throws Exception` | `Result<T, E>` |

---

## DB 마이그레이션

Oracle DB → PostgreSQL 마이그레이션 (19개 테이블)

| Oracle | PostgreSQL |
|--------|------------|
| `NUMBER` | `BIGINT` / `INTEGER` |
| `VARCHAR2` | `VARCHAR` |
| `DATE` | `TIMESTAMP` |
| Sequence | `SERIAL` / `BIGSERIAL` |

---

## 관련 프로젝트

- [SQUEER 원본 (Spring Boot)](https://github.com/anggla01301/SQUEER) — Java/Spring Boot 원본
