//! Self-Development & System Tools Module for Paraclea
//!
//! Provides tool parsing and robust execution capabilities (updating memory/persona,
//! file system access, and command execution).

use crate::persona::PersonaManager;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

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

    /// Action verb mapping for CLI UI display.
    pub fn action_verb(&self, tool: &str) -> &'static str {
        match tool {
            "read_file" => "reading...",
            "write_file" | "soul_replace" | "memory_replace" | "persona_replace" => "editing...",
            "daily_log_append" => "logging...",
            "execute_command" => "running...",
            _ => "processing...",
        }
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

    /// Extract string argument checking key aliases.
    fn get_arg_str(&self, val: &Value, keys: &[&str]) -> Result<String> {
        if let Some(s) = val.as_str() {
            return Ok(s.to_string());
        }
        if let Some(obj) = val.as_object() {
            for key in keys {
                if let Some(v) = obj.get(*key) {
                    if let Some(s) = v.as_str() {
                        return Ok(s.to_string());
                    }
                    return Ok(v.to_string());
                }
            }
            if let Some((_, first_val)) = obj.iter().next() {
                if let Some(s) = first_val.as_str() {
                    return Ok(s.to_string());
                }
                return Ok(first_val.to_string());
            }
        }
        anyhow::bail!("Missing required argument (checked keys: {:?})", keys)
    }

    /// Execute specified ToolCall and return execution output summary.
    pub fn execute(&self, call: &ToolCall) -> Result<String> {
        match call.tool.as_str() {
            "soul_replace" => {
                let content = self.get_arg_str(&call.arguments, &["content", "text", "soul"])?;
                self.persona.write_file("SOUL.md", &content)?;
                Ok("Updated SOUL.md".to_string())
            }
            "memory_replace" => {
                let content = self.get_arg_str(&call.arguments, &["content", "text", "memory"])?;
                self.persona.write_file("MEMORY.md", &content)?;
                Ok("Updated MEMORY.md".to_string())
            }
            "persona_replace" => {
                let file = self.get_arg_str(&call.arguments, &["file", "filename", "path"])?;
                let content = self.get_arg_str(&call.arguments, &["content", "text"])?;
                self.persona.write_file(&file, &content)?;
                Ok(format!("Updated persona file '{}'", file))
            }
            "daily_log_append" => {
                let content = self.get_arg_str(
                    &call.arguments,
                    &["content", "entry", "text", "log", "message"],
                )?;
                self.persona.append_daily_log(&content)?;
                Ok("Appended entry to daily log".to_string())
            }
            "read_file" => {
                let path = self.get_arg_str(&call.arguments, &["path", "file", "filename"])?;
                let content = std::fs::read_to_string(&path)?;
                Ok(format!("File contents of {}:\n{}", path, content))
            }
            "write_file" => {
                let path = self.get_arg_str(&call.arguments, &["path", "file", "filename"])?;
                let content = self.get_arg_str(&call.arguments, &["content", "text"])?;
                std::fs::write(&path, &content)?;
                Ok(format!("Wrote content to {}", path))
            }
            "execute_command" => {
                let cmd = self.get_arg_str(&call.arguments, &["command", "cmd"])?;
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
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
