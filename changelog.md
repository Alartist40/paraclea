# Changelog

All notable changes to the **Paraclea** project will be documented in this file.

## [0.9.0] - 2026-09-26

### Added
- **Toast Notifications**: Header toast system for theme changes, language/translation/model selection, backup status, stream errors, and cancellation — rendered on its own header row with a gold highlight for 4 seconds, never obscuring status badges.
- **Chat Auto-Scroll**: Conversation pins to the newest message during streaming and on send. Manual scrolling (Up/PageUp/wheel) disengages; scrolling back to the bottom or sending re-engages. Row math is unicode-width aware (`unicode-width`) so wrapped CJK/fullwidth text anchors correctly.
- **Stream Cancellation**: `Esc` anywhere aborts the running Ollama task via `AbortHandle`, bumps the stream generation id so stale tokens are dropped, and preserves the partial reply in history tagged `[cancelled]`.
- **Library Chapter Navigation**: `[`/`]` (or `p`/`n`, `h`/`l`) move between chapters with clamping against real volume chapter counts via `LibraryEngine::get_chapter_count`.
- **Mouse Wheel Scrolling**: Wheel scrolls Chat, Bible, and Library views (clamped, chat re-engages auto-scroll at bottom) and zooms the 3D Galaxy camera.
- **Alt+1–Alt+7**: Global deck switching from any focus, including while a modal is open (the modal closes on switch).
- **Stateful Modal Lists**: Language/translation pickers, Bible book list, Library lists, and the command palette all render via `ListState` + `render_stateful_widget`, so 66-language and 160-translation menus actually scroll.
- **Real Modal Filtering**: Picker search ranks exact tag → prefix → substring matches, preserves selection by item identity while typing, and shows an empty-state row when nothing matches.

### Changed
- **High-Contrast Foundations**: Explicit `theme.bg()` root frame fill plus `panel_bg()` on every block (26/26) and `input_bg()` on the prompt — no more terminal-background bleed-through. `tab_inactive`, `border_normal`, CrimsonCodex primary, and EmeraldMatrix secondary all rebalanced for AA/AAA contrast; zero `Color::DarkGray` remaining.
- **Keyboard Model Standardized**: The app boots into `MainViewport` focus so `1`–`7` switch decks immediately on launch; digits type freely in the prompt (`2 Corinthians`, `3 John`); `/` opens the command palette and `?` opens help from an empty prompt. All F1–F7 bindings and references removed.
- **Dynamic Status Counts**: Sidebar and Doctor show live Bible language/translation counts and library category counts instead of hardcoded values.
- **Chapter Clamping**: Book switching, `/bible Genesis 999`, and translation reloads all clamp to valid chapter ranges.
- **Toast Expiry**: Toasts clear from state after 4s (previously retained forever); stream failures now raise an error toast.

### Fixed
- `/clear` now aborts any in-flight stream and drops its tokens, so a cleared chat can no longer be repopulated by a finishing response.
- Galaxy mouse wheel zoom sign corrected — wheel-up zooms in (was: both directions zoomed out and could saturate at max distance).
- First scroll-up from auto-scroll no longer jumps to the top of the conversation (scroll syncs to the rendered bottom anchor before decrementing).
- Removed stale `total_lines - 10` scroll heuristics; keyboard and mouse handlers now clamp against the renderer-computed `chat_max_scroll`.
- Modal picker highlight no longer disappears past the visible rows of long lists.

## [0.8.0] - 2026-09-18

### Added
- **Mazzaroth Galaxy Engine (`crates/mazzaroth/`)**: Standalone, database-agnostic 3D celestial renderer extracted from Paraclea. Any program can implement the `DatabaseSchema` trait (3 required methods) to get a full 3D galaxy visualization with hierarchical orbital physics, theme-aware colors, mouse drag rotation, and pause/resume simulation.
- **Cross-Platform Portability**: Replaced all 23 `HOME` environment variable lookups with `dirs::home_dir()`, all 7 hardcoded `/tmp/` paths with `std::env::temp_dir()`, and all 6 `Command::new("sh")` calls with cross-platform `shell_command()`/`shell_arg()` helpers. Paraclea now runs on Linux (ARM64/x86_64), macOS, and Windows.
- **Platform-Specific Audio Playback**: Added macOS (`afplay`) and Windows (`PowerShell SoundPlayer`) audio fallbacks alongside Linux (`aplay`/`paplay`/`pw-play`).
- **Cross-Platform USB Backup Detection**: Added macOS (`/Volumes`) and Windows (drive letters `D:\`..`Z:\`) USB mount scanning for encrypted backups.
- **Theme Persistence**: Theme choice now persists across restarts via `Config.theme` field (YAML). Saved on every toggle.
- **CrimsonCodex Theme**: Replaced MonasteryAmber with CrimsonCodex (Deep Red & Cream) for better visual distinction. Changed CelestialMidnight secondary from gold to silver to avoid overlap.
- **Galaxy Pause**: `Space` now freezes both camera auto-spin AND orbital simulation. HUD shows "PAUSED" state.
- **Galaxy Mouse Drag**: Left-click drag rotates the galaxy camera.
- **Scroll Keys**: Added `Home`/`End` for Chat, Bible, and Library views. Added `PageUp`/`PageDown` for Library.
- **Tab Tile-Switching Architecture**: 3-tile layout with focus cycling (Sidebar → MainViewport → PromptInput) and visual border highlighting.

## [0.7.0] - 2026-08-23

### Added
- **Complete Multi-Language Bible Standardization & Deduplication (`scripts/organize_library.py`)**: Aggregated, deduplicated, and normalized 219 unique Bible versions across 30 languages into `~/.paraclea/bibles/<lang>/`.
- **Multi-Category Non-Scripture Library Engine (`src/library.rs`)**: Standardized organization for Ellen G. White writings, Medical field manuals, Wilderness Survival books, Psychology texts, and Classics.
- **Custom Cross-Reference Graph Linker (`src/crossref.rs`)**: Connected Scripture verses to non-scripture passages, storing bidirectional links in Dendrite graph memory.
- **Centralized Runtime Directory Architecture (`~/.paraclea/`)**: Updated all path resolution logic so configuration, persona, databases, and Qdrant vector storage live strictly inside `~/.paraclea/`.
- **Expanded System Doctor Metrics (`paraclea doctor`)**: Updated system doctor diagnostics to report covered Bible languages, formatted Bible versions, library categories, and total ingested books.

## [0.5.0] - 2026-08-23

### Added
- **Dendrite v2 Knowledge Graph Memory (`src/dendrite/`)**: Integrated Cynapse's 4-tier knowledge graph memory system with `[[wiki-links]]`, `#tags`, auto-wired bidirectional backlinks, and fast in-memory BM25 search.
- **SQLite WAL & FTS5 Full-Text Store (`src/dendrite/store.rs`)**: Thread-safe SQLite persistence for knowledge graph nodes with WAL mode and FTS5 full-text search triggers.
- **Asynchronous Background Reflection Worker (`src/dendrite/reflection.rs`)**: Non-blocking background Tokio task that distills conversation turns into user study habits, preferences, and key facts.
- **Dendrite Slash Command (`/memory` & `/dendrite`)**: Added `/memory` command to inspect graph nodes and perform instant FTS5/BM25 memory searches.
- **Advanced Self-Healing System Doctor (`run_doctor`)**: System hardware & CPU architecture probing, executable placement validation, live inference testing, Qdrant auto-repair, Reticulum daemon probing, and SQLite integrity checks.

## [0.4.0] - 2026-08-21

### Added
- **Reticulum Mesh Network Module (`src/mesh.rs`)**: Integrated zero-trust off-grid mesh engine using Reticulum Network Stack (RNS). Auto-discovers local devices over WiFi/Ethernet, serial lines, and LoRa radios without internet.
- **Reticulum Slash Commands (`/mesh`)**: Added `/mesh status`, `/mesh announce`, `/mesh peers`, and `/mesh identity` interactive commands.
- **Full Terminal Line Editing (`rustyline`)**: Integrated `rustyline::DefaultEditor` across all REPL prompts for native Left/Right arrow cursor movement, smooth backspacing, and command history scroll.
- **140+ CSV Bible Translations (`src/bible.rs`)**: Added `CsvBibleReader` parsing 140+ offline Bible translation files across 30+ languages.
- **Interactive Testament & Numbered Book Selector**: Added 3-tier navigation menu for Old Testament (39 books), New Testament (27 books), and direct search with alias normalization.

### Fixed
- **LLM Infinite Token Repetition Glitch**: Added `OllamaOptions` with `repeat_penalty: 1.18` and `num_predict: 1024` to eliminate repetitive phrase loops.
- **Startup CPU/Fan Spike Fix (`src/heartbeat.rs`)**: Consumed Tokio `interval` startup tick to prevent immediate background LLM memory reflection on launch.

## [0.3.0] - 2026-08-20

### Added
- **Ollama Unlimited Vision OCR (`frob/unlimited-ocr:q8_0`)**: Added Base64 document vision OCR integration via Ollama `/api/generate` API for document photo and scan text extraction.
- **File Format Auto-Detection (`src/detect.rs`)**: Created `FileType` enum for format labeling and automated pipeline dispatch.
- **Diagnostic System Doctor (`paraclea doctor`)**: Added system diagnostic tool inspecting Ollama server, Qdrant Vector DB, Pocket TTS engine, and verifying presence of key model categories.
- **Unified Ingestion Router (`paraclea ingest <file>`)**: Added single file ingestion command supporting text, markdown, JSON, and document image OCR indexing.
- **Direct Vision OCR CLI Command (`paraclea ocr <image>`)**: Added direct document OCR extraction CLI subcommand.
- **Updated One-Line Installer (`install.sh`)**: Updated installer script with platform detection, Qdrant binary download, model checks, and binary installation.

## [0.2.0] - 2026-08-20

### Added
- **Qdrant Vector Database HTTP Client (`src/qdrant.rs`)**: Integrated local Qdrant REST API client for vector collections.
- **Ollama Vector Embeddings (`src/ollama.rs`)**: Added `embed()` method generating 768-dimensional text embeddings via `nomic-embed-text`.
- **Bible & Book Ingestion Engine (`src/ingest.rs`)**: Created `BibleIngestor` and `BookIngestor` supporting 3-verse overlapping semantic chunking.
- **Vector RAG Retrieval & Multi-Model Router (`src/rag.rs`)**: Added `RagEngine` for query vector retrieval with Scripture citations and automatic model routing.
- **Proverbs 31 / Paraclea Helper Soul Protocol**: Updated persona files to the Paraclea (Παράκλησις) Helper identity.

## [0.1.0] - 2026-08-19

### Added
- **Pure Rust Engine Core**: Replaced python prototype with a fast, dependency-free Rust binary (6.4 MB release size).
- **Starling CLI Command Interface**: Integrated `clap` CLI parser allowing global command execution.
- **Gold & Purple Terminal UI**: Implemented vibrant Gold (`#FFD700`) and Purple (`#B14AED`) terminal styling.
- **OmniBot Persona Architecture**: Integrated dynamic markdown persona files.
- **Pocket TTS Integration**: Added `PocketTtsEngine` for CPU speech synthesis.
- **Offline Ollama Engine**: Added `OllamaClient` for local inference.
- **Single-Line Installer (`install.sh`)**: Added shell installer script for Linux/macOS with PATH setup.
