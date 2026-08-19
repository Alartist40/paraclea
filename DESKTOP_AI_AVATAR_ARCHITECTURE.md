# Desktop AI Anime Avatar Architecture
## Nikke-Style Interactive AI — STT → LLM → TTS with Emotion-Driven Avatar
### Rust + Go Stack, Runs on Computer and Phone

---

## What You're Building

A real-time conversational AI with a 2.5D anime avatar that:
1. **Hears you** — captures speech, converts to text (STT)
2. **Thinks** — generates response using your LeafcutterLLM
3. **Feels** — analyzes sentiment of the response, maps to emotion
4. **Speaks** — converts response to speech with lip sync (TTS)
5. **Moves** — avatar displays emotion via mesh deformation in real-time

**The magic:** The avatar isn't pre-scripted. Every eyebrow raise, smile, head tilt is driven by what the AI is saying and how it's saying it.

---

## Complete Technology Stack

### Core Pipeline (Rust)

| Component | Crate | Purpose |
|-----------|-------|---------|
| **Async runtime** | `tokio` | Event loop, channels, WebSocket |
| **STT** | `whisper-cpp-plus` | Real-time speech-to-text with VAD |
| **TTS** | `piper-tts-rs` | Neural text-to-speech |
| **Sentiment** | `rust-bert` | Emotion detection from text |
| **Audio I/O** | `cpal` | Cross-platform audio capture/playback |
| **Serialization** | `serde` + `serde_json` | Config, state, protocol |

### LLM (Your Own)

| Component | Language | Purpose |
|-----------|----------|---------|
| **LeafcutterLLM** | Go + Rust | Local GGUF inference engine |

### Avatar Rendering

| Component | Language | Purpose |
|-----------|----------|---------|
| **Bevy Engine** | Rust | Game engine, rendering loop, GPU |
| **Inochi2D** (recommended) or **Live2D Cubism** | Rust via `bevy_inochi2d` or `live2d-cubism-core-sys` | 2D mesh deformation avatar format |
| **Renderer** | wgpu (Vulkan/Metal/DX12/WebGPU) | GPU-accelerated 2D rendering |

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        DESKTOP / PHONE                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  AVATAR RENDERER (Bevy Engine + Inochi2D/Live2D)            │   │
│  │  60 FPS, GPU-accelerated                                     │   │
│  │                                                              │   │
│  │  ┌────────────┐  ┌──────────┐  ┌──────────┐               │   │
│  │  │ Emotion    │  │ Lip Sync │  │ Physics  │               │   │
│  │  │ State Mach │  │ Mouth    │  │ Hair/Eye │               │   │
│  │  └─────┬──────┘  └────┬─────┘  └────┬─────┘               │   │
│  │        │              │             │                       │   │
│  │        └──────────────┼─────────────┘                       │   │
│  │                       ▼                                      │   │
│  │              ┌────────────────┐                              │   │
│  │              │ Mesh Deformer  │                              │   │
│  │              │ (vertex shader)│                              │   │
│  │              └───────┬────────┘                              │   │
│  │                      ▼                                       │   │
│  │              ┌────────────────┐                              │   │
│  │              │ GPU Render     │                              │   │
│  │              │ (wgpu/Vulkan)  │                              │   │
│  │              └───────┬────────┘                              │   │
│  │                      ▼                                       │   │
│  │              ┌────────────────┐                              │   │
│  │              │ Window Display │◄── User sees this            │   │
│  │              └────────────────┘                              │   │
│  └──────────────────────┬──────────────────────────────────────┘   │
│                         │ emotion params (Rust channel)            │
│                         ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  CORE PIPELINE (Rust + Tokio async)                          │   │
│  │                                                              │   │
│  │  ┌──────────┐     ┌──────────┐     ┌──────────┐           │   │
│  │  │ STT      │     │ LLM      │     │ TTS      │           │   │
│  │  │ Whisper  │────►│ Leafcut- │────►│ Piper    │           │   │
│  │  │ cpp-plus │     │ terLLM   │     │ tts-rs   │           │   │
│  │  └────┬─────┘     │ (Go API) │     └────┬─────┘           │   │
│  │       │            └────┬─────┘          │                 │   │
│  │       │                 │                │                 │   │
│  │       │    ┌────────────┴────────────────┘                 │   │
│  │       │    │                                               │   │
│  │       └───►│ Sentiment Analysis (rust-bert)                 │   │
│  │            │                                               │   │
│  │            │ "I'm so happy for you!"                       │   │
│  │            │ ──► {joy: 0.95, surprise: 0.2}               │   │
│  │            │                                               │   │
│  │            └──────────┬────────────────────────────────────┘   │
│  │                       │                                         │
│  │                       ▼                                         │
│  │            ┌────────────────────┐                               │
│  │            │ Emotion Mapper     │                               │
│  │            │ joy:0.95 ──►       │                               │
│  │            │   brow_raise=0.8   │                               │
│  │            │   mouth_smile=1.0  │                               │
│  │            │   eye_wide=0.7     │                               │
│  │            │   head_tilt=15deg  │                               │
│  │            └────────┬───────────┘                               │
│  │                     │                                           │
│  └─────────────────────┼───────────────────────────────────────────┘
│                        │
│                        │ tokio::sync::broadcast
│                        ▼
┌────────────────────────┴───────────────────────────────────────────┐
│                                                                    │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│   │ Microphone  │    │ GPU         │    │ Speakers    │         │
│   │ (cpal)      │    │ (Vulkan/    │    │ (cpal)      │         │
│   │             │    │  Metal/D3D) │    │             │         │
│   └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

---

## The Data Flow (One Conversation Turn)

```
┌────────────────────────────────────────────────────────────────────┐
│  STEP 1: USER SPEAKS                                               │
│  Microphone → cpal → 16kHz PCM f32 buffer                         │
│                                                                    │
│  STEP 2: SPEECH-TO-TEXT (whisper-cpp-plus)                        │
│  PCM chunks → VAD (Silero) → Whisper Tiny/Base → "Hello!"         │
│                                                                    │
│  STEP 3: LLM INFERENCE (LeafcutterLLM via Go API)                 │
│  "Hello!" → HTTP POST to localhost:8080 →                         │
│  "Hey there! I'm doing great, how are you?"                       │
│                                                                    │
│  STEP 4: SENTIMENT ANALYSIS (rust-bert)                           │
│  "Hey there! I'm doing great..." → SentimentModel                 │
│  → {joy: 0.92, trust: 0.45, anticipation: 0.30}                   │
│                                                                    │
│  STEP 5: EMOTION MAPPING                                           │
│  {joy: 0.92} → Avatar params:                                     │
│    brow_raise: 0.8, mouth_smile: 1.0, eye_wide: 0.6,            │
│    head_bob: 0.5, blush: 0.4                                       │
│  → Send to Bevy via tokio channel                                  │
│                                                                    │
│  STEP 6: TEXT-TO-SPEECH (piper-tts-rs)                            │
│  "Hey there! I'm doing great..." → Piper ONNX → PCM audio         │
│  → Audio playback via cpal                                        │
│                                                                    │
│  STEP 7: LIP SYNC                                                  │
│  PCM audio → RMS energy analysis → mouth shape sequence:          │
│  [A, A, B, C, B, A, A, D, D, C, B, A]                             │
│  → Send to Bevy via tokio channel                                  │
│                                                                    │
│  STEP 8: AVATAR RENDERING (Bevy + Inochi2D)                       │
│  60 FPS loop:                                                      │
│    - Interpolate emotion params (200ms blend time)                │
│    - Apply lip sync mouth shape                                    │
│    - Update physics (hair bounce, blink)                          │
│    - Deform mesh vertices                                          │
│    - Render to GPU                                                 │
│    → User sees smiling, talking anime character                   │
└────────────────────────────────────────────────────────────────────┘
```

---

## The Crates — Detailed

### 1. whisper-cpp-plus (STT)

```toml
[dependencies]
whisper-cpp-plus = "0.1"
```

```rust
use whisper_cpp_plus::{WhisperContext, WhisperStreamPcm, WhisperStreamPcmConfig,
                       FullParams, SamplingStrategy, PcmReader, PcmReaderConfig};

// Load model (Tiny = fast, Base = balanced, Small = accurate)
let ctx = WhisperContext::new("models/ggml-tiny.bin")?;
let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 })
    .language("en");

// Real-time streaming from microphone
let config = WhisperStreamPcmConfig { use_vad: true, ..Default::default() };
let mic_reader = PcmReader::new(Box::new(mic_source), PcmReaderConfig::default());

let mut stream = WhisperStreamPcm::new(&ctx, params, config, mic_reader)?;

stream.run(|segments, _start_ms, _end_ms| {
    for seg in segments {
        println!("User said: {}", seg.text);
        // Send to LLM pipeline...
    }
})?;
```

**Models for different quality/speed tradeoffs:**

| Model | Size | Speed (real-time) | Quality | VRAM |
|-------|------|-------------------|---------|------|
| tiny | 39MB | ~32x faster | Good | ~1GB |
| base | 74MB | ~16x faster | Better | ~1GB |
| small | 244MB | ~6x faster | Great | ~2GB |
| medium | 769MB | ~2x faster | Excellent | ~5GB |

For real-time conversation on desktop: **tiny** or **base** is fine.

### 2. piper-tts-rs (TTS)

```toml
[dependencies]
piper-tts-rs = "0.1"
```

```rust
use piper_tts_rs::PiperSession;

let tts = PiperSession::new(
    "models/en_US-lessac-medium.onnx",
    "models/en_US-lessac-medium.onnx.json",
    None,  // or Some("gpu") for CUDA
)?;

// Generate speech (returns 22050 Hz mono f32 PCM)
let mut audio_buffer: Vec<f32> = Vec::new();
tts.generate_speech_to_buffer(&mut audio_buffer, "Hello, I'm your AI companion!")?;

// Play via cpal
play_audio(&audio_buffer, 22050)?;
```

### 3. rust-bert (Sentiment Analysis)

```toml
[dependencies]
rust-bert = "0.21"
tch = "0.13"  # PyTorch C++ backend
```

```rust
use rust_bert::pipelines::sentiment::{Sentiment, SentimentModel, SentimentPolarity};

let sentiment_model = SentimentModel::new(Default::default())?;

fn analyze_emotion(text: &str, model: &SentimentModel) -> EmotionState {
    let sentiments = model.predict(&[text]);
    
    // Map sentiment to 8-dimensional Plutchik emotion space
    let polarity = sentiments[0].polarity;
    let score = sentiments[0].score;
    
    match polarity {
        SentimentPolarity::Positive => EmotionState {
            joy: score,
            trust: score * 0.5,
            anticipation: score * 0.3,
            ..Default::default()
        },
        SentimentPolarity::Negative => EmotionState {
            sadness: score * 0.6,
            fear: score * 0.3,
            anger: score * 0.2,
            ..Default::default()
        },
    }
}
```

**For richer emotion detection**, use rust-bert's `ZeroShotClassificationModel`:

```rust
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;

let classifier = ZeroShotClassificationModel::new(Default::default())?;

let candidate_labels = &[
    "joy", "sadness", "anger", "fear", "surprise", 
    "love", "excitement", "confusion", "empathy", "curiosity"
];

let output = classifier.predict(
    &[text],
    candidate_labels,
    None,  // No template
    128,   // Max length
);

// output[0] = [Label("joy", 0.85), Label("excitement", 0.72), ...]
```

### 4. Emotion State Machine

```rust
// Emotion parameters that drive the avatar
#[derive(Clone, Debug, Default)]
pub struct EmotionState {
    // Plutchik primary emotions (0.0 - 1.0)
    pub joy: f32,
    pub trust: f32,
    pub fear: f32,
    pub surprise: f32,
    pub sadness: f32,
    pub disgust: f32,
    pub anger: f32,
    pub anticipation: f32,
    
    // Derived expressions
    pub love: f32,       // joy + trust
    pub submission: f32,  // trust + fear
    pub awe: f32,        // fear + surprise
    pub disapproval: f32, // surprise + sadness
    pub remorse: f32,    // sadness + disgust
    pub contempt: f32,   // disgust + anger
    pub aggressiveness: f32, // anger + anticipation
    pub optimism: f32,   // anticipation + joy
}

// Maps emotion state to avatar morph targets
impl EmotionState {
    pub fn to_avatar_params(&self) -> AvatarParams {
        AvatarParams {
            brow_raise: self.joy * 0.8 + self.surprise * 1.0,
            brow_furrow: self.anger * 0.8 + self.disgust * 0.5,
            mouth_smile: self.joy * 1.0 + self.optimism * 0.5,
            mouth_frown: self.sadness * 0.8 + self.remorse * 0.4,
            mouth_open: self.surprise * 0.7 + self.fear * 0.3,
            eye_wide: self.surprise * 0.8 + self.joy * 0.3,
            eye_squint: self.anger * 0.5 + self.joy * 0.2,
            blush: self.love * 0.8 + self.submission * 0.4,
            head_tilt: (self.trust - self.fear) * 15.0,  // degrees
            head_bob: self.joy * 0.3 + self.excitement() * 0.5,
        }
    }
    
    pub fn excitement(&self) -> f32 {
        (self.joy + self.anticipation + self.surprise) / 3.0
    }
}
```

---

## Avatar Format: Inochi2D (Recommended) vs Live2D Cubism

### Option A: Inochi2D (Fully Open-Source, Free)

**Inochi2D** is a free, open-source VTuber avatar format that supports mesh deformation, physics, and real-time puppeteering. It was created specifically because Live2D's licensing is restrictive.

| Feature | Inochi2D |
|---------|----------|
| Cost | **100% free, MIT license** |
| Mesh deformation | Yes — full vertex-level control |
| Physics | Yes — spring/damper for hair, clothing |
| Expressions | Yes — parameter blend system |
| Lip sync | Yes — mouth shape parameter mapping |
| Rust support | `bevy_inochi2d` crate (Bevy integration) |
| Creator tool | Inochi2D Creator (free) |
| File format | `.inx` (open specification) |

**Rust integration:**
```toml
[dependencies]
bevy = "0.13"
bevy_inochi2d = "0.1"
```

```rust
use bevy::prelude::*;
use bevy_inochi2d::Inochi2DPlugin;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    
    // Load Inochi2D avatar
    commands.spawn(Inochi2DPuppetBundle {
        puppet: asset_server.load("avatars/my_character.inx"),
        transform: Transform::default(),
        ..default()
    });
}

fn update_emotion(
    mut puppets: Query<&mut Inochi2DPuppet>,
    emotion: Res<CurrentEmotion>,
) {
    for mut puppet in &mut puppets {
        let params = emotion.to_avatar_params();
        
        // Drive puppet parameters directly
        puppet.set_param("BrowLeftAngle", params.brow_raise);
        puppet.set_param("BrowRightAngle", params.brow_raise);
        puppet.set_param("MouthForm", params.mouth_smile - params.mouth_frown);
        puppet.set_param("MouthOpen", params.mouth_open);
        puppet.set_param("EyeLeftOpen", 1.0 - params.eye_squint * 0.3);
        puppet.set_param("EyeRightOpen", 1.0 - params.eye_squint * 0.3);
        puppet.set_param("CheekFlush", params.blush);
        puppet.set_param("HeadAngle", params.head_tilt);
    }
}
```

### Option B: Live2D Cubism (Industry Standard, Proprietary)

Live2D Cubism is what Nikke actually uses. The SDK is free for indie/hobby use but has licensing restrictions.

| Feature | Live2D Cubism |
|---------|--------------|
| Cost | Free for indie/revenue < $100K/year |
| Mesh deformation | Yes — the gold standard |
| Physics | Yes — advanced with multi-physics |
| Expressions | Yes — expression clip system |
| Lip sync | Yes — form analysis |
| Rust support | `live2d-cubism-core-sys` (FFI bindings) |
| Creator tool | Live2D Cubism Editor (free for indie) |
| File format | `.moc3` (proprietary) |

**Rust integration:**
```toml
[dependencies]
live2d-cubism-core-sys = "0.1"
```

```rust
use live2d_cubism_core_sys::core as cubism;

// Load model
let core = cubism::CubismCore::default();
let moc = core.moc_from_bytes(include_bytes!("character.moc3"))?;
let mut model = cubism::Model::from_moc(&moc);

// Update parameters
{
    let mut dyn = model.dynamic.write();
    dyn.set_parameter_value("ParamBrowLY", 0.8);   // Raise left brow
    dyn.set_parameter_value("ParamBrowRY", 0.8);   // Raise right brow
    dyn.set_parameter_value("ParamMouthForm", 1.0); // Smile
    dyn.set_parameter_value("ParamEyeLOpen", 1.0);  // Open eyes
    dyn.set_parameter_value("ParamEyeROpen", 1.0);
    dyn.update();
}
```

**Recommendation:** Start with **Inochi2D** (fully free, open-source, great Rust support). If you need Live2D's specific features later, you can convert or switch.

---

## Creating Your Avatar from Figma Artwork

### The Workflow

```
Figma (your artboard)
    │
    ▼  Export each body part as PNG (transparent background)
    │
Inochi2D Creator (free)
    │
    ├── 1. Import PNG parts as layers
    ├── 2. Draw mesh deformation grid on each part
    ├── 3. Add deformers (rotation, angle XY, etc.)
    ├── 4. Set up physics (hair bounce, eye follow)
    ├── 5. Create expression presets
    │       - Joy: brows up, mouth smile, eyes wide
    │       - Sad: brows down-outer, mouth frown
    │       - Surprise: brows up-high, mouth O, eyes wide
    │       - Angry: brows down-inner, mouth flat, eyes narrow
    │       - Love: heart eyes, blush, gentle smile
    ├── 6. Set up lip sync mouth shapes
    │       - A: neutral closed
    │       - B: slightly open
    │       - C: round (oo)
    │       - D: teeth showing
    │       - E: tongue out
    │       - F: narrow
    │
    └── 7. Export as .inx file
    │
    ▼
Bevy + bevy_inochi2d
    │
    └── Load .inx, drive parameters from emotion pipeline
```

### Figma Export Checklist

In Figma, create frames for each movable body part:

| Part | Dimensions | Notes |
|------|-----------|-------|
| `head_base` | ~200×200 | Full face shape, ears |
| `body` | ~150×200 | Neck, shoulders, torso |
| `eye_L_open` | ~40×30 | Left eye, open state |
| `eye_L_half` | ~40×30 | Left eye, half-closed |
| `eye_L_closed` | ~40×30 | Left eye, fully closed |
| `eye_R_*` | ~40×30 | Mirror of left eye |
| `brow_L_neutral` | ~50×10 | Left eyebrow, neutral |
| `brow_L_raised` | ~50×10 | Left eyebrow, raised |
| `brow_L_lowered` | ~50×10 | Left eyebrow, lowered |
| `brow_R_*` | ~50×10 | Mirror of left brow |
| `mouth_neutral` | ~50×20 | Closed, neutral |
| `mouth_smile` | ~60×25 | Smiling |
| `mouth_frown` | ~50×20 | Frowning |
| `mouth_A` through `mouth_F` | ~50×30 | Lip sync shapes |
| `hair_front` | ~200×150 | Bangs/fringe |
| `hair_back` | ~200×200 | Back hair |
| `blush` | ~40×20 | Cheek blush (optional) |

**Export settings:** PNG, transparent background, 2× resolution for crispness.

---

## Project Structure

```
ai-avatar/
├── Cargo.toml                 # Rust workspace
├── src/
│   ├── main.rs                # Entry point, async runtime
│   ├── lib.rs                 # Library root
│   ├── stt.rs                 # Speech-to-text (whisper-cpp-plus)
│   ├── tts.rs                 # Text-to-speech (piper-tts-rs)
│   ├── llm.rs                 # LeafcutterLLM HTTP client
│   ├── sentiment.rs           # Emotion analysis (rust-bert)
│   ├── emotion.rs             # Emotion state machine
│   ├── avatar.rs              # Avatar parameter mapping
│   ├── audio.rs               # Audio I/O (cpal)
│   ├── config.rs              # Configuration
│   └── protocol.rs            # Internal messaging protocol
├── avatar_renderer/           # Bevy sub-crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Bevy app entry
│       ├── puppet.rs          # Inochi2D puppet control
│       ├── lip_sync.rs        # Lip sync animation
│       └── emotion_driver.rs  # Emotion → parameter mapping
├── assets/
│   └── avatars/
│       └── my_character.inx   # Your Inochi2D avatar
├── models/                    # ML models (not in git)
│   ├── whisper-tiny.bin
│   ├── piper-en_US-lessac-medium.onnx
│   └── piper-en_US-lessac-medium.onnx.json
└── config.toml                # App configuration
```

---

## Cargo.toml (Workspace)

```toml
[workspace]
members = [".", "avatar_renderer"]
resolver = "2"

[package]
name = "ai-avatar"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# STT
whisper-cpp-plus = "0.1"

# TTS
piper-tts-rs = "0.1"

# NLP / Sentiment
rust-bert = "0.21"
tch = "0.13"

# Audio I/O
cpal = "0.15"
ringbuf = "0.3"

# HTTP client for LLM API
reqwest = { version = "0.11", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Concurrency
crossbeam = "0.8"
parking_lot = "0.12"

# Math (for emotion interpolation)
nalgebra = "0.32"

[profile.release]
opt-level = 3
lto = true
```

---

## Performance Targets

| Component | Latency | Hardware |
|-----------|---------|----------|
| STT (Whisper Tiny) | ~300-500ms | CPU |
| LLM (1B model) | ~1-3s | CPU or GPU |
| TTS (Piper) | ~100-300ms | CPU |
| Sentiment analysis | ~50-100ms | CPU |
| Avatar render | ~16ms (60 FPS) | GPU |
| **Total turn** | **~2-5 seconds** | |

---

## Mobile Support

Since everything is in Rust:

| Platform | Support | Notes |
|----------|---------|-------|
| **Linux** | Native | Primary development target |
| **macOS** | Native | Metal renderer in Bevy |
| **Windows** | Native | DX12 renderer in Bevy |
| **Android** | Via Bevy | Compile with `cargo apk` |
| **iOS** | Via Bevy | Compile with `cargo-mobile` |
| **Web (WASM)** | Via Bevy + WebGPU | Experimental |

---

## What "Alive" Looks Like — The Nikke Factor

This architecture achieves the "alive" feeling through:

1. **Continuous animation** — 60 FPS render loop, never frozen
2. **Procedural idle motion** — breathing (subtle scale), micro-head-sway, random blinks
3. **Emotion interpolation** — expressions blend smoothly over 200ms, not snap
4. **Lip sync** — mouth shapes match actual phonemes from TTS audio
5. **Physics** — hair bounces, clothing sways (spring-damper simulation)
6. **Sentiment reactivity** — the avatar mirrors the emotional content of what the AI says
7. **No pre-scripted animations** — everything is computed in real-time based on the conversation

The avatar isn't playing canned animations. It's a **real-time puppet** whose strings are pulled by the AI's emotional state.

---

## Key Resources

| Resource | URL |
|----------|-----|
| whisper-cpp-plus | github.com/operator-kit/whisper-cpp-plus-rs |
| piper-tts-rs | github.com/WrldEngine/piper-tts-rs |
| rust-bert | github.com/guillaume-be/rust-bert |
| Inochi2D | inochi2d.com |
| Inochi2D Creator | github.com/Inochi2D/inochi-creator |
| bevy_inochi2d | lib.rs/crates/bevy_inochi2d |
| live2d-cubism-core-sys | github.com/James2022-rgb/live2d-cubism-core-sys |
| Bevy Engine | bevyengine.org |
| LeafcutterLLM | github.com/Alartist40/LeafcutterLLM |

---

*This architecture gives you a fully Rust/Go native, local-only, real-time interactive AI avatar with mesh-deformation animation that runs on desktop and mobile. No Python in the hot path. No cloud APIs. Fully sovereign.*
