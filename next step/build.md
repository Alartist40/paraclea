Here is your complete, step-by-step build guide for Paraclea. Rust-first. No bloat. Only what you need, with explanations for every tool.

---

## Phase 0: Prerequisites

Install these once on your build machine (laptop or OPI 6+):

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Ollama (Go binary, single install)
curl -fsSL https://ollama.com/install.sh | sh

# Git
sudo apt-get install git build-essential cmake pkg-config libssl-dev
```

Hardware needed:
- **Hub:** Laptop or OPI 6+ / RPi 5 (8GB+ RAM recommended)
- **Sensor:** ESP32-S3 Sense
- **Storage:** 128GB+ SD card or SSD for models + data

---

## Phase 1: Download Core Data & Tools

### Step 1.1 — Clone the Bible Database (Data Only)

```bash
mkdir -p ~/paraclea && cd ~/paraclea
git clone https://github.com/scrollmapper/bible_databases.git
```

This gives you JSON, CSV, and SQLite files. No Python needed to use the data.

### Step 1.2 — Download Qdrant (Single Binary)

```bash
cd ~/paraclea
# For ARM64 (OPI 6+, RPi 5):
wget https://github.com/qdrant/qdrant/releases/latest/download/qdrant-aarch64-unknown-linux-gnu.tar.gz
tar -xzf qdrant-aarch64-unknown-linux-gnu.tar.gz
chmod +x qdrant
```

Qdrant is a **vector database** written in Rust. It stores embeddings (numerical representations of text) so Paraclea can find relevant Bible verses by meaning, not just keyword matching. Single binary, no Docker, no MySQL.

### Step 1.3 — Pull Ollama Models

```bash
# The workhorse: 3B params, ~2GB, 256k context, vision-capable
ollama pull ministral-3:3b

# The embedding engine: 274MB, turns text into vectors
ollama pull nomic-embed-text

# The heavy lifter for deep reasoning (optional, ~5GB)
ollama pull qwen3:8b
```

**What each model does:**
- `ministral-3:3b` — Answers questions, reads images from ESP32 cam, routes tool calls. Default for everything.
- `nomic-embed-text` — Converts Bible verses and your notes into vectors. Never speaks; only embeds.
- `qwen3:8b` — Handles complex theology, sermon writing, multi-step reasoning. Slower but deeper. Paraclea calls it automatically when needed.

### Step 1.4 — book-to-skill (One-Time Python Preprocessor)

```bash
cd ~/paraclea
git clone https://github.com/virgiliojr94/book-to-skill.git
```

**What it does:** Converts PDF/EPUB books into structured Markdown skills. You only run this when adding a new book. It outputs `SKILL.md`, `chapters/`, `glossary.md`, etc. Your Rust ingestion tool reads these Markdown files.

**You do not need to understand its Python internals.** Treat it like `gcc` — it is a compiler you invoke, not code you maintain.

---

## Phase 2: Create the Rust Project

### Step 2.1 — Scaffold

```bash
cd ~/paraclea
cargo new --bin paraclea-hub
cd paraclea-hub
```

### Step 2.2 — Cargo.toml

```toml
[package]
name = "paraclea-hub"
version = "0.1.0"
edition = "2021"

[dependencies]
# HTTP server & client
axum = "0.7"
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Bible data parsing
csv = "1.3"

# Async traits
async-trait = "0.1"

# Time & scheduling for heartbeat
chrono = "0.4"
tokio-cron-scheduler = "0.13"

# TOML for persona config
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Optional: SQLite for mutable memory/notes
rusqlite = { version = "0.32", features = ["bundled"] }

# Optional: WebSocket for ESP32
tokio-tungstenite = "0.23"
futures = "0.3"
```

### Step 2.3 — Directory Structure

```
paraclea-hub/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, starts HTTP server + Qdrant
│   ├── router.rs         # Axum routes (REST API + WebSocket)
│   ├── models.rs         # Data structs (Verse, Chunk, Query, etc.)
│   ├── ollama.rs         # Talks to Ollama HTTP API
│   ├── qdrant.rs         # Talks to Qdrant HTTP API
│   ├── ingest.rs         # Reads Bible JSON/CSV, chunks, embeds
│   ├── memory.rs         # Reads/writes persona files & SQLite notes
│   ├── tts.rs            # Pocket TTS integration
│   ├── stt.rs            # Whisper.cpp integration
│   └── config.rs         # Loads config.toml
├── config.toml           # Hub settings
└── data/
    ├── persona/          # SOUL.md, IDENTITY.md, MEMORY.md, etc.
    ├── bible/            # Read-only Bible SQLite/JSON
    ├── notes/            # Mutable daily logs
    └── books/            # book-to-skill outputs go here
```

---

## Phase 3: Build the Components

### Step 3.1 — Config File (`config.toml`)

```toml
[ollama]
url = "http://localhost:11434"
default_model = "ministral-3:3b"
embed_model = "nomic-embed-text"
heavy_model = "qwen3:8b"

[qdrant]
url = "http://localhost:6333"
collection_bible = "bible"
collection_books = "books"
collection_survival = "survival"

[hub]
listen_addr = "0.0.0.0:3000"
data_dir = "./data"
heartbeat_interval_minutes = 60

[esp32]
enabled = true
websocket_port = 3001
```

### Step 3.2 — Core Data Models (`src/models.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BibleVerse {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
    pub translation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextChunk {
    pub id: String,
    pub text: String,
    pub source: String,      // "kjv", "egw", "survival"
    pub book: Option<String>,
    pub chapter: Option<u32>,
    pub verses: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
    pub context: Option<String>, // "bible", "survival", "all"
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<String>,
    pub model_used: String,
}
```

### Step 3.3 — Ollama Client (`src/ollama.rs`)

```rust
use reqwest::Client;
use serde_json::json;

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    // Generate embedding vector for a text chunk
    pub async fn embed(&self, text: &str, model: &str) -> anyhow::Result<Vec<f32>> {
        let resp = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&json!({
                "model": model,
                "prompt": text
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let embedding = resp["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No embedding in response"))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    // Chat with a model, streaming response
    pub async fn chat(
        &self,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let resp = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": model,
                "system": system,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": 0.3,
                    "num_ctx": 8192
                }
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp["response"].as_str().unwrap_or("").to_string())
    }
}
```

### Step 3.4 — Qdrant Client (`src/qdrant.rs`)

```rust
use reqwest::Client;
use serde_json::json;

pub struct QdrantClient {
    client: Client,
    base_url: String,
}

impl QdrantClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn create_collection(&self, name: &str, dim: usize) -> anyhow::Result<()> {
        self.client
            .put(format!("{}/collections/{}", self.base_url, name))
            .json(&json!({
                "vectors": {
                    "size": dim,
                    "distance": "Cosine"
                }
            }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn upsert(
        &self,
        collection: &str,
        id: String,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.client
            .put(format!("{}/collections/{}/points", self.base_url, collection))
            .json(&json!({
                "points": [{
                    "id": id,
                    "vector": vector,
                    "payload": payload
                }]
            }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: u64,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self
            .client
            .post(format!("{}/collections/{}/points/search", self.base_url, collection))
            .json(&json!({
                "vector": vector,
                "limit": limit,
                "with_payload": true
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp["result"].as_array().cloned().unwrap_or_default())
    }
}
```

### Step 3.5 — Bible Ingestion (`src/ingest.rs`)

This reads `bible_databases/json/kjv.json` and populates Qdrant:

```rust
use crate::{ollama::OllamaClient, qdrant::QdrantClient};
use serde_json::Value;
use std::fs;

pub async fn ingest_bible(
    ollama: &OllamaClient,
    qdrant: &QdrantClient,
    json_path: &str,
) -> anyhow::Result<()> {
    let data = fs::read_to_string(json_path)?;
    let bible: Value = serde_json::from_str(&data)?;

    // bible_databases JSON structure: {"Genesis": {"1": {"1": "In the beginning..."}}}
    for (book, chapters) in bible.as_object().unwrap() {
        for (chapter, verses) in chapters.as_object().unwrap() {
            for (verse_num, verse_text) in verses.as_object().unwrap() {
                let text = verse_text.as_str().unwrap();
                let id = format!("{}-{}-{}", book, chapter, verse_num);
                
                // Chunk: group 3 verses together for semantic coherence
                // (Simplified: here we do single verses; in production, buffer 3)
                let embedding = ollama.embed(text, "nomic-embed-text").await?;
                
                let payload = serde_json::json!({
                    "book": book,
                    "chapter": chapter.parse::<u32>().unwrap_or(0),
                    "verse": verse_num.parse::<u32>().unwrap_or(0),
                    "text": text,
                    "source": "kjv"
                });

                qdrant.upsert("bible", id, embedding, payload).await?;
            }
        }
    }
    println!("Bible ingestion complete");
    Ok(())
}
```

**Run this once:** `cargo run --bin ingest` (or make it a subcommand). It takes ~30 minutes for the full Bible because each verse calls Ollama for embedding. After this, Qdrant stores everything.

### Step 3.6 — The Query Router (`src/router.rs`)

```rust
use crate::{ollama::OllamaClient, qdrant::QdrantClient, models::*};
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

pub struct AppState {
    pub ollama: OllamaClient,
    pub qdrant: QdrantClient,
    pub soul: String, // contents of SOUL.md
}

pub async fn query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    // 1. Embed the user's question
    let query_vec = match state.ollama.embed(&req.question, "nomic-embed-text").await {
        Ok(v) => v,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };

    // 2. Decide which collection to search
    let collection = match req.context.as_deref() {
        Some("bible") => "bible",
        Some("survival") => "survival",
        _ => "bible", // default
    };

    // 3. Retrieve top 5 relevant chunks from Qdrant
    let results = match state.qdrant.search(collection, query_vec, 5).await {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };

    // 4. Build context string from retrieved verses
    let mut context = String::new();
    let mut sources = Vec::new();
    for r in results {
        if let (Some(text), Some(book)) = (
            r["payload"]["text"].as_str(),
            r["payload"]["book"].as_str(),
        ) {
            context.push_str(&format!("[{} {}:{}] {}\n\n", 
                book, 
                r["payload"]["chapter"].as_u64().unwrap_or(0),
                r["payload"]["verse"].as_u64().unwrap_or(0),
                text
            ));
            sources.push(format!("{} {}:{}", book, 
                r["payload"]["chapter"].as_u64().unwrap_or(0),
                r["payload"]["verse"].as_u64().unwrap_or(0)
            ));
        }
    }

    // 5. Decide model (simple heuristic)
    let model = if req.question.len() > 200 
        || req.question.contains("analyze") 
        || req.question.contains("compare") {
        "qwen3:8b"
    } else {
        "ministral-3:3b"
    };

    // 6. Build prompt
    let prompt = format!(
        "Use the following Scripture to answer the question. Quote verses precisely.\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer with gentleness, wisdom, and Scripture references.",
        context, req.question
    );

    // 7. Generate answer
    let answer = match state.ollama.chat(model, &state.soul, &prompt).await {
        Ok(a) => a,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };

    Json(serde_json::to_value(QueryResponse {
        answer,
        sources,
        model_used: model.to_string(),
    }).unwrap())
}
```

### Step 3.7 — Main Entry Point (`src/main.rs`)

```rust
mod config;
mod ingest;
mod models;
mod ollama;
mod qdrant;
mod router;
mod memory;

use axum::{routing::post, Router};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // 1. Start Qdrant (or assume it is already running)
    info!("Starting Paraclea Hub...");
    
    // 2. Load persona
    let soul = std::fs::read_to_string("data/persona/SOUL.md")
        .unwrap_or_else(|_| "You are Paraclea, a helpful assistant.".to_string());

    // 3. Connect to services
    let ollama = ollama::OllamaClient::new("http://localhost:11434".to_string());
    let qdrant = qdrant::QdrantClient::new("http://localhost:6333".to_string());

    // 4. Ensure collections exist (run once)
    qdrant.create_collection("bible", 768).await.ok();
    qdrant.create_collection("books", 768).await.ok();
    qdrant.create_collection("survival", 768).await.ok();

    let state = Arc::new(router::AppState {
        ollama,
        qdrant,
        soul,
    });

    // 5. Start HTTP API
    let app = Router::new()
        .route("/api/query", post(router::query))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Paraclea Hub listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Phase 4: Persona Files (The Soul)

Create these files before first run:

```bash
mkdir -p ~/paraclea/paraclea-hub/data/persona
mkdir -p ~/paraclea/paraclea-hub/data/notes/daily
```

### `data/persona/SOUL.md`
Use the full Proverbs 31 personality we built earlier. Paraclea reads this on startup and injects it as the system prompt.

### `data/persona/IDENTITY.md`
```
Name: Paraclea
Nature: Helper, Companion, Multiplier
Emoji: 🕊️
Voice: Gentle, warm, dignified
```

### `data/persona/USER.md`
```
Name: [Your name]
Preferences: [Voice speed, topics of interest, study habits]
Family: [Optional, for personalized counsel]
```

### `data/persona/MEMORY.md`
```
# Long-Term Memory
## Key Insights
- [Date] Insight from study session...

## Recurring Themes
- Faith and diligence
- Prophetic timelines
```

### `data/persona/HEARTBEAT.md`
```
# Heartbeat Rules
Every 24 hours:
1. Read logs/daily/YYYY-MM-DD.md
2. Extract 3 key insights
3. Append to MEMORY.md
4. Never modify Bible DB
5. Never delete, only append
```

---

## Phase 5: Voice I/O

### Pocket TTS (Text-to-Speech)

```bash
# Install pocketsphinx + espeak (lightweight, offline)
sudo apt-get install espeak-ng ffmpeg
```

Or use a Rust TTS crate:
```toml
# In Cargo.toml
tts = "0.26"  # cross-platform TTS
```

In `src/tts.rs`:
```rust
pub fn speak(text: &str) {
    let tts = tts::Tts::default().unwrap();
    tts.speak(text, false).unwrap();
}
```

### Whisper.cpp (Speech-to-Text)

```bash
cd ~/paraclea
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
bash models/download-ggml-model.sh tiny.en
make
# Produces: ./main -m models/ggml-tiny.en.bin -f audio.wav
```

**What it does:** Converts your voice (from ESP32 mic or laptop mic) into text. The `tiny.en` model is 39 MB and runs at real-time speed on CPU.

In `src/stt.rs`, shell out to it:
```rust
pub async fn transcribe(audio_path: &str) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("./whisper.cpp/main")
        .args(["-m", "./whisper.cpp/models/ggml-tiny.en.bin", 
               "-f", audio_path, "-np", "-nt"])
        .output()
        .await?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

---

## Phase 6: Running the Stack

### Terminal 1: Qdrant
```bash
cd ~/paraclea
./qdrant
```

### Terminal 2: Ollama
```bash
ollama serve
```

### Terminal 3: Ingest Bible (One Time)
```bash
cd ~/paraclea/paraclea-hub
cargo run --release -- ingest-bible \
  --input ../bible_databases/json/kjv.json \
  --collection bible
```

### Terminal 4: Paraclea Hub
```bash
cd ~/paraclea/paraclea-hub
cargo run --release
```

### Test It
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question": "What does Proverbs 31 say about diligence?", "context": "bible"}'
```

---

## Phase 7: Adding Books with book-to-skill

When you want to add a medical or survival book:

```bash
# 1. Convert book to skill (one-time Python)
cd ~/paraclea/book-to-skill
python scripts/extract.py ~/Downloads/where-there-is-no-doctor.pdf \
  --mode text --install-missing ask

# 2. This creates a skill folder with chapters/*.md
# 3. Your Rust ingest tool reads those .md files and chunks them
cargo run --release -- ingest-book \
  --input ~/paraclea/book-to-skill/output/where-there-is-no-doctor/chapters/ \
  --collection survival
```

**What book-to-skill gives you:** Structured Markdown with frameworks, techniques, and anti-patterns. Your Rust tool treats each chapter section as a chunk, embeds it, and stores it in the `survival` Qdrant collection.

---

## What You Are Skipping (And Why)

| Nomad Component | What It Is | Why Skip It For Now |
|-----------------|------------|---------------------|
| **Kiwix / ZIM files** | Offline Wikipedia viewer | 100GB of data you do not need yet. Replace with specific books via book-to-skill. |
| **Kolibri** | Khan Academy offline server | Education platform for K-12. Not needed for Bible study. |
| **CyberChef** | Web app for encryption/encoding/decoding | Cool tool, but purely utility. Add later if you need to decode signals or encrypt notes. |
| **ProtoMaps / PMTiles** | Offline OpenStreetMap | Only useful if you need geographic navigation. Skip until you need maps. |
| **FlatNotes** | Markdown note-taking web app | Replaced by Paraclea's `MEMORY.md` + daily logs system. |
| **Docker** | Container runtime | Heavy on SBCs. Paraclea uses native binaries only. |
| **MySQL** | Relational database | Overkill. Qdrant (vectors) + SQLite (notes) is enough. |

---

## The Complete Git Repo Checklist

```bash
# Data
git clone https://github.com/scrollmapper/bible_databases.git

# Preprocessor (Python, one-time use)
git clone https://github.com/virgiliojr94/book-to-skill.git

# STT engine (C++, compile once)
git clone https://github.com/ggerganov/whisper.cpp.git

# Your project (Rust, you build this)
cargo new paraclea-hub
```

---

## Summary

| Step | Action | Result |
|------|--------|--------|
| 1 | Install Rust, Ollama, build tools | Dev environment ready |
| 2 | Clone Bible DB, download Qdrant binary | Data + vector DB ready |
| 3 | Pull 3 Ollama models | AI engine ready |
| 4 | Scaffold Rust project with Axum + reqwest | Hub skeleton |
| 5 | Implement Ollama client, Qdrant client, ingestion | Can embed and retrieve |
| 6 | Implement query router with model switching | Paraclea can answer |
| 7 | Create persona files | Paraclea has a soul |
| 8 | Add Whisper.cpp + Pocket TTS | Voice I/O works |
| 9 | Ingest Bible | Full RAG on Scripture |
| 10 | Add books via book-to-skill | Expandable knowledge |

**Start with steps 1–6.** Get the `/api/query` endpoint working with the Bible. Everything else is an upgrade.

Want me to write the complete `main.rs` and `router.rs` as a single compilable file you can copy-paste and run?
