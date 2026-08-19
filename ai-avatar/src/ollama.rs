//! Ollama Local LLM Client Module for Paraclea
//!
//! Provides async client for Ollama API (`http://127.0.0.1:11434/api/chat`).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct OllamaChatResponse {
    pub message: ChatMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: u64,
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: Client,
    pub endpoint: String,
    pub model: String,
}

impl OllamaClient {
    /// Create new Ollama client.
    pub fn new(endpoint: &str, model: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client for Ollama")?;

        Ok(Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
        })
    }

    /// Health check to verify Ollama server connectivity.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => {
                debug!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// List available installed Ollama models.
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.endpoint);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(Vec::new());
        }
        #[derive(Deserialize)]
        struct TagsResp {
            models: Vec<ModelItem>,
        }
        #[derive(Deserialize)]
        struct ModelItem {
            name: String,
        }
        let tags: TagsResp = resp.json().await?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Send full chat request to Ollama and return assistant response text.
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/api/chat", self.endpoint);
        let req = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            temperature: Some(0.7),
        };

        debug!("Sending chat request to Ollama model '{}'", self.model);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .with_context(|| format!("Failed to reach Ollama at {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama HTTP {}: {}", status, body);
        }

        let body: OllamaChatResponse = resp
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        info!(
            "Ollama response received (took {} ms)",
            body.total_duration / 1_000_000
        );
        Ok(body.message.content)
    }
}
