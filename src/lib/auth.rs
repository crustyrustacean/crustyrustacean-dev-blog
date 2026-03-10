// src/lib/auth.rs

//! Authentication and authorization utilities.
//!
//! This module provides JWT token handling, password hashing,
//! and authentication extractors for Actix Web.

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use actix_web::http::header::{AUTHORIZATION, COOKIE};
use thiserror::Error;
use uuid::Uuid;
use std::future::{ready, Ready};

use crate::{ApiError, configuration::JwtSettings};

/// JWT encoding and decoding keys.
#[derive(Clone)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    /// Create keys from a secret string.
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    /// Create keys from JWT configuration.
    pub fn from_config(config: &JwtSettings) -> Self {
        Self::new(config.secret.expose_secret().as_bytes())
    }
}

/// JWT claims structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// Expiration time.
    pub exp: i64,
    /// Issued at time.
    pub iat: i64,
}

/// Authentication error types.
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

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => ApiError::Unauthorized(err.to_string()),
            AuthError::MissingToken | AuthError::InvalidToken | AuthError::TokenExpired => {
                ApiError::Unauthorized(err.to_string())
            }
            AuthError::PasswordHashError(msg) => ApiError::InternalServerError(msg),
        }
    }
}

/// Authenticated user information extracted from JWT.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub role: crate::models::Role,
}

/// Optional authentication - doesn't fail if no token present.
#[derive(Clone, Debug)]
pub struct OptionalUser {
    pub user: Option<AuthenticatedUser>,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Try to get token from Authorization header first
        let token = if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    Some(auth_str[7..].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Some(cookie_header) = req.headers().get(COOKIE) {
            // Fall back to cookie
            if let Ok(cookie_str) = cookie_header.to_str() {
                cookie_str
                    .split(';')
                    .find_map(|c| {
                        let c = c.trim();
                        if c.starts_with("authToken=") {
                            Some(c[10..].to_string())
                        } else {
                            None
                        }
                    })
            } else {
                None
            }
        } else {
            None
        };

        let token = match token {
            Some(t) => t,
            None => return ready(Err(AuthError::MissingToken.into())),
        };

        // Get JWT keys from app data
        let keys = match req.app_data::<actix_web::web::Data<crate::state::AppState>>() {
            Some(state) => state.jwt_keys.clone(),
            None => return ready(Err(ApiError::InternalServerError("App state not available".to_string()))),
        };

        // Validate token
        match validate_token(&token, &keys) {
            Ok(claims) => {
                let user_id = match Uuid::parse_str(&claims.sub) {
                    Ok(id) => id,
                    Err(_) => return ready(Err(AuthError::InvalidToken.into())),
                };
                
                // TODO: Fetch user role from database
                // For now, default to Subscriber
                ready(Ok(AuthenticatedUser {
                    user_id,
                    role: crate::models::Role::Subscriber,
                }))
            }
            Err(e) => ready(Err(e)),
        }
    }
}

impl FromRequest for OptionalUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match AuthenticatedUser::from_request(req, payload) {
            Ok(user) => ready(Ok(OptionalUser { user: Some(user) })),
            Err(_) => ready(Ok(OptionalUser { user: None })),
        }
    }
}

/// Validate a JWT token and return the claims.
pub fn validate_token(token: &str, keys: &Keys) -> Result<Claims, ApiError> {
    let token_data = decode::<Claims>(token, &keys.decoding, &Validation::new(Algorithm::HS256))
        .map_err(|_| AuthError::InvalidToken)?;

    // Check if token is expired
    if token_data.claims.exp < Utc::now().timestamp() {
        return Err(AuthError::TokenExpired.into());
    }

    Ok(token_data.claims)
}

/// Generate a JWT token for a user.
pub fn generate_token(user_id: Uuid, keys: &Keys, expiration_hours: i64) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiration_hours);
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(&Header::default(), &claims, &keys.encoding)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to generate token: {}", e)))
}

/// Hash a password using Argon2.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::PasswordHashError(e.to_string()))
}

/// Verify a password against a hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Author user extractor - requires Author or Admin role.
pub struct AuthorUser {
    pub user_id: Uuid,
    pub role: crate::models::Role,
}

impl FromRequest for AuthorUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match AuthenticatedUser::from_request(req, payload) {
            Ok(user) => {
                if user.role.is_author() || user.role.is_admin() {
                    ready(Ok(AuthorUser {
                        user_id: user.user_id,
                        role: user.role,
                    }))
                } else {
                    ready(Err(ApiError::Forbidden("Author role required".to_string())))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

/// Admin user extractor - requires Admin role.
pub struct AdminUser {
    pub user_id: Uuid,
}

impl FromRequest for AdminUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match AuthenticatedUser::from_request(req, payload) {
            Ok(user) => {
                if user.role.is_admin() {
                    ready(Ok(AdminUser { user_id: user.user_id }))
                } else {
                    ready(Err(ApiError::Forbidden("Admin role required".to_string())))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

/// API key user extractor.
pub struct ApiKeyUser {
    pub user_id: Uuid,
}

impl FromRequest for ApiKeyUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Check for X-API-Key header
        let api_key = match req.headers().get("X-API-Key") {
            Some(header) => match header.to_str() {
                Ok(key) => key.to_string(),
                Err(_) => return ready(Err(ApiError::Unauthorized("Invalid API key header".to_string()))),
            },
            None => return ready(Err(ApiError::Unauthorized("Missing API key".to_string()))),
        };

        // TODO: Validate API key against database
        // For now, return an error
        ready(Err(ApiError::Unauthorized("API key validation not implemented".to_string())))
    }
}
