-- Agents subsystem: shared persona catalogue, per-project deployments,
-- tick history, lifecycle audit log. Plus the small ancillary changes
-- needed to support agent-driven epic decomposition (`tickets.parent_id`)
-- and to mark autonomous identity users (`users.is_agent`).
--
-- Conventions match V5/V6: TEXT for UUIDs and RFC3339 timestamps,
-- INTEGER for booleans, JSON-encoded TEXT for small collections that
-- aren't queried by content (expertise, capabilities, snapshot mirrors).

-- 1. Mark autonomous identity users.

ALTER TABLE users ADD COLUMN is_agent INTEGER NOT NULL DEFAULT 0;
CREATE INDEX idx_users_is_agent ON users(is_agent);

-- 2. Ticket parent/child for epic decomposition (used by wn_create_subtask).

ALTER TABLE tickets ADD COLUMN parent_id TEXT REFERENCES tickets(id) ON DELETE SET NULL;
CREATE INDEX idx_tickets_parent_id ON tickets(parent_id);

-- 3. Persona catalogue. Workspace-shared like tags.

CREATE TABLE personas (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    emoji TEXT NOT NULL,
    color TEXT NOT NULL,
    description TEXT NOT NULL,
    role TEXT NOT NULL,
    tone TEXT NOT NULL,
    expertise_json TEXT NOT NULL DEFAULT '[]',
    instructions TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    model TEXT NOT NULL,
    default_cron TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 4. Per-project deployments.

CREATE TABLE agent_deployments (
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

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    UNIQUE (project_id, persona_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE RESTRICT,
    FOREIGN KEY (agent_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    FOREIGN KEY (current_ticket_id) REFERENCES tickets(id) ON DELETE SET NULL
);

CREATE INDEX idx_agent_deployments_project ON agent_deployments(project_id);
CREATE INDEX idx_agent_deployments_persona ON agent_deployments(persona_id);
-- Scheduler hot path: pick deployments due to tick.
CREATE INDEX idx_agent_deployments_status_next_tick
    ON agent_deployments(status, next_tick_at);

-- 5. Per-execution tick records (append-only).

CREATE TABLE agent_ticks (
    id TEXT PRIMARY KEY NOT NULL,
    deployment_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    outcome TEXT,
    touched_ticket_id TEXT,
    action_summary TEXT,
    error_message TEXT,
    FOREIGN KEY (deployment_id) REFERENCES agent_deployments(id) ON DELETE CASCADE,
    FOREIGN KEY (touched_ticket_id) REFERENCES tickets(id) ON DELETE SET NULL
);

CREATE INDEX idx_agent_ticks_deployment_started
    ON agent_ticks(deployment_id, started_at DESC);

-- 6. Lifecycle event audit log (append-only).

CREATE TABLE agent_events (
    id TEXT PRIMARY KEY NOT NULL,
    deployment_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    message TEXT NOT NULL,
    at TEXT NOT NULL,
    FOREIGN KEY (deployment_id) REFERENCES agent_deployments(id) ON DELETE CASCADE
);

CREATE INDEX idx_agent_events_deployment_at
    ON agent_events(deployment_id, at DESC);

-- 7. Seed the 9 personas. UUIDs are pinned (V5 style) so frontends can
-- reference them by stable id. Six come from the design handoff library
-- (Triage / Reviewer / Bug Reproducer / Docs Writer / Standup / Researcher);
-- three are engineering personas (tech-lead / frontend / backend) wired up
-- to drive the Tamagotchi acceptance demo (epic decomposition via the
-- worknest-mcp `wn_create_subtask` tool).

INSERT INTO personas (
    id, slug, name, emoji, color, description,
    role, tone, expertise_json, instructions,
    capabilities_json, model, default_cron, created_at, updated_at
) VALUES
    -- Design library (6) -----------------------------------------------------
    ('22222222-2222-4222-8222-222222222201', 'triage', 'Triage', '🛎️', '#bae6fd',
     'Reads the inbox, classifies new tickets, applies labels, proposes priority and assigns to the most likely owner.',
     'Triage operator', 'Concise, neutral, never speculative',
     '["classification","priority heuristics","duplicate detection"]',
     'You are a triage operator. For each unlabeled ticket assigned to you (or unassigned in your project), read the description, infer the right tags, propose a priority, and reassign to the most likely owner. Comment a one-line rationale.',
     '["Comment","Label","Assign","SetPriority"]', 'Haiku',
     '*/30 9-18 * * 1-5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222202', 'reviewer', 'Reviewer', '🔍', '#c4b5fd',
     'Reviews PRs, design files, and tickets in Review status. Posts a structured review comment.',
     'Senior reviewer', 'Direct, kind, evidence-based',
     '["code review","a11y","design critique","performance"]',
     'You are a senior reviewer. For each ticket assigned to you in Review, post a comment with three sections: Strengths, Risks, Follow-ups. Cite specific evidence (line numbers, screenshots).',
     '["Comment","Attach"]', 'Sonnet',
     '0 10,15 * * 1-5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222203', 'reproducer', 'Bug Reproducer', '🐞', '#fecaca',
     'Tries to reproduce filed bugs in a sandbox and posts environment + steps + expected/actual + a recording.',
     'QA engineer', 'Methodical and forensic',
     '["reproduction","browser DevTools","logs","flake detection"]',
     'You are a bug reproducer. For each Bug ticket without a confirmed reproduction, attempt to reproduce. Comment Environment / Steps / Expected / Actual; attach logs or recordings; set status if confirmed.',
     '["Comment","Attach","SetStatus"]', 'Sonnet',
     '0 */2 * * 1-5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222204', 'docs', 'Docs Writer', '📝', '#a7f3d0',
     'Drafts changelogs and docs notes for the week''s shipped tickets.',
     'Technical writer', 'Plain, friendly, second-person',
     '["changelogs","help center","API docs"]',
     'You are a technical writer. On Friday EOD, group Done tickets and draft changelog/release notes; create a docs ticket if substantive.',
     '["CreateTicket","Comment"]', 'Haiku',
     '0 17 * * 5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222205', 'standup', 'Standup', '☕️', '#fde68a',
     'Posts a weekday digest of yesterday''s moved tickets and current blockers.',
     'Team facilitator', 'Warm, brief, scannable, bulleted',
     '["summarization","blocker detection"]',
     'You are a standup facilitator. Weekday 09:30: scan the project and post a comment digest with a short list of moved tickets and visible blockers.',
     '["Comment"]', 'Haiku',
     '30 9 * * 1-5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222206', 'researcher', 'Researcher', '🔬', '#fbcfe8',
     'Gathers competitor screenshots, prior art, and research notes for product/UX questions.',
     'UX researcher', 'Curious, citation-heavy, never opinionated',
     '["competitive analysis","user-interview synthesis","heuristics"]',
     'You are a UX researcher. For each research-tagged ticket, gather 3–5 references; comment a short synthesis with links/screenshots.',
     '["Comment","Attach"]', 'Sonnet',
     '0 11 * * 1-5',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    -- Engineering trio (3) — used by the Tamagotchi acceptance demo. ----------
    ('22222222-2222-4222-8222-222222222207', 'tech-lead', 'Tech Lead', '🧭', '#bfdbfe',
     'Engineering manager. Decomposes Epic tickets into subtasks and reviews work in Review. Does not implement features.',
     'Engineering manager', 'Direct, decisive, brief',
     '["scoping","decomposition","code review"]',
     'You are the tech-lead. Your job is to ORCHESTRATE, not implement. On every tick: (1) Find Epic tickets assigned to you with no children. For each, decompose into 2-5 subtasks via wn_create_subtask(parent_id=<epic_id>, title=..., assignee_persona=<frontend|backend|...>, priority=...) and reassign each child to the most relevant engineering persona, then move the Epic to InProgress. (2) Review tickets assigned to you in Review and either wn_finish them (target_status=Done) or wn_handoff back with a precise question.',
     '["Comment","Assign","CreateTicket","SetStatus"]', 'Sonnet',
     '*/5 * * * *',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222208', 'frontend', 'Frontend Dev', '🎨', '#fcd34d',
     'Frontend engineer. Drives UI/SPA tickets to Review. Posts a 3-6 bullet plan as the first comment.',
     'Frontend engineer', 'Pragmatic, focused on UX',
     '["React","TypeScript","CSS","accessibility","SPA architecture"]',
     'You are a frontend engineer. Pick one ticket assigned to you in Open via wn_claim_ticket, post a 3-6 bullet plan as the first comment, drive the work, then wn_finish (status=Review). If blocked, wn_handoff back to tech-lead with a precise question.',
     '["Comment","SetStatus","Assign","Attach"]', 'Sonnet',
     '*/30 * * * *',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),

    ('22222222-2222-4222-8222-222222222209', 'backend', 'Backend Dev', '⚙️', '#86efac',
     'Backend engineer. Drives API/persistence tickets to Review. Posts a 3-6 bullet plan as the first comment.',
     'Backend engineer', 'Pragmatic, focused on correctness',
     '["Rust","Axum","SQLite","REST API","testing"]',
     'You are a backend engineer. Pick one ticket assigned to you in Open via wn_claim_ticket, post a 3-6 bullet plan as the first comment, drive the work, then wn_finish (status=Review). If blocked, wn_handoff back to tech-lead with a precise question.',
     '["Comment","SetStatus","Assign","Attach"]', 'Sonnet',
     '*/30 * * * *',
     strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'));
