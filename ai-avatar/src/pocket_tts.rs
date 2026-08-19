//! Pocket TTS Client Engine for Paraclea
//!
//! Provides text-to-speech synthesis using local Pocket TTS server (`http://localhost:8000/tts`)
//! with fallback to CLI invocation (`pocket-tts generate`).

use anyhow::{Context, Result};
use reqwest::Client;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct PocketTtsEngine {
    client: Client,
    pub server_url: String,
    pub voice: String,
    pub cli_path: Option<String>,
}

impl PocketTtsEngine {
    /// Initialize Pocket TTS engine with server URL and voice preset.
    pub fn new(server_url: &str, voice: &str, cli_path: Option<&str>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client for Pocket TTS")?;

        Ok(Self {
            client,
            server_url: server_url.trim_end_matches('/').to_string(),
            voice: voice.to_string(),
            cli_path: cli_path.map(|s| s.to_string()),
        })
    }

    /// Health check to verify if Pocket TTS HTTP server is active.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/docs", self.server_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Synthesize text into WAV audio bytes.
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        // 1. Attempt Pocket TTS HTTP server request
        if self.health_check().await {
            let url = format!("{}/tts", self.server_url);
            let params = [("text", text), ("voice_url", &self.voice)];

            debug!("Sending Pocket TTS HTTP request for text: '{}'", text);
            let resp = self
                .client
                .post(&url)
                .form(&params)
                .send()
                .await
                .with_context(|| format!("Pocket TTS HTTP request failed to {}", url))?;

            if resp.status().is_success() {
                let audio_bytes = resp.bytes().await?.to_vec();
                info!(
                    "Pocket TTS synthesized {} audio bytes via HTTP server",
                    audio_bytes.len()
                );
                return Ok(audio_bytes);
            }
        }

        // 2. Fallback to CLI execution if available
        if let Some(ref cli) = self.cli_path {
            info!("Pocket TTS HTTP server inactive, attempting CLI fallback: {}", cli);
            let output = Command::new(cli)
                .arg("generate")
                .arg("--text")
                .arg(text)
                .arg("--voice")
                .arg(&self.voice)
                .arg("-q")
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    let default_wav = std::path::Path::new("tts_output.wav");
                    if default_wav.exists() {
                        let bytes = std::fs::read(default_wav)?;
                        let _ = std::fs::remove_file(default_wav);
                        info!("Pocket TTS synthesized {} audio bytes via CLI", bytes.len());
                        return Ok(bytes);
                    }
                }
                Ok(out) => {
                    warn!(
                        "Pocket TTS CLI returned error code: {}",
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
                Err(e) => {
                    warn!("Failed to invoke Pocket TTS CLI binary: {}", e);
                }
            }
        }

        anyhow::bail!("Pocket TTS server (at {}) and CLI fallback are both unavailable.", self.server_url)
    }
}
