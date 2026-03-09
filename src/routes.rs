use axum::{Router, middleware,routing::{post, patch}};
use crate::auth::controller::*;
use crate::auth::middleware::jwt_middleware;

pub fn create_router()->Router{
    let public_routes=Router::new()
        .route("/api/auth/signup/user",post(sign_up_user))
        .route("/api/auth/login",post(login))
        .route("/api/auth/refresh",post(refresh));

    let private_routes=Router::new()
        .route("/api/auth/logout",post(logout))
        .route("/api/auth/role",patch(set_user_role))
        .layer(middleware::from_fn(jwt_middleware));

    Router::new()
        .merge(public_routes)
        .merge(private_routes)
}