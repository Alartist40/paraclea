## Goal

Build the **ai-avatar** core pipeline: a real-time conversational AI system that runs entirely on the user's machine. The pipeline flows as:

```
Microphone → Audio I/O (cpal) → VAD → STT (Whisper) → LLM (LeafcutterLLM)
→ Sentiment/Emotion → TTS (Piper) → Speakers
```

Now enhanced with:
- **Hybrid Memory** (SQLite sessions + DENDRITE knowledge graph) from Cynapse Mini architecture
- **Live Streaming** response architecture from Pathfinder-EYE
- **LLM Response Caching** for faster repeated queries
- **Background Curator** for automatic memory consolidation
- **RAM Safety Monitor** for system protection

The avatar renderer (Bevy + Inochi2D/Live2D) is **explicitly out of scope for now**.

---

## Current State

✅ **Compiles and runs** — `cargo build --release` succeeds, binary starts, memory system initializes, audio streams start, mock pipeline executes end-to-end.

| Component | Status | Notes |
|-----------|--------|-------|
| Project structure | ✅ Complete | Workspace-ready, modular crates |
| Audio I/O (cpal) | ✅ Complete | Capture + playback + ring buffers working |
| VAD | ✅ Complete | Energy-based detector, configurable thresholds |
| STT interface | ✅ Complete | `SttEngine` trait + Whisper impl + Mock impl |
| LLM client | ✅ Complete | HTTP client supporting Rust (8081) and Go (8080) flavours |
| **LLM Streaming** | ✅ Ready | `generate_stream()` method ready; falls back to word-by-word yield until server supports true streaming |
| **LLM Cache** | ✅ Complete | LRU cache with 100-entry capacity, keyed by prompt hash |
| Sentiment/Emotion | ✅ Complete | `SentimentAnalyzer` trait + rule-based impl + Plutchik state machine |
| TTS interface | ✅ Complete | `TtsEngine` trait + Piper impl + Mock impl |
| Pipeline orchestration | ✅ Complete | Tokio async task graph with channels |
| Config loading | ✅ Complete | TOML-based, falls back to embedded defaults |
| **Hybrid Memory** | ✅ Complete | SQLite session storage + DENDRITE knowledge graph |
| **DENDRITE Graph** | ✅ Complete | Nodes with wiki-links, backlinks, tags, FTS5 search |
| **Context Assembly** | ✅ Complete | Dynamic prompt building from graph + history |
| **Memory Compaction** | ✅ Complete | Old messages summarized automatically |
| **Curator** | ✅ Complete | Background task extracts facts from conversations |
| **Safety Monitor** | ✅ Complete | RAM watchdog, emergency shutdown at 92% |
| Real Whisper STT | ⚠️ Blocked | Needs `libclang` / `clang` system package |
| Real Piper TTS | ⚠️ Blocked | CMake fails because workspace path contains spaces (`AI agent`) |
| LeafcutterLLM server test | ⚠️ Pending | Server not running during test; client logic is correct |

---

## Active Files

### Core Pipeline
- `ai-avatar/Cargo.toml` — Workspace manifest with feature flags (`mock_stt`, `mock_tts`, `whisper_stt`, `piper_tts`)
- `ai-avatar/config.toml` — Runtime configuration (audio, VAD, LLM endpoint, model paths, memory, cache, safety, curator)
- `ai-avatar/src/main.rs` — Entry point: logging, LLM health check, safety monitor, pipeline start, Ctrl+C shutdown
- `ai-avatar/src/config.rs` — `Config::load()` from TOML; all sub-config structs with defaults
- `ai-avatar/src/audio.rs` — `AudioSystem` (cpal streams), `EnergyVad`, `SpeechSegment`, utility fns
- `ai-avatar/src/stt.rs` — `SttEngine` trait, `WhisperEngine` (feature-gated), `MockSttEngine`, `create_engine()`
- `ai-avatar/src/llm.rs` — `LlmClient` with Rust/Go flavour support, `generate()`, **`generate_stream()`**, cache integration
- `ai-avatar/src/sentiment.rs` — `SentimentAnalyzer` trait, `RuleBasedAnalyzer`, `MockAnalyzer`
- `ai-avatar/src/emotion.rs` — `EmotionState` (Plutchik), `AvatarParams`, `Smoother`, `AvatarSmoother`
- `ai-avatar/src/tts.rs` — `TtsEngine` trait, `PiperEngine` (feature-gated), `MockTtsEngine`, `create_engine()`
- `ai-avatar/src/pipeline.rs` — `Pipeline` struct, `PipelineState`, async tasks (VAD+STT, bridge, LLM, sentiment+TTS), curator spawn

### New: Memory & Intelligence Systems
- `ai-avatar/src/memory/mod.rs` — `Memory` trait, `Message`, `Role`, `estimate_tokens()`, `now_timestamp()`
- `ai-avatar/src/memory/sqlite.rs` — `SqliteMemory`: messages table, summaries table, compaction logic
- `ai-avatar/src/memory/hybrid.rs` — `HybridMemory`: dual-layer memory, persona seeding, `build_conversation_prompt()`, `save_fact()`
- `ai-avatar/src/dendrite/mod.rs` — `Dendrite`: in-memory knowledge graph with wiki-links, backlinks, BFS neighbors
- `ai-avatar/src/dendrite/store.rs` — `DendriteStore`: SQLite persistence, FTS5 index with sync triggers, fallback LIKE search
- `ai-avatar/src/dendrite/context.rs` — `DendriteContext`: relevance scoring, neighborhood expansion, dynamic prompt assembly
- `ai-avatar/src/curator.rs` — `Curator`: background consolidation, `rule_based_extractor()` for fact extraction
- `ai-avatar/src/cache.rs` — `ResponseCache`: LRU cache for LLM responses, hit/miss stats
- `ai-avatar/src/safety.rs` — `SafetyMonitor`: RAM usage watchdog with `sysinfo`

---

## Recent Changes (This Session)

### Architecture Enhancements from Cynapse Mini & Pathfinder-EYE

1. **Hybrid Memory System**
   - Ported Cynapse Mini's dual-layer memory design: SQLite sessions + DENDRITE knowledge graph.
   - `HybridMemory` seeds default persona nodes (`identity`, `soul`, `user`) on first run.
   - Conversation history is persisted to SQLite and included in LLM prompts dynamically.

2. **DENDRITE Knowledge Graph**
   - Wiki-link syntax `[[node-id]]` auto-creates graph edges.
   - Backlinks are auto-maintained on upsert/delete.
   - FTS5 full-text search with SQLite triggers for automatic index sync.
   - Fallback to `LIKE` queries if FTS5 is unavailable.

3. **Dynamic Context Assembly**
   - `DendriteContext::build_prompt()` scores nodes by: title match (+15), content match (+2/occurrence), tag match (+5), recency boost (7-day decay), connectivity bonus (+0.3/link), type priority.
   - Includes 1-hop neighborhood expansion for richer context.
   - Respects a configurable token budget (default 6000).

4. **Memory Compaction**
   - When messages exceed `max_history + 20`, oldest messages are compacted into summaries.
   - Summaries are stored in SQLite and prepended to future prompts as `[Previous context summary]`.

5. **Background Curator**
   - Ported from Cynapse Mini's `curator.rs`.
   - Runs every 10 minutes (configurable), reviews recent conversation history.
   - `rule_based_extractor()` pulls preferences ("I like X") and identity facts without needing an LLM call.
   - Facts are deduplicated and stored as `NodeType::Memory` in DENDRITE.

6. **LLM Response Caching**
   - Ported from Pathfinder-EYE's AI response cache.
   - `ResponseCache`: LRU with 100-entry capacity, keyed by MD5 hash of prompt.
   - Cache hit logs at `info` level; stats available via `cache.stats()`.

7. **Streaming Architecture (Ready)**
   - `LlmClient::generate_stream()` yields tokens word-by-word.
   - Currently simulates streaming by splitting the full response (server doesn't stream yet).
   - When LeafcutterLLM server adds SSE/NDJSON streaming, swap the implementation.

8. **RAM Safety Monitor**
   - Ported from Pathfinder-EYE's safety monitor.
   - Checks RAM every 5 seconds via `sysinfo`.
   - Triggers emergency shutdown at 92% usage.
   - Warns at 80% of threshold.

9. **Prompt Engineering**
   - Replaced static prompts with `build_conversation_prompt()` which assembles:
     - DENDRITE graph context (relevant memories + persona)
     - Recent summary contexts
     - Last N messages from conversation history
     - Current user message

---

## Failed Attempts

### 1. `rusqlite` feature `bundled-fulltext` doesn't exist
- **What:** Tried to use `features = ["bundled", "bundled-fulltext"]`.
- **Why it failed:** The feature is called `bundled`, which already includes FTS5 in rusqlite 0.34.
- **Fix:** Changed to `features = ["bundled"]`.

### 2. `rusqlite::Connection` is not `Sync`
- **What:** `Memory` trait required `Send + Sync`.
- **Why it failed:** SQLite connections use `RefCell` internally and cannot be shared across threads.
- **Fix:** Changed `Memory` trait to only require `Send`. `HybridMemory` is always accessed through `tokio::sync::Mutex`, which provides the necessary synchronisation.

### 3. Borrow checker issues in DENDRITE graph mutations
- **What:** Tried to borrow `self.nodes` mutably while holding an immutable reference to a node.
- **Why it failed:** Rust's borrow rules prevent this.
- **Fix:** Clone the needed data (old links, existing node IDs) before the mutable borrow.

---

## Next Steps

1. **Install `clang` and test real Whisper STT**
   ```bash
   sudo pacman -S clang
   cd ai-avatar
   cargo run --release --no-default-features --features whisper_stt,mock_tts
   ```
   Download `ggml-tiny.bin` to `models/` if not present.

2. **Fix or workaround Piper TTS build**
   - Option A: Temporarily symlink/copy project to `/tmp/ai-avatar` (no spaces) and build there.
   - Option B: Investigate if `piper-tts-rs-sys` can be patched to quote CMake paths.
   - Option C: Swap `piper-tts-rs` for a subprocess call to the official `piper` binary.

3. **End-to-end integration test with LeafcutterLLM**
   - Start the Rust server from `LeafcutterLLM/rust/`:
     ```bash
     cd /home/xander/Documents/portfolio/LeafcutterLLM/rust
     cargo run --release -- --model ../Qwen3.5-9B-IQ4_NL.gguf --port 8081
     ```
   - Then run `ai-avatar` and verify the full turn: speak → transcribe → LLM → sentiment → TTS → hear beep.
   - Check `data/sessions.db` and `data/dendrite.db` are populated.

4. **Add true LLM streaming to LeafcutterLLM server**
   - Modify `LeafcutterLLM/rust/src/api/mod.rs` to support SSE/NDJSON output.
   - Then update `ai-avatar/src/llm.rs` `generate_stream()` to parse the real stream.
   - This enables sentence-level TTS (start speaking before full response arrives).

5. **Enhance curator with LLM-driven extraction**
   - Replace `rule_based_extractor` with an actual LLM call that generates structured JSON facts.
   - This will extract much richer memories than regex patterns.

6. **(Future) Avatar renderer integration**
   - When ready, create `avatar_renderer/` Bevy sub-crate.
   - Read `PipelineState.emotion` and `PipelineState.avatar_smooth` from the renderer loop.

---

## Context to Preserve

- **LeafcutterLLM code is read-only** — located at `/home/xander/Documents/portfolio/LeafcutterLLM/`. Do not edit it, but reference it freely. The Rust server API (`/generate` returning `text`) is preferred over the Go server API (`/generate` returning `tokens`).
- **Feature flags are the gate** — Real engines are behind Cargo features so CI and new devs never hit dependency failures.
- **Mock engines produce audible feedback** — `MockTtsEngine` doesn't just log; it generates a 440 Hz sine-wave beep so you can actually hear the pipeline working end-to-end without voice models.
- **Architecture doc lives at** `DESKTOP_AI_AVATAR_ARCHITECTURE.md` — it describes the full vision including Bevy/Inochi2D.
- **Source architectures:**
  - `/home/xander/Documents/THE-PATHFINDER-EYE/` — Live streaming, vision pipeline, safety monitor, response caching
  - `/home/xander/Documents/portfolio/cynapse-mini/` — Hybrid memory, DENDRITE graph, context assembly, curator
- **Rust version:** 1.95.0 (Arch Linux). Tokio 1.35+, cpal 0.15, reqwest 0.12, rusqlite 0.34.
- **Databases created at runtime:** `data/sessions.db` (conversation history), `data/dendrite.db` (knowledge graph + FTS5).
