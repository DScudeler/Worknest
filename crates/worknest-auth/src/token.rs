//! JWT token generation and validation

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use worknest_core::models::UserId;

use crate::{AuthError, Result};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Username
    pub username: String,
    /// Token expiration timestamp
    pub exp: i64,
    /// Token issued at timestamp
    pub iat: i64,
    /// Token ID (for revocation)
    pub jti: String,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: UserId, username: String, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expires_in_hours);

        Self {
            sub: user_id.0.to_string(),
            username,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Get user ID from claims
    pub fn user_id(&self) -> Result<UserId> {
        let uuid = Uuid::parse_str(&self.sub).map_err(|_| AuthError::TokenInvalid)?;
        Ok(UserId::from_uuid(uuid))
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Get expiration time
    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(Utc::now)
    }
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// JWT token string
    pub token: String,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    /// Token ID (for revocation)
    pub token_id: String,
}

impl AuthToken {
    /// Create new auth token from claims
    pub fn new(token: String, claims: &Claims) -> Self {
        Self {
            token,
            expires_at: claims.expires_at(),
            token_id: claims.jti.clone(),
        }
    }
}

/// Token manager for JWT operations
pub struct TokenManager {
    secret: String,
    expires_in_hours: i64,
}

impl TokenManager {
    /// Create a new token manager
    ///
    /// # Arguments
    /// * `secret` - Secret key for signing tokens
    /// * `expires_in_hours` - Token expiration time in hours (default: 24)
    pub fn new(secret: String, expires_in_hours: Option<i64>) -> Self {
        Self {
            secret,
            expires_in_hours: expires_in_hours.unwrap_or(24),
        }
    }

    /// Generate a JWT token for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `username` - Username
    ///
    /// # Returns
    /// An AuthToken containing the JWT string and expiration info
    pub fn generate_token(&self, user_id: UserId, username: String) -> Result<AuthToken> {
        let claims = Claims::new(user_id, username, self.expires_in_hours);

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(AuthToken::new(token, &claims))
    }

    /// Verify and decode a JWT token
    ///
    /// # Arguments
    /// * `token` - The JWT token string to verify
    ///
    /// # Returns
    /// The decoded claims if valid
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            if e.to_string().contains("ExpiredSignature") {
                AuthError::TokenExpired
            } else {
                AuthError::TokenInvalid
            }
        })?;

        Ok(token_data.claims)
    }

    /// Refresh a token (generate a new one with updated expiration)
    ///
    /// # Arguments
    /// * `token` - The current token to refresh
    ///
    /// # Returns
    /// A new AuthToken with extended expiration
    pub fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        let claims = self.verify_token(token)?;
        self.generate_token(claims.user_id()?, claims.username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> TokenManager {
        TokenManager::new("test_secret_key_123".to_string(), Some(24))
    }

    #[test]
    fn test_generate_token() {
        let manager = create_test_manager();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let auth_token = manager.generate_token(user_id, username.clone()).unwrap();

        assert!(!auth_token.token.is_empty());
        assert!(auth_token.token.split('.').count() == 3); // JWT has 3 parts
    }

    #[test]
    fn test_verify_token() {
        let manager = create_test_manager();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let auth_token = manager.generate_token(user_id, username.clone()).unwrap();
        let claims = manager.verify_token(&auth_token.token).unwrap();

        assert_eq!(claims.username, username);
        assert_eq!(claims.user_id().unwrap(), user_id);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_verify_invalid_token() {
        let manager = create_test_manager();
        let invalid_token = "invalid.token.here";

        let result = manager.verify_token(invalid_token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::TokenInvalid));
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let manager1 = TokenManager::new("secret1".to_string(), Some(24));
        let manager2 = TokenManager::new("secret2".to_string(), Some(24));

        let user_id = UserId::new();
        let auth_token = manager1
            .generate_token(user_id, "testuser".to_string())
            .unwrap();

        let result = manager2.verify_token(&auth_token.token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::TokenInvalid));
    }

    #[test]
    fn test_claims_user_id() {
        let user_id = UserId::new();
        let claims = Claims::new(user_id, "testuser".to_string(), 24);

        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_claims_expiration() {
        let user_id = UserId::new();
        let claims = Claims::new(user_id, "testuser".to_string(), 24);

        assert!(!claims.is_expired());
        assert!(claims.expires_at() > Utc::now());
    }

    #[test]
    fn test_refresh_token() {
        let manager = create_test_manager();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let auth_token = manager.generate_token(user_id, username.clone()).unwrap();
        let refreshed = manager.refresh_token(&auth_token.token).unwrap();

        // New token should be different
        assert_ne!(auth_token.token, refreshed.token);

        // But should have same user info
        let claims = manager.verify_token(&refreshed.token).unwrap();
        assert_eq!(claims.username, username);
        assert_eq!(claims.user_id().unwrap(), user_id);
    }
}
