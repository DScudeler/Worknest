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

/// Hash a password without running the strength validator. Reserved for
/// system-internal callers (e.g. the agents subsystem, which generates a
/// random unguessable password for autonomous-agent users that never log in
/// via `/api/auth/login`).
pub fn hash_password_unchecked(password: &str) -> Result<String> {
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
/// - At most 72 characters (bcrypt limit)
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
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

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AuthError::PasswordValidation(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(AuthError::PasswordValidation(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AuthError::PasswordValidation(
            "Password must contain at least one digit".to_string(),
        ));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AuthError::PasswordValidation(
            "Password must contain at least one special character".to_string(),
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
        let password = "Sh0r!";
        let result = hash_password(password);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_too_long() {
        // 73 chars with complexity requirements met
        let password = format!("Aa1!{}", "a".repeat(69));
        let result = hash_password(&password);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_missing_uppercase() {
        let result = hash_password("lowercase1!");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_missing_lowercase() {
        let result = hash_password("UPPERCASE1!");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_missing_digit() {
        let result = hash_password("NoDigits!here");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_missing_special() {
        let result = hash_password("NoSpecial1here");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }

    #[test]
    fn test_password_edge_cases() {
        // Minimum length with all requirements
        let min_password = "Aa1!abcd";
        assert!(hash_password(min_password).is_ok());

        // Maximum length with all requirements
        let max_password = format!("Aa1!{}", "b".repeat(68));
        assert!(hash_password(&max_password).is_ok());
    }
}
