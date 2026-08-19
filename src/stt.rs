//! Speech-to-text engine.
//!
//! Uses `whisper-rs` when the `whisper_stt` feature is enabled,
//! otherwise falls back to a mock implementation that echos a
//! canned phrase so the rest of the pipeline can be tested.

use crate::config::SttConfig;
use anyhow::Result;
use tracing::{info, warn};

/// Trait for speech-to-text engines.
pub trait SttEngine: Send + Sync {
    /// Transcribe a mono f32 PCM buffer (16 kHz) to text.
    fn transcribe(&mut self, audio: &[f32]) -> Result<String>;
}

// ═══════════════════════════════════════════════════════════════════════════
// Whisper implementation (feature-gated)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "whisper_stt")]
pub struct WhisperEngine {
    ctx: whisper_rs::WhisperContext,
    params: whisper_rs::FullParams<'static, 'static>,
}

#[cfg(feature = "whisper_stt")]
impl WhisperEngine {
    pub fn new(config: &SttConfig) -> Result<Self> {
        if !config.model_path.exists() {
            anyhow::bail!(
                "Whisper model not found at {}.  Download a model from https://huggingface.co/ggerganov/whisper.cpp",
                config.model_path.display()
            );
        }

        let ctx = whisper_rs::WhisperContext::new_with_params(
            config.model_path.to_str().unwrap(),
            whisper_rs::WhisperContextParameters::default(),
        )
        .with_context(|| {
            format!(
                "failed to load whisper model from {}",
                config.model_path.display()
            )
        })?;

        let mut params = whisper_rs::FullParams::new(
            whisper_rs::SamplingStrategy::Greedy { best_of: 1 },
        );
        params.set_n_threads(4);
        params.set_translate(false);
        params.set_language(Some(&config.language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        info!(
            "Whisper engine loaded: {} (lang={})",
            config.model_path.display(),
            config.language
        );
        Ok(Self { ctx, params })
    }
}

#[cfg(feature = "whisper_stt")]
impl SttEngine for WhisperEngine {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String> {
        let mut state = self
            .ctx
            .create_state()
            .context("failed to create whisper state")?;

        state
            .full(self.params.clone(), audio)
            .context("whisper inference failed")?;

        let n_segments = state.full_n_segments();
        let mut text = String::new();
        for i in 0..n_segments {
            let seg = state
                .full_get_segment_text(i)
                .unwrap_or_default();
            text.push_str(&seg);
            text.push(' ');
        }

        let text = text.trim().to_string();
        debug!("STT result: {}", text);
        Ok(text)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Mock implementation
// ═══════════════════════════════════════════════════════════════════════════

/// A mock STT engine for testing without a Whisper model.
pub struct MockSttEngine {
    counter: std::sync::atomic::AtomicUsize,
}

impl MockSttEngine {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl SttEngine for MockSttEngine {
    fn transcribe(&mut self, _audio: &[f32]) -> Result<String> {
        let idx = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let phrases = [
            "Hello, how are you doing today?",
            "What is the weather like?",
            "Tell me a joke.",
            "Can you help me with Rust programming?",
            "I am feeling happy today.",
        ];
        let text = phrases[idx % phrases.len()].to_string();
        warn!("[MOCK STT] Would transcribe {} samples → '{}'", _audio.len(), text);
        Ok(text)
    }
}

/// Build the configured STT engine.
pub fn create_engine(config: &SttConfig) -> Result<Box<dyn SttEngine>> {
    if config.use_mock {
        info!("Using mock STT engine");
        return Ok(Box::new(MockSttEngine::new()));
    }

    #[cfg(feature = "whisper_stt")]
    {
        info!("Using Whisper STT engine");
        Ok(Box::new(WhisperEngine::new(config)?))
    }

    #[cfg(not(feature = "whisper_stt"))]
    {
        warn!("whisper_stt feature disabled — falling back to mock STT");
        Ok(Box::new(MockSttEngine::new()))
    }
}
