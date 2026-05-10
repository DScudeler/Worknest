//! Repository for [`AgentDeployment`](worknest_core::models::AgentDeployment).
//!
//! Owns the FSM transitions and the SQL `flock`-style claim that the scheduler
//! uses to take a deployment off the run queue without races. All status moves
//! flow through [`Self::transition_status`] which guarantees that a stale
//! actor with the wrong `from`-state observes a 0-row update and bails out.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;

use crate::repositories::{parse_datetime, parse_uuid};
use crate::repository::Repository;
use crate::{DbError, DbPool, Result};
use worknest_core::models::{
    ActivationStep, AgentDeployment, AgentDeploymentId, AgentModel, AgentStatus, Capability,
    PersonaId, ProjectId, TicketId, UserId,
};

const COLUMNS: &str = "id, project_id, persona_id, agent_user_id, \
                       snapshot_name, snapshot_role, snapshot_tone, snapshot_expertise_json, \
                       snapshot_instructions, snapshot_capabilities_json, snapshot_model, snapshot_taken_at, \
                       workspace_path, cron_expression, next_tick_at, tick_locked_at, tick_lock_token, \
                       status, last_error_step, error_message, error_count, current_ticket_id, \
                       runs_today, touched_this_week, success_rate, last_activity_at, \
                       instance_index, created_at, updated_at";

pub struct AgentDeploymentRepository {
    pool: Arc<DbPool>,
}

fn caps_to_json(caps: &[Capability]) -> String {
    let strs: Vec<String> = caps.iter().map(|c| c.to_string()).collect();
    serde_json::to_string(&strs).unwrap_or_else(|_| "[]".into())
}

fn caps_from_json(s: &str) -> Vec<Capability> {
    let strs: Vec<String> = serde_json::from_str(s).unwrap_or_default();
    strs.into_iter()
        .filter_map(|n| match n.as_str() {
            "Comment" => Some(Capability::Comment),
            "Label" => Some(Capability::Label),
            "Assign" => Some(Capability::Assign),
            "SetPriority" => Some(Capability::SetPriority),
            "SetStatus" => Some(Capability::SetStatus),
            "Attach" => Some(Capability::Attach),
            "CreateTicket" => Some(Capability::CreateTicket),
            "Close" => Some(Capability::Close),
            _ => None,
        })
        .collect()
}

fn exp_to_json(items: &[String]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".into())
}

fn exp_from_json(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

fn model_to_str(m: &AgentModel) -> &'static str {
    match m {
        AgentModel::Haiku => "Haiku",
        AgentModel::Sonnet => "Sonnet",
        AgentModel::Opus => "Opus",
    }
}

fn model_from_str(s: &str) -> AgentModel {
    match s {
        "Haiku" => AgentModel::Haiku,
        "Opus" => AgentModel::Opus,
        _ => AgentModel::Sonnet,
    }
}

fn parse_dt_opt(row: &Row, idx: usize) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let s: Option<String> = row.get(idx)?;
    match s {
        Some(s) => Ok(Some(parse_datetime(&s)?)),
        None => Ok(None),
    }
}

fn row_to_deployment(row: &Row) -> rusqlite::Result<AgentDeployment> {
    let id_str: String = row.get(0)?;
    let project_id_str: String = row.get(1)?;
    let persona_id_str: String = row.get(2)?;
    let agent_user_id: Option<String> = row.get(3)?;

    let snapshot_name: Option<String> = row.get(4)?;
    let snapshot_role: Option<String> = row.get(5)?;
    let snapshot_tone: Option<String> = row.get(6)?;
    let snapshot_expertise_json: String = row.get(7)?;
    let snapshot_instructions: Option<String> = row.get(8)?;
    let snapshot_capabilities_json: String = row.get(9)?;
    let snapshot_model: Option<String> = row.get(10)?;
    let snapshot_taken_at = parse_dt_opt(row, 11)?;

    let workspace_path: Option<String> = row.get(12)?;
    let cron_expression: String = row.get(13)?;
    let next_tick_at = parse_dt_opt(row, 14)?;
    let tick_locked_at = parse_dt_opt(row, 15)?;
    let tick_lock_token: Option<String> = row.get(16)?;

    let status_str: String = row.get(17)?;
    let last_error_step_str: Option<String> = row.get(18)?;
    let error_message: Option<String> = row.get(19)?;
    let error_count: i32 = row.get(20)?;
    let current_ticket_id: Option<String> = row.get(21)?;

    let runs_today: i32 = row.get(22)?;
    let touched_this_week: i32 = row.get(23)?;
    let success_rate: f64 = row.get(24)?;
    let last_activity_at = parse_dt_opt(row, 25)?;

    let instance_index: i32 = row.get(26)?;
    let created_at_str: String = row.get(27)?;
    let updated_at_str: String = row.get(28)?;

    let map_err = |e: worknest_core::CoreError| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    };

    Ok(AgentDeployment {
        id: AgentDeploymentId::from_uuid(parse_uuid(&id_str)?),
        project_id: ProjectId::from_uuid(parse_uuid(&project_id_str)?),
        persona_id: PersonaId::from_uuid(parse_uuid(&persona_id_str)?),
        agent_user_id: match agent_user_id {
            Some(s) => Some(UserId::from_uuid(parse_uuid(&s)?)),
            None => None,
        },
        snapshot_name,
        snapshot_role,
        snapshot_tone,
        snapshot_expertise: exp_from_json(&snapshot_expertise_json),
        snapshot_instructions,
        snapshot_capabilities: caps_from_json(&snapshot_capabilities_json),
        snapshot_model: snapshot_model.as_deref().map(model_from_str),
        snapshot_taken_at,
        workspace_path,
        cron_expression,
        next_tick_at,
        tick_locked_at,
        tick_lock_token,
        status: AgentStatus::parse(&status_str).map_err(map_err)?,
        last_error_step: match last_error_step_str {
            Some(s) => Some(ActivationStep::parse(&s).map_err(map_err)?),
            None => None,
        },
        error_message,
        error_count,
        current_ticket_id: match current_ticket_id {
            Some(s) => Some(TicketId::from_uuid(parse_uuid(&s)?)),
            None => None,
        },
        runs_today,
        touched_this_week,
        success_rate: success_rate as f32,
        last_activity_at,
        instance_index,
        created_at: parse_datetime(&created_at_str)?,
        updated_at: parse_datetime(&updated_at_str)?,
    })
}

/// Snapshot bundle written to a deployment by the activation pipeline's
/// `SnapshotPersona` step.
#[derive(Debug, Clone)]
pub struct PersonaSnapshot {
    pub name: String,
    pub role: String,
    pub tone: String,
    pub expertise: Vec<String>,
    pub instructions: String,
    pub capabilities: Vec<Capability>,
    pub model: AgentModel,
}

impl AgentDeploymentRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!(
            "SELECT {COLUMNS} FROM agent_deployments WHERE project_id = ?1 ORDER BY created_at DESC"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(params![project_id.to_string()], row_to_deployment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    /// All deployments of a given persona within a project. Multiple rows
    /// are expected when an operator scales a persona ("3× Backend Dev").
    /// Ordered by `instance_index` ascending.
    pub fn list_by_project_and_persona(
        &self,
        project_id: ProjectId,
        persona_id: PersonaId,
    ) -> Result<Vec<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!(
            "SELECT {COLUMNS} FROM agent_deployments \
             WHERE project_id = ?1 AND persona_id = ?2 \
             ORDER BY instance_index ASC"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(
                params![project_id.to_string(), persona_id.to_string()],
                row_to_deployment,
            )
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    /// Guarded status transition. Returns `true` iff one row was changed.
    /// The caller's CAS guarantee: a stale dispatcher with a different `from`
    /// observes `false` and exits without redoing the work.
    pub fn transition_status(
        &self,
        id: AgentDeploymentId,
        from: &[AgentStatus],
        to: AgentStatus,
    ) -> Result<bool> {
        if from.is_empty() {
            return Err(DbError::Query(
                "transition_status requires at least one allowed source state".into(),
            ));
        }
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let placeholders = std::iter::repeat_n("?", from.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "UPDATE agent_deployments SET status = ?, updated_at = ? \
             WHERE id = ? AND status IN ({placeholders})"
        );
        let now = Utc::now().to_rfc3339();
        let mut p: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(to.to_string()),
            Box::new(now),
            Box::new(id.to_string()),
        ];
        for s in from {
            p.push(Box::new(s.to_string()));
        }
        let refs: Vec<&dyn rusqlite::ToSql> = p.iter().map(|b| b.as_ref()).collect();
        let rows = conn
            .execute(&sql, rusqlite::params_from_iter(refs))
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows == 1)
    }

    pub fn record_activation_failure(
        &self,
        id: AgentDeploymentId,
        step: ActivationStep,
        message: &str,
    ) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET status = 'Error', last_error_step = ?1, \
             error_message = ?2, error_count = error_count + 1, updated_at = ?3 \
             WHERE id = ?4",
            params![
                step.to_string(),
                message,
                Utc::now().to_rfc3339(),
                id.to_string()
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn set_agent_user(&self, id: AgentDeploymentId, user_id: UserId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET agent_user_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![user_id.to_string(), Utc::now().to_rfc3339(), id.to_string()],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn apply_snapshot(&self, id: AgentDeploymentId, snap: &PersonaSnapshot) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE agent_deployments SET snapshot_name = ?1, snapshot_role = ?2, \
             snapshot_tone = ?3, snapshot_expertise_json = ?4, snapshot_instructions = ?5, \
             snapshot_capabilities_json = ?6, snapshot_model = ?7, snapshot_taken_at = ?8, \
             updated_at = ?9 WHERE id = ?10",
            params![
                snap.name,
                snap.role,
                snap.tone,
                exp_to_json(&snap.expertise),
                snap.instructions,
                caps_to_json(&snap.capabilities),
                model_to_str(&snap.model),
                now,
                now,
                id.to_string(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn set_workspace_path(&self, id: AgentDeploymentId, path: &str) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET workspace_path = ?1, updated_at = ?2 WHERE id = ?3",
            params![path, Utc::now().to_rfc3339(), id.to_string()],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn set_next_tick_at(&self, id: AgentDeploymentId, next: DateTime<Utc>) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET next_tick_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![next.to_rfc3339(), Utc::now().to_rfc3339(), id.to_string()],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn clear_error_fields(&self, id: AgentDeploymentId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET last_error_step = NULL, error_message = NULL, \
             error_count = 0, updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id.to_string()],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Read-only candidate scan for the scheduler.
    pub fn find_due_for_tick(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!(
            "SELECT {COLUMNS} FROM agent_deployments \
             WHERE status = 'Running' AND next_tick_at IS NOT NULL AND next_tick_at <= ?1 \
             ORDER BY next_tick_at ASC LIMIT ?2"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(params![now.to_rfc3339(), limit as i64], row_to_deployment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    /// Per-candidate atomic claim (the SQL analogue of `flock -n`).
    /// Returns the rows that this caller successfully locked.
    pub fn claim_due_for_tick(
        &self,
        now: DateTime<Utc>,
        lock_token: &str,
        stale_threshold_secs: i64,
        limit: usize,
    ) -> Result<Vec<AgentDeployment>> {
        let candidates = self.find_due_for_tick(now, limit)?;
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let stale_before = (now - chrono::Duration::seconds(stale_threshold_secs)).to_rfc3339();
        let now_str = now.to_rfc3339();
        let mut won = Vec::new();
        for c in candidates {
            let rows = conn
                .execute(
                    "UPDATE agent_deployments SET tick_locked_at = ?1, tick_lock_token = ?2, updated_at = ?3 \
                     WHERE id = ?4 AND status = 'Running' AND next_tick_at <= ?5 \
                     AND (tick_locked_at IS NULL OR tick_locked_at < ?6)",
                    params![now_str, lock_token, now_str, c.id.to_string(), now_str, stale_before],
                )
                .map_err(|e| DbError::Query(e.to_string()))?;
            if rows == 1 {
                won.push(c);
            }
        }
        Ok(won)
    }

    /// Release a tick lock only if the token matches.
    pub fn release_tick_lock(&self, id: AgentDeploymentId, lock_token: &str) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_deployments SET tick_locked_at = NULL, tick_lock_token = NULL, \
             updated_at = ?1 WHERE id = ?2 AND tick_lock_token = ?3",
            params![Utc::now().to_rfc3339(), id.to_string(), lock_token],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Update materialised stats and clear the lock in one tx.
    #[allow(clippy::too_many_arguments)]
    pub fn bump_after_tick(
        &self,
        id: AgentDeploymentId,
        lock_token: &str,
        next_tick_at: Option<DateTime<Utc>>,
        last_activity_at: Option<DateTime<Utc>>,
        runs_today: i32,
        touched_this_week: i32,
        success_rate: f32,
        error_count: i32,
        error_message: Option<&str>,
    ) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| DbError::Transaction(e.to_string()))?;
        tx.execute(
            "UPDATE agent_deployments SET next_tick_at = ?1, last_activity_at = ?2, \
             runs_today = ?3, touched_this_week = ?4, success_rate = ?5, \
             error_count = ?6, error_message = ?7, tick_locked_at = NULL, tick_lock_token = NULL, \
             updated_at = ?8 WHERE id = ?9 AND tick_lock_token = ?10",
            params![
                next_tick_at.map(|d| d.to_rfc3339()),
                last_activity_at.map(|d| d.to_rfc3339()),
                runs_today,
                touched_this_week,
                success_rate as f64,
                error_count,
                error_message,
                Utc::now().to_rfc3339(),
                id.to_string(),
                lock_token,
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        tx.commit()
            .map_err(|e| DbError::Transaction(e.to_string()))?;
        Ok(())
    }

    /// Deployments stuck mid-activation longer than `staleness_secs` — the
    /// recovery sweep on startup uses this to resume them.
    pub fn find_stuck_in_activation(&self, staleness_secs: i64) -> Result<Vec<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let cutoff = (Utc::now() - chrono::Duration::seconds(staleness_secs)).to_rfc3339();
        let sql = format!(
            "SELECT {COLUMNS} FROM agent_deployments \
             WHERE status IN ('Pending','Registering','Granting','Snapshotting','Provisioning','Bootstrapping','Scheduling') \
             AND updated_at < ?1"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(params![cutoff], row_to_deployment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }
}

impl Repository<AgentDeployment, AgentDeploymentId> for AgentDeploymentRepository {
    fn find_by_id(&self, id: AgentDeploymentId) -> Result<Option<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!("SELECT {COLUMNS} FROM agent_deployments WHERE id = ?1");
        let row = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?
            .query_row(params![id.to_string()], row_to_deployment)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(row)
    }

    fn find_all(&self) -> Result<Vec<AgentDeployment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!("SELECT {COLUMNS} FROM agent_deployments ORDER BY created_at DESC");
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_deployment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    /// Insert a deployment row, auto-assigning `instance_index` to
    /// `MAX(instance_index) + 1` for sibling rows with the same
    /// `(project_id, persona_id)`. The subquery executes inside the same
    /// INSERT statement, so SQLite's writer serialization makes the read
    /// and the write atomic across pool connections — no transaction
    /// needed. The caller's `e.instance_index` is ignored; the assigned
    /// value is returned via `RETURNING`.
    fn create(&self, e: &AgentDeployment) -> Result<AgentDeployment> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let assigned: i32 = conn
            .query_row(
                "INSERT INTO agent_deployments (
                    id, project_id, persona_id, agent_user_id,
                    snapshot_name, snapshot_role, snapshot_tone, snapshot_expertise_json,
                    snapshot_instructions, snapshot_capabilities_json, snapshot_model, snapshot_taken_at,
                    workspace_path, cron_expression, next_tick_at, tick_locked_at, tick_lock_token,
                    status, last_error_step, error_message, error_count, current_ticket_id,
                    runs_today, touched_this_week, success_rate, last_activity_at,
                    instance_index, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7, ?8,
                    ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16, ?17,
                    ?18, ?19, ?20, ?21, ?22,
                    ?23, ?24, ?25, ?26,
                    COALESCE(
                        (SELECT MAX(instance_index) FROM agent_deployments
                         WHERE project_id = ?2 AND persona_id = ?3),
                        0
                    ) + 1,
                    ?27, ?28
                 )
                 RETURNING instance_index",
                params![
                    e.id.to_string(),
                    e.project_id.to_string(),
                    e.persona_id.to_string(),
                    e.agent_user_id.map(|u| u.to_string()),
                    e.snapshot_name,
                    e.snapshot_role,
                    e.snapshot_tone,
                    exp_to_json(&e.snapshot_expertise),
                    e.snapshot_instructions,
                    caps_to_json(&e.snapshot_capabilities),
                    e.snapshot_model.as_ref().map(model_to_str),
                    e.snapshot_taken_at.map(|d| d.to_rfc3339()),
                    e.workspace_path,
                    e.cron_expression,
                    e.next_tick_at.map(|d| d.to_rfc3339()),
                    e.tick_locked_at.map(|d| d.to_rfc3339()),
                    e.tick_lock_token,
                    e.status.to_string(),
                    e.last_error_step.as_ref().map(|s| s.to_string()),
                    e.error_message,
                    e.error_count,
                    e.current_ticket_id.map(|t| t.to_string()),
                    e.runs_today,
                    e.touched_this_week,
                    e.success_rate as f64,
                    e.last_activity_at.map(|d| d.to_rfc3339()),
                    e.created_at.to_rfc3339(),
                    e.updated_at.to_rfc3339(),
                ],
                |r| r.get(0),
            )
            .map_err(|err| DbError::Query(err.to_string()))?;
        let mut out = e.clone();
        out.instance_index = assigned;
        Ok(out)
    }

    fn update(&self, _e: &AgentDeployment) -> Result<AgentDeployment> {
        Err(DbError::Query(
            "AgentDeployment.update is not implemented; use the dedicated transition / setter helpers"
                .into(),
        ))
    }

    fn delete(&self, id: AgentDeploymentId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let rows = conn
            .execute(
                "DELETE FROM agent_deployments WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("Deployment {id} not found")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::project_repository::ProjectRepository;
    use crate::repositories::user_repository::UserRepository;
    use crate::{connection::init_memory_pool, migrations::run_migrations};
    use worknest_core::models::{PersonaId, Project, User};

    fn setup() -> (
        Arc<DbPool>,
        AgentDeploymentRepository,
        ProjectRepository,
        UserRepository,
    ) {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);
        let depl = AgentDeploymentRepository::new(Arc::clone(&pool));
        let proj = ProjectRepository::new(Arc::clone(&pool));
        let user = UserRepository::new(Arc::clone(&pool));
        (pool, depl, proj, user)
    }

    fn seed_persona_id(pool: &DbPool) -> PersonaId {
        // Picked from V7 seed data.
        let conn = pool.get().unwrap();
        let id: String = conn
            .query_row(
                "SELECT id FROM personas WHERE slug = 'tech-lead'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        PersonaId::from_string(&id).unwrap()
    }

    fn make_user(repo: &UserRepository, name: &str) -> User {
        let mut u = User::new(name.into(), format!("{name}@x.test"));
        u.is_agent = false;
        repo.create_with_password(&u, "x").unwrap();
        u
    }

    fn make_project(repo: &ProjectRepository, owner: &User) -> Project {
        let p = Project::new("Demo".into(), owner.id);
        repo.create(&p).unwrap();
        p
    }

    fn new_deployment(project_id: ProjectId, persona_id: PersonaId) -> AgentDeployment {
        let now = Utc::now();
        AgentDeployment {
            id: AgentDeploymentId::new(),
            project_id,
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
            instance_index: 0, // ignored by create(); auto-assigned via subquery
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn create_auto_assigns_instance_index() {
        let (pool, depl, proj, user) = setup();
        let owner = make_user(&user, "alice");
        let p = make_project(&proj, &owner);
        let persona_id = seed_persona_id(&pool);

        // First deployment of (project, persona) gets index 1.
        let d1 = new_deployment(p.id, persona_id);
        let stored1 = depl.create(&d1).unwrap();
        assert_eq!(stored1.instance_index, 1);
        assert_eq!(
            depl.find_by_id(d1.id).unwrap().unwrap().instance_index,
            1,
            "instance_index 1 should round-trip through the row mapper"
        );

        // Second deployment of the SAME (project, persona) is allowed and
        // gets index 2 — the V11 migration removed the unique constraint.
        let d2 = new_deployment(p.id, persona_id);
        let stored2 = depl.create(&d2).unwrap();
        assert_eq!(stored2.instance_index, 2);
        assert_ne!(d1.id, d2.id);

        // list_by_project_and_persona surfaces both, ordered by index.
        let listed = depl.list_by_project_and_persona(p.id, persona_id).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].instance_index, 1);
        assert_eq!(listed[1].instance_index, 2);
    }

    #[test]
    fn transition_status_guards() {
        let (pool, depl, proj, user) = setup();
        let owner = make_user(&user, "bob");
        let p = make_project(&proj, &owner);
        let persona_id = seed_persona_id(&pool);
        let d = new_deployment(p.id, persona_id);
        depl.create(&d).unwrap();
        // Wrong from-state → no rows changed.
        let changed = depl
            .transition_status(d.id, &[AgentStatus::Running], AgentStatus::Paused)
            .unwrap();
        assert!(!changed);
        // Correct from-state → one row changed.
        let changed = depl
            .transition_status(d.id, &[AgentStatus::Pending], AgentStatus::Granting)
            .unwrap();
        assert!(changed);
        let after = depl.find_by_id(d.id).unwrap().unwrap();
        assert_eq!(after.status, AgentStatus::Granting);
    }
}
