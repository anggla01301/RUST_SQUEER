use anyhow::{Context, Result};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;

// JWT 생성과 검증을 담당하는 유틸리티다.

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_email: String,
    pub user_type: String,
    pub token_type: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub token_type: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtUtil {
    secret: String,
    expiration: i64,
    refresh_expiration: i64,
}

impl JwtUtil {
    pub fn new() -> Result<Self> {
        let secret = env::var("JWT_SECRET").context("JWT_SECRET 없음")?;
        let expiration = env::var("JWT_EXPIRATION")
            .context("JWT_EXPIRATION 없음")?
            .parse()
            .context("JWT_EXPIRATION 숫자 파싱 실패")?;
        let refresh_expiration = env::var("JWT_REFRESH_EXPIRATION")
            .context("JWT_REFRESH_EXPIRATION 없음")?
            .parse()
            .context("JWT_REFRESH_EXPIRATION 숫자 파싱 실패")?;

        Ok(Self {
            secret,
            expiration,
            refresh_expiration,
        })
    }

    pub fn generate_token(&self, user_id: i64, user_email: &str, user_type: &str) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            user_email: user_email.to_string(),
            user_type: user_type.to_string(),
            token_type: "access".to_string(),
            iat: now,
            exp: now + self.expiration / 1000,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
    }

    pub fn generate_refresh_token(&self, user_id: i64) -> String {
        let now = Utc::now().timestamp();
        let claims = RefreshClaims {
            sub: user_id.to_string(),
            token_type: "refresh".to_string(),
            iat: now,
            exp: now + self.refresh_expiration / 1000,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
    }

    pub fn get_user_id(&self, token: &str) -> Option<i64> {
        self.decode_access(token)
            .ok()
            .and_then(|c| c.claims.sub.parse().ok())
    }

    pub fn get_user_type(&self, token: &str) -> Option<String> {
        self.decode_access(token).ok().map(|c| c.claims.user_type)
    }

    pub fn get_refresh_user_id(&self, token: &str) -> Option<i64> {
        self.decode_refresh(token)
            .ok()
            .and_then(|c| c.claims.sub.parse().ok())
    }

    pub fn validate_token(&self, token: &str) -> bool {
        self.decode_access(token).is_ok()
    }

    pub fn validate_refresh_token(&self, token: &str) -> bool {
        self.decode_refresh(token).is_ok()
    }

    fn decode_access(
        &self,
        token: &str,
    ) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
        let data = self.decode::<Claims>(token)?;
        if data.claims.token_type == "access" {
            Ok(data)
        } else {
            Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ))
        }
    }

    fn decode_refresh(
        &self,
        token: &str,
    ) -> Result<jsonwebtoken::TokenData<RefreshClaims>, jsonwebtoken::errors::Error> {
        let data = self.decode::<RefreshClaims>(token)?;
        if data.claims.token_type == "refresh" {
            Ok(data)
        } else {
            Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ))
        }
    }

    fn decode<T: DeserializeOwned>(
        &self,
        token: &str,
    ) -> Result<jsonwebtoken::TokenData<T>, jsonwebtoken::errors::Error> {
        decode::<T>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
    }
}
