# Changelog

All notable changes to the **Paraclea** project will be documented in this file.

## [0.3.0] - 2026-08-20

### Added
- **Ollama Unlimited Vision OCR (`frob/unlimited-ocr:q8_0`)**: Added Base64 document vision OCR integration via Ollama `/api/generate` API for document photo and scan text extraction (`src/ollama.rs`).
- **File Format Auto-Detection (`src/detect.rs`)**: Created `FileType` enum (`Image`, `Pdf`, `Epub`, `Docx`, `Text`, `Json`, `Html`, `Rtf`) for format labeling and automated pipeline dispatch.
- **Diagnostic System Doctor (`paraclea doctor`)**: Added system diagnostic tool inspecting Ollama server, Qdrant Vector DB, Pocket TTS engine, and verifying presence of key model categories (`Embedding`, `Chat`, `Heavy Reasoning`, `Document Vision OCR`).
- **Unified Ingestion Router (`paraclea ingest <file>`)**: Added single file ingestion command supporting text, markdown, JSON, and document image OCR indexing.
- **Direct Vision OCR CLI Command (`paraclea ocr <image>`)**: Added direct document OCR extraction CLI subcommand.
- **Updated One-Line Installer (`install.sh`)**: Updated installer script with platform detection (`x86_64`, `aarch64`), Qdrant binary download, model checks, and binary installation.

## [0.2.0] - 2026-08-20

### Added
- **Qdrant Vector Database HTTP Client (`src/qdrant.rs`)**: Integrated local Qdrant REST API client (`http://localhost:6333`) for vector collections (`bible`, `books`, `survival`).
- **Ollama Vector Embeddings (`src/ollama.rs`)**: Added `embed()` method generating 768-dimensional text embeddings via `nomic-embed-text`.
- **Bible & Book Ingestion Engine (`src/ingest.rs`)**: Created `BibleIngestor` and `BookIngestor` supporting 3-verse overlapping semantic chunking.
- **Vector RAG Retrieval & Multi-Model Router (`src/rag.rs`)**: Added `RagEngine` for query vector retrieval with Scripture citations and automatic model routing (`ministral-3:3b` vs `qwen3:8b`).
- **Proverbs 31 / Paraclea Helper Soul Protocol**: Updated `persona/SOUL.md` and `src/persona.rs` to the Paraclea (Παράκλησις) Helper identity.

## [0.1.0] - 2026-08-19

### Added
- **Pure Rust Engine Core**: Replaced python prototype with a fast, dependency-free Rust binary (`6.4 MB` release size).
- **Starling CLI Command Interface**: Integrated `clap` CLI parser allowing global command execution (`paraclea`, `paraclea list`, `paraclea run <num|name>`).
- **Gold & Purple Terminal UI**: Implemented vibrant Gold (`#FFD700`) and Purple (`#B14AED`) terminal styling.
- **OmniBot Persona Architecture**: Integrated dynamic markdown persona files (`IDENTITY.md`, `SOUL.md`, `USER.md`, `MEMORY.md`, `TOOLS.md`, `HEARTBEAT.md`).
- **Pocket TTS Integration**: Added `PocketTtsEngine` for CPU speech synthesis using cute female voices (`alba`, `cosette`, `eve`, `mary`, `vera`).
- **Offline Ollama Engine**: Added `OllamaClient` for local inference.
- **Single-Line Installer (`install.sh`)**: Added shell installer script for Linux/macOS with PATH setup.
