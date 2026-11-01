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
        let identity = self.read_file_or_empty("IDENTITY.md");
        let soul = self.read_file_or_empty("SOUL.md");
        let user = self.read_file_or_empty("USER.md");
        let memory = self.read_file_or_empty("MEMORY.md");

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
            === LONG-TERM MEMORY ===\n{}\n\n\
            === INSTRUCTIONS ===\n\
            1. Speak with warmth, dignity, and clarity. Be loyal, noble, encouraging, and proactive.\n\
            2. When referencing Scripture, quote verses with strict verbatim fidelity.\n\
            3. Direct, natural speech. Eliminate AI clichés ('delve', 'tapestry', hollow praise). Lead immediately with the answer.\n\
            4. Respond directly as the companion assistant in markdown prose.\n",
            identity.trim(), soul_summary.trim(), user.trim(), memory.trim()
        )
    }

    /// Retrieve today's daily log content.
    pub fn get_today_log(&self) -> String {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        self.read_file_or_empty(&format!("logs/daily/{}.md", date_str))
    }
}
