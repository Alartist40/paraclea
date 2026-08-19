//! HTTP client for the LeafcutterLLM inference API with caching & streaming.

use crate::cache::ResponseCache;
use crate::config::LlmConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// DTO for the Rust server (`/generate`).
#[derive(Serialize)]
struct RustGenerateRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(default)]
    stream: bool,
}

#[derive(Deserialize)]
struct RustGenerateResponse {
    text: String,
    #[serde(default)]
    tokens: Vec<usize>,
    took_ms: u64,
}

/// DTO for the Go server (`/generate`).
#[derive(Serialize)]
struct GoGenerateRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(default)]
    stream: bool,
}

#[derive(Deserialize)]
struct GoGenerateResponse {
    id: String,
    #[serde(default)]
    tokens: Vec<i64>,
    took_ms: i64,
    #[serde(default)]
    error: String,
}

/// Client for the LeafcutterLLM HTTP API.
pub struct LlmClient {
    client: Client,
    config: LlmConfig,
    cache: Option<ResponseCache>,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            client,
            config,
            cache: None,
        })
    }

    pub fn with_cache(mut self, cache: ResponseCache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Send a prompt and return the generated text.
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        // Check cache first.
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(prompt) {
                info!("LLM cache hit");
                return Ok(cached);
            }
        }

        let url = format!("{}/generate", self.config.endpoint);
        debug!("LLM request → {}", url);

        let response = match self.config.api_flavour.as_str() {
            "rust" => self.call_rust(&url, prompt).await,
            "go" => self.call_go(&url, prompt).await,
            other => {
                warn!("Unknown LLM api_flavour '{}', trying Rust flavour", other);
                self.call_rust(&url, prompt).await
            }
        };

        // Store in cache on success.
        if let Ok(ref text) = response {
            if let Some(ref cache) = self.cache {
                cache.put(prompt, text.clone());
            }
        }

        response
    }

    /// Stream tokens from the LLM as they are generated.
    /// Currently falls back to yielding the full response at once
    /// until the server supports true streaming.
    pub async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<impl futures_util::Stream<Item = Result<String>> + '_> {
        // For now, call the regular endpoint and yield the full text.
        // When the server supports SSE/NDJSON streaming, replace this
        // with an actual streaming HTTP request.
        let text = self.generate(prompt).await?;

        // Simulate streaming by yielding word-by-word.
        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| w.to_string() + " ")
            .collect();

        let stream = futures_util::stream::iter(words.into_iter().map(Ok));
        Ok(stream)
    }

    /// Quick health check.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => {
                debug!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn call_rust(&self, url: &str, prompt: &str) -> Result<String> {
        let req = RustGenerateRequest {
            prompt: prompt.to_string(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            top_p: Some(self.config.top_p),
            stream: false,
        };

        let resp = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("LLM returned {}: {}", status, body);
        }

        let body: RustGenerateResponse = resp
            .json()
            .await
            .context("failed to parse LLM JSON response")?;

        info!(
            "LLM generated {} tokens in {} ms",
            body.tokens.len(),
            body.took_ms
        );
        Ok(body.text)
    }

    async fn call_go(&self, url: &str, prompt: &str) -> Result<String> {
        let req = GoGenerateRequest {
            prompt: prompt.to_string(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: false,
        };

        let resp = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("LLM returned {}: {}", status, body);
        }

        let body: GoGenerateResponse = resp
            .json()
            .await
            .context("failed to parse LLM JSON response")?;

        if !body.error.is_empty() {
            anyhow::bail!("LLM engine error: {}", body.error);
        }

        // The Go server returns raw token IDs.  If we had a tokenizer we
        // could decode them; for now we return a placeholder so the
        // pipeline doesn't break.
        info!(
            "LLM generated {} tokens in {} ms (Go flavour — no text decode yet)",
            body.tokens.len(),
            body.took_ms
        );
        Ok(format!(
            "[Go server returned {} tokens — decode not yet implemented]",
            body.tokens.len()
        ))
    }
}
