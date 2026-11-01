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

    /// Check health using a fast 1-second timeout so offline server checks don't stall.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.server_url);
        let fast_client = Client::builder()
            .timeout(Duration::from_millis(800))
            .build()
            .unwrap_or_else(|_| self.client.clone());
        match fast_client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Auto-spawn background pocket-tts daemon server if not running.
    pub fn ensure_server_running(&self) {
        let port = self.server_url.rsplit(':').next().unwrap_or("8000").parse::<u16>().unwrap_or(8000);
        let cli_bin = self.resolve_cli_path();

        let resolved_voice = self.resolve_voice_path();
        let _ = Command::new(&cli_bin)
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--default-voice")
            .arg(&resolved_voice)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    /// Fast resolution of CLI binary path without running --help.
    pub fn resolve_cli_path(&self) -> String {
        let cli = self.cli_path.as_deref().unwrap_or("pocket-tts");
        let direct_path = std::path::Path::new(cli);
        if direct_path.is_file() {
            return cli.to_string();
        }
        let local_bin = crate::home_dir().join(".local/bin").join(cli);
        if local_bin.is_file() {
            return local_bin.to_string_lossy().to_string();
        }
        cli.to_string()
    }

    /// Resolve voice safetensors path.
    pub fn resolve_voice_path(&self) -> String {
        if self.voice.ends_with(".safetensors") || self.voice.ends_with(".wav") {
            let p = std::path::Path::new(&self.voice);
            if p.exists() {
                return self.voice.clone();
            }
            let global_path = crate::home_dir().join(".paraclea/voices").join(&self.voice);
            if global_path.exists() {
                return global_path.to_string_lossy().to_string();
            }
        }
        self.voice.clone()
    }

    /// Clean markdown formatting, bold/italics, and header tags for natural speech synthesis.
    pub fn clean_spoken_text(raw: &str) -> String {
        let mut clean = raw
            .replace("**", "")
            .replace('*', "")
            .replace("###", "")
            .replace("##", "")
            .replace(['#', '`', '^', '~'], "");

        // Remove bullet points at line starts
        clean = clean
            .lines()
            .map(|l| {
                let trimmed = l.trim_start();
                if let Some(stripped) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("• ")).or_else(|| trimmed.strip_prefix("* ")) {
                    stripped
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        clean.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Synthesize text into WAV audio bytes.
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let cleaned = Self::clean_spoken_text(text);
        if cleaned.trim().is_empty() {
            return Ok(Vec::new());
        }
        let text = cleaned.as_str();

        // 1. Attempt Pocket TTS HTTP server request
        let is_online = self.health_check().await;
        if is_online {
            let url = format!("{}/tts", self.server_url);

            // Pocket TTS HTTP API: voice_url is only accepted for URLs (http/https/hf) or builtin voice names.
            // When using Diana or custom safetensors file, the server was loaded with --default-voice, so omitting voice_url uses it directly.
            let mut params = vec![("text", text.to_string())];
            let is_url_or_builtin = self.voice.starts_with("http://")
                || self.voice.starts_with("https://")
                || self.voice.starts_with("hf://")
                || matches!(self.voice.as_str(), "alba" | "giovanni" | "lola" | "juergen" | "rafael" | "estelle");
            if is_url_or_builtin {
                params.push(("voice_url", self.voice.clone()));
            }

            if let Ok(resp) = self.client.post(&url).form(&params).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        return Ok(bytes.to_vec());
                    }
                }
            }
        } else {
            // Spawn server in background for subsequent sentences
            self.ensure_server_running();
        }

        // 2. Fallback to CLI execution with direct binary resolution and --quantize in spawn_blocking
        if self.cli_path.is_some() {
            let unique_out = crate::temp_dir().join(format!("paraclea_tts_{}_{}.wav", std::process::id(), uuid::Uuid::new_v4()));
            let resolved_voice = self.resolve_voice_path();
            let cli_cmd = self.resolve_cli_path();
            let text_owned = text.to_string();

            let run_res = tokio::task::spawn_blocking(move || {
                Command::new(&cli_cmd)
                    .arg("generate")
                    .arg("--text")
                    .arg(&text_owned)
                    .arg("--voice")
                    .arg(&resolved_voice)
                    .arg("--output-path")
                    .arg(&unique_out)
                    .arg("--quantize")
                    .arg("-q")
                    .output()
                    .map(|out| (out, unique_out))
            }).await;

            if let Ok(Ok((out, unique_out))) = run_res {
                if out.status.success() {
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
            }
        }

        anyhow::bail!("Pocket TTS synthesis unavailable.")
    }
}
