//! Background Heartbeat Self-Maintenance Module for Paraclea
//!
//! Periodically reviews interaction logs, condenses long-term memory,
//! and performs background reflection using Ollama.

use crate::ollama::{ChatMessage, OllamaClient};
use crate::persona::PersonaManager;
use crate::tools::ToolExecutor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct HeartbeatLoop {
    pub interval: Duration,
    pub persona: PersonaManager,
    pub ollama: OllamaClient,
    pub tools: ToolExecutor,
}

impl HeartbeatLoop {
    /// Initialize Heartbeat background task.
    pub fn new(interval_minutes: u64, persona: PersonaManager, ollama: OllamaClient) -> Self {
        let tools = ToolExecutor::new(persona.clone());
        Self {
            interval: Duration::from_secs(interval_minutes * 60),
            persona,
            ollama,
            tools,
        }
    }

    /// Run the background maintenance loop until shutdown signal.
    pub async fn run(self, shutdown: Arc<AtomicBool>) {
        info!(
            "Heartbeat background maintenance loop started (interval: {} mins)",
            self.interval.as_secs() / 60
        );
        let mut timer = tokio::time::interval(self.interval);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Consume initial tick at t=0 so maintenance pass doesn't fire immediately on app launch
        timer.tick().await;

        loop {
            timer.tick().await;
            if shutdown.load(Ordering::Relaxed) {
                info!("Heartbeat loop received shutdown signal");
                break;
            }

            if let Err(e) = self.perform_maintenance().await {
                error!("Heartbeat maintenance error: {}", e);
            }
        }
    }

    /// Execute a single self-maintenance pass.
    async fn perform_maintenance(&self) -> anyhow::Result<()> {
        let heartbeat_rules = self.persona.read_file_or_empty("HEARTBEAT.md");
        let today_log = self.persona.get_today_log();

        if today_log.trim().is_empty() {
            info!("Heartbeat: No activity in daily log to process.");
            return Ok(());
        }

        info!("Heartbeat: Running self-maintenance pass on interaction log...");

        let system_prompt = format!(
            "You are Paraclea undergoing periodic self-maintenance.\n\n\
            === HEARTBEAT PROTOCOL ===\n{}\n\n\
            === TODAY'S LOGS ===\n{}\n\n\
            === INSTRUCTIONS ===\n\
            Analyze the daily interaction logs. If new durable facts or user preferences are found,\n\
            output a tool call JSON block to update MEMORY.md (`memory_replace`) or SOUL.md (`soul_replace`).\n\
            Otherwise, respond with a brief status note.\n",
            heartbeat_rules, today_log
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Perform periodic self-maintenance pass.".to_string(),
            },
        ];

        let response = self.ollama.chat(messages).await?;

        if let Some(tool_call) = self.tools.parse_tool_call(&response) {
            info!("Heartbeat triggered tool invocation: {}", tool_call.tool);
            match self.tools.execute(&tool_call) {
                Ok(out) => info!("Heartbeat tool output: {}", out),
                Err(e) => error!("Heartbeat tool error: {}", e),
            }
        } else {
            info!("Heartbeat pass completed cleanly without memory modifications.");
        }

        Ok(())
    }
}
