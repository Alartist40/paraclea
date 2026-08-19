# 🌸 Paraclea — Cute AI Companion & Self-Developing Assistant Engine

> **A light, fast, and independent AI companion built in pure Rust, powered by Pocket TTS speech synthesis, offline Ollama LLMs, and an OmniBot-inspired self-developing persona system.**

---

## 🌟 Overview

**Paraclea** is a personal AI assistant and companion designed to run completely offline on CPU with zero cloud dependencies. She combines:
- **Cute Girl Persona**: Warm, intelligent, loyal, witty, and expressive personality.
- **Pure Rust Core**: Fast, lightweight (~6.4MB release binary), memory-efficient, and free from heavy Python environment nightmares.
- **OmniBot Self-Development**: Dynamic Markdown persona management (`SOUL.md`, `IDENTITY.md`, `USER.md`, `MEMORY.md`, `TOOLS.md`, `HEARTBEAT.md`) that Paraclea can inspect and rewrite autonomously.
- **Pocket TTS Speech Synthesis**: Smooth, low-latency, CPU-driven speech synthesis with customizable female voice presets (`alba`, `cosette`, `eve`, `mary`, `vera`, or custom `.wav`/`.safetensors`).
- **Offline Ollama Engine**: Local inference powered by Ollama models (`llama3.2`, `qwen2.5`, `mistral`, etc.).
- **Gold & Purple UI**: Elegant terminal theme built in Rust.

---

## ⚡ One-Line Installation

Install Paraclea on any Linux or macOS system with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash
```

---

## 🚀 Usage & Commands

Once installed, run `paraclea` from anywhere in your terminal:

```bash
# Display help and version information
paraclea --help

# List all available Ollama and local models (numbered)
paraclea list

# Run Paraclea with a specific model by number or name
paraclea run 1
paraclea run llama3.2

# Run Paraclea in interactive companion mode (default model)
paraclea
```

---

## 🛠️ Architecture & Features

```
                            ┌─────────────────────────────────┐
                            │      PARACLEA CORE (Rust)       │
                            │                                 │
                            │  ┌───────────────────────────┐  │
                            │  │      Persona System       │  │
                            │  │ (IDENTITY, SOUL, USER,    │  │
                            │  │  MEMORY, TOOLS, HEARTBEAT)│  │
                            │  └─────────────┬─────────────┘  │
                            │                │                │
                            │                ▼                │
                            │  ┌───────────────────────────┐  │
                            │  │     Ollama LLM Engine     │  │
                            │  │ (Offline local inference) │  │
                            │  └─────────────┬─────────────┘  │
                            └────────────────┼────────────────┘
                                             │
             ┌───────────────────────────────┼───────────────────────────────┐
             ▼                               ▼                               ▼
   ┌───────────────────┐           ┌───────────────────┐           ┌───────────────────┐
   │ Tool Executor     │           │ Pocket TTS Engine │           │ Heartbeat Loop    │
   │ (soul_replace,    │           │ (CPU Speech API)  │           │ (Background       │
   │  memory_replace,  │           └─────────┬─────────┘           │  Self-Maintenance)│
   │  execute_command) │                     │                     └───────────────────┘
   └───────────────────┘                     ▼
                                   ┌───────────────────┐
                                   │ Audio Playback    │
                                   │ (rodio speakers)  │
                                   └───────────────────┘
```

### Self-Development Capabilities
Paraclea can execute tool calls during conversation:
- `soul_replace(content)` — Update her behavioral guidelines in `SOUL.md`.
- `memory_replace(content)` — Update long-term consolidated memory facts in `MEMORY.md`.
- `persona_replace(file, content)` — Update persona markdown files.
- `daily_log_append(content)` — Log interaction turns into `persona/logs/daily/`.
- `read_file(path)` & `write_file(path, content)` — Inspect or modify system files.
- `execute_command(command)` — Run shell commands safely.

---

## 🎨 Theme & Customization

Paraclea features a **Gold & Purple** theme:
- **Gold (`#FFD700`)**: Prompts, status indicators, banner headers, tool logs.
- **Purple (`#B14AED`)**: Paraclea responses, borders, section headers.

---

## 📜 License

Licensed under the [MIT License](LICENSE).
