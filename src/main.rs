use anyhow::Context;

mod achievement;
mod attendance;
mod auth;
mod common;
mod config;
mod coupon;
mod event;
mod location;
mod membership;
mod mission;
mod notification;
mod openapi;
mod payment;
mod randombox;
mod ranking;
mod routes;
mod search;
mod servicecenter;
mod state;
mod status;
mod store;
mod user;

use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL 없음")?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("DB 연결 실패")?;  

    // main은 인프라를 준비하고, AppState에 넘겨 전체 의존성을 한 번에 조립한다.
    let state = AppState::new(pool)?;
    let app = create_router().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7777")
        .await
        .context("포트 7777 바인딩 실패")?;
    tracing::info!("서버 시작: http://localhost:7777");

    axum::serve(listener, app)
        .await
        .context("HTTP 서버 실행 실패")?;
    Ok(())
}
