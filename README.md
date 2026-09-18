# Paraclea — Pure Rust AI Companion Engine & Multi-Category Library

> **A fast, lightweight, and completely offline AI companion engine built in pure Rust. Features a 160-version multi-language Bible database across 30 languages, complete Ellen G. White writings (174 chapters), 35-chapter wilderness survival & medical field manuals, self-improving Dendrite v2 knowledge graph memory, off-grid Reticulum mesh communications, a 3D Galaxy Atlas, and a dedicated Desktop Web Application GUI.**

---

## Overview & Purpose

**Paraclea (Παράκλησις)** is an independent local AI assistant and offline library engine designed to run 100% locally on CPU without cloud APIs or internet connectivity. It provides deep Bible study, multi-translation comparison, structured reading across spiritual, survival, medical, and educational literature, off-grid peer-to-peer mesh messaging, and personalized adaptive memory — accessible both via terminal REPL (`paraclea`) and desktop browser interface (`paraclea-gui`).

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
│   ├── paraclea-cli/                   # CLI binary — Gold/Purple styled terminal interface
│   ├── paraclea-gui/                   # GUI binary — Axum web server + Askama templates
│   ├── paraclea-tui/                   # TUI binary — ratatui terminal UI with 7 tabs
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

# 3. Install binaries to system PATH
install -m 755 target/release/paraclea ~/.local/bin/paraclea
install -m 755 target/release/paraclea-gui ~/.local/bin/paraclea-gui

# 4. Run diagnostic health doctor
paraclea doctor
```

---

## Running Paraclea

### 1. Terminal Companion REPL Shell (`paraclea`)
Run `paraclea` to launch the interactive terminal shell:
```bash
paraclea
```

#### Terminal Interactive Commands:
- `/bible` — Open the 100% alphabetical Bible language and translation selector.
- `/read` — Open the Unified Reader to browse Spiritual, Survival, Medical, and Educational books.
- `/compare` — Compare a Scripture verse side-by-side across multiple translations with AI study commentary.
- `/study` — Deep Scripture study with 340,000 Treasury of Scripture Knowledge cross-reference links.
- `/memory` — Inspect and search stored Dendrite v2 knowledge graph nodes.
- `/mesh` — View Reticulum off-grid network status, cryptographic identity, active peers, or broadcast messages.
- `/galaxy` — Open the 3D Galaxy Atlas (Mazzaroth engine).
- `/doctor` — Run full system diagnostic checks.
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
| **RAM Footprint** | ~25 MB base memory |
| **Bible Database** | 160 Formatted JSON Files (30 Languages, 66 Books) |
| **Non-Scripture Library** | 7 Books across 4 Categories (211 Total Chapters) |
| **Graph Database** | SQLite WAL Mode + FTS5 Full-Text Search |
| **Mesh Protocol** | Reticulum Network Stack (RNS 512-bit Crypto Identity) |
| **Galaxy Engine** | Mazzaroth — standalone, database-agnostic 3D renderer |
| **Supported OS** | Linux (ARM64/x86_64), macOS (Intel/Apple Silicon), Windows (x86_64) |
| **Tests** | 28/28 passing |
| **Clippy** | 0 warnings |

---

## Terminal Palette & Styling

Paraclea uses an elegant **Gold & Purple** terminal color palette:
- **Gold (`#FFD700`)**: Prompts, status indicators, banner headers, system doctor output.
- **Purple (`#B14AED`)**: Paraclea AI responses, section borders, chapter headings.

5 switchable themes: RoyalByzantium (Gold & Purple), CrimsonCodex (Red & Cream), CyberScholar (Cyan & Magenta), EmeraldMatrix (Mint & Green), CelestialMidnight (Indigo & Silver).

---

## License

Licensed under the [MIT License](LICENSE).
