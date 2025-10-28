# Worknest API Integration Guide

## Overview

This guide explains how to set up and use the Worknest backend API with the frontend application.

## Architecture

Worknest uses a **dual-mode architecture**:

- **Demo Mode** (`?mode=demo`): In-memory data, no backend required
- **Integrated Mode** (default): Connects to backend REST API

## Backend Server Setup

### 1. Environment Configuration

Create a `.env` file in the project root:

```bash
# Server Configuration
PORT=3000

# Database Configuration
WORKNEST_DB_PATH=./worknest.db

# JWT Secret Key (Change in production!)
WORKNEST_SECRET_KEY=dev-secret-key-please-change-in-production

# Logging Level
RUST_LOG=worknest_api=debug,tower_http=debug
```

### 2. Build and Run Backend

```bash
# Build the API server
cargo build --release -p worknest-api

# Run the server
./target/release/worknest-api
```

The server will start on `http://localhost:3000` and create a SQLite database at `./worknest.db`.

### 3. Verify Server is Running

```bash
# Check health endpoint
curl http://localhost:3000/health
# Should return: OK

# Test registration
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'
```

## Frontend Setup

### 1. Build Frontend

```bash
# Start trunk dev server with hot reload
trunk serve --release
```

Frontend will be available at `http://localhost:8080`

### 2. Usage Modes

**Demo Mode** (no backend required):
```
http://localhost:8080?mode=demo
```

**Integrated Mode** (requires backend running):
```
http://localhost:8080
http://localhost:8080?mode=integrated
```

## API Endpoints

### Authentication

#### Register
```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "string",
  "email": "string",
  "password": "string"
}

Response: {
  "user": {
    "id": "uuid",
    "username": "string",
    "email": "string"
  },
  "token": "jwt_token"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}

Response: {
  "user": { "id": "uuid", "username": "string", "email": "string" },
  "token": "jwt_token"
}
```

### Projects

All project endpoints require `Authorization: Bearer <token>` header.

#### List Projects
```http
GET /api/projects
Authorization: Bearer <token>

Response: [
  {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "archived": false,
    "created_by": "uuid",
    "created_at": "iso8601",
    "updated_at": "iso8601"
  }
]
```

#### Get Project
```http
GET /api/projects/{id}
Authorization: Bearer <token>
```

#### Create Project
```http
POST /api/projects
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "string",
  "description": "string"
}
```

#### Update Project
```http
PUT /api/projects/{id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "string",
  "description": "string"
}
```

#### Delete Project
```http
DELETE /api/projects/{id}
Authorization: Bearer <token>
```

#### Archive Project
```http
POST /api/projects/{id}/archive
Authorization: Bearer <token>
```

### Tickets

#### List All Tickets
```http
GET /api/tickets
Authorization: Bearer <token>
```

#### List Project Tickets
```http
GET /api/projects/{project_id}/tickets
Authorization: Bearer <token>
```

#### Get Ticket
```http
GET /api/tickets/{id}
Authorization: Bearer <token>
```

#### Create Ticket
```http
POST /api/tickets
Authorization: Bearer <token>
Content-Type: application/json

{
  "project_id": "uuid",
  "title": "string",
  "description": "string",
  "ticket_type": "task|bug|feature|epic",
  "priority": "low|medium|high|critical"
}
```

#### Update Ticket
```http
PUT /api/tickets/{id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "string",
  "description": "string",
  "status": "open|inprogress|review|done|closed",
  "priority": "low|medium|high|critical",
  "assignee_id": "uuid"
}
```

#### Delete Ticket
```http
DELETE /api/tickets/{id}
Authorization: Bearer <token>
```

## Token Persistence

The frontend automatically:
- Saves authentication token to `localStorage` on successful login
- Restores session from `localStorage` on app load
- Clears token on logout

## Current Implementation Status

### ✅ Completed
- Backend API server with all endpoints
- Database migrations and schema
- JWT authentication
- API client implementation
- Token persistence with localStorage
- Session auto-restore
- Demo mode for offline development

### 🚧 In Progress
- Async state management for API calls
- Complete frontend-backend integration
- Error handling and loading states

### 📋 Next Steps
1. Implement proper async state management pattern
2. Connect register screen to API
3. Connect project CRUD operations
4. Connect ticket CRUD operations
5. Add loading indicators and error messages
6. E2E integration testing

## Development Workflow

### Running Both Servers

**Terminal 1** - Backend API:
```bash
./target/release/worknest-api
```

**Terminal 2** - Frontend:
```bash
trunk serve --release
```

**Browser**:
```
http://localhost:8080  # Integrated mode (connects to backend)
http://localhost:8080?mode=demo  # Demo mode (no backend needed)
```

## Troubleshooting

### Backend won't start
- Check if port 3000 is already in use: `lsof -i :3000`
- Verify `.env` file exists with correct configuration
- Check database file permissions

### Frontend can't connect to backend
- Verify backend is running on port 3000
- Check browser console for CORS errors
- Ensure API client base URL is `http://localhost:3000`

### Token not persisting
- Check browser localStorage in DevTools
- Verify browser allows localStorage (not in private mode)
- Clear localStorage and try logging in again

## Security Notes

⚠️ **Development Configuration**:
- Default secret key is for development only
- Use a strong random key in production: `openssl rand -base64 32`
- Database is SQLite for development; consider PostgreSQL for production
- CORS is set to permissive mode; configure properly for production

## Architecture Decisions

### Why Two Modes?

1. **Demo Mode**: Allows frontend development without backend
2. **Integrated Mode**: Full production-ready with backend API

### Why Localhost:3000?

- Standard port for backend APIs
- Avoids conflicts with frontend dev server (8080)
- Easy to remember and document

### Why Token in localStorage?

- Simplest cross-tab persistence
- Survives page reloads
- Standard approach for web apps
- Note: Consider httpOnly cookies for production security

## Further Reading

- [Axum Documentation](https://docs.rs/axum)
- [egui Documentation](https://docs.rs/egui)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
