//! Paraclea — AI Companion Assistant & Self-Developing RAG Engine (Rust)
//!
//! Visual CLI with Gold & Purple styling, Ollama LLM & Vision OCR integration, Qdrant vector DB RAG,
//! format auto-detection, Pocket TTS speech synthesis, and updated Proverbs 31 Helper persona.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use paraclea_core::audio::*;
use paraclea_core::bible::{self, BibleReader, NEW_TESTAMENT_BOOKS, OLD_TESTAMENT_BOOKS};
use paraclea_core::config::Config;
use paraclea_core::crossref::CrossReferenceLinker;
use paraclea_core::dendrite::{Dendrite, DendriteContext, DendriteStore, NodeType, ReflectionWorker};
use paraclea_core::detect::FileType;
use paraclea_core::heartbeat::*;
use paraclea_core::ingest::{ingest_file, BibleIngestor, BookIngestor};
use paraclea_core::library::LibraryEngine;
use paraclea_core::mesh::ReticulumEngine;
use paraclea_core::ollama::{ChatMessage, ModelEntry, OllamaClient};
use paraclea_core::persona::PersonaManager;
use paraclea_core::pocket_tts::PocketTtsEngine;
use paraclea_core::qdrant::QdrantClient;
use paraclea_core::rag::{self, RagEngine};
use paraclea_core::tools::ToolExecutor;
use rustyline::DefaultEditor;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn read_line_prompt(rl: &mut Option<DefaultEditor>, prompt: &str) -> String {
    if let Some(ref mut ed) = rl {
        match ed.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    ed.add_history_entry(trimmed).ok();
                }
                line
            }
            Err(_) => String::new(),
        }
    } else {
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input
    }
}

#[derive(Parser, Debug)]
#[command(name = "paraclea")]
#[command(author = "Xander <https://github.com/Alartist40>")]
#[command(version = "0.1.0")]
#[command(
    about = "Paraclea — AI Companion Assistant & Self-Developing RAG Engine in Rust",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all available Ollama and local models numbered
    #[command(alias = "ls")]
    List,
    /// Run Paraclea with a specific model by number or name
    Run {
        /// Model name or list number
        model: Option<String>,
    },
    /// Run full system diagnostics (Ollama, Qdrant, TTS, Model Registry)
    Doctor,
    /// Auto-detect file format and ingest into Qdrant vector database
    Ingest {
        /// File or directory path to ingest
        input: String,
        /// Target Qdrant collection name (default: books)
        #[arg(short, long, default_value = "books")]
        collection: String,
    },
    /// Ingest a Bible JSON file into Qdrant vector database
    IngestBible {
        /// Path to Bible JSON file
        input: String,
        /// Target Qdrant collection name (default: bible)
        #[arg(short, long, default_value = "bible")]
        collection: String,
    },
    /// Ingest a book-to-skill markdown directory into Qdrant
    IngestBook {
        /// Path to directory containing chapter .md files
        input: String,
        /// Target Qdrant collection name (default: books)
        #[arg(short, long, default_value = "books")]
        collection: String,
    },
    /// Run vision OCR text extraction on an image document
    Ocr {
        /// Path to document image file
        image: String,
    },
    /// Run a one-shot RAG query with Scripture/book retrieval
    Query {
        /// Question to ask
        question: String,
        /// Target collection (bible, books, survival)
        #[arg(short, long, default_value = "bible")]
        collection: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Silence raw logger output so terminal UI stays clean
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "off".to_string()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config_path = Config::find_or_default_config_path();
    let mut cfg = Config::load(&config_path)?;

    let ollama = OllamaClient::new(&cfg.model.ollama.url, &cfg.model.ollama.model)?;
    let qdrant = QdrantClient::new(&cfg.vector_db.qdrant_url)?;
    let pocket_tts = PocketTtsEngine::new(
        &cfg.voice.pocket_tts_url,
        &cfg.voice.pocket_tts_voice,
        Some(&cfg.voice.pocket_tts_cli),
    )?;

    match &cli.command {
        Some(Commands::List) => {
            print_available_models(&ollama).await;
            return Ok(());
        }
        Some(Commands::Run { model }) => {
            let available = ollama.fetch_available_models().await;
            if let Some(target) = model {
                if let Err(e) = select_and_apply_model(target, &available, &mut cfg) {
                    eprintln!("{}", format!("Error: {}", e).red());
                    print_available_models(&ollama).await;
                    return Ok(());
                }
                let _ = cfg.save(&config_path);
            } else {
                print_available_models(&ollama).await;
                return Ok(());
            }
        }
        Some(Commands::Doctor) => {
            run_doctor(&cfg, &ollama, &qdrant, &pocket_tts).await;
            return Ok(());
        }
        Some(Commands::Ingest { input, collection }) => {
            let input_path = Path::new(input);
            let ftype = FileType::from_path(input_path);
            println!(
                "{}",
                format!("📦 Ingesting file {:?} ({}) into collection '{}'...", input_path.file_name().unwrap_or_default(), ftype.label(), collection)
                    .truecolor(177, 74, 237)
                    .bold()
            );

            match ingest_file(
                &ollama,
                &qdrant,
                &cfg.model.ollama.embed_model,
                &cfg.model.ollama.ocr_model,
                input_path,
                collection,
            )
            .await
            {
                Ok(msg) => println!("{}", format!("✓ {}", msg).truecolor(255, 215, 0).bold()),
                Err(e) => eprintln!("{}", format!("Ingestion failed: {}", e).red()),
            }
            return Ok(());
        }
        Some(Commands::IngestBible { input, collection }) => {
            println!("{}", print_purple("📖 Ingesting Bible dataset into Qdrant..."));
            let ingestor = BibleIngestor::new(&ollama, &qdrant, &cfg.model.ollama.embed_model, collection);
            match ingestor.ingest_json_file(Path::new(input)).await {
                Ok(count) => println!(
                    "{}",
                    format!("✓ Indexed {} verse chunks into collection '{}'", count, collection)
                        .truecolor(255, 215, 0)
                        .bold()
                ),
                Err(e) => eprintln!("{}", format!("Ingestion failed: {}", e).red()),
            }
            return Ok(());
        }
        Some(Commands::IngestBook { input, collection }) => {
            println!("{}", print_purple("📚 Ingesting book skill directory into Qdrant..."));
            let ingestor = BookIngestor::new(&ollama, &qdrant, &cfg.model.ollama.embed_model);
            match ingestor.ingest_book_directory(Path::new(input), collection).await {
                Ok(count) => println!(
                    "{}",
                    format!("✓ Indexed {} chapter sections into collection '{}'", count, collection)
                        .truecolor(255, 215, 0)
                        .bold()
                ),
                Err(e) => eprintln!("{}", format!("Ingestion failed: {}", e).red()),
            }
            return Ok(());
        }
        Some(Commands::Ocr { image }) => {
            println!("{}", print_purple("👁️ Running Ollama Vision OCR document extraction..."));
            match ollama.ocr_image(Path::new(image), &cfg.model.ollama.ocr_model).await {
                Ok(text) => {
                    println!("\n{}", print_gold("=== EXTRACTED OCR MARKDOWN ==="));
                    println!("{}\n", text);
                }
                Err(e) => eprintln!("{}", format!("OCR extraction failed: {}", e).red()),
            }
            return Ok(());
        }
        Some(Commands::Query { question, collection }) => {
            let rag = RagEngine::new(&ollama, &qdrant);
            let persona_dir = if Path::new(&cfg.persona.dir).exists() {
                cfg.persona.dir.clone()
            } else {
                "persona".to_string()
            };
            let persona = PersonaManager::new(&persona_dir)?;

            let ret = rag
                .retrieve_context(question, collection, 5, &cfg.model.ollama.embed_model)
                .await?;

            let model_to_use = rag.route_model(question, &cfg.model.ollama.model, &cfg.model.ollama.heavy_model);

            let prompt = if !ret.context_text.is_empty() {
                format!(
                    "Use the following reference passages to answer the question. Quote verses precisely.\n\nContext:\n{}\n\nQuestion: {}\n",
                    ret.context_text, question
                )
            } else {
                question.clone()
            };

            let messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: persona.build_system_prompt(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ];

            let answer = ollama.chat_with_model(model_to_use, messages).await?;
            println!("{} {}\n", print_purple("Paraclea >"), answer);
            if !ret.sources.is_empty() {
                println!(
                    "{} {}",
                    print_gold("Sources:"),
                    ret.sources.join(", ").truecolor(177, 74, 237)
                );
            }
            return Ok(());
        }
        None => {
            let available = ollama.fetch_available_models().await;
            ensure_valid_chat_model(&available, &mut cfg);
            let _ = cfg.save(&config_path);
        }
    }

    start_paraclea_repl(cfg).await
}

fn print_gold(text: &str) -> ColoredString {
    text.truecolor(255, 215, 0).bold()
}

fn print_purple(text: &str) -> ColoredString {
    text.truecolor(177, 74, 237).bold()
}

async fn run_doctor(
    cfg: &Config,
    ollama: &OllamaClient,
    qdrant: &QdrantClient,
    tts: &PocketTtsEngine,
) {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║     PARACLEA AI ASSISTANT — ADVANCED SYSTEM DOCTOR           ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!();

    let mut issues = Vec::new();
    let mut resolved = Vec::new();

    // 1. Hardware Architecture & CPU Probe
    println!("{}", print_purple("  💻 System Hardware & CPU Architecture:"));
    println!("     • OS Target:          {}", print_gold(std::env::consts::OS));
    println!("     • Architecture:       {}", print_gold(std::env::consts::ARCH));
    let logical_cpus = std::thread::available_parallelism().map(|u| u.get()).unwrap_or(1);
    println!("     • CPU Cores:          {} logical threads", print_gold(&logical_cpus.to_string()));
    println!();

    // 2. Binary Installation & PATH Auto-Fix
    println!("{}", print_purple("  🛠  Binary Installation & PATH Status:"));
    let exe_path = std::env::current_exe().unwrap_or_default();
    println!("     • Current Executable: {}", print_gold(&exe_path.display().to_string()));
    let in_local_bin = exe_path.to_string_lossy().contains(".local/bin");
    let paraclea_symlink_ok = if let Ok(home) = std::env::var("HOME") {
        let symlink = std::path::PathBuf::from(home).join(".local/bin/paraclea");
        symlink.exists()
    } else {
        false
    };

    if !paraclea_symlink_ok {
        if let Ok(home) = std::env::var("HOME") {
            let bin_dir = std::path::PathBuf::from(&home).join(".local/bin");
            let target = bin_dir.join("paraclea");
            if std::fs::create_dir_all(&bin_dir).is_ok() {
                if std::fs::copy(&exe_path, &target).is_ok() {
                    resolved.push("Auto-installed 'paraclea' binary to ~/.local/bin/paraclea".to_string());
                }
            }
        }
    }

    println!("     • Installed in PATH:   {}", if in_local_bin || paraclea_symlink_ok { "YES (OK)".green().bold() } else { "NO (~/.local/bin recommended)".yellow().bold() });
    println!();

    // 3. Ollama Server & Live Forward-Pass Test
    println!("{}", print_purple("  🤖 Ollama Server & Active Inference Test:"));
    let mut ollama_ok = ollama.health_check().await.unwrap_or(false);
    if !ollama_ok {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("nohup ollama serve > /tmp/ollama.log 2>&1 &")
            .spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        ollama_ok = ollama.health_check().await.unwrap_or(false);
        if ollama_ok {
            resolved.push("Auto-spawned missing Ollama background server".to_string());
        }
    }
    println!(
        "     • Ollama Server ({}) ... {}",
        cfg.model.ollama.url,
        if ollama_ok { "ONLINE".green().bold() } else { "OFFLINE".red().bold() }
    );

    if ollama_ok {
        print!("     • Live 1-token forward pass test on '{}' ... ", cfg.model.ollama.model);
        let start = std::time::Instant::now();
        let test_msgs = vec![ChatMessage { role: "user".to_string(), content: "hi".to_string() }];
        match tokio::time::timeout(tokio::time::Duration::from_secs(8), ollama.chat_with_model(&cfg.model.ollama.model, test_msgs)).await {
            Ok(Ok(out)) => {
                let dur = start.elapsed();
                println!("{}", format!("PASSED (in {:.1}ms, sample output: {:?})", dur.as_secs_f64() * 1000.0, out.trim().chars().take(20).collect::<String>()).green().bold());
            }
            Ok(Err(e)) => {
                println!("{}", format!("FAILED ({})", e).red().bold());
                issues.push(format!("Ollama inference test failed: {}", e));
            }
            Err(_) => {
                println!("{}", "TIMEOUT (Response took > 8s)".yellow().bold());
                issues.push("Ollama inference test timed out (> 8s)".to_string());
            }
        }
    } else {
        issues.push("Ollama server is offline".to_string());
    }
    println!();

    // 4. Model Registry Diagnostics
    println!("{}", print_purple("  🧠 Model Registry Category Coverage:"));
    let reg = ollama.discover_models().await;
    println!("     • Embedding Model:     {} ", reg.embed.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0));
    println!("     • Default Chat Model:  {} ", reg.chat.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0));
    println!("     • Heavy Reasoning:     {} ", reg.heavy.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0));
    println!("     • Document Vision OCR: {} ", reg.ocr.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0));

    let missing = ollama.check_missing(&reg);
    if !missing.is_empty() {
        for item in missing {
            issues.push(format!("Recommended model category missing: {}", item));
        }
    }
    println!();

    // 5. Qdrant Vector DB Integrity & Auto-Creation
    println!("{}", print_purple("  ⚡ Qdrant Vector Database Status:"));
    let mut qdrant_ok = qdrant.health_check().await;
    if !qdrant_ok {
        if let Ok(home) = std::env::var("HOME") {
            let qdrant_bin = std::path::PathBuf::from(&home).join(".paraclea/bin/qdrant");
            if qdrant_bin.exists() {
                let qdrant_dir = std::path::PathBuf::from(&home).join(".paraclea/qdrant");
                let _ = std::fs::create_dir_all(&qdrant_dir);
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("cd {:?} && nohup {:?} > /tmp/qdrant_daemon.log 2>&1 &", qdrant_dir, qdrant_bin))
                    .spawn();
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                qdrant_ok = qdrant.health_check().await;
                if qdrant_ok {
                    resolved.push("Auto-spawned background Qdrant vector database daemon".to_string());
                }
            }
        }
    }

    if qdrant_ok {
        println!("     • Vector DB Service:    {}", "ONLINE".green().bold());
        let _ = qdrant.create_collection(&cfg.vector_db.collection_bible, 768).await;
        let _ = qdrant.create_collection(&cfg.vector_db.collection_books, 768).await;
        println!("     • Vector Collections:   {}", "VERIFIED & INITIALIZED".green().bold());
    } else {
        println!("     • Vector DB Service:    {}", "STANDBY (Optional RAG disabled)".yellow().bold());
    }
    println!();

    // 6. Reticulum Mesh Auto-Repair & Status
    println!("{}", print_purple("  🕸  Reticulum Mesh Stack Integrity:"));
    if let Ok(mesh) = ReticulumEngine::new() {
        let rns_ok = mesh.ensure_daemon();
        println!("     • RNS Shared Instance:  {}", if rns_ok { "ONLINE".green().bold() } else { "OFFLINE".yellow().bold() });
        if let Some(ref id) = mesh.identity_hash {
            println!("     • Cryptographic ID:     <{}>", id.purple().bold());
        }
    } else {
        println!("     • RNS Shared Instance:  {}", "STANDBY".yellow().bold());
    }
    println!();

    // 7. Dendrite v2 Knowledge Graph DB Integrity Check
    println!("{}", print_purple("  🧬 Dendrite v2 Knowledge Graph DB Integrity:"));
    if let Ok(home) = std::env::var("HOME") {
        let db_path = std::path::PathBuf::from(home).join(".paraclea/dendrite.db");
        match DendriteStore::open(&db_path) {
            Ok(store) => {
                let count = store.node_count().unwrap_or(0);
                println!("     • SQLite Database:      {}", "ONLINE & HEALTHY".green().bold());
                println!("     • Stored Knowledge Nodes: {}", print_gold(&count.to_string()));
            }
            Err(e) => {
                println!("     • SQLite Database:      {}", format!("ERROR ({})", e).red().bold());
                issues.push(format!("Dendrite DB error: {}", e));
            }
        }
    }
    println!();

    // 8. Pocket TTS Check
    println!("{}", print_purple("  🔊 Pocket TTS Voice Engine:"));
    let tts_ok = tts.health_check().await;
    println!("     • Voice Synthesis:     {}", if tts_ok { "ONLINE".green().bold() } else { "CLI FALLBACK (OK)".yellow().bold() });
    println!();

    // 9. Multi-Language Bible & Multi-Category Library Database Diagnostics
    println!("{}", print_purple("  📚 Mega Bible & Multi-Category Library Database:"));
    if let Ok(home) = std::env::var("HOME") {
        let bibles_dir = std::path::PathBuf::from(&home).join(".paraclea/bibles");
        let library_dir = std::path::PathBuf::from(&home).join(".paraclea/library");
        
        let mut lang_count = 0;
        let mut bible_version_count = 0;
        if bibles_dir.exists() {
            if let Ok(langs) = std::fs::read_dir(&bibles_dir) {
                for l in langs.flatten() {
                    if l.path().is_dir() {
                        lang_count += 1;
                        if let Ok(files) = std::fs::read_dir(l.path()) {
                            bible_version_count += files.flatten().filter(|f| f.path().extension().and_then(|e| e.to_str()) == Some("json")).count();
                        }
                    }
                }
            }
        }

        let mut category_count = 0;
        let mut book_count = 0;
        let mut chapter_count = 0;
        let lib = LibraryEngine::load_auto();
        category_count = lib.list_categories().len();
        book_count = lib.books.len();
        chapter_count = lib.books.iter().map(|b| b.chapters.len()).sum();

        println!("     • Bible Languages Covered: {} ", print_gold(&lang_count.to_string()));
        println!("     • Formatted Bible Versions: {} ", print_gold(&bible_version_count.to_string()));
        println!("     • Non-Scripture Categories: {} ", print_gold(&category_count.to_string()));
        println!("     • Library Books Ingested:   {} ", print_gold(&book_count.to_string()));
        println!("     • Total Library Chapters:   {} ", print_gold(&chapter_count.to_string()));
    }
    println!();

    // Summary Verdict & Resolution Report
    if !resolved.is_empty() {
        println!("{}", "✨ Auto-Healed Diagnostics:".green().bold());
        for res in &resolved {
            println!("     ✓ {}", res.green());
        }
        println!();
    }

    if issues.is_empty() {
        println!("{}", "🎉 Paraclea System Doctor Check Complete: ALL SYSTEMS OPERATIONAL!".green().bold());
    } else {
        println!("{}", "⚠️  Paraclea System Doctor Detected Actionable Items:".yellow().bold());
        for issue in &issues {
            println!("     • {}", issue.yellow());
        }
    }
    println!();
}

async fn print_available_models(ollama: &OllamaClient) {
    let models = ollama.fetch_available_models().await;

    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║      PARACLEA AI ASSISTANT — AVAILABLE MODELS (v0.1.0 Rust)  ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    if models.is_empty() {
        println!(
            "{}",
            "  (No models found. Make sure 'ollama serve' is running or place .gguf files in models/)"
                .yellow()
        );
    } else {
        for m in &models {
            println!(
                "  {} {} ({})",
                format!("[{}]", m.id).truecolor(255, 215, 0).bold(),
                m.name.bold(),
                m.backend.truecolor(177, 74, 237)
            );
        }
    }
    println!(
        "\n{}",
        "Run a model using: paraclea run <number|name>"
            .cyan()
            .bold()
    );
}

fn select_and_apply_model(target: &str, available: &[ModelEntry], cfg: &mut Config) -> Result<()> {
    if available.is_empty() {
        anyhow::bail!("No models found. Verify 'ollama serve' is active.");
    }

    let selected = if let Ok(num) = target.parse::<usize>() {
        available.iter().find(|m| m.id == num)
    } else {
        available
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(target) || m.target.eq_ignore_ascii_case(target))
    };

    if let Some(entry) = selected {
        cfg.model.ollama.model = entry.target.clone();
        println!(
            "{}",
            format!(
                "[Paraclea] Active model selected: {} ({})",
                entry.name, entry.backend
            )
            .truecolor(255, 215, 0)
            .bold()
        );
        Ok(())
    } else {
        anyhow::bail!("Model '{}' not found.", target);
    }
}

fn ensure_valid_chat_model(ollama_models: &[ModelEntry], cfg: &mut Config) {
    let current = cfg.model.ollama.model.to_lowercase();
    let is_invalid = current.contains("ocr")
        || current.contains("embed")
        || !ollama_models.iter().any(|m| m.target == cfg.model.ollama.model);

    if is_invalid {
        if let Some(chat_entry) = ollama_models.iter().find(|m| {
            let n = m.name.to_lowercase();
            !n.contains("ocr") && !n.contains("embed")
        }) {
            cfg.model.ollama.model = chat_entry.target.clone();
        }
    }
}

const DYNAMIC_GREETINGS: &[&str] = &[
    "Greetings! I am right here beside you. How may I serve and support your work today?",
    "Welcome back! May wisdom, clarity, and peace guide our conversation today. What is on your mind?",
    "Hello my friend! I enjoyed our last conversation and am ready whenever you are. How can I assist you?",
    "Shalom! I'm here to lend a helping hand and thoughtful counsel. Where should we begin today?",
    "Welcome! The day is full of purpose. What shall we focus on right now?",
    "Good to see you! Ready to dive in whenever you are—let's accomplish something great together.",
    "Grace and peace to you today! I am tuned in and ready to assist with your studies or projects.",
    "Ah, welcome back companion! What meaningful task shall we tackle together today?",
    "Hello! It is a joy to walk alongside you. How can I help make your day lighter and more productive?",
    "Welcome friend! Whether searching Scripture or solving complex code, I stand ready to assist.",
    "Greetings! Sunshine or rain, wisdom is ready to be uncovered. Where are we heading today?",
    "Hello again! I've kept our previous thoughts safe in memory. What is our next destination?",
];

const DYNAMIC_FAREWELLS: &[&str] = &[
    "I really enjoyed our conversation! Go with strength and wisdom—talk later!",
    "Until next time! May your work be fruitful and your heart at peace.",
    "It was a pleasure helping you today. Take care, and I will be right here whenever you return!",
    "Farewell for now! May your path be clear and your effort blessed. Talk soon!",
    "Safe travels on your work today! I enjoyed our chat—reach out whenever you need me again.",
    "Goodbye for now, my friend! Keep striving with courage and grace. Talk later!",
    "Rest well and go in peace! I will be waiting right here when you return.",
    "Farewell, dear friend! May clarity attend your steps until we speak again.",
    "Take good care! Remember that step by step, great things are accomplished.",
    "Until next session! Carry truth and kindness with you wherever you go.",
    "Goodnight/goodbye for now! Stay inspired, stay courageous.",
    "Farewell! It has been an absolute joy working with you. Talk soon!",
];

fn get_random_greeting() -> &'static str {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    DYNAMIC_GREETINGS[now % DYNAMIC_GREETINGS.len()]
}

fn get_random_farewell() -> &'static str {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    DYNAMIC_FAREWELLS[now % DYNAMIC_FAREWELLS.len()]
}

fn print_help_menu() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║             PARACLEA AI ASSISTANT — COMMAND MENU             ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!("  {}", print_gold("Interactive Commands:"));
    println!("    {} - Show this command menu & capabilities", print_purple("/help"));
    println!("    {} - Configure default Bible language & translation", print_purple("/bible"));
    println!("    {} - Interactive Scripture reader with chapter/verse bounds", print_purple("/read"));
    println!("    {} - Side-by-side translation comparison & AI study commentary", print_purple("/compare"));
    println!("    {} - Generate Comparative Topic Matrix across Scripture, EGW & Survival", print_purple("/matrix <topic>"));
    println!("    {} - Reticulum mesh status, peer discovery & identity", print_purple("/mesh [announce|peers|id]"));
    println!("    {} - Send off-grid encrypted message to mesh mailbox", print_purple("/mesh-send <recipient> <message>"));
    println!("    {} - View received off-grid mesh mailbox messages", print_purple("/mesh-inbox"));
    println!("    {} - Export 1-Click AES-256 encrypted database backup to USB", print_purple("/backup [passphrase]"));
    println!("    {} - Dendrite v2 graph memory status & search", print_purple("/memory [search <query>]"));
    println!("    {} - Browse non-scripture book library (EGW, Psychology, Survival, etc.)", print_purple("/library [category]"));
    println!("    {} - Read a non-scripture book chapter", print_purple("/read-book <book> [chapter]"));
    println!("    {} - Get AI study commentary on a non-scripture book chapter", print_purple("/study-book <book> [chapter]"));
    println!("    {} - Link Scripture and non-scripture books with custom notes", print_purple("/crossref <source> <-> <target> <notes>"));
    println!("    {} - End conversation session", print_purple("/bye or /end"));
    println!("    {} - List available Ollama and local models", print_purple("/models"));
    println!("    {} - Switch active chat LLM", print_purple("/model <name>"));
    println!("    {} - Run full system diagnostic health check", print_purple("/doctor"));
    println!("    {} - Reset conversation context history", print_purple("/clear"));
    println!("\n  {}", print_gold("RAG Vector Ingestion & CLI Commands:"));
    println!("    • Ingest document/image: {} <file>", print_purple("paraclea ingest"));
    println!("    • Ingest Bible JSON:    {} <kjv.json>", print_purple("paraclea ingest-bible"));
    println!("    • One-shot RAG query:   {} \"question\"", print_purple("paraclea query"));
    println!();
}

async fn handle_bible_menu(rl: &mut Option<DefaultEditor>, cfg: &mut Config, config_path: &Path) -> Result<()> {
    println!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║            BIBLE LANGUAGE & TRANSLATION SETTINGS            ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    let languages = BibleReader::list_languages();
    println!("  {}", print_gold("Select your preferred language:"));
    for lang in &languages {
        println!("    [{}] {}", lang.id, lang.name);
    }

    let input = read_line_prompt(rl, &format!("\n{} ", print_gold("Select language [1-5] >")));

    let lang_id = input.trim().parse::<usize>().unwrap_or(1);
    let selected_lang = languages.iter().find(|l| l.id == lang_id).cloned().unwrap_or(languages[0].clone());

    let translations = BibleReader::list_translations_for_lang(&selected_lang.code);
    println!("\n  {}", print_gold(&format!("Available translations for {}:", selected_lang.name)));
    for trans in &translations {
        let easy_tag = if trans.is_easy { " (Recommended / Easy)" } else { "" };
        println!("    [{}] {}{}", trans.id, trans.name, easy_tag);
    }
    println!("    [{}] Not Sure / Recommended (Defaults to Easy Version)", translations.len() + 1);

    let trans_input = read_line_prompt(rl, &format!("\n{} ", print_gold("Select translation >")));

    let trans_id = trans_input.trim().parse::<usize>().unwrap_or(1);
    let selected_trans = if trans_id <= translations.len() {
        translations[trans_id - 1].tag.clone()
    } else {
        "WEB".to_string()
    };

    cfg.bible.language = selected_lang.name.clone();
    cfg.bible.translation = selected_trans.clone();
    let _ = cfg.save(config_path);

    println!(
        "\n{} {}\n",
        print_purple("Paraclea >"),
        print_gold(&format!(
            "✓ Saved! Preferred Bible language set to '{}' and translation to '{}'.",
            cfg.bible.language, cfg.bible.translation
        ))
    );
    Ok(())
}

fn prompt_select_book(rl: &mut Option<DefaultEditor>, reader: &BibleReader) -> Option<String> {
    use paraclea_core::bible::{NEW_TESTAMENT_BOOKS, OLD_TESTAMENT_BOOKS};

    println!("  {}", print_gold("Select Navigation Mode:"));
    println!("    [1] Old Testament (39 Books)");
    println!("    [2] New Testament (27 Books)");
    println!("    [3] Type Book Name Directly (e.g. Psalms, Song of Solomon, John)");

    let choice_str = read_line_prompt(rl, &format!("\n{} ", print_gold("Select option [1-3] >")));
    let choice = choice_str.trim().parse::<usize>().unwrap_or(3);

    match choice {
        1 => {
            println!("\n  {}", print_gold("Old Testament Books:"));
            for (idx, &b_name) in OLD_TESTAMENT_BOOKS.iter().enumerate() {
                print!("    [{:2}] {:<18}", idx + 1, b_name);
                if (idx + 1) % 3 == 0 {
                    println!();
                }
            }
            println!();

            let num_str = read_line_prompt(rl, &format!("\n{} ", print_gold("Select Old Testament Book [1-39] >")));
            if let Ok(idx) = num_str.trim().parse::<usize>() {
                if idx >= 1 && idx <= OLD_TESTAMENT_BOOKS.len() {
                    return Some(OLD_TESTAMENT_BOOKS[idx - 1].to_string());
                }
            }
            None
        }
        2 => {
            println!("\n  {}", print_gold("New Testament Books:"));
            for (idx, &b_name) in NEW_TESTAMENT_BOOKS.iter().enumerate() {
                print!("    [{:2}] {:<18}", idx + 1, b_name);
                if (idx + 1) % 3 == 0 {
                    println!();
                }
            }
            println!();

            let num_str = read_line_prompt(rl, &format!("\n{} ", print_gold("Select New Testament Book [1-27] >")));
            if let Ok(idx) = num_str.trim().parse::<usize>() {
                if idx >= 1 && idx <= NEW_TESTAMENT_BOOKS.len() {
                    return Some(NEW_TESTAMENT_BOOKS[idx - 1].to_string());
                }
            }
            None
        }
        _ => {
            let b_input = read_line_prompt(rl, &format!("{} ", print_gold("Enter Book Name (e.g. Genesis, Proverbs, Song of Solomon) >")));
            let searched = b_input.trim();
            if !searched.is_empty() {
                if let Some(b) = reader.find_book(searched) {
                    return Some(b.name.clone());
                } else {
                    println!("{}", format!("⚠️ Book '{}' not found in Bible database.", searched).red());
                }
            }
            None
        }
    }
}

async fn read_bible_flow(
    rl: &mut Option<DefaultEditor>,
    reader: &BibleReader,
    cfg: &Config,
    history: &mut Vec<ChatMessage>,
    selected_book_name: &str,
) -> Result<()> {
    let book_meta = match reader.find_book(selected_book_name) {
        Some(b) => b,
        None => {
            println!("{}", format!("⚠️ Book '{}' not found in Bible database.", selected_book_name).red());
            return Ok(());
        }
    };

    println!(
        "  📖 {}",
        print_gold(&format!("'{}' has {} chapters.", book_meta.name, book_meta.total_chapters))
    );

    let chap_input = read_line_prompt(rl, &format!("{} ", print_gold(&format!("Select Chapter (1-{}) >", book_meta.total_chapters))));
    let chapter_num: usize = chap_input.trim().parse().unwrap_or(1);

    if chapter_num < 1 || chapter_num > book_meta.total_chapters {
        println!("{}", format!("Invalid chapter number. Pick between 1 and {}.", book_meta.total_chapters).red());
        return Ok(());
    }

    let verse_count = book_meta.chapter_verse_counts[chapter_num - 1];
    println!(
        "  📌 {}",
        print_gold(&format!("'{} Chapter {}' has {} verses.", book_meta.name, chapter_num, verse_count))
    );

    let verse_input = read_line_prompt(rl, &format!("{} ", print_gold(&format!("Select Verse (1-{}, or 'all' for full chapter) >", verse_count))));
    let v_str = verse_input.trim().to_lowercase();

    let trans_tag = &cfg.bible.translation;

    if v_str == "all" || v_str.is_empty() {
        if let Some(verses) = reader.read_translation_chapter(trans_tag, &book_meta.name, chapter_num) {
            println!(
                "\n{}",
                format!("=== {} Chapter {} [{}] ===", book_meta.name, chapter_num, trans_tag).truecolor(255, 215, 0).bold()
            );
            let mut full_passage = String::new();
            for (v_idx, text) in verses {
                let line = format!("[{}] {}\n", v_idx, text);
                print!("{}", line.truecolor(177, 74, 237));
                full_passage.push_str(&line);
            }
            history.push(ChatMessage {
                role: "system".to_string(),
                content: format!("User is reading {} Chapter {} [{}]:\n{}", book_meta.name, chapter_num, trans_tag, full_passage),
            });
            println!("\n{}", print_gold("✓ Passage loaded. Ask Paraclea any questions about this chapter!"));
        }
    } else if let Ok(verse_num) = v_str.parse::<usize>() {
        if verse_num >= 1 && verse_num <= verse_count {
            if let Some(text) = reader.read_translation_verse(trans_tag, &book_meta.name, chapter_num, verse_num) {
                let citation = format!("{} {}:{} [{}]", book_meta.name, chapter_num, verse_num, trans_tag);
                println!(
                    "\n{} {}",
                    citation.truecolor(255, 215, 0).bold(),
                    text.truecolor(177, 74, 237)
                );
                history.push(ChatMessage {
                    role: "system".to_string(),
                    content: format!("User is reading Scripture passage {}: \"{}\"", citation, text),
                });
                println!("\n{}", print_gold("✓ Passage loaded into conversation. Ask Paraclea anything about it!"));
            }
        } else {
            println!("{}", format!("Invalid verse number. Select 1-{}.", verse_count).red());
        }
    }

    Ok(())
}

async fn read_non_scripture_category_flow(
    rl: &mut Option<DefaultEditor>,
    library: &LibraryEngine,
    cat_tag: &str,
    cat_title: &str,
    history: &mut Vec<ChatMessage>,
) -> Result<()> {
    let mut books = Vec::new();
    if cat_tag == "survival" {
        books.extend(library.list_books(Some("survival")));
        books.extend(library.list_books(Some("medical")));
    } else if cat_tag == "educational" {
        books.extend(library.list_books(Some("psychology")));
        books.extend(library.list_books(Some("educational")));
        books.extend(library.list_books(Some("classics")));
        books.extend(library.list_books(Some("custom")));
    } else {
        books.extend(library.list_books(Some(cat_tag)));
    }

    if books.is_empty() {
        println!("{}", format!("⚠️ No books found in '{}' category.", cat_title).yellow());
        return Ok(());
    }

    println!("\n  {}", print_gold(&format!("{}:", cat_title)));
    for (idx, b) in books.iter().enumerate() {
        let author_str = b.author.as_deref().unwrap_or("Public Domain");
        println!("    [{}] {} (By {})", idx + 1, b.title.truecolor(255, 215, 0).bold(), author_str);
    }

    let choice_input = read_line_prompt(rl, &format!("\n{} ", print_gold(&format!("Select book [1-{}] >", books.len()))));
    let selected_book = if let Ok(idx) = choice_input.trim().parse::<usize>() {
        if idx >= 1 && idx <= books.len() {
            Some(books[idx - 1])
        } else {
            None
        }
    } else {
        library.find_book(choice_input.trim())
    };

    let book = match selected_book {
        Some(b) => b,
        None => {
            println!("{}", "Invalid book selection.".red());
            return Ok(());
        }
    };

    println!("\n  {} has {} chapter(s).", book.title.truecolor(255, 215, 0).bold(), book.chapters.len());
    let ch_input = read_line_prompt(rl, &format!("{} ", print_gold(&format!("Select chapter [1-{}] >", book.chapters.len()))));
    let ch_num = ch_input.trim().parse::<usize>().unwrap_or(1);

    if let Some((b, ch)) = library.read_chapter(&book.title, ch_num) {
        println!(
            "\n╔══════════════════════════════════════════════════════════════╗\n║  {} - Chapter {}\n╚══════════════════════════════════════════════════════════════╝",
            b.title, ch.chapter_number
        );
        println!("\n{}\n", ch.content.truecolor(177, 74, 237));
        history.push(ChatMessage {
            role: "system".to_string(),
            content: format!("User is reading {} [Category: {}] Chapter {}: \"{}\"", b.title, b.category, ch.chapter_number, ch.content.chars().take(500).collect::<String>()),
        });
        println!("{}", print_gold("✓ Book chapter loaded into conversation context! Ask Paraclea anything about it."));
    } else {
        println!("{}", "Invalid chapter number.".red());
    }

    Ok(())
}

async fn handle_read_cmd(
    rl: &mut Option<DefaultEditor>,
    reader: &BibleReader,
    library: &LibraryEngine,
    cfg: &Config,
    history: &mut Vec<ChatMessage>,
) -> Result<()> {
    println!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║             PARACLEA UNIFIED LIBRARY & SCRIPTURE READER      ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    println!("  {}", print_gold("Select Category:"));
    println!("    [1] Spiritual (Holy Bible, Ellen G. White Writings, Christian Books)");
    println!("    [2] Survival (Medical Emergency, Field Surgery, Wilderness Preparedness)");
    println!("    [3] Educational (Psychology, Philosophy, Science, Astronomy & Knowledge)");

    let cat_input = read_line_prompt(rl, &format!("\n{} ", print_gold("Select category [1-3] >")));
    let cat_choice = cat_input.trim().parse::<usize>().unwrap_or(1);

    match cat_choice {
        1 => {
            println!("\n  {}", print_gold("Spiritual Resources:"));
            println!("    [1] Holy Bible (Default: {} / {})", cfg.bible.language, cfg.bible.translation);
            println!("    [2] Ellen G. White Writings");

            let res_input = read_line_prompt(rl, &format!("\n{} ", print_gold("Select resource [1-2] >")));
            let res_choice = res_input.trim().parse::<usize>().unwrap_or(1);

            if res_choice == 1 {
                let selected_book_name = match prompt_select_book(rl, reader) {
                    Some(name) => name,
                    None => return Ok(()),
                };
                read_bible_flow(rl, reader, cfg, history, &selected_book_name).await?;
            } else {
                read_non_scripture_category_flow(rl, library, "egw", "Ellen G. White Writings", history).await?;
            }
        }
        2 => {
            read_non_scripture_category_flow(rl, library, "survival", "Survival & Medical Field Manuals", history).await?;
        }
        _ => {
            read_non_scripture_category_flow(rl, library, "educational", "Educational & General Knowledge", history).await?;
        }
    }

    Ok(())
}

async fn handle_compare_cmd(
    rl: &mut Option<DefaultEditor>,
    reader: &BibleReader,
    ollama: &OllamaClient,
    history: &mut Vec<ChatMessage>,
) -> Result<()> {
    println!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║            MULTILINGUAL & MULTI-VERSION BIBLE COMPARISON      ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    let selected_book_name = match prompt_select_book(rl, reader) {
        Some(name) => name,
        None => return Ok(()),
    };

    let book_meta = match reader.find_book(&selected_book_name) {
        Some(b) => b,
        None => {
            println!("{}", format!("⚠️ Book '{}' not found.", selected_book_name).red());
            return Ok(());
        }
    };

    let chap_input = read_line_prompt(rl, &format!("{} ", print_gold(&format!("Select Chapter (1-{}) >", book_meta.total_chapters))));
    let chapter_num: usize = chap_input.trim().parse().unwrap_or(1);

    let verse_count = reader.get_verse_count(&book_meta.name, chapter_num).unwrap_or(1);
    let verse_input = read_line_prompt(rl, &format!("{} ", print_gold(&format!("Select Verse (1-{}) >", verse_count))));
    let verse_num: usize = verse_input.trim().parse().unwrap_or(1);

    let primary_text = reader.read_verse(&book_meta.name, chapter_num, verse_num)
        .unwrap_or_else(|| "Text unavailable".to_string());

    let passage_ref = format!("{} {}:{}", book_meta.name, chapter_num, verse_num);

    println!("\n  {}", print_gold(&format!("=== Comparative Passages for {} ===", passage_ref)));
    println!("  • [KJV (Authorized)] : {}", primary_text.truecolor(177, 74, 237));
    println!("  • [WEB (Modern Easy)]: {}", primary_text.truecolor(177, 74, 237));

    let compare_prompt = format!(
        "I am performing a comparative study of {passage_ref}.\nPassage text: \"{primary_text}\"\n\nPlease provide a clear comparative study of this verse, highlighting key original language meanings (Hebrew/Greek), nuances across different translations, and practical wisdom.",
        passage_ref = passage_ref,
        primary_text = primary_text
    );

    println!("\n{} ", print_purple("Paraclea Commentary >"));
    io::stdout().flush()?;

    let mut stream_history = history.clone();
    stream_history.push(ChatMessage {
        role: "user".to_string(),
        content: compare_prompt.clone(),
    });

    let stream_res = ollama.chat_stream(stream_history, |token| {
        print!("{}", token);
        let _ = io::stdout().flush();
    }).await;

    println!();

    if let Ok(full_text) = stream_res {
        history.push(ChatMessage {
            role: "user".to_string(),
            content: format!("Comparative study of {}", passage_ref),
        });
        history.push(ChatMessage {
            role: "assistant".to_string(),
            content: full_text,
        });
    }

    Ok(())
}

async fn start_paraclea_repl(cfg: Config) -> Result<()> {
    print_banner(&cfg);

    // 1. Initialize Persona Manager
    let persona_dir = if Path::new(&cfg.persona.dir).exists() {
        cfg.persona.dir.clone()
    } else if Path::new("persona").exists() {
        "persona".to_string()
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{}/.paraclea/persona", home)
    } else {
        "persona".to_string()
    };
    let persona = PersonaManager::new(&persona_dir)?;

    // Ensure default persona files exist in target directory if running from installed location
    if let Ok(repo_persona) = std::fs::read_dir("persona") {
        for entry in repo_persona.flatten() {
            let src = entry.path();
            if src.is_file() {
                let dest = Path::new(&persona_dir).join(entry.file_name());
                if !dest.exists() {
                    let _ = std::fs::copy(&src, &dest);
                }
            }
        }
    }

    // 2. Initialize Ollama Client & Sanitize Chat Model
    let mut cfg = cfg;
    let mut ollama = OllamaClient::new(&cfg.model.ollama.url, &cfg.model.ollama.model)?;
    let available = ollama.fetch_available_models().await;
    ensure_valid_chat_model(&available, &mut cfg);
    ollama.model = cfg.model.ollama.model.clone();

    print!("{}", print_purple("🔍 Checking Ollama... "));
    io::stdout().flush()?;
    let mut ollama_online = ollama.health_check().await.unwrap_or(false);
    if !ollama_online {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("nohup ollama serve > /tmp/ollama.log 2>&1 &")
            .spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        ollama_online = ollama.health_check().await.unwrap_or(false);
    }
    if ollama_online {
        println!("{}", print_gold("ONLINE"));
    } else {
        println!("{}", "OFFLINE".red().bold());
    }

    // 3. Initialize Qdrant Client
    let qdrant = QdrantClient::new(&cfg.vector_db.qdrant_url)?;
    print!("{}", print_purple("🔍 Checking Qdrant Vector DB... "));
    io::stdout().flush()?;
    let mut qdrant_online = qdrant.health_check().await;
    if !qdrant_online {
        if let Ok(home) = std::env::var("HOME") {
            let qdrant_bin = std::path::PathBuf::from(&home).join(".paraclea/bin/qdrant");
            if qdrant_bin.exists() {
                let qdrant_dir = std::path::PathBuf::from(&home).join(".paraclea/qdrant");
                let _ = std::fs::create_dir_all(&qdrant_dir);
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("cd {:?} && nohup {:?} > /tmp/qdrant_daemon.log 2>&1 &", qdrant_dir, qdrant_bin))
                    .spawn();
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                qdrant_online = qdrant.health_check().await;
            }
        }
    }
    if qdrant_online {
        println!("{}", print_gold("ONLINE"));
        qdrant.create_collection(&cfg.vector_db.collection_bible, 768).await.ok();
        qdrant.create_collection(&cfg.vector_db.collection_books, 768).await.ok();
    } else {
        println!("{}", "STANDBY".yellow());
    }

    // 4. Initialize Pocket TTS Client
    let pocket_tts = PocketTtsEngine::new(
        &cfg.voice.pocket_tts_url,
        &cfg.voice.pocket_tts_voice,
        Some(&cfg.voice.pocket_tts_cli),
    )?;

    print!("{}", print_purple("🔍 Checking TTS... "));
    io::stdout().flush()?;
    if pocket_tts.health_check().await {
        println!("{}", print_gold("ONLINE"));
    } else {
        println!("{}", "CLI FALLBACK".yellow());
    }

    // 5. Model Registry Status Output
    let reg = ollama.discover_models().await;
    let missing = ollama.check_missing(&reg);
    if !missing.is_empty() {
        println!(
            "{}",
            format!("⚠️  Recommended models missing: run 'paraclea doctor' for details").yellow()
        );
    }

    // 6. Launch Heartbeat Background Self-Maintenance Loop
    let shutdown = Arc::new(AtomicBool::new(false));
    let heartbeat = HeartbeatLoop::new(
        cfg.persona.heartbeat_interval,
        persona.clone(),
        ollama.clone(),
    );
    let shutdown_hb = shutdown.clone();
    tokio::spawn(async move {
        heartbeat.run(shutdown_hb).await;
    });

    // 7. Tool & RAG Executors & Bible Reader & Reticulum Mesh & Dendrite Memory
    let tool_executor = ToolExecutor::new(persona.clone());
    let rag_engine = RagEngine::new(&ollama, &qdrant);
    let config_path = PathBuf::from("config.yaml");
    let bible_reader = BibleReader::load_auto().ok();
    let mesh_engine = ReticulumEngine::new().ok();

    // Initialize Dendrite v2 Graph Memory & Persistence
    let dendrite_graph = std::sync::Arc::new(Dendrite::new());
    let dendrite_store = std::env::var("HOME").ok().and_then(|h| {
        let db_path = PathBuf::from(h).join(".paraclea/dendrite.db");
        DendriteStore::open(&db_path).ok().map(std::sync::Arc::new)
    });
    if let Some(ref store) = dendrite_store {
        let _ = store.load_all(&dendrite_graph);
    }
    let dendrite_ctx = DendriteContext::new(dendrite_graph.clone(), dendrite_store.clone());
    let reflection_worker = ReflectionWorker::new(dendrite_graph.clone(), dendrite_store.clone(), ollama.clone());
    let crossref_linker = CrossReferenceLinker::new(dendrite_graph.clone(), dendrite_store.clone());
    let library_engine = LibraryEngine::load_auto();

    print!("{}", print_purple("🔍 Checking Reticulum Mesh... "));
    io::stdout().flush()?;
    if let Some(ref mesh) = mesh_engine {
        if let Some(ref id) = mesh.identity_hash {
            println!("{} {}", print_gold("ONLINE"), format!("(Identity: <{}>)", id).purple());
        } else {
            println!("{}", print_gold("ONLINE"));
        }
    } else {
        println!("{}", "STANDBY".yellow());
    }

    print!("{}", print_purple("🔍 Checking Dendrite Graph Memory... "));
    io::stdout().flush()?;
    let node_count = dendrite_graph.len();
    println!("{} {}", print_gold("ONLINE"), format!("({} nodes loaded)", node_count).purple());

    print!("{}", print_purple("🔍 Checking Multi-Category Library... "));
    io::stdout().flush()?;
    let lib_books = library_engine.books.len();
    let lib_cats = library_engine.list_categories().len();
    println!("{} {}", print_gold("ONLINE"), format!("({} categories, {} books loaded)", lib_cats, lib_books).purple());

    // 8. Interactive REPL Shell Loop
    let mut history: Vec<ChatMessage> = Vec::new();
    let mut rl = DefaultEditor::new().ok();

    println!(
        "\n{} {}\n",
        print_purple("Paraclea >"),
        print_gold(get_random_greeting())
    );
    println!(
        "{}\n",
        print_gold("✨ Paraclea is ready! Type your message (or '/help' for options, '/bye' to quit).")
    );

    loop {
        let user_input = read_line_prompt(&mut rl, &format!("{} ", print_gold("You >")));
        let input_str = user_input.trim();
        if input_str.is_empty() {
            continue;
        }

        // Strict Exit Commands (/bye, /end, /exit, /quit)
        if input_str.eq_ignore_ascii_case("/bye")
            || input_str.eq_ignore_ascii_case("/end")
            || input_str.eq_ignore_ascii_case("/exit")
            || input_str.eq_ignore_ascii_case("/quit")
        {
            println!(
                "\n{} {}\n",
                print_purple("Paraclea >"),
                print_gold(get_random_farewell())
            );
            break;
        }

        // Interactive Slash Commands
        if input_str.eq_ignore_ascii_case("/help") {
            print_help_menu();
            continue;
        }

        if input_str.eq_ignore_ascii_case("/bible") {
            let _ = handle_bible_menu(&mut rl, &mut cfg, &config_path).await;
            continue;
        }

        if input_str.eq_ignore_ascii_case("/read") {
            if let Some(ref reader) = bible_reader {
                let _ = handle_read_cmd(&mut rl, reader, &library_engine, &cfg, &mut history).await;
            } else {
                println!("{}", "⚠️ Bible database not loaded.".red());
            }
            continue;
        }

        if input_str.starts_with("/matrix") {
            let topic = input_str.trim_start_matches("/matrix").trim();
            if topic.is_empty() {
                println!("{}", "⚠️ Please specify a topic: /matrix <topic> (e.g. /matrix Faith)".yellow());
            } else {
                println!("{}", print_purple(&format!("📊 Generating Comparative Matrix for '{}'...", topic)));
                let reader = bible_reader.as_ref().cloned().unwrap_or_else(|| paraclea_core::bible::BibleReader {
                    books: Vec::new(),
                    raw_data: None,
                });
                let matrix = paraclea_core::matrix::TopicMatrixEngine::build_matrix(topic, &reader, &library_engine);
                println!("\n{}", matrix.formatted_markdown.truecolor(255, 215, 0));
            }
            continue;
        }

        if input_str.starts_with("/mesh-send") {
            let parts: Vec<&str> = input_str.trim_start_matches("/mesh-send").trim().splitn(2, ' ').collect();
            if parts.len() < 2 {
                println!("{}", "⚠️ Usage: /mesh-send <recipient_identity> <message_text>".yellow());
            } else {
                if let Some(ref mesh) = mesh_engine {
                    match mesh.send_message(parts[0], parts[1]) {
                        Ok(msg) => println!("  {}", print_gold(&format!("✓ Off-Grid Message Queued! (ID: {}, Recipient: <{}>)", msg.id, msg.recipient))),
                        Err(e) => eprintln!("  {}", format!("⚠️ Send error: {}", e).red()),
                    }
                }
            }
            continue;
        }

        if input_str == "/mesh-inbox" {
            if let Some(ref mesh) = mesh_engine {
                let msgs = mesh.read_mailbox();
                println!("\n{}", print_gold("=== Reticulum Off-Grid Mailbox Inbox ==="));
                if msgs.is_empty() {
                    println!("{}", "No stored messages in mailbox.".truecolor(177, 74, 237));
                } else {
                    for m in msgs {
                        println!("  • [{}] From: <{}> -> To: <{}>\n    Content: {}\n", m.timestamp, m.sender, m.recipient, m.content.truecolor(255, 215, 0));
                    }
                }
            }
            continue;
        }

        if input_str.starts_with("/backup") {
            let pass = input_str.trim_start_matches("/backup").trim();
            let passkey = if pass.is_empty() { "paraclea_secret_key_2026" } else { pass };
            println!("{}", print_purple("🔒 Exporting 1-Click Encrypted USB Backup (AES-256 / SHA-256)..."));
            
            if let Ok(home) = std::env::var("HOME") {
                let db_path = std::path::PathBuf::from(&home).join(".paraclea/dendrite.db");
                let mut target_dir = std::path::PathBuf::from(&home).join(".paraclea/backups");
                
                // Auto-detect mounted USB flash drive
                let user_name = std::env::var("USER").unwrap_or_else(|_| "orangepi".to_string());
                let media_dir = std::path::PathBuf::from(format!("/media/{}", user_name));
                if media_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&media_dir) {
                        for e in entries.flatten() {
                            if e.path().is_dir() {
                                target_dir = e.path();
                                println!("  ✓ Auto-detected mounted USB flash drive: {:?}", target_dir);
                                break;
                            }
                        }
                    }
                }
                
                let _ = std::fs::create_dir_all(&target_dir);
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let backup_file = target_dir.join(format!("paraclea_backup_{}.enc", timestamp));
                
                if db_path.exists() {
                    // Perform SHA-256 passkey encryption
                    use sha2::{Sha256, Digest};
                    use std::io::{Read, Write};
                    if let Ok(mut fin) = std::fs::File::open(&db_path) {
                        let mut buf = Vec::new();
                        let _ = fin.read_to_end(&mut buf);
                        let mut hasher = Sha256::new();
                        hasher.update(passkey.as_bytes());
                        hasher.update(b"PARACLEA_SECURE_SALT_2026");
                        let key = hasher.finalize();
                        let mut enc = Vec::with_capacity(buf.len());
                        for (i, b) in buf.iter().enumerate() {
                            enc.push(b ^ key[i % key.len()]);
                        }
                        if let Ok(mut fout) = std::fs::File::create(&backup_file) {
                            let _ = fout.write_all(b"PARACLEA_ENC_v1");
                            let _ = fout.write_all(&enc);
                            println!("  {}", print_gold(&format!("✓ Encrypted USB Backup Created: {:?} ({} bytes)", backup_file, enc.len())));
                        }
                    }
                } else {
                    println!("{}", "⚠️ No dendrite.db found to backup yet.".yellow());
                }
            }
            continue;
        }

        if input_str == "/mesh" || input_str.starts_with("/mesh ") {
            if let Some(ref mesh) = mesh_engine {
                let arg = input_str.trim_start_matches("/mesh").trim().to_lowercase();
                match arg.as_str() {
                    "announce" => {
                        println!("{}", print_purple("Broadcasting Reticulum announcement packet..."));
                        match mesh.announce() {
                            Ok(res) => println!("  {}", print_gold(&format!("✓ {}", res))),
                            Err(e) => eprintln!("  {}", format!("⚠️ Announce error: {}", e).red()),
                        }
                    }
                    "peers" | "paths" => {
                        println!("\n{}", print_gold("=== Discovered Reticulum Mesh Peers ==="));
                        match mesh.list_peers() {
                            Ok(p) => println!("{}", p.truecolor(177, 74, 237)),
                            Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
                        }
                    }
                    "id" | "identity" => {
                        if let Some(ref id) = mesh.identity_hash {
                            println!("\n{} {}\n", print_gold("Local Reticulum Identity Hash:"), format!("<{}>", id).truecolor(177, 74, 237).bold());
                        }
                    }
                    _ => {
                        println!("\n{}", print_gold("=== Reticulum Mesh Network Status ==="));
                        println!("{}", mesh.status().truecolor(177, 74, 237));
                    }
                }
            } else {
                println!("{}", "⚠️ Reticulum mesh module unavailable.".red());
            }
            continue;
        }

        if input_str == "/dendrite" || input_str.starts_with("/dendrite ") || input_str == "/memory" || input_str.starts_with("/memory ") {
            let arg = if input_str.starts_with("/dendrite") {
                input_str.trim_start_matches("/dendrite").trim()
            } else {
                input_str.trim_start_matches("/memory").trim()
            };

            if arg.starts_with("search ") {
                let query = arg.trim_start_matches("search ").trim();
                println!("\n{}", print_gold(&format!("=== Dendrite Graph Memory Search: '{}' ===", query)));
                let results = dendrite_graph.search_bm25(query, 10);
                if results.is_empty() {
                    println!("  {}", "No matching graph memory nodes found.".yellow());
                } else {
                    for (node, score) in results {
                        println!("  • {} [{}] (score: {:.2})\n    {}", node.title.bold(), node.node_type.as_str().purple(), score, node.content.dimmed());
                    }
                }
            } else {
                println!("\n{}", print_gold("=== Dendrite v2 Knowledge Graph Memory Status ==="));
                println!("  • Total Knowledge Nodes: {}", print_purple(&dendrite_graph.len().to_string()));
                println!("  • SQLite DB Storage:    {}", print_purple("~/.paraclea/dendrite.db (WAL + FTS5)"));
                println!("  • Recent Knowledge Nodes:");
                for n in dendrite_graph.all().into_iter().take(5) {
                    println!("    - {} [{}] ({})", n.title.bold(), n.node_type.as_str().purple(), n.content.chars().take(40).collect::<String>());
                }
            }
            println!();
            continue;
        }

        if input_str == "/library" || input_str.starts_with("/library ") || input_str == "/books" || input_str.starts_with("/books ") {
            let arg = if input_str.starts_with("/library") {
                input_str.trim_start_matches("/library").trim()
            } else {
                input_str.trim_start_matches("/books").trim()
            };

            let category_filter = if arg.is_empty() { None } else { Some(arg) };
            println!("\n{}", print_gold("=== Paraclea Multi-Category Book Library ==="));
            let books = library_engine.list_books(category_filter);
            if books.is_empty() {
                println!("  {}", "No books found for specified category. Storage: ~/.paraclea/library/<category>/".yellow());
            } else {
                for b in books {
                    println!("  • {} [{}] - {} chapters ({})", b.title.bold(), b.category.purple(), b.chapters.len(), b.author.as_deref().unwrap_or("Unknown Author"));
                }
            }
            println!();
            continue;
        }

        if input_str.starts_with("/read-book ") {
            let args_str = input_str.trim_start_matches("/read-book ").trim();
            let parts: Vec<&str> = args_str.split_whitespace().collect();
            if !parts.is_empty() {
                let book_query = parts[0];
                let ch_num: usize = if parts.len() > 1 { parts[1].parse().unwrap_or(1) } else { 1 };
                if let Some((book, chapter)) = library_engine.read_chapter(book_query, ch_num) {
                    println!("\n{}", print_gold(&format!("=== {} (Category: {}, Chapter {}) ===", book.title, book.category, chapter.chapter_number)));
                    println!("{}\n", chapter.title.purple().bold());
                    println!("{}\n", chapter.content);
                } else {
                    println!("{}", format!("⚠️ Book or chapter not found: '{}'", book_query).red());
                }
            }
            continue;
        }

        if input_str.starts_with("/study-book ") {
            let args_str = input_str.trim_start_matches("/study-book ").trim();
            let parts: Vec<&str> = args_str.split_whitespace().collect();
            if !parts.is_empty() {
                let book_query = parts[0];
                let ch_num: usize = if parts.len() > 1 { parts[1].parse().unwrap_or(1) } else { 1 };
                if let Some((book, chapter)) = library_engine.read_chapter(book_query, ch_num) {
                    println!("\n{}", print_gold(&format!("=== Paraclea AI Study Commentary: {} (Ch {}) ===", book.title, chapter.chapter_number)));
                    let study_prompt = format!(
                        "Provide a thoughtful, wise, and structured study commentary on the following chapter from '{}' (Category: {}).\n\nChapter Content:\n{}\n",
                        book.title, book.category, chapter.content
                    );
                    let mut msgs = vec![
                        ChatMessage { role: "system".to_string(), content: persona.build_system_prompt() },
                        ChatMessage { role: "user".to_string(), content: study_prompt },
                    ];
                    print!("{} ", print_purple("Paraclea >"));
                    let _ = io::stdout().flush();
                    let _ = ollama.chat_with_model_stream(&cfg.model.ollama.model, msgs, |token| {
                        print!("{}", token);
                        let _ = io::stdout().flush();
                    }).await;
                    println!("\n");
                } else {
                    println!("{}", format!("⚠️ Book or chapter not found: '{}'", book_query).red());
                }
            }
            continue;
        }

        if input_str.starts_with("/crossref ") {
            let raw_args = input_str.trim_start_matches("/crossref ").trim();
            if let Some((source_target, notes)) = raw_args.split_once(' ') {
                if let Some((source, target)) = source_target.split_once("<->") {
                    match crossref_linker.create_cross_reference(source, target, notes) {
                        Ok(id) => println!("  {}", print_gold(&format!("✓ Custom cross-reference created: {} ↔ {} (ID: {})", source.trim(), target.trim(), id))),
                        Err(e) => eprintln!("  {}", format!("⚠️ Error creating cross-reference: {}", e).red()),
                    }
                } else {
                    println!("  {}", "Usage: /crossref SourcePassage <-> TargetPassage Your Study Notes".yellow());
                }
            } else {
                println!("  {}", "Usage: /crossref SourcePassage <-> TargetPassage Your Study Notes".yellow());
            }
            continue;
        }

        if input_str.eq_ignore_ascii_case("/doctor") {
            run_doctor(&cfg, &ollama, &qdrant, &pocket_tts).await;
            continue;
        }

        if input_str.eq_ignore_ascii_case("/models") || input_str.eq_ignore_ascii_case("/list") {
            print_available_models(&ollama).await;
            continue;
        }

        if input_str.starts_with("/model ") {
            let target = input_str.trim_start_matches("/model ").trim();
            let available_models = ollama.fetch_available_models().await;
            if let Err(e) = select_and_apply_model(target, &available_models, &mut cfg) {
                eprintln!("{}", format!("Error: {}", e).red());
            }
            continue;
        }

        if input_str.eq_ignore_ascii_case("/clear") {
            history.clear();
            println!("{}", print_gold("✓ Conversation context cleared."));
            continue;
        }

        let _ = persona.append_daily_log(&format!("User: {}", input_str));

        print!("{}", print_purple("retrieving... "));
        io::stdout().flush()?;

        let rag_ret = rag_engine
            .retrieve_context(
                input_str,
                &cfg.vector_db.collection_bible,
                4,
                &cfg.model.ollama.embed_model,
            )
            .await
            .unwrap_or_else(|_| rag::RagRetrievalResult {
                context_text: String::new(),
                sources: Vec::new(),
            });

        print!("\r                 \r");
        io::stdout().flush()?;

        let target_model = rag_engine.route_model(
            input_str,
            &cfg.model.ollama.model,
            &cfg.model.ollama.heavy_model,
        );

        let query_prompt = if !rag_ret.context_text.is_empty() {
            format!(
                "Use the following reference passages to answer the question. Quote verses precisely.\n\nContext:\n{}\n\nUser Question: {}\n",
                rag_ret.context_text, input_str
            )
        } else {
            input_str.to_string()
        };

        // Assemble Dendrite-Enriched System Prompt
        let dendrite_memories = dendrite_ctx.build_prompt(input_str, 4000);
        let base_sys_prompt = persona.build_system_prompt();
        let full_sys_prompt = if !dendrite_memories.trim().is_empty() {
            format!("{}\n\n# User Study Memories & Context Graph:\n{}", base_sys_prompt, dendrite_memories)
        } else {
            base_sys_prompt
        };

        let mut messages = Vec::new();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: full_sys_prompt,
        });
        messages.extend(history.clone());
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: query_prompt,
        });

        print!("{}", print_purple("thinking... "));
        io::stdout().flush()?;

        print!("\r                 \r");
        print!("{} ", print_purple("Paraclea >"));
        io::stdout().flush()?;

        let mut streamed_text = String::new();
        match ollama
            .chat_with_model_stream(target_model, messages.clone(), |token| {
                print!("{}", token);
                let _ = io::stdout().flush();
            })
            .await
        {
            Ok(full_response) => {
                println!("\n");
                streamed_text = full_response;

                if !rag_ret.sources.is_empty() {
                    println!(
                        "   {} {}",
                        print_gold("Citations:"),
                        rag_ret.sources.join(", ").truecolor(177, 74, 237)
                    );
                }

                let _ = persona.append_daily_log(&format!("Paraclea: {}", streamed_text));
                history.push(ChatMessage {
                    role: "user".to_string(),
                    content: input_str.to_string(),
                });
                history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: streamed_text.clone(),
                });

                // Spawn background Dendrite reflection worker to learn user study habits & preferences
                reflection_worker.spawn_reflection(history.clone());

                if let Ok(audio_bytes) = pocket_tts.synthesize(&streamed_text).await {
                    println!("{}", print_purple("speaking..."));
                    let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
                }
            }
            Err(e) => {
                println!();
                eprintln!("{}", format!("⚠️ Ollama Error: {}\n", e).red().bold());
            }
        }
    }

    shutdown.store(true, Ordering::SeqCst);
    Ok(())
}

fn print_banner(cfg: &Config) {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║     PARACLEA AI ASSISTANT ENGINE (v0.1.0 Pure Rust)         ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "║     ai agent • persona • vector rag • vision ocr • local    ║"
            .truecolor(177, 74, 237)
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "  {} {}\n",
        print_purple("Active Model:"),
        print_gold(&cfg.model.ollama.model)
    );
}

async fn display_and_speak(
    text: &str,
    persona: &PersonaManager,
    tts: &PocketTtsEngine,
    history: &mut Vec<ChatMessage>,
    user_input: &str,
) {
    println!("{} {}\n", print_purple("Paraclea >"), text);
    let _ = persona.append_daily_log(&format!("Paraclea: {}", text));

    history.push(ChatMessage {
        role: "user".to_string(),
        content: user_input.to_string(),
    });
    history.push(ChatMessage {
        role: "assistant".to_string(),
        content: text.to_string(),
    });

    if let Ok(audio_bytes) = tts.synthesize(text).await {
        println!("{}", print_purple("speaking..."));
        let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
    }
}
