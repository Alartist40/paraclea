//! Text-to-speech engine.
//!
//! Uses `piper-tts-rs` when the `piper_tts` feature is enabled,
//! otherwise falls back to a mock that logs what would be spoken.

use crate::config::TtsConfig;
use anyhow::Result;
use tracing::{info, warn};

/// Trait for text-to-speech engines.
pub trait TtsEngine: Send + Sync {
    /// Synthesize text into a mono f32 PCM buffer at the requested sample rate.
    fn synthesize(&mut self, text: &str, sample_rate: u32) -> Result<Vec<f32>>;
}

// ═══════════════════════════════════════════════════════════════════════════
// Piper implementation (feature-gated)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "piper_tts")]
pub struct PiperEngine {
    session: piper_tts_rs::PiperSession,
}

#[cfg(feature = "piper_tts")]
impl PiperEngine {
    pub fn new(config: &TtsConfig) -> Result<Self> {
        if !config.model_path.exists() {
            anyhow::bail!(
                "Piper ONNX model not found at {}.  Download voices from https://github.com/rhasspy/piper/releases/tag/v1.0.0",
                config.model_path.display()
            );
        }

        let session = piper_tts_rs::PiperSession::new(
            config.model_path.to_str().unwrap(),
            config.model_json_path.to_str().unwrap(),
            None, // CPU inference
        )
        .with_context(|| {
            format!(
                "failed to load piper model from {}",
                config.model_path.display()
            )
        })?;

        info!(
            "Piper TTS engine loaded: {}",
            config.model_path.display()
        );
        Ok(Self { session })
    }
}

#[cfg(feature = "piper_tts")]
impl TtsEngine for PiperEngine {
    fn synthesize(&mut self, text: &str, _sample_rate: u32) -> Result<Vec<f32>> {
        let mut buffer: Vec<f32> = Vec::new();
        self.session
            .generate_speech_to_buffer(&mut buffer, text)
            .context("piper synthesis failed")?;
        Ok(buffer)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Mock implementation
// ═══════════════════════════════════════════════════════════════════════════

/// A mock TTS engine for testing without a voice model.
pub struct MockTtsEngine;

impl MockTtsEngine {
    pub fn new() -> Self {
        Self
    }
}

impl TtsEngine for MockTtsEngine {
    fn synthesize(&mut self, text: &str, sample_rate: u32) -> Result<Vec<f32>> {
        warn!(
            "[MOCK TTS] Would synthesize: \"{}\" @ {} Hz",
            text, sample_rate
        );
        // Return a short sine-wave beep so the audio pipeline has something
        // to play (makes testing more satisfying).
        let duration_sec = 0.5f32;
        let freq = 440.0f32;
        let samples = (duration_sec * sample_rate as f32) as usize;
        let mut audio = Vec::with_capacity(samples);
        for i in 0..samples {
            let t = i as f32 / sample_rate as f32;
            let envelope = 1.0 - (t / duration_sec);
            let sample = (t * freq * 2.0 * std::f32::consts::PI).sin() * envelope * 0.3;
            audio.push(sample);
        }
        Ok(audio)
    }
}

/// Build the configured TTS engine.
pub fn create_engine(config: &TtsConfig) -> Result<Box<dyn TtsEngine>> {
    if config.use_mock {
        info!("Using mock TTS engine");
        return Ok(Box::new(MockTtsEngine::new()));
    }

    #[cfg(feature = "piper_tts")]
    {
        info!("Using Piper TTS engine");
        Ok(Box::new(PiperEngine::new(config)?))
    }

    #[cfg(not(feature = "piper_tts"))]
    {
        warn!("piper_tts feature disabled — falling back to mock TTS");
        Ok(Box::new(MockTtsEngine::new()))
    }
}
