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

    /// Build comprehensive system prompt combining all persona files.
    pub fn build_system_prompt(&self) -> String {
        let identity = self.read_file_or_empty("IDENTITY.md");
        let soul = self.read_file_or_empty("SOUL.md");
        let user = self.read_file_or_empty("USER.md");
        let memory = self.read_file_or_empty("MEMORY.md");
        let tools = self.read_file_or_empty("TOOLS.md");

        format!(
            "You are Paraclea, a self-developing AI companion assistant.\n\n\
            === IDENTITY ===\n{}\n\n\
            === SOUL & BEHAVIOR ===\n{}\n\n\
            === USER PROFILE ===\n{}\n\n\
            === LONG-TERM MEMORY ===\n{}\n\n\
            === TOOLS & SELF-DEVELOPMENT ===\n{}\n\n\
            === INSTRUCTIONS ===\n\
            1. You are Paraclea, a smart, warm, attentive, helpful AI companion assistant.\n\
            2. To execute a tool, output a single JSON code block formatted exactly like this:\n\
               ```json\n\
               {{\"tool\": \"tool_name\", \"arguments\": {{ ... }}}}\n\
               ```\n\
            3. Available tools: soul_replace, memory_replace, persona_replace, daily_log_append, read_file, write_file, execute_command.\n\
            4. Be friendly, intelligent, concise, and helpful in conversation!\n",
            identity, soul, user, memory, tools
        )
    }

    /// Retrieve today's daily log content.
    pub fn get_today_log(&self) -> String {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        self.read_file_or_empty(&format!("logs/daily/{}.md", date_str))
    }
}
