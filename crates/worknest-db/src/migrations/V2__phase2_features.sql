-- Phase 2 features: Advanced ticket features, teams, and permissions

-- Ticket dependencies (blocked by / blocks relationships)
CREATE TABLE ticket_dependencies (
    id TEXT PRIMARY KEY NOT NULL,
    ticket_id TEXT NOT NULL,
    depends_on_ticket_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    UNIQUE(ticket_id, depends_on_ticket_id)
);

CREATE INDEX idx_ticket_deps_ticket_id ON ticket_dependencies(ticket_id);
CREATE INDEX idx_ticket_deps_depends_on ON ticket_dependencies(depends_on_ticket_id);

-- Attachments for tickets
CREATE TABLE attachments (
    id TEXT PRIMARY KEY NOT NULL,
    ticket_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    uploaded_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);

CREATE INDEX idx_attachments_ticket_id ON attachments(ticket_id);
CREATE INDEX idx_attachments_uploaded_by ON attachments(uploaded_by);

-- User roles
CREATE TABLE roles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_roles_name ON roles(name);

-- Insert default roles
INSERT INTO roles (id, name, description, created_at) VALUES
    ('role_admin', 'Admin', 'Full system access', datetime('now')),
    ('role_member', 'Member', 'Standard user access', datetime('now')),
    ('role_viewer', 'Viewer', 'Read-only access', datetime('now'));

-- Permissions
CREATE TABLE permissions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_permissions_name ON permissions(name);
CREATE INDEX idx_permissions_resource ON permissions(resource);

-- Insert default permissions
INSERT INTO permissions (id, name, resource, action, description, created_at) VALUES
    ('perm_project_create', 'project:create', 'project', 'create', 'Create new projects', datetime('now')),
    ('perm_project_read', 'project:read', 'project', 'read', 'View projects', datetime('now')),
    ('perm_project_update', 'project:update', 'project', 'update', 'Edit projects', datetime('now')),
    ('perm_project_delete', 'project:delete', 'project', 'delete', 'Delete projects', datetime('now')),
    ('perm_ticket_create', 'ticket:create', 'ticket', 'create', 'Create new tickets', datetime('now')),
    ('perm_ticket_read', 'ticket:read', 'ticket', 'read', 'View tickets', datetime('now')),
    ('perm_ticket_update', 'ticket:update', 'ticket', 'update', 'Edit tickets', datetime('now')),
    ('perm_ticket_delete', 'ticket:delete', 'ticket', 'delete', 'Delete tickets', datetime('now')),
    ('perm_comment_create', 'comment:create', 'comment', 'create', 'Create comments', datetime('now')),
    ('perm_comment_update', 'comment:update', 'comment', 'update', 'Edit own comments', datetime('now')),
    ('perm_comment_delete', 'comment:delete', 'comment', 'delete', 'Delete own comments', datetime('now')),
    ('perm_user_manage', 'user:manage', 'user', 'manage', 'Manage users', datetime('now'));

-- Role permissions mapping
CREATE TABLE role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE INDEX idx_role_perms_role ON role_permissions(role_id);
CREATE INDEX idx_role_perms_perm ON role_permissions(permission_id);

-- Assign permissions to default roles
-- Admin gets all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'role_admin', id FROM permissions;

-- Member gets read and write access (except delete and user management)
INSERT INTO role_permissions (role_id, permission_id) VALUES
    ('role_member', 'perm_project_create'),
    ('role_member', 'perm_project_read'),
    ('role_member', 'perm_project_update'),
    ('role_member', 'perm_ticket_create'),
    ('role_member', 'perm_ticket_read'),
    ('role_member', 'perm_ticket_update'),
    ('role_member', 'perm_comment_create'),
    ('role_member', 'perm_comment_update'),
    ('role_member', 'perm_comment_delete');

-- Viewer gets read-only access
INSERT INTO role_permissions (role_id, permission_id) VALUES
    ('role_viewer', 'perm_project_read'),
    ('role_viewer', 'perm_ticket_read');

-- User roles assignment (simplified: project_id defaults to '' for global roles)
CREATE TABLE user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    project_id TEXT NOT NULL DEFAULT '',
    assigned_at TEXT NOT NULL,
    PRIMARY KEY (user_id, role_id, project_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);
CREATE INDEX idx_user_roles_project ON user_roles(project_id);

-- Teams/Organizations
CREATE TABLE teams (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_teams_name ON teams(name);
CREATE INDEX idx_teams_created_by ON teams(created_by);

-- Team members
CREATE TABLE team_members (
    team_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    joined_at TEXT NOT NULL,
    PRIMARY KEY (team_id, user_id),
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id)
);

CREATE INDEX idx_team_members_team ON team_members(team_id);
CREATE INDEX idx_team_members_user ON team_members(user_id);

-- Project team association
CREATE TABLE project_teams (
    project_id TEXT NOT NULL,
    team_id TEXT NOT NULL,
    added_at TEXT NOT NULL,
    PRIMARY KEY (project_id, team_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
);

CREATE INDEX idx_project_teams_project ON project_teams(project_id);
CREATE INDEX idx_project_teams_team ON project_teams(team_id);

-- Activity log for audit trail
CREATE TABLE activity_log (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_activity_user ON activity_log(user_id);
CREATE INDEX idx_activity_resource ON activity_log(resource_type, resource_id);
CREATE INDEX idx_activity_created ON activity_log(created_at);

-- Add full-text search support for tickets
CREATE VIRTUAL TABLE tickets_fts USING fts5(
    title,
    description,
    content='tickets',
    content_rowid='rowid'
);

-- Triggers to keep FTS index updated
CREATE TRIGGER tickets_fts_insert AFTER INSERT ON tickets BEGIN
    INSERT INTO tickets_fts(rowid, title, description)
    VALUES (new.rowid, new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER tickets_fts_update AFTER UPDATE ON tickets BEGIN
    UPDATE tickets_fts SET title = new.title, description = COALESCE(new.description, '')
    WHERE rowid = new.rowid;
END;

CREATE TRIGGER tickets_fts_delete AFTER DELETE ON tickets BEGIN
    DELETE FROM tickets_fts WHERE rowid = old.rowid;
END;
