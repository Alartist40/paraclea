//! Configuration management for ai-avatar

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Audio capture and playback settings.
    pub audio: AudioConfig,
    /// Speech-to-text engine settings.
    pub stt: SttConfig,
    /// LLM API client settings.
    pub llm: LlmConfig,
    /// Text-to-speech engine settings.
    pub tts: TtsConfig,
    /// Emotion and sentiment pipeline settings.
    pub emotion: EmotionConfig,
    /// Memory system settings.
    pub memory: MemoryConfig,
    /// Response cache settings.
    pub cache: CacheConfig,
    /// Safety monitor settings.
    pub safety: SafetyConfig,
    /// Curator background task settings.
    pub curator: CuratorConfig,
    /// General application settings.
    pub app: AppConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            stt: SttConfig::default(),
            llm: LlmConfig::default(),
            tts: TtsConfig::default(),
            emotion: EmotionConfig::default(),
            memory: MemoryConfig::default(),
            cache: CacheConfig::default(),
            safety: SafetyConfig::default(),
            curator: CuratorConfig::default(),
            app: AppConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from `config.toml` in the current directory,
    /// falling back to defaults if the file is missing or fields are omitted.
    pub fn load() -> Result<Self> {
        let path = PathBuf::from("config.toml");
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let config: Config = toml::from_str(&text)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            Ok(config)
        } else {
            tracing::warn!("config.toml not found, using defaults");
            Ok(Config::default())
        }
    }
}

/// Audio I/O configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Target sample rate for the pipeline (Hz).
    /// Whisper expects 16 kHz.
    pub sample_rate: u32,
    /// Number of channels to capture (1 = mono).
    pub channels: u16,
    /// Size of the audio ring buffer in samples.
    pub ring_buffer_samples: usize,
    /// Size of each VAD frame in milliseconds.
    pub vad_frame_ms: usize,
    /// RMS energy threshold for voice detection (0.0 - 1.0).
    pub vad_energy_threshold: f32,
    /// Number of consecutive silent frames before speech segment ends.
    pub vad_silence_frames: usize,
    /// Number of consecutive loud frames before speech segment starts.
    pub vad_speech_frames: usize,
    /// Pre-roll: keep this many frames before speech start.
    pub vad_pre_roll_frames: usize,
    /// Post-roll: keep this many frames after speech end.
    pub vad_post_roll_frames: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16_000,
            channels: 1,
            ring_buffer_samples: 16_000 * 10, // 10 seconds @ 16kHz
            vad_frame_ms: 30,
            vad_energy_threshold: 0.015,
            vad_silence_frames: 15,
            vad_speech_frames: 3,
            vad_pre_roll_frames: 5,
            vad_post_roll_frames: 10,
        }
    }
}

/// STT engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    /// Path to the Whisper model file (e.g. `ggml-tiny.bin`).
    pub model_path: PathBuf,
    /// Language code (e.g. "en", "auto" for auto-detect).
    pub language: String,
    /// Use the mock STT engine instead of Whisper.
    pub use_mock: bool,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/ggml-tiny.bin"),
            language: "en".to_string(),
            use_mock: false,
        }
    }
}

/// LLM client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Base URL for the LeafcutterLLM HTTP API.
    pub endpoint: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Default max tokens per generation.
    pub max_tokens: usize,
    /// Sampling temperature.
    pub temperature: f32,
    /// Top-p sampling.
    pub top_p: f32,
    /// API flavour: `"rust"` for the Axum server (port 8081) or
    /// `"go"` for the Go scheduler server (port 8080).
    pub api_flavour: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8081".to_string(),
            timeout_secs: 120,
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            api_flavour: "rust".to_string(),
        }
    }
}

/// TTS engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Path to the Piper ONNX model.
    pub model_path: PathBuf,
    /// Path to the Piper model JSON config.
    pub model_json_path: PathBuf,
    /// Use the mock TTS engine instead of Piper.
    pub use_mock: bool,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/en_US-lessac-medium.onnx"),
            model_json_path: PathBuf::from("models/en_US-lessac-medium.onnx.json"),
            use_mock: false,
        }
    }
}

/// Emotion pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionConfig {
    /// Smoothing factor for emotion interpolation (0.0 = instant, 1.0 = never changes).
    pub blend_factor: f32,
    /// Enable debug printing of emotion states.
    pub debug: bool,
}

impl Default for EmotionConfig {
    fn default() -> Self {
        Self {
            blend_factor: 0.3,
            debug: true,
        }
    }
}

/// Memory system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to the SQLite sessions database.
    pub sessions_db: PathBuf,
    /// Path to the DENDRITE knowledge graph database.
    pub dendrite_db: PathBuf,
    /// Max recent messages to keep in active context.
    pub max_history: usize,
    /// Max tokens for DENDRITE context assembly.
    pub max_context_tokens: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            sessions_db: PathBuf::from("data/sessions.db"),
            dendrite_db: PathBuf::from("data/dendrite.db"),
            max_history: 20,
            max_context_tokens: 6000,
        }
    }
}

/// Response cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Max cached responses.
    pub capacity: usize,
    /// Enable caching.
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            enabled: true,
        }
    }
}

/// Safety monitor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// RAM usage threshold before emergency shutdown (percent).
    pub ram_threshold: f32,
    /// Enable the safety monitor.
    pub enabled: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            ram_threshold: 92.0,
            enabled: true,
        }
    }
}

/// Curator background task configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratorConfig {
    /// Interval between curation runs (minutes).
    pub interval_minutes: u64,
    /// Enable the curator.
    pub enabled: bool,
}

impl Default for CuratorConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 10,
            enabled: true,
        }
    }
}

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Log filter directive (e.g. `info,ai_avatar=debug`).
    pub log_filter: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_filter: "info,ai_avatar=debug".to_string(),
        }
    }
}
