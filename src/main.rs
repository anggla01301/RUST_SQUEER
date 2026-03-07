use axum::Router;


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


#[tokio::main]
async fn main() {
    //로깅 초기화
    tracing_subscriber::fmt::init();

    //.env로드
    dotenv::dotenv().ok();

    //라우터(일반 빈 라우터)
    let app=Router::new();

    //서버 실행
    let listener=tokio::net::TcpListener::bind("0.0.0.0:7777").await.unwrap();
    tracing::info!("서버 시작: http://localhost:7777:");
    axum::serve(listener, app).await.unwrap();


}