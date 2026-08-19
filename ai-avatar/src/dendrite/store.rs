//! SQLite persistence layer for DENDRITE with FTS5 full-text search.

use crate::dendrite::{Node, NodeType};
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

pub struct DendriteStore {
    conn: Connection,
    has_fts5: bool,
}

impl DendriteStore {
    pub fn open(path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))?;
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open dendrite db at {}", path.display()))?;

        // Main nodes table.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dendrite_nodes (
                id         TEXT PRIMARY KEY,
                title      TEXT NOT NULL,
                content    TEXT NOT NULL DEFAULT '',
                type       TEXT NOT NULL DEFAULT 'custom',
                tags       TEXT NOT NULL DEFAULT '[]',
                links      TEXT NOT NULL DEFAULT '[]',
                backlinks  TEXT NOT NULL DEFAULT '[]',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Try FTS5; fall back to a simple fallback table.
        let has_fts5 = match conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS dendrite_fts USING fts5(
                id UNINDEXED,
                title,
                content,
                tags,
                tokenize = 'porter unicode61'
            )",
            [],
        ) {
            Ok(_) => {
                // Set up triggers to keep FTS index in sync.
                let _ = conn.execute(
                    "CREATE TRIGGER IF NOT EXISTS dendrite_fts_insert
                     AFTER INSERT ON dendrite_nodes BEGIN
                       INSERT INTO dendrite_fts(id, title, content, tags)
                       VALUES (NEW.id, NEW.title, NEW.content, NEW.tags);
                     END",
                    [],
                );
                let _ = conn.execute(
                    "CREATE TRIGGER IF NOT EXISTS dendrite_fts_update
                     AFTER UPDATE ON dendrite_nodes BEGIN
                       UPDATE dendrite_fts SET title=NEW.title, content=NEW.content, tags=NEW.tags
                       WHERE id=NEW.id;
                     END",
                    [],
                );
                let _ = conn.execute(
                    "CREATE TRIGGER IF NOT EXISTS dendrite_fts_delete
                     AFTER DELETE ON dendrite_nodes BEGIN
                       DELETE FROM dendrite_fts WHERE id=OLD.id;
                     END",
                    [],
                );
                info!("Dendrite FTS5 index active");
                true
            }
            Err(e) => {
                warn!("FTS5 not available ({}), falling back to substring search", e);
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dendrite_fts_fallback (
                        id TEXT PRIMARY KEY,
                        title TEXT,
                        content TEXT,
                        tags TEXT
                    )",
                    [],
                )?;
                false
            }
        };

        Ok(Self { conn, has_fts5 })
    }

    pub fn load_all(&self) -> Result<HashMap<String, Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, type, tags, links, backlinks, created_at, updated_at FROM dendrite_nodes"
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let type_str: String = row.get(3)?;
            let tags_json: String = row.get(4)?;
            let links_json: String = row.get(5)?;
            let backlinks_json: String = row.get(6)?;
            let created_at: i64 = row.get(7)?;
            let updated_at: i64 = row.get(8)?;

            let node_type = type_str.parse().unwrap_or(NodeType::Custom);
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let links: Vec<String> = serde_json::from_str(&links_json).unwrap_or_default();
            let backlinks: Vec<String> = serde_json::from_str(&backlinks_json).unwrap_or_default();

            Ok((id, Node {
                id: row.get(0)?,
                title,
                content,
                node_type,
                tags,
                links,
                backlinks,
                created_at,
                updated_at,
            }))
        })?;

        let mut nodes = HashMap::new();
        for row in rows {
            let (id, node) = row?;
            nodes.insert(id, node);
        }
        Ok(nodes)
    }

    pub fn save(&self, node: &Node) -> Result<()> {
        let tags_json = serde_json::to_string(&node.tags)?;
        let links_json = serde_json::to_string(&node.links)?;
        let backlinks_json = serde_json::to_string(&node.backlinks)?;

        self.conn.execute(
            "INSERT INTO dendrite_nodes (id, title, content, type, tags, links, backlinks, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               title=excluded.title,
               content=excluded.content,
               type=excluded.type,
               tags=excluded.tags,
               links=excluded.links,
               backlinks=excluded.backlinks,
               updated_at=excluded.updated_at",
            params![
                node.id, node.title, node.content, node.node_type.to_string(),
                tags_json, links_json, backlinks_json,
                node.created_at, node.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM dendrite_nodes WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn fts_search(&self, query: &str) -> Result<Vec<String>> {
        let q = query.to_lowercase();
        if self.has_fts5 {
            let sql = "SELECT id FROM dendrite_fts WHERE dendrite_fts MATCH ?1 LIMIT 50";
            let mut stmt = self.conn.prepare(sql)?;
            let rows = stmt.query_map([q], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;
            rows.collect::<Result<Vec<_>, _>>().context("FTS5 search failed")
        } else {
            // Fallback: LIKE search on fallback table.
            let pattern = format!("%{}%", q);
            let mut stmt = self.conn.prepare(
                "SELECT id FROM dendrite_fts_fallback WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1 LIMIT 50"
            )?;
            let rows = stmt.query_map([pattern], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;
            rows.collect::<Result<Vec<_>, _>>().context("fallback search failed")
        }
    }
}
