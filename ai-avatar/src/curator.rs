//! Curator — background memory consolidation.
//!
//! Periodically reviews recent conversation history and extracts durable
//! facts into the DENDRITE knowledge graph.

use crate::memory::{HybridMemory, Memory, now_timestamp};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Background task that extracts facts from conversation history.
pub struct Curator {
    interval: Duration,
    last_run: i64,
}

impl Curator {
    pub fn new(interval_minutes: u64) -> Self {
        Self {
            interval: Duration::from_secs(interval_minutes * 60),
            last_run: 0,
        }
    }

    /// Run the curator loop.  Pass in a closure that generates fact summaries
    /// from conversation text (e.g. an LLM call).
    pub async fn run<F>(
        &mut self,
        memory: Arc<Mutex<HybridMemory>>,
        mut extractor: F,
        shutdown: &std::sync::atomic::AtomicBool,
    ) where
        F: FnMut(&str) -> Result<Vec<(String, Vec<String>)>> + Send,
    {
        let mut interval = tokio::time::interval(self.interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                info!("Curator shutting down");
                return;
            }

            if let Err(e) = self.run_maintenance(&memory, &mut extractor).await {
                error!("Curator maintenance failed: {}", e);
            }
        }
    }

    async fn run_maintenance<F>(
        &mut self,
        memory: &Arc<Mutex<HybridMemory>>,
        extractor: &mut F,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<Vec<(String, Vec<String>)>>,
    {
        let mem = memory.lock().await;

        // Fetch messages since last run.
        let messages = mem.get_history_since(self.last_run)?;
        drop(mem);

        if messages.len() < 2 {
            debug!("Not enough new messages for curation");
            return Ok(());
        }

        // Format conversation for extraction.
        let conversation: String = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        info!("Curator reviewing {} messages", messages.len());

        match extractor(&conversation) {
            Ok(facts) if !facts.is_empty() => {
                let mut mem = memory.lock().await;
                for (content, tags) in &facts {
                    if let Err(e) = mem.save_fact(content, tags) {
                        warn!("Failed to save fact: {}", e);
                    }
                }
                info!("Curator saved {} facts", facts.len());
            }
            Ok(_) => {
                debug!("Curator found no facts to extract");
            }
            Err(e) => {
                error!("Fact extraction failed: {}", e);
            }
        }

        self.last_run = now_timestamp();
        Ok(())
    }
}

/// A simple rule-based extractor that pulls key facts without an LLM.
/// Useful as a fallback when the LLM is unavailable.
pub fn rule_based_extractor(conversation: &str) -> Result<Vec<(String, Vec<String>)>> {
    let mut facts = Vec::new();

    // Pattern: "I like/love/enjoy/hate/prefer X"
    let re = regex::Regex::new(r"(?i)(i\s+(like|love|enjoy|hate|prefer|dislike)\s+(.+?))[\.\n!]")?;
    for cap in re.captures_iter(conversation) {
        if let Some(m) = cap.get(0) {
            facts.push((
                m.as_str().to_string(),
                vec!["preference".to_string()],
            ));
        }
    }

    // Pattern: "My name is X" / "I am X" / "Call me X"
    let re2 = regex::Regex::new(r"(?i)(my name is|i am|call me)\s+([A-Z][a-zA-Z\s]+)[\.\n!]")?;
    for cap in re2.captures_iter(conversation) {
        if let Some(m) = cap.get(0) {
            facts.push((
                m.as_str().to_string(),
                vec!["identity".to_string()],
            ));
        }
    }

    Ok(facts)
}
