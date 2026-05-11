-- Tags / labels for tickets.
--
-- The new React UI exposes a categorical tag picker on tickets ("bug",
-- "feature", "design", "research", "docs", "chore" by default) with paired
-- background/foreground colors that match the design tokens. Until now the
-- backend had no concept of tags, so the UI mocked them. This migration
-- introduces a real tags table plus a many-to-many ticket_tags join.

CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color_bg TEXT NOT NULL,
    color_fg TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE ticket_tags (
    ticket_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (ticket_id, tag_id),
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_ticket_tags_ticket ON ticket_tags(ticket_id);
CREATE INDEX idx_ticket_tags_tag ON ticket_tags(tag_id);

-- Seed the design's six default tags. UUIDs are pinned so frontends can
-- reference them by stable id; if a deployment wants different defaults it
-- can DELETE these rows and INSERT its own.
INSERT INTO tags (id, name, color_bg, color_fg, created_at) VALUES
    ('11111111-1111-4111-8111-111111111101', 'bug',      '#fee2e2', '#991b1b', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('11111111-1111-4111-8111-111111111102', 'feature',  '#dbeafe', '#1e40af', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('11111111-1111-4111-8111-111111111103', 'design',   '#fce7f3', '#9d174d', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('11111111-1111-4111-8111-111111111104', 'research', '#e0e7ff', '#3730a3', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('11111111-1111-4111-8111-111111111105', 'docs',     '#d1fae5', '#065f46', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('11111111-1111-4111-8111-111111111106', 'chore',    '#f1f5f9', '#475569', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
