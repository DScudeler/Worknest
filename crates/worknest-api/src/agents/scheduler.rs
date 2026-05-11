//! Cron-driven tick scheduler. One iteration every 15 seconds:
//!
//! 1. Generate a fresh `lock_token`.
//! 2. `claim_due_for_tick` — atomic per-row UPDATE that takes ownership of
//!    deployments whose `next_tick_at <= now`.
//! 3. For each claimed row, `tokio::spawn(execute_tick(...))`.
//!
//! `execute_tick` records a tick row, runs the `claude` subprocess (with
//! `flock` defence-in-depth), recomputes `next_tick_at`, and refreshes the
//! materialised stats via `bump_after_tick`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, TimeZone, Utc};
use fs2::FileExt;
use uuid::Uuid;

use worknest_core::models::{
    AgentDeployment, AgentDeploymentId, AgentEventKind, AgentStatus, TickOutcome,
};
use worknest_db::{AgentDeploymentRepository, AgentEventRepository, AgentTickRepository};

use super::activation;
use super::tick_executor::{TickExecutor, TickRequest};
use super::AgentsConfig;

/// Subset of `AppState` the scheduler needs.
#[derive(Clone)]
pub struct SchedulerState {
    pub deployment_repo: Arc<AgentDeploymentRepository>,
    pub tick_repo: Arc<AgentTickRepository>,
    pub event_repo: Arc<AgentEventRepository>,
    pub agents_config: Arc<AgentsConfig>,
    pub executor: Arc<dyn TickExecutor>,
}

const TICK_LOOP_INTERVAL_SECS: u64 = 15;
// Must exceed `AgentsConfig::tick_timeout_secs` (default 1800) so a
// still-running tick is never falsely treated as crashed and reclaimed.
const STALE_LOCK_SECS: i64 = 2400; // 40 minutes
const CLAIM_BATCH: usize = 16;
const MAX_TICK_FAILURES: i32 = 3;

/// Long-running task. Spawn this from `main()`.
pub async fn run_loop(state: SchedulerState) {
    let mut interval = tokio::time::interval(Duration::from_secs(TICK_LOOP_INTERVAL_SECS));
    loop {
        interval.tick().await;
        if let Err(e) = run_one_iteration(&state).await {
            tracing::error!("scheduler iteration failed: {:?}", e);
        }
    }
}

pub async fn run_one_iteration(state: &SchedulerState) -> std::io::Result<()> {
    let now = Utc::now();
    let lock_token = Uuid::new_v4().to_string();
    let depl_repo = state.deployment_repo.clone();
    let token_for_claim = lock_token.clone();
    let claimed = match tokio::task::spawn_blocking(move || {
        depl_repo.claim_due_for_tick(now, &token_for_claim, STALE_LOCK_SECS, CLAIM_BATCH)
    })
    .await
    {
        Ok(Ok(rows)) => rows,
        Ok(Err(e)) => {
            tracing::error!("claim_due_for_tick: {:?}", e);
            return Ok(());
        },
        Err(je) => {
            tracing::error!("claim_due_for_tick join: {:?}", je);
            return Ok(());
        },
    };
    for d in claimed {
        let s = state.clone();
        let token = lock_token.clone();
        tokio::spawn(async move { execute_tick(s, d, token).await });
    }
    Ok(())
}

async fn execute_tick(state: SchedulerState, deployment: AgentDeployment, lock_token: String) {
    let id = deployment.id;
    let workspace = match deployment.workspace_path.as_ref() {
        Some(p) => PathBuf::from(p),
        None => {
            tracing::error!("tick: deployment {id} has no workspace_path; skipping");
            release_lock(&state, id, &lock_token).await;
            return;
        },
    };

    // Open the tick row.
    let tick_repo = state.tick_repo.clone();
    let tick = match tokio::task::spawn_blocking(move || tick_repo.start(id))
        .await
        .ok()
        .and_then(|r| r.ok())
    {
        Some(t) => t,
        None => {
            tracing::error!("tick: failed to open tick row for deployment {id}");
            release_lock(&state, id, &lock_token).await;
            return;
        },
    };
    let tick_id = tick.id;

    // Acquire the on-disk advisory lock (defence in depth on top of the SQL claim).
    let lock_path = workspace.join("tick.lock");
    let lock_file = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(e) => {
            finish_tick(
                &state,
                id,
                tick_id,
                &lock_token,
                TickOutcome::Failure,
                None,
                None,
                Some(&format!("could not open lock file: {e}")),
            )
            .await;
            return;
        },
    };
    if let Err(e) = lock_file.try_lock_exclusive() {
        // Another tick already running — record skipped.
        finish_tick(
            &state,
            id,
            tick_id,
            &lock_token,
            TickOutcome::Skipped,
            Some("another tick holds the workspace lock"),
            None,
            Some(&e.to_string()),
        )
        .await;
        return;
    }

    // Run the subprocess.
    let log_path = workspace.join(format!("logs/{tick_id}.log"));
    let timeout = Duration::from_secs(state.agents_config.tick_timeout_secs);
    let exec = state.executor.clone();
    // The workspace dir IS the git worktree (when project has a repo_path)
    // and IS the dir holding CLAUDE.md/.mcp.json/.claude/. Either way,
    // launching claude with cwd = workspace gives it both the rendered
    // config AND the codebase. The persona's tier (snapshot_model) maps to
    // a wire model id passed via `--model` so each persona honours its
    // configured Haiku/Sonnet/Opus pick.
    let model_id = deployment.snapshot_model.map(|m| m.wire_id());
    let req = TickRequest {
        workspace: &workspace,
        log_path: &log_path,
        timeout,
        model: model_id,
    };
    let report = exec.run(req).await;
    let _ = FileExt::unlock(&lock_file);

    match report {
        Ok(r) if r.success => {
            finish_tick(
                &state,
                id,
                tick_id,
                &lock_token,
                TickOutcome::Success,
                Some(&r.summary),
                None,
                None,
            )
            .await;
        },
        Ok(r) => {
            finish_tick(
                &state,
                id,
                tick_id,
                &lock_token,
                TickOutcome::Failure,
                Some(&r.summary),
                None,
                Some(&r.stderr_tail),
            )
            .await;
        },
        Err(e) => {
            finish_tick(
                &state,
                id,
                tick_id,
                &lock_token,
                TickOutcome::Failure,
                None,
                None,
                Some(&format!("subprocess failed: {e}")),
            )
            .await;
        },
    }
}

async fn release_lock(state: &SchedulerState, id: AgentDeploymentId, lock_token: &str) {
    let repo = state.deployment_repo.clone();
    let token = lock_token.to_string();
    let _ = tokio::task::spawn_blocking(move || repo.release_tick_lock(id, &token)).await;
}

#[allow(clippy::too_many_arguments)]
async fn finish_tick(
    state: &SchedulerState,
    deployment_id: AgentDeploymentId,
    tick_id: worknest_core::models::AgentTickId,
    lock_token: &str,
    outcome: TickOutcome,
    summary: Option<&str>,
    touched_ticket: Option<worknest_core::models::TicketId>,
    error_message: Option<&str>,
) {
    // Close the tick row.
    let tick_repo = state.tick_repo.clone();
    let summary_owned = summary.map(|s| s.to_string());
    let error_owned = error_message.map(|s| s.to_string());
    let _ = tokio::task::spawn_blocking(move || {
        tick_repo.finish(
            tick_id,
            outcome,
            summary_owned.as_deref(),
            touched_ticket,
            error_owned.as_deref(),
        )
    })
    .await;

    // Recompute stats and next_tick_at, then bump the deployment row.
    let depl_repo = state.deployment_repo.clone();
    let depl = match tokio::task::spawn_blocking(move || {
        worknest_db::Repository::find_by_id(&*depl_repo, deployment_id)
    })
    .await
    {
        Ok(Ok(Some(d))) => d,
        _ => {
            tracing::error!("finish_tick: deployment {deployment_id} disappeared mid-tick");
            return;
        },
    };

    let start_today = start_of_today_utc();
    let start_week = start_of_week_utc();
    let tick_repo = state.tick_repo.clone();
    let today_stats =
        tokio::task::spawn_blocking(move || tick_repo.aggregate_stats(deployment_id, start_today))
            .await
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or_default();
    let tick_repo = state.tick_repo.clone();
    let week_stats =
        tokio::task::spawn_blocking(move || tick_repo.aggregate_stats(deployment_id, start_week))
            .await
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or_default();

    let success_rate = if week_stats.runs > 0 {
        week_stats.successes as f32 / week_stats.runs as f32
    } else {
        0.0
    };

    // Increment error_count on failure; reset on success.
    let new_error_count = match outcome {
        TickOutcome::Success => 0,
        TickOutcome::Skipped => depl.error_count,
        TickOutcome::Failure => depl.error_count + 1,
    };
    let surfaced_error = if outcome == TickOutcome::Failure {
        error_message.map(|s| s.to_string())
    } else {
        None
    };

    let next_tick = activation::next_fire_after(&depl.cron_expression, Utc::now()).ok();
    let depl_repo = state.deployment_repo.clone();
    let token = lock_token.to_string();
    let surfaced_error_for_bump = surfaced_error.clone();
    let _ = tokio::task::spawn_blocking(move || {
        depl_repo.bump_after_tick(
            deployment_id,
            &token,
            next_tick,
            Some(Utc::now()),
            today_stats.runs,
            week_stats.touched_tickets,
            success_rate,
            new_error_count,
            surfaced_error_for_bump.as_deref(),
        )
    })
    .await;

    // If we crossed the failure threshold, transition Running → Error.
    if new_error_count >= MAX_TICK_FAILURES && outcome == TickOutcome::Failure {
        let depl_repo = state.deployment_repo.clone();
        let _ = tokio::task::spawn_blocking(move || {
            depl_repo.transition_status(deployment_id, &[AgentStatus::Running], AgentStatus::Error)
        })
        .await;
        let event_repo = state.event_repo.clone();
        let msg = format!(
            "deployment paused after {MAX_TICK_FAILURES} consecutive failures: {}",
            surfaced_error.as_deref().unwrap_or("unknown")
        );
        let _ = tokio::task::spawn_blocking(move || {
            event_repo.record(
                deployment_id,
                AgentEventKind::TickFailedThreshold,
                &msg,
                serde_json::Value::Null,
            )
        })
        .await;
    }
}

fn start_of_today_utc() -> chrono::DateTime<Utc> {
    let now = Utc::now();
    Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap()
}

fn start_of_week_utc() -> chrono::DateTime<Utc> {
    let now = Utc::now();
    let weekday_from_monday = now.weekday().num_days_from_monday() as i64;
    let monday = now - chrono::Duration::days(weekday_from_monday);
    Utc.with_ymd_and_hms(monday.year(), monday.month(), monday.day(), 0, 0, 0)
        .unwrap()
}

#[cfg(test)]
mod tests {
    //! End-to-end smoke tests for the scheduler. The real `claude` subprocess
    //! is replaced by a mock TickExecutor so the test verifies *our* wiring
    //! (claim → spawn → record tick → bump stats → release lock → fail-
    //! threshold transition) without depending on the external CLI.

    use super::*;
    use crate::agents::tick_executor::{TickExecutor, TickReport};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Duration;
    use worknest_core::models::{
        AgentDeployment, AgentDeploymentId, AgentStatus, Persona, PersonaId, Project, User,
    };
    use worknest_db::{
        connection::init_memory_pool, run_migrations, AgentDeploymentRepository,
        AgentEventRepository, AgentTickRepository, PersonaRepository, ProjectRepository,
        Repository, UserRepository,
    };

    struct MockExec {
        succeed: bool,
    }
    #[async_trait]
    impl TickExecutor for MockExec {
        async fn run(&self, _req: TickRequest<'_>) -> std::io::Result<TickReport> {
            Ok(TickReport {
                success: self.succeed,
                summary: if self.succeed {
                    "ok".into()
                } else {
                    "boom".into()
                },
                stderr_tail: if self.succeed {
                    String::new()
                } else {
                    "fake failure".into()
                },
            })
        }
    }

    struct Fixture {
        state: SchedulerState,
        deployment: AgentDeployment,
        deployment_repo: Arc<AgentDeploymentRepository>,
        tick_repo: Arc<AgentTickRepository>,
        event_repo: Arc<AgentEventRepository>,
        _tmp: tempfile::TempDir,
    }

    fn build_fixture(succeed: bool) -> Fixture {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);

        let user_repo = Arc::new(UserRepository::new(Arc::clone(&pool)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&pool)));
        let persona_repo = Arc::new(PersonaRepository::new(Arc::clone(&pool)));
        let deployment_repo = Arc::new(AgentDeploymentRepository::new(Arc::clone(&pool)));
        let tick_repo = Arc::new(AgentTickRepository::new(Arc::clone(&pool)));
        let event_repo = Arc::new(AgentEventRepository::new(Arc::clone(&pool)));

        // User → Project (V4 backfill auto-adds owner as project_member, but
        // the test bypasses that path; that's fine since the scheduler's
        // claim doesn't query membership).
        let owner = User::new("alice".into(), "alice@x.test".into());
        user_repo.create_with_password(&owner, "x").unwrap();
        let project = Project::new("Demo".into(), owner.id);
        project_repo.create(&project).unwrap();

        let persona: Persona = persona_repo.find_by_slug("triage").unwrap().unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(workspace.join("logs")).unwrap();
        std::fs::write(workspace.join("tick.lock"), b"").unwrap();

        let deployment = AgentDeployment {
            id: AgentDeploymentId::new(),
            project_id: project.id,
            persona_id: persona.id as PersonaId,
            agent_user_id: None,
            snapshot_name: None,
            snapshot_role: None,
            snapshot_tone: None,
            snapshot_expertise: vec![],
            snapshot_instructions: None,
            snapshot_capabilities: vec![],
            snapshot_model: None,
            snapshot_taken_at: None,
            workspace_path: Some(workspace.display().to_string()),
            // 5-field cron evaluating every minute keeps next_tick_at always
            // computable with no special-casing.
            cron_expression: "* * * * *".into(),
            // 1s in the past → claim picks it up immediately.
            next_tick_at: Some(Utc::now() - chrono::Duration::seconds(1)),
            tick_locked_at: None,
            tick_lock_token: None,
            status: AgentStatus::Running,
            last_error_step: None,
            error_message: None,
            error_count: 0,
            current_ticket_id: None,
            runs_today: 0,
            touched_this_week: 0,
            success_rate: 0.0,
            last_activity_at: None,
            instance_index: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        deployment_repo.create(&deployment).unwrap();

        let agents_config = Arc::new(super::super::AgentsConfig {
            agents_dir: tmp.path().to_path_buf(),
            mcp_dir: None,
            claude_bin: "claude".into(),
            worknest_url: "http://localhost:0".into(),
            tick_timeout_secs: 5,
        });
        let executor: Arc<dyn TickExecutor> = Arc::new(MockExec { succeed });
        let state = SchedulerState {
            deployment_repo: deployment_repo.clone(),
            tick_repo: tick_repo.clone(),
            event_repo: event_repo.clone(),
            agents_config,
            executor,
        };
        Fixture {
            state,
            deployment,
            deployment_repo,
            tick_repo,
            event_repo,
            _tmp: tmp,
        }
    }

    /// Drive one scheduler iteration and wait for spawned children to finish.
    async fn fire_once(state: &SchedulerState) {
        run_one_iteration(state).await.unwrap();
        // The iteration spawns tokio::spawn children we don't have handles to;
        // give them time to land their DB writes. In-memory SQLite is fast,
        // so 200ms is plenty.
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tick_success_records_row_and_releases_lock() {
        let f = build_fixture(true);
        fire_once(&f.state).await;

        // A single tick row should exist with outcome=Success.
        let ticks = f
            .tick_repo
            .list_for_deployment(f.deployment.id, 10)
            .unwrap();
        assert_eq!(ticks.len(), 1, "expected exactly one tick row");
        assert_eq!(
            ticks[0].outcome,
            Some(worknest_core::models::TickOutcome::Success)
        );
        assert!(ticks[0].finished_at.is_some());

        // Deployment row: lock cleared, stats refreshed, error_count zeroed,
        // status still Running, next_tick_at advanced.
        let after = f
            .deployment_repo
            .find_by_id(f.deployment.id)
            .unwrap()
            .unwrap();
        assert_eq!(after.status, AgentStatus::Running);
        assert!(after.tick_locked_at.is_none(), "lock should be cleared");
        assert!(after.tick_lock_token.is_none());
        assert_eq!(after.error_count, 0);
        assert!(after.runs_today >= 1);
        assert!(after.last_activity_at.is_some());
        assert!(
            after.next_tick_at.unwrap() > Utc::now(),
            "next_tick_at should be in the future"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn three_consecutive_failures_transition_to_error() {
        let f = build_fixture(false);
        // Fire 3 times. Between iterations, reset next_tick_at to now so the
        // next claim re-claims the row (the bump_after_tick already pushed
        // it forward by ~1 minute via the cron schedule).
        for _ in 0..3 {
            fire_once(&f.state).await;
            f.deployment_repo
                .set_next_tick_at(f.deployment.id, Utc::now() - chrono::Duration::seconds(1))
                .unwrap();
        }

        let after = f
            .deployment_repo
            .find_by_id(f.deployment.id)
            .unwrap()
            .unwrap();
        assert_eq!(
            after.status,
            AgentStatus::Error,
            "deployment should be paused after threshold failures"
        );
        assert_eq!(after.error_count, 3);
        let ticks = f
            .tick_repo
            .list_for_deployment(f.deployment.id, 10)
            .unwrap();
        assert_eq!(ticks.len(), 3);
        assert!(ticks
            .iter()
            .all(|t| t.outcome == Some(worknest_core::models::TickOutcome::Failure)));

        // The lifecycle event log should contain a TickFailedThreshold marker.
        let events = f
            .event_repo
            .list_for_deployment(f.deployment.id, 50)
            .unwrap();
        assert!(events
            .iter()
            .any(|e| e.kind == worknest_core::models::AgentEventKind::TickFailedThreshold));
    }
}
