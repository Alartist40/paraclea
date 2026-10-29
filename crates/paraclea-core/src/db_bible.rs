//! Unified SQLite WAL + FTS5 Bible Storage Engine.
//!
//! Provides zero-RAM parsing overhead, sub-millisecond verse lookups,
//! and BM25 full-text search across all translations with zero duplicates.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbVerse {
    pub translation: String,
    pub book: String,
    pub chapter: usize,
    pub verse: usize,
    pub text: String,
}

pub struct SqliteBibleStore {
    conn: Connection,
}

impl SqliteBibleStore {
    /// Open or create the unified SQLite Bible database.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;

             CREATE TABLE IF NOT EXISTS verses (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 translation TEXT NOT NULL,
                 book TEXT NOT NULL,
                 chapter INTEGER NOT NULL,
                 verse INTEGER NOT NULL,
                 text TEXT NOT NULL,
                 UNIQUE(translation, book, chapter, verse)
             );

             CREATE INDEX IF NOT EXISTS idx_verses_lookup 
             ON verses(translation, book, chapter, verse);

             CREATE VIRTUAL TABLE IF NOT EXISTS verses_fts USING fts5(
                 translation,
                 book,
                 chapter UNINDEXED,
                 verse UNINDEXED,
                 text,
                 content='verses',
                 content_rowid='id'
             );

             CREATE TRIGGER IF NOT EXISTS verses_ai AFTER INSERT ON verses BEGIN
                 INSERT INTO verses_fts(rowid, translation, book, chapter, verse, text)
                 VALUES (new.id, new.translation, new.book, new.chapter, new.verse, new.text);
             END;",
        )?;

        Ok(Self { conn })
    }

    /// Open an in-memory database for fast testing or ephemeral sessions.
    pub fn in_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    /// Insert a batch of verses within a single fast transaction.
    pub fn insert_verses(&mut self, verses: &[DbVerse]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut inserted = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO verses (translation, book, chapter, verse, text)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for v in verses {
                stmt.execute(params![
                    v.translation.to_uppercase(),
                    v.book,
                    v.chapter as i64,
                    v.verse as i64,
                    v.text.trim()
                ])?;
                inserted += 1;
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    /// Get a single verse with sub-millisecond index lookup.
    pub fn get_verse(
        &self,
        translation: &str,
        book: &str,
        chapter: usize,
        verse: usize,
    ) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT text FROM verses 
             WHERE translation = ?1 AND LOWER(book) = LOWER(?2) AND chapter = ?3 AND verse = ?4
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![
            translation.to_uppercase(),
            book,
            chapter as i64,
            verse as i64
        ])?;

        if let Some(row) = rows.next()? {
            let text: String = row.get(0)?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Read an entire chapter in ordered sequence.
    pub fn get_chapter(
        &self,
        translation: &str,
        book: &str,
        chapter: usize,
    ) -> Result<Vec<(usize, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT verse, text FROM verses 
             WHERE translation = ?1 AND LOWER(book) = LOWER(?2) AND chapter = ?3
             ORDER BY verse ASC",
        )?;

        let rows = stmt.query_map(
            params![translation.to_uppercase(), book, chapter as i64],
            |row| Ok((row.get::<_, i64>(0)? as usize, row.get::<_, String>(1)?)),
        )?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Perform full-text search across scriptures using BM25 relevance ranking.
    pub fn search_fts(
        &self,
        query: &str,
        translation_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DbVerse>> {
        let sql = if translation_filter.is_some() {
            "SELECT translation, book, chapter, verse, text FROM verses_fts 
             WHERE verses_fts MATCH ?1 AND translation = ?2
             ORDER BY rank LIMIT ?3"
        } else {
            "SELECT translation, book, chapter, verse, text FROM verses_fts 
             WHERE verses_fts MATCH ?1 
             ORDER BY rank LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(sql)?;

        let mut results = Vec::new();
        if let Some(trans) = translation_filter {
            let rows = stmt.query_map(params![query, trans.to_uppercase(), limit as i64], |row| {
                Ok(DbVerse {
                    translation: row.get(0)?,
                    book: row.get(1)?,
                    chapter: row.get::<_, i64>(2)? as usize,
                    verse: row.get::<_, i64>(3)? as usize,
                    text: row.get(4)?,
                })
            })?;
            for r in rows {
                results.push(r?);
            }
        } else {
            let rows = stmt.query_map(params![query, limit as i64], |row| {
                Ok(DbVerse {
                    translation: row.get(0)?,
                    book: row.get(1)?,
                    chapter: row.get::<_, i64>(2)? as usize,
                    verse: row.get::<_, i64>(3)? as usize,
                    text: row.get(4)?,
                })
            })?;
            for r in rows {
                results.push(r?);
            }
        }

        Ok(results)
    }

    /// Integrity check for database health.
    pub fn integrity_check(&self) -> Result<bool> {
        let mut stmt = self.conn.prepare("PRAGMA integrity_check;")?;
        let res: String = stmt.query_row([], |row| row.get(0))?;
        Ok(res.to_lowercase() == "ok")
    }

    /// Total verse count in database.
    pub fn total_verses(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM verses;", [], |r| r.get(0))
            .unwrap_or(0);
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_bible_store_operations() -> Result<()> {
        let mut store = SqliteBibleStore::in_memory()?;
        assert!(store.integrity_check()?);

        let sample_verses = vec![
            DbVerse {
                translation: "KJV".to_string(),
                book: "John".to_string(),
                chapter: 3,
                verse: 16,
                text: "For God so loved the world, that he gave his only begotten Son...".to_string(),
            },
            DbVerse {
                translation: "KJV".to_string(),
                book: "Genesis".to_string(),
                chapter: 1,
                verse: 1,
                text: "In the beginning God created the heaven and the earth.".to_string(),
            },
        ];

        let count = store.insert_verses(&sample_verses)?;
        assert_eq!(count, 2);
        assert_eq!(store.total_verses()?, 2);

        // Sub-millisecond lookup
        let text = store.get_verse("KJV", "John", 3, 16)?;
        assert!(text.is_some());
        assert!(text.unwrap().contains("begotten Son"));

        // FTS search
        let hits = store.search_fts("beginning", Some("KJV"), 5)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].book, "Genesis");

        Ok(())
    }
}
