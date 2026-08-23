# 🕊️ Paraclea — Pure Rust AI Companion Engine & Multi-Category Library

> **A fast, lightweight, and completely offline AI companion engine built in pure Rust. Features a 216-version multi-language Bible database, complete Ellen G. White writings, 40-chapter wilderness survival & medical field manuals, self-improving Dendrite v2 knowledge graph memory, and off-grid Reticulum mesh communications.**

---

## 🌟 Overview & Purpose

**Paraclea (Παράκλησις)** is an independent local AI assistant and offline library engine designed to run 100% locally on CPU without cloud APIs or internet connectivity. It provides deep Bible study, multi-translation comparison, structured reading across spiritual, survival, and educational literature, off-grid peer-to-peer mesh messaging, and personalized adaptive memory.

---

## 📊 Paraclea Database Scale & Capabilities

Paraclea houses an extensive offline database optimized for fast local recall and low resource usage:

### 📚 1. Multi-Language Bible Treasury
- **216 Formatted Bible Translations** spanning **29 Languages**, stored locally in `$HOME/.paraclea/bibles/` with 100% strict alphabetical menu navigation.
- **Languages Included**:
  - 🇬🇧 **English** (162 versions including *Authorized KJV 1611*, *NKJV*, *NIV*, *NLT*, *ESV*, *NASB*, *BSB*, *ASV 1901*, *Douay-Rheims 1899*, *Geneva 1599*, *WEB*, *YLT 1898*, *Revised Version 1885*, *Benton Septuagint*)
  - 🇪SPAN **Spanish (Español)** (*Reina-Valera 1909*, *RV 1960*, *Biblia del Siglo de Oro*)
  - 🇩🇪 **German (Deutsch)** (*Luther 1912*, *Elberfelder*, *Textbibel*)
  - 🇫🇷 **French (Français)** (*Louis Segond*)
  - 🇬🇭 **Twi** (*Akuapem Nkwa Asɛm*, *Asante Nkwa Asɛm*)
  - 🇿🇦 **Zulu (isiZulu)** & **Xhosa (isiXhosa)**
  - 🇿🇦 **Afrikaans** & **Sepedi (Northern Sotho)** & **Setswana**
  - 🇰🇪 **Swahili (Kiswahili)** & **Luganda**
  - 🇪🇹 **Amharic (አማርኛ)** & **Oromo**
  - 🇳🇬 **Hausa**, **Yoruba**, & **Igbo**
  - 🇿🇼 **Shona**
  - 🇮🇳 **Hindi (हिन्दी)**, **Bengali (বাংলা)**, **Tamil (தமிழ்)**, **Telugu (తెలుగు)**, **Gujarati (ગુજરાતી)**, **Kannada (ಕನ್ನಡ)**, **Malayalam (മലയാളം)**, & **Punjabi (ਪੰਜਾਬੀ)**
  - 🇮🇱 **Hebrew (עברית)** (*Westminster Leningrad Codex*) & 🇬🇷 **Greek (Ελληνικά)** (*Textus Receptus*, *Byzantine 1904*, *Septuagint*)
  - 🇵🇹 **Portuguese (Português)**, 🇷🇺 **Russian (Русский)**, 🇭🇺 **Hungarian (Magyar)**, & 🇳🇵 **Nepali (नेपाली)**

### 📖 2. Complete Multi-Category Library (`/read`)
Ingested multi-chapter JSON collections stored under `$HOME/.paraclea/library/`:

- **[1] Spiritual Category**:
  - **The Desire of Ages** (Ellen G. White — **86 Full Chapters**)
  - **The Great Controversy** (Ellen G. White — **42 Full Chapters**)
  - **Education** (Ellen G. White — **35 Full Chapters**)
  - **Steps to Christ** (Ellen G. White — **11 Full Chapters**)
- **[2] Survival & Medical Category**:
  - **Libre Survival & Bushcraft Manual (FM 21-76)** (US Army & Contributors — **32 Full Chapters** covering Firecraft, Water Procurement, Edible Plants, Shelter, Tracking, Ropes & Knots, Signaling, Desert & Cold Weather Operations)
  - **Field Trauma & Emergency First Aid Manual** (Medical Corps — **3 Full Chapters** covering Field Triage, Dangerous Arthropods, and Poisonous Plants)
- **[3] Educational Category**:
  - Psychology, Philosophy, and General Science Classics (*Principles of Mind & Psychology*)

### 🔗 3. 340,000 Scripture Cross-References & Graph Memory
- **340,000 Cross-References**: Treasury of Scripture Knowledge (TSK) verse linker connecting biblical passages dynamically.
- **Dendrite v2 Knowledge Graph Memory (`$HOME/.paraclea/dendrite.db`)**: SQLite WAL-backed graph memory that tracks user study patterns, preferences, and key notes across conversations.

---

## 🛠️ System Architecture

```
                             ┌─────────────────────────────────┐
                             │      PARACLEA CORE (Rust)       │
                             │                                 │
                             │  ┌───────────────────────────┐  │
                             │  │   Persona & SOUL System   │  │
                             │  │ (SOUL.md, IDENTITY, TOOLS)│  │
                             │  └─────────────┬─────────────┘  │
                             │                │                │
                             │                ▼                │
                             │  ┌───────────────────────────┐  │
                             │  │  Dendrite v2 Memory &     │  │
                             │  │  Multi-Model Router       │  │
                             │  └─────────────┬─────────────┘  │
                             └────────────────┼────────────────┘
                                              │
         ┌───────────────────┬────────────────┼───────────────────┬───────────────────┐
         ▼                   ▼                ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Qdrant Vector   │ │ Ollama LLM /    │ │ Reticulum Mesh  │ │ Pocket TTS      │ │ 216 Bibles &    │
│ Engine (RAG)    │ │ Vision OCR      │ │ Stack (LoRa)    │ │ Speech Engine   │ │ Library DB      │
└─────────────────┘ └─────────────────┘ └─────────────────┘ └─────────────────┘ └─────────────────┘
```

---

## ⚡ Installation & Setup

### Prerequisites
- **Rust Toolchain**: `cargo` (1.75+)
- **Ollama**: Running at `http://localhost:11434` with model `ministral-3:3b` or `qwen3.5:9b`
- **Qdrant Vector DB**: Running at `http://localhost:6333` (optional, for RAG features)
- **Python 3**: For database standardization script

### Automated One-Line Installation

```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash
```

### Manual Installation from Source

```bash
# 1. Clone repository
git clone https://github.com/Alartist40/paraclea.git
cd paraclea

# 2. Build release binary
cargo build --release

# 3. Install binary to system PATH
install -m 755 target/release/paraclea ~/.local/bin/paraclea

# 4. Standardize Bible and Library databases into ~/.paraclea/
python3 scripts/organize_library.py

# 5. Run diagnostic health doctor
paraclea doctor
```

---

## 🚀 Interactive Command Suite

Run `paraclea` to launch the interactive REPL shell:

```bash
# Launch Paraclea Interactive REPL
paraclea
```

### Interactive REPL Commands
- `/bible` — Open the 100% alphabetical Bible language and translation selector.
- `/read` — Open the Unified Reader to browse **Spiritual** (*Desire of Ages*, *Great Controversy*, *Education*, *Steps to Christ*), **Survival** (*FM 21-76*), and **Educational** books.
- `/compare` — Compare a Scripture verse side-by-side across multiple translations with AI study commentary.
- `/study` — Deep Scripture study with 340,000 Treasury of Scripture Knowledge cross-reference links.
- `/memory` — Inspect and search stored Dendrite v2 knowledge graph nodes.
- `/mesh` — View Reticulum off-grid network status, cryptographic identity (`RNS`), active peers, or broadcast messages.
- `/doctor` — Run full system diagnostic checks (Ollama, Qdrant, TTS, Reticulum, SQLite integrity).
- `/help` — Display command overview and usage instructions.
- `/bye` — Exit Paraclea.

---

## 💻 Technical Specifications

| Parameter | Specification |
| :--- | :--- |
| **Language** | 100% Pure Rust (Edition 2021) |
| **Binary Size** | ~12 MB (Release Profile) |
| **Idle CPU Overhead** | **0.0% CPU** |
| **RAM Footprint** | ~25 MB base memory |
| **Bible Database** | 216 Formatted JSON Files (29 Languages) |
| **Graph Database** | SQLite WAL Mode + FTS5 Full-Text Search |
| **Mesh Protocol** | Reticulum Network Stack (RNS 512-bit Crypto Identity) |
| **Supported OS** | Linux (aarch64 / x86_64), macOS |

---

## 🎨 Terminal Palette & Styling

Paraclea uses an elegant **Gold & Purple** terminal color palette:
- **Gold (`#FFD700`)**: Prompts, status indicators, banner headers, system doctor output.
- **Purple (`#B14AED`)**: Paraclea AI responses, section borders, chapter headings.

---

## 📜 License

Licensed under the [MIT License](LICENSE).
