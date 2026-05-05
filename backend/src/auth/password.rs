//! Password hashing and verification
//!
//! Uses Argon2id for secure password hashing (OWASP recommended).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::{AppError, AppResult};

/// Service for password operations
#[derive(Clone, Default)]
pub struct PasswordService;

impl PasswordService {
    /// Create a new password service
    pub fn new() -> Self {
        Self
    }

    /// Hash a password using Argon2id
    ///
    /// Returns the PHC string format hash (includes algorithm, params, salt, and hash)
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        // Validate password strength
        self.validate_password_strength(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
    }

    /// Verify a password against a stored hash
    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Validate password strength requirements
    ///
    /// Requirements:
    /// - At least 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one digit
    fn validate_password_strength(&self, password: &str) -> AppResult<()> {
        if password.len() < 8 {
            return Err(AppError::Validation(
                "Password must be at least 8 characters long".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::Validation(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AppError::Validation(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AppError::Validation(
                "Password must contain at least one digit".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let service = PasswordService::new();
        let password = "SecurePass123";

        let hash = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("WrongPassword1", &hash).unwrap());
    }

    #[test]
    fn test_password_too_short() {
        let service = PasswordService::new();
        let result = service.hash_password("Short1A");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_no_uppercase() {
        let service = PasswordService::new();
        let result = service.hash_password("lowercase123");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_no_lowercase() {
        let service = PasswordService::new();
        let result = service.hash_password("UPPERCASE123");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_no_digit() {
        let service = PasswordService::new();
        let result = service.hash_password("NoDigitsHere");
        assert!(result.is_err());
    }
}
