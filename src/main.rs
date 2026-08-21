//! Paraclea — AI Companion Assistant & Self-Developing RAG Engine (Rust)
//!
//! Visual CLI with Gold & Purple styling, Ollama LLM & Vision OCR integration, Qdrant vector DB RAG,
//! format auto-detection, Pocket TTS speech synthesis, and updated Proverbs 31 Helper persona.

mod audio;
mod bible;
mod config;
mod detect;
mod heartbeat;
mod ingest;
mod ollama;
mod persona;
mod pocket_tts;
mod qdrant;
mod rag;
mod tools;

use anyhow::Result;
use audio::AudioPlayer;
use bible::BibleReader;
use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use detect::FileType;
use heartbeat::HeartbeatLoop;
use ingest::{ingest_file, BibleIngestor, BookIngestor};
use ollama::{ChatMessage, ModelEntry, OllamaClient};
use persona::PersonaManager;
use pocket_tts::PocketTtsEngine;
use qdrant::QdrantClient;
use rag::RagEngine;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tools::ToolExecutor;

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
        "║     PARACLEA AI ASSISTANT — SYSTEM DOCTOR DIAGNOSTICS        ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    // 1. Ollama Check
    let mut ollama_ok = ollama.health_check().await.unwrap_or(false);
    if !ollama_ok {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("nohup ollama serve > /tmp/ollama.log 2>&1 &")
            .spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        ollama_ok = ollama.health_check().await.unwrap_or(false);
    }
    println!(
        "  🔍 Checking Ollama Server ({}) ... {}",
        cfg.model.ollama.url,
        if ollama_ok { "ONLINE".green().bold() } else { "OFFLINE".red().bold() }
    );

    // 2. Qdrant Check
    let mut qdrant_ok = qdrant.health_check().await;
    if !qdrant_ok {
        if let Ok(home) = std::env::var("HOME") {
            let qdrant_bin = std::path::PathBuf::from(home).join(".paraclea/bin/qdrant");
            if qdrant_bin.exists() {
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("nohup {:?} > /tmp/qdrant_daemon.log 2>&1 &", qdrant_bin))
                    .spawn();
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                qdrant_ok = qdrant.health_check().await;
            }
        }
    }
    println!(
        "  🔍 Checking Qdrant Vector DB ({}) ... {}",
        cfg.vector_db.qdrant_url,
        if qdrant_ok { "ONLINE".green().bold() } else { "OFFLINE (Standby)".yellow().bold() }
    );

    // 3. Pocket TTS Check
    let tts_ok = tts.health_check().await;
    println!(
        "  🔍 Checking Pocket TTS Engine ({}) ... {}",
        cfg.voice.pocket_tts_url,
        if tts_ok { "ONLINE".green().bold() } else { "CLI FALLBACK".yellow().bold() }
    );

    // 4. Model Registry Diagnostics
    println!("\n{}", print_purple("  🧠 Model Registry Status:"));
    let reg = ollama.discover_models().await;

    println!(
        "     • Embedding Model:     {} ",
        reg.embed.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0)
    );
    println!(
        "     • Default Chat Model:  {} ",
        reg.chat.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0)
    );
    println!(
        "     • Heavy Reasoning:     {} ",
        reg.heavy.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0)
    );
    println!(
        "     • Document Vision OCR: {} ",
        reg.ocr.as_deref().unwrap_or("MISSING").truecolor(255, 215, 0)
    );

    let missing = ollama.check_missing(&reg);
    if !missing.is_empty() {
        println!("\n{}", "  ⚠️  Recommended models missing:".yellow().bold());
        for item in missing {
            println!("     - {}", item);
        }
    } else {
        println!("\n{}", "  🎉 All recommended model categories present and operational!".green().bold());
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
];

const DYNAMIC_FAREWELLS: &[&str] = &[
    "I really enjoyed our conversation! Go with strength and wisdom—talk later!",
    "Until next time! May your work be fruitful and your heart at peace.",
    "It was a pleasure helping you today. Take care, and I will be right here whenever you return!",
    "Farewell for now! May your path be clear and your effort blessed. Talk soon!",
    "Safe travels on your work today! I enjoyed our chat—reach out whenever you need me again.",
    "Goodbye for now, my friend! Keep striving with courage and grace. Talk later!",
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

async fn handle_bible_menu(cfg: &mut Config, config_path: &Path) -> Result<()> {
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

    print!("\n{} ", print_gold("Select language [1-5] >"));
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;

    let lang_id = input.trim().parse::<usize>().unwrap_or(1);
    let selected_lang = languages.iter().find(|l| l.id == lang_id).cloned().unwrap_or(languages[0].clone());

    let translations = BibleReader::list_translations_for_lang(&selected_lang.code);
    println!("\n  {}", print_gold(&format!("Available translations for {}:", selected_lang.name)));
    for trans in &translations {
        let easy_tag = if trans.is_easy { " (Recommended / Easy)" } else { "" };
        println!("    [{}] {}{}", trans.id, trans.name, easy_tag);
    }
    println!("    [{}] Not Sure / Recommended (Defaults to Easy Version)", translations.len() + 1);

    print!("\n{} ", print_gold("Select translation >"));
    io::stdout().flush()?;

    let mut trans_input = String::new();
    stdin.read_line(&mut trans_input)?;

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

async fn handle_read_cmd(reader: &BibleReader, history: &mut Vec<ChatMessage>) -> Result<()> {
    println!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════╗"
            .truecolor(177, 74, 237)
            .bold()
    );
    println!(
        "{}",
        "║                SCRIPTURE READER & STUDY NAVIGATOR            ║"
            .truecolor(255, 215, 0)
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝"
            .truecolor(177, 74, 237)
            .bold()
    );

    let stdin = io::stdin();
    print!("{} ", print_gold("Enter Book Name (e.g. Genesis, Proverbs, John) >"));
    io::stdout().flush()?;

    let mut book_input = String::new();
    stdin.read_line(&mut book_input)?;
    let book_name = book_input.trim();

    let book_meta = match reader.find_book(book_name) {
        Some(b) => b,
        None => {
            println!("{}", format!("⚠️ Book '{}' not found in Bible database.", book_name).red());
            return Ok(());
        }
    };

    println!(
        "  📖 {}",
        print_gold(&format!("'{}' has {} chapters.", book_meta.name, book_meta.total_chapters))
    );

    print!("{} ", print_gold(&format!("Select Chapter (1-{}) >", book_meta.total_chapters)));
    io::stdout().flush()?;

    let mut chap_input = String::new();
    stdin.read_line(&mut chap_input)?;
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

    print!("{} ", print_gold(&format!("Select Verse (1-{}, or 'all' for full chapter) >", verse_count)));
    io::stdout().flush()?;

    let mut verse_input = String::new();
    stdin.read_line(&mut verse_input)?;
    let v_str = verse_input.trim().to_lowercase();

    if v_str == "all" || v_str.is_empty() {
        if let Some(verses) = reader.read_chapter(&book_meta.name, chapter_num) {
            println!(
                "\n{}",
                format!("=== {} Chapter {} ===", book_meta.name, chapter_num).truecolor(255, 215, 0).bold()
            );
            let mut full_passage = String::new();
            for (v_idx, text) in verses {
                let line = format!("[{}] {}\n", v_idx, text);
                print!("{}", line.truecolor(177, 74, 237));
                full_passage.push_str(&line);
            }
            history.push(ChatMessage {
                role: "system".to_string(),
                content: format!("User is reading {} Chapter {}:\n{}", book_meta.name, chapter_num, full_passage),
            });
            println!("\n{}", print_gold("✓ Passage loaded. Ask Paraclea any questions about this chapter!"));
        }
    } else if let Ok(verse_num) = v_str.parse::<usize>() {
        if verse_num >= 1 && verse_num <= verse_count {
            if let Some(text) = reader.read_verse(&book_meta.name, chapter_num, verse_num) {
                let citation = format!("{} {}:{}", book_meta.name, chapter_num, verse_num);
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

async fn handle_compare_cmd(
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

    let stdin = io::stdin();
    print!("{} ", print_gold("Enter Book Name (e.g. John, Genesis, Proverbs) >"));
    io::stdout().flush()?;

    let mut book_input = String::new();
    stdin.read_line(&mut book_input)?;
    let book_name = book_input.trim();

    let book_meta = match reader.find_book(book_name) {
        Some(b) => b,
        None => {
            println!("{}", format!("⚠️ Book '{}' not found.", book_name).red());
            return Ok(());
        }
    };

    print!("{} ", print_gold(&format!("Select Chapter (1-{}) >", book_meta.total_chapters)));
    io::stdout().flush()?;

    let mut chap_input = String::new();
    stdin.read_line(&mut chap_input)?;
    let chapter_num: usize = chap_input.trim().parse().unwrap_or(1);

    let verse_count = reader.get_verse_count(&book_meta.name, chapter_num).unwrap_or(1);
    print!("{} ", print_gold(&format!("Select Verse (1-{}) >", verse_count)));
    io::stdout().flush()?;

    let mut verse_input = String::new();
    stdin.read_line(&mut verse_input)?;
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
    } else {
        "persona".to_string()
    };
    let persona = PersonaManager::new(&persona_dir)?;

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
            let qdrant_bin = std::path::PathBuf::from(home).join(".paraclea/bin/qdrant");
            if qdrant_bin.exists() {
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("nohup {:?} > /tmp/qdrant_daemon.log 2>&1 &", qdrant_bin))
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

    // 7. Tool & RAG Executors & Bible Reader
    let tool_executor = ToolExecutor::new(persona.clone());
    let rag_engine = RagEngine::new(&ollama, &qdrant);
    let config_path = PathBuf::from("config.yaml");
    let bible_reader = BibleReader::load_auto().ok();

    // 8. Interactive REPL Shell Loop
    let mut history: Vec<ChatMessage> = Vec::new();
    println!(
        "\n{} {}\n",
        print_purple("Paraclea >"),
        print_gold(get_random_greeting())
    );
    println!(
        "{}\n",
        print_gold("✨ Paraclea is ready! Type your message (or '/help' for options, '/bye' to quit).")
    );

    let stdin = io::stdin();
    loop {
        print!("{} ", print_gold("You >"));
        io::stdout().flush()?;

        let mut user_input = String::new();
        if stdin.read_line(&mut user_input).is_err() || user_input.trim().is_empty() {
            continue;
        }

        let input_str = user_input.trim();

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
            let _ = handle_bible_menu(&mut cfg, &config_path).await;
            continue;
        }

        if input_str.eq_ignore_ascii_case("/read") {
            if let Some(ref reader) = bible_reader {
                let _ = handle_read_cmd(reader, &mut history).await;
            } else {
                println!("{}", "⚠️ Bible database not loaded.".red());
            }
            continue;
        }

        if input_str.eq_ignore_ascii_case("/compare") {
            if let Some(ref reader) = bible_reader {
                let _ = handle_compare_cmd(reader, &ollama, &mut history).await;
            } else {
                println!("{}", "⚠️ Bible database not loaded.".red());
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

        let mut messages = Vec::new();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: persona.build_system_prompt(),
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
