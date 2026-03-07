use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

//Claims->JWT payload 구조체
//Java의 Claims 대응
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    //subject(userId)
    pub sub: String,
    //.claim("userEmail")대응
    pub user_email: String,
    //.claim("userType")대응
    pub user_type: String,
    //issuedAt
    pub iat: i64,
    //expiration
    pub exp: i64,
}

//RefreshClaims->refresh token payload
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    //userId
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

pub struct JwtUtil {
    secret: String,
    //밀리초
    expiration: i64,
    //밀리초
    refresh_expiration: i64,
}

impl JwtUtil {
    //@Value 생성자 대응->.env에서 읽어서 생성
    pub fn new() -> Self {
        Self {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET 없음"),
            expiration: env::var("JWT_EXPIRATION")
                .expect("JWT_EXPIRATION 없음")
                .parse()
                .unwrap(),
            refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                .expect("JWT_REFRESH_EXPIRATION 없음")
                .parse()
                .unwrap(),
        }
    }
    //generateToken()대응
    pub fn generate_token(&self, user_id: i64, user_email: &str, user_type: &str) -> String {
        let now = Utc::now().timestamp();//현재 시간(초)
        let exp=now+self.expiration/1000;//밀리초->초 변환

        let claims=Claims{
            sub: user_id.to_string(),
            user_email: user_email.to_string(),
            user_type: user_type.to_string(),
            iat: now,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        ).unwrap()

    }
    //generateRefreshToken()대응
    pub fn generate_refresh_token(&self, user_id: i64)->String{
        let now=Utc::now().timestamp();
        let exp=now+self.refresh_expiration/1000;

        let claims =RefreshClaims{
            sub: user_id.to_string(),
            iat: now,
            exp,

        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),

        ).unwrap()
    }
    //getUserId() 대응
    pub fn get_user_id(&self, token: &str)->Option<i64>{
        self.get_claims(token)
            .ok()
            .and_then(|c|c.claims.sub.parse().ok())
    }
    //getUserType() 대응
    pub fn get_user_type(&self, token: &str)->Option<String>{
        self.get_claims(token)
            .ok()
            .map(|c| c.claims.user_type)
    }

    //validateToken() 대응
    pub fn validate_token(&self, token: &str)->bool{
        match self.get_claims(token){
            Ok(_)=>true,
            Err(e)=>{
                //Java의 catch 블록들 대응
                match e.kind(){
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature=>{
                        tracing::debug!("JwtUtil 만료된 토큰");
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature=>{
                        tracing::warn!("JwtUtil 서명 불일치");
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken=>{
                        tracing::debug!("JwtUtil 잘못된 형식의 토큰");
                    }
                    _=>{
                        tracing::warn!("JwtUtil JWT 오류: {:?}",e);
                    }
                }
                false
            }
        }

    }
    //getClaims() 대응(private)
    fn get_claims(&self, token: &str)->Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error>{
        decode::<Claims>(
            token, &DecodingKey::from_secret(self.secret.as_bytes()),&Validation::new(Algorithm::HS256),

        )

    }

}
