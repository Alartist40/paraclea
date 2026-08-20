# 🕊️ Paraclea — AI Companion Assistant & Self-Developing RAG Engine

> **A fast, lightweight, and independent AI companion built in pure Rust, powered by local Qdrant vector storage, Ollama LLM & Vision OCR, Pocket TTS speech synthesis, and a Proverbs 31 Helper persona.**

---

## 🌟 Overview

**Paraclea (Παράκλησις)** is a personal AI companion and offline reference assistant designed to run completely locally on CPU with zero cloud dependencies.

### Key Capabilities
- **Proverbs 31 Helper Persona**: Gentle, wise, dignified, courageous, humble, reverent, proactive, and industrious helper identity (`SOUL.md`).
- **Offline Qdrant Vector Engine**: Fast local vector storage (`http://localhost:6333`) for Scripture RAG, book-to-skill knowledge bases, and survival guides.
- **Ollama Vision OCR (`frob/unlimited-ocr:q8_0`)**: Direct Base64 vision document OCR for photos of book pages, manuscript scans, and handwritten documents.
- **File Format Auto-Detection (`src/detect.rs`)**: Automatic format detection (`.png`, `.jpg`, `.pdf`, `.md`, `.txt`, `.json`, `.html`) and intelligent processing pipeline routing.
- **Multi-Model Routing**: Automatically routes simpler queries to lightweight models (`ministral-3:3b`) and complex theological/analytical queries to heavy reasoning models (`qwen3:8b` / `ornith:9b`).
- **System Doctor Diagnostics (`paraclea doctor`)**: Self-diagnoses connectivity and verifies presence of all 4 key model categories (`Embedding`, `Chat`, `Heavy Reasoning`, `Document Vision OCR`).
- **Pure Rust Core**: Lightweight, fast, memory-efficient, and free from heavy Python environment dependencies.
- **Pocket TTS Speech Synthesis**: Smooth CPU-driven speech synthesis using natural female voice presets (`alba`, `cosette`, `eve`, `mary`, `vera`).
- **Gold & Purple UI**: Elegant terminal theme built in Rust.

---

## ⚡ One-Line Installation

Install Paraclea on any Linux or macOS system with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash
```

---

## 🚀 Usage & Commands

Run `paraclea` from anywhere in your terminal:

```bash
# Display help and version information
paraclea --help

# Run full system diagnostics (Ollama, Qdrant, TTS, Model Registry)
paraclea doctor

# List all available Ollama and local models
paraclea list

# Run Paraclea with a specific model by number or name
paraclea run 1
paraclea run ministral-3:3b

# Auto-detect file format and ingest into Qdrant vector database
paraclea ingest /path/to/document.png --collection books
paraclea ingest /path/to/notes.txt --collection survival

# Ingest a Bible JSON database into Qdrant
paraclea ingest-bible /path/to/kjv.json --collection bible

# Run vision OCR text extraction on an image document
paraclea ocr /path/to/scan.jpg

# One-shot RAG query with vector retrieval & Scripture citations
paraclea query "What does Proverbs 31 say about diligence?" --collection bible

# Interactive companion shell mode
paraclea
```

---

## 🛠️ System Architecture

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
                             │  │  Multi-Model Router & RAG │  │
                             │  │ (Ministral, Qwen3, Nomic) │  │
                             │  └─────────────┬─────────────┘  │
                             └────────────────┼────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────────────┐
              ▼                               ▼                               ▼
    ┌───────────────────┐           ┌───────────────────┐           ┌───────────────────┐
    │ Qdrant Vector DB  │           │ Vision OCR Engine │           │ Pocket TTS Engine │
    │ (Cosine Search)   │           │ (Unlimited OCR)   │           │ (CPU Speech API)  │
    └───────────────────┘           └───────────────────┘           └───────────────────┘
```

---

## 🎨 Theme & Customization

Paraclea features a **Gold & Purple** theme:
- **Gold (`#FFD700`)**: Prompts, status indicators, banner headers, tool logs.
- **Purple (`#B14AED`)**: Paraclea responses, borders, section headers, citations.

---

## 📜 License

Licensed under the [MIT License](LICENSE).
