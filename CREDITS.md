# Credits & Acknowledgments

Paraclea stands on the shoulders of incredible open-source AI projects and Rust libraries:

- **[Starling STT-TTS](https://github.com/Alartist40/Starling-STT-TTS)** — Architecture inspiration for zero-bloat CLI command routing (`list`, `run`), one-line installer, and terminal prompt design.
- **[OmniBot](https://github.com/nazirlouis/OmniBot)** — Architecture inspiration for markdown persona files (`SOUL.md`, `IDENTITY.md`, `USER.md`, `MEMORY.md`, `TOOLS.md`, `HEARTBEAT.md`), self-updating tool calls, and background heartbeat memory loops.
- **[Kyutai Pocket TTS](https://github.com/kyutai-labs/pocket-tts)** — Fast, low-latency, CPU-driven text-to-speech engine.
- **[Ollama](https://ollama.com/)** — Local offline LLM inference engine.
- **Rust Ecosystem**:
  - `tokio` — Asynchronous runtime
  - `clap` — Command-line argument parsing
  - `colored` — Terminal text styling
  - `rodio` — Audio playback engine
  - `reqwest` — Async HTTP client
  - `serde` & `serde_yaml` — Serialization
