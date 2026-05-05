-- Project membership: who, besides the project owner (`projects.created_by`),
-- may read tickets/comments and contribute to a project.
--
-- The previous visibility rule was owner-only. Multi-agent setups (e.g. the
-- cl_agent swarm) need every persona to share read/comment access on a
-- single project. `created_by` continues to gate write operations on the
-- project itself (update/delete/archive); membership grants the
-- ticket/comment surface only.

CREATE TABLE project_members (
    project_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',
    added_at TEXT NOT NULL,
    PRIMARY KEY (project_id, user_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_project_members_project ON project_members(project_id);
CREATE INDEX idx_project_members_user ON project_members(user_id);

-- Backfill: every existing project's owner becomes an explicit member, so the
-- "owner OR member" union check has uniform shape going forward.
INSERT INTO project_members (project_id, user_id, role, added_at)
SELECT id, created_by, 'owner', datetime('now') FROM projects;
