use chrono::{Utc};
use jsonwebtoken::{
    decode, encode,
    Algorithm,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ACCESS_TOKEN_EXPIRY, REFRESH_TOKEN_EXPIRY};
#[derive(thiserror::Error, Debug)]
pub enum JwtError {
    #[error("invalid token")]
    InvalidToken,

    #[error("token expired")]
    Expired,

    #[error("failed to create token")]
    CreationFailed,

    #[error("failed to create token")]
    Jwt(jsonwebtoken::errors::Error),
}

#[derive(Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub token_type: TokenType,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtService {
    pub secret: String,
}

impl JwtService {
    pub fn create_token(&self, user_id: String, token_type: TokenType, expires_in: usize) -> String {
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id,
            token_type,
            exp: expires_in,
            iat: now,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
    }

    pub fn create_access_token(&self, user_id: Uuid) -> String {
        self.create_token(
            user_id.to_string(),
            TokenType::Access,
            (Utc::now() + ACCESS_TOKEN_EXPIRY).timestamp() as usize,
        )
    }

    pub fn create_refresh_token(&self, user_id: Uuid) -> String {
        self.create_token(
            user_id.to_string(),
            TokenType::Refresh,
            (Utc::now() + REFRESH_TOKEN_EXPIRY).timestamp() as usize,
        )
    }

    pub fn decode_token(&self, token: String) -> Result<(Claims, String), JwtError> {
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(JwtError::Jwt)?;

        let token_type = match token_data.claims.token_type {
            TokenType::Access => "access".to_string(),
            TokenType::Refresh => "refresh".to_string(),
        };

        Ok((token_data.claims, token_type))
    }
}