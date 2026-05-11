//! Git worktree bootstrap for a deployment.
//!
//! Each project has a single "canonical" checkout (cloned once into
//! `<AGENTS_DIR>/_projects/<pid>/repo`, or the operator-supplied local path
//! used directly). Each deployment's workspace dir IS its own git worktree,
//! sharing `.git/` with the canonical. The branch name is
//! `swarm/<persona-slug>` for the first instance of a persona and
//! `swarm/<persona-slug>-<n>` for sibling instances (n ≥ 2) so multiple
//! deployments of the same persona can coexist as parallel worktrees.
//!
//! `repo_path == None` is a no-op aside from `mkdir -p <workspace>`: agents
//! that don't write code (Triage, Standup, Docs Writer, …) skip the worktree
//! step but still get a workspace directory for the rendered config files.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git not found on PATH ({0})")]
    Spawn(String),
    #[error("git command failed: {stderr}")]
    Failed { stderr: String },
    #[error("invalid repo_path {0:?}")]
    InvalidRepoPath(String),
    #[error("io error in {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for GitError {
    fn from(source: std::io::Error) -> Self {
        GitError::Io {
            context: "git op".into(),
            source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapOutcome {
    /// Path to the per-deployment worktree (`<workspace>/repo`). Always present.
    pub worktree_path: PathBuf,
    /// Branch name created/reused on the canonical repo.
    pub branch: String,
    /// Path to the canonical checkout the worktree shares its `.git/` with.
    pub canonical_path: PathBuf,
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ssh://")
        || s.starts_with("git@")
        || s.starts_with("git://")
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    let output = cmd.output().map_err(|e| GitError::Spawn(e.to_string()))?;
    if !output.status.success() {
        return Err(GitError::Failed {
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

/// Resolve (or create) the canonical checkout that worktrees share. Returns
/// the absolute path to the canonical repo.
fn resolve_canonical(
    repo_path: &str,
    project_id: &str,
    agents_dir: &Path,
) -> Result<PathBuf, GitError> {
    let trimmed = repo_path.trim();
    if trimmed.is_empty() {
        return Err(GitError::InvalidRepoPath(repo_path.to_string()));
    }

    // Local path that's already a git repo → use directly.
    let p = Path::new(trimmed);
    if p.is_dir() {
        let dot_git = p.join(".git");
        if dot_git.exists() {
            return Ok(p.to_path_buf());
        }
        return Err(GitError::InvalidRepoPath(format!(
            "{} is a directory but is not a git repo (no .git/)",
            p.display()
        )));
    }

    // Treat as a clone URL; cache once per project.
    if !looks_like_url(trimmed) {
        return Err(GitError::InvalidRepoPath(format!(
            "{trimmed} is neither an existing local repo nor a recognised clone URL"
        )));
    }
    let cache_root = agents_dir.join("_projects").join(project_id);
    std::fs::create_dir_all(&cache_root).map_err(|e| GitError::Io {
        context: "create _projects cache".into(),
        source: e,
    })?;
    let cache = cache_root.join("repo");
    if cache.join(".git").exists() {
        // Best-effort fetch; ignore failure (offline runs are valid).
        let _ = run_git(
            &[
                "-C",
                &cache.display().to_string(),
                "fetch",
                "--all",
                "--prune",
            ],
            None,
        );
        return Ok(cache);
    }
    run_git(
        &[
            "clone",
            trimmed,
            cache
                .to_str()
                .ok_or_else(|| GitError::InvalidRepoPath(cache.display().to_string()))?,
        ],
        None,
    )?;
    Ok(cache)
}

/// Bootstrap (or re-bootstrap) the per-deployment worktree.
///
/// The deployment's workspace dir IS the git worktree, with project source
/// files checked out at the workspace root. When `project_repo_path` is
/// `None` the function only ensures the dir exists — the agent gets a
/// config-only workspace and cannot edit code.
///
/// Idempotent: if `<workspace>/.git` already exists the function returns
/// success without touching anything. This is what makes the pipeline safe
/// to retry from `BootstrapWorktree` after a crash.
pub fn bootstrap(
    project_repo_path: Option<&str>,
    project_id: &str,
    persona_slug: &str,
    instance_index: i32,
    workspace: &Path,
    agents_dir: &Path,
) -> Result<Option<BootstrapOutcome>, GitError> {
    let Some(rp) = project_repo_path else {
        // No repo — ensure the workspace exists as a plain directory so
        // the subsequent ProvisionWorkspace step has somewhere to write.
        std::fs::create_dir_all(workspace).map_err(|e| GitError::Io {
            context: "mkdir workspace (no repo_path)".into(),
            source: e,
        })?;
        return Ok(None);
    };
    let canonical = resolve_canonical(rp, project_id, agents_dir)?;
    // Instance 1 keeps the legacy `swarm/<slug>` name so existing
    // deployments from before V11 don't need re-bootstrapping. Sibling
    // instances get a `-<n>` suffix to avoid the worktree-shares-branch
    // collision that git rejects.
    let branch = if instance_index <= 1 {
        format!("swarm/{persona_slug}")
    } else {
        format!("swarm/{persona_slug}-{instance_index}")
    };

    // Already bootstrapped → no-op.
    if workspace.join(".git").exists() {
        return Ok(Some(BootstrapOutcome {
            worktree_path: workspace.to_path_buf(),
            branch,
            canonical_path: canonical,
        }));
    }

    // Ensure parent of the workspace exists (the dir itself will be
    // created by `git worktree add`, which expects either no dir or an
    // empty one).
    if let Some(parent) = workspace.parent() {
        std::fs::create_dir_all(parent).map_err(|e| GitError::Io {
            context: "mkdir workspace parent".into(),
            source: e,
        })?;
    }

    if workspace.exists() {
        let non_empty = workspace
            .read_dir()
            .map(|mut it| it.next().is_some())
            .unwrap_or(false);
        if non_empty {
            return Err(GitError::InvalidRepoPath(format!(
                "{} exists and is not empty; refusing to overwrite",
                workspace.display()
            )));
        }
        // Empty dir — git worktree add will accept it.
    }

    let canonical_str = canonical
        .to_str()
        .ok_or_else(|| GitError::InvalidRepoPath(canonical.display().to_string()))?;
    let workspace_str = workspace
        .to_str()
        .ok_or_else(|| GitError::InvalidRepoPath(workspace.display().to_string()))?;

    // `worktree add -B` force-creates the branch from HEAD so a deployment
    // that's never been bootstrapped still works. Subsequent retries take
    // the early-return above.
    run_git(
        &[
            "-C",
            canonical_str,
            "worktree",
            "add",
            "-B",
            &branch,
            workspace_str,
        ],
        None,
    )?;

    Ok(Some(BootstrapOutcome {
        worktree_path: workspace.to_path_buf(),
        branch,
        canonical_path: canonical,
    }))
}

/// Best-effort cleanup when a deployment is deleted. Detaches the worktree
/// from its canonical so `git worktree list` stays clean, then removes the
/// directory.
pub fn tear_down_worktree(workspace: &Path) {
    if !workspace.join(".git").exists() {
        return;
    }
    if let Some(s) = workspace.to_str() {
        let _ = run_git(&["worktree", "remove", "--force", s], None);
    }
}
