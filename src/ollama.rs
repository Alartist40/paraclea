//! Ollama Local LLM Client & Model Management for Paraclea
//!
//! Async HTTP client for Ollama API (`http://127.0.0.1:11434/api/chat`).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub id: usize,
    pub name: String,
    pub backend: String,
    pub target: String,
}

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

    /// Fetch all available models from Ollama and local models directory.
    pub async fn fetch_available_models(&self) -> Vec<ModelEntry> {
        let mut list = Vec::new();
        let mut id = 1;

        let url = format!("{}/api/tags", self.endpoint);
        if let Ok(resp) = self.client.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(val) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = val["models"].as_array() {
                        for m in models {
                            if let Some(name) = m["name"].as_str() {
                                list.push(ModelEntry {
                                    id,
                                    name: name.to_string(),
                                    backend: "ollama".to_string(),
                                    target: name.to_string(),
                                });
                                id += 1;
                            }
                        }
                    }
                }
            }
        }

        // Scan local models directory for GGUF files
        let models_dir = Path::new("models");
        if models_dir.exists() && models_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(models_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "gguf" {
                                let filename = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                list.push(ModelEntry {
                                    id,
                                    name: filename,
                                    backend: "local".to_string(),
                                    target: path.to_string_lossy().to_string(),
                                });
                                id += 1;
                            }
                        }
                    }
                }
            }
        }

        list
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
