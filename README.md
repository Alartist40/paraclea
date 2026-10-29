# Paraclea — Pure Rust AI Companion Engine & Multi-Category Library

> **A fast, lightweight, and completely offline AI companion engine built in pure Rust. Features a 160-version multi-language Bible database across 30 languages, complete Ellen G. White writings (174 chapters), 35-chapter wilderness survival & medical field manuals, self-improving Dendrite v2 knowledge graph memory, off-grid Reticulum mesh communications, a 3D Galaxy Atlas, and a dedicated Desktop Web Application GUI.**

---

## Overview & Purpose

**Paraclea (Παράκλησις)** is an independent local AI assistant and offline library engine designed to run 100% locally on CPU without cloud APIs or internet connectivity. It provides deep Bible study, multi-translation comparison, structured reading across spiritual, survival, medical, and educational literature, off-grid peer-to-peer mesh messaging, and personalized adaptive memory — accessible via terminal TUI (`paraclea`), the classic line REPL (`paraclea --repl`), and the desktop browser interface (`paraclea-gui`).

### Cross-Platform

Paraclea runs on **any device** from ARM64 single-board computers to x86 desktops:

| Platform | Status |
|----------|--------|
| Linux ARM64 (Orange Pi, Raspberry Pi) | Primary target |
| Linux x86_64 | Fully supported |
| macOS (Intel / Apple Silicon) | Fully supported |
| Windows (x86_64) | Fully supported |

All platform-specific code uses cross-platform helpers (`dirs::home_dir()`, `std::env::temp_dir()`, `#[cfg(target_os)]` gating). Zero `unsafe` code. Zero C dependencies in application code. Pure Rust throughout.

---

## Database Scale & Capabilities

### Multi-Language Bible Treasury (All 66 Books Covered)
- **160 Formatted Bible Translations** spanning **30 Languages**, stored locally in `~/.paraclea/bibles/` with 100% strict alphabetical menu navigation.
- **Complete Scripture Coverage**: Supports all **66 Books of the Holy Bible** (39 Old Testament + 27 New Testament books) with instant chapter selection and prev/next navigation.
- **Languages Included**:
  - **English** (162 versions including *Authorized KJV 1611*, *NKJV*, *NIV*, *NLT*, *ESV*, *NASB*, *BSB*, *ASV 1901*, *Douay-Rheims 1899*, *Geneva 1599*, *WEB*, *YLT 1898*, *Revised Version 1885*, *Benton Septuagint*)
  - **Spanish** (*Reina-Valera 1909*, *RV 1960*, *Biblia del Siglo de Oro*)
  - **German** (*Luther 1912*, *Elberfelder*, *Textbibel*)
  - **French** (*Louis Segond*)
  - **Twi** (*Akuapem Nkwa Asɛm*, *Asante Nkwa Asɛm*)
  - **Zulu** & **Xhosa**
  - **Afrikaans** & **Sepedi** & **Setswana**
  - **Swahili** & **Luganda**
  - **Amharic** & **Oromo**
  - **Hausa**, **Yoruba**, & **Igbo**
  - **Shona**
  - **Hindi**, **Bengali**, **Tamil**, **Telugu**, **Gujarati**, **Kannada**, **Malayalam**, & **Punjabi**
  - **Hebrew** (*Westminster Leningrad Codex*) & **Greek** (*Textus Receptus*, *Byzantine 1904*, *Septuagint*)
  - **Portuguese**, **Russian**, **Hungarian**, & **Nepali**

### Complete Multi-Category Library (211 Total Chapters)
Ingested multi-chapter JSON collections stored under `~/.paraclea/library/`:

- **[1] Spiritual Category (Ellen G. White — 174 Full Chapters)**:
  - **The Desire of Ages** (86 Full Chapters)
  - **The Great Controversy** (42 Full Chapters)
  - **Education** (35 Full Chapters)
  - **Steps to Christ** (11 Full Chapters)
- **[2] Survival Category (32 Full Chapters)**:
  - **Libre Survival & Bushcraft Manual (FM 21-76)** (32 Full Chapters)
- **[3] Medical Emergency Category (3 Full Chapters)**:
  - **Field Trauma & Emergency First Aid Manual** (3 Full Chapters)
- **[4] Psychology & Educational Category (2 Full Chapters)**:
  - **Principles of Mind & Wellness** (2 Full Chapters)

### 340,000 Scripture Cross-References & Graph Memory
- **340,000 Cross-References**: Treasury of Scripture Knowledge (TSK) verse linker connecting biblical passages dynamically.
- **Dendrite v2 Knowledge Graph Memory (`~/.paraclea/dendrite.db`)**: SQLite WAL-backed graph memory that tracks user study patterns, preferences, and key notes across conversations.

### 3D Galaxy Atlas (Mazzaroth Engine)
- **Mazzaroth**: Standalone, database-agnostic 3D celestial renderer extracted from Paraclea.
- **DatabaseSchema trait**: Any program can implement 3 methods to get a 3D galaxy visualization.
- **Bible = sun, Languages = planets, Translations = moons, Library = asteroid belt**.
- 350-star parallax background, theme-aware colors, mouse drag rotation, pause/resume simulation.

---

## System Architecture

```
                             +-----------------------------------------+
                             |      PARACLEA CORE (Rust)               |
                             |                                         |
                             |  +-----------------------------------+  |
                             |  |   Persona & SOUL System           |  |
                             |  | (SOUL.md, IDENTITY, TOOLS)        |  |
                             |  +-----------------+-----------------+  |
                             |                    |                    |
                             |                    v                    |
                             |  +-----------------------------------+  |
                             |  |  Dendrite v2 Memory &             |  |
                             |  |  Multi-Model Router               |  |
                             |  +-----------------+-----------------+  |
                             +--------------------+--------------------+
                                                  |
         +-------------------+--------------------+-------------------+-------------------+
         v                   v                    v                   v                   v
+-----------------+ +-----------------+ +-----------------+ +-----------------+ +-----------------+
| Qdrant Vector   | | Ollama LLM /    | | Reticulum Mesh  | | Pocket TTS      | | 160 Bibles &    |
| Engine (RAG)    | | Vision OCR      | | Stack (LoRa)    | | Speech Engine   | | 211 Ch Library  |
+-----------------+ +-----------------+ +-----------------+ +-----------------+ +-----------------+
```

### Crate Structure

```
paraclea/
├── Cargo.toml                          # Workspace root — 5 crates
├── crates/
│   ├── paraclea-core/                  # Library crate — all domain logic
│   ├── paraclea-cli/                   # CLI binary — launches the TUI (default) or REPL (`--repl`)
│   ├── paraclea-gui/                   # GUI binary — Axum web server + Askama templates
│   ├── paraclea-tui/                   # TUI library — ratatui interface with 7 decks (bin: paraclea)
│   └── mazzaroth/                      # Standalone galaxy renderer (generic, reusable)
├── persona/                            # Persona files (SOUL, IDENTITY, USER, MEMORY, TOOLS, HEARTBEAT)
└── data/                               # Bible & library data files
```

---

## Installation & Setup

### Prerequisites
- **Rust Toolchain**: `cargo` (1.75+)
- **Ollama**: Running at `http://localhost:11434` with model `ministral-3:3b` or `qwen3.5:9b`
- **Qdrant Vector DB**: Running at `http://localhost:6333` (optional, for RAG features)

### Automated One-Line Installation

```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash
```

### Manual Installation from Source

```bash
# 1. Clone repository
git clone https://github.com/Alartist40/paraclea.git
cd paraclea

# 2. Build release workspace binaries
cargo build --release --workspace

# 3. Install binaries to system PATH (install to both — PATH order varies by shell)
install -m 755 target/release/paraclea ~/.local/bin/paraclea
install -m 755 target/release/paraclea ~/.cargo/bin/paraclea
install -m 755 target/release/paraclea-gui ~/.local/bin/paraclea-gui

# 4. Run diagnostic health doctor
paraclea doctor
```

---

## Running Paraclea

### 1. Terminal Companion TUI (`paraclea`)
Run `paraclea` to launch the interactive ratatui terminal interface (7 decks: Chat, Bible, Library, Crossref, Galaxy, Mesh, Doctor):
```bash
paraclea            # TUI (default)
paraclea --repl     # classic line REPL
```

**Keyboard model** — boots into the viewport so deck shortcuts work immediately:
- `1`–`7` switch decks on launch; in the prompt, digits just type (`2 Corinthians`, `3 John`)
- `Alt+1`–`Alt+7` switch decks from anywhere; `Tab` cycles focus (Sidebar → Viewport → Prompt)
- `/` command palette, `?` help, `Esc` cancels AI generation (partial reply preserved), `Ctrl+T` cycles 6 themes (persisted)
- Chat auto-scrolls to the newest message; manual scrolling disengages, scrolling to the bottom re-engages
- Live Braille waveform audio spectrum (`⡀⣀⣄⣤⣦⣶⣿`) animates while speech audio is active
- Mouse wheel scrolls Chat/Bible/Library and zooms the Galaxy; all lists scroll (stateful selection)

#### Terminal Interactive Commands:
- `/bible` — Open the 100% alphabetical Bible language and translation selector (supports dual synchronized parallel reading via `c`, interlinear mode via `x`).
- `/read` — Open the Unified Reader to browse Spiritual, Survival, Medical, and Educational books.
- `/compare` — Compare a Scripture verse side-by-side across multiple translations with AI study commentary.
- `/interlinear` — Toggle Greek & Hebrew interlinear mode with Strong's tags and root lemmas.
- `/strongs` — Inspect Greek and Hebrew lexemes, transliterations, and biblical roots via Strong's Concordance.
- `/chains` — Explore prophetic synthesis chains linking Old Testament, New Testament, and Ellen G. White writings.
- `/dashboard` (or `/daily`) — View the field devotional briefing, daily scripture, active track, and mesh status.
- `/study` — Deep Scripture study with 340,000 Treasury of Scripture Knowledge cross-reference links.
- `/memory` — Inspect and search stored Dendrite v2 knowledge graph nodes.
- `/mesh` — View Reticulum off-grid network status, cryptographic identity, active peers, or broadcast messages.
- `/packets` — Real-time LoRa RNode RF signal oscilloscope (RSSI/SNR) and raw encrypted packet telemetry.
- `/voice` — Trigger Candle offline voice transcription & speech input.
- `/galaxy` — Open the 3D Galaxy Atlas with hierarchical ASCII topology (`@` Nexus, `o` Book, `*` Chapter, `.` Verse).
- `/doctor` — Run full system diagnostic checks.
- `/clear` — Clear the conversation (aborts any in-flight generation).
- `/help` — Display command overview and usage instructions.

### 2. Desktop Application Web GUI (`paraclea-gui`)
Run `paraclea-gui` to launch the desktop server and web interface:
```bash
paraclea-gui
```
- Listens on `http://127.0.0.1:7860` and automatically opens your desktop web browser.
- Features live AI chat, dropdown selectors for all 66 Bible books & 160 translations, category tab filtering for all 211 non-scripture library chapters, Dendrite memory inspector, Reticulum mesh stack status, and live System Doctor diagnostics.

---

## Technical Specifications

| Parameter | Specification |
| :--- | :--- |
| **Language** | 100% Pure Rust (Edition 2021) |
| **Binary Size** | ~12 MB (`paraclea`), ~14 MB (`paraclea-gui`) |
| **Idle CPU Overhead** | **0.0% CPU** |
| **Core Architecture** | 34 Pure Rust Modules (`crates/paraclea-core`) |
| **Database Engine** | Unified SQLite WAL Mode + FTS5 BM25 Virtual Table (`verses_fts`) |
| **Hallucination Shield** | ExactQuoteGuard (Verbatim Scripture Divergence Interceptor) |
| **RAM Footprint** | ~25 MB base memory |
| **Bible Database** | 160 Formatted JSON Files (30 Languages, 66 Books) |
| **Non-Scripture Library** | 7 Books across 4 Categories (211 Total Chapters) |
| **Lexicon Engine** | Strong's Greek & Hebrew Lexicon with tolerant lookup |
| **Vector Engine** | In-Process Embedded Vector Store (Zero-Daemon Pure Rust) |
| **Speech Engine** | Candle Pure Rust Offline STT (16kHz Log-Mel) + Async Audio Streamer |
| **Graph Database** | SQLite WAL Mode + FTS5 Full-Text Search |
| **Mesh Protocol** | Reticulum Network Stack (RNS 512-bit Crypto Identity) |
| **Audio Engine** | Asynchronous double-buffered jitter ring (<100ms TTFS) |
| **Galaxy Engine** | Mazzaroth — standalone, database-agnostic 3D renderer |
| **Supported OS** | Linux (ARM64/x86_64), macOS (Intel/Apple Silicon), Windows (x86_64) |
| **Edge Acceleration** | Needle 3 Co-Processor (28.4 MB L3-Resident / Deterministic JSON) |
| **Tests** | 69/69 passing |
| **Clippy** | 0 warnings (`--all-targets`) |

---

## Architectural Guides & Technical Documentation

- **[Communication & Anti-Slop Optimization](docs/COMMUNICATION_AND_SLOP_OPTIMIZATION.md)**: SlopMonster-inspired 32-token micro-directive eliminating conversational AI boilerplate.
- **[Dendrite AutoHarness & Context Optimization](docs/DENDRITE_AUTOHARNESS_AND_CONTEXT_OPTIMIZATION.md)**: Biological self-learning, opportunity-relative dynamic pruning, scripture read receipts, and massive database context management for small local models.
- **[Audio Streaming Optimization Guide](docs/AUDIO_STREAMING_OPTIMIZATION.md)**: Asynchronous double-buffered jitter ring and low-latency audio playback pipeline.
- **[Reticulum Off-Grid Guide](docs/RETICULUM_OFFGRID_GUIDE.md)**: Autonomous cryptographic mesh networking and packet forwarding.
- **[USB Sneakernet Sync Guide](docs/USB_SNEAKERNET_SYNC_GUIDE.md)**: Air-gapped database synchronization via USB storage devices.
- **[AppImage Updates Guide](docs/APPIMAGE_UPDATES_GUIDE.md)**: Self-contained packaging and update distribution.

---

## Terminal Palette & Styling

Paraclea includes **6 switchable themes** (`Ctrl+T` or `/theme`):
- **RoyalByzantium**: Gold & Imperial Purple (Default)
- **CrimsonCodex**: Deep Red & Cream
- **CyberScholar**: High-Tech Cyan & Magenta
- **EmeraldMatrix**: Mint & Deep Matrix Green
- **CelestialMidnight**: Indigo & Stellar Silver
- **WildernessEInk**: Ultra-high-contrast paper white & obsidian ink (Zero-glare for outdoor sunlight & e-paper)

---

## License

Licensed under the [MIT License](LICENSE).
