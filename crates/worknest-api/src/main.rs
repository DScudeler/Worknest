//! Worknest REST API Server
//!
//! Online-first API server for web and optionally desktop clients.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use worknest_auth::AuthService;
use worknest_core::models::User;
use worknest_db::{init_pool, run_migrations, DbPool, UserRepository};

/// Shared application state
#[derive(Clone)]
struct AppState {
    pool: Arc<DbPool>,
    auth_service: Arc<AuthService>,
    user_repo: Arc<UserRepository>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worknest_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let db_path =
        std::env::var("WORKNEST_DB_PATH").unwrap_or_else(|_| "./worknest-api.db".to_string());

    tracing::info!("Initializing database at: {}", db_path);
    let pool = Arc::new(init_pool(&db_path).expect("Failed to initialize database pool"));
    run_migrations(&mut pool.get().expect("Failed to get connection"))
        .expect("Failed to run migrations");

    // Initialize services
    let secret_key = std::env::var("WORKNEST_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("Using default secret key - set WORKNEST_SECRET_KEY in production!");
        "dev-secret-key-change-in-production".to_string()
    });

    let user_repo = Arc::new(UserRepository::new(Arc::clone(&pool)));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&user_repo),
        secret_key,
        Some(24), // 24 hour token expiration
    ));

    let state = AppState {
        pool,
        auth_service,
        user_repo,
    };

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))

        // Authentication
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))

        // Apply middleware
        .layer(CorsLayer::permissive()) // TODO: Configure CORS properly
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server error");
}

// ============================================================================
// Health Check
// ============================================================================

async fn health_check() -> &'static str {
    "OK"
}

// ============================================================================
// Authentication Routes
// ============================================================================

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    user: UserDto,
    token: String,
}

#[derive(Debug, Serialize)]
struct UserDto {
    id: String,
    username: String,
    email: String,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }
    }
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::info!("Register request for username: {}", req.username);

    // Register user
    let user = state
        .auth_service
        .register(&req.username, &req.email, &req.password)
        .map_err(|e| {
            tracing::error!("Registration failed: {:?}", e);
            AppError::BadRequest(format!("Registration failed: {}", e))
        })?;

    // Generate token
    let token = state
        .auth_service
        .login(&req.username, &req.password)
        .map_err(|e| {
            tracing::error!("Login after registration failed: {:?}", e);
            AppError::Internal("Failed to generate token".to_string())
        })?;

    Ok(Json(AuthResponse {
        user: user.into(),
        token: token.token,
    }))
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::info!("Login request for username: {}", req.username);

    // Login
    let token = state
        .auth_service
        .login(&req.username, &req.password)
        .map_err(|e| {
            tracing::error!("Login failed: {:?}", e);
            AppError::Unauthorized("Invalid credentials".to_string())
        })?;

    // Get user
    let user = state
        .auth_service
        .get_user_from_token(&token.token)
        .map_err(|e| {
            tracing::error!("Failed to get user from token: {:?}", e);
            AppError::Internal("Failed to retrieve user".to_string())
        })?;

    Ok(Json(AuthResponse {
        user: user.into(),
        token: token.token,
    }))
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
