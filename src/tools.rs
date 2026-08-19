//! Self-Development & System Tools Module for Paraclea
//!
//! Provides tool parsing and execution capabilities (updating memory/persona,
//! file system access, and command execution).

use crate::persona::PersonaManager;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ToolExecutor {
    pub persona: PersonaManager,
}

impl ToolExecutor {
    /// Create new ToolExecutor linked to PersonaManager.
    pub fn new(persona: PersonaManager) -> Self {
        Self { persona }
    }

    /// Extract tool call JSON block from response text if present.
    pub fn parse_tool_call(&self, text: &str) -> Option<ToolCall> {
        let json_re = Regex::new(r"(?s)```(?:json)?\s*(\{\s*[\x22']tool[\x22'].*?\})\s*```").ok()?;
        if let Some(caps) = json_re.captures(text) {
            if let Some(json_str) = caps.get(1) {
                if let Ok(tc) = serde_json::from_str::<ToolCall>(json_str.as_str()) {
                    return Some(tc);
                }
            }
        }

        // Direct raw JSON attempt
        if let Ok(tc) = serde_json::from_str::<ToolCall>(text.trim()) {
            return Some(tc);
        }

        None
    }

    /// Execute specified ToolCall and return execution output summary.
    pub fn execute(&self, call: &ToolCall) -> Result<String> {
        info!("Tool invocation: {}", call.tool);
        match call.tool.as_str() {
            "soul_replace" => {
                let content = call.arguments["content"]
                    .as_str()
                    .context("Missing 'content' argument")?;
                self.persona.write_file("SOUL.md", content)?;
                Ok("Successfully updated SOUL.md".to_string())
            }
            "memory_replace" => {
                let content = call.arguments["content"]
                    .as_str()
                    .context("Missing 'content' argument")?;
                self.persona.write_file("MEMORY.md", content)?;
                Ok("Successfully updated MEMORY.md".to_string())
            }
            "persona_replace" => {
                let file = call.arguments["file"]
                    .as_str()
                    .context("Missing 'file' argument")?;
                let content = call.arguments["content"]
                    .as_str()
                    .context("Missing 'content' argument")?;
                self.persona.write_file(file, content)?;
                Ok(format!("Successfully updated persona file '{}'", file))
            }
            "daily_log_append" => {
                let content = call.arguments["content"]
                    .as_str()
                    .context("Missing 'content' argument")?;
                self.persona.append_daily_log(content)?;
                Ok("Appended entry to daily log".to_string())
            }
            "read_file" => {
                let path = call.arguments["path"]
                    .as_str()
                    .context("Missing 'path' argument")?;
                let content = std::fs::read_to_string(path)?;
                Ok(format!("File contents of {}:\n{}", path, content))
            }
            "write_file" => {
                let path = call.arguments["path"]
                    .as_str()
                    .context("Missing 'path' argument")?;
                let content = call.arguments["content"]
                    .as_str()
                    .context("Missing 'content' argument")?;
                std::fs::write(path, content)?;
                Ok(format!("Successfully wrote content to {}", path))
            }
            "execute_command" => {
                let cmd = call.arguments["command"]
                    .as_str()
                    .context("Missing 'command' argument")?;
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .with_context(|| format!("Failed to execute command: {}", cmd))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(format!(
                    "Command exit status {}:\nSTDOUT:\n{}\nSTDERR:\n{}",
                    output.status, stdout, stderr
                ))
            }
            unknown => anyhow::bail!("Unknown tool: {}", unknown),
        }
    }
}
