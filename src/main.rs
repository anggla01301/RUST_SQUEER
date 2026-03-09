use axum::Router;
use axum::extract::Extension;


mod achievement;
mod auth;
mod common;
mod config;
mod coupon;
mod event;
mod location;
mod membership;
mod mission;
mod notification;
mod payment;
mod randombox;
mod ranking;
mod search;
mod servicecenter;
mod status;
mod store;
mod user;
mod state;
mod routes;

use state::AppState;
use routes::create_router;

#[tokio::main]
async fn main() {
    //로깅 초기화
    tracing_subscriber::fmt::init();

    //.env로드
    dotenv::dotenv().ok();

    //DB 연결
    let database_url=std::env::var("DATABASE_URL").expect("DATABASE_URL 없음");
    let pool=sqlx::postgres::Pg


    //서버 실행
    let listener=tokio::net::TcpListener::bind("0.0.0.0:7777").await.unwrap();
    tracing::info!("서버 시작: http://localhost:7777:");
    axum::serve(listener, app).await.unwrap();


}