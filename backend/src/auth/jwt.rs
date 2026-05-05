//! JWT token generation and validation
//!
//! Uses HS256 algorithm with configurable expiration.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// JWT claims payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User email
    pub email: String,
    /// User display name
    pub username: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: Uuid, email: String, username: String, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        Self {
            sub: user_id,
            email,
            username,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }

    /// Get the user ID from claims
    #[allow(dead_code)] // Convenient accessor for future use
    pub fn user_id(&self) -> Uuid {
        self.sub
    }
}

/// JWT service for token operations
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

impl JwtService {
    /// Create a new JWT service with the given secret
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration_hours,
        }
    }

    /// Generate a new JWT token for a user
    pub fn generate_token(
        &self,
        user_id: Uuid,
        email: &str,
        username: &str,
    ) -> AppResult<String> {
        let claims = Claims::new(
            user_id,
            email.to_string(),
            username.to_string(),
            self.expiration_hours,
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> AppResult<TokenData<Claims>> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Unauthorized("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::Unauthorized("Invalid token format".to_string())
                }
                _ => AppError::Unauthorized(format!("Token validation failed: {}", e)),
            }
        })
    }

    /// Validate and decode a JWT token without enforcing expiration.
    ///
    /// Signature is still verified and claims are decoded safely.
    pub fn validate_token_allow_expired(&self, token: &str) -> AppResult<TokenData<Claims>> {
        let mut validation = Validation::default();
        validation.validate_exp = false;

        decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::Unauthorized("Invalid token format".to_string())
                }
                _ => AppError::Unauthorized(format!("Token validation failed: {}", e)),
            }
        })
    }

    /// Extract claims from a token without full validation (for debugging)
    #[allow(dead_code)] // Useful for debugging and testing
    pub fn decode_without_validation(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Unauthorized(format!("Failed to decode token: {}", e)))
    }
}

/// Standalone function to validate a token with a given secret
/// Used by WebSocket upgrade handler
pub fn validate_token(token: &str, secret: &str) -> AppResult<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::default();
    validation.validate_exp = true;

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Unauthorized("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::Unauthorized("Invalid token format".to_string())
                }
                _ => AppError::Unauthorized(format!("Token validation failed: {}", e)),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let service = JwtService::new("test-secret-key-at-least-32-chars", 24);
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let username = "testuser";

        let token = service.generate_token(user_id, email, username).unwrap();
        let decoded = service.validate_token(&token).unwrap();

        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.email, email);
        assert_eq!(decoded.claims.username, username);
    }

    #[test]
    fn test_expired_token() {
        let service = JwtService::new("test-secret-key-at-least-32-chars", -1); // Expired immediately
        let user_id = Uuid::new_v4();

        let token = service.generate_token(user_id, "test@example.com", "test").unwrap();
        let result = service.validate_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_allow_expired_token() {
        let service = JwtService::new("test-secret-key-at-least-32-chars", -1); // Expired immediately
        let user_id = Uuid::new_v4();

        let token = service.generate_token(user_id, "test@example.com", "test").unwrap();
        let result = service.validate_token_allow_expired(&token);

        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.claims.sub, user_id);
    }

    #[test]
    fn test_allow_expired_rejects_tampered_token() {
        let service = JwtService::new("test-secret-key-at-least-32-chars", -1); // Expired immediately
        let user_id = Uuid::new_v4();

        let token = service.generate_token(user_id, "test@example.com", "test").unwrap();
        let mut tampered = token.clone();
        tampered.push('x');

        let result = service.validate_token_allow_expired(&tampered);
        assert!(result.is_err());
    }
}
