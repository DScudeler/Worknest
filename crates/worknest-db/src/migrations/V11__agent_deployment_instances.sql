-- Allow several deployments of the same persona per project so a project
-- can horizontally scale a worker (e.g. 3× Backend Dev) to absorb load.
--
-- All instances of a persona share `agent_user_id` (the existing
-- find-or-create logic in `register_identity` reuses
-- `agent-<slug>@worknest.local`), so work distributes for free via the
-- optimistic-concurrency `wn_claim_ticket` race. The new
-- `instance_index` column only exists to disambiguate the per-instance
-- git worktree branch (`swarm/<slug>` for index 1, `swarm/<slug>-<n>`
-- for ≥2) and to label the row in the UI.
--
-- SQLite cannot DROP a CONSTRAINT in place, so we rebuild the table.

PRAGMA foreign_keys = OFF;

CREATE TABLE agent_deployments_new (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    persona_id TEXT NOT NULL,
    agent_user_id TEXT,

    snapshot_name TEXT,
    snapshot_role TEXT,
    snapshot_tone TEXT,
    snapshot_expertise_json TEXT NOT NULL DEFAULT '[]',
    snapshot_instructions TEXT,
    snapshot_capabilities_json TEXT NOT NULL DEFAULT '[]',
    snapshot_model TEXT,
    snapshot_taken_at TEXT,

    workspace_path TEXT,
    cron_expression TEXT NOT NULL,
    next_tick_at TEXT,
    tick_locked_at TEXT,
    tick_lock_token TEXT,

    status TEXT NOT NULL,
    last_error_step TEXT,
    error_message TEXT,
    error_count INTEGER NOT NULL DEFAULT 0,
    current_ticket_id TEXT,

    runs_today INTEGER NOT NULL DEFAULT 0,
    touched_this_week INTEGER NOT NULL DEFAULT 0,
    success_rate REAL NOT NULL DEFAULT 0.0,
    last_activity_at TEXT,

    instance_index INTEGER NOT NULL DEFAULT 1,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE RESTRICT,
    FOREIGN KEY (agent_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    FOREIGN KEY (current_ticket_id) REFERENCES tickets(id) ON DELETE SET NULL
);

INSERT INTO agent_deployments_new (
    id, project_id, persona_id, agent_user_id,
    snapshot_name, snapshot_role, snapshot_tone, snapshot_expertise_json,
    snapshot_instructions, snapshot_capabilities_json, snapshot_model, snapshot_taken_at,
    workspace_path, cron_expression, next_tick_at, tick_locked_at, tick_lock_token,
    status, last_error_step, error_message, error_count, current_ticket_id,
    runs_today, touched_this_week, success_rate, last_activity_at,
    instance_index, created_at, updated_at
)
SELECT
    id, project_id, persona_id, agent_user_id,
    snapshot_name, snapshot_role, snapshot_tone, snapshot_expertise_json,
    snapshot_instructions, snapshot_capabilities_json, snapshot_model, snapshot_taken_at,
    workspace_path, cron_expression, next_tick_at, tick_locked_at, tick_lock_token,
    status, last_error_step, error_message, error_count, current_ticket_id,
    runs_today, touched_this_week, success_rate, last_activity_at,
    1, created_at, updated_at
FROM agent_deployments;

DROP TABLE agent_deployments;
ALTER TABLE agent_deployments_new RENAME TO agent_deployments;

-- Recreate the indexes from V7 verbatim, plus a non-unique composite
-- index for the next_instance_index lookup (cheap MAX scan).
CREATE INDEX idx_agent_deployments_project ON agent_deployments(project_id);
CREATE INDEX idx_agent_deployments_persona ON agent_deployments(persona_id);
CREATE INDEX idx_agent_deployments_status_next_tick
    ON agent_deployments(status, next_tick_at);
CREATE INDEX idx_agent_deployments_project_persona
    ON agent_deployments(project_id, persona_id);

PRAGMA foreign_keys = ON;
