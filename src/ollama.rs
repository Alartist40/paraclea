//! Ollama Local LLM Client, Vision OCR & Model Registry for Paraclea
//!
//! Async HTTP client for Ollama API (`/api/chat`, `/api/generate`, `/api/embeddings`).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARSET[((triple >> 18) & 63) as usize] as char);
        result.push(CHARSET[((triple >> 12) & 63) as usize] as char);
        if i + 1 < data.len() {
            result.push(CHARSET[((triple >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < data.len() {
            result.push(CHARSET[(triple & 63) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }
    result
}

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub id: usize,
    pub name: String,
    pub backend: String,
    pub target: String,
}

#[derive(Debug, Default, Clone)]
pub struct ModelRegistry {
    pub embed: Option<String>,
    pub chat: Option<String>,
    pub heavy: Option<String>,
    pub ocr: Option<String>,
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
            .timeout(Duration::from_secs(180))
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

    /// Discover available Ollama models categorized by role.
    pub async fn discover_models(&self) -> ModelRegistry {
        let mut reg = ModelRegistry::default();
        let available = self.fetch_available_models().await;

        for m in available {
            let name = m.name.to_lowercase();
            if (name.contains("embed") || name.contains("nomic")) && reg.embed.is_none() {
                reg.embed = Some(m.name.clone());
            } else if (name.contains("ocr") || name.contains("vision") || name.contains("llava")) && reg.ocr.is_none() {
                reg.ocr = Some(m.name.clone());
            } else if (name.contains("ornith") || name.contains("qwen3") || name.contains("deepseek")) && reg.heavy.is_none() {
                reg.heavy = Some(m.name.clone());
            } else if (name.contains("ministral") || name.contains("llama3.2") || name.contains("phi")) && reg.chat.is_none() {
                reg.chat = Some(m.name.clone());
            }
        }

        reg
    }

    /// Check missing recommended model types.
    pub fn check_missing(&self, reg: &ModelRegistry) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if reg.embed.is_none() { missing.push("nomic-embed-text (Embedding)"); }
        if reg.chat.is_none() { missing.push("ministral-3:3b (Default Chat)"); }
        if reg.heavy.is_none() { missing.push("qwen3:8b / ornith:9b (Heavy Reasoning)"); }
        if reg.ocr.is_none() { missing.push("frob/unlimited-ocr:q8_0 (Document Vision OCR)"); }
        missing
    }

    /// Perform vision OCR text extraction on an image using Base64 encoding.
    pub async fn ocr_image(&self, image_path: &Path, ocr_model: &str) -> Result<String> {
        let image_bytes = fs::read(image_path)
            .with_context(|| format!("Failed to read image file: {:?}", image_path))?;

        let b64 = base64_encode(&image_bytes);
        let url = format!("{}/api/generate", self.endpoint);
        let body = serde_json::json!({
            "model": ocr_model,
            "prompt": "Convert this document page into clean structured markdown. Preserve headings, lists, tables, and paragraphs.",
            "images": [b64],
            "stream": false,
            "options": {
                "temperature": 0.0
            }
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama Vision OCR failed: {}", err);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let extracted_text = res_json["response"].as_str().unwrap_or("").trim().to_string();
        Ok(extracted_text)
    }

    /// Generate text embedding vector using specified embedding model.
    pub async fn embed(&self, text: &str, embed_model: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.endpoint);
        let body = serde_json::json!({
            "model": embed_model,
            "prompt": text
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send embedding request to {}", url))?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama embedding request failed: {}", err);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let embedding = res_json["embedding"]
            .as_array()
            .context("No embedding array in Ollama response")?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
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

    /// Send chat request with target model name.
    pub async fn chat_with_model(&self, model_name: &str, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/api/chat", self.endpoint);
        let req = OllamaChatRequest {
            model: model_name.to_string(),
            messages,
            stream: false,
            temperature: Some(0.3),
        };

        debug!("Sending chat request to Ollama model '{}'", model_name);
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

    /// Send full chat request to Ollama using default model.
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        self.chat_with_model(&self.model, messages).await
    }
}
