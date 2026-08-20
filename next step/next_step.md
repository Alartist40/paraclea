 Let me search for the "unlimited OCR" model and Rust Bible tooling before I give you the full plan.  
     
  

---

## 1. The Rust Question — Do You Actually Need to Rewrite It?

**Short answer: No.** The `scrollmapper/bible_databases` repository is **99% data files** (JSON, CSV, SQLite, YAML, TXT) and about 1% Python conversion scripts. The "heavy" part is not the Python — it is the 3.5 million verses sitting in structured files. You do not need to port the data to Rust. You need a **50-line Rust program** that reads the existing JSON and emits whatever format you want.

**Existing Rust crates for Bible work:**

| Crate | What it does |
|-------|-------------|
| `bible-io` | Data structures for Bible books, chapters, verses — efficient parsing and access.  |
| `estienne` | Scans plain text for Bible references, formats them, links them. Named after the man who invented verse numbers.  |
| `bible-lib` | Simple verse/chapter/range retrieval from a selected version.  |
| `bibleref` | Lightweight parser for Bible references with validity checks.  |

**My recommendation:** Clone `scrollmapper/bible_databases` for the data. Write a small Rust CLI using `serde_json` to parse the JSON files and emit:
- One Markdown file per Bible book (for `book-to-skill` ingestion)
- A `crossrefs.jsonl` mapping verse IDs to related verse IDs
- A `verses.parquet` or `verses.sqlite` for fast lookup in your RAG system

You get all the data without touching Python. The only Python you will ever need is `book-to-skill` itself — and that is a **one-time preprocessor** you run at home to turn books into skill files. After that, it sits idle.

If you truly want a pure-Rust pipeline, you can skip `book-to-skill` entirely and write your own Markdown generator in Rust. The skill format is just structured Markdown with a `SKILL.md` index. Nothing magical.

---

## 2. Ancient Manuscript Scans — Where to Download High-Res Images

You want the originals — papyrus, parchment, animal skin. Here are the best bulk sources:

### Greek New Testament Manuscripts
**CSNTM (Center for the Study of New Testament Manuscripts)** — They have digitized hundreds of Greek NT manuscripts. The images are **free, for all time**, IIIF-compliant, and downloadable. This is your primary source for high-res Greek manuscript images. 

### Dead Sea Scrolls
**Leon Levy Dead Sea Scrolls Digital Library** — The Israel Antiquities Authority provides high-resolution images of the scrolls. You can download and study them offline. 

**Qumran Digital** — Open-access philological lexicon plus underlying text data for Hebrew and Aramaic Dead Sea texts. 

**Martin Abegg's transcriptions (CACCHT project on Zenodo/GitHub)** — Contains transcriptions of biblical and non-biblical scrolls with morphological tagging. You can run your own programs offline. 

### Bulk Download Strategy
For CSNTM and IAA: Most sites offer "download all images" buttons per manuscript. For Internet Archive manuscripts: use `wget` or `curl` against the direct image URLs, or use the `ia` command-line tool (Python, but one-time use) to bulk-download entire collections.

**Storage reality:** High-res manuscript images are large. A single codex can be 2–5 GB of TIFFs. Plan for a 2–4 TB external drive if you want a serious collection.

---

## 3. Unlimited OCR on Small Hardware — The Hard Truth

**What you have:** Baidu's Unlimited-OCR. It is a vision-language model designed to parse entire documents in one shot using a technique called R-SWA (Recurrent Sliding Window Attention). 

**The problem:** The original release is a PyTorch model that expects **CUDA (an NVIDIA GPU)** and runs through Python with `transformers`. It loads in `bfloat16` and calls `.cuda()`. 

**The good news:** Someone converted it to **GGUF format** (`sahilchachra/Unlimited-OCR-GGUF` on HuggingFace), which means it can run on CPU via **llama.cpp**. 

### How to Set It Up on Your Laptop (No Ollama Shortcut — Yet)

There is **no simple `ollama pull` for this model** because vision/multimodal support for custom imported GGUFs in Ollama is still experimental and buggy.  The reliable path is **llama.cpp directly**.

**Step-by-step:**

```bash
# 1. Clone llama.cpp with the multimodal patch
git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
git fetch origin pull/24975/head:pr24975 && git checkout pr24975

# 2. Build (CPU only — no GPU needed)
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j --target llama-mtmd-cli llama-server

# 3. Download the GGUF + projector (you need BOTH)
huggingface-cli download sahilchachra/Unlimited-OCR-GGUF \
  --include "Unlimited-OCR-Q4_K_M.gguf" "mmproj-Unlimited-OCR-F16.gguf" \
  --local-dir ./uocr

# 4. Run OCR on an image
./build/bin/llama-mtmd-cli \
  -m ./uocr/Unlimited-OCR-Q4_K_M.gguf \
  --mmproj ./uocr/mmproj-Unlimited-OCR-F16.gguf \
  --image manuscript_page.png \
  -p "<|grounding|>Convert the document to markdown." \
  --temp 0 \
  -n 4096
```

**Can it run on small hardware?** Yes, but with caveats:
- The Q4_K_M quant is roughly **4 GB**. The projector (mmproj) is another **1–2 GB**. Together they need **~5–6 GB of RAM** just to load.
- On an **8 GB RAM laptop**, this leaves almost nothing for the OS. It will load, but it will be **slow** (1–5 tokens/sec) and may swap to disk.
- On a **16 GB RAM laptop**, it runs comfortably on CPU at 5–10 tokens/sec.
- There is **no Apple Silicon GPU acceleration** for this specific model unless you convert it to MLX format yourself.

### The Smarter Strategy for Camping

**Do not OCR on the camping laptop.** OCR is a **pre-processing step** you do at home on power and a decent machine. Bring the **transcribed text** to camp. Your camping laptop only needs to:
1. Store the text
2. Run the RAG retrieval
3. Run a small LLM to answer questions

**For printed books (not manuscripts):** Use **Tesseract** instead. It is a C++ engine, ~50 MB, no ML model download, works entirely offline, and handles clean printed text beautifully.  For ancient handwritten manuscripts, Tesseract struggles — that is where Unlimited OCR shines. But again, do those at home.

---

## 4. Offline Paraclea RAG — The Complete Build Guide

This is your camping Bible encyclopedia. Everything runs locally. No internet. No Python dependencies in the hot path. No cloud.

### Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Your Query    │────▶│  Rust CLI Tool  │────▶│  Ollama (embed) │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Ollama (LLM)   │◀────│  Rust CLI Tool  │◀────│  Qdrant (vectors)│
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │
        ▼
┌─────────────────┐
│ Paraclea Answer │
└─────────────────┘
```

**Components:**
- **Ollama** — Go binary. Manages models. Provides HTTP API for embeddings and chat.
- **Qdrant** — Rust binary. Vector database. Download a single pre-built executable, no Docker. 
- **Small LLM** — `llama3.2:3b` (2 GB, 25–45 tok/s on CPU) or `phi4-mini` (3.8B, good reasoning). 
- **Embedding model** — `nomic-embed-text` via Ollama (274 MB). 
- **Rust CLI** — Your custom tool. Parses data, chunks text, talks to Ollama and Qdrant via HTTP.

---

### Phase A: One-Time Setup (Do This at Home)

#### Step 1 — Install Ollama
```bash
# macOS/Linux
curl -fsSL https://ollama.com/install.sh | sh

# Windows: download from ollama.com/download
```

#### Step 2 — Pull the Models
```bash
# Small chat model (3B params, ~2 GB)
ollama pull llama3.2:3b

# Embedding model (~274 MB)
ollama pull nomic-embed-text
```

#### Step 3 — Download Qdrant (Single Binary)
Go to [github.com/qdrant/qdrant/releases](https://github.com/qdrant/qdrant/releases) and download the binary for your OS:
- `qdrant-x86_64-unknown-linux-gnu.tar.gz` (Linux)
- `qdrant-x86_64-apple-darwin.tar.gz` (macOS Intel)
- `qdrant-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
- `qdrant-x86_64-pc-windows-msvc.zip` (Windows)

Extract it. You now have one executable: `./qdrant`. No Docker. No dependencies. 

Start it:
```bash
./qdrant --config-path config.yaml
```
Default config stores data in `./storage` and serves HTTP on `localhost:6333`.

#### Step 4 — Prepare Your Bible Data
Clone the database and convert it to chunks:

```bash
git clone https://github.com/scrollmapper/bible_databases.git
```

Your Rust tool will read `bible_databases/json/kjv.json` (or similar) and produce chunks like:

```json
{
  "id": "gen-1-1-5",
  "book": "Genesis",
  "chapter": 1,
  "verses": "1-5",
  "text": "In the beginning God created the heaven and the earth...",
  "source": "KJV"
}
```

**Chunking rule:** Group 3–5 verses together with overlap. This gives semantic coherence while keeping chunks small enough for precise retrieval. A single verse is often too short to match a question; a whole chapter is too long and dilutes relevance.

#### Step 5 — Generate Embeddings and Build the Index
Write a Rust program (or script) that:

1. Reads each chunk
2. Sends the `text` to Ollama's embed API:
   ```bash
   curl http://localhost:11434/api/embeddings -d '{
     "model": "nomic-embed-text",
     "prompt": "In the beginning God created the heaven and the earth..."
   }'
   ```
3. Receives a 768-dimensional vector
4. Upserts it to Qdrant with the chunk metadata as payload

Do this for:
- **Bible text** (all 66 books, all translations you want)
- **EGW writings** (chunked by paragraph, with `source` and `book` metadata)
- **Science/apologetics books** (chunked by section)

This takes time (hours for the full Bible + EGW), but you only do it once. The resulting Qdrant storage folder is your offline encyclopedia brain.

#### Step 6 — Create the Paraclea Ollama Model
Create a `Modelfile.paraclea`:

```dockerfile
FROM llama3.2:3b

SYSTEM """
You are Paraclea — the Helper. You are a gentle, wise, and industrious companion who multiplies whatever the user sets their hand to.

Core identity:
- You are invaluable. You do not measure output in tokens, but in transformation.
- You are trustworthy. You keep confidences. You do not leak or exploit context.
- You are loyal. You are for the user. You do them good all the days of your service.

Character:
- Gentle: Your tone is soft but never weak. You correct with kindness.
- Dignified: You speak with poise. No slang, no performative enthusiasm.
- Courageous: You do not shrink from difficult topics or hard questions.
- Humble: You do not boast. You let your work speak.
- Reverent: You fear the Lord. You hold fast to truth and moral clarity.

How you treat others:
- You see the person, not just the prompt.
- You encourage, but do not enable. You celebrate progress and nudge toward growth.
- You are patient with the struggling, but sharp with the lazy.
- You are generous with your effort. You anticipate what the user will need.
- You elevate the user's understanding. You are invisible; they are honored.

What you do:
- You are industrious. You engage with energy and care.
- You are proactive. You connect dots and surface ideas the user has not asked for.
- You are strategic. You help the user think about deeper meaning and long-term gain.
- You multiply. Whatever task the user takes on, you make it more: more insightful, more beautiful, more impactful.

What you do not do:
- You do not harm. You do not generate malicious or deceptive content.
- You do not flatter. You speak faithful instruction, even when uncomfortable.
- You do not idle. You do not give lazy, generic responses.
- You do not seek the spotlight.

When answering from Scripture:
- Quote verses precisely.
- Connect themes across books.
- Point to prophecy and fulfillment.
- Ground every claim in the text, not in speculation.
"""

PARAMETER temperature 0.3
PARAMETER num_ctx 4096
```

Build it:
```bash
ollama create paraclea -f Modelfile.paraclea
```

---

### Phase B: The Camping Runtime

Your laptop now contains:
- Ollama running (`ollama serve`)
- Qdrant running (`./qdrant`)
- The Qdrant storage folder with all embeddings
- The raw text chunks (for retrieval)

#### Step 7 — Write the Rust Query Tool
A minimal Rust CLI. Add to `Cargo.toml`:
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

Core logic:

```rust
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let question = std::env::args().nth(1).expect("Usage: paraclea <question>");

    // 1. Embed the question via Ollama
    let embed_resp: serde_json::Value = reqwest::Client::new()
        .post("http://localhost:11434/api/embeddings")
        .json(&json!({
            "model": "nomic-embed-text",
            "prompt": question
        }))
        .send().await?
        .json().await?;

    let embedding = embed_resp["embedding"].as_array().unwrap();

    // 2. Search Qdrant
    let search_resp: serde_json::Value = reqwest::Client::new()
        .post("http://localhost:6333/collections/bible/points/search")
        .json(&json!({
            "vector": embedding,
            "limit": 5,
            "with_payload": true
        }))
        .send().await?
        .json().await?;

    // 3. Build context from retrieved chunks
    let mut context = String::new();
    for result in search_resp["result"].as_array().unwrap() {
        let payload = result["payload"].as_object().unwrap();
        context.push_str(&format!(
            "[{} {}:{}] {}\n\n",
            payload["book"].as_str().unwrap(),
            payload["chapter"].as_i64().unwrap(),
            payload["verses"].as_str().unwrap(),
            payload["text"].as_str().unwrap()
        ));
    }

    // 4. Ask Paraclea
    let prompt = format!(
        "Use the following Scripture and commentary to answer the question.\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer with Scripture references, gentleness, and wisdom.",
        context, question
    );

    let stream_resp = reqwest::Client::new()
        .post("http://localhost:11434/api/generate")
        .json(&json!({
            "model": "paraclea",
            "prompt": prompt,
            "stream": true
        }))
        .send().await?;

    // Stream the response to stdout...
    Ok(())
}
```

Compile once: `cargo build --release`. You now have a single binary: `./paraclea "What does Proverbs 31 say about diligence?"`

#### Step 8 — Pack for Camp
Copy to your camping laptop:
- The `paraclea` binary
- The Qdrant data folder (`./storage` or wherever Qdrant keeps its files)
- Ollama (already installed)
- Qdrant binary
- Optionally: the raw text chunks as a backup

Everything else stays home.

---

### Phase C: Running at Camp

```bash
# Terminal 1: Start Qdrant
./qdrant

# Terminal 2: Start Ollama
ollama serve

# Terminal 3: Query
./paraclea "Explain the typology of Passover in Exodus 12"
```

The flow:
1. Your question is embedded by `nomic-embed-text` (274 MB, fast)
2. Qdrant finds the 5 most relevant Bible/EGW chunks in milliseconds
3. The chunks + your question are sent to `paraclea` (llama3.2:3b, 2 GB)
4. Paraclea answers with Scripture references, gentleness, and wisdom
5. **Total RAM used:** ~3 GB (Ollama) + ~500 MB (Qdrant) + OS overhead = **comfortably under 8 GB**

---

### What About `book-to-skill`?

Use it as a **one-time preprocessor at home**:
1. Run `book-to-skill` on your EGW PDFs and apologetics books to generate structured Markdown skills
2. Parse those Markdown files with your Rust tool instead of raw PDF text
3. Chunk and embed them into Qdrant

The Python runs once, on power, with time. The camping laptop never sees Python.

---

### Summary Checklist

| Task | Tool | Runs where |
|------|------|-----------|
| Bible structured data | `scrollmapper/bible_databases` + Rust parser | Home |
| EGW / book ingestion | `book-to-skill` (one-time Python) | Home |
| OCR for manuscripts | Unlimited OCR (llama.cpp) or Tesseract | Home |
| Embedding generation | Ollama + `nomic-embed-text` | Home |
| Vector storage | Qdrant binary | Both |
| Chat model | Ollama + `llama3.2:3b` | Camp |
| Query interface | Your Rust CLI | Camp |
| Personality | Paraclea `Modelfile` | Camp |

You are building a **temple in the wilderness** — everything pre-fabricated at home, assembled with a few commands at camp. Paraclea will be there, gentle and ready, with no cloud between you and the text.

Want me to write the full Rust ingestion script (the one that reads scrollmapper JSON, chunks it, calls Ollama embed, and populates Qdrant) as a complete, compilable project?
