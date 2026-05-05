//! Worknest REST API Server
//!
//! Online-first API server for web and optionally desktop clients.

mod rate_limit;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Multipart, Path, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rate_limit::RateLimiter;
use worknest_auth::AuthService;
use worknest_core::models::{
    Attachment, AttachmentId, Comment, CommentId, Priority, Project, ProjectId, Tag, TagId, Ticket,
    TicketId, TicketStatus, TicketType, User, UserId,
};
use worknest_db::{
    init_pool, run_migrations, AttachmentRepository, CommentRepository, DbError, ProjectRepository,
    Repository, TagRepository, TicketFilters, TicketRepository, TicketSort, UserRepository,
};

/// Maximum file upload size (10 MB)
const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Shared application state
#[derive(Clone)]
struct AppState {
    auth_service: Arc<AuthService>,
    user_repo: Arc<UserRepository>,
    project_repo: Arc<ProjectRepository>,
    ticket_repo: Arc<TicketRepository>,
    comment_repo: Arc<CommentRepository>,
    attachment_repo: Arc<AttachmentRepository>,
    tag_repo: Arc<TagRepository>,
    rate_limiter: RateLimiter,
}

// ============================================================================
// Database I/O helper
// ============================================================================

/// Run a synchronous repository operation on the tokio blocking pool.
///
/// Repository methods use `rusqlite` which is fully synchronous; calling them
/// directly from an async handler would block a worker thread. This helper
/// wraps the closure in `spawn_blocking` and translates the resulting
/// `DbError` into an `AppError`. Handlers can still further refine the
/// returned error (e.g. wrap an `Option::None` into `NotFound`).
async fn db<F, T>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> std::result::Result<T, DbError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|je| {
            tracing::error!("DB blocking task panicked: {:?}", je);
            AppError::Internal("Database task failed".to_string())
        })?
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            match e {
                DbError::NotFound(msg) => AppError::NotFound(msg),
                DbError::ConstraintViolation(msg) => AppError::BadRequest(msg),
                _ => AppError::Internal("Database error".to_string()),
            }
        })
}

// ============================================================================
// Authorization helpers
// ============================================================================

/// Whether `user_id` may read a project, given an already-fetched
/// `is_member` flag. Owner always passes; members pass for read + comment +
/// claim flows. Project-mutation routes additionally require ownership and
/// must not rely on this helper alone.
fn project_visible_to(user_id: UserId, project: &Project, is_member: bool) -> bool {
    project.created_by == user_id || is_member
}

/// Whether `user_id` may read or interact with a ticket: ticket creator,
/// assignee, project owner, or any project member.
fn ticket_visible_to(
    user_id: UserId,
    ticket: &Ticket,
    project: Option<&Project>,
    is_member: bool,
) -> bool {
    ticket.created_by == user_id
        || ticket.assignee_id == Some(user_id)
        || project.is_some_and(|p| p.created_by == user_id)
        || is_member
}

/// Load a project and verify the caller may access it. Membership is
/// resolved against `project_members` so workers added to a project (by the
/// owner) can read it without owning it.
async fn load_project_for_access(
    state: &AppState,
    user_id: UserId,
    project_id: ProjectId,
) -> Result<Project, AppError> {
    let repo = state.project_repo.clone();
    let project = db(move || repo.find_by_id(project_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let mrepo = state.project_repo.clone();
    let is_member = db(move || mrepo.is_member(project_id, user_id)).await?;

    if !project_visible_to(user_id, &project, is_member) {
        return Err(AppError::Forbidden);
    }
    Ok(project)
}

/// Load a ticket plus its parent project, verifying access. Returns Forbidden
/// on access denial — but we deliberately use NotFound for missing rows so
/// callers can't enumerate IDs.
async fn load_ticket_for_access(
    state: &AppState,
    user_id: UserId,
    ticket_id: TicketId,
) -> Result<(Ticket, Option<Project>), AppError> {
    let trepo = state.ticket_repo.clone();
    let ticket = db(move || trepo.find_by_id(ticket_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let prepo = state.project_repo.clone();
    let pid = ticket.project_id;
    let project = db(move || prepo.find_by_id(pid)).await?;

    let mrepo = state.project_repo.clone();
    let is_member = db(move || mrepo.is_member(pid, user_id)).await?;

    if !ticket_visible_to(user_id, &ticket, project.as_ref(), is_member) {
        return Err(AppError::Forbidden);
    }
    Ok((ticket, project))
}

/// Stricter check for project-mutation routes (update, delete, archive, and
/// member management). Membership alone is not sufficient — caller must
/// own the project.
async fn load_project_for_owner(
    state: &AppState,
    user_id: UserId,
    project_id: ProjectId,
) -> Result<Project, AppError> {
    let repo = state.project_repo.clone();
    let project = db(move || repo.find_by_id(project_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.created_by != user_id {
        return Err(AppError::Forbidden);
    }
    Ok(project)
}

// ============================================================================
// Authentication Middleware & Extractor
// ============================================================================

/// Middleware to verify JWT token and attach authenticated user to request
async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".to_string()))?;

    // Verify token and get user
    let user = state.auth_service.get_user_from_token(token).map_err(|e| {
        tracing::warn!("Token verification failed: {:?}", e);
        AppError::Unauthorized("Invalid or expired token".to_string())
    })?;

    // Attach user to request extensions for handlers to use
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Middleware to add security headers to all responses
async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    // CSP scoped for the egui WASM frontend:
    //   - 'wasm-unsafe-eval' lets the WASM module load (eframe needs it); the
    //     deprecated 'unsafe-eval' is dropped.
    //   - 'unsafe-inline' on script-src is dropped — Trunk emits external JS.
    //   - style-src keeps 'unsafe-inline' because egui's bootstrap CSS is
    //     inline in index.html.
    //   - connect-src is restricted to 'self'; deployments fronting the API
    //     from a separate origin should override via reverse-proxy headers
    //     rather than widening this policy.
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'wasm-unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'",
        ),
    );
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

/// Extractor for authenticated user
struct AuthUser(User);

impl axum::extract::FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .map(AuthUser)
            .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
    }
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

    // Require WORKNEST_SECRET_KEY unconditionally; only allow a dev fallback when
    // WORKNEST_ENV is explicitly set to "development". Forgetting WORKNEST_ENV used
    // to silently fall back to a hardcoded constant — full auth bypass for anyone
    // who could read the source.
    let is_dev = std::env::var("WORKNEST_ENV")
        .map(|v| v.eq_ignore_ascii_case("development") || v.eq_ignore_ascii_case("dev"))
        .unwrap_or(false);
    let secret_key = match std::env::var("WORKNEST_SECRET_KEY") {
        Ok(key) => {
            if key.len() < 32 {
                panic!("WORKNEST_SECRET_KEY must be at least 32 bytes");
            }
            key
        },
        Err(_) if is_dev => {
            tracing::warn!(
                "WORKNEST_SECRET_KEY not set; using insecure development fallback. \
                 Set WORKNEST_SECRET_KEY (>=32 bytes) for any non-development use."
            );
            "dev-secret-key-change-in-production-do-not-use".to_string()
        },
        Err(_) => {
            panic!(
                "WORKNEST_SECRET_KEY must be set (>=32 bytes). \
                 To use a development fallback, set WORKNEST_ENV=development."
            );
        },
    };

    let user_repo = Arc::new(UserRepository::new(Arc::clone(&pool)));
    let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&pool)));
    let ticket_repo = Arc::new(TicketRepository::new(Arc::clone(&pool)));
    let comment_repo = Arc::new(CommentRepository::new(Arc::clone(&pool)));
    let attachment_repo = Arc::new(AttachmentRepository::new(Arc::clone(&pool)));
    let tag_repo = Arc::new(TagRepository::new(Arc::clone(&pool)));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&user_repo),
        secret_key,
        Some(24), // 24 hour token expiration
    ));

    // Rate limiter: 10 requests per minute per IP
    let rate_limiter = RateLimiter::new(10, 60);

    // Repos hold their own Arc<DbPool>; we don't need to retain another reference.
    drop(pool);

    let state = AppState {
        auth_service,
        user_repo,
        project_repo,
        ticket_repo,
        comment_repo,
        attachment_repo,
        tag_repo,
        rate_limiter,
    };

    // Configure CORS from environment
    let allowed_origins = std::env::var("WORKNEST_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let cors = {
        let mut cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

        for origin in allowed_origins.split(',') {
            if let Ok(origin) = origin.trim().parse::<HeaderValue>() {
                cors = cors.allow_origin(origin);
            }
        }

        cors
    };

    // Build router
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Users
        .route("/api/users", get(list_users))
        .route("/api/users/me", get(get_current_user).put(update_current_user))
        .route("/api/users/me/password", post(change_password))
        // Projects
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/{id}",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route("/api/projects/{id}/archive", post(archive_project))
        // Project membership: any member can list, only owner can add/remove.
        .route(
            "/api/projects/{id}/members",
            get(list_project_members).post(add_project_member),
        )
        .route(
            "/api/projects/{id}/members/{user_id}",
            axum::routing::delete(remove_project_member),
        )
        // Tickets
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/search", get(search_tickets))
        .route(
            "/api/tickets/{id}",
            get(get_ticket).put(update_ticket).delete(delete_ticket),
        )
        // Tags (categorical labels). Read-only for now; admins manage the
        // catalogue via DB seed (V5 migration). Phase 7+ may surface CRUD.
        .route("/api/tags", get(list_tags))
        // Dashboard stats: aggregate counts the home screen wants on first
        // paint (open tickets, assigned-to-me, due-this-week, active projects).
        .route("/api/stats", get(get_stats))
        // Comments
        .route(
            "/api/tickets/{ticket_id}/comments",
            get(list_comments_for_ticket).post(create_comment),
        )
        .route(
            "/api/comments/{id}",
            put(update_comment).delete(delete_comment),
        )
        // Attachments — explicit body limit so axum rejects oversized requests
        // before we read them. The handler also caps streamed bytes at
        // MAX_UPLOAD_SIZE; allow ~1 MB headroom for multipart framing.
        .route(
            "/api/tickets/{ticket_id}/attachments",
            get(list_attachments_for_ticket).post(upload_attachment).layer(
                DefaultBodyLimit::max(MAX_UPLOAD_SIZE + 1024 * 1024),
            ),
        )
        .route(
            "/api/attachments/{id}",
            get(download_attachment).delete(delete_attachment),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Combine routes and apply global middleware
    let app = public_routes
        .merge(protected_routes)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Background task: clean up rate limiter every 5 minutes
    let cleanup_limiter = state.rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            cleanup_limiter.cleanup().await;
        }
    });

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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server error");
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
    user: User,
    token: String,
}

async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Rate limit check
    if !state.rate_limiter.check(addr.ip()).await {
        return Err(AppError::TooManyRequests);
    }

    // Input length validation
    validate_username(&req.username)?;
    validate_email(&req.email)?;
    validate_password_length(&req.password)?;

    tracing::info!("Register request received");

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
        user,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Rate limit check
    if !state.rate_limiter.check(addr.ip()).await {
        return Err(AppError::TooManyRequests);
    }

    tracing::info!("Login request received");

    // Login
    let token = state
        .auth_service
        .login(&req.username, &req.password)
        .map_err(|e| {
            tracing::debug!("Login failed: {:?}", e);
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
        user,
        token: token.token,
    }))
}

async fn logout() -> StatusCode {
    // Client-side token invalidation - server acknowledges logout
    StatusCode::NO_CONTENT
}

// ============================================================================
// User Routes
// ============================================================================

/// Public projection of a user — no email or timestamps. Used for assignee
/// dropdowns and other listings where the full `User` would leak PII.
#[derive(Debug, Serialize)]
struct PublicUser {
    id: UserId,
    username: String,
}

async fn list_users(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicUser>>, AppError> {
    let repo = state.user_repo.clone();
    let users = db(move || repo.find_all()).await?;

    let public: Vec<PublicUser> = users
        .into_iter()
        .map(|u| PublicUser {
            id: u.id,
            username: u.username,
        })
        .collect();
    Ok(Json(public))
}

async fn get_current_user(AuthUser(user): AuthUser) -> Result<Json<User>, AppError> {
    Ok(Json(user))
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    username: Option<String>,
    email: Option<String>,
    /// Display name. Empty string clears the field; absent leaves it.
    full_name: Option<String>,
    /// Avatar URL. Empty string clears the field; absent leaves it.
    avatar_url: Option<String>,
}

async fn update_current_user(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    let mut updated_user = user;

    if let Some(username) = req.username {
        validate_username(&username)?;
        updated_user.username = username;
    }
    if let Some(email) = req.email {
        validate_email(&email)?;
        updated_user.email = email;
    }
    if let Some(full_name) = req.full_name {
        let trimmed = full_name.trim();
        if trimmed.len() > 100 {
            return Err(AppError::BadRequest(
                "Full name must be at most 100 characters".to_string(),
            ));
        }
        updated_user.full_name = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Some(avatar_url) = req.avatar_url {
        let trimmed = avatar_url.trim();
        if trimmed.len() > 2048 {
            return Err(AppError::BadRequest(
                "Avatar URL must be at most 2048 characters".to_string(),
            ));
        }
        updated_user.avatar_url = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }

    updated_user
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = state.user_repo.clone();
    let saved = db(move || repo.update(&updated_user)).await?;
    Ok(Json(saved))
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

async fn change_password(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    validate_password_length(&req.new_password)?;

    state
        .auth_service
        .change_password(user.id, &req.old_password, &req.new_password)
        .map_err(|e| {
            tracing::debug!("Password change failed: {:?}", e);
            match e {
                worknest_auth::AuthError::InvalidCredentials => {
                    AppError::Unauthorized("Current password is incorrect".to_string())
                },
                worknest_auth::AuthError::PasswordValidation(msg) => AppError::BadRequest(msg),
                _ => AppError::Internal("Failed to change password".to_string()),
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Project Routes
// ============================================================================

async fn list_projects(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, AppError> {
    let repo = state.project_repo.clone();
    let uid = user.id;
    let visible = db(move || repo.find_visible_to(uid)).await?;
    Ok(Json(visible))
}

async fn get_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    let project = load_project_for_access(&state, user.id, project_id).await?;
    Ok(Json(project))
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
}

async fn create_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, AppError> {
    // Input validation
    if req.name.len() > 500 {
        return Err(AppError::BadRequest(
            "Project name must be at most 500 characters".to_string(),
        ));
    }
    if let Some(ref desc) = req.description {
        if desc.len() > 10000 {
            return Err(AppError::BadRequest(
                "Description must be at most 10000 characters".to_string(),
            ));
        }
    }

    let mut project = Project::new(req.name, user.id);
    project.description = req.description;

    // Validate
    project
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = state.project_repo.clone();
    let creator = user.id;
    let project_id = project.id;
    let created_project = db(move || {
        let p = repo.create(&project)?;
        // Seed owner-as-member so the visibility "owner OR member" union has
        // a uniform shape; matches the V4 backfill for existing projects.
        repo.add_member(project_id, creator, "owner")?;
        Ok::<_, DbError>(p)
    })
    .await?;
    Ok(Json(created_project))
}

#[derive(Debug, Deserialize)]
struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
    /// Field name matches the model. Accept the legacy `is_archived` spelling
    /// from older clients via serde alias.
    #[serde(alias = "is_archived")]
    archived: Option<bool>,
}

async fn update_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    let mut project = load_project_for_owner(&state, user.id, project_id).await?;

    // Update fields if provided
    if let Some(name) = req.name {
        if name.len() > 500 {
            return Err(AppError::BadRequest(
                "Project name must be at most 500 characters".to_string(),
            ));
        }
        project.name = name;
    }
    if let Some(description) = req.description {
        if description.len() > 10000 {
            return Err(AppError::BadRequest(
                "Description must be at most 10000 characters".to_string(),
            ));
        }
        project.description = Some(description);
    }
    if let Some(archived) = req.archived {
        project.archived = archived;
    }

    project
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = state.project_repo.clone();
    let updated_project = db(move || repo.update(&project)).await?;
    Ok(Json(updated_project))
}

async fn delete_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    load_project_for_owner(&state, user.id, project_id).await?;

    let repo = state.project_repo.clone();
    db(move || repo.delete(project_id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn archive_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    load_project_for_owner(&state, user.id, project_id).await?;

    let repo = state.project_repo.clone();
    let archived_project = db(move || repo.archive(project_id)).await?;

    Ok(Json(archived_project))
}

// ============================================================================
// Project Member Routes
// ============================================================================

#[derive(Debug, Deserialize)]
struct AddMemberRequest {
    user_id: String,
    /// Optional role label; defaults to "member". The owner is auto-recorded
    /// as role="owner" by `create_project`. The role string is informational
    /// today — visibility checks treat any membership row as read-access.
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProjectMember {
    user_id: String,
    role: String,
}

async fn list_project_members(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ProjectMember>>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    // Any member (including owner) can read the membership roster.
    load_project_for_access(&state, user.id, project_id).await?;

    let repo = state.project_repo.clone();
    let members = db(move || repo.list_members(project_id)).await?;

    Ok(Json(
        members
            .into_iter()
            .map(|(uid, role)| ProjectMember {
                user_id: uid.0.to_string(),
                role,
            })
            .collect(),
    ))
}

async fn add_project_member(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<ProjectMember>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;
    let target_uid = UserId::from_string(&req.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Only the project owner may add members.
    load_project_for_owner(&state, user.id, project_id).await?;

    // Verify the user exists so we don't silently create dangling rows.
    let urepo = state.user_repo.clone();
    let exists = db(move || urepo.find_by_id(target_uid)).await?;
    if exists.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let role = req.role.unwrap_or_else(|| "member".to_string());
    let role_for_db = role.clone();
    let prepo = state.project_repo.clone();
    db(move || prepo.add_member(project_id, target_uid, &role_for_db)).await?;

    Ok(Json(ProjectMember {
        user_id: target_uid.0.to_string(),
        role,
    }))
}

async fn remove_project_member(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;
    let target_uid = UserId::from_string(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Only the project owner may remove members. Removing the owner is a
    // no-op for visibility (owner remains visible via `created_by`) but the
    // row is deleted as requested.
    load_project_for_owner(&state, user.id, project_id).await?;

    let repo = state.project_repo.clone();
    let removed = db(move || repo.remove_member(project_id, target_uid)).await?;
    if !removed {
        return Err(AppError::NotFound("Member not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Ticket Routes
// ============================================================================

/// Wire-format ticket: the inner `Ticket` plus its tag set. Older clients
/// that don't know about tags ignore the extra field (serde Deserialize on
/// `Ticket` is permissive by default), so this is additive.
#[derive(Debug, Serialize)]
struct TicketResponse {
    #[serde(flatten)]
    ticket: Ticket,
    tags: Vec<Tag>,
}

impl TicketResponse {
    fn new(ticket: Ticket, tags: Vec<Tag>) -> Self {
        Self { ticket, tags }
    }
}

/// Enrich a single ticket with its tags. One DB hop.
async fn enrich_ticket(state: &AppState, ticket: Ticket) -> Result<TicketResponse, AppError> {
    let trepo = state.tag_repo.clone();
    let id = ticket.id;
    let tags = db(move || trepo.list_for_ticket(id)).await?;
    Ok(TicketResponse::new(ticket, tags))
}

/// Enrich many tickets with their tags. Single batched DB hop.
async fn enrich_tickets(
    state: &AppState,
    tickets: Vec<Ticket>,
) -> Result<Vec<TicketResponse>, AppError> {
    if tickets.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<TicketId> = tickets.iter().map(|t| t.id).collect();
    let trepo = state.tag_repo.clone();
    let mut by_ticket = db(move || trepo.list_for_tickets(&ids)).await?;
    Ok(tickets
        .into_iter()
        .map(|t| {
            let tags = by_ticket.remove(&t.id).unwrap_or_default();
            TicketResponse::new(t, tags)
        })
        .collect())
}

/// Parse a list of TagId strings, surfacing the first invalid one as 400.
fn parse_tag_ids(raw: &[String]) -> Result<Vec<TagId>, AppError> {
    raw.iter()
        .map(|s| {
            TagId::from_string(s).map_err(|_| AppError::BadRequest(format!("Invalid tag id: {s}")))
        })
        .collect()
}

async fn list_tags(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Tag>>, AppError> {
    let repo = state.tag_repo.clone();
    let tags = db(move || repo.list_all()).await?;
    Ok(Json(tags))
}

#[derive(Debug, Serialize)]
struct DashboardStats {
    /// Tickets in any visible project still in `Open` status.
    open_tickets: usize,
    /// Tickets currently assigned to the caller (any status).
    assigned_to_me: usize,
    /// My tickets with `due_date` falling in the current ISO week.
    due_this_week: usize,
    /// Non-archived projects the caller can see.
    active_projects: usize,
}

async fn get_stats(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, AppError> {
    // Projects the caller owns or is a member of, non-archived only.
    let project_repo = state.project_repo.clone();
    let user_id = user.id;
    let visible_projects = db(move || project_repo.find_all())
        .await?
        .into_iter()
        .filter(|p| !p.archived)
        .collect::<Vec<_>>();

    // Re-check each project for membership; the helper short-circuits when
    // the caller owns it. We still need an explicit check for non-owner
    // members.
    let mrepo = state.project_repo.clone();
    let mut active_projects = 0usize;
    for p in &visible_projects {
        let pid = p.id;
        let owns = p.created_by == user_id;
        let is_member = if owns {
            true
        } else {
            let r = mrepo.clone();
            db(move || r.is_member(pid, user_id)).await?
        };
        if is_member {
            active_projects += 1;
        }
    }

    // All tickets the caller can see, with the existing visibility filter.
    let trepo = state.ticket_repo.clone();
    let filters = TicketFilters {
        caller_id: Some(user.id),
        ..TicketFilters::default()
    };
    let visible_tickets = db(move || trepo.find_with_filters(&filters)).await?;

    // ISO week window: Monday 00:00 UTC of this week to next Monday 00:00 UTC.
    let now = chrono::Utc::now();
    let weekday = now.date_naive().weekday().num_days_from_monday() as i64;
    let week_start =
        now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - chrono::Duration::days(weekday);
    let week_end = week_start + chrono::Duration::days(7);

    let mut open_tickets = 0usize;
    let mut assigned_to_me = 0usize;
    let mut due_this_week = 0usize;
    for t in &visible_tickets {
        if matches!(t.status, TicketStatus::Open) {
            open_tickets += 1;
        }
        if t.assignee_id == Some(user_id) {
            assigned_to_me += 1;
            if let Some(due) = t.due_date {
                if due >= week_start && due < week_end {
                    due_this_week += 1;
                }
            }
        }
    }

    Ok(Json(DashboardStats {
        open_tickets,
        assigned_to_me,
        due_this_week,
        active_projects,
    }))
}

async fn list_tickets(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    // Filters and authorization push into SQL via TicketFilters.
    let mut filters = TicketFilters {
        caller_id: Some(user.id),
        ..TicketFilters::default()
    };

    if let Some(s) = params.get("project_id") {
        filters.project_id = Some(
            ProjectId::from_string(s)
                .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?,
        );
    }
    if let Some(s) = params.get("status") {
        filters.status = Some(match s.to_lowercase().as_str() {
            "open" => TicketStatus::Open,
            "inprogress" => TicketStatus::InProgress,
            "review" => TicketStatus::Review,
            "done" => TicketStatus::Done,
            "closed" => TicketStatus::Closed,
            _ => return Err(AppError::BadRequest("Invalid status".to_string())),
        });
    }
    if let Some(s) = params.get("priority") {
        filters.priority = Some(match s.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(AppError::BadRequest("Invalid priority".to_string())),
        });
    }
    if let Some(s) = params.get("assignee_id") {
        filters.assignee_id = Some(if s == "me" {
            user.id
        } else {
            UserId::from_string(s)
                .map_err(|_| AppError::BadRequest("Invalid assignee ID".to_string()))?
        });
    }
    filters.sort = match params.get("sort").map(String::as_str) {
        Some("created_at") => TicketSort::CreatedAtDesc,
        Some("updated_at") => TicketSort::UpdatedAtDesc,
        Some("priority") => TicketSort::PriorityHighFirst,
        _ => TicketSort::None,
    };
    filters.limit = params.get("limit").and_then(|l| l.parse().ok());
    filters.offset = params.get("offset").and_then(|o| o.parse().ok());

    let repo = state.ticket_repo.clone();
    let tickets = db(move || repo.find_with_filters(&filters)).await?;
    let enriched = enrich_tickets(&state, tickets).await?;
    Ok(Json(enriched))
}

async fn get_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TicketResponse>, AppError> {
    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let (ticket, _project) = load_ticket_for_access(&state, user.id, ticket_id).await?;
    let response = enrich_ticket(&state, ticket).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct CreateTicketRequest {
    project_id: String,
    title: String,
    description: Option<String>,
    ticket_type: String,
    priority: Option<String>,
    /// Optional tag set. Sent as TagId strings; an unknown id rejects the
    /// whole create with 400 so the ticket and its tags stay consistent.
    #[serde(default)]
    tag_ids: Vec<String>,
}

async fn create_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<TicketResponse>, AppError> {
    // Input validation
    if req.title.len() > 500 {
        return Err(AppError::BadRequest(
            "Title must be at most 500 characters".to_string(),
        ));
    }
    if let Some(ref desc) = req.description {
        if desc.len() > 10000 {
            return Err(AppError::BadRequest(
                "Description must be at most 10000 characters".to_string(),
            ));
        }
    }

    let project_id = ProjectId::from_string(&req.project_id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    // Caller must own the project they're filing into.
    load_project_for_access(&state, user.id, project_id).await?;

    let ticket_type = match req.ticket_type.to_lowercase().as_str() {
        "task" => TicketType::Task,
        "bug" => TicketType::Bug,
        "feature" => TicketType::Feature,
        "epic" => TicketType::Epic,
        _ => return Err(AppError::BadRequest("Invalid ticket type".to_string())),
    };

    let mut ticket = Ticket::new(project_id, req.title, ticket_type, user.id);
    ticket.description = req.description;

    if let Some(priority_str) = req.priority {
        ticket.priority = match priority_str.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(AppError::BadRequest("Invalid priority".to_string())),
        };
    }

    // Validate
    ticket
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Validate tag ids before we touch tickets — surfaces 400 with a
    // useful message instead of orphaning a created ticket on a bad tag.
    let tag_ids = parse_tag_ids(&req.tag_ids)?;
    let trepo_check = state.tag_repo.clone();
    let tag_ids_check = tag_ids.clone();
    db(move || trepo_check.verify_exists(&tag_ids_check)).await?;

    let repo = state.ticket_repo.clone();
    let created_ticket = db(move || repo.create(&ticket)).await?;

    // Attach tags (best effort; if this fails we surface the error and the
    // ticket stays present without tags — caller can retry update).
    if !tag_ids.is_empty() {
        let trepo = state.tag_repo.clone();
        let tid = created_ticket.id;
        db(move || trepo.set_for_ticket(tid, &tag_ids)).await?;
    }

    let response = enrich_ticket(&state, created_ticket).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct UpdateTicketRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    ticket_type: Option<String>,
    assignee_id: Option<String>,
    /// If present, replaces the ticket's full tag set with this list.
    /// Absent means "leave tags untouched"; an empty array means "clear".
    tag_ids: Option<Vec<String>>,
}

async fn update_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateTicketRequest>,
) -> Result<Response, AppError> {
    // Input validation
    if let Some(ref title) = req.title {
        if title.len() > 500 {
            return Err(AppError::BadRequest(
                "Title must be at most 500 characters".to_string(),
            ));
        }
    }
    if let Some(ref desc) = req.description {
        if desc.len() > 10000 {
            return Err(AppError::BadRequest(
                "Description must be at most 10000 characters".to_string(),
            ));
        }
    }

    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let (mut ticket, _project) = load_ticket_for_access(&state, user.id, ticket_id).await?;

    // Optimistic concurrency: If-Match must match current updated_at.
    if let Some(if_match) = headers.get("if-match").and_then(|v| v.to_str().ok()) {
        let provided = chrono::DateTime::parse_from_rfc3339(if_match.trim().trim_matches('"'))
            .map_err(|_| {
                AppError::BadRequest("Invalid If-Match (expected RFC3339 timestamp)".to_string())
            })?;
        if provided.timestamp_millis() != ticket.updated_at.timestamp_millis() {
            return Err(AppError::PreconditionFailed(
                "Ticket modified since GET".to_string(),
            ));
        }
    }
    // Update fields if provided
    if let Some(title) = req.title {
        ticket.title = title;
    }
    if let Some(description) = req.description {
        ticket.description = Some(description);
    }
    if let Some(status_str) = req.status {
        ticket.status = match status_str.to_lowercase().as_str() {
            "open" => TicketStatus::Open,
            "inprogress" | "in progress" => TicketStatus::InProgress,
            "review" => TicketStatus::Review,
            "done" => TicketStatus::Done,
            "closed" => TicketStatus::Closed,
            _ => return Err(AppError::BadRequest("Invalid status".to_string())),
        };
    }
    if let Some(priority_str) = req.priority {
        ticket.priority = match priority_str.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => return Err(AppError::BadRequest("Invalid priority".to_string())),
        };
    }
    if let Some(ticket_type_str) = req.ticket_type {
        ticket.ticket_type = match ticket_type_str.to_lowercase().as_str() {
            "task" => TicketType::Task,
            "bug" => TicketType::Bug,
            "feature" => TicketType::Feature,
            "epic" => TicketType::Epic,
            _ => return Err(AppError::BadRequest("Invalid ticket type".to_string())),
        };
    }
    if let Some(assignee_id_str) = req.assignee_id {
        if assignee_id_str.is_empty() {
            ticket.assignee_id = None;
        } else {
            ticket.assignee_id = Some(
                UserId::from_string(&assignee_id_str)
                    .map_err(|_| AppError::BadRequest("Invalid assignee ID".to_string()))?,
            );
        }
    }

    // Validate
    ticket
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Tag set replacement (validated up front so we don't half-apply).
    let tag_ids_opt = if let Some(raw) = req.tag_ids.as_ref() {
        let parsed = parse_tag_ids(raw)?;
        let trepo_check = state.tag_repo.clone();
        let parsed_check = parsed.clone();
        db(move || trepo_check.verify_exists(&parsed_check)).await?;
        Some(parsed)
    } else {
        None
    };

    let repo = state.ticket_repo.clone();
    let updated_ticket = db(move || repo.update(&ticket)).await?;

    if let Some(tag_ids) = tag_ids_opt {
        let trepo = state.tag_repo.clone();
        let tid = updated_ticket.id;
        db(move || trepo.set_for_ticket(tid, &tag_ids)).await?;
    }

    // Return ETag with the new updated_at so clients can chain CAS updates.
    let etag = updated_ticket.updated_at.to_rfc3339();
    let mut headers_out = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&etag) {
        headers_out.insert("etag", v);
    }
    let response = enrich_ticket(&state, updated_ticket).await?;
    Ok((headers_out, Json(response)).into_response())
}

async fn delete_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    load_ticket_for_access(&state, user.id, ticket_id).await?;

    let repo = state.ticket_repo.clone();
    db(move || repo.delete(ticket_id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn search_tickets(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let query = params
        .get("q")
        .ok_or_else(|| AppError::BadRequest("Missing 'q' query parameter".to_string()))?;

    // Optional project_id filter; if specified, the caller must own that project.
    let project_id = match params.get("project_id") {
        Some(project_id_str) => {
            let pid = ProjectId::from_string(project_id_str)
                .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;
            load_project_for_access(&state, user.id, pid).await?;
            Some(pid)
        },
        None => None,
    };

    let trepo = state.ticket_repo.clone();
    let q = query.clone();
    let tickets = db(move || trepo.search(&q, project_id)).await?;

    // Strip results the caller can't see (search may cross projects).
    let prepo = state.project_repo.clone();
    let owned_project_ids: std::collections::HashSet<ProjectId> = db(move || prepo.find_all())
        .await?
        .into_iter()
        .filter(|p| p.created_by == user.id)
        .map(|p| p.id)
        .collect();
    let visible: Vec<Ticket> = tickets
        .into_iter()
        .filter(|t| {
            t.created_by == user.id
                || t.assignee_id == Some(user.id)
                || owned_project_ids.contains(&t.project_id)
        })
        .collect();

    let enriched = enrich_tickets(&state, visible).await?;
    Ok(Json(enriched))
}

// ============================================================================
// Comment Routes
// ============================================================================

async fn list_comments_for_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<Comment>>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    load_ticket_for_access(&state, user.id, ticket_id).await?;

    let repo = state.comment_repo.clone();
    let comments = db(move || repo.find_by_ticket(ticket_id)).await?;

    Ok(Json(comments))
}

#[derive(Debug, Deserialize)]
struct CreateCommentRequest {
    content: String,
}

async fn create_comment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, AppError> {
    // Input validation
    if req.content.len() > 10000 {
        return Err(AppError::BadRequest(
            "Comment content must be at most 10000 characters".to_string(),
        ));
    }

    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    // Caller must be able to see the ticket to comment on it.
    load_ticket_for_access(&state, user.id, ticket_id).await?;

    let comment = Comment::new(ticket_id, user.id, req.content);

    // Validate
    comment
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = state.comment_repo.clone();
    let created_comment = db(move || repo.create(&comment)).await?;
    Ok(Json(created_comment))
}

#[derive(Debug, Deserialize)]
struct UpdateCommentRequest {
    content: String,
}

async fn update_comment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<Comment>, AppError> {
    // Input validation
    if req.content.len() > 10000 {
        return Err(AppError::BadRequest(
            "Comment content must be at most 10000 characters".to_string(),
        ));
    }

    let comment_id = CommentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid comment ID".to_string()))?;

    let repo = state.comment_repo.clone();
    let mut comment = db(move || repo.find_by_id(comment_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    if user.id != comment.user_id {
        return Err(AppError::Forbidden);
    }

    comment.content = req.content;
    comment
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = state.comment_repo.clone();
    let updated_comment = db(move || repo.update(&comment)).await?;
    Ok(Json(updated_comment))
}

async fn delete_comment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let comment_id = CommentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid comment ID".to_string()))?;

    let repo = state.comment_repo.clone();
    let comment = db(move || repo.find_by_id(comment_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    if user.id != comment.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = state.comment_repo.clone();
    db(move || repo.delete(comment_id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Attachment Routes
// ============================================================================

async fn list_attachments_for_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<Attachment>>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    load_ticket_for_access(&state, user.id, ticket_id).await?;

    let repo = state.attachment_repo.clone();
    let attachments = db(move || repo.find_by_ticket(ticket_id)).await?;

    Ok(Json(attachments))
}

async fn delete_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let attachment_id = AttachmentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid attachment ID".to_string()))?;

    let repo = state.attachment_repo.clone();
    let attachment = db(move || repo.find_by_id(attachment_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Caller must be able to access the parent ticket — or be the uploader.
    if attachment.uploaded_by != user.id {
        load_ticket_for_access(&state, user.id, attachment.ticket_id).await?;
    }

    // Delete file first; if FS unlink fails we keep the row so cleanup can
    // retry. Inverse of the previous order, which left orphan files on disk.
    if let Err(e) = tokio::fs::remove_file(&attachment.file_path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::error!("Failed to remove attachment file: {:?}", e);
            return Err(AppError::Internal(
                "Failed to delete attachment file".to_string(),
            ));
        }
    }

    let repo = state.attachment_repo.clone();
    db(move || repo.delete(attachment_id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Strict MIME allowlist. Magic-byte match wins; otherwise the extension must
/// be one of the document/text types we accept. Anything else is rejected —
/// no `application/octet-stream` catch-all (the previous behavior let any
/// binary through).
fn validate_mime_type(data: &[u8], filename: &str) -> Result<String, AppError> {
    if data.len() >= 3 && data[..3] == [0xFF, 0xD8, 0xFF] {
        return Ok("image/jpeg".to_string());
    }
    if data.len() >= 4 && data[..4] == [0x89, 0x50, 0x4E, 0x47] {
        return Ok("image/png".to_string());
    }
    if data.len() >= 4 && data[..4] == [0x47, 0x49, 0x46, 0x38] {
        return Ok("image/gif".to_string());
    }
    if data.len() >= 4 && data[..4] == [0x25, 0x50, 0x44, 0x46] {
        return Ok("application/pdf".to_string());
    }

    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "doc" => Ok("application/msword".to_string()),
        "docx" => Ok(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ),
        "txt" => Ok("text/plain".to_string()),
        _ => Err(AppError::BadRequest(
            "Unsupported file type. Allowed: jpg, png, gif, pdf, doc, docx, txt".to_string(),
        )),
    }
}

/// Sanitize filename to prevent directory traversal
fn sanitize_filename(filename: &str) -> String {
    // Extract just the file name component, stripping any path
    let name = std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Replace any remaining problematic characters
    name.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_")
}

async fn upload_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Attachment>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    // Caller must be able to access the ticket they're attaching to.
    load_ticket_for_access(&state, user.id, ticket_id).await?;

    let upload_dir = PathBuf::from("./uploads");
    tokio::fs::create_dir_all(&upload_dir).await.map_err(|e| {
        tracing::error!("Failed to create uploads directory: {:?}", e);
        AppError::Internal("Failed to create uploads directory".to_string())
    })?;

    // Process multipart form, streaming chunks so we never buffer more than
    // MAX_UPLOAD_SIZE bytes regardless of declared Content-Length.
    let mut filename: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))?
    {
        if field.name().unwrap_or("") != "file" {
            continue;
        }
        filename = field.file_name().map(|s| s.to_string());

        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?
        {
            if buf.len() + chunk.len() > MAX_UPLOAD_SIZE {
                return Err(AppError::BadRequest(format!(
                    "File too large. Maximum size is {} MB",
                    MAX_UPLOAD_SIZE / (1024 * 1024)
                )));
            }
            buf.extend_from_slice(&chunk);
        }
        file_data = Some(buf);
    }

    let filename = filename.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;
    let file_data =
        file_data.ok_or_else(|| AppError::BadRequest("No file data provided".to_string()))?;

    // Strict MIME allowlist (no octet-stream fallback).
    let mime_type = validate_mime_type(&file_data, &filename)?;

    // sanitize_filename strips path separators and special chars, so the join
    // below cannot escape upload_dir. The previous canonicalize-based check
    // was a no-op for not-yet-written files and provided no real safety.
    let safe_filename = sanitize_filename(&filename);
    let unique_filename = format!("{}_{}", uuid::Uuid::new_v4(), safe_filename);
    let file_path = upload_dir.join(&unique_filename);

    tokio::fs::write(&file_path, &file_data)
        .await
        .map_err(|e| {
            tracing::error!("Failed to write file: {:?}", e);
            AppError::Internal("Failed to save file".to_string())
        })?;

    let attachment = Attachment::new(
        ticket_id,
        filename,
        file_data.len() as i64,
        mime_type,
        file_path.to_string_lossy().to_string(),
        user.id,
    );

    // Synchronous cleanup on the error path — single-file unlink is fast and
    // the previous tokio::spawn detached the cleanup, leaking on shutdown.
    if let Err(e) = attachment.validate() {
        let _ = std::fs::remove_file(&file_path);
        return Err(AppError::BadRequest(e.to_string()));
    }

    let repo = state.attachment_repo.clone();
    let attachment_for_db = attachment.clone();
    let created_attachment = match db(move || repo.create(&attachment_for_db)).await {
        Ok(a) => a,
        Err(e) => {
            let _ = std::fs::remove_file(&file_path);
            return Err(e);
        },
    };

    Ok(Json(created_attachment))
}

async fn download_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let attachment_id = AttachmentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid attachment ID".to_string()))?;

    let repo = state.attachment_repo.clone();
    let attachment = db(move || repo.find_by_id(attachment_id))
        .await?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Caller must be able to access the parent ticket — uploaders can always
    // re-download their own files even if their ticket access is later revoked.
    if attachment.uploaded_by != user.id {
        load_ticket_for_access(&state, user.id, attachment.ticket_id).await?;
    }

    // Read file from disk
    let file_data = tokio::fs::read(&attachment.file_path).await.map_err(|e| {
        tracing::error!("Failed to read file: {:?}", e);
        AppError::NotFound("File not found on disk".to_string())
    })?;

    // Return file with appropriate headers
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, attachment.mime_type.clone()),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}\"",
                    sanitize_filename(&attachment.filename)
                ),
            ),
            (header::CONTENT_LENGTH, file_data.len().to_string()),
        ],
        file_data,
    ))
}

// ============================================================================
// Input Validation Helpers
// ============================================================================

fn validate_username(username: &str) -> Result<(), AppError> {
    if username.len() > 50 {
        return Err(AppError::BadRequest(
            "Username must be at most 50 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if email.len() > 254 {
        return Err(AppError::BadRequest(
            "Email must be at most 254 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_password_length(password: &str) -> Result<(), AppError> {
    if password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be at most 72 characters".to_string(),
        ));
    }
    Ok(())
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden,
    PreconditionFailed(String),
    NotFound(String),
    TooManyRequests,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => {
                // Strip any SQL or path information from client-facing messages
                let sanitized = sanitize_error_message(&msg);
                (StatusCode::BAD_REQUEST, sanitized)
            },
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "You do not have permission to perform this action".to_string(),
            ),
            AppError::PreconditionFailed(msg) => (StatusCode::PRECONDITION_FAILED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests. Please try again later.".to_string(),
            ),
            AppError::Internal(msg) => {
                // Log the real error server-side, return generic message to client
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred".to_string(),
                )
            },
        };

        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

/// Sanitize error messages to prevent leaking internal details
fn sanitize_error_message(msg: &str) -> String {
    // If message contains SQL-related terms or file paths, return generic message
    let lower = msg.to_lowercase();
    if lower.contains("sqlite")
        || lower.contains("sql")
        || lower.contains("pragma")
        || lower.contains("table")
        || (msg.contains('/') && (msg.contains("src/") || msg.contains("crates/")))
    {
        return "Invalid request".to_string();
    }
    msg.to_string()
}
