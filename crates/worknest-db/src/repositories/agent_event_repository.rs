//! Append-only lifecycle audit log per deployment.

use chrono::Utc;
use rusqlite::{params, Row};
use std::sync::Arc;

use crate::repositories::{parse_datetime, parse_uuid};
use crate::{DbError, DbPool, Result};
use worknest_core::models::{AgentDeploymentId, AgentEvent, AgentEventId, AgentEventKind};

pub struct AgentEventRepository {
    pool: Arc<DbPool>,
}

fn row_to_event(row: &Row) -> rusqlite::Result<AgentEvent> {
    let id_str: String = row.get(0)?;
    let depl_str: String = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let payload_str: String = row.get(3)?;
    let at_str: String = row.get(5)?;
    let map_err = |e: worknest_core::CoreError| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    };
    Ok(AgentEvent {
        id: AgentEventId::from_uuid(parse_uuid(&id_str)?),
        deployment_id: AgentDeploymentId::from_uuid(parse_uuid(&depl_str)?),
        kind: AgentEventKind::parse(&kind_str).map_err(map_err)?,
        payload: serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Null),
        message: row.get(4)?,
        at: parse_datetime(&at_str)?,
    })
}

impl AgentEventRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn record(
        &self,
        deployment_id: AgentDeploymentId,
        kind: AgentEventKind,
        message: &str,
        payload: serde_json::Value,
    ) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "INSERT INTO agent_events (id, deployment_id, kind, payload_json, message, at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                AgentEventId::new().to_string(),
                deployment_id.to_string(),
                kind.to_string(),
                payload.to_string(),
                message,
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn list_for_deployment(
        &self,
        deployment_id: AgentDeploymentId,
        limit: i64,
    ) -> Result<Vec<AgentEvent>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, deployment_id, kind, payload_json, message, at \
                 FROM agent_events WHERE deployment_id = ?1 \
                 ORDER BY at DESC LIMIT ?2",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(params![deployment_id.to_string(), limit], row_to_event)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }
}
