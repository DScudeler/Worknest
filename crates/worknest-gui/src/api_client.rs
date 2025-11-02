//! API client for communicating with the Worknest backend server

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worknest_core::models::{Comment, Project, Ticket, User};

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Create API client with default localhost URL for development
    pub fn new_default() -> Self {
        Self::new("http://localhost:3000".to_string())
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    // Auth endpoints
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        let response = self
            .client
            .post(self.api_url("/auth/register"))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Registration failed: {}", response.status()))
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        let response = self
            .client
            .post(self.api_url("/auth/login"))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Login failed: {}", response.status()))
        }
    }

    // User endpoints
    pub async fn get_current_user(&self, token: &str) -> Result<User> {
        let response = self
            .client
            .get(self.api_url("/users/me"))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get user: {}", response.status()))
        }
    }

    pub async fn get_users(&self, token: &str) -> Result<Vec<User>> {
        let response = self
            .client
            .get(self.api_url("/users"))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get users: {}", response.status()))
        }
    }

    // Project endpoints
    pub async fn get_projects(&self, token: &str) -> Result<Vec<Project>> {
        let response = self
            .client
            .get(self.api_url("/projects"))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get projects: {}", response.status()))
        }
    }

    pub async fn get_project(&self, token: &str, id: Uuid) -> Result<Project> {
        let response = self
            .client
            .get(self.api_url(&format!("/projects/{}", id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get project: {}", response.status()))
        }
    }

    pub async fn create_project(
        &self,
        token: &str,
        request: CreateProjectRequest,
    ) -> Result<Project> {
        let response = self
            .client
            .post(self.api_url("/projects"))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to create project: {}", response.status()))
        }
    }

    pub async fn update_project(
        &self,
        token: &str,
        id: Uuid,
        request: UpdateProjectRequest,
    ) -> Result<Project> {
        let response = self
            .client
            .put(self.api_url(&format!("/projects/{}", id)))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to update project: {}", response.status()))
        }
    }

    pub async fn delete_project(&self, token: &str, id: Uuid) -> Result<()> {
        let response = self
            .client
            .delete(self.api_url(&format!("/projects/{}", id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete project: {}", response.status()))
        }
    }

    pub async fn archive_project(&self, token: &str, id: Uuid) -> Result<Project> {
        let response = self
            .client
            .put(self.api_url(&format!("/projects/{}", id)))
            .bearer_auth(token)
            .json(&UpdateProjectRequest {
                name: None,
                description: None,
                is_archived: Some(true),
            })
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to archive project: {}", response.status()))
        }
    }

    pub async fn unarchive_project(&self, token: &str, id: Uuid) -> Result<Project> {
        let response = self
            .client
            .put(self.api_url(&format!("/projects/{}", id)))
            .bearer_auth(token)
            .json(&UpdateProjectRequest {
                name: None,
                description: None,
                is_archived: Some(false),
            })
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!(
                "Failed to unarchive project: {}",
                response.status()
            ))
        }
    }

    // Ticket endpoints
    pub async fn get_tickets(&self, token: &str, project_id: Option<Uuid>) -> Result<Vec<Ticket>> {
        let url = self.api_url("/tickets");

        let mut request = self.client.get(&url).bearer_auth(token);

        // Add project_id as query parameter if provided
        if let Some(pid) = project_id {
            request = request.query(&[("project_id", pid.to_string())]);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get tickets: {}", response.status()))
        }
    }

    pub async fn get_ticket(&self, token: &str, id: Uuid) -> Result<Ticket> {
        let response = self
            .client
            .get(self.api_url(&format!("/tickets/{}", id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get ticket: {}", response.status()))
        }
    }

    pub async fn create_ticket(&self, token: &str, request: CreateTicketRequest) -> Result<Ticket> {
        let response = self
            .client
            .post(self.api_url("/tickets"))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to create ticket: {}", response.status()))
        }
    }

    pub async fn update_ticket(
        &self,
        token: &str,
        id: Uuid,
        request: UpdateTicketRequest,
    ) -> Result<Ticket> {
        let response = self
            .client
            .put(self.api_url(&format!("/tickets/{}", id)))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to update ticket: {}", response.status()))
        }
    }

    pub async fn delete_ticket(&self, token: &str, id: Uuid) -> Result<()> {
        let response = self
            .client
            .delete(self.api_url(&format!("/tickets/{}", id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete ticket: {}", response.status()))
        }
    }

    // Comment endpoints
    pub async fn get_ticket_comments(&self, token: &str, ticket_id: Uuid) -> Result<Vec<Comment>> {
        let response = self
            .client
            .get(self.api_url(&format!("/tickets/{}/comments", ticket_id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to get comments: {}", response.status()))
        }
    }

    pub async fn create_comment(
        &self,
        token: &str,
        ticket_id: Uuid,
        request: CreateCommentRequest,
    ) -> Result<Comment> {
        let response = self
            .client
            .post(self.api_url(&format!("/tickets/{}/comments", ticket_id)))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to create comment: {}", response.status()))
        }
    }

    pub async fn update_comment(
        &self,
        token: &str,
        comment_id: Uuid,
        request: UpdateCommentRequest,
    ) -> Result<Comment> {
        let response = self
            .client
            .put(self.api_url(&format!("/comments/{}", comment_id)))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Failed to update comment: {}", response.status()))
        }
    }

    pub async fn delete_comment(&self, token: &str, comment_id: Uuid) -> Result<()> {
        let response = self
            .client
            .delete(self.api_url(&format!("/comments/{}", comment_id)))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete comment: {}", response.status()))
        }
    }
}

// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTicketRequest {
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub ticket_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTicketRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub ticket_type: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}
