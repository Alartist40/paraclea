//! Configuration Module for Paraclea
//!
//! Manages loading and saving YAML configuration settings.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub system: SystemConfig,
    pub model: ModelConfig,
    #[serde(default)]
    pub vector_db: VectorDbConfig,
    pub voice: VoiceConfig,
    pub persona: PersonaConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub backend: String,
    pub ollama: OllamaConfig,
    pub local: LocalConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
    #[serde(default = "default_heavy_model")]
    pub heavy_model: String,
    #[serde(default = "default_embed_model")]
    pub embed_model: String,
    #[serde(default = "default_ocr_model")]
    pub ocr_model: String,
}

fn default_heavy_model() -> String {
    "qwen3:8b".to_string()
}

fn default_embed_model() -> String {
    "nomic-embed-text".to_string()
}

fn default_ocr_model() -> String {
    "frob/unlimited-ocr:q8_0".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorDbConfig {
    pub qdrant_url: String,
    pub collection_bible: String,
    pub collection_books: String,
    pub collection_survival: String,
}

impl Default for VectorDbConfig {
    fn default() -> Self {
        Self {
            qdrant_url: "http://localhost:6333".to_string(),
            collection_bible: "bible".to_string(),
            collection_books: "books".to_string(),
            collection_survival: "survival".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceConfig {
    pub pocket_tts_url: String,
    pub pocket_tts_voice: String,
    pub pocket_tts_cli: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonaConfig {
    pub dir: String,
    pub heartbeat_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                name: "Paraclea".to_string(),
                version: "0.1.0".to_string(),
            },
            model: ModelConfig {
                backend: "ollama".to_string(),
                ollama: OllamaConfig {
                    url: "http://localhost:11434".to_string(),
                    model: "ministral-3:3b".to_string(),
                    heavy_model: "qwen3:8b".to_string(),
                    embed_model: "nomic-embed-text".to_string(),
                    ocr_model: "frob/unlimited-ocr:q8_0".to_string(),
                },
                local: LocalConfig {
                    path: "models".to_string(),
                },
            },
            vector_db: VectorDbConfig::default(),
            voice: VoiceConfig {
                pocket_tts_url: "http://localhost:8000".to_string(),
                pocket_tts_voice: "alba".to_string(),
                pocket_tts_cli: "/home/xander/Documents/reference/pocket-tts/.venv/bin/pocket-tts".to_string(),
            },
            persona: PersonaConfig {
                dir: "persona".to_string(),
                heartbeat_interval: 15,
            },
        }
    }
}

impl Config {
    /// Load config from file or return default.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file at {:?}", path.as_ref()))?;
        let cfg: Config = serde_yaml::from_str(&content).unwrap_or_else(|_| Self::default());
        Ok(cfg)
    }

    /// Save config to YAML file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            let _ = fs::create_dir_all(parent);
        }
        let yaml_str = serde_yaml::to_string(self)?;
        fs::write(path, yaml_str)?;
        Ok(())
    }

    /// Determine config path from current directory or user home.
    pub fn find_or_default_config_path() -> PathBuf {
        let cwd_cfg = PathBuf::from("config.yaml");
        if cwd_cfg.exists() {
            return cwd_cfg;
        }
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            let user_cfg = home.join(".config/paraclea/config.yaml");
            if user_cfg.exists() {
                return user_cfg;
            }
        }
        cwd_cfg
    }
}
