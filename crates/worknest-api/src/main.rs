//! Worknest REST API Server
//!
//! Online-first API server for web and optionally desktop clients.

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Multipart, Path, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use worknest_auth::AuthService;
use worknest_core::models::{
    Attachment, AttachmentId, Comment, CommentId, Priority, Project, ProjectId, Ticket, TicketId,
    TicketStatus, TicketType, User,
};
use worknest_db::{
    init_pool, run_migrations, AttachmentRepository, CommentRepository, DbError, DbPool,
    ProjectRepository, Repository, TicketRepository, UserRepository,
};

/// Shared application state
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    pool: Arc<DbPool>,
    auth_service: Arc<AuthService>,
    #[allow(dead_code)]
    user_repo: Arc<UserRepository>,
    project_repo: Arc<ProjectRepository>,
    ticket_repo: Arc<TicketRepository>,
    comment_repo: Arc<CommentRepository>,
    attachment_repo: Arc<AttachmentRepository>,
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

    // Initialize services
    let secret_key = std::env::var("WORKNEST_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("Using default secret key - set WORKNEST_SECRET_KEY in production!");
        "dev-secret-key-change-in-production".to_string()
    });

    let user_repo = Arc::new(UserRepository::new(Arc::clone(&pool)));
    let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&pool)));
    let ticket_repo = Arc::new(TicketRepository::new(Arc::clone(&pool)));
    let comment_repo = Arc::new(CommentRepository::new(Arc::clone(&pool)));
    let attachment_repo = Arc::new(AttachmentRepository::new(Arc::clone(&pool)));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&user_repo),
        secret_key,
        Some(24), // 24 hour token expiration
    ));

    let state = AppState {
        pool,
        auth_service,
        user_repo,
        project_repo,
        ticket_repo,
        comment_repo,
        attachment_repo,
    };

    // Build router
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Projects
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/{id}",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route("/api/projects/{id}/archive", post(archive_project))
        // Tickets
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route(
            "/api/tickets/{id}",
            get(get_ticket).put(update_ticket).delete(delete_ticket),
        )
        // Comments
        .route(
            "/api/tickets/{ticket_id}/comments",
            get(list_comments_for_ticket).post(create_comment),
        )
        .route(
            "/api/comments/{id}",
            put(update_comment).delete(delete_comment),
        )
        // Attachments
        .route(
            "/api/tickets/{ticket_id}/attachments",
            get(list_attachments_for_ticket).post(upload_attachment),
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
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
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
// Project Routes
// ============================================================================

#[derive(Debug, Serialize)]
struct ProjectDto {
    id: String,
    name: String,
    description: Option<String>,
    color: Option<String>,
    archived: bool,
    created_by: String,
    created_at: String,
    updated_at: String,
}

impl From<Project> for ProjectDto {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            description: project.description,
            color: project.color,
            archived: project.archived,
            created_by: project.created_by.to_string(),
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }
    }
}

async fn list_projects(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectDto>>, AppError> {
    let projects = state.project_repo.find_all().map_err(|e| {
        tracing::error!("Failed to list projects: {:?}", e);
        AppError::Internal("Failed to retrieve projects".to_string())
    })?;

    Ok(Json(projects.into_iter().map(ProjectDto::from).collect()))
}

async fn get_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectDto>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    let project = state
        .project_repo
        .find_by_id(project_id)
        .map_err(|e| {
            tracing::error!("Failed to get project: {:?}", e);
            AppError::Internal("Failed to retrieve project".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    Ok(Json(project.into()))
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
) -> Result<Json<ProjectDto>, AppError> {
    let mut project = Project::new(req.name, user.id);
    project.description = req.description;

    // Validate
    project.validate().map_err(|e| {
        tracing::error!("Project validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let created_project = state.project_repo.create(&project).map_err(|e| {
        tracing::error!("Failed to create project: {:?}", e);
        AppError::Internal("Failed to create project".to_string())
    })?;

    Ok(Json(created_project.into()))
}

#[derive(Debug, Deserialize)]
struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
}

async fn update_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectDto>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    let mut project = state
        .project_repo
        .find_by_id(project_id)
        .map_err(|e| {
            tracing::error!("Failed to get project: {:?}", e);
            AppError::Internal("Failed to retrieve project".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    // Update fields if provided
    if let Some(name) = req.name {
        project.name = name;
    }
    if let Some(description) = req.description {
        project.description = Some(description);
    }

    // Validate
    project.validate().map_err(|e| {
        tracing::error!("Project validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let updated_project = state.project_repo.update(&project).map_err(|e| {
        tracing::error!("Failed to update project: {:?}", e);
        AppError::Internal("Failed to update project".to_string())
    })?;

    Ok(Json(updated_project.into()))
}

async fn delete_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    state.project_repo.delete(project_id).map_err(|e| {
        tracing::error!("Failed to delete project: {:?}", e);
        match e {
            DbError::NotFound(_) => AppError::NotFound("Project not found".to_string()),
            _ => AppError::Internal("Failed to delete project".to_string()),
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn archive_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectDto>, AppError> {
    let project_id = ProjectId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

    let archived_project = state.project_repo.archive(project_id).map_err(|e| {
        tracing::error!("Failed to archive project: {:?}", e);
        match e {
            DbError::NotFound(_) => AppError::NotFound("Project not found".to_string()),
            _ => AppError::Internal("Failed to archive project".to_string()),
        }
    })?;

    Ok(Json(archived_project.into()))
}

// ============================================================================
// Ticket Routes
// ============================================================================

#[derive(Debug, Serialize)]
struct TicketDto {
    id: String,
    project_id: String,
    title: String,
    description: Option<String>,
    ticket_type: String,
    status: String,
    priority: String,
    assignee_id: Option<String>,
    created_by: String,
    created_at: String,
    updated_at: String,
}

impl From<Ticket> for TicketDto {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id.to_string(),
            project_id: ticket.project_id.to_string(),
            title: ticket.title,
            description: ticket.description,
            ticket_type: format!("{:?}", ticket.ticket_type),
            status: format!("{:?}", ticket.status),
            priority: format!("{:?}", ticket.priority),
            assignee_id: ticket.assignee_id.map(|id| id.to_string()),
            created_by: ticket.created_by.to_string(),
            created_at: ticket.created_at.to_rfc3339(),
            updated_at: ticket.updated_at.to_rfc3339(),
        }
    }
}

async fn list_tickets(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TicketDto>>, AppError> {
    let tickets = state.ticket_repo.find_all().map_err(|e| {
        tracing::error!("Failed to list tickets: {:?}", e);
        AppError::Internal("Failed to retrieve tickets".to_string())
    })?;

    // Filter by project_id if provided
    let filtered_tickets = if let Some(project_id_str) = params.get("project_id") {
        let project_id = ProjectId::from_string(project_id_str)
            .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

        tickets.into_iter()
            .filter(|t| t.project_id == project_id)
            .collect()
    } else {
        tickets
    };

    Ok(Json(filtered_tickets.into_iter().map(TicketDto::from).collect()))
}

async fn get_ticket(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TicketDto>, AppError> {
    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let ticket = state
        .ticket_repo
        .find_by_id(ticket_id)
        .map_err(|e| {
            tracing::error!("Failed to get ticket: {:?}", e);
            AppError::Internal("Failed to retrieve ticket".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    Ok(Json(ticket.into()))
}

#[derive(Debug, Deserialize)]
struct CreateTicketRequest {
    project_id: String,
    title: String,
    description: Option<String>,
    ticket_type: String,
    priority: Option<String>,
}

async fn create_ticket(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<TicketDto>, AppError> {
    let project_id = ProjectId::from_string(&req.project_id)
        .map_err(|_| AppError::BadRequest("Invalid project ID".to_string()))?;

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
    ticket.validate().map_err(|e| {
        tracing::error!("Ticket validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let created_ticket = state.ticket_repo.create(&ticket).map_err(|e| {
        tracing::error!("Failed to create ticket: {:?}", e);
        AppError::Internal("Failed to create ticket".to_string())
    })?;

    Ok(Json(created_ticket.into()))
}

#[derive(Debug, Deserialize)]
struct UpdateTicketRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assignee_id: Option<String>,
}

async fn update_ticket(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTicketRequest>,
) -> Result<Json<TicketDto>, AppError> {
    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let mut ticket = state
        .ticket_repo
        .find_by_id(ticket_id)
        .map_err(|e| {
            tracing::error!("Failed to get ticket: {:?}", e);
            AppError::Internal("Failed to retrieve ticket".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

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
            "inprogress" => TicketStatus::InProgress,
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
    if let Some(assignee_id_str) = req.assignee_id {
        if assignee_id_str.is_empty() {
            ticket.assignee_id = None;
        } else {
            use worknest_core::models::UserId;
            ticket.assignee_id = Some(
                UserId::from_string(&assignee_id_str)
                    .map_err(|_| AppError::BadRequest("Invalid assignee ID".to_string()))?,
            );
        }
    }

    // Validate
    ticket.validate().map_err(|e| {
        tracing::error!("Ticket validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let updated_ticket = state.ticket_repo.update(&ticket).map_err(|e| {
        tracing::error!("Failed to update ticket: {:?}", e);
        AppError::Internal("Failed to update ticket".to_string())
    })?;

    Ok(Json(updated_ticket.into()))
}

async fn delete_ticket(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let ticket_id = TicketId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    state.ticket_repo.delete(ticket_id).map_err(|e| {
        tracing::error!("Failed to delete ticket: {:?}", e);
        match e {
            DbError::NotFound(_) => AppError::NotFound("Ticket not found".to_string()),
            _ => AppError::Internal("Failed to delete ticket".to_string()),
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Comment Routes
// ============================================================================

#[derive(Debug, Serialize)]
struct CommentDto {
    id: String,
    ticket_id: String,
    user_id: String,
    content: String,
    created_at: String,
    updated_at: String,
}

impl From<Comment> for CommentDto {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id.to_string(),
            ticket_id: comment.ticket_id.to_string(),
            user_id: comment.user_id.to_string(),
            content: comment.content,
            created_at: comment.created_at.to_rfc3339(),
            updated_at: comment.updated_at.to_rfc3339(),
        }
    }
}

async fn list_comments_for_ticket(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<CommentDto>>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let comments = state.comment_repo.find_by_ticket(ticket_id).map_err(|e| {
        tracing::error!("Failed to list comments: {:?}", e);
        AppError::Internal("Failed to retrieve comments".to_string())
    })?;

    Ok(Json(comments.into_iter().map(CommentDto::from).collect()))
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
) -> Result<Json<CommentDto>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let comment = Comment::new(ticket_id, user.id, req.content);

    // Validate
    comment.validate().map_err(|e| {
        tracing::error!("Comment validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let created_comment = state.comment_repo.create(&comment).map_err(|e| {
        tracing::error!("Failed to create comment: {:?}", e);
        AppError::Internal("Failed to create comment".to_string())
    })?;

    Ok(Json(created_comment.into()))
}

#[derive(Debug, Deserialize)]
struct UpdateCommentRequest {
    content: String,
}

async fn update_comment(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<CommentDto>, AppError> {
    let comment_id = CommentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid comment ID".to_string()))?;

    let mut comment = state
        .comment_repo
        .find_by_id(comment_id)
        .map_err(|e| {
            tracing::error!("Failed to get comment: {:?}", e);
            AppError::Internal("Failed to retrieve comment".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    comment.content = req.content;

    // Validate
    comment.validate().map_err(|e| {
        tracing::error!("Comment validation failed: {:?}", e);
        AppError::BadRequest(e.to_string())
    })?;

    let updated_comment = state.comment_repo.update(&comment).map_err(|e| {
        tracing::error!("Failed to update comment: {:?}", e);
        AppError::Internal("Failed to update comment".to_string())
    })?;

    Ok(Json(updated_comment.into()))
}

async fn delete_comment(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let comment_id = CommentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid comment ID".to_string()))?;

    state.comment_repo.delete(comment_id).map_err(|e| {
        tracing::error!("Failed to delete comment: {:?}", e);
        match e {
            DbError::NotFound(_) => AppError::NotFound("Comment not found".to_string()),
            _ => AppError::Internal("Failed to delete comment".to_string()),
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Attachment Routes
// ============================================================================

#[derive(Debug, Serialize)]
struct AttachmentDto {
    id: String,
    ticket_id: String,
    filename: String,
    file_size: i64,
    mime_type: String,
    uploaded_by: String,
    created_at: String,
}

impl From<Attachment> for AttachmentDto {
    fn from(attachment: Attachment) -> Self {
        Self {
            id: attachment.id.to_string(),
            ticket_id: attachment.ticket_id.to_string(),
            filename: attachment.filename,
            file_size: attachment.file_size,
            mime_type: attachment.mime_type,
            uploaded_by: attachment.uploaded_by.to_string(),
            created_at: attachment.created_at.to_rfc3339(),
        }
    }
}

async fn list_attachments_for_ticket(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<AttachmentDto>>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    let attachments = state
        .attachment_repo
        .find_by_ticket(ticket_id)
        .map_err(|e| {
            tracing::error!("Failed to list attachments: {:?}", e);
            AppError::Internal("Failed to retrieve attachments".to_string())
        })?;

    Ok(Json(
        attachments.into_iter().map(AttachmentDto::from).collect(),
    ))
}

async fn delete_attachment(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let attachment_id = AttachmentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid attachment ID".to_string()))?;

    // Get attachment to find file path
    let attachment = state
        .attachment_repo
        .find_by_id(attachment_id)
        .map_err(|e| {
            tracing::error!("Failed to get attachment: {:?}", e);
            AppError::Internal("Failed to retrieve attachment".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Delete from database
    state.attachment_repo.delete(attachment_id).map_err(|e| {
        tracing::error!("Failed to delete attachment: {:?}", e);
        match e {
            DbError::NotFound(_) => AppError::NotFound("Attachment not found".to_string()),
            _ => AppError::Internal("Failed to delete attachment".to_string()),
        }
    })?;

    // Delete file from disk (ignore errors if file doesn't exist)
    let _ = fs::remove_file(&attachment.file_path);

    Ok(StatusCode::NO_CONTENT)
}

async fn upload_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentDto>, AppError> {
    let ticket_id = TicketId::from_string(&ticket_id)
        .map_err(|_| AppError::BadRequest("Invalid ticket ID".to_string()))?;

    // Verify ticket exists
    state
        .ticket_repo
        .find_by_id(ticket_id)
        .map_err(|e| {
            tracing::error!("Failed to get ticket: {:?}", e);
            AppError::Internal("Failed to verify ticket".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    // Create uploads directory if it doesn't exist
    let upload_dir = PathBuf::from("./uploads");
    fs::create_dir_all(&upload_dir).map_err(|e| {
        tracing::error!("Failed to create uploads directory: {:?}", e);
        AppError::Internal("Failed to create uploads directory".to_string())
    })?;

    // Process multipart form
    let mut filename: Option<String> = None;
    let mut file_data: Option<Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            file_data =
                Some(field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read file data: {}", e))
                })?);
        }
    }

    let filename = filename.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;
    let file_data =
        file_data.ok_or_else(|| AppError::BadRequest("No file data provided".to_string()))?;

    // Generate unique filename
    let file_ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let unique_filename = format!(
        "{}_{}",
        uuid::Uuid::new_v4(),
        filename.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_")
    );

    let file_path = upload_dir.join(&unique_filename);

    // Write file to disk
    fs::write(&file_path, &file_data).map_err(|e| {
        tracing::error!("Failed to write file: {:?}", e);
        AppError::Internal("Failed to save file".to_string())
    })?;

    // Detect MIME type based on extension
    let mime_type = match file_ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
    .to_string();

    // Create attachment record
    let attachment = Attachment::new(
        ticket_id,
        filename,
        file_data.len() as i64,
        mime_type,
        file_path.to_string_lossy().to_string(),
        user.id,
    );

    // Validate
    attachment.validate().map_err(|e| {
        tracing::error!("Attachment validation failed: {:?}", e);
        // Clean up file on validation error
        let _ = fs::remove_file(&file_path);
        AppError::BadRequest(e.to_string())
    })?;

    let created_attachment = state.attachment_repo.create(&attachment).map_err(|e| {
        tracing::error!("Failed to create attachment: {:?}", e);
        // Clean up file on database error
        let _ = fs::remove_file(&file_path);
        AppError::Internal("Failed to create attachment".to_string())
    })?;

    Ok(Json(created_attachment.into()))
}

async fn download_attachment(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let attachment_id = AttachmentId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid attachment ID".to_string()))?;

    let attachment = state
        .attachment_repo
        .find_by_id(attachment_id)
        .map_err(|e| {
            tracing::error!("Failed to get attachment: {:?}", e);
            AppError::Internal("Failed to retrieve attachment".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Read file from disk
    let file_data = fs::read(&attachment.file_path).map_err(|e| {
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
                format!("attachment; filename=\"{}\"", attachment.filename),
            ),
            (header::CONTENT_LENGTH, file_data.len().to_string()),
        ],
        file_data,
    ))
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
