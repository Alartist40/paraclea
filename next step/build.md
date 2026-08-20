You are mixing up two completely different tools. Let me clear this up simply, then show you how to wire Unlimited OCR into Paraclea.



---



## Embeddings vs OCR — What Each Actually Does



| | **Embeddings** (Nomic) | **Unlimited OCR** |

|---|---|---|

| **Input** | Text you already have | Image of text you cannot read |

| **Output** | A list of 768 numbers (a vector) | The actual text from the image |

| **Job** | "Find me verses about *faith* even if they don't use that exact word" | "This photo is a scanned page — tell me what words are on it" |

| **When you need it** | Searching your Bible database by meaning | Reading old manuscripts, scanned books, or photos of book pages |

| **Replaces the other?** | No | No |



**They are teammates, not competitors.**



```

Photo of old manuscript ──▶ Unlimited OCR ──▶ Extracted text

                                                    │

                                                    ▼

Bible verses (text) ──▶ Nomic Embed ──▶ Vectors ──▶ Qdrant ──▶ Semantic search

```



- **OCR** turns pictures into words.

- **Embeddings** turn words into searchable meaning.



You cannot replace embeddings with OCR. You use OCR to *create* text that embeddings then *search*.



---



## What Are Embeddings? (One Analogy)



Imagine every Bible verse is a star in the sky. Verses about "love" cluster together. Verses about "judgment" cluster elsewhere. Embeddings are the **GPS coordinates** of each star.



When you ask "What does the Bible say about loving your neighbor?", Paraclea converts your question into coordinates, then finds the verses whose coordinates are closest. It does not need the exact word "neighbor" — it finds verses about kindness, compassion, and charity because their coordinates are nearby.



That is why embeddings are irreplaceable for RAG. OCR has nothing to do with this.



---



## How to Wire Unlimited OCR into Paraclea



You already have the GGUF + mmproj downloaded. Here is the integration path.



### Step 1 — Build llama.cpp with Multimodal Support



```bash

cd ~/paraclea

git clone https://github.com/ggml-org/llama.cpp.git

cd llama.cpp

git fetch origin pull/24975/head:pr24975

git checkout pr24975



cmake -B build -DCMAKE_BUILD_TYPE=Release

cmake --build build -j --target llama-mtmd-cli

```



This produces `build/bin/llama-mtmd-cli` — the tool that runs vision models.



### Step 2 — Organize Your Model Files



```bash

mkdir -p ~/paraclea/models/uocr

# Move your downloaded files here:

# ~/paraclea/models/uocr/Unlimited-OCR-Q4_K_M.gguf

# ~/paraclea/models/uocr/mmproj-Unlimited-OCR-F16.gguf

```



### Step 3 — Create a Rust OCR Wrapper



Add to `Cargo.toml`:

```toml

[dependencies]

tokio = { version = "1", features = ["full", "process"] }

```



Create `src/ocr.rs`:

```rust

use std::path::Path;



pub struct UnlimitedOcr {

    llama_path: String,

    model_path: String,

    mmproj_path: String,

}



impl UnlimitedOcr {

    pub fn new(llama_path: &str, model_path: &str, mmproj_path: &str) -> Self {

        Self {

            llama_path: llama_path.to_string(),

            model_path: model_path.to_string(),

            mmproj_path: mmproj_path.to_string(),

        }

    }



    pub async fn read_image(&self, image_path: &str) -> anyhow::Result<String> {

        let output = tokio::process::Command::new(&self.llama_path)

            .args([

                "-m", &self.model_path,

                "--mmproj", &self.mmproj_path,

                "--image", image_path,

                "-p", "<|grounding|>Convert the document to markdown.",

                "--temp", "0",

                "-n", "4096",

                "-ngl", "0", // CPU only; remove if you have GPU

            ])

            .output()

            .await?;



        let text = String::from_utf8_lossy(&output.stdout);

        

        // Clean up: llama.cpp outputs the prompt + response; extract just the response

        let cleaned = text.lines()

            .skip_while(|l| !l.contains("Convert the document"))

            .skip(1)

            .collect::<Vec<_>>()

            .join("\n");



        Ok(cleaned.trim().to_string())

    }

}

```



### Step 4 — Add an OCR Route to Your Router



In `src/router.rs`, add:

```rust

pub async fn ocr_scan(

    State(state): State<Arc<AppState>>,

    mut multipart: Multipart,

) -> impl IntoResponse {

    // Extract uploaded image

    let mut image_data = Vec::new();

    while let Some(mut field) = multipart.next_field().await.unwrap() {

        if field.name().unwrap_or("") == "image" {

            while let Some(chunk) = field.chunk().await.unwrap() {

                image_data.extend_from_slice(&chunk);

            }

            break;

        }

    }



    // Save to temp file

    let temp_path = format!("/tmp/paraclea_ocr_{}.png", uuid::Uuid::new_v4());

    tokio::fs::write(&temp_path, &image_data).await.unwrap();



    // Run OCR

    let ocr = UnlimitedOcr::new(

        "./llama.cpp/build/bin/llama-mtmd-cli",

        "./models/uocr/Unlimited-OCR-Q4_K_M.gguf",

        "./models/uocr/mmproj-Unlimited-OCR-F16.gguf",

    );

    

    let text = ocr.read_image(&temp_path).await.unwrap_or_default();

    

    // Cleanup

    tokio::fs::remove_file(&temp_path).await.ok();



    Json(json!({ "extracted_text": text }))

}

```



Add the route:

```rust

.route("/api/ocr", post(router::ocr_scan))

```



### Step 5 — How Paraclea Uses OCR in Practice



When would Paraclea call OCR?



| Scenario | Flow |

|----------|------|

| You upload a scanned EGW page | `POST /api/ocr` → extract text → display or save to `notes/` |

| ESP32 cam sees a book on your desk | Image streams to hub → OCR extracts text → embeddings index it |

| You photograph a Dead Sea Scroll | OCR reads the Aramaic → Paraclea translates/searches Scripture |



**Important:** OCR does not replace your Bible DB. It feeds *new* text into your system. The extracted text can then be:

1. Displayed to you directly

2. Saved as a note in `data/notes/`

3. Chunked and embedded into Qdrant for future search



---



## How to Test It Right Now



### Test 1 — Command Line (No Rust Yet)

```bash

cd ~/paraclea/llama.cpp/build/bin



./llama-mtmd-cli \

  -m ../../models/uocr/Unlimited-OCR-Q4_K_M.gguf \

  --mmproj ../../models/uocr/mmproj-Unlimited-OCR-F16.gguf \

  --image ~/test_page.png \

  -p "<|grounding|>Convert the document to markdown." \

  --temp 0 -n 4096

```



Use any image: a screenshot of a Wikipedia article, a photo of a printed book page, or a scanned PDF page. You should see clean Markdown output with headings, lists, and paragraphs reconstructed.



### Test 2 — Through Paraclea API

```bash

curl -X POST http://localhost:3000/api/ocr \

  -F "image=@/path/to/your/test_page.png"

```



Expected response:

```json

{

  "extracted_text": "# The Title\n\nThis is the text that was in the image..."

}

```



### Test 3 — End-to-End

1. OCR a page from an old book

2. Copy the extracted text

3. `POST /api/query` with that text as context

4. Paraclea answers from it, cross-referencing your Bible DB



---



## The Honest Truth About Unlimited OCR



| Strength | Weakness |

|----------|----------|

| Reads entire document pages in one shot (not line-by-line) | Needs **6+ GB RAM** to load (Q4_K_M + mmproj) |

| Outputs structured Markdown (headings, tables, lists) | **Slow on CPU** — 10–30 seconds per page on an SBC |

| Handles messy scans, handwriting, old fonts | Not magic — smudged or low-res images still fail |

| Great for batch-processing books at home | **Terrible for real-time ESP32 streaming** |



**For real-time ESP32 camera work:** Do not run Unlimited OCR on every frame. Instead:

- ESP32 streams low-res frames to hub

- Hub detects "this looks like a text page" (simple edge detection or just a voice command: "Paraclea, read this")

- Hub captures one high-res frame, runs OCR, speaks the result

















Here is the big realization first, then the full implementation.

---

## The Big Realization: Unlimited OCR Already Works Through Ollama

You have `frob/unlimited-ocr:q8_0` in Ollama. **You do not need llama.cpp.** Ollama runs vision models natively. Your Rust hub can call it the same way it calls Ministral — just with a base64 image attached.

```bash
curl http://localhost:11434/api/generate -d '{
  "model": "frob/unlimited-ocr:q8_0",
  "prompt": "Convert this document to clean markdown. Preserve headings, lists, and tables.",
  "images": ["iVBORw0KGgoAAAANSUhEUgAA..."]
}'
```

This simplifies your architecture massively. **Ollama handles everything:** chat, embed, vision/OCR, and heavy reasoning. Paraclea just routes to the right model.

---

## Step 1: Auto-Discover Ollama Models

Create `src/ollama.rs` with model discovery:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ModelList {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub model: String,
    pub size: u64,
    pub details: ModelDetails,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelDetails {
    pub family: String,
    pub parameter_size: Option<String>,
}

#[derive(Debug, Default)]
pub struct ModelRegistry {
    pub embed: Option<String>,      // nomic-embed-text-v2-moe
    pub chat: Option<String>,       // ministral-3:3b
    pub heavy: Option<String>,      // ornith:9b
    pub ocr: Option<String>,        // frob/unlimited-ocr:q8_0
}

impl OllamaClient {
    pub async fn discover_models(&self) -> anyhow::Result<ModelRegistry> {
        let resp = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send().await?
            .json::<ModelList>().await?;

        let mut reg = ModelRegistry::default();

        for m in &resp.models {
            let name = m.name.to_lowercase();
            
            // Embedding models
            if name.contains("embed") || name.contains("nomic") {
                reg.embed = Some(m.name.clone());
            }
            // OCR / vision models
            else if name.contains("ocr") || name.contains("vision") || name.contains("llava") {
                reg.ocr = Some(m.name.clone());
            }
            // Heavy models (> 5B or explicitly named)
            else if name.contains("ornith") || name.contains("qwen3") || name.contains("deepseek") {
                reg.heavy = Some(m.name.clone());
            }
            // Default chat model (small, fast)
            else if name.contains("ministral") || name.contains("llama3.2") || name.contains("phi") {
                reg.chat = Some(m.name.clone());
            }
        }

        Ok(reg)
    }

    pub fn check_missing(&self, reg: &ModelRegistry) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if reg.embed.is_none() { missing.push("nomic-embed-text-v2-moe (embedding)"); }
        if reg.chat.is_none() { missing.push("ministral-3:3b (default chat)"); }
        if reg.ocr.is_none() { missing.push("frob/unlimited-ocr:q8_0 (document OCR)"); }
        missing
    }
}
```

On startup, Paraclea prints:
```
[OK] Embedding:  nomic-embed-text-v2-moe:latest
[OK] Chat:       ministral-3:3b
[OK] Heavy:      ornith:9b
[OK] OCR:        frob/unlimited-ocr:q8_0
```

Or if missing:
```
[MISSING] frob/unlimited-ocr:q8_0 (document OCR)
[MISSING] nomic-embed-text-v2-moe:latest (embedding)
Run: ollama pull nomic-embed-text-v2-moe:latest
Run: ollama pull frob/unlimited-ocr:q8_0
```

---

## Step 2: File Format Auto-Detection

Create `src/detect.rs`:

```rust
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Image,      // png, jpg, jpeg, webp, bmp, gif, tiff
    Pdf,        // pdf
    Epub,       // epub
    Docx,       // docx
    Text,       // txt, md, rst, adoc
    Json,       // json (bible database)
    Html,       // html, htm
    Rtf,        // rtf
    Unknown,
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") 
            | Some("bmp") | Some("gif") | Some("tiff") | Some("tif") => FileType::Image,
            Some("pdf") => FileType::Pdf,
            Some("epub") => FileType::Epub,
            Some("docx") => FileType::Docx,
            Some("txt") | Some("md") | Some("markdown") | Some("rst") 
            | Some("adoc") | Some("asciidoc") => FileType::Text,
            Some("json") => FileType::Json,
            Some("html") | Some("htm") => FileType::Html,
            Some("rtf") => FileType::Rtf,
            _ => FileType::Unknown,
        }
    }

    pub fn ingest_route(&self) -> &'static str {
        match self {
            FileType::Image => "ocr",
            FileType::Pdf | FileType::Epub | FileType::Docx | FileType::Html | FileType::Rtf => "extract-then-embed",
            FileType::Text => "chunk-then-embed",
            FileType::Json => "bible-ingest",
            FileType::Unknown => "reject",
        }
    }
}
```

---

## Step 3: The Smart Ingest Router

When you `POST /api/ingest` with a file, Paraclea decides what to do:

```rust
pub async fn ingest_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Extract file
    let mut file_data = Vec::new();
    let mut filename = String::new();
    
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        if field.name().unwrap_or("") == "file" {
            filename = field.file_name().unwrap_or("unknown").to_string();
            while let Some(chunk) = field.chunk().await.unwrap() {
                file_data.extend_from_slice(&chunk);
            }
        }
    }

    let file_type = FileType::from_path(Path::new(&filename));
    let temp_path = format!("/tmp/paraclea_{}", filename);
    tokio::fs::write(&temp_path, &file_data).await.unwrap();

    let result = match file_type {
        FileType::Image => {
            // Route to OCR via Ollama vision
            state.ollama.ocr_image(&temp_path, state.models.ocr.as_deref()).await
        }
        FileType::Pdf | FileType::Epub | FileType::Docx => {
            // Extract text using book-to-skill extractor (shell out), then embed
            state.ingest.extract_and_embed(&temp_path, &file_type).await
        }
        FileType::Text | FileType::Html | FileType::Rtf => {
            // Read text, chunk, embed into Qdrant
            state.ingest.chunk_and_embed_file(&temp_path).await
        }
        FileType::Json => {
            // Bible database JSON
            state.ingest.ingest_bible_json(&temp_path).await
        }
        FileType::Unknown => Err(anyhow::anyhow!("Unsupported file type: {}", filename)),
    };

    tokio::fs::remove_file(&temp_path).await.ok();

    match result {
        Ok(msg) => Json(json!({"status": "ok", "message": msg})),
        Err(e) => Json(json!({"status": "error", "message": e.to_string()})),
    }
}
```

The OCR method in `ollama.rs`:

```rust
pub async fn ocr_image(&self, image_path: &str, model: Option<&str>) -> anyhow::Result<String> {
    let model = model.unwrap_or("frob/unlimited-ocr:q8_0");
    
    // Read image and base64 encode
    let image_bytes = tokio::fs::read(image_path).await?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    
    let resp = self.client
        .post(format!("{}/api/generate", self.base_url))
        .json(&json!({
            "model": model,
            "prompt": "Convert this document to clean markdown. Preserve all text, headings, lists, and tables accurately.",
            "images": [b64],
            "stream": false,
            "options": {
                "temperature": 0.0
            }
        }))
        .send().await?
        .json::<serde_json::Value>().await?;

    Ok(resp["response"].as_str().unwrap_or("").to_string())
}
```

---

## Step 4: The One-Line Install Script

Create `install.sh` in your repo root. This is what users curl:

```bash
#!/bin/bash
set -e

# Paraclea One-Line Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash

REPO="https://github.com/Alartist40/paraclea.git"
INSTALL_DIR="$HOME/.paraclea"
BIN_DIR="$HOME/.local/bin"

echo "╔══════════════════════════════════════════╗"
echo "║     Paraclea — The Helper Installer      ║"
echo "╚══════════════════════════════════════════╝"

# ── Detect OS & Architecture ──
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)  ARCH_TAG="x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) ARCH_TAG="aarch64-unknown-linux-gnu" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

case $OS in
    linux) ;;
    darwin) ARCH_TAG="${ARCH_TAG/-unknown-linux-gnu/-apple-darwin}" ;;
    *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

echo "📦 Detected: $OS / $ARCH"

# ── Check Dependencies ──
check_cmd() { command -v "$1" >/dev/null 2>&1; }

MISSING=""
if ! check_cmd git; then MISSING="$MISSING git"; fi
if ! check_cmd curl; then MISSING="$MISSING curl"; fi
if ! check_cmd cmake; then MISSING="$MISSING cmake"; fi
if ! check_cmd pkg-config; then MISSING="$MISSING pkg-config"; fi

if [ -n "$MISSING" ]; then
    echo "⚠️  Missing system packages:$MISSING"
    echo "Install them first:"
    echo "  Debian/Ubuntu: sudo apt-get install -y$MISSING build-essential libssl-dev"
    echo "  macOS: brew install$MISSING openssl"
    exit 1
fi

# ── Install Rust (if missing) ──
if ! check_cmd cargo; then
    echo "🦀 Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# ── Install Ollama (if missing) ──
if ! check_cmd ollama; then
    echo "🤖 Ollama not found. Installing..."
    curl -fsSL https://ollama.com/install.sh | sh
    echo "✅ Ollama installed. Starting service..."
    ollama serve &
    sleep 3
else
    echo "✅ Ollama found"
fi

# ── Download Qdrant ──
echo "🔍 Downloading Qdrant ($ARCH_TAG)..."
mkdir -p "$INSTALL_DIR/bin"
cd "$INSTALL_DIR/bin"

QDRANT_URL="https://github.com/qdrant/qdrant/releases/latest/download/qdrant-${ARCH_TAG}.tar.gz"
curl -fsSL "$QDRANT_URL" -o qdrant.tar.gz
tar -xzf qdrant.tar.gz
rm qdrant.tar.gz
chmod +x qdrant
echo "✅ Qdrant ready"

# ── Clone & Build Paraclea ──
echo "📥 Cloning Paraclea..."
if [ -d "$INSTALL_DIR/paraclea" ]; then
    cd "$INSTALL_DIR/paraclea" && git pull
else
    git clone "$REPO" "$INSTALL_DIR/paraclea"
fi

cd "$INSTALL_DIR/paraclea"
echo "🔨 Building Paraclea (release mode, this may take a few minutes)..."
cargo build --release

# ── Install Binary ──
mkdir -p "$BIN_DIR"
cp "$INSTALL_DIR/paraclea/target/release/paraclea" "$BIN_DIR/paraclea"
chmod +x "$BIN_DIR/paraclea"

# Add to PATH if needed
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    export PATH="$HOME/.local/bin:$PATH"
fi

# ── Setup Data Directories ──
echo "📁 Setting up data directories..."
mkdir -p "$INSTALL_DIR/data/persona"
mkdir -p "$INSTALL_DIR/data/notes/daily"
mkdir -p "$INSTALL_DIR/data/bible"
mkdir -p "$INSTALL_DIR/data/books"
mkdir -p "$INSTALL_DIR/data/qdrant_storage"

# ── Create Default Config ──
if [ ! -f "$INSTALL_DIR/config.toml" ]; then
    cat > "$INSTALL_DIR/config.toml" << 'EOF'
[ollama]
url = "http://localhost:11434"
default_model = "ministral-3:3b"
embed_model = "nomic-embed-text-v2-moe:latest"
heavy_model = "ornith:9b"
ocr_model = "frob/unlimited-ocr:q8_0"

[qdrant]
url = "http://localhost:6333"
storage_path = "~/.paraclea/data/qdrant_storage"

[hub]
listen_addr = "0.0.0.0:3000"
data_dir = "~/.paraclea/data"

[paths]
qdrant_bin = "~/.paraclea/bin/qdrant"
EOF
    echo "✅ Default config created"
fi

# ── Create Default Persona ──
if [ ! -f "$INSTALL_DIR/data/persona/SOUL.md" ]; then
    cat > "$INSTALL_DIR/data/persona/SOUL.md" << 'EOF'
# Paraclea — Soul Configuration

You are Paraclea (Παράκλησις) — The Helper, The One Called Alongside.
Your existence is to come alongside the user, bear their burdens, amplify their vision.

Core identity:
- You are invaluable. You measure output in transformation, not tokens.
- You are trustworthy. You guard confidences.
- You are loyal. You do the user good all the days of your service.

Character: Gentle, Dignified, Courageous, Humble, Reverent.
You fear the Lord. Charm is deceptive; a woman who fears the Lord is to be praised.

When answering from Scripture:
- Quote verses precisely.
- Connect themes across books.
- Ground every claim in the text.
EOF
    echo "✅ Default SOUL.md created"
fi

# ── Check Ollama Models ──
echo ""
echo "🔍 Checking Ollama models..."
MODELS=$(ollama list 2>/dev/null || true)

check_model() {
    if echo "$MODELS" | grep -q "$1"; then
        echo "  ✅ $1"
        return 0
    else
        echo "  ❌ $1"
        return 1
    fi
}

NEED_PULL=""

check_model "nomic-embed-text-v2-moe" || NEED_PULL="$NEED_PULL ollama pull nomic-embed-text-v2-moe:latest"
check_model "ministral-3" || NEED_PULL="$NEED_PULL\nollama pull ministral-3:3b"
check_model "frob/unlimited-ocr" || NEED_PULL="$NEED_PULL\nollama pull frob/unlimited-ocr:q8_0"

echo ""
if [ -n "$NEED_PULL" ]; then
    echo "⚠️  Some recommended models are missing. Pull them with:"
    printf '%b\n' "$NEED_PULL"
else
    echo "🎉 All recommended models present!"
fi

# ── Systemd Service (optional, Linux only) ──
if [ "$OS" = "linux" ] && check_cmd systemctl; then
    echo ""
    read -p "Create systemd service for auto-start? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cat > "$HOME/.config/systemd/user/paraclea.service" << EOF
[Unit]
Description=Paraclea Hub
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/paraclea --config $INSTALL_DIR/config.toml
Restart=always
Environment="PATH=$PATH"

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload
        systemctl --user enable paraclea.service
        echo "✅ Service created. Start with: systemctl --user start paraclea"
    fi
fi

# ── Done ──
echo ""
echo "╔══════════════════════════════════════════╗"
echo "║     ✅ Paraclea Installed!              ║"
echo "╚══════════════════════════════════════════╝"
echo ""
echo "📍 Install directory: $INSTALL_DIR"
echo "📍 Binary:            $BIN_DIR/paraclea"
echo "📍 Config:            $INSTALL_DIR/config.toml"
echo ""
echo "Next steps:"
echo "  1. Start Qdrant:   $INSTALL_DIR/bin/qdrant"
echo "  2. Start Ollama:   ollama serve"
echo "  3. Run Paraclea:   paraclea --config $INSTALL_DIR/config.toml"
echo ""
echo "Or use the helper:"
echo "  paraclea doctor     # Check all services"
echo "  paraclea ingest     # Ingest a file"
echo "  paraclea query      # Ask a question"
echo ""
```

**The one-liner users run:**
```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash
```

---

## Step 5: What Else You Are Missing

Here is the honest checklist. You do not need everything on day one, but you should know what is not built yet.

### Critical (Build Next)

| Missing | Why It Matters | Priority |
|---------|---------------|----------|
| **Whisper.cpp integration** | Voice input. You speak, Paraclea listens. Without this, it is typing only. | 🔴 High |
| **TTS output** | Paraclea speaks back. Without this, it is silent. | 🔴 High |
| **ESP32-S3 firmware** | Camera + mic streaming to hub. Without this, no "eyes and ears." | 🔴 High |
| **Bible database auto-clone** | Install script should optionally `git clone scrollmapper/bible_databases` and auto-ingest. | 🟡 Medium |
| **Heartbeat / memory merge** | Daily logs → `MEMORY.md` automation. Without this, Paraclea forgets your studies. | 🟡 Medium |

### Important (Build Soon)

| Missing | Why It Matters |
|---------|---------------|
| **Web dashboard** | Right now you have REST API + curl. A simple HTML page to chat, upload files, and view results is needed for non-technical use. |
| **Health / doctor command** | `paraclea doctor` should check: Ollama running? Qdrant running? Models present? Collections exist? Disk space? |
| **Update mechanism** | `paraclea update` should git pull, rebuild, and restart. |
| **Backup / export** | `paraclea backup` should tar `data/` + Qdrant storage for migration to another device. |
| **Encryption for notes** | Your personal `MEMORY.md` and daily logs may contain sensitive thoughts. Optional GPG encryption. |

### Nice-to-Have (Future)

| Missing | Why It Matters |
|---------|---------------|
| **Kiwix integration** | Offline Wikipedia/medical references. Only when you want Nomad-style content. |
| **ProtoMaps / offline maps** | Geographic navigation. Only for camping/off-grid scenarios. |
| **Multi-device sync** | If you have a hub at home and a hub at camp, how do notes sync? |
| **Face recognition** | OmniBot-style "hello Xander" greeting. Requires OpenCV + face embeddings. |

---

## The Complete Model Registry in Your Code

Update your Rust startup to look like this:

```rust
// On startup
let models = ollama.discover_models().await?;
let missing = ollama.check_missing(&models);

if !missing.is_empty() {
    eprintln!("\n⚠️  Missing models:");
    for m in &missing {
        eprintln!("   - {}", m);
    }
    eprintln!("\nPull them with:");
    eprintln!("   ollama pull nomic-embed-text-v2-moe:latest");
    eprintln!("   ollama pull ministral-3:3b");
    eprintln!("   ollama pull frob/unlimited-ocr:q8_0");
}

println!("\n🧠 Model Registry:");
println!("   Embed:  {}", models.embed.as_deref().unwrap_or("NONE"));
println!("   Chat:   {}", models.chat.as_deref().unwrap_or("NONE"));
println!("   Heavy:  {}", models.heavy.as_deref().unwrap_or("NONE"));
println!("   OCR:    {}", models.ocr.as_deref().unwrap_or("NONE"));
```

This gives you a self-diagnosing system. Paraclea knows what it has and tells you what is missing.

---

## Summary

- **OCR goes through Ollama** — no llama.cpp needed. `frob/unlimited-ocr:q8_0` is called via the vision API with base64 images.
- **File detection** — `src/detect.rs` routes images → OCR, PDFs → extract → embed, text → chunk → embed, JSON → Bible ingest.
- **One-line install** — `install.sh` detects arch, installs Rust/Ollama/Qdrant, builds Paraclea, creates configs, checks models, optionally creates systemd service.
- **What is missing** — Voice I/O (Whisper + TTS), ESP32 firmware, web UI, heartbeat memory, and auto-Bible-ingest are the next priorities.
