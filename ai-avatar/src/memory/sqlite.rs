//! SQLite-backed session storage.

use crate::memory::{Message, Role, now_timestamp};
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tracing::{debug, info};

/// Simple SQLite memory: stores raw messages in a single table.
pub struct SqliteMemory {
    conn: Connection,
    max_history: usize,
}

impl SqliteMemory {
    pub fn open(path: PathBuf, max_history: usize) -> Result<Self> {
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))?;
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open SQLite at {}", path.display()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS summaries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                message_count INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        info!("SQLite memory initialised at {}", path.display());
        Ok(Self { conn, max_history })
    }

    pub fn save_message_inner(&mut self, msg: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (role, content, timestamp) VALUES (?1, ?2, ?3)",
            params![msg.role.to_string(), msg.content, msg.timestamp],
        )?;
        debug!("Saved message: {:?}", msg.role);
        Ok(())
    }

    pub fn get_history_inner(&self, limit: usize) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content, timestamp FROM messages ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| {
            let role_str: String = row.get(0)?;
            let role = role_str.parse().unwrap_or(Role::System);
            Ok(Message {
                role,
                content: row.get(1)?,
                timestamp: row.get(2)?,
            })
        })?;

        let mut messages: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
        messages.reverse(); // chronological order
        Ok(messages)
    }

    pub fn count_messages(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn compact_inner(&mut self) -> Result<()> {
        let total = self.count_messages()?;
        let threshold = self.max_history + 20;
        if total <= threshold {
            return Ok(());
        }

        let to_compact = total - self.max_history;
        info!("Compacting {} old messages", to_compact);

        // Grab oldest messages to compact.
        let mut stmt = self.conn.prepare(
            "SELECT role, content, timestamp FROM messages ORDER BY timestamp ASC LIMIT ?1"
        )?;
        let rows = stmt.query_map([to_compact], |row| {
            Ok(Message {
                role: row.get::<_, String>(0)?.parse().unwrap_or(Role::System),
                content: row.get(1)?,
                timestamp: row.get(2)?,
            })
        })?;
        let old: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;

        // Create a simple placeholder summary (LLM summary can be added later).
        let summary_text = format!("[{} older messages compacted]", old.len());
        self.conn.execute(
            "INSERT INTO summaries (content, message_count, timestamp) VALUES (?1, ?2, ?3)",
            params![summary_text, old.len(), now_timestamp()],
        )?;

        // Delete compacted messages.
        let cutoff = old.last().map(|m| m.timestamp).unwrap_or(0);
        self.conn.execute(
            "DELETE FROM messages WHERE timestamp <= ?1",
            [cutoff],
        )?;

        info!("Compaction complete: {} messages removed", old.len());
        Ok(())
    }

    pub fn get_summaries(&self, limit: usize) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM summaries ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;
        rows.collect::<Result<Vec<_>, _>>().context("failed to fetch summaries")
    }

    pub fn clear_inner(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM messages", [])?;
        self.conn.execute("DELETE FROM summaries", [])?;
        info!("Cleared all session history");
        Ok(())
    }

    pub fn get_history_since_inner(&self, since: i64) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content, timestamp FROM messages WHERE timestamp >= ?1 ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map([since], |row| {
            Ok(Message {
                role: row.get::<_, String>(0)?.parse().unwrap_or(Role::System),
                content: row.get(1)?,
                timestamp: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().context("failed to fetch recent history")
    }
}
