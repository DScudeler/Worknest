//! Per-deployment workspace directories.
//!
//! Layout under `<AGENTS_DIR>/<deployment_id>/`:
//!
//! ```text
//!   CLAUDE.md
//!   .mcp.json
//!   tick.lock                  // advisory file lock for the tick executor
//!   .claude/
//!     settings.json
//!     token                    // chmod 600 — JWT for the agent identity user
//!     commands/
//!       agent-tick.md
//!   logs/
//!     <tick_id>.log             // stdout+stderr per tick run
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use worknest_core::models::{AgentDeployment, Persona};

use super::templates::{AGENT_TICK_MD, CLAUDE_MD, GUARD_SCRIPT, MCP_JSON, SETTINGS_JSON};
use super::templates_render::{render, RenderError};

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("io error in {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("MCP directory not configured (set WORKNEST_AGENT_MCP_DIR)")]
    McpDirMissing,
}

fn io<C: Into<String>>(context: C) -> impl FnOnce(std::io::Error) -> WorkspaceError {
    move |e| WorkspaceError::Io {
        context: context.into(),
        source: e,
    }
}

/// Render each persona's `instructions` into a Markdown body for inlining
/// into `CLAUDE.md`. Currently a single passthrough; kept as a function so
/// future overlay merging has one place to grow.
fn persona_definition(persona: &Persona) -> String {
    let expertise = if persona.expertise.is_empty() {
        String::new()
    } else {
        format!("\n\n**Expertise**: {}.", persona.expertise.join(", "))
    };
    format!(
        "**Role**: {role}\n\n**Tone**: {tone}{expertise}\n\n{instructions}",
        role = persona.role,
        tone = persona.tone,
        instructions = persona.instructions,
    )
}

/// Provision (or re-provision) the workspace for a deployment. Idempotent:
/// re-running rewrites all generated files from the current snapshot/template
/// state and is safe at any point in the activation pipeline.
///
/// `mcp_dir` and `worknest_url` come from `AgentsConfig`. `jwt` is the agent
/// identity user's freshly minted token.
pub fn provision(
    deployment: &AgentDeployment,
    persona: &Persona,
    agents_dir: &Path,
    mcp_dir: Option<&Path>,
    worknest_url: &str,
    jwt: &str,
    personas_map: &HashMap<String, String>,
) -> Result<PathBuf, WorkspaceError> {
    let mcp_dir = mcp_dir.ok_or(WorkspaceError::McpDirMissing)?;

    let dir = agents_dir.join(deployment.id.to_string());
    let claude_dir = dir.join(".claude");
    let commands_dir = claude_dir.join("commands");
    let logs_dir = dir.join("logs");

    std::fs::create_dir_all(&dir).map_err(io("create deployment dir"))?;
    std::fs::create_dir_all(&claude_dir).map_err(io("create .claude dir"))?;
    std::fs::create_dir_all(&commands_dir).map_err(io("create commands dir"))?;
    std::fs::create_dir_all(&logs_dir).map_err(io("create logs dir"))?;

    let project_id = deployment.project_id.to_string();
    let deployment_id = deployment.id.to_string();
    let token_file = claude_dir.join("token");
    let guard_script = claude_dir.join("guard-worktree.sh");

    // Project-shared persona→user_id map, written under
    // <agents_dir>/_projects/<project_id>/personas.json. The MCP server's
    // WORKNEST_PERSONAS env var points every deployment in this project at
    // the same file so handoffs (`wn_handoff(to_persona="frontend", ...)`)
    // can resolve any sibling agent's user_id.
    let project_dir = agents_dir.join("_projects").join(&project_id);
    std::fs::create_dir_all(&project_dir).map_err(io("create _projects dir"))?;
    let personas_path = project_dir.join("personas.json");
    let personas_json =
        serde_json::to_string_pretty(personas_map).map_err(|e| WorkspaceError::Io {
            context: "serialize personas.json".into(),
            source: std::io::Error::other(e.to_string()),
        })?;
    std::fs::write(&personas_path, personas_json + "\n").map_err(io("write personas.json"))?;

    // Build the placeholder map once and reuse for every template.
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("persona_slug", persona.slug.clone());
    vars.insert("persona_name", persona.name.clone());
    vars.insert("persona_definition", persona_definition(persona));
    vars.insert("project_id", project_id);
    vars.insert("worknest_url", worknest_url.to_string());
    vars.insert("deployment_id", deployment_id);
    vars.insert("token_file", token_file.display().to_string());
    vars.insert("mcp_dir", mcp_dir.display().to_string());
    vars.insert("personas_path", personas_path.display().to_string());
    vars.insert("workspace_path", dir.display().to_string());
    vars.insert("guard_script", guard_script.display().to_string());

    let claude_md = render(CLAUDE_MD, &vars)?;
    let mcp_json = render(MCP_JSON, &vars)?;
    let settings_json = render(SETTINGS_JSON, &vars)?;
    let agent_tick_md = render(AGENT_TICK_MD, &vars)?;

    std::fs::write(dir.join("CLAUDE.md"), claude_md).map_err(io("write CLAUDE.md"))?;
    std::fs::write(dir.join(".mcp.json"), mcp_json).map_err(io("write .mcp.json"))?;
    std::fs::write(claude_dir.join("settings.json"), settings_json)
        .map_err(io("write settings.json"))?;
    std::fs::write(commands_dir.join("agent-tick.md"), agent_tick_md)
        .map_err(io("write agent-tick.md"))?;

    // Token file: chmod 600 on Unix so it's not world-readable.
    std::fs::write(&token_file, jwt).map_err(io("write token"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&token_file)
            .map_err(io("stat token file"))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&token_file, perms).map_err(io("chmod token file"))?;
    }

    // PreToolUse worktree guard: each deployment ships its own copy so a
    // moved or repackaged Worknest install can't break already-active
    // agents. chmod 755 — Claude Code spawns it via the hook runner.
    std::fs::write(&guard_script, GUARD_SCRIPT).map_err(io("write guard script"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&guard_script)
            .map_err(io("stat guard script"))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&guard_script, perms).map_err(io("chmod guard script"))?;
    }

    // Touch the advisory lock file so flock() has a target.
    let lock_path = dir.join("tick.lock");
    if !lock_path.exists() {
        std::fs::write(&lock_path, b"").map_err(io("create tick.lock"))?;
    }

    // When the workspace IS a git worktree, keep the agent-config files out
    // of `git status` so the agent doesn't accidentally `git add .` them.
    // Writing a top-level `.gitignore` doesn't work because the canonical
    // checkout's tracked .gitignore wins; instead append our patterns to
    // the worktree's `info/exclude` (the git-managed personal exclude).
    if dir.join(".git").exists() {
        if let Err(e) = ensure_worktree_excludes(&dir) {
            tracing::warn!(
                "could not write info/exclude in {}: {:?} — agent may see CLAUDE.md as untracked",
                dir.display(),
                e
            );
        }
    }

    Ok(dir)
}

/// Append agent-config exclude patterns to the worktree's `info/exclude`
/// file. Idempotent via a magic-marker line.
fn ensure_worktree_excludes(workspace: &Path) -> std::io::Result<()> {
    use std::process::Command;
    const MARKER: &str = "# worknest-agent-config";
    // `--git-common-dir` returns the *shared* .git/ that all worktrees of
    // this repo see for `info/exclude` (linked worktrees have a separate
    // `--absolute-git-dir` that git does NOT consult for excludes).
    let out = Command::new("git")
        .args(["-C"])
        .arg(workspace)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()?;
    if !out.status.success() {
        return Err(std::io::Error::other(format!(
            "rev-parse failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let gitdir = std::path::PathBuf::from(&raw);
    // If git ≥ 2.31 honoured --path-format=absolute we get an absolute path
    // already; older git returns a relative path → resolve from workspace.
    let gitdir = if gitdir.is_absolute() {
        gitdir
    } else {
        workspace.join(gitdir)
    };
    let exclude = gitdir.join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap())?;
    let existing = std::fs::read_to_string(&exclude).unwrap_or_default();
    if existing.contains(MARKER) {
        return Ok(());
    }
    let mut out_text = existing;
    if !out_text.is_empty() && !out_text.ends_with('\n') {
        out_text.push('\n');
    }
    out_text.push_str(MARKER);
    out_text.push_str("\nCLAUDE.md\n.mcp.json\n.claude/\nlogs/\ntick.lock\n");
    std::fs::write(&exclude, out_text)?;
    Ok(())
}

/// Best-effort cleanup of a deployment's workspace. Logs but does not surface
/// errors — a failed teardown is not worth blocking the DELETE handler over.
pub fn tear_down(agents_dir: &Path, deployment_id: worknest_core::models::AgentDeploymentId) {
    let dir = agents_dir.join(deployment_id.to_string());
    if let Err(e) = std::fs::remove_dir_all(&dir) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!("failed to remove workspace {}: {:?}", dir.display(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use worknest_core::models::{
        AgentDeploymentId, AgentModel, AgentStatus, Capability, PersonaId, ProjectId,
    };

    fn sample_persona() -> Persona {
        let now = Utc::now();
        Persona {
            id: PersonaId::new(),
            slug: "tech-lead".into(),
            name: "Tech Lead".into(),
            emoji: "🧭".into(),
            color: "#bfdbfe".into(),
            description: "x".into(),
            role: "Engineering manager".into(),
            tone: "Direct".into(),
            expertise: vec!["scoping".into()],
            instructions: "Decompose epics.".into(),
            capabilities: vec![Capability::CreateTicket, Capability::Comment],
            model: AgentModel::Sonnet,
            default_cron: "*/5 * * * *".into(),
            created_at: now,
            updated_at: now,
        }
    }

    fn sample_deployment(persona_id: PersonaId) -> AgentDeployment {
        let now = Utc::now();
        AgentDeployment {
            id: AgentDeploymentId::new(),
            project_id: ProjectId::new(),
            persona_id,
            agent_user_id: None,
            snapshot_name: None,
            snapshot_role: None,
            snapshot_tone: None,
            snapshot_expertise: vec![],
            snapshot_instructions: None,
            snapshot_capabilities: vec![],
            snapshot_model: None,
            snapshot_taken_at: None,
            workspace_path: None,
            cron_expression: "*/5 * * * *".into(),
            next_tick_at: None,
            tick_locked_at: None,
            tick_lock_token: None,
            status: AgentStatus::Pending,
            last_error_step: None,
            error_message: None,
            error_count: 0,
            current_ticket_id: None,
            runs_today: 0,
            touched_this_week: 0,
            success_rate: 0.0,
            last_activity_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn provisions_all_files() {
        let tmp = tempfile_dir();
        let mcp = tempfile_dir();
        let persona = sample_persona();
        let depl = sample_deployment(persona.id);
        let mut personas_map = HashMap::new();
        personas_map.insert(persona.slug.clone(), uuid::Uuid::new_v4().to_string());
        let dir = provision(
            &depl,
            &persona,
            &tmp,
            Some(&mcp),
            "http://localhost:3000",
            "fake.jwt.token",
            &personas_map,
        )
        .unwrap();

        // The shared personas.json lands under _projects/<pid>/.
        let personas_path = tmp
            .join("_projects")
            .join(depl.project_id.to_string())
            .join("personas.json");
        assert!(personas_path.exists());
        let mcp_json = std::fs::read_to_string(dir.join(".mcp.json")).unwrap();
        assert!(
            mcp_json.contains(&personas_path.display().to_string()),
            "WORKNEST_PERSONAS should point at the shared personas.json"
        );

        assert!(dir.join("CLAUDE.md").exists());
        assert!(dir.join(".mcp.json").exists());
        assert!(dir.join(".claude/settings.json").exists());
        assert!(dir.join(".claude/commands/agent-tick.md").exists());
        assert!(dir.join(".claude/token").exists());
        assert!(dir.join(".claude/guard-worktree.sh").exists());
        assert!(dir.join("tick.lock").exists());

        // Settings should pin CLAUDE_PROJECT_DIR to the workspace and wire
        // the PreToolUse hook to the per-deployment guard script.
        let settings = std::fs::read_to_string(dir.join(".claude/settings.json")).unwrap();
        assert!(settings.contains(&dir.display().to_string()));
        assert!(settings.contains("guard-worktree.sh"));
        assert!(settings.contains("\"PreToolUse\""));

        // Guard script should be executable on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.join(".claude/guard-worktree.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }

        let claude_md = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert!(claude_md.contains("Tech Lead"));
        assert!(claude_md.contains(&depl.project_id.to_string()));

        let mcp_json = std::fs::read_to_string(dir.join(".mcp.json")).unwrap();
        assert!(mcp_json.contains(&mcp.display().to_string()));
    }

    fn tempfile_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("worknest-agents-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
