//! Agents subsystem: activation pipeline + cron-driven tick scheduler.
//!
//! Each deployment gets its own on-disk workspace under `WORKNEST_AGENTS_DIR`
//! (a git worktree of the project repo when `repo_path` is set), a JWT-backed
//! agent identity, an `flock`-style advisory lock for tick mutual exclusion,
//! and a stateless tick that drives one ticket toward a terminal state per
//! session. The MCP server agents talk to lives in this repo at
//! `worknest-mcp/`; nothing outside the workspace tree is required.

pub mod activation;
pub mod git;
pub mod scheduler;
pub mod templates;
pub mod templates_render;
pub mod tick_executor;
pub mod workspace;

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Runtime configuration for the agents subsystem. Built once at boot from
/// environment variables and stored on `AppState`.
#[derive(Debug, Clone)]
pub struct AgentsConfig {
    /// Root directory holding one subfolder per deployment. Default
    /// `~/.local/share/worknest/agents` (or `./worknest-agents` if HOME
    /// is unset).
    pub agents_dir: PathBuf,
    /// Absolute path to the in-repo `worknest-mcp/` directory (the Python
    /// package referenced from each deployment's `.mcp.json`). Auto-detected
    /// from the binary location at boot; set `WORKNEST_AGENT_MCP_DIR` to
    /// override (deployed binaries that live outside the repo, packaged
    /// builds, etc.).
    pub mcp_dir: Option<PathBuf>,
    /// Binary name (or absolute path) for the `claude` CLI. Default `claude`,
    /// resolved via `PATH`.
    pub claude_bin: String,
    /// Public URL the MCP server uses to reach Worknest, e.g.
    /// `http://localhost:3000`. Defaults to `http://localhost:<PORT>`.
    pub worknest_url: String,
    /// Hard ceiling per tick subprocess in seconds. Default 600.
    pub tick_timeout_secs: u64,
}

impl AgentsConfig {
    pub fn from_env(default_port: u16) -> Self {
        let agents_dir = std::env::var("WORKNEST_AGENTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join(".local/share/worknest/agents")
            });
        let mcp_dir = std::env::var("WORKNEST_AGENT_MCP_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(default_mcp_dir);
        let claude_bin = std::env::var("WORKNEST_CLAUDE_BIN").unwrap_or_else(|_| "claude".into());
        let worknest_url = std::env::var("WORKNEST_PUBLIC_URL")
            .unwrap_or_else(|_| format!("http://localhost:{default_port}"));
        let tick_timeout_secs = std::env::var("WORKNEST_AGENT_TICK_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);
        Self {
            agents_dir,
            mcp_dir,
            claude_bin,
            worknest_url,
            tick_timeout_secs,
        }
    }
}

/// Wrapped Arc so handlers can clone cheaply.
pub type SharedAgentsConfig = Arc<AgentsConfig>;

/// Preflight diagnostics. Logged at boot so the operator notices
/// misconfigurations *before* a deployment fails activation. Each check is
/// non-fatal — agents simply won't tick until the underlying problem is
/// fixed, but the rest of the API runs fine.
pub fn preflight_check(cfg: &AgentsConfig) {
    // MCP directory must exist with a pyproject.toml; the .venv is recommended
    // but not required (uv resolves on first invocation).
    match cfg.mcp_dir.as_deref() {
        None => {
            tracing::warn!(
                "[agents preflight] could not locate the worknest-mcp/ directory and \
                 WORKNEST_AGENT_MCP_DIR is not set. Agent activations will fail at \
                 ProvisionWorkspace until this is resolved. Run `cd worknest-mcp && uv \
                 sync` from a clone of this repo, or point WORKNEST_AGENT_MCP_DIR at the \
                 worknest-mcp/ directory of an existing checkout."
            );
        },
        Some(dir) => {
            if !dir.is_dir() {
                tracing::warn!(
                    "[agents preflight] WORKNEST_AGENT_MCP_DIR={} is not a directory.",
                    dir.display()
                );
            } else if !dir.join("pyproject.toml").exists() {
                tracing::warn!(
                    "[agents preflight] WORKNEST_AGENT_MCP_DIR={} has no pyproject.toml — does \
                     this point at the worknest-mcp package?",
                    dir.display()
                );
            } else if !dir.join(".venv").exists() {
                tracing::warn!(
                    "[agents preflight] No .venv inside {}. The first tick will trigger `uv \
                     sync`; if the agent has no network or `uv` is missing, ticks will fail. \
                     Recommended: run `uv sync` once now from inside that dir.",
                    dir.display()
                );
            } else {
                tracing::info!(
                    "[agents preflight] MCP dir OK at {} (.venv present)",
                    dir.display()
                );
            }
        },
    }

    // claude binary discoverability.
    match resolve_binary(&cfg.claude_bin) {
        Some(path) => {
            tracing::info!(
                "[agents preflight] claude CLI resolved to {} (from WORKNEST_CLAUDE_BIN={})",
                path.display(),
                cfg.claude_bin
            );
        },
        None => {
            tracing::warn!(
                "[agents preflight] claude CLI not found (WORKNEST_CLAUDE_BIN={}). Ticks will \
                 fail to spawn a subprocess until claude is on PATH or this var is set to an \
                 absolute path.",
                cfg.claude_bin
            );
        },
    }

    // git binary — required for any project with repo_path.
    if which_path("git").is_none() {
        tracing::warn!(
            "[agents preflight] `git` not found on PATH. Deployments to projects with a \
             repo_path will fail at BootstrapWorktree."
        );
    }

    // Agents dir must be writable.
    match std::fs::create_dir_all(&cfg.agents_dir) {
        Ok(()) => {
            tracing::info!(
                "[agents preflight] agents dir OK at {}",
                cfg.agents_dir.display()
            );
        },
        Err(e) => {
            tracing::warn!(
                "[agents preflight] could not create agents dir {}: {:?}",
                cfg.agents_dir.display(),
                e
            );
        },
    }
}

/// Auto-locate the in-repo `worknest-mcp/` directory.
///
/// Tries, in order:
///   1. `CARGO_MANIFEST_DIR/../../worknest-mcp` — the workspace-relative
///      location when running `cargo run -p worknest-api` from a checkout.
///   2. Walking upward from the running executable's directory looking for
///      a `worknest-mcp/pyproject.toml` (covers `cargo run`, an installed
///      `target/release/worknest-api`, and packaged builds where the binary
///      ships next to the MCP source).
///   3. Walking upward from CWD as a last resort.
///
/// Returns `None` if no candidate validates — the operator must then set
/// `WORKNEST_AGENT_MCP_DIR` explicitly. The preflight check warns clearly
/// in that case.
fn default_mcp_dir() -> Option<PathBuf> {
    fn validate(p: PathBuf) -> Option<PathBuf> {
        if p.join("pyproject.toml").is_file() {
            Some(p)
        } else {
            None
        }
    }
    fn walk_up(start: &Path) -> Option<PathBuf> {
        let mut cur = Some(start);
        while let Some(d) = cur {
            if let Some(hit) = validate(d.join("worknest-mcp")) {
                return Some(hit);
            }
            cur = d.parent();
        }
        None
    }

    // 1. Cargo manifest dir → walk up from `crates/worknest-api/`.
    if let Some(hit) = walk_up(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        return Some(hit);
    }
    // 2. Exe dir → walk up from `target/<profile>/worknest-api`.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(hit) = walk_up(parent) {
                return Some(hit);
            }
        }
    }
    // 3. CWD fallback.
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(hit) = walk_up(&cwd) {
            return Some(hit);
        }
    }
    None
}

fn resolve_binary(spec: &str) -> Option<PathBuf> {
    let p = std::path::Path::new(spec);
    if p.is_absolute() {
        if p.is_file() {
            return Some(p.to_path_buf());
        }
        return None;
    }
    which_path(spec)
}

/// Minimal `which` — scan PATH for an executable file. Avoids pulling in the
/// `which` crate just for the preflight check.
fn which_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_finds_a_known_binary() {
        // `sh` is on PATH on every supported dev environment we care about.
        assert!(which_path("sh").is_some());
        assert!(which_path("definitely-not-a-real-binary-zzzz").is_none());
    }

    #[test]
    fn resolve_binary_handles_absolute_and_path_lookup() {
        let sh = which_path("sh").expect("sh on PATH");
        assert_eq!(resolve_binary(sh.to_str().unwrap()), Some(sh));
        assert!(resolve_binary("sh").is_some());
        assert!(resolve_binary("definitely-not-a-real-binary-zzzz").is_none());
    }
}
