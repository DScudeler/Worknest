//! Password hashing and verification

use bcrypt::{hash, verify};

use crate::{AuthError, Result};

/// Default bcrypt cost factor (12)
/// Higher = more secure but slower
const BCRYPT_COST: u32 = 12;

/// Hash a password using bcrypt
///
/// # Arguments
/// * `password` - The plain text password to hash
///
/// # Returns
/// The hashed password string
pub fn hash_password(password: &str) -> Result<String> {
    validate_password(password)?;

    hash(password, BCRYPT_COST).map_err(|e| AuthError::Internal(e.to_string()))
}

/// Verify a password against a hash
///
/// # Arguments
/// * `password` - The plain text password to verify
/// * `hash` - The hashed password to verify against
///
/// # Returns
/// True if the password matches the hash, false otherwise
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    verify(password, hash).map_err(|e| AuthError::Internal(e.to_string()))
}

/// Validate password strength
///
/// Requirements:
/// - At least 8 characters
/// - At most 128 characters (bcrypt limit)
///
/// # Arguments
/// * `password` - The password to validate
fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(AuthError::PasswordValidation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    if password.len() > 72 {
        return Err(AuthError::PasswordValidation(
            "Password must be at most 72 characters (bcrypt limit)".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        // Hash should be different from password
        assert_ne!(password, hash);

        // Hash should start with bcrypt prefix
        assert!(hash.starts_with("$2"));

        // Hash should be consistent length
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword456!";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_password_too_short() {
        let password = "short";
        let result = hash_password(password);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_too_long() {
        let password = "a".repeat(73);
        let result = hash_password(&password);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_edge_cases() {
        // Minimum length
        let min_password = "12345678";
        assert!(hash_password(min_password).is_ok());

        // Maximum length
        let max_password = "a".repeat(72);
        assert!(hash_password(&max_password).is_ok());
    }
}
