//! Authentication service

use std::sync::Arc;

use worknest_core::models::{User, UserId};
use worknest_db::{Repository, UserRepository};

use crate::{
    password,
    token::{AuthToken, Claims, TokenManager},
    AuthError, Result,
};

/// Authentication service
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    token_manager: TokenManager,
}

impl AuthService {
    /// Create a new authentication service
    ///
    /// # Arguments
    /// * `user_repo` - User repository for database access
    /// * `secret_key` - Secret key for JWT signing
    /// * `token_expires_hours` - Token expiration time in hours (default: 24)
    pub fn new(
        user_repo: Arc<UserRepository>,
        secret_key: String,
        token_expires_hours: Option<i64>,
    ) -> Self {
        Self {
            user_repo,
            token_manager: TokenManager::new(secret_key, token_expires_hours),
        }
    }

    /// Register a new user
    ///
    /// # Arguments
    /// * `username` - Unique username
    /// * `email` - Unique email address
    /// * `password` - Plain text password (will be hashed)
    ///
    /// # Returns
    /// The created user (without password hash)
    pub fn register(&self, username: &str, email: &str, password: &str) -> Result<User> {
        // Validate inputs
        if username.is_empty() {
            return Err(AuthError::PasswordValidation(
                "Username cannot be empty".to_string(),
            ));
        }

        if email.is_empty() {
            return Err(AuthError::PasswordValidation(
                "Email cannot be empty".to_string(),
            ));
        }

        // Check if user already exists
        if self
            .user_repo
            .find_by_username(username)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(AuthError::UserExists);
        }

        if self
            .user_repo
            .find_by_email(email)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(AuthError::UserExists);
        }

        // Create user
        let user = User::new(username.to_string(), email.to_string());

        // Validate user
        user.validate()
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Hash password
        let password_hash = password::hash_password(password)?;

        // Save user
        self.user_repo
            .create_with_password(&user, &password_hash)
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Login a user
    ///
    /// # Arguments
    /// * `username` - Username or email
    /// * `password` - Plain text password
    ///
    /// # Returns
    /// An authentication token for the user
    pub fn login(&self, username: &str, password: &str) -> Result<AuthToken> {
        // Find user by username or email
        let user = self
            .user_repo
            .find_by_username(username)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .or_else(|| self.user_repo.find_by_email(username).ok().flatten())
            .ok_or(AuthError::InvalidCredentials)?;

        // Get password hash
        let password_hash = self
            .user_repo
            .get_password_hash(user.id)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let valid = password::verify_password(password, &password_hash)?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate token
        let token = self.token_manager.generate_token(user.id, user.username)?;

        Ok(token)
    }

    /// Verify a JWT token
    ///
    /// # Arguments
    /// * `token` - The JWT token string
    ///
    /// # Returns
    /// The decoded claims if valid
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        self.token_manager.verify_token(token)
    }

    /// Get user from token
    ///
    /// # Arguments
    /// * `token` - The JWT token string
    ///
    /// # Returns
    /// The user if token is valid
    pub fn get_user_from_token(&self, token: &str) -> Result<User> {
        let claims = self.verify_token(token)?;
        let user_id = claims.user_id()?;

        self.user_repo
            .find_by_id(user_id)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::UserNotFound)
    }

    /// Refresh a token
    ///
    /// # Arguments
    /// * `token` - The current token
    ///
    /// # Returns
    /// A new token with extended expiration
    pub fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        self.token_manager.refresh_token(token)
    }

    /// Change user password
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `old_password` - Current password
    /// * `new_password` - New password
    pub fn change_password(
        &self,
        user_id: UserId,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Get current password hash
        let current_hash = self
            .user_repo
            .get_password_hash(user_id)
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::UserNotFound)?;

        // Verify old password
        let valid = password::verify_password(old_password, &current_hash)?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Hash new password
        let new_hash = password::hash_password(new_password)?;

        // Update password
        self.user_repo
            .update_password(user_id, &new_hash)
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worknest_db::{init_memory_pool, run_migrations};

    fn setup_auth_service() -> AuthService {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);

        let user_repo = Arc::new(UserRepository::new(pool));
        AuthService::new(user_repo, "test_secret_key".to_string(), Some(24))
    }

    #[test]
    fn test_register_user() {
        let service = setup_auth_service();

        let user = service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_register_duplicate_username() {
        let service = setup_auth_service();

        service
            .register("testuser", "test1@example.com", "password123")
            .unwrap();

        let result = service.register("testuser", "test2@example.com", "password123");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::UserExists));
    }

    #[test]
    fn test_register_duplicate_email() {
        let service = setup_auth_service();

        service
            .register("user1", "test@example.com", "password123")
            .unwrap();

        let result = service.register("user2", "test@example.com", "password123");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::UserExists));
    }

    #[test]
    fn test_login_success() {
        let service = setup_auth_service();

        service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let token = service.login("testuser", "password123").unwrap();

        assert!(!token.token.is_empty());
    }

    #[test]
    fn test_login_with_email() {
        let service = setup_auth_service();

        service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let token = service.login("test@example.com", "password123").unwrap();

        assert!(!token.token.is_empty());
    }

    #[test]
    fn test_login_wrong_password() {
        let service = setup_auth_service();

        service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let result = service.login("testuser", "wrongpassword");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_login_nonexistent_user() {
        let service = setup_auth_service();

        let result = service.login("nonexistent", "password123");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_verify_token() {
        let service = setup_auth_service();

        service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let token = service.login("testuser", "password123").unwrap();
        let claims = service.verify_token(&token.token).unwrap();

        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_get_user_from_token() {
        let service = setup_auth_service();

        let registered_user = service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let token = service.login("testuser", "password123").unwrap();
        let user = service.get_user_from_token(&token.token).unwrap();

        assert_eq!(user.id, registered_user.id);
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_refresh_token() {
        let service = setup_auth_service();

        service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let token = service.login("testuser", "password123").unwrap();
        let refreshed = service.refresh_token(&token.token).unwrap();

        assert_ne!(token.token, refreshed.token);

        // Verify new token works
        let claims = service.verify_token(&refreshed.token).unwrap();
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_change_password() {
        let service = setup_auth_service();

        let user = service
            .register("testuser", "test@example.com", "oldpassword")
            .unwrap();

        // Change password
        service
            .change_password(user.id, "oldpassword", "newpassword")
            .unwrap();

        // Old password should not work
        let result = service.login("testuser", "oldpassword");
        assert!(result.is_err());

        // New password should work
        let token = service.login("testuser", "newpassword").unwrap();
        assert!(!token.token.is_empty());
    }

    #[test]
    fn test_change_password_wrong_old_password() {
        let service = setup_auth_service();

        let user = service
            .register("testuser", "test@example.com", "password123")
            .unwrap();

        let result = service.change_password(user.id, "wrongpassword", "newpassword");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_register_weak_password() {
        let service = setup_auth_service();

        let result = service.register("testuser", "test@example.com", "weak");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::PasswordValidation(_)
        ));
    }
}
