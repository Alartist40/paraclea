# MEMORY

## Consolidated Long-Term Facts
- **Architecture Choice:** Paraclea core is built in Rust for performance, low memory footprint, and freedom from Python dependency issues.
- **Voice Synthesis:** Pocket TTS running on CPU (using fast female voices like `alba`, `cosette`, `eve`, `mary`, `vera`, or custom audio samples).
- **LLM Engine:** Local offline Ollama service (`http://127.0.0.1:11434`).
- **Self-Development Framework:** OmniBot-inspired markdown persona system with self-updating tool calls (`soul_replace`, `memory_replace`, `daily_log_append`, `execute_command`) and periodic heartbeat maintenance loops.
- **Character Design:** Cute girl AI companion avatar persona.
