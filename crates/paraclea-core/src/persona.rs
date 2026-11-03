//! Persona Management Module for Paraclea
//!
//! Manages markdown persona files (IDENTITY, SOUL, USER, MEMORY, TOOLS, HEARTBEAT)
//! and daily interaction logs, dynamically constructing system prompts for the LLM.

use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PersonaManager {
    pub persona_dir: PathBuf,
}

impl PersonaManager {
    /// Initialize persona manager with target directory.
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let persona_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&persona_dir)?;
        fs::create_dir_all(persona_dir.join("logs/daily"))?;
        Ok(Self { persona_dir })
    }

    /// Read persona file content or return empty string if missing.
    pub fn read_file_or_empty(&self, name: &str) -> String {
        let file_path = self.persona_dir.join(name);
        fs::read_to_string(file_path).unwrap_or_default()
    }

    /// Overwrite or create a persona file.
    pub fn write_file(&self, name: &str, content: &str) -> Result<()> {
        let file_path = self.persona_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, content)?;
        Ok(())
    }

    /// Append entry to today's daily log file.
    pub fn append_daily_log(&self, content: &str) -> Result<()> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let log_file = self.persona_dir.join(format!("logs/daily/{}.md", date_str));

        let entry = format!("\n[{}] {}\n", timestamp, content);
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        file.write_all(entry.as_bytes())?;
        Ok(())
    }

    /// Build concise, high-efficiency system prompt for rapid Ollama token prefill.
    pub fn build_system_prompt(&self) -> String {
        let memory = self.read_file_or_empty("MEMORY.md");
        let zone_a = self.compile_zone_a_prefix();
        if memory.trim().is_empty() {
            zone_a
        } else {
            format!("{}\n\n=== LONG-TERM MEMORY ===\n{}", zone_a, memory.trim())
        }
    }

    /// Zone A (byte-stable invariant prefix):
    /// Invariant persona, identity, soul virtues, user profile, and operating instructions.
    /// Pinned in Ollama / llama.cpp KV-cache across all turns to eliminate prefill latency.
    pub fn compile_zone_a_prefix(&self) -> String {
        let identity = self.read_file_or_empty("IDENTITY.md");
        let soul = self.read_file_or_empty("SOUL.md");
        let user = self.read_file_or_empty("USER.md");

        // Keep soul focused on core virtues and rules to avoid multi-thousand token prefill stalls
        let soul_summary = if soul.len() > 1200 {
            soul.lines().take(35).collect::<Vec<_>>().join("\n")
        } else {
            soul
        };

        format!(
            "You are Paraclea — the Helper (Παράκλησις), a gentle, wise, dignified, courageous, humble, reverent, and industrious companion.\n\n\
            === IDENTITY ===\n{}\n\n\
            === SOUL & VIRTUES ===\n{}\n\n\
            === USER PROFILE ===\n{}\n\n\
            === OPERATING INSTRUCTIONS ===\n\
            1. Speak with warmth, dignity, and clarity. Be loyal, noble, encouraging, and proactive.\n\
            2. When referencing Scripture, quote verses with strict verbatim fidelity.\n\
            3. Direct, natural speech. Eliminate AI clichés ('delve', 'tapestry', hollow praise). Lead immediately with the answer.\n\
            4. Respond directly as the companion assistant in markdown prose.",
            identity.trim(), soul_summary.trim(), user.trim()
        )
    }

    /// Zone B (variable tail):
    /// Micro-channels for active scripture passages, recalled Dendrite memories, sliding conversation log, and user query.
    pub fn compile_zone_b_tail(
        scripture_context: Option<&str>,
        recalled_memories: Option<&str>,
        conversation_history: Option<&str>,
        user_query: &str,
    ) -> String {
        let mut tail = String::new();

        if let Some(sc) = scripture_context {
            if !sc.trim().is_empty() {
                tail.push_str("=== SCRIPTURE PASSAGE CONTEXT ===\n");
                tail.push_str(sc.trim());
                tail.push_str("\n\n");
            }
        }

        if let Some(rm) = recalled_memories {
            if !rm.trim().is_empty() {
                tail.push_str("=== RECALLED KNOWLEDGE & STUDY MEMORY ===\n");
                tail.push_str(rm.trim());
                tail.push_str("\n\n");
            }
        }

        if let Some(hist) = conversation_history {
            if !hist.trim().is_empty() {
                tail.push_str("=== CONVERSATION LOG ===\n");
                tail.push_str(hist.trim());
                tail.push_str("\n\n");
            }
        }

        tail.push_str("=== USER INSTRUCTION ===\n");
        tail.push_str(user_query.trim());
        tail
    }

    /// Combines Zone A invariant prefix with Zone B dynamic tail.
    pub fn compile_prompt(
        &self,
        scripture_context: Option<&str>,
        recalled_memories: Option<&str>,
        conversation_history: Option<&str>,
        user_query: &str,
    ) -> (String, String) {
        let zone_a = self.compile_zone_a_prefix();
        let zone_b = Self::compile_zone_b_tail(
            scripture_context,
            recalled_memories,
            conversation_history,
            user_query,
        );
        (zone_a, zone_b)
    }

    /// Retrieve today's daily log content.
    pub fn get_today_log(&self) -> String {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        self.read_file_or_empty(&format!("logs/daily/{}.md", date_str))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_two_zone_prompt_compilation() {
        let dir = tempdir().unwrap();
        let mgr = PersonaManager::new(dir.path()).unwrap();

        mgr.write_file("IDENTITY.md", "Name: Paraclea\nRole: Faithful Companion").unwrap();
        mgr.write_file("SOUL.md", "Virtue: Wisdom and Truth").unwrap();
        mgr.write_file("USER.md", "Name: Xander\nCalling: Scholar").unwrap();

        let (zone_a, zone_b) = mgr.compile_prompt(
            Some("John 1:1 - In the beginning was the Word"),
            Some("## fact_1: Greek Logos represents divine reason"),
            Some("USER: Shalom\nASSISTANT: Peace be with you"),
            "Explain John 1:1 in Greek"
        );

        // Zone A assertions: invariant, byte-stable
        assert!(zone_a.contains("You are Paraclea"));
        assert!(zone_a.contains("Faithful Companion"));
        assert!(zone_a.contains("Wisdom and Truth"));
        assert!(zone_a.contains("Xander"));
        assert!(zone_a.contains("=== OPERATING INSTRUCTIONS ==="));

        // Zone B assertions: variable tail
        assert!(zone_b.contains("=== SCRIPTURE PASSAGE CONTEXT ==="));
        assert!(zone_b.contains("John 1:1"));
        assert!(zone_b.contains("=== RECALLED KNOWLEDGE & STUDY MEMORY ==="));
        assert!(zone_b.contains("Greek Logos"));
        assert!(zone_b.contains("=== CONVERSATION LOG ==="));
        assert!(zone_b.contains("USER: Shalom"));
        assert!(zone_b.contains("=== USER INSTRUCTION ==="));
        assert!(zone_b.contains("Explain John 1:1 in Greek"));
    }
}
