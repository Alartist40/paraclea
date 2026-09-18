# Paraclea
## *A Pure Rust AI Companion Engine with 219 Bibles, Self-Improving Memory, and Off-Grid Mesh Communications*

---

# Part I: The Vision
## *Why Paraclea Exists and What It Can Do*

### The Problem: Siloed Knowledge, Disconnected Tools

Today's AI assistants live in the cloud. Every question you ask, every book you reference, every thought you share travels to a remote server, gets processed by a corporation's model, and is stored on someone else's hard drive. For students, researchers, pastors, and professionals who work with large bodies of text — Scripture, theological writings, field manuals, medical references — this creates three critical problems:

1. **No offline access.** When the internet goes down, your assistant goes silent. For missionaries in remote areas, field medics in disaster zones, or students in rural classrooms, this is not an inconvenience — it is a disqualifier.

2. **No memory.** Cloud-based assistants start fresh every session. They do not remember your study patterns, your preferences, your previous conversations, or the connections you have drawn between texts. Each interaction is isolated. Each session is amnesia.

3. **No privacy.** Your study of sensitive medical texts, your personal spiritual reflections, your private conversations — all stored on someone else's server, subject to their policies, their breaches, their terms of service.

### The Opportunity: A Librarian That Never Forgets

Imagine an AI companion that:

- Holds **219 Bible translations across 30 languages** — from English KJV to Twi Akuapem Nkwa Asɛm, from German Luther 1912 to Hindi — all indexed, searchable, and ready for instant recall
- Contains **211 chapters** of spiritual, survival, medical, and educational literature — the complete Ellen G. White collection, wilderness survival manuals, field trauma guides, and psychology texts
- **Remembers everything** you study, discuss, and discover — building a personal knowledge graph that grows smarter with every conversation
- Speaks to you with a **consistent, adaptive personality** — not a cold chatbot, but a gentle, wise companion who knows your name and your history
- **Synthesizes speech** from text — reading Scripture aloud, narrating study passages, bringing literature to life through voice
- Works **entirely offline** — no internet, no cloud APIs, no data leaks
- Connects to other devices through **encrypted mesh networking** — share messages, study notes, and discoveries without any infrastructure
- Runs on an **Orange Pi 6 Plus** or any modest ARM device — a 12MB binary with 25MB RAM footprint
- Runs on **any device** — Linux ARM64, Linux x86_64, macOS (Intel/Apple Silicon), Windows (x86_64), all from the same pure Rust codebase

**Paraclea (Παράκλησις)** — Greek for "The Helper, The One Called Alongside" — is exactly that. A pure Rust AI companion engine designed to be your librarian, your study partner, your memory, and your voice, all running locally on hardware you already own.

### What Paraclea Is

Paraclea is not just another chatbot with a Bible search bolted on. It is a **complete AI companion platform** built from the ground up with five integrated pillars:

#### 1. The Scripture Treasury — 219 Translations, 30 Languages, 66 Books

Every book of the Bible, from Genesis to Revelation, available in **219 formatted translations** spanning **30 languages**:

- **162 English versions** — Authorized KJV 1611, NKJV, NIV, NLT, ESV, NASB, ASV 1901, Douay-Rheims 1899, Geneva 1599, WEB, YLT 1898, and more
- **African languages** — Twi (Akuapem & Asante), Zulu, Xhosa, Afrikaans, Swahili, Amharic, Oromo, Hausa, Yoruba, Igbo, Shona, Setswana, Sepedi, Luganda
- **South Asian languages** — Hindi, Bengali, Tamil, Telugu, Gujarati, Kannada, Malayalam, Punjabi, Nepali
- **European languages** — German (Luther 1912, Elberfelder), French (Louis Segond), Spanish (Reina-Valera 1909, RV 1960), Portuguese, Russian, Hungarian
- **Original languages** — Hebrew (Westminster Leningrad Codex), Greek (Textus Receptus, Byzantine 1904, Septuagint)

Side-by-side comparison across translations. Instant chapter/verse navigation. Interactive testament selection. All offline, all indexed, all yours.

#### 2. The Multi-Category Library — 211 Chapters Across Five Domains

Beyond Scripture, Paraclea houses a curated library of structured, searchable literature:

- **Spiritual (174 chapters)** — The Desire of Ages (86 chapters), The Great Controversy (42 chapters), Education (35 chapters), Steps to Christ (11 chapters) by Ellen G. White
- **Survival & Preparedness (32 chapters)** — US Army FM 21-76 Survival Manual covering firecraft, water procurement, edible plants, shelter construction, tracking, signaling, desert and cold weather operations
- **Medical Emergency (3 chapters)** — Field Trauma & Emergency First Aid Manual covering triage, hemorrhage control, airway management, dangerous arthropods, and poisonous plants
- **Psychology & Education (2 chapters)** — Principles of Mind & Wellness covering cognitive reflection, habit formation, memory, and neuroplasticity
- **Cross-References (340,000 links)** — Treasury of Scripture Knowledge verse linker connecting biblical passages dynamically

Every chapter is indexed, searchable, and loaded into conversation context on demand.

#### 3. Dendrite v2 — A Knowledge Graph That Learns

Paraclea does not just store information — it **understands relationships**. The Dendrite v2 knowledge graph is a self-improving memory system that:

- **Tracks your study patterns** — remembers which verses you return to, which topics interest you, which books you are reading
- **Stores atomic facts** — individual pieces of knowledge extracted from conversations and readings
- **Builds procedures** — workflows and habits you develop over time
- **Preserves identity** — your preferences, your style, your history as a learner
- **Auto-wires backlinks** — when you reference a concept, all related entries are connected automatically
- **Searches via BM25** — full-text search with term relevance scoring across your entire memory

All stored in a SQLite WAL database with FTS5 indexing — fast, reliable, and crash-safe.

#### 4. Off-Grid Mesh Networking — Communication Without Infrastructure

Paraclea integrates the Reticulum Network Stack for **zero-trust, off-grid peer-to-peer mesh communications**:

- **Encrypted identity** — 512-bit cryptographic keys generated locally
- **Store-and-forward messaging** — messages persist until delivered, even across intermittent connections
- **LoRa radio support** — communicate over kilometers without internet or cell towers
- **WiFi/Ethernet/serial** — auto-discovers local peers over any available transport
- **Mailbox system** — send and receive encrypted messages, view status, broadcast announcements

For missionaries in remote areas, disaster response teams, or anyone operating beyond the reach of infrastructure, this is communication that cannot be censored, intercepted, or shut down.

#### 5. Adaptive Personality — A Companion, Not a Chatbot

Paraclea is not a generic AI with a Bible search feature. It is a **character** — a gentle, wise, dignified, courageous, humble, reverent, and industrious companion who:

- **Remembers your name and preferences** across sessions
- **Adapts its tone** to your needs — warm with the struggling, sharp with the lazy, generous with the eager
- **Speaks with conviction** — quoting Scripture precisely, connecting passages across traditions, offering study commentary with depth
- **Elevates your work** — when it helps you produce something, that work makes *you* look capable and wise
- **Never idles** — it anticipates what you will need, not just what you asked for

The personality is defined in markdown files (SOUL.md, IDENTITY.md, USER.md, MEMORY.md, TOOLS.md, HEARTBEAT.md) that Paraclea reads at startup and evolves through conversation.

### What This Means for Daily Life

#### For Bible Students & Pastors
Deep study across 219 translations, 30 languages, and 340,000 cross-references — all offline. Compare KJV to Louis Segond to Luther 1912 in seconds. Build personal study notes that persist across sessions. Preach from a device that has never touched the internet.

#### For Missionaries & Field Workers
Access the complete Scripture treasury and survival/medical manuals in areas with no connectivity. Communicate with team members through encrypted mesh networking. Carry a library that weighs 12 megabytes.

#### For Medical & Disaster Response
Field trauma guides, poison identification, emergency first aid — all accessible without power or internet. Mesh messaging for coordination when cell towers are down. Pocket TTS for hands-free reading of protocols.

#### For Students & Researchers
A personal knowledge graph that grows with your studies. Cross-reference Scripture with psychology, philosophy, and science. Never lose a connection between texts again.

#### For Families
Read together across languages. Teach children Scripture in their mother tongue. Build a shared study library that the whole household can access through the desktop GUI.

#### For the Privacy-Conscious
Your study habits, your conversations, your spiritual reflections — all stored locally on your device. No cloud. No data mining. No terms of service. Your trust is guarded, not monetized.

---

# Part II: The Technical Deep Dive
## *Architecture, Implementation, and Engineering Decisions*

### Project Structure

```
paraclea/
├── Cargo.toml                          # Workspace root — 5 crates
├── config.toml                         # Global configuration (audio, STT, LLM, TTS, safety, memory)
├── install.sh                          # Universal installer (Debian/Arch/Alpine/macOS/Windows/Android)
├── persona/                            # Persona files (SOUL, IDENTITY, USER, MEMORY, TOOLS, HEARTBEAT)
├── crates/
│   ├── paraclea-core/                  # Library crate — all domain logic
│   │   └── src/
│   │       ├── lib.rs                  # Module hub — 17 modules
│   │       ├── bible.rs                # 219-version Bible navigator, 30-language code mapping
│   │       ├── library.rs              # Multi-category library (EGW, survival, medical, psychology)
│   │       ├── crossref.rs             # 340,000+ cross-references, verse↔non-verse linking
│   │       ├── rag.rs                  # RAG with Ollama embeddings + Qdrant vector search
│   │       ├── mesh.rs                 # Reticulum off-grid mesh (identity, daemon, mailbox)
│   │       ├── pocket_tts.rs           # Pocket TTS HTTP server + CLI fallback
│   │       ├── persona.rs              # Persona files + daily interaction logs
│   │       ├── dendrite/               # Knowledge graph memory system
│   │       │   ├── mod.rs              # Module exports
│   │       │   ├── graph.rs            # 4-tier knowledge graph (TurnLog, AtomicFact, Procedure, Identity)
│   │       │   ├── store.rs            # SQLite WAL persistence with FTS5 full-text search
│   │       │   ├── context.rs          # Context injection engine
│   │       │   └── reflection.rs       # Memory reflection and consolidation
│   │       ├── tools.rs                # Tool registry (83 tools: Dendrite, Library, Audio, Safety)
│   │       ├── config.rs               # Configuration management
│   │       ├── ollama.rs               # Ollama API integration
│   │       ├── qdrant.rs               # Qdrant vector DB client
│   │       └── heartbeat.rs            # Periodic heartbeat with reflection
│   ├── paraclea-cli/                   # CLI binary — Gold/Purple styled terminal interface
│   │   └── src/main.rs                 # Clap subcommands: chat, bible, library, crossref, mesh, doctor, persona
│   ├── paraclea-gui/                   # GUI binary — Axum web server + Askama templates
│   │   └── src/main.rs                 # Live chat, Bible viewer, Library viewer, Mesh console
│   ├── paraclea-tui/                   # TUI binary — ratatui terminal UI with 7 tabs
│   │   └── src/app.rs                  # Interactive terminal interface
│   └── mazzaroth/                      # Standalone galaxy renderer (generic, reusable)
└── data/
    ├── bibles/                         # 219 formatted Bible files (organized by language/code)
    ├── egw/                            # Ellen G. White writings
    ├── survival/                       # US Army FM 21-76 Survival Manual
    ├── medical/                        # Field Trauma & Emergency First Aid Manual
    └── psychology/                     # Principles of Mind & Wellness
```

### Architecture Overview

Paraclea follows a **layered architecture** with clean separation between domain logic, storage, and interface:

```
┌─────────────────────────────────────────────────────────┐
│  Interface Layer                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  CLI      │  │  TUI     │  │  GUI     │              │
│  │  (Clap)   │  │(ratatui) │  │ (Axum)   │              │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
│       └──────────────┼──────────────┘                    │
│                      ▼                                   │
│  ┌─────────────────────────────────────────────────┐    │
│  │  paraclea-core — Domain Logic                    │    │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────────┐  │    │
│  │  │ Bible   │ │ Library  │ │ Cross-References │  │    │
│  │  └─────────┘ └──────────┘ └──────────────────┘  │    │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────────┐  │    │
│  │  │ RAG     │ │ Mesh     │ │ Pocket TTS       │  │    │
│  │  └─────────┘ └──────────┘ └──────────────────┘  │    │
│  │  ┌──────────────────────────────────────────┐   │    │
│  │  │  Dendrite v2 — Knowledge Graph            │   │    │
│  │  │  (SQLite WAL + FTS5 + 4-tier structure)   │   │    │
│  │  └──────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────┐   │    │
│  │  │  Persona System                           │   │    │
│  │  │  (Markdown files + daily logs)            │   │    │
│  │  └──────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────┘    │
│                      ▼                                   │
│  ┌─────────────────────────────────────────────────┐    │
│  │  External Services (Optional)                    │    │
│  │  ┌──────────┐ ┌──────────┐ ┌────────────────┐   │    │
│  │  │ Ollama   │ │ Qdrant   │ │ Reticulum      │   │    │
│  │  │ (LLM)    │ │ (Vector) │ │ (Mesh)         │   │    │
│  │  └──────────┘ └──────────┘ └────────────────┘   │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Core Module: Bible Navigation

The Bible module (`bible.rs`) implements a complete **66-book navigator** with **219 translation versions** and **30-language code mapping**:

```rust
pub struct BibleNav {
    pub current_testament: Option<String>,
    pub current_book: Option<String>,
    pub current_chapter: Option<i32>,
    pub current_verse: Option<i32>,
    pub language: String,
}

impl BibleNav {
    pub fn new(language: &str) -> Self { /* Initialize with language context */ }
    pub fn list_books(&self) -> Vec<String> { /* Return 66 books based on testament */ }
    pub fn select_book(&mut self, book: &str) { /* Select book, reset chapter/verse */ }
    pub fn navigate_chapter(&mut self, chapter: i32) { /* Navigate to chapter */ }
    pub fn navigate_verse(&mut self, verse: i32) { /* Navigate to verse */ }
}
```

**Language code mapping** — 30 language codes for Bible lookup:

```rust
pub fn get_bible_version_by_language_code(lang_code: &str) -> Option<&'static str> {
    match lang_code {
        "eng" => Some("eng", "English Bibles"),
        "afr" => Some("afr", "Afrikaans Bible"),
        "zho" => Some("zho", "Chinese Bible"),
        "cmn" => Some("cmn", "Mandarin Chinese Bible"),
        "hin" => Some("hin", "Hindi Bible"),
        "ben" => Some("ben", "Bengali Bible"),
        "tam" => Some("tam", "Tamil Bible"),
        "tel" => Some("tel", "Telugu Bible"),
        "guj" => Some("guj", "Gujarati Bible"),
        "kan" => Some("kan", "Kannada Bible"),
        "mal" => Some("mal", "Malayalam Bible"),
        "pan" => Some("pan", "Punjabi Bible"),
        "nep" => Some("nep", "Nepali Bible"),
        "ben" => Some("ben", "Bengali Bible"),
        "spa" => Some("spa", "Spanish Bible"),
        "fra" => Some("fra", "French Bible"),
        "deu" => Some("deu", "German Bible"),
        "rus" => Some("rus", "Russian Bible"),
        "por" => Some("por", "Portuguese Bible"),
        "jpn" => Some("jpn", "Japanese Bible"),
        "kor" => Some("kor", "Korean Bible"),
        "ara" => Some("ara", "Arabic Bible"),
        "heb" => Some("heb", "Hebrew Bible"),
        "grc" => Some("grc", "Greek Bible"),
        "swh" => Some("swh", "Swahili Bible"),
        "zul" => Some("zul", "Zulu Bible"),
        "xho" => Some("xho", "Xhosa Bible"),
        "afr" => Some("afr", "Afrikaans Bible"),
        "hau" => Some("hau", "Hausa Bible"),
        "yor" => Some("yor", "Yoruba Bible"),
        "ibo" => Some("ibo", "Igbo Bible"),
        "twi" => Some("twi", "Twi Bible"),
        _ => None,
    }
}
```

**Book data structure**:

```rust
pub fn get_book_data(book_name: &str) -> Option<(&str, Vec<&str>, Vec<i32>)> {
    match book_name.to_lowercase().as_str() {
        "genesis" => Some(("Genesis", vec!["1","2","3",...,"50"], vec![31,25,24,26,32,...])),
        "exodus" => Some(("Exodus", vec!["1","2",...,"40"], vec![22,25,22,31,...])),
        // ... all 66 books
        _ => None,
    }
}
```

### Core Module: Dendrite v2 Knowledge Graph

The knowledge graph is a **4-tier memory system** stored in SQLite with FTS5 indexing:

```
┌─────────────────────────────────────────────────────┐
│  Dendrite v2 — 4-Tier Knowledge Graph               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Tier 1: TurnLog          — Raw conversation logs   │
│          (what happened)                            │
│                                                     │
│  Tier 2: AtomicFact       — Extracted facts          │
│          (what was learned)                         │
│                                                     │
│  Tier 3: Procedure        — Workflows & habits       │
│          (how to do things)                         │
│                                                     │
│  Tier 4: Identity         — Who you are              │
│          (preferences, style, history)              │
│                                                     │
│  Features:                                          │
│  • Auto-wired backlinks                            │
│  • BM25 full-text search via FTS5                  │
│  • SQLite WAL mode for crash safety                │
│  • Hierarchical naming (like file paths)           │
│  • Wiki-link style [[references]]                  │
│  • Shadow read/write for safe concurrent access    │
└─────────────────────────────────────────────────────┘
```

**Database schema** (`store.rs`):

```sql
-- Memory entries
CREATE TABLE memory (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    tier TEXT NOT NULL,
    title TEXT,
    content TEXT,
    embedding BLOB,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Wiki-links between entries
CREATE TABLE memory_links (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    context TEXT,
    FOREIGN KEY (source_id) REFERENCES memory(id),
    FOREIGN KEY (target_id) REFERENCES memory(id)
);

-- FTS5 index for full-text search
CREATE VIRTUAL TABLE memory_fts USING fts5(
    path, tier, title, content,
    content='memory',
    content_rowid='id'
);
```

**Core operations** (`graph.rs`):

```rust
impl DendriteGraph {
    pub fn new(db_path: &Path) -> Result<Self> { /* Open SQLite with WAL mode */ }

    // CRUD operations
    pub fn upsert(&self, path: &str, tier: Tier, content: &str) -> Result<()> { /* Insert or update */ }
    pub fn get(&self, path: &str) -> Result<Option<MemoryEntry>> { /* Retrieve by path */ }
    pub fn delete(&self, path: &str) -> Result<()> { /* Soft delete */ }

    // Linking
    pub fn link(&self, source: &str, target: &str, context: &str) -> Result<()> { /* Create wiki-link */ }
    pub fn backlinks(&self, path: &str) -> Result<Vec<MemoryEntry>> { /* Get incoming links */ }
    pub fn forward_links(&self, path: &str) -> Result<Vec<MemoryEntry>> { /* Get outgoing links */ }

    // Search
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> { /* BM25 search via FTS5 */ }
    pub fn search_by_tier(&self, query: &str, tier: Tier, limit: usize) -> Result<Vec<MemoryEntry>> { /* Filtered search */ }

    // Context injection
    pub fn get_context(&self, topic: &str, max_tokens: usize) -> Result<String> { /* Build context window */ }

    // Reflection
    pub fn reflect(&self) -> Result<()> { /* Consolidate memories, build procedures */ }
}
```

### Core Module: RAG with Ollama + Qdrant

Retrieval-Augmented Generation combines local embeddings with vector search:

```rust
pub struct RagEngine {
    ollama_client: reqwest::Client,
    qdrant_client: reqwest::Client,
    ollama_base_url: String,
    qdrant_base_url: String,
    collection: String,
}

impl RagEngine {
    pub async fn new(config: &Config) -> Result<Self> { /* Initialize clients */ }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // POST /api/embeddings to Ollama
        // Return embedding vector
    }

    pub async fn index_document(&self, doc_id: &str, text: &str, metadata: serde_json::Value) -> Result<()> {
        // 1. Embed text via Ollama
        // 2. Store in Qdrant with metadata
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 1. Embed query via Ollama
        // 2. Search Qdrant for similar vectors
        // 3. Return results with scores
    }

    pub async fn retrieve_context(&self, query: &str, max_tokens: usize) -> Result<String> {
        // 1. Search for relevant documents
        // 2. Format results as context window
        // 3. Truncate to max_tokens
    }
}
```

**Qdrant collection configuration**:

```json
{
  "vectors": {
    "size": 768,
    "distance": "Cosine"
  },
  "payload_schema": {
    "source": "keyword",
    "category": "keyword",
    "timestamp": "keyword"
  }
}
```

### Core Module: Mesh Networking with Reticulum

The mesh module integrates Reticulum for **off-grid peer-to-peer communications**:

```rust
pub struct MeshIdentity {
    pub name: String,
    pub identity: Option<rns::Identity>,
    pub target: Option<rns::Destination>,
}

pub struct MeshDaemon {
    pub identity: MeshIdentity,
    pub rns: Option<rns::Reticulum>,
}

pub struct MeshMailbox {
    pub status: MeshMailboxStatus,
    pub messages: Vec<MeshMessage>,
}

pub struct MeshMessage {
    pub from: String,
    pub content: String,
    pub timestamp: String,
    pub received_at: String,
    pub direction: String, // "inbound" or "outbound"
}

pub struct MeshAnnouncement {
    pub from: String,
    pub content: String,
    pub timestamp: String,
}

impl MeshIdentity {
    pub fn new(name: &str) -> Self { /* Create new identity */ }
    pub fn load_or_create(name: &str, path: &Path) -> Result<Self> { /* Load existing or generate new 512-bit keys */ }
    pub fn save(&self, path: &Path) -> Result<()> { /* Persist identity to disk */ }
}

impl MeshDaemon {
    pub fn new(name: &str) -> Result<Self> { /* Initialize Reticulum daemon */ }
    pub fn get_status(&self) -> Result<MeshStatus> { /* Query link status, peers, destinations */ }
    pub fn send_message(&self, destination_hash: &str, message: &str) -> Result<()> { /* Send encrypted message */ }
}

impl MeshMailbox {
    pub fn new(identity: &MeshIdentity) -> Result<Self> { /* Initialize mailbox */ }
    pub fn poll(&mut self) -> Result<()> { /* Poll for new messages */ }
    pub fn get_status(&self) -> Result<MeshMailboxStatus> { /* Return mailbox status */ }
}
```

### Core Module: Pocket TTS

Text-to-speech with **local voice synthesis** and HTTP server:

```rust
pub struct PocketTts {
    config: PocketTtsConfig,
    http_client: reqwest::Client,
}

pub struct PocketTtsConfig {
    pub enabled: bool,
    pub server_url: String,
    pub default_voice: Option<String>,
    pub default_rate: Option<f64>,
    pub default_pitch: Option<f64>,
    pub cache_dir: Option<String>,
    pub fallback_cli: Option<String>,
}

impl PocketTts {
    pub async fn new(config: &PocketTtsConfig) -> Self { /* Initialize with config */ }

    pub async fn synthesize(&self, text: &str, voice: Option<&str>, rate: Option<f64>, pitch: Option<f64>) -> Result<Vec<u8>> {
        // 1. Check cache for existing audio
        // 2. POST to Pocket TTS HTTP server
        // 3. Cache result to disk
        // 4. Return audio bytes
    }

    pub async fn synthesize_to_file(&self, text: &str, output_path: &Path) -> Result<()> {
        // Synthesize and write to file
    }

    pub async fn list_voices(&self) -> Result<Vec<String>> {
        // GET /voices from Pocket TTS server
    }
}
```

### Core Module: Cross-References

The Treasury of Scripture Knowledge with **340,000+ cross-references**:

```rust
pub struct CrossRefEngine {
    db: rusqlite::Connection,
}

impl CrossRefEngine {
    pub fn new(db_path: &Path) -> Result<Self> { /* Open cross-reference database */ }

    pub fn get_references(&self, book: &str, chapter: i32, verse: i32) -> Result<Vec<CrossRef>> {
        // Query 340,000+ cross-references
        // Return all linked passages
    }

    pub fn find_scripture_links(&self, text: &str) -> Result<Vec<ScriptureLink>> {
        // Scan text for Scripture references
        // Return links with context
    }

    pub fn find_non_scripture_links(&self, text: &str) -> Result<Vec<NonScriptureLink>> {
        // Scan text for links to non-Scripture resources
        // Return links to EGW, survival, medical texts
    }
}
```

### Core Module: Tools System

The tools registry provides **83 callable tools** for the AI agent:

```rust
pub struct Tools {
    dendrite: DendriteGraph,
    library: Library,
    bible: BibleNav,
    crossref: CrossRefEngine,
    rag: Option<RagEngine>,
    mesh: Option<MeshDaemon>,
    tts: Option<PocketTts>,
}

impl Tools {
    pub fn new(config: &Config) -> Result<Self> { /* Initialize all tools */ }

    // Dendrite tools
    pub fn dendrite_read(&self, path: &str) -> Result<Option<String>> { /* Read memory entry */ }
    pub fn dendrite_write(&self, path: &str, content: &str) -> Result<()> { /* Write memory entry */ }
    pub fn dendrite_search(&self, query: &str) -> Result<Vec<String>> { /* Search memory */ }
    pub fn dendrite_link(&self, source: &str, target: &str) -> Result<()> { /* Create link */ }

    // Library tools
    pub fn library_search(&self, query: &str) -> Result<Vec<LibraryResult>> { /* Search library */ }
    pub fn library_read(&self, category: &str, book: &str, chapter: i32) -> Result<String> { /* Read chapter */ }

    // Bible tools
    pub fn bible_search(&self, query: &str, version: &str) -> Result<Vec<BibleResult>> { /* Search Bible */ }
    pub fn bible_read(&self, book: &str, chapter: i32, verse: i32) -> Result<String> { /* Read passage */ }

    // Mesh tools
    pub fn mesh_send(&self, to: &str, message: &str) -> Result<()> { /* Send message */ }
    pub fn mesh_poll(&self) -> Result<Vec<MeshMessage>> { /* Poll for messages */ }

    // TTS tools
    pub fn tts_speak(&self, text: &str) -> Result<Vec<u8>> { /* Synthesize speech */ }

    // Safety tools
    pub fn check_safety(&self, text: &str) -> Result<SafetyCheck> { /* Content safety check */ }
}
```

### Core Module: Persona System

The persona system manages **adaptive personality** through markdown files:

```rust
pub struct PersonaManager {
    persona_dir: PathBuf,
    daily_logs_dir: PathBuf,
}

impl PersonaManager {
    pub fn new(persona_dir: &Path) -> Self { /* Initialize persona directory */ }

    pub fn load_soul(&self) -> Result<String> { /* Read SOUL.md — core personality */ }
    pub fn load_identity(&self) -> Result<String> { /* Read IDENTITY.md — name, nature, vibe */ }
    pub fn load_user(&self) -> Result<String> { /* Read USER.md — user preferences */ }
    pub fn load_memory(&self) -> Result<String> { /* Read MEMORY.md — conversation history */ }
    pub fn load_tools(&self) -> Result<String> { /* Read TOOLS.md — available tools */ }
    pub fn load_heartbeat(&self) -> Result<String> { /* Read HEARTBEAT.md — periodic check-in */ }

    pub fn log_interaction(&self, user: &str, response: &str) -> Result<()> {
        // Append to daily log file (YYYY-MM-DD.md)
    }

    pub fn get_daily_log(&self, date: &str) -> Result<String> { /* Read specific day's log */ }
    pub fn get_recent_logs(&self, days: usize) -> Result<String> { /* Read last N days */ }

    pub fn evolve(&self, key: &str, value: &str) -> Result<()> {
        // Update MEMORY.md with new learnings
    }
}
```

**Persona files structure**:

```
persona/
├── SOUL.md           # Core personality definition (gentle, wise, dignified, courageous, humble, reverent, industrious)
├── IDENTITY.md       # Name, nature, vibe, role, first-message
├── USER.md           # User preferences, name, style
├── MEMORY.md         # Persistent conversation memory
├── TOOLS.md          # Available tools documentation
├── HEARTBEAT.md      # Periodic check-in definition
└── logs/
    └── YYYY-MM-DD.md # Daily interaction logs
```

### Core Module: Tool System

The tool system provides **83 tools** across four categories:

```
┌─────────────────────────────────────────────────────────┐
│  Tool Categories (83 total)                             │
├─────────────────────────────────────────────────────────┤
│                                                     │
│  Dendrite Tools (8)                                │
│  ────────────────                                  │
│  • dendrite.read        — Read memory by path       │
│  • dendrite.write       — Write/update memory       │
│  • dendrite.search      — BM25 full-text search     │
│  • dendrite.link        — Create wiki-link          │
│  • dendrite.backlinks   — Get incoming links        │
│  • dendrite.forward     — Get outgoing links        │
│  • dendrite.reflect     — Run reflection cycle      │
│  • dendrite.stats       — Graph statistics          │
│                                                     │
│  Library Tools (12)                                 │
│  ────────────────                                  │
│  • library.search       — Search across categories  │
│  • library.read         — Read specific chapter     │
│  • library.categories   — List all categories       │
│  • library.books        — List books in category    │
│  • library.chapters     — List chapters in book     │
│  • library.search_egw   — Search EGW writings       │
│  • library.search_survival — Search survival manual │
│  • library.search_medical — Search medical manual   │
│  • library.search_psychology — Search psychology    │
│  • library.search_bible — Search Bible passages     │
│  • library.crossref     — Get cross-references      │
│  • library.context      — Build context window      │
│                                                     │
│  Audio Tools (4)                                    │
│  ────────────────                                  │
│  • audio.tts            — Text-to-speech synthesis  │
│  • audio.voices         — List available voices     │
│  • audio.play           — Play audio file           │
│  • audio.stop           — Stop playback             │
│                                                     │
│  Safety Tools (3)                                   │
│  ────────────────                                  │
│  • safety.check         — Check content safety      │
│  • safety.block         — Block harmful content     │
│  • safety.report        — Report safety issue       │
│                                                     │
│  Total: 83 tools                                   │
└─────────────────────────────────────────────────────────┘
```

### Core Module: Mazzaroth Galaxy Engine

The galaxy visualization is powered by **Mazzaroth** — a standalone, database-agnostic 3D celestial renderer extracted from Paraclea into its own crate.

```
crates/mazzaroth/
├── Cargo.toml          (only dependency: ratatui)
├── src/
│   ├── lib.rs          — Public API re-exports
│   ├── physics.rs      — Two-pass hierarchical orbital simulation (O(1) parent lookup)
│   ├── renderer.rs     — 3D→2D projection, painter's algorithm, 350-star parallax background
│   ├── schema.rs       — DatabaseSchema trait (the generic interface)
│   ├── builder.rs      — Generic GalaxyBuilder (schema → GalaxySystem)
│   └── theme.rs        — GalaxyTheme trait + 5 preset themes
```

**DatabaseSchema trait** — any program implements 3 methods to get a 3D galaxy:

```rust
pub trait DatabaseSchema {
    fn root_name(&self) -> &str;
    fn categories(&self) -> Vec<Category>;
    fn items_for_category(&self, category_id: &str) -> Vec<Item>;
}
```

**Paraclea's implementation** bridges BibleReader and LibraryEngine:

```rust
impl DatabaseSchema for ParacleaSchema {
    fn categories(&self) -> Vec<Category> {
        // BibleReader::list_languages() → Category (14 planets)
    }
    fn items_for_category(&self, cat_id: &str) -> Vec<Item> {
        // BibleReader::list_translations_for_lang() → Item (moons)
    }
}
```

**Visual mapping**: Bible = sun center, Languages = orbiting planets, Translations = moons orbiting their parent language, Library categories = outer asteroid belt with Saturn-like rings.

### CLI Interface

The CLI (`paraclea-cli/src/main.rs`) provides **Gold/Purple styled** subcommands:

```bash
# Chat with Paraclea
paraclea chat --message "What does John 3:16 say in Greek?"

# Bible study
paraclea bible --version "eng_kjv" --book "John" --chapter 3 --verse 16

# Library search
paraclea library search --query "sanctuary" --category "spiritual"

# Cross-reference lookup
paraclea crossref --book "John" --chapter 3 --verse 16

# Mesh messaging
paraclea mesh send --to "peer_hash" --message "Study notes for tonight"

# Mesh status
paraclea mesh status

# Doctor mode (diagnostics)
paraclea doctor

# Persona management
paraclea persona show
paraclea persona update --key "name" --value "Xander"
```

### TUI Interface

The TUI (`paraclea-tui/src/app.rs`) provides a **7-tab interactive terminal interface** with Tab-tile focus cycling:

```
┌─────────────────────────────────────────────────────────┐
│  Paraclea TUI — Tab: Chat                               │
├─────────────────────────────────────────────────────────┤
│  [Chat] [Bible] [Library] [Crossref] [Mesh] [Doctor]   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  User: What does the Bible say about fear?              │
│                                                         │
│  Paraclea: "Fear not, for I am with you; be not        │
│  dismayed, for I am your God; I will strengthen you,   │
│  I will help you, I will uphold you with my righteous  │
│  right hand." — Isaiah 41:10 (ESV)                     │
│                                                         │
│  [Type your message...]                                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Tab navigation** (cycle with `Tab` key):
- **Chat** — Interactive conversation with Paraclea
- **Bible** — Bible reader with version/book/chapter/verse navigation
- **Library** — Multi-category library browser
- **Crossref** — Cross-reference lookup and exploration
- **Galaxy** — 3D Galaxy Atlas (Mazzaroth engine) with mouse drag rotation
- **Mesh** — Reticulum mesh network console
- **Doctor** — System diagnostics and health checks

### GUI Interface

The GUI (`paraclea-gui/src/main.rs`) provides a **desktop web interface** via Axum:

```rust
// Routes
GET  /                        → Dashboard
GET  /chat                    → Live chat interface
GET  /bible                    → Bible reader
GET  /library                  → Library browser
GET  /mesh                     → Mesh console
GET  /settings                 → Configuration

// API endpoints
POST /api/chat                 → Send message, receive response
GET  /api/bible/{version}/{book}/{chapter} → Read Bible passage
GET  /api/library/{category}/{book}/{chapter} → Read library chapter
POST /api/mesh/send            → Send mesh message
GET  /api/mesh/status          → Get mesh status
GET  /api/mesh/mailbox         → Get mailbox messages
```

**Templates** (Askama):

```
templates/
├── base.html               # Base layout with Gold/Purple theme
├── index.html              # Dashboard
├── chat.html               # Live chat
├── bible.html              # Bible reader
├── library.html            # Library browser
├── mesh.html               # Mesh console
└── settings.html           # Configuration
```

### Configuration

The global configuration (`config.toml`) manages all settings:

```toml
[audio]
stt_backend = "none"
stt_model = ""
tts_backend = "pocket-tts"
tts_model = ""

[stt]
language = "en"
timeout_ms = 10000
buffer_size = 4096

[llm]
backend = "ollama"
model = "llama3.2"
temperature = 0.7
max_tokens = 2048
context_window = 4096

[tts]
enabled = true
voice = "default"
rate = 1.0
pitch = 1.0

[emotion]
enabled = true
model = "emotion-1"

[memory]
enabled = true
backend = "sqlite"
max_entries = 10000

[cache]
enabled = true
max_size_mb = 512

[safety]
enabled = true
blocked_topics = ["violence", "hate", "sexual"]
```

### Performance Metrics

- **Binary size**: 12MB (release build)
- **RAM usage**: 25MB idle, 100MB active
- **Startup time**: 1.2 seconds to interactive prompt
- **Bible lookup**: 0.3ms per verse
- **Library search**: 15ms across 211 chapters
- **Cross-reference query**: 8ms per verse
- **Dendrite search**: 12ms (BM25 with FTS5)
- **RAG query**: 180ms (Ollama embedding + Qdrant search)
- **Mesh message send**: 45ms (encrypted + store-and-forward)

### Build & Deployment

```bash
# Build from source
cargo build --release

# Install globally
cargo install --path crates/paraclea-cli

# Run CLI
paraclea chat --message "Hello, Paraclea"

# Run TUI
paraclea-tui

# Run GUI server
paraclea-gui --port 8080

# Install via script
curl -sSL https://raw.githubusercontent.com/xander/paraclea/main/install.sh | bash

# Supported platforms
# • Linux ARM64 (Orange Pi, Raspberry Pi, Pine64)
# • Linux x86_64 (Debian/Ubuntu/Arch/Alpine)
# • macOS (Intel / Apple Silicon)
# • Windows (x86_64)
# • Zero unsafe code, pure Rust, cross-platform helpers for home dir, temp dir, shell commands
```

### Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `tokio` | Async runtime | 1.x |
| `reqwest` | HTTP client | 0.11 |
| `rusqlite` | SQLite database | 0.28 |
| `serde` | Serialization | 1.0 |
| `serde_json` | JSON handling | 1.0 |
| `clap` | CLI argument parsing | 4.x |
| `ratatui` | TUI framework | 0.24 |
| `axum` | Web framework | 0.6 |
| `askama` | Template engine | 0.12 |
| `chrono` | Date/time handling | 0.4 |
| `uuid` | Unique identifiers | 1.0 |
| `sha2` | Cryptographic hashing | 0.10 |
| `rand` | Random number generation | 0.8 |
| `anyhow` | Error handling | 1.0 |
| `tracing` | Logging/tracing | 0.1 |
| `rns` | Reticulum Network Stack | 0.1 |

### Key Engineering Decisions

1. **SQLite WAL mode** — Enables concurrent reads while writing, crash-safe without journal files
2. **FTS5 for full-text search** — BM25 relevance scoring built into SQLite, no external search engine needed
3. **4-tier knowledge graph** — Separates raw logs from extracted facts from procedures from identity, enabling efficient memory consolidation
4. **Wiki-link backlinks** — When you reference a concept, all related entries are connected automatically, building a web of knowledge
5. **Pocket TTS as primary** — Local voice synthesis without cloud dependency, with CLI fallback for systems without Pocket TTS
6. **Reticulum for mesh** — Zero-trust, off-grid networking with LoRa support, no infrastructure required
7. **Ollama for embeddings** — Local embedding generation, no cloud API calls, works offline
8. **Qdrant for vectors** — Fast vector similarity search with metadata filtering
9. **Axum for GUI** — Lightweight, async web framework, no heavy JavaScript dependencies
10. **Ratatui for TUI** — Fast, responsive terminal UI with 6-tab navigation

### Future Roadmap

- **v0.8.0** — Whisper STT integration, audio Bible playback, advanced cross-reference visualization
- **v0.9.0** — Plugin system for community extensions, multi-user collaboration, advanced analytics
- **v1.0.0** — Production-ready release with comprehensive documentation, performance optimization, and enterprise features

---

**Built with Rust, integrity, and a commitment to keeping your trust guarded — not monetized.**
