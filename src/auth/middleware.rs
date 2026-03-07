use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum::http::header::AUTHORIZATION;

use crate::auth::jwt::JwtUtil;

pub async fn jwt_middleware(mut request: Request, next: Next, )->Result<Response,(StatusCode,String)> {

    //request.getHeader("Authorization") 대응
    let header = request.headers().get(AUTHORIZATION);

    //토큰 없으면 그냥 통과
    //if(header==null||!header.startswith("Bearer "))
    let token = match header {
        None => return Ok(next.run(request).await), //filterChain.doFilter() 대응
        Some(h) => {
            let value = h.to_str().unwrap_or("");
            if !value.starts_with("Bearer ") {
                return Ok(next.run(request).await); //그냥 통과
            }
            value[7..].trim().to_string() //header.substring(7).trim() 대응
        }
    };

    tracing::debug!("JwtFilter Authorization header received.");

    let jwt_util = JwtUtil::new();

    if !jwt_util.validate_token(&token) {
        tracing::debug!("JwtFilter 유효하지 않은 토큰");
        return Err((StatusCode::UNAUTHORIZED, "유효하지 않은 토큰".to_string()));
    }

    let user_id = match jwt_util.get_user_id(&token) {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "유저 없음".to_string())),
    };

    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}