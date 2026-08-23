# Changelog

All notable changes to the **Paraclea** project will be documented in this file.

## [0.7.0] - 2026-08-23

### Added
- **Complete Multi-Language Bible Standardization & Deduplication (`scripts/organize_library.py`)**: Aggregated, deduplicated, and normalized 219 unique Bible versions across 30 languages (including Twi Akuapem & Asante, Zulu, Xhosa, Afrikaans, Swahili, Amharic, Hausa, Yoruba, Igbo, Luganda, Shona, Setswana, Sepedi, German, French, Spanish, Hebrew, Arabic, Hindi, Tamil, Telugu, etc.) into `$HOME/.paraclea/bibles/<lang>/`.
- **Multi-Category Non-Scripture Library Engine (`src/library.rs`)**: Standardized organization for Ellen G. White writings (`egw`), Medical field manuals (`medical`), Wilderness Survival books (`survival`), Psychology texts (`psychology`), and Classics (`classics`).
- **Custom Cross-Reference Graph Linker (`src/crossref.rs`)**: Connected Scripture verses (`Genesis 1:1`) to non-scripture passages (`psychology/principles_of_mind Ch 1`), storing bidirectional links in Dendrite graph memory (`$HOME/.paraclea/dendrite.db`).
- **Centralized Runtime Directory Architecture (`$HOME/.paraclea/`)**: Updated all path resolution logic (`src/config.rs`, `src/persona.rs`, `src/main.rs`) so configuration, persona, databases, and Qdrant vector storage live strictly inside `$HOME/.paraclea/`.
- **Expanded System Doctor Metrics (`paraclea doctor`)**: Updated system doctor diagnostics to report covered Bible languages, formatted Bible versions, library categories, and total ingested books.

## [0.5.0] - 2026-08-23

### Added
- **Dendrite v2 Knowledge Graph Memory (`src/dendrite/`)**: Integrated Cynapse's 4-tier knowledge graph memory system (`TurnLog`, `AtomicFact`, `Procedure`, `Identity`) with `[[wiki-links]]`, `#tags`, auto-wired bidirectional backlinks, and fast in-memory BM25 search.
- **SQLite WAL & FTS5 Full-Text Store (`src/dendrite/store.rs`)**: Thread-safe SQLite persistence for knowledge graph nodes in `$HOME/.paraclea/dendrite.db` with WAL mode and FTS5 full-text search triggers.
- **Asynchronous Background Reflection Worker (`src/dendrite/reflection.rs`)**: Non-blocking background Tokio task that distills conversation turns into user study habits, preferences, and key facts without slowing down live response output.
- **Dendrite Slash Command (`/memory` & `/dendrite`)**: Added `/memory` command to inspect graph nodes and perform instant FTS5/BM25 memory searches (`/memory search <query>`).
- **Advanced Self-Healing System Doctor (`run_doctor`)**: Upgraded `paraclea doctor` based on LeafcutterLLM:
  - System hardware & CPU architecture probing (`aarch64` / `x86_64`, logical threads, OS target).
  - Executable placement and PATH validation with auto-installation to `~/.local/bin/paraclea`.
  - Active 1-token live forward-pass inference test on Ollama LLMs with duration timing in ms.
  - Qdrant Vector DB daemon auto-repair & collection auto-creation (`bible`, `books`).
  - Reticulum Mesh daemon probing (`rnsd`) & identity key verification.
  - Dendrite SQLite database integrity check & node count reporting.

## [0.4.0] - 2026-08-21

### Added
- **Reticulum Mesh Network Module (`src/mesh.rs`)**: Integrated zero-trust off-grid mesh engine using Reticulum Network Stack (RNS). Auto-discovers local devices over WiFi/Ethernet, serial lines, and LoRa radios without internet.
- **Reticulum Slash Commands (`/mesh`)**: Added `/mesh status`, `/mesh announce`, `/mesh peers`, and `/mesh identity` interactive commands.
- **Full Terminal Line Editing (`rustyline`)**: Integrated `rustyline::DefaultEditor` across all REPL prompts for native Left/Right arrow cursor movement, smooth backspacing, and command history scroll.
- **140+ CSV Bible Translations (`src/bible.rs`)**: Added `CsvBibleReader` parsing 140+ offline Bible translation files across 30+ languages.
- **Interactive Testament & Numbered Book Selector**: Added 3-tier navigation menu for Old Testament (39 books), New Testament (27 books), and direct search with alias normalization ("songs of solomon" -> "Song of Solomon").

### Fixed
- **LLM Infinite Token Repetition Glitch**: Added `OllamaOptions` with `repeat_penalty: 1.18` and `num_predict: 1024` to eliminate repetitive phrase loops.
- **Startup CPU/Fan Spike Fix (`src/heartbeat.rs`)**: Consumed Tokio `interval` startup tick to prevent immediate background LLM memory reflection on launch.

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
