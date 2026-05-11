//! Static templates rendered into per-deployment workspaces. Bundled into
//! the binary via `include_str!` so the running API doesn't depend on the
//! source tree at runtime.

pub const CLAUDE_MD: &str = include_str!("templates/agent-claude-md.tmpl");
pub const MCP_JSON: &str = include_str!("templates/agent-mcp.json.tmpl");
pub const SETTINGS_JSON: &str = include_str!("templates/agent-settings.json.tmpl");
pub const AGENT_TICK_MD: &str = include_str!("templates/agent-tick.md");

/// PreToolUse hook script. Writes to `<deployment>/.claude/guard-worktree.sh`
/// at provision time, chmod 755, so each deployment is self-contained.
pub const GUARD_SCRIPT: &str = include_str!("scripts/guard-worktree.sh");
