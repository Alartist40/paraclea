//! DENDRITE v2 — Asynchronous Background Reflection Worker for Paraclea.
//!
//! Analyzes recent user conversation turns out-of-band to distill key facts (L1),
//! study preferences, and procedural habits (L2), storing them into the Dendrite graph
//! and SQLite database without blocking live response generation.

use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::dendrite::graph::{Dendrite, NodeType};
use crate::dendrite::store::DendriteStore;
use crate::ollama::{ChatMessage, OllamaClient};

pub struct ReflectionWorker {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    ollama: OllamaClient,
}

impl ReflectionWorker {
    pub fn new(
        graph: Arc<Dendrite>,
        store: Option<Arc<DendriteStore>>,
        ollama: OllamaClient,
    ) -> Self {
        Self { graph, store, ollama }
    }

    /// Spawn reflection task on recent conversation history in a background Tokio task.
    pub fn spawn_reflection(&self, history: Vec<ChatMessage>) {
        if history.len() < 2 {
            return;
        }

        let graph = self.graph.clone();
        let store = self.store.clone();
        let ollama = self.ollama.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::do_reflection(graph, store, ollama, &history).await {
                info!("[dendrite reflection worker] background task error: {}", e);
            }
        });
    }

    async fn do_reflection(
        graph: Arc<Dendrite>,
        store: Option<Arc<DendriteStore>>,
        ollama: OllamaClient,
        history: &[ChatMessage],
    ) -> Result<(), String> {
        let mut transcript = String::new();
        for msg in history.iter().take(6) {
            if !msg.content.trim().is_empty() {
                transcript.push_str(&format!("{}: {}\n", msg.role, msg.content));
            }
        }

        if transcript.is_empty() {
            return Ok(());
        }

        let prompt = format!(
            "Analyze the conversation transcript below. Extract key user facts, study preferences, favorite topics, or procedural habits.\n\
             Return ONLY a short bullet list of extracted facts or study habits, preceded by '#fact' or '#procedure'.\n\n\
             TRANSCRIPT:\n{}\n\nFACTS & HABITS:",
            transcript
        );

        let system_prompt = "You are a memory curator. Summarize key user facts and study habits concisely.".to_string();

        let req_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let output = match ollama.chat_with_model("ministral-3:3b", req_messages).await {
            Ok(res) => res.trim().to_string(),
            Err(e) => return Err(e.to_string()),
        };

        if output.is_empty() {
            return Ok(());
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64;

        let node_id = format!("reflection_{}", timestamp);
        let title = format!("User Study Memory {}", timestamp);

        let node_type = if output.contains("#procedure") {
            NodeType::Procedure
        } else {
            NodeType::AtomicFact
        };

        let node = graph.upsert(
            &node_id,
            &title,
            &output,
            node_type,
            Some(vec!["#reflection".into(), "#user_habit".into()]),
        );

        if let Some(s) = store {
            let _ = s.save(&node);
        }

        info!("✓ [Dendrite Memory] Distilled & stored user reflection node: {}", node_id);
        Ok(())
    }
}
