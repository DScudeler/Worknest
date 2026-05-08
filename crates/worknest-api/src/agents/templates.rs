//! Static templates rendered into per-deployment workspaces. Bundled into
//! the binary via `include_str!` so the running API doesn't depend on the
//! source tree at runtime.

pub const CLAUDE_MD: &str = include_str!("templates/agent-claude-md.tmpl");
pub const MCP_JSON: &str = include_str!("templates/agent-mcp.json.tmpl");
pub const SETTINGS_JSON: &str = include_str!("templates/agent-settings.json.tmpl");
pub const AGENT_TICK_MD: &str = include_str!("templates/agent-tick.md");
