// src/lib/auth.rs

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Cookie, authorization::Bearer},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{AppConfig, AppError};

#[derive(Clone)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config.jwt_secret.as_bytes())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Missing authorization header")]
    MissingToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Password hashing error: {0}")]
    PasswordHashError(String),
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => AppError::Unauthorized(err.to_string()),
            AuthError::MissingToken | AuthError::InvalidToken | AuthError::TokenExpired => {
                AppError::Unauthorized(err.to_string())
            }
            AuthError::PasswordHashError(msg) => AppError::InternalServerError(msg),
        }
    }
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

// Optional authentication - doesn't fail if no token present
pub struct OptionalUser {
    pub user: Option<AuthenticatedUser>,
}

impl FromRequestParts<crate::AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to get token from Authorization header first
        let token = if let Ok(TypedHeader(Authorization(bearer))) =
            parts.extract::<TypedHeader<Authorization<Bearer>>>().await
        {
            bearer.token().to_string()
        } else if let Ok(TypedHeader(cookie)) = parts.extract::<TypedHeader<Cookie>>().await {
            // Fall back to cookie
            cookie
                .get("authToken")
                .ok_or(AuthError::MissingToken)?
                .to_string()
        } else {
            return Err(AuthError::MissingToken.into());
        };

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 60;

        let token_data = decode::<Claims>(&token, &state.jwt_keys.decoding, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        let user_id =
            Uuid::parse_str(&token_data.claims.sub).map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthenticatedUser { user_id })
    }
}

impl FromRequestParts<crate::AppState> for OptionalUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalUser { user: Some(user) }),
            Err(_) => Ok(OptionalUser { user: None }),
        }
    }
}

pub fn generate_token(user_id: Uuid, keys: &Keys) -> Result<String, AuthError> {
    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let header = Header::new(Algorithm::HS256);

    encode(&header, &claims, &keys.encoding)
        .map_err(|err| AuthError::PasswordHashError(err.to_string()))
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| AuthError::PasswordHashError(err.to_string()))?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|err| AuthError::PasswordHashError(err.to_string()))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// API Key authentication
pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LEN: usize = 32;
    let mut rng = rand::rng();

    let key: String = (0..KEY_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("crdev_{}", key)
}

pub fn hash_api_key(key: &str) -> Result<String, AuthError> {
    hash_password(key)
}

pub fn verify_api_key(key: &str, hash: &str) -> Result<bool, AuthError> {
    verify_password(key, hash)
}

// Extractor for API key authentication
pub struct ApiKeyUser {
    pub user_id: Uuid,
}

impl FromRequestParts<crate::AppState> for ApiKeyUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to get API key from X-API-Key header
        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Find all API keys and check each one
        let mut rows = conn
            .query("SELECT id, user_id, key_hash, expires_at FROM api_keys", ())
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut found_user_id: Option<(Uuid, Uuid)> = None;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let key_id: String = row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            let user_id: String = row
                .get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            let key_hash: String = row
                .get(2)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            let expires_at: Option<String> = row.get(3).ok();

            // Check if key is expired
            if let Some(exp) = expires_at
                && let Ok(exp_date) = chrono::DateTime::parse_from_rfc3339(&exp)
                && exp_date.with_timezone(&Utc) < Utc::now()
            {
                continue;
            }

            // Verify the API key
            if verify_api_key(api_key, &key_hash)? {
                let key_uuid = Uuid::parse_str(&key_id)
                    .map_err(|_| AppError::InternalServerError("Invalid key ID".to_string()))?;
                let user_uuid = Uuid::parse_str(&user_id)
                    .map_err(|_| AppError::InternalServerError("Invalid user ID".to_string()))?;
                found_user_id = Some((key_uuid, user_uuid));
                break;
            }
        }

        let (key_id, user_id) = found_user_id.ok_or(AuthError::InvalidToken)?;

        // Update last_used_at
        let now = Utc::now();
        conn.execute(
            "UPDATE api_keys SET last_used_at = ? WHERE id = ?",
            libsql::params![now.to_rfc3339(), key_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(ApiKeyUser { user_id })
    }
}
