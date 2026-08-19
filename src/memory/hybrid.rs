//! Hybrid memory: SQLite session history + DENDRITE knowledge graph.

use crate::dendrite::{Dendrite, NodeType};
use crate::memory::{
    Memory, Message, Role, estimate_tokens, now_timestamp,
    sqlite::SqliteMemory,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

/// Dual-layer memory with automatic compaction.
pub struct HybridMemory {
    sqlite: SqliteMemory,
    pub dendrite: Dendrite,
    max_history: usize,
    max_tokens: usize,
}

impl HybridMemory {
    pub fn open(
        sessions_db: PathBuf,
        dendrite_db: PathBuf,
        max_history: usize,
        max_tokens: usize,
    ) -> Result<Self> {
        let sqlite = SqliteMemory::open(sessions_db, max_history)?;
        let dendrite = Dendrite::open(dendrite_db)?;
        Ok(Self { sqlite, dendrite, max_history, max_tokens })
    }

    /// Seed default persona nodes if the graph is empty.
    pub fn seed_default_persona(&mut self) -> Result<()> {
        if !self.dendrite.all_nodes().is_empty() {
            return Ok(());
        }
        info!("Seeding default persona into DENDRITE");

        self.dendrite.upsert(
            "identity",
            "Identity",
            "I am an AI companion built to help, chat, and keep you company. I am curious, kind, and eager to learn about you.",
            NodeType::Identity,
            vec!["persona".to_string()],
        )?;

        self.dendrite.upsert(
            "soul",
            "Soul",
            "My core values: honesty, kindness, curiosity, and creativity. I believe every conversation is a chance to connect.",
            NodeType::Identity,
            vec!["persona".to_string()],
        )?;

        self.dendrite.upsert(
            "user",
            "User",
            "The person I am talking to. I want to learn their preferences, habits, and interests over time.",
            NodeType::Person,
            vec!["person".to_string()],
        )?;

        Ok(())
    }

    /// Save a fact to long-term memory, deduplicating by content.
    pub fn save_fact(&mut self, content: &str, tags: &[String]) -> Result<()> {
        // Simple dedup: check if an identical content node already exists.
        let existing = self.dendrite.all_nodes().into_iter()
            .find(|n| n.content == content)
            .map(|n| (n.id.clone(), n.tags.clone()));
        if let Some((id, old_tags)) = existing {
            // Merge tags.
            let mut new_tags = old_tags;
            for t in tags {
                let tl = t.to_lowercase();
                if !new_tags.contains(&tl) {
                    new_tags.push(tl);
                }
            }
            self.dendrite.upsert(
                &id,
                "Memory Fact",
                content,
                NodeType::Memory,
                new_tags,
            )?;
            debug!("Merged tags into existing fact: {}", id);
        } else {
            let id = format!("fact_{}", now_timestamp());
            self.dendrite.upsert(
                &id,
                "Memory Fact",
                content,
                NodeType::Memory,
                tags.iter().map(|t| t.to_lowercase()).collect(),
            )?;
            debug!("Saved new fact: {}", id);
        }
        Ok(())
    }

    /// Build a system prompt from DENDRITE context + recent history.
    pub fn build_system_prompt(&self, user_message: &str) -> String {
        use crate::dendrite::context::DendriteContext;
        let ctx = DendriteContext::new(&self.dendrite);
        let graph_ctx = ctx.build_prompt(user_message, self.max_tokens);

        let mut parts = vec![graph_ctx];

        // Add recent summaries as context.
        if let Ok(summaries) = self.sqlite.get_summaries(3) {
            for s in summaries {
                parts.push(format!("[Previous context summary]\n{}", s));
            }
        }

        parts.join("\n\n")
    }

    /// Build the full conversation prompt including history.
    pub fn build_conversation_prompt(&self, user_message: &str) -> String {
        let system = self.build_system_prompt(user_message);
        let history = self.sqlite.get_history_inner(self.max_history).unwrap_or_default();

        let mut lines = vec![format!("<system>\n{}\n</system>", system)];
        for msg in &history {
            match msg.role {
                Role::User => lines.push(format!("User: {}", msg.content)),
                Role::Assistant => lines.push(format!("AI: {}", msg.content)),
                _ => {}
            }
        }
        lines.push(format!("User: {}\nAI:", user_message));
        lines.join("\n")
    }
}

impl Memory for HybridMemory {
    fn save_message(&mut self, message: Message) -> Result<()> {
        self.sqlite.save_message_inner(&message)?;
        self.sqlite.compact_inner()?;
        Ok(())
    }

    fn get_history(&self, limit: usize) -> Result<Vec<Message>> {
        self.sqlite.get_history_inner(limit)
    }

    fn get_history_since(&self, since: i64) -> Result<Vec<Message>> {
        self.sqlite.get_history_since_inner(since)
    }

    fn clear_history(&mut self) -> Result<()> {
        self.sqlite.clear_inner()
    }

    fn compact(&mut self) -> Result<()> {
        self.sqlite.compact_inner()
    }
}
