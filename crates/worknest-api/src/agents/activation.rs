//! Activation pipeline. Drives a deployment from `Pending` to `Running`
//! through six idempotent steps. Each step is its own short transaction; the
//! `transition_status` guards make every step safe to re-run, so a stale
//! dispatcher cannot produce duplicate work.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use rand::Rng;

use worknest_auth::AuthService;
use worknest_core::models::{
    ActivationStep, AgentDeployment, AgentDeploymentId, AgentEventKind, AgentStatus, Persona,
    ProjectId, User, UserId,
};
use worknest_db::{
    AgentDeploymentRepository, AgentEventRepository, PersonaRepository, PersonaSnapshot,
    ProjectRepository, UserRepository,
};

use super::AgentsConfig;

/// Subset of `AppState` the activation pipeline needs. Decoupled from the
/// monolithic `AppState` struct so unit tests can build a smaller harness.
#[derive(Clone)]
pub struct ActivationState {
    pub auth_service: Arc<AuthService>,
    pub user_repo: Arc<UserRepository>,
    pub project_repo: Arc<ProjectRepository>,
    pub persona_repo: Arc<PersonaRepository>,
    pub deployment_repo: Arc<AgentDeploymentRepository>,
    pub event_repo: Arc<AgentEventRepository>,
    pub agents_config: Arc<AgentsConfig>,
}

#[derive(Debug, thiserror::Error)]
pub enum ActivationError {
    #[error("deployment {0} not found")]
    DeploymentNotFound(AgentDeploymentId),
    #[error("persona for deployment {0} not found")]
    PersonaNotFound(AgentDeploymentId),
    #[error("identity user not yet registered for deployment {0}")]
    NoAgentUser(AgentDeploymentId),
    #[error("workspace error: {0}")]
    Workspace(#[from] super::workspace::WorkspaceError),
    #[error("git error: {0}")]
    Git(#[from] super::git::GitError),
    #[error("invalid cron expression {expr:?}: {source}")]
    Cron {
        expr: String,
        #[source]
        source: cron::error::Error,
    },
    #[error("cron schedule {0:?} has no upcoming tick")]
    CronNoUpcoming(String),
    #[error("DB: {0}")]
    Db(#[from] worknest_db::DbError),
    #[error("auth: {0}")]
    Auth(String),
}

/// Run the pipeline starting at `start_step`. Steps before `start_step` are
/// presumed already complete (this is how Retry resumes from `last_error_step`).
pub async fn run_pipeline(
    state: ActivationState,
    deployment_id: AgentDeploymentId,
    start_step: ActivationStep,
) {
    if let Err(e) = run_pipeline_inner(state.clone(), deployment_id, start_step).await {
        tracing::error!(
            "activation pipeline failed for deployment {}: {:?}",
            deployment_id,
            e
        );
    }
}

async fn run_pipeline_inner(
    state: ActivationState,
    deployment_id: AgentDeploymentId,
    start_step: ActivationStep,
) -> Result<(), ActivationError> {
    let mut step = start_step;
    loop {
        let result = run_step(&state, deployment_id, step).await;
        match result {
            Ok(Some(next)) => step = next,
            Ok(None) => {
                tracing::info!("activation complete for deployment {}", deployment_id);
                return Ok(());
            },
            Err(e) => {
                let msg = e.to_string();
                tracing::warn!(
                    "activation step {step:?} failed for deployment {deployment_id}: {msg}"
                );
                let _ = state
                    .deployment_repo
                    .record_activation_failure(deployment_id, step, &msg);
                let _ = state.event_repo.record(
                    deployment_id,
                    AgentEventKind::ActivationFailed,
                    &format!("{step}: {msg}"),
                    serde_json::json!({"step": step.to_string(), "message": msg}),
                );
                return Err(e);
            },
        }
    }
}

async fn run_step(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
    step: ActivationStep,
) -> Result<Option<ActivationStep>, ActivationError> {
    match step {
        ActivationStep::RegisterIdentity => {
            register_identity(state, deployment_id).await?;
            Ok(Some(ActivationStep::GrantMembership))
        },
        ActivationStep::GrantMembership => {
            grant_membership(state, deployment_id).await?;
            Ok(Some(ActivationStep::SnapshotPersona))
        },
        ActivationStep::SnapshotPersona => {
            snapshot_persona(state, deployment_id).await?;
            Ok(Some(ActivationStep::BootstrapWorktree))
        },
        ActivationStep::BootstrapWorktree => {
            bootstrap_worktree(state, deployment_id).await?;
            Ok(Some(ActivationStep::ProvisionWorkspace))
        },
        ActivationStep::ProvisionWorkspace => {
            provision_workspace(state, deployment_id).await?;
            Ok(Some(ActivationStep::ScheduleNextTick))
        },
        ActivationStep::ScheduleNextTick => {
            schedule_next_tick(state, deployment_id).await?;
            Ok(Some(ActivationStep::MarkRunning))
        },
        ActivationStep::MarkRunning => {
            mark_running(state, deployment_id).await?;
            Ok(None)
        },
    }
}

fn rand_password() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| rng.random_range(b'!'..=b'~') as char)
        .collect()
}

async fn register_identity(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let persona = load_persona(state, deployment_id, depl.persona_id).await?;

    // Determine or create the agent identity user (deterministic email keyed
    // on persona slug). `find_agent_by_email` returns ConstraintViolation if a
    // human happens to own the address, which we surface as an error.
    let email = format!("agent-{}@worknest.local", persona.slug);
    let user_repo = state.user_repo.clone();
    let email_for_lookup = email.clone();
    let existing =
        tokio::task::spawn_blocking(move || user_repo.find_agent_by_email(&email_for_lookup))
            .await
            .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let user = if let Some(u) = existing {
        u
    } else {
        let username = format!("agent-{}", persona.slug);
        let mut user = User::new(username.clone(), email.clone());
        user.is_agent = true;
        user.full_name = Some(persona.name.clone());
        let pwd = rand_password();
        // Agents never log in with this password — the identity authenticates
        // via the freshly minted JWT. Bypass the policy validator so we don't
        // reject statistically valid (but policy-non-compliant) random strings.
        let pwd_hash = worknest_auth::password::hash_password_unchecked(&pwd)
            .map_err(|e| ActivationError::Auth(e.to_string()))?;
        let user_repo = state.user_repo.clone();
        let user_for_create = user.clone();
        tokio::task::spawn_blocking(move || {
            user_repo.create_with_password(&user_for_create, &pwd_hash)
        })
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??
    };

    // Record the agent_user_id on the deployment.
    let depl_repo = state.deployment_repo.clone();
    let user_id = user.id;
    tokio::task::spawn_blocking(move || depl_repo.set_agent_user(deployment_id, user_id))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    // Drive the FSM forward.
    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Pending, AgentStatus::Registering],
            AgentStatus::Granting,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    record_event(
        state,
        deployment_id,
        AgentEventKind::IdentityRegistered,
        &format!("agent identity user: {}", user.username),
        serde_json::json!({"user_id": user.id.to_string(), "email": email}),
    )
    .await;
    Ok(())
}

async fn grant_membership(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let agent_user_id = depl
        .agent_user_id
        .ok_or(ActivationError::NoAgentUser(deployment_id))?;

    let proj_repo = state.project_repo.clone();
    let project_id = depl.project_id;
    tokio::task::spawn_blocking(move || proj_repo.add_member(project_id, agent_user_id, "agent"))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Granting],
            AgentStatus::Snapshotting,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    record_event(
        state,
        deployment_id,
        AgentEventKind::MembershipGranted,
        &format!("agent {agent_user_id} added to project {project_id} as 'agent'"),
        serde_json::Value::Null,
    )
    .await;
    Ok(())
}

async fn snapshot_persona(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let persona = load_persona(state, deployment_id, depl.persona_id).await?;
    let snap = PersonaSnapshot {
        name: persona.name.clone(),
        role: persona.role.clone(),
        tone: persona.tone.clone(),
        expertise: persona.expertise.clone(),
        instructions: persona.instructions.clone(),
        capabilities: persona.capabilities.clone(),
        model: persona.model,
    };
    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || depl_repo.apply_snapshot(deployment_id, &snap))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Snapshotting],
            AgentStatus::Bootstrapping,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    record_event(
        state,
        deployment_id,
        AgentEventKind::PersonaSnapshotted,
        &format!("snapshot captured from persona '{}'", persona.slug),
        serde_json::json!({"persona_slug": persona.slug}),
    )
    .await;
    Ok(())
}

async fn provision_workspace(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let persona = load_persona(state, deployment_id, depl.persona_id).await?;
    let agent_user_id = depl
        .agent_user_id
        .ok_or(ActivationError::NoAgentUser(deployment_id))?;

    // Mint a fresh JWT for the agent identity user (long-lived: existing
    // generate_token uses the configured token TTL).
    let username = format!("agent-{}", persona.slug);
    let token = state
        .auth_service
        .generate_token_for_user(agent_user_id, username)
        .map_err(|e| ActivationError::Auth(e.to_string()))?;

    // Build the project-shared personas map (slug → agent user_id) so the
    // rendered .mcp.json's WORKNEST_PERSONAS env var points at a file that
    // already includes every sibling agent in this project. This is what
    // wn_handoff(to_persona="frontend", …) consults to resolve the target
    // user_id without round-tripping through Worknest.
    let (personas_map, peer_personas) = build_personas_map(state, depl.project_id).await?;

    // Render the workspace.
    let cfg = state.agents_config.clone();
    let agents_dir = cfg.agents_dir.clone();
    let mcp_dir = cfg.mcp_dir.clone();
    let url = cfg.worknest_url.clone();
    let depl_for_provision = depl.clone();
    let persona_for_provision = persona.clone();
    let jwt = token.token.clone();
    let map_for_provision = personas_map.clone();
    let peers_for_provision = peer_personas.clone();
    let workspace_path = tokio::task::spawn_blocking(move || {
        super::workspace::provision(
            &depl_for_provision,
            &persona_for_provision,
            &agents_dir,
            mcp_dir.as_deref(),
            &url,
            &jwt,
            &super::workspace::PersonaRoster {
                by_slug: &map_for_provision,
                all: &peers_for_provision,
            },
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    // Persist the workspace path on the deployment row.
    let depl_repo = state.deployment_repo.clone();
    let path_str = workspace_path.display().to_string();
    tokio::task::spawn_blocking(move || depl_repo.set_workspace_path(deployment_id, &path_str))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Provisioning],
            AgentStatus::Scheduling,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    record_event(
        state,
        deployment_id,
        AgentEventKind::WorkspaceProvisioned,
        &format!("workspace at {}", workspace_path.display()),
        serde_json::Value::Null,
    )
    .await;
    Ok(())
}

async fn bootstrap_worktree(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let persona = load_persona(state, deployment_id, depl.persona_id).await?;

    // Derive the workspace path from agents_dir + deployment_id. This step
    // runs before ProvisionWorkspace, which is the one that persists the
    // path on the deployment row, so we can't read it back here.
    let agents_dir = state.agents_config.agents_dir.clone();
    let workspace_pb = agents_dir.join(deployment_id.to_string());

    // Resolve the project's repo_path (None ⇒ skip).
    let project_repo = state.project_repo.clone();
    let project_id = depl.project_id;
    let project = tokio::task::spawn_blocking(move || {
        worknest_db::Repository::find_by_id(&*project_repo, project_id)
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??
    .ok_or_else(|| ActivationError::Auth("project missing".into()))?;
    let repo_path = project.repo_path.clone();

    let project_id_str = depl.project_id.to_string();
    let persona_slug = persona.slug.clone();
    let instance_index = depl.instance_index;
    let workspace_pb_for_bootstrap = workspace_pb.clone();
    let agents_dir_for_bootstrap = agents_dir.clone();

    let outcome = tokio::task::spawn_blocking(move || {
        super::git::bootstrap(
            repo_path.as_deref(),
            &project_id_str,
            &persona_slug,
            instance_index,
            &workspace_pb_for_bootstrap,
            &agents_dir_for_bootstrap,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))?
    .map_err(ActivationError::Git)?;

    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Bootstrapping],
            AgentStatus::Provisioning,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let (msg, payload) = match outcome {
        Some(o) => (
            format!("worktree at {} on {}", o.worktree_path.display(), o.branch),
            serde_json::json!({
                "worktree_path": o.worktree_path.display().to_string(),
                "branch": o.branch,
                "canonical_path": o.canonical_path.display().to_string(),
            }),
        ),
        None => (
            "no repo_path on project; worktree skipped".to_string(),
            serde_json::json!({"skipped": true}),
        ),
    };
    record_event(
        state,
        deployment_id,
        AgentEventKind::WorktreeBootstrapped,
        &msg,
        payload,
    )
    .await;
    Ok(())
}

async fn schedule_next_tick(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    let depl = load_deployment(state, deployment_id).await?;
    let next = next_fire_after(&depl.cron_expression, Utc::now())?;
    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || depl_repo.set_next_tick_at(deployment_id, next))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || {
        depl_repo.transition_status(
            deployment_id,
            &[AgentStatus::Scheduling],
            AgentStatus::Running,
        )
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;

    record_event(
        state,
        deployment_id,
        AgentEventKind::TickScheduled,
        &format!("next tick at {}", next.to_rfc3339()),
        serde_json::json!({"next_tick_at": next.to_rfc3339(), "cron": depl.cron_expression}),
    )
    .await;
    Ok(())
}

async fn mark_running(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
) -> Result<(), ActivationError> {
    // Clear any leftover error fields and bump status to Running. The previous
    // step already set Running, so this is mostly a no-op + an event marker
    // for the audit log + the error_field reset.
    let depl_repo = state.deployment_repo.clone();
    tokio::task::spawn_blocking(move || depl_repo.clear_error_fields(deployment_id))
        .await
        .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;
    record_event(
        state,
        deployment_id,
        AgentEventKind::MarkedRunning,
        "deployment is running",
        serde_json::Value::Null,
    )
    .await;
    Ok(())
}

/// Compute the next firing instant for a 5-field cron expression.
pub fn next_fire_after(
    cron_expr: &str,
    after: chrono::DateTime<Utc>,
) -> Result<chrono::DateTime<Utc>, ActivationError> {
    // `cron::Schedule::from_str` expects 7 fields (sec min hour dom mon dow year);
    // rewrite a 5-field input to that shape with seconds=0 and year=*.
    let prepared = if cron_expr.split_whitespace().count() == 5 {
        format!("0 {cron_expr} *")
    } else {
        cron_expr.to_string()
    };
    let schedule = cron::Schedule::from_str(&prepared).map_err(|source| ActivationError::Cron {
        expr: cron_expr.to_string(),
        source,
    })?;
    schedule
        .after(&after)
        .next()
        .ok_or_else(|| ActivationError::CronNoUpcoming(cron_expr.to_string()))
}

async fn load_deployment(
    state: &ActivationState,
    id: AgentDeploymentId,
) -> Result<AgentDeployment, ActivationError> {
    let repo = state.deployment_repo.clone();
    let row: Option<AgentDeployment> =
        tokio::task::spawn_blocking(move || worknest_db::Repository::find_by_id(&*repo, id))
            .await
            .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;
    row.ok_or(ActivationError::DeploymentNotFound(id))
}

async fn load_persona(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
    persona_id: worknest_core::models::PersonaId,
) -> Result<Persona, ActivationError> {
    let repo = state.persona_repo.clone();
    let row: Option<Persona> = tokio::task::spawn_blocking(move || {
        worknest_db::Repository::find_by_id(&*repo, persona_id)
    })
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;
    row.ok_or(ActivationError::PersonaNotFound(deployment_id))
}

async fn record_event(
    state: &ActivationState,
    deployment_id: AgentDeploymentId,
    kind: AgentEventKind,
    message: &str,
    payload: serde_json::Value,
) {
    let repo = state.event_repo.clone();
    let msg = message.to_string();
    let _ =
        tokio::task::spawn_blocking(move || repo.record(deployment_id, kind, &msg, payload)).await;
}

/// Recovery sweep: pick up deployments stuck in non-terminal activation
/// states and resume them. Called once at boot.
pub async fn recover_stuck(state: ActivationState) {
    let repo = state.deployment_repo.clone();
    let stuck = match tokio::task::spawn_blocking(move || repo.find_stuck_in_activation(30)).await {
        Ok(Ok(rows)) => rows,
        Ok(Err(e)) => {
            tracing::error!("recover_stuck: db error: {:?}", e);
            return;
        },
        Err(je) => {
            tracing::error!("recover_stuck: task join error: {:?}", je);
            return;
        },
    };
    for d in stuck {
        let step = match d.status {
            AgentStatus::Pending | AgentStatus::Registering => ActivationStep::RegisterIdentity,
            AgentStatus::Granting => ActivationStep::GrantMembership,
            AgentStatus::Snapshotting => ActivationStep::SnapshotPersona,
            AgentStatus::Provisioning => ActivationStep::ProvisionWorkspace,
            AgentStatus::Bootstrapping => ActivationStep::BootstrapWorktree,
            AgentStatus::Scheduling => ActivationStep::ScheduleNextTick,
            _ => continue,
        };
        tracing::info!("recover_stuck: resuming deployment {} at {step:?}", d.id);
        let s = state.clone();
        tokio::spawn(async move { run_pipeline(s, d.id, step).await });
    }
}

// Used by other modules that need the imports — silence the
// "unused-import" warning when the build is a no-op for some features.
#[allow(dead_code)]
fn _imports_used(_: &UserId) {}

/// Build the project's persona roster: the slug→user_id map (consumed by
/// the MCP server's `wn_handoff` resolver) plus the full `Persona` records
/// for every active deployment in the project. Run inside
/// `provision_workspace` AFTER step 1 (RegisterIdentity) so the current
/// deployment's `agent_user_id` is already non-NULL and lands in the map.
///
/// The full `Persona` list is what we render into each agent's `CLAUDE.md`
/// as the "Peers in this project" section so the LLM can pick a real
/// teammate slug when handing off or decomposing — without it, agents
/// only see the hardcoded `<frontend|backend|...>` example string and
/// route everything to those two.
async fn build_personas_map(
    state: &ActivationState,
    project_id: ProjectId,
) -> Result<(HashMap<String, String>, Vec<Persona>), ActivationError> {
    let depl_repo = state.deployment_repo.clone();
    let persona_repo = state.persona_repo.clone();
    let result = tokio::task::spawn_blocking(
        move || -> Result<(HashMap<String, String>, Vec<Persona>), worknest_db::DbError> {
            let mut map = HashMap::new();
            let mut personas = Vec::new();
            for d in depl_repo.list_by_project(project_id)? {
                let Some(uid) = d.agent_user_id else { continue };
                if let Some(p) = worknest_db::Repository::find_by_id(&*persona_repo, d.persona_id)?
                {
                    map.insert(p.slug.clone(), uid.to_string());
                    personas.push(p);
                }
            }
            Ok((map, personas))
        },
    )
    .await
    .map_err(|je| ActivationError::Auth(format!("blocking task: {je}")))??;
    Ok(result)
}
