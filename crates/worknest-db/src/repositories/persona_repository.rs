//! Repository for the workspace-shared persona catalogue (V7).

use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;

use crate::repositories::{parse_datetime, parse_uuid};
use crate::repository::Repository;
use crate::{DbError, DbPool, Result};
use worknest_core::models::{AgentModel, Capability, Persona, PersonaId};

pub struct PersonaRepository {
    pool: Arc<DbPool>,
}

const PERSONA_COLUMNS: &str = "id, slug, name, emoji, color, description, role, tone, expertise_json, instructions, capabilities_json, model, default_cron, created_at, updated_at";

fn capabilities_to_json(caps: &[Capability]) -> String {
    let strs: Vec<String> = caps.iter().map(|c| c.to_string()).collect();
    serde_json::to_string(&strs).unwrap_or_else(|_| "[]".into())
}

fn capabilities_from_json(s: &str) -> Vec<Capability> {
    let strs: Vec<String> = serde_json::from_str(s).unwrap_or_default();
    strs.into_iter()
        .filter_map(|name| match name.as_str() {
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

fn expertise_to_json(items: &[String]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".into())
}

fn expertise_from_json(s: &str) -> Vec<String> {
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

fn row_to_persona(row: &Row) -> rusqlite::Result<Persona> {
    let id_str: String = row.get(0)?;
    let id = PersonaId::from_uuid(parse_uuid(&id_str)?);
    let expertise_json: String = row.get(8)?;
    let capabilities_json: String = row.get(10)?;
    let model_str: String = row.get(11)?;
    let created_at_str: String = row.get(13)?;
    let updated_at_str: String = row.get(14)?;
    Ok(Persona {
        id,
        slug: row.get(1)?,
        name: row.get(2)?,
        emoji: row.get(3)?,
        color: row.get(4)?,
        description: row.get(5)?,
        role: row.get(6)?,
        tone: row.get(7)?,
        expertise: expertise_from_json(&expertise_json),
        instructions: row.get(9)?,
        capabilities: capabilities_from_json(&capabilities_json),
        model: model_from_str(&model_str),
        default_cron: row.get(12)?,
        created_at: parse_datetime(&created_at_str)?,
        updated_at: parse_datetime(&updated_at_str)?,
    })
}

impl PersonaRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn list_all(&self) -> Result<Vec<Persona>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!("SELECT {PERSONA_COLUMNS} FROM personas ORDER BY name");
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_persona)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows)
    }

    pub fn find_by_slug(&self, slug: &str) -> Result<Option<Persona>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!("SELECT {PERSONA_COLUMNS} FROM personas WHERE slug = ?1");
        let p = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?
            .query_row(params![slug], row_to_persona)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(p)
    }

    pub fn verify_exists(&self, id: PersonaId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM personas WHERE id = ?1",
                params![id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        if count == 0 {
            return Err(DbError::NotFound(format!("Persona {id} not found")));
        }
        Ok(())
    }
}

impl Repository<Persona, PersonaId> for PersonaRepository {
    fn find_by_id(&self, id: PersonaId) -> Result<Option<Persona>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let sql = format!("SELECT {PERSONA_COLUMNS} FROM personas WHERE id = ?1");
        let p = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?
            .query_row(params![id.to_string()], row_to_persona)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(p)
    }

    fn find_all(&self) -> Result<Vec<Persona>> {
        self.list_all()
    }

    fn create(&self, entity: &Persona) -> Result<Persona> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "INSERT INTO personas (id, slug, name, emoji, color, description, role, tone, expertise_json, instructions, capabilities_json, model, default_cron, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                entity.id.to_string(),
                entity.slug,
                entity.name,
                entity.emoji,
                entity.color,
                entity.description,
                entity.role,
                entity.tone,
                expertise_to_json(&entity.expertise),
                entity.instructions,
                capabilities_to_json(&entity.capabilities),
                model_to_str(&entity.model),
                entity.default_cron,
                entity.created_at.to_rfc3339(),
                entity.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                DbError::ConstraintViolation(format!("Persona slug '{}' already exists", entity.slug))
            } else {
                DbError::Query(e.to_string())
            }
        })?;
        Ok(entity.clone())
    }

    fn update(&self, entity: &Persona) -> Result<Persona> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let now = Utc::now();
        let rows = conn
            .execute(
                "UPDATE personas SET slug = ?1, name = ?2, emoji = ?3, color = ?4, description = ?5,
                                     role = ?6, tone = ?7, expertise_json = ?8, instructions = ?9,
                                     capabilities_json = ?10, model = ?11, default_cron = ?12, updated_at = ?13
                 WHERE id = ?14",
                params![
                    entity.slug,
                    entity.name,
                    entity.emoji,
                    entity.color,
                    entity.description,
                    entity.role,
                    entity.tone,
                    expertise_to_json(&entity.expertise),
                    entity.instructions,
                    capabilities_to_json(&entity.capabilities),
                    model_to_str(&entity.model),
                    entity.default_cron,
                    now.to_rfc3339(),
                    entity.id.to_string(),
                ],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DbError::ConstraintViolation("Persona slug already exists".into())
                } else {
                    DbError::Query(e.to_string())
                }
            })?;
        if rows == 0 {
            return Err(DbError::NotFound(format!(
                "Persona {} not found",
                entity.id
            )));
        }
        let mut updated = entity.clone();
        updated.updated_at = now;
        Ok(updated)
    }

    fn delete(&self, id: PersonaId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        conn.execute(
            "DELETE FROM personas WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(|e| {
            if e.to_string().contains("FOREIGN KEY constraint failed") {
                DbError::ConstraintViolation("Persona is in use by one or more deployments".into())
            } else {
                DbError::Query(e.to_string())
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection::init_memory_pool, migrations::run_migrations};

    fn setup() -> PersonaRepository {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);
        PersonaRepository::new(pool)
    }

    #[test]
    fn lists_seeded_personas() {
        let repo = setup();
        let personas = repo.list_all().unwrap();
        assert_eq!(personas.len(), 9);
        let slugs: Vec<&str> = personas.iter().map(|p| p.slug.as_str()).collect();
        for required in [
            "triage",
            "reviewer",
            "reproducer",
            "docs",
            "standup",
            "researcher",
            "tech-lead",
            "frontend",
            "backend",
        ] {
            assert!(
                slugs.contains(&required),
                "missing seeded persona: {required}"
            );
        }
    }

    #[test]
    fn round_trip_capabilities_and_expertise() {
        let repo = setup();
        let p = repo.find_by_slug("tech-lead").unwrap().unwrap();
        assert!(p.capabilities.contains(&Capability::CreateTicket));
        assert!(!p.expertise.is_empty());
        assert_eq!(p.model, AgentModel::Sonnet);
    }
}
