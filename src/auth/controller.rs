use axum::{
    extract::{Extension,Json},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use std::collections::HashMap;
use std::env;

use crate::auth::model::{LoginRequestDto, StoreSignUpRequestDto, UserSignUpRequestDto};
use crate::auth::repository::UserRepository;
use crate::auth::jwt::JwtUtil;
use crate::auth::service::{AuthError, AuthService};

//AuthError->HTTP 상태코드 변환
//@ControllerAdvice+@ExceptionHandler 대응
impl IntoResponse for AuthError{
    fn into_response(self)->Response{
        let (status, message)=match self{
            AuthError::EmailDuplicated=>(StatusCode::CONFLICT,"이메일 중복"),
            AuthError::SocialEmailUpDuplicated=>(StatusCode::CONFLICT,"소셜 이메일 중복"),
            AuthError::EmailNotFound=>(StatusCode::NOT_FOUND,"이메일 없음"),
            AuthError::PasswordNotMatch=>(StatusCode::UNAUTHORIZED,"비밀번호 불일치"),
            AuthError::AccountWithdraw=>(StatusCode::FORBIDDEN,"탈퇴한 계정"),
            AuthError::UserNotFound=>(StatusCode::NOT_FOUND,"유저 없음"),
            AuthError::UserInfoNotFound=>(StatusCode::NOT_FOUND,"유저 정보 없음"),
        };
        (status, message).into_response()
    }
}

//@PostMapping("/signup/user") 대응
pub async fn sign_up_user(
    Extension(auth_service): Extension<AuthService>,
    Json(dto): Json<UserSignUpRequestDto>, //@RequestBody @Valid 대응
)->impl IntoResponse{
    match auth_service.sign_up_user(dto).await{
        Ok(_)=>(StatusCode::OK, "회원가입 성공").into_response(),
        Err(e)=>e.into_response(),
    }
}

//@PostMapping("/signup/store") 대응
pub async fn sign_up_store(
    Extension(auth_service): Extension<AuthService>,
    Json(dto): Json<StoreSignUpRequestDto>,
)->impl IntoResponse{
    match auth_service.sign_up_store(dto).await{
        Ok(_)=>(StatusCode::OK,"회원가입 성공").into_response(),
        Err(e)=>e.into_response(),
    }
}

//@PostMapping("/login") 대응
pub async fn login(
    Extension(auth_service): Extension<AuthService>,
    Json(dto): Json<LoginRequestDto>,
)->impl IntoResponse{
    match auth_service.login(dto).await{
        Ok(response)=>{
            //refreshToken->HttpOnly 쿠키에 저장
            //Cookie cookie=new Cookie("refreshToken",...) 대응
            let refresh_expiration=env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or("604800000".to_string())
                .parse::<i64>()
                .unwrap_or(604800000);
            let max_age=refresh_expiration/1000;//밀리초->초

            let cookie=format!(
                "refreshToken={}; HttpOnly; Path=/; Max-Age={}",
                response.refresh_token,max_age
            );
            let mut headers=HeaderMap::new();
            headers.insert(SET_COOKIE,cookie.parse().unwrap());

            (StatusCode::OK,headers,Json(response)).into_response()
        }
        Err(e)=>e.into_response(),
    }
}

//@PostMapping("/role") 대응
pub async fn set_user_role(
    Extension(auth_service): Extension<AuthService>,
    Extension(user_id): Extension<i64>,
    Json(request): Json<HashMap<String,String>>,
)->impl IntoResponse{
    let user_type=match request.get("userType"){
        Some(t)=>t.clone(),
        None=>return (StatusCode::BAD_REQUEST,"userType 없음").into_response(),
    };

    match auth_service.update_user_type(user_id, &user_type).await{
        Ok(_)=>(StatusCode::OK, "역할 설정 완료").into_response(),
        Err(e)=>e.into_response(),
    }
}

//@PostMapping("/logout") 대응
pub async fn logout(
    Extension(auth_service): Extension<AuthService>,
    Extension(user_id): Extension<i64>, //SecurityContextHolder 대응
)->impl IntoResponse{
    match auth_service.logout(user_id).await{
        Ok(_)=>{
            //쿠키 만료 처리
            //cookie.setMaxAge(0) 대응
            let cookie="refreshToken=; HttpOnly; Path=/; Max-Age=0".to_string();
            let mut headers=HeaderMap::new();
            headers.insert(SET_COOKIE,cookie.parse().unwrap());

            (StatusCode::OK, headers,"로그아웃 성공").into_response()
        }
        Err(e)=>e.into_response(),
    }
}

//@PostMapping("/refresh") 대응
pub async fn refresh(
    Extension(user_repository): Extension<UserRepository>,
    headers:HeaderMap,
)-> impl IntoResponse{
    let jwt_util=JwtUtil::new();

    //쿠키에서 refreshToken 꺼내기
    //request.getCookies() 대응
    let refresh_token=headers.get("cookie").and_then(|v|v.to_str().ok())
        .and_then(|cookies|{
            cookies.split(';').find_map(|c|{
                let c=c.trim();
                c.strip_prefix("refreshToken=")//"refreshToekn=" 제거
            })
        })
        .map(|s| s.to_string());
    //refreshToken null 체크 + 유효성 검사
    let refresh_token=match refresh_token{
        Some(t) if jwt_util.validate_token(&t)=>t,
        _=>return (StatusCode::UNAUTHORIZED,"유효하지 않은 리프레시 토큰").into_response(),
    };
    //userId 추출
    let user_id=match jwt_util.get_user_id(&refresh_token){
        Some(id)=>id,
        None=>return(StatusCode::UNAUTHORIZED,"유효하지 않은 토큰").into_response(),
    };

    //DB에서 유저 조회
    let user=match user_repository.find_by_id(user_id).await{
        Some(u)=>u,
        None=>return (StatusCode::NOT_FOUND,"유저 없음").into_response(),
    };
    //DB의 refreshToken과 비교(탈취 감지)
    let stored_token=user.refresh_token.as_deref().unwrap_or("");
    if refresh_token!=stored_token{
        return (StatusCode::UNAUTHORIZED,"토큰 불일치").into_response();
    }

    //새 accessToken 발급
    let new_access_token=jwt_util.generate_token(
        user_id,
        user.user_email.as_deref().unwrap_or(""),
        &user.user_type,
    );
    //Map.of("accessToken", newAccessToken) 대응
    let body=HashMap::from([("accessToken",new_access_token)]);
    (StatusCode::OK, Json(body)).into_response()

}
