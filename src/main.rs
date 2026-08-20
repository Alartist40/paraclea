//! Paraclea — AI Companion Assistant & Self-Developing RAG Engine (Rust)
//!
//! Visual CLI with Gold & Purple styling, Ollama LLM integration, Qdrant vector database RAG,
//! Scripture & book-to-skill ingestion, Pocket TTS speech synthesis, and updated Proverbs 31 Helper persona.

mod audio;
mod config;
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
use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use heartbeat::HeartbeatLoop;
use ingest::{BibleIngestor, BookIngestor};
use ollama::{ChatMessage, ModelEntry, OllamaClient};
use persona::PersonaManager;
use pocket_tts::PocketTtsEngine;
use qdrant::QdrantClient;
use rag::RagEngine;
use std::io::{self, Write};
use std::path::Path;
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
            if !available.is_empty() {
                if !available.iter().any(|m| m.target == cfg.model.ollama.model) {
                    let first_model = &available[0];
                    cfg.model.ollama.model = first_model.target.clone();
                    let _ = cfg.save(&config_path);
                }
            }
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

async fn start_paraclea_repl(cfg: Config) -> Result<()> {
    print_banner(&cfg);

    // 1. Initialize Persona Manager
    let persona_dir = if Path::new(&cfg.persona.dir).exists() {
        cfg.persona.dir.clone()
    } else {
        "persona".to_string()
    };
    let persona = PersonaManager::new(&persona_dir)?;

    // 2. Initialize Ollama Client
    let ollama = OllamaClient::new(&cfg.model.ollama.url, &cfg.model.ollama.model)?;

    print!("{}", print_purple("🔍 Checking Ollama... "));
    io::stdout().flush()?;
    match ollama.health_check().await {
        Ok(true) => println!("{}", print_gold("ONLINE")),
        _ => println!("{}", "OFFLINE".red().bold()),
    }

    // 3. Initialize Qdrant Client
    let qdrant = QdrantClient::new(&cfg.vector_db.qdrant_url)?;
    print!("{}", print_purple("🔍 Checking Qdrant Vector DB... "));
    io::stdout().flush()?;
    if qdrant.health_check().await {
        println!("{}", print_gold("ONLINE"));
        qdrant.create_collection(&cfg.vector_db.collection_bible, 768).await.ok();
        qdrant.create_collection(&cfg.vector_db.collection_books, 768).await.ok();
    } else {
        println!("{}", "STANDBY (Run './qdrant' for vector search)".yellow());
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

    // 5. Launch Heartbeat Background Self-Maintenance Loop
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

    // 6. Tool & RAG Executors
    let tool_executor = ToolExecutor::new(persona.clone());
    let rag_engine = RagEngine::new(&ollama, &qdrant);

    // 7. Interactive REPL Shell Loop
    let mut history: Vec<ChatMessage> = Vec::new();
    println!(
        "\n{}\n",
        print_gold("✨ Paraclea is ready! Type your message (or 'exit' to quit).")
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
        if input_str.eq_ignore_ascii_case("exit") || input_str.eq_ignore_ascii_case("quit") {
            println!(
                "\n{} {}\n",
                print_purple("Paraclea >"),
                print_gold("Goodbye master! See you soon!")
            );
            break;
        }

        // Log user turn to daily interaction log
        let _ = persona.append_daily_log(&format!("User: {}", input_str));

        print!("{}", print_purple("retrieving... "));
        io::stdout().flush()?;

        // Perform RAG retrieval if Qdrant is online
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

        // Build current system prompt & message context
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

        match ollama.chat_with_model(target_model, messages.clone()).await {
            Ok(response_text) => {
                print!("\r                 \r");
                io::stdout().flush()?;

                // Check for tool execution request
                if let Some(tool_call) = tool_executor.parse_tool_call(&response_text) {
                    let verb = tool_executor.action_verb(&tool_call.tool);
                    print!("{} ", print_gold(verb));
                    io::stdout().flush()?;

                    match tool_executor.execute(&tool_call) {
                        Ok(tool_result) => {
                            print!("\r                 \r");
                            io::stdout().flush()?;

                            let mut tool_messages = messages.clone();
                            tool_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: response_text,
                            });
                            tool_messages.push(ChatMessage {
                                role: "user".to_string(),
                                content: format!(
                                    "[TOOL RESULT for {}]: {}",
                                    tool_call.tool, tool_result
                                ),
                            });

                            if let Ok(final_text) = ollama.chat_with_model(target_model, tool_messages).await {
                                display_and_speak(
                                    &final_text,
                                    &persona,
                                    &pocket_tts,
                                    &mut history,
                                    input_str,
                                )
                                .await;
                            }
                        }
                        Err(_e) => {
                            print!("\r                 \r");
                            io::stdout().flush()?;
                            display_and_speak(
                                &response_text,
                                &persona,
                                &pocket_tts,
                                &mut history,
                                input_str,
                            )
                            .await;
                        }
                    }
                } else {
                    display_and_speak(
                        &response_text,
                        &persona,
                        &pocket_tts,
                        &mut history,
                        input_str,
                    )
                    .await;
                }
            }
            Err(e) => {
                print!("\r                 \r");
                io::stdout().flush()?;
                println!("{}", format!("⚠️ Ollama Error: {}\n", e).red().bold());
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
        "║     ai agent • persona • vector rag • local                 ║"
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

    // Synthesize & play speech audio
    if let Ok(audio_bytes) = tts.synthesize(text).await {
        println!("{}", print_purple("speaking..."));
        let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
    }
}
