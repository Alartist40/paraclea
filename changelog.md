# Changelog

All notable changes to the **Paraclea** project will be documented in this file.

## [0.1.0] - 2026-08-19

### Added
- **Pure Rust Engine Core**: Replaced python prototype with a fast, dependency-free Rust binary (`6.4 MB` release size).
- **Starling CLI Command Interface**: Integrated `clap` CLI parser allowing global command execution (`paraclea`, `paraclea list`, `paraclea run <num|name>`).
- **Gold & Purple Terminal UI**: Implemented vibrant Gold (`#FFD700`) and Purple (`#B14AED`) terminal styling for prompts, banners, and tool logs.
- **OmniBot Persona Architecture**: Integrated dynamic markdown persona files (`IDENTITY.md`, `SOUL.md`, `USER.md`, `MEMORY.md`, `TOOLS.md`, `HEARTBEAT.md`) and daily interaction logging under `persona/logs/daily/`.
- **Pocket TTS Integration**: Added `PocketTtsEngine` supporting local HTTP server (`http://localhost:8000/tts`) with automatic CLI runner fallback for CPU speech synthesis using cute female voices (`alba`, `cosette`, `eve`, `mary`, `vera`).
- **Offline Ollama Engine**: Added `OllamaClient` for local inference supporting models such as `llama3.2`, `qwen2.5`, `mistral`, and `gemma2`.
- **Self-Development Tool Executor**: Added tool parsing and execution capabilities (`soul_replace`, `memory_replace`, `persona_replace`, `daily_log_append`, `read_file`, `write_file`, `execute_command`).
- **Background Heartbeat Loop**: Added Tokio async timer for periodic background reflection and long-term memory consolidation.
- **Single-Line Installer (`install.sh`)**: Added shell installer script for Linux/macOS with automatic PATH setup.
