//! Append-only tick history (per scheduler firing).

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::sync::Arc;

use crate::repositories::{parse_datetime, parse_uuid};
use crate::{DbError, DbPool, Result};
use worknest_core::models::{AgentDeploymentId, AgentTick, AgentTickId, TickOutcome, TicketId};

pub struct AgentTickRepository {
    pool: Arc<DbPool>,
}

#[derive(Debug, Clone, Default)]
pub struct TickStats {
    pub runs: i32,
    pub successes: i32,
    pub touched_tickets: i32,
}

fn parse_dt_opt(row: &Row, idx: usize) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let s: Option<String> = row.get(idx)?;
    match s {
        Some(s) => Ok(Some(parse_datetime(&s)?)),
        None => Ok(None),
    }
}

fn row_to_tick(row: &Row) -> rusqlite::Result<AgentTick> {
    let id_str: String = row.get(0)?;
    let deployment_id_str: String = row.get(1)?;
    let started_at_str: String = row.get(2)?;
    let outcome: Option<String> = row.get(4)?;
    let touched: Option<String> = row.get(5)?;
    let map_err = |e: worknest_core::CoreError| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    };
    Ok(AgentTick {
        id: AgentTickId::from_uuid(parse_uuid(&id_str)?),
        deployment_id: AgentDeploymentId::from_uuid(parse_uuid(&deployment_id_str)?),
        started_at: parse_datetime(&started_at_str)?,
        finished_at: parse_dt_opt(row, 3)?,
        outcome: match outcome {
            Some(s) => Some(TickOutcome::parse(&s).map_err(map_err)?),
            None => None,
        },
        touched_ticket_id: match touched {
            Some(s) => Some(TicketId::from_uuid(parse_uuid(&s)?)),
            None => None,
        },
        action_summary: row.get(6)?,
        error_message: row.get(7)?,
    })
}

impl AgentTickRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn start(&self, deployment_id: AgentDeploymentId) -> Result<AgentTick> {
        let tick = AgentTick {
            id: AgentTickId::new(),
            deployment_id,
            started_at: Utc::now(),
            finished_at: None,
            outcome: None,
            touched_ticket_id: None,
            action_summary: None,
            error_message: None,
        };
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "INSERT INTO agent_ticks (id, deployment_id, started_at) VALUES (?1, ?2, ?3)",
            params![
                tick.id.to_string(),
                tick.deployment_id.to_string(),
                tick.started_at.to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(tick)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finish(
        &self,
        tick_id: AgentTickId,
        outcome: TickOutcome,
        action_summary: Option<&str>,
        touched_ticket_id: Option<TicketId>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "UPDATE agent_ticks SET finished_at = ?1, outcome = ?2, action_summary = ?3, \
             touched_ticket_id = ?4, error_message = ?5 WHERE id = ?6",
            params![
                Utc::now().to_rfc3339(),
                outcome.to_string(),
                action_summary,
                touched_ticket_id.map(|t| t.to_string()),
                error_message,
                tick_id.to_string(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn list_for_deployment(
        &self,
        deployment_id: AgentDeploymentId,
        limit: i64,
    ) -> Result<Vec<AgentTick>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, deployment_id, started_at, finished_at, outcome, \
                        touched_ticket_id, action_summary, error_message \
                 FROM agent_ticks WHERE deployment_id = ?1 \
                 ORDER BY started_at DESC LIMIT ?2",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(params![deployment_id.to_string(), limit], row_to_tick)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    /// Aggregate tick stats since `since` for one deployment.
    pub fn aggregate_stats(
        &self,
        deployment_id: AgentDeploymentId,
        since: DateTime<Utc>,
    ) -> Result<TickStats> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let row: (i64, i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), \
                        SUM(CASE WHEN outcome = 'Success' THEN 1 ELSE 0 END), \
                        COUNT(DISTINCT touched_ticket_id) \
                 FROM agent_ticks \
                 WHERE deployment_id = ?1 AND started_at >= ?2 AND outcome IS NOT NULL",
                params![deployment_id.to_string(), since.to_rfc3339()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                        row.get(2)?,
                    ))
                },
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(TickStats {
            runs: row.0 as i32,
            successes: row.1 as i32,
            touched_tickets: row.2 as i32,
        })
    }
}
