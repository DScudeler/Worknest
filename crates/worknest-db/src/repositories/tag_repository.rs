//! Repository for Tag operations.

use rusqlite::{params, Row};
use std::collections::HashMap;
use std::sync::Arc;

use crate::repositories::{parse_datetime, parse_uuid};
use crate::{DbError, DbPool, Result};
use worknest_core::models::{Tag, TagId, TicketId};

pub struct TagRepository {
    pool: Arc<DbPool>,
}

fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
    let id_str: String = row.get(0)?;
    let created_at_str: String = row.get(4)?;
    Ok(Tag {
        id: TagId::from_uuid(parse_uuid(&id_str)?),
        name: row.get(1)?,
        color_bg: row.get(2)?,
        color_fg: row.get(3)?,
        created_at: parse_datetime(&created_at_str)?,
    })
}

impl TagRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// List every tag, alphabetical by name.
    pub fn list_all(&self) -> Result<Vec<Tag>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, color_bg, color_fg, created_at
                 FROM tags ORDER BY name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tags = stmt
            .query_map([], row_to_tag)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tags)
    }

    /// Tags attached to a single ticket, alphabetical.
    pub fn list_for_ticket(&self, ticket_id: TicketId) -> Result<Vec<Tag>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.color_bg, t.color_fg, t.created_at
                 FROM tags t
                 INNER JOIN ticket_tags tt ON tt.tag_id = t.id
                 WHERE tt.ticket_id = ?1
                 ORDER BY t.name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tags = stmt
            .query_map(params![ticket_id.to_string()], row_to_tag)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tags)
    }

    /// Batch-load tags for many tickets in a single query. Returns a map
    /// keyed by `TicketId`; tickets with no tags are simply absent from
    /// the map (callers should default to an empty Vec).
    pub fn list_for_tickets(
        &self,
        ticket_ids: &[TicketId],
    ) -> Result<HashMap<TicketId, Vec<Tag>>> {
        let mut out: HashMap<TicketId, Vec<Tag>> = HashMap::new();
        if ticket_ids.is_empty() {
            return Ok(out);
        }
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        // Build "?,?,?,..." placeholders for the IN list.
        let placeholders = std::iter::repeat_n("?", ticket_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT tt.ticket_id, t.id, t.name, t.color_bg, t.color_fg, t.created_at
             FROM ticket_tags tt
             INNER JOIN tags t ON t.id = tt.tag_id
             WHERE tt.ticket_id IN ({placeholders})
             ORDER BY tt.ticket_id, t.name"
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::Query(e.to_string()))?;

        let id_strings: Vec<String> = ticket_ids.iter().map(|id| id.to_string()).collect();
        let params_iter = rusqlite::params_from_iter(id_strings.iter());

        let rows = stmt
            .query_map(params_iter, |row| {
                let ticket_id_str: String = row.get(0)?;
                let tag_id_str: String = row.get(1)?;
                let created_at_str: String = row.get(5)?;
                let ticket_id = TicketId::from_uuid(parse_uuid(&ticket_id_str)?);
                let tag = Tag {
                    id: TagId::from_uuid(parse_uuid(&tag_id_str)?),
                    name: row.get(2)?,
                    color_bg: row.get(3)?,
                    color_fg: row.get(4)?,
                    created_at: parse_datetime(&created_at_str)?,
                };
                Ok((ticket_id, tag))
            })
            .map_err(|e| DbError::Query(e.to_string()))?;

        for row in rows {
            let (ticket_id, tag) = row.map_err(|e| DbError::Query(e.to_string()))?;
            out.entry(ticket_id).or_default().push(tag);
        }
        Ok(out)
    }

    /// Replace the tag set for a ticket. Diffs against the current set
    /// inside a single transaction so concurrent edits stay consistent.
    pub fn set_for_ticket(&self, ticket_id: TicketId, tag_ids: &[TagId]) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| DbError::Query(e.to_string()))?;

        tx.execute(
            "DELETE FROM ticket_tags WHERE ticket_id = ?1",
            params![ticket_id.to_string()],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;

        for tag_id in tag_ids {
            tx.execute(
                "INSERT OR IGNORE INTO ticket_tags (ticket_id, tag_id) VALUES (?1, ?2)",
                params![ticket_id.to_string(), tag_id.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        }

        tx.commit().map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Verify every TagId in the slice exists. Returns the first unknown id
    /// as an error so the caller can surface a 400.
    pub fn verify_exists(&self, tag_ids: &[TagId]) -> Result<()> {
        if tag_ids.is_empty() {
            return Ok(());
        }
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        for id in tag_ids {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM tags WHERE id = ?1",
                    params![id.to_string()],
                    |row| row.get(0),
                )
                .map_err(|e| DbError::Query(e.to_string()))?;
            if count == 0 {
                return Err(DbError::NotFound(format!("Tag {id} not found")));
            }
        }
        Ok(())
    }
}
