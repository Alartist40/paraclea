//! Pocket TTS Client Engine for Paraclea
//!
//! Provides text-to-speech synthesis using local Pocket TTS server (`http://localhost:8000/tts`)
//! with fallback to CLI invocation (`pocket-tts generate`).

use anyhow::{Context, Result};
use reqwest::Client;
use std::process::Command;
use std::time::Duration;

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

    /// Resolve voice file path if local or in ~/.paraclea/voices.
    pub fn resolve_voice_path(&self) -> String {
        if self.voice.ends_with(".safetensors") || self.voice.ends_with(".wav") {
            let p = std::path::Path::new(&self.voice);
            if p.exists() {
                self.voice.clone()
            } else {
                let global_path = crate::home_dir().join(".paraclea/voices").join(&self.voice);
                if global_path.exists() {
                    global_path.to_string_lossy().to_string()
                } else {
                    self.voice.clone()
                }
            }
        } else {
            self.voice.clone()
        }
    }

    /// Clean markdown formatting and special characters before speech synthesis.
    pub fn clean_for_speech(text: &str) -> String {
        let mut clean = String::with_capacity(text.len());
        let mut in_parenthesis_link = false;

        for ch in text.chars() {
            match ch {
                '*' | '#' | '_' | '`' | '~' | '>' | '[' | ']' | '{' | '}' | '|' => continue,
                '(' if in_parenthesis_link => continue,
                ')' if in_parenthesis_link => {
                    in_parenthesis_link = false;
                    continue;
                }
                c if c.is_control() && c != '\n' && c != ' ' => continue,
                c => clean.push(c),
            }
        }
        clean.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Ensure the Pocket TTS HTTP daemon is running in background for fast sentence generation.
    pub async fn ensure_server_running(&self) {
        if self.health_check().await {
            return;
        }

        if let Some(ref cli) = self.cli_path {
            let cli_cmd = if Command::new(cli).arg("--help").output().is_ok() {
                cli.to_string()
            } else {
                let local_bin = crate::home_dir().join(".local/bin").join(cli);
                if local_bin.exists() {
                    local_bin.to_string_lossy().to_string()
                } else {
                    cli.to_string()
                }
            };

            let resolved_voice = self.resolve_voice_path();
            let _ = Command::new(&cli_cmd)
                .arg("serve")
                .arg("--port")
                .arg("8000")
                .arg("--default-voice")
                .arg(&resolved_voice)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            // Wait briefly for server ready
            for _ in 0..15 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                if self.health_check().await {
                    break;
                }
            }
        }
    }

    /// Synthesize text into WAV audio bytes.
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let cleaned = Self::clean_for_speech(text);
        if cleaned.trim().is_empty() {
            return Ok(Vec::new());
        }
        let text = &cleaned;

        // 1. Attempt Pocket TTS HTTP server request (auto-spawning daemon if needed)
        self.ensure_server_running().await;
        if self.health_check().await {
            let url = format!("{}/tts", self.server_url);
            let params = [("text", text), ("voice_url", &self.voice)];

            let resp = self
                .client
                .post(&url)
                .form(&params)
                .send()
                .await

                .with_context(|| format!("Pocket TTS HTTP request failed to {}", url))?;

            if resp.status().is_success() {
                let audio_bytes = resp.bytes().await?.to_vec();
                return Ok(audio_bytes);
            }
        }

        // 2. Fallback to CLI execution if available
        if let Some(ref cli) = self.cli_path {
            let unique_out = crate::temp_dir().join(format!("paraclea_tts_{}_{}.wav", std::process::id(), uuid::Uuid::new_v4()));
            let resolved_voice = self.resolve_voice_path();

            let cli_cmd = if Command::new(cli).arg("--help").output().is_ok() {
                cli.to_string()
            } else {
                let local_bin = crate::home_dir().join(".local/bin").join(cli);
                if local_bin.exists() {
                    local_bin.to_string_lossy().to_string()
                } else {
                    cli.to_string()
                }
            };

            let output = Command::new(&cli_cmd)
                .arg("generate")
                .arg("--text")
                .arg(text)
                .arg("--voice")
                .arg(&resolved_voice)
                .arg("--output-path")
                .arg(&unique_out)
                .arg("-q")
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    let fallback_default = std::path::Path::new("tts_output.wav");
                    if unique_out.exists() {
                        let bytes = std::fs::read(&unique_out)?;
                        let _ = std::fs::remove_file(&unique_out);
                        return Ok(bytes);
                    } else if fallback_default.exists() {
                        let bytes = std::fs::read(fallback_default)?;
                        let _ = std::fs::remove_file(fallback_default);
                        return Ok(bytes);
                    }
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    anyhow::bail!("Pocket TTS CLI error: {}", err);
                }
                Err(e) => {
                    anyhow::bail!("Failed to execute '{}': {}", cli_cmd, e);
                }
            }
        }

        anyhow::bail!("Pocket TTS synthesis unavailable.")
    }
}
