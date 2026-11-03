//! DENDRITE SQLite persistence for Paraclea.
//!
//! Stores knowledge graph nodes and full-text search indices in `$HOME/.paraclea/dendrite.db`.

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use crate::dendrite::graph::{CompactNodeCard, Dendrite, Node, NodeType};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dendrite_nodes (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL DEFAULT '',
    type            TEXT NOT NULL DEFAULT 'custom',
    tags            TEXT NOT NULL DEFAULT '[]',
    links           TEXT NOT NULL DEFAULT '[]',
    backlinks       TEXT NOT NULL DEFAULT '[]',
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    topic_key       TEXT,
    revision_count  INTEGER NOT NULL DEFAULT 1,
    content_hash    TEXT,
    duplicate_count INTEGER NOT NULL DEFAULT 0,
    last_seen_at    INTEGER NOT NULL DEFAULT 0,
    review_after    INTEGER,
    state           TEXT NOT NULL DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_dendrite_updated ON dendrite_nodes(updated_at DESC);
"#;

const FTS5_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS dendrite_fts USING fts5(
    id         UNINDEXED,
    title,
    content,
    tags,
    topic_key,
    tokenize = 'porter unicode61'
);
"#;

const FTS_FALLBACK_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dendrite_fts_fallback (
    id        TEXT PRIMARY KEY,
    title     TEXT,
    content   TEXT,
    tags      TEXT,
    topic_key TEXT
);
"#;

const TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ai
AFTER INSERT ON dendrite_nodes BEGIN
    INSERT INTO dendrite_fts(id, title, content, tags, topic_key)
    VALUES (new.id, new.title, new.content, new.tags, new.topic_key);
END;

CREATE TRIGGER IF NOT EXISTS dendrite_nodes_au
AFTER UPDATE ON dendrite_nodes BEGIN
    DELETE FROM dendrite_fts WHERE id = old.id;
    INSERT INTO dendrite_fts(id, title, content, tags, topic_key)
    VALUES (new.id, new.title, new.content, new.tags, new.topic_key);
END;

CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ad
AFTER DELETE ON dendrite_nodes BEGIN
    DELETE FROM dendrite_fts WHERE id = old.id;
END;
"#;

/// Persists DENDRITE nodes to SQLite.
pub struct DendriteStore {
    conn: Mutex<Connection>,
    has_fts: bool,
}

fn lock_conn(conn: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    conn.lock().unwrap_or_else(|e| e.into_inner())
}

impl DendriteStore {
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<DendriteStore> {
        if let Some(parent) = db_path.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open_with_flags(
            db_path.as_ref(),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("open graph db")?;
        conn.busy_timeout(Duration::from_millis(5000))
            .context("set busy_timeout")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL")?;

        let mut store = DendriteStore {
            conn: Mutex::new(conn),
            has_fts: true,
        };
        store.migrate().context("graph db migrate")?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let conn = lock_conn(&self.conn);
        conn.execute_batch(SCHEMA).context("create core table")?;

        // Inspect existing columns for non-destructive backwards migration
        let mut existing_cols = HashSet::new();
        if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(dendrite_nodes)") {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) {
                for col in rows.flatten() {
                    existing_cols.insert(col);
                }
            }
        }

        let migrations = [
            ("topic_key", "ALTER TABLE dendrite_nodes ADD COLUMN topic_key TEXT"),
            ("revision_count", "ALTER TABLE dendrite_nodes ADD COLUMN revision_count INTEGER NOT NULL DEFAULT 1"),
            ("content_hash", "ALTER TABLE dendrite_nodes ADD COLUMN content_hash TEXT"),
            ("duplicate_count", "ALTER TABLE dendrite_nodes ADD COLUMN duplicate_count INTEGER NOT NULL DEFAULT 0"),
            ("last_seen_at", "ALTER TABLE dendrite_nodes ADD COLUMN last_seen_at INTEGER NOT NULL DEFAULT 0"),
            ("review_after", "ALTER TABLE dendrite_nodes ADD COLUMN review_after INTEGER"),
            ("state", "ALTER TABLE dendrite_nodes ADD COLUMN state TEXT NOT NULL DEFAULT 'active'"),
        ];

        for (col, sql) in migrations {
            if !existing_cols.contains(col) {
                let _ = conn.execute(sql, []);
            }
        }

        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_dendrite_topic ON dendrite_nodes(topic_key)", []);
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_dendrite_hash ON dendrite_nodes(content_hash)", []);

        match conn.execute_batch(FTS5_SCHEMA) {
            Ok(()) => {
                self.has_fts = true;
                conn.execute_batch(TRIGGERS).context("create fts triggers")?;
            }
            Err(_) => {
                self.has_fts = false;
                let _ = conn.execute_batch(FTS_FALLBACK_SCHEMA);
            }
        }
        Ok(())
    }

    /// Upsert a node into SQLite. created_at is preserved on conflict.
    pub fn save(&self, node: &Node) -> Result<()> {
        let tags = serde_json::to_string(&node.tags)?;
        let links = serde_json::to_string(&node.links)?;
        let backlinks = serde_json::to_string(&node.backlinks)?;

        let mut conn = lock_conn(&self.conn);
        let tx = conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO dendrite_nodes
                (id, title, content, type, tags, links, backlinks, created_at, updated_at,
                 topic_key, revision_count, content_hash, duplicate_count, last_seen_at, review_after, state)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title           = excluded.title,
                content         = excluded.content,
                type            = excluded.type,
                tags            = excluded.tags,
                links           = excluded.links,
                backlinks       = excluded.backlinks,
                updated_at      = excluded.updated_at,
                topic_key       = excluded.topic_key,
                revision_count  = excluded.revision_count,
                content_hash    = excluded.content_hash,
                duplicate_count = excluded.duplicate_count,
                last_seen_at    = excluded.last_seen_at,
                review_after    = excluded.review_after,
                state           = excluded.state
            "#,
            params![
                node.id,
                node.title,
                node.content,
                node.node_type.as_str(),
                tags,
                links,
                backlinks,
                node.created_at,
                node.updated_at,
                node.topic_key,
                node.revision_count,
                node.content_hash,
                node.duplicate_count,
                node.last_seen_at,
                node.review_after,
                node.state,
            ],
        )?;

        if !self.has_fts {
            tx.execute(
                r#"
                INSERT INTO dendrite_fts_fallback (id, title, content, tags, topic_key)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    title     = excluded.title,
                    content   = excluded.content,
                    tags      = excluded.tags,
                    topic_key = excluded.topic_key
                "#,
                params![node.id, node.title, node.content, tags, node.topic_key],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Executes an in-memory evolutionary upsert and persists the outcome to SQLite.
    pub fn save_by_topic(&self, graph: &Dendrite, node: Node) -> Result<Node> {
        let (persisted_node, _mutated) = graph.upsert_by_topic(node);
        self.save(&persisted_node)?;
        Ok(persisted_node)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let mut conn = lock_conn(&self.conn);
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM dendrite_nodes WHERE id = ?", [id])?;
        if !self.has_fts {
            tx.execute("DELETE FROM dendrite_fts_fallback WHERE id = ?", [id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Hydrate all stored nodes directly into the graph's node map.
    pub fn load_all(&self, graph: &Dendrite) -> Result<()> {
        let conn = lock_conn(&self.conn);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, type, tags, links, backlinks, created_at, updated_at,
                   topic_key, revision_count, content_hash, duplicate_count, last_seen_at, review_after, state
            FROM dendrite_nodes
            ORDER BY updated_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let node_type: String = row.get(3)?;
            let tags: String = row.get(4)?;
            let links: String = row.get(5)?;
            let backlinks: String = row.get(6)?;
            let topic_key: Option<String> = row.get(9)?;
            let revision_count: u32 = row.get(10)?;
            let content_hash: Option<String> = row.get(11)?;
            let duplicate_count: u32 = row.get(12)?;
            let last_seen_at: i64 = row.get(13)?;
            let review_after: Option<i64> = row.get(14)?;
            let state: String = row.get(15)?;

            Ok(Node {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                node_type: NodeType::from_str(&node_type),
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                links: serde_json::from_str(&links).unwrap_or_default(),
                backlinks: serde_json::from_str(&backlinks).unwrap_or_default(),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                topic_key,
                revision_count,
                content_hash,
                duplicate_count,
                last_seen_at,
                review_after,
                state,
            })
        })?;

        for node in rows {
            let node = node.context("reading node row")?;
            graph.insert_hydrated(node);
        }
        Ok(())
    }

    /// Full-text search, returning matching node IDs ordered by rank.
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let limit = if limit == 0 { 10 } else { limit };
        let conn = lock_conn(&self.conn);

        if self.has_fts {
            let mut stmt = conn.prepare(
                r#"
                SELECT id FROM dendrite_fts
                WHERE dendrite_fts MATCH ?
                ORDER BY rank
                LIMIT ?
                "#,
            )?;
            let rows = stmt.query_map(params![query, limit as i64], |row| row.get(0))?;
            let mut ids = Vec::new();
            for id in rows.flatten() {
                ids.push(id);
            }
            return Ok(ids);
        }

        let pattern = format!("%{query}%");
        let mut stmt = conn.prepare(
            r#"
            SELECT id FROM dendrite_fts_fallback
            WHERE title LIKE ? OR content LIKE ? OR tags LIKE ? OR topic_key LIKE ?
            LIMIT ?
            "#,
        )?;
        let rows =
            stmt.query_map(params![pattern, pattern, pattern, pattern, limit as i64], |row| {
                row.get(0)
            })?;
        let mut ids = Vec::new();
        for id in rows.flatten() {
            ids.push(id);
        }
        Ok(ids)
    }

    /// Number of rows in the core table.
    pub fn node_count(&self) -> Result<usize> {
        let conn = lock_conn(&self.conn);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM dendrite_nodes", [], |r| {
            r.get(0)
        })?;
        Ok(count as usize)
    }

    /// Retrieve Layer 1 compact cards directly from SQLite index.
    pub fn get_layer1_cards(&self, limit: usize) -> Result<Vec<CompactNodeCard>> {
        let limit = if limit == 0 { 50 } else { limit };
        let conn = lock_conn(&self.conn);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, type, tags, topic_key, revision_count, updated_at
            FROM dendrite_nodes
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let node_type: String = row.get(3)?;
            let tags_raw: String = row.get(4)?;
            let content: String = row.get(2)?;
            let char_count = content.chars().count();
            let preview = if char_count > 150 {
                let truncated: String = content.chars().take(150).collect();
                format!("{}...", truncated)
            } else {
                content
            };

            Ok(CompactNodeCard {
                id: row.get(0)?,
                title: row.get(1)?,
                preview,
                node_type: NodeType::from_str(&node_type),
                tags: serde_json::from_str(&tags_raw).unwrap_or_default(),
                topic_key: row.get(5)?,
                revision_count: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut cards = Vec::new();
        for card in rows {
            cards.push(card?);
        }
        Ok(cards)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_topic_key_upsert_and_progressive_disclosure() {
        let tmp = NamedTempFile::new().unwrap();
        let store = DendriteStore::open(tmp.path()).unwrap();
        let graph = Dendrite::new();

        // 1. Initial insert with topic_key
        let mut node1 = Node::new("genesis_covenant", "The Abrahamic Covenant", "God promises Abraham [[isaac]] and blessing. #covenant #promise", NodeType::Concept);
        node1.topic_key = Some("biblical_covenant_abraham".to_string());

        let res1 = store.save_by_topic(&graph, node1).unwrap();
        assert_eq!(res1.id, "genesis_covenant");
        assert_eq!(res1.revision_count, 1);
        assert_eq!(res1.duplicate_count, 0);

        // 2. Duplicate observation (same topic_key and content) -> rolling deduplication hit
        let mut dup_node = Node::new("other_id", "Abraham Covenant Duplicate", "God promises Abraham [[isaac]] and blessing. #covenant #promise", NodeType::Concept);
        dup_node.topic_key = Some("biblical_covenant_abraham".to_string());

        let res2 = store.save_by_topic(&graph, dup_node).unwrap();
        assert_eq!(res2.id, "genesis_covenant");
        assert_eq!(res2.revision_count, 1);
        assert_eq!(res2.duplicate_count, 1);

        // 3. Evolutionary update (same topic_key, altered content) -> revision_count increment
        let mut updated_node = Node::new("yet_another_id", "The Abrahamic Covenant Expanded", "God covenants with Abraham concerning [[isaac]], seed, and all nations. #covenant #seed", NodeType::Concept);
        updated_node.topic_key = Some("biblical_covenant_abraham".to_string());

        let res3 = store.save_by_topic(&graph, updated_node).unwrap();
        assert_eq!(res3.id, "genesis_covenant");
        assert_eq!(res3.revision_count, 2);
        assert!(res3.content.contains("seed"));

        // 4. Verify Progressive Disclosure Layer 1 cards
        let cards = store.get_layer1_cards(10).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "genesis_covenant");
        assert_eq!(cards[0].revision_count, 2);

        // 5. Verify Progressive Disclosure Layer 2 cluster context
        let cluster = graph.get_cluster_context("genesis_covenant").unwrap();
        assert_eq!(cluster.central_id, "genesis_covenant");
        assert!(cluster.outgoing_links.contains(&"isaac".to_string()));
    }

    #[test]
    fn test_dendrite_schema_migration() {
        let tmp = NamedTempFile::new().unwrap();
        {
            // Simulate legacy SQLite table without engram columns
            let conn = Connection::open(tmp.path()).unwrap();
            conn.execute_batch(r#"
                CREATE TABLE dendrite_nodes (
                    id         TEXT PRIMARY KEY,
                    title      TEXT NOT NULL,
                    content    TEXT NOT NULL DEFAULT '',
                    type       TEXT NOT NULL DEFAULT 'custom',
                    tags       TEXT NOT NULL DEFAULT '[]',
                    links      TEXT NOT NULL DEFAULT '[]',
                    backlinks  TEXT NOT NULL DEFAULT '[]',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                INSERT INTO dendrite_nodes (id, title, content, type, tags, links, backlinks, created_at, updated_at)
                VALUES ('legacy_1', 'Old Fact', 'Pre-existing content', 'atomic_fact', '[]', '[]', '[]', 1000, 2000);
            "#).unwrap();
        }

        // Open with updated DendriteStore -> must migrate cleanly
        let store = DendriteStore::open(tmp.path()).unwrap();
        let graph = Dendrite::new();
        store.load_all(&graph).unwrap();

        assert_eq!(graph.len(), 1);
        let node = graph.get("legacy_1").unwrap();
        assert_eq!(node.title, "Old Fact");
        assert_eq!(node.revision_count, 1);
        assert_eq!(node.state, "active");
        assert_eq!(node.duplicate_count, 0);
    }
}
