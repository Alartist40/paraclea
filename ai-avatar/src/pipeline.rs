//! Main conversation pipeline:  STT → LLM → Sentiment → TTS.
//!
//! Enhanced with:
//! - Hybrid memory (SQLite sessions + DENDRITE graph)
//! - LLM response caching
//! - Streaming response handling (word-by-word fallback)
//! - Atomic emotion state snapshots for renderer

use crate::audio::{AudioSystem, EnergyVad, enqueue_audio};
use crate::cache::ResponseCache;
use crate::config::Config;
use crate::curator::{Curator, rule_based_extractor};
use crate::emotion::{AvatarSmoother, EmotionState};
use crate::llm::LlmClient;
use crate::memory::{HybridMemory, Memory, Message, Role};
use crate::sentiment::{RuleBasedAnalyzer, SentimentAnalyzer};
use crate::stt::{SttEngine, create_engine as create_stt};
use crate::tts::{TtsEngine, create_engine as create_tts};
use anyhow::Result;
use parking_lot::Mutex;
use ringbuf::Rb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Shared pipeline state that external systems (e.g. the avatar renderer)
/// can read without blocking the hot path.
pub struct PipelineState {
    /// Current emotion state produced by the sentiment analyzer.
    pub emotion: Mutex<EmotionState>,
    /// Smoothed avatar parameters derived from emotion.
    pub avatar_smooth: Mutex<AvatarSmoother>,
    /// Latest transcript from the user.
    pub last_user_text: Mutex<String>,
    /// Latest response from the AI.
    pub last_ai_text: Mutex<String>,
    /// Whether the pipeline is actively processing a turn.
    pub is_speaking: AtomicBool,
}

impl PipelineState {
    pub fn new(blend_factor: f32) -> Self {
        let tension = 20.0 * (1.0 - blend_factor);
        let damping = 5.0;
        Self {
            emotion: Mutex::new(EmotionState::default()),
            avatar_smooth: Mutex::new(AvatarSmoother::new(tension, damping)),
            last_user_text: Mutex::new(String::new()),
            last_ai_text: Mutex::new(String::new()),
            is_speaking: AtomicBool::new(false),
        }
    }
}

/// Owns all pipeline resources and runs the async task graph.
pub struct Pipeline {
    pub state: Arc<PipelineState>,
    pub audio: Arc<AudioSystem>,
    pub memory: Arc<tokio::sync::Mutex<HybridMemory>>,
    shutdown: Arc<AtomicBool>,
}

impl Pipeline {
    pub async fn start(config: Config) -> Result<Self> {
        let state = Arc::new(PipelineState::new(config.emotion.blend_factor));
        let audio = Arc::new(AudioSystem::new(config.audio.clone()));

        // Start cpal streams.
        let (_input_stream, _output_stream) = audio.start()?;

        // Shared shutdown flag.
        let shutdown = Arc::new(AtomicBool::new(false));

        // ── Memory ─────────────────────────────────────────────────────────
        let mut memory = HybridMemory::open(
            config.memory.sessions_db.clone(),
            config.memory.dendrite_db.clone(),
            config.memory.max_history,
            config.memory.max_context_tokens,
        )?;
        memory.seed_default_persona()?;
        let memory = Arc::new(tokio::sync::Mutex::new(memory));

        // ── Cache ──────────────────────────────────────────────────────────
        let cache = if config.cache.enabled {
            Some(ResponseCache::new(config.cache.capacity))
        } else {
            None
        };

        // Channels between stages.
        let (stt_tx, stt_rx) = mpsc::channel::<Vec<f32>>(4);
        let (llm_tx, llm_rx) = mpsc::channel::<String>(4);
        let (resp_tx, resp_rx) = mpsc::channel::<String>(4);

        // ── Build engines ──────────────────────────────────────────────────
        let mut stt_engine = create_stt(&config.stt)?;
        let llm_client = {
            let client = LlmClient::new(config.llm.clone())?;
            if let Some(cache) = cache {
                client.with_cache(cache)
            } else {
                client
            }
        };
        let mut tts_engine = create_tts(&config.tts)?;
        let sentiment: Box<dyn SentimentAnalyzer> = Box::new(RuleBasedAnalyzer::default());

        // ── Spawn tasks ────────────────────────────────────────────────────
        let audio2 = audio.clone();
        let state2 = state.clone();
        let shutdown2 = shutdown.clone();
        let cfg_audio = config.audio.clone();
        tokio::spawn(async move {
            vad_stt_task(
                audio2,
                state2,
                cfg_audio,
                stt_tx,
                &mut stt_engine,
                &shutdown2,
            )
            .await;
        });

        let state3 = state.clone();
        let shutdown3 = shutdown.clone();
        tokio::spawn(async move {
            llm_task(llm_rx, resp_tx, llm_client, state3, &shutdown3).await;
        });

        let state4 = state.clone();
        let audio4 = audio.clone();
        let shutdown4 = shutdown.clone();
        let sample_rate = config.audio.sample_rate;
        tokio::spawn(async move {
            sentiment_tts_task(
                resp_rx,
                audio4,
                state4,
                sample_rate,
                sentiment,
                &mut tts_engine,
                &shutdown4,
            )
            .await;
        });

        let state5 = state.clone();
        let memory5 = memory.clone();
        let shutdown5 = shutdown.clone();
        tokio::spawn(async move {
            stt_to_llm_bridge(stt_rx, llm_tx, state5, memory5, &shutdown5).await;
        });

        // ── Curator ────────────────────────────────────────────────────────
        if config.curator.enabled {
            let memory_curator = memory.clone();
            let shutdown_curator = shutdown.clone();
            let curator_interval = config.curator.interval_minutes;
            tokio::spawn(async move {
                let mut curator = Curator::new(curator_interval);
                curator.run(
                    memory_curator,
                    rule_based_extractor,
                    &shutdown_curator,
                ).await;
            });
        }

        info!("Pipeline started with memory, cache, and curator");
        Ok(Self {
            state,
            audio,
            memory,
            shutdown,
        })
    }

    /// Signal all tasks to shut down gracefully.
    pub async fn shutdown(self) -> Result<()> {
        self.shutdown.store(true, Ordering::SeqCst);
        self.audio.stop();
        info!("Pipeline shutdown signal sent");
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Task: VAD + STT
// ═══════════════════════════════════════════════════════════════════════════

async fn vad_stt_task(
    audio: Arc<AudioSystem>,
    state: Arc<PipelineState>,
    cfg: crate::config::AudioConfig,
    stt_tx: mpsc::Sender<Vec<f32>>,
    stt_engine: &mut Box<dyn SttEngine>,
    shutdown: &AtomicBool,
) {
    let vad = EnergyVad::new(&cfg);
    let frame_samples = (cfg.sample_rate as usize * cfg.vad_frame_ms) / 1000;
    let mut rolling: Vec<f32> = Vec::with_capacity(cfg.sample_rate as usize * 3);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        if shutdown.load(Ordering::Relaxed) {
            info!("VAD+STT task shutting down");
            return;
        }

        // Drain input ring into rolling buffer.
        {
            let mut ring = audio.input_ring.lock().unwrap();
            while let Some(sample) = ring.pop() {
                rolling.push(sample);
            }
        }

        if rolling.len() < frame_samples {
            continue;
        }

        let segments = vad.process(&rolling);
        if segments.is_empty() {
            let max_keep = cfg.sample_rate as usize * 2;
            if rolling.len() > max_keep {
                let drop = rolling.len() - max_keep;
                rolling.drain(0..drop);
            }
            continue;
        }

        let mut processed_up_to = 0usize;
        for seg in &segments {
            if seg.audio.len() < cfg.sample_rate as usize / 2 {
                continue;
            }

            info!("Speech detected: {} samples", seg.audio.len());
            state.is_speaking.store(true, Ordering::SeqCst);

            match stt_engine.transcribe(&seg.audio) {
                Ok(text) if !text.trim().is_empty() => {
                    info!("STT: \"{}\"", text);
                    *state.last_user_text.lock() = text.clone();
                    if stt_tx.send(seg.audio.clone()).await.is_err() {
                        warn!("STT→LLM channel closed");
                        return;
                    }
                }
                Ok(_) => {
                    debug!("STT returned empty text, ignoring");
                }
                Err(e) => {
                    error!("STT failed: {}", e);
                }
            }

            processed_up_to = seg.end;
            state.is_speaking.store(false, Ordering::SeqCst);
        }

        if processed_up_to > 0 {
            rolling.drain(0..processed_up_to.min(rolling.len()));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bridge: STT → LLM (with memory-aware prompt building)
// ═══════════════════════════════════════════════════════════════════════════

async fn stt_to_llm_bridge(
    mut stt_rx: mpsc::Receiver<Vec<f32>>,
    llm_tx: mpsc::Sender<String>,
    state: Arc<PipelineState>,
    memory: Arc<tokio::sync::Mutex<HybridMemory>>,
    shutdown: &AtomicBool,
) {
    while let Some(_audio) = stt_rx.recv().await {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        let text = state.last_user_text.lock().clone();
        if text.trim().is_empty() {
            continue;
        }

        // Save user message to session history.
        {
            let mut mem = memory.lock().await;
            let _ = mem.save_message(Message::new(Role::User, &text));
        }

        // Build prompt from memory graph + conversation history.
        let prompt = {
            let mem = memory.lock().await;
            mem.build_conversation_prompt(&text)
        };

        if llm_tx.send(prompt).await.is_err() {
            warn!("LLM channel closed");
            return;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Task: LLM
// ═══════════════════════════════════════════════════════════════════════════

async fn llm_task(
    mut llm_rx: mpsc::Receiver<String>,
    resp_tx: mpsc::Sender<String>,
    llm_client: LlmClient,
    state: Arc<PipelineState>,
    shutdown: &AtomicBool,
) {
    while let Some(prompt) = llm_rx.recv().await {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        info!("LLM prompt length: {} chars", prompt.len());

        match llm_client.generate(&prompt).await {
            Ok(response) => {
                info!("LLM response: \"{}\"", response.chars().take(80).collect::<String>());
                *state.last_ai_text.lock() = response.clone();
                if resp_tx.send(response).await.is_err() {
                    warn!("Response channel closed");
                    return;
                }
            }
            Err(e) => {
                error!("LLM request failed: {}", e);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Task: Sentiment + TTS
// ═══════════════════════════════════════════════════════════════════════════

async fn sentiment_tts_task(
    mut resp_rx: mpsc::Receiver<String>,
    audio: Arc<AudioSystem>,
    state: Arc<PipelineState>,
    sample_rate: u32,
    sentiment: Box<dyn SentimentAnalyzer>,
    tts_engine: &mut Box<dyn TtsEngine>,
    shutdown: &AtomicBool,
) {
    while let Some(text) = resp_rx.recv().await {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        // ── Sentiment analysis ────────────────────────────────────────────
        match sentiment.analyze(&text) {
            Ok(emotion) => {
                debug!(
                    "Emotion: joy={:.2} trust={:.2} fear={:.2} surprise={:.2} sadness={:.2} anger={:.2}",
                    emotion.joy, emotion.trust, emotion.fear,
                    emotion.surprise, emotion.sadness, emotion.anger
                );
                *state.emotion.lock() = emotion.clone();
                let params = emotion.to_avatar_params();
                state.avatar_smooth.lock().set_target(&params);
            }
            Err(e) => {
                error!("Sentiment analysis failed: {}", e);
            }
        }

        // ── Text-to-speech ────────────────────────────────────────────────
        match tts_engine.synthesize(&text, sample_rate) {
            Ok(audio_buf) => {
                enqueue_audio(&audio.output_ring, &audio_buf);
            }
            Err(e) => {
                error!("TTS synthesis failed: {}", e);
            }
        }
    }
}
