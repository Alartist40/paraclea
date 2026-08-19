//! Paraclea — Cute AI Companion Assistant & Self-Developing Engine (Rust)
//!
//! Visual CLI with Gold & Purple styling, Ollama LLM integration, Pocket TTS speech synthesis,
//! self-updating persona management, and CLI subcommand routing (`paraclea list`, `paraclea run <model>`).

mod audio;
mod config;
mod heartbeat;
mod ollama;
mod persona;
mod pocket_tts;
mod tools;

use anyhow::Result;
use audio::AudioPlayer;
use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use heartbeat::HeartbeatLoop;
use ollama::{ChatMessage, ModelEntry, OllamaClient};
use persona::PersonaManager;
use pocket_tts::PocketTtsEngine;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tools::ToolExecutor;
use tracing::error;

#[derive(Parser, Debug)]
#[command(name = "paraclea")]
#[command(author = "Xander <https://github.com/Alartist40>")]
#[command(version = "0.1.0")]
#[command(
    about = "Paraclea — Cute AI Companion & Self-Developing Assistant Engine in Rust",
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
    /// Run Paraclea with a specific model by number or name (e.g. 'paraclea run 1' or 'paraclea run llama3.2')
    Run {
        /// Model name or list number
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logging initialization
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "warn,paraclea=info".to_string()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config_path = Config::find_or_default_config_path();
    let mut cfg = Config::load(&config_path)?;

    let temp_ollama = OllamaClient::new(&cfg.model.ollama.url, &cfg.model.ollama.model)?;

    match &cli.command {
        Some(Commands::List) => {
            print_available_models(&temp_ollama).await;
            return Ok(());
        }
        Some(Commands::Run { model }) => {
            let available = temp_ollama.fetch_available_models().await;
            if let Some(target) = model {
                if let Err(e) = select_and_apply_model(target, &available, &mut cfg) {
                    eprintln!("{}", format!("Error: {}", e).red());
                    print_available_models(&temp_ollama).await;
                    return Ok(());
                }
                let _ = cfg.save(&config_path);
            } else {
                print_available_models(&temp_ollama).await;
                return Ok(());
            }
        }
        None => {
            // Default run mode — verify model availability
            let available = temp_ollama.fetch_available_models().await;
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

    print!("{}", print_purple("🔍 Checking Ollama server... "));
    io::stdout().flush()?;
    match ollama.health_check().await {
        Ok(true) => println!("{}", print_gold("ONLINE (Ollama ready)")),
        _ => println!("{}", "OFFLINE (Make sure 'ollama serve' is running)".red().bold()),
    }

    // 3. Initialize Pocket TTS Client
    let pocket_tts = PocketTtsEngine::new(
        &cfg.voice.pocket_tts_url,
        &cfg.voice.pocket_tts_voice,
        Some(&cfg.voice.pocket_tts_cli),
    )?;

    print!("{}", print_purple("🔍 Checking Pocket TTS engine... "));
    io::stdout().flush()?;
    if pocket_tts.health_check().await {
        println!("{}", print_gold("ONLINE (FastAPI daemon active)"));
    } else {
        println!(
            "{}",
            "CLI FALLBACK (Pocket TTS daemon offline — using local CLI runner)".yellow()
        );
    }

    // 4. Launch Heartbeat Background Self-Maintenance Loop
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

    // 5. Tool Executor
    let tool_executor = ToolExecutor::new(persona.clone());

    // 6. Interactive REPL Shell Loop
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
                print_gold("Goodbye master! See you soon! (*^▽^*)")
            );
            break;
        }

        // Log user turn to daily interaction log
        let _ = persona.append_daily_log(&format!("User: {}", input_str));

        // Build current system prompt & message context
        let mut messages = Vec::new();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: persona.build_system_prompt(),
        });
        messages.extend(history.clone());
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: input_str.to_string(),
        });

        print!("{}", print_purple("Paraclea is thinking... "));
        io::stdout().flush()?;

        match ollama.chat(messages.clone()).await {
            Ok(response_text) => {
                print!("\r                         \r");
                io::stdout().flush()?;

                // Check for tool execution request
                if let Some(tool_call) = tool_executor.parse_tool_call(&response_text) {
                    println!(
                        "{}",
                        format!("🛠️  [Tool Invoked]: {}", tool_call.tool)
                            .truecolor(255, 215, 0)
                            .bold()
                    );
                    match tool_executor.execute(&tool_call) {
                        Ok(tool_result) => {
                            println!(
                                "{}",
                                format!("   Output: {}", tool_result).truecolor(177, 74, 237)
                            );
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

                            if let Ok(final_text) = ollama.chat(tool_messages).await {
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
                        Err(e) => {
                            error!("Tool execution error: {}", e);
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
                print!("\r                         \r");
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
        "║     cute avatar • omnibot persona • pocket tts • zero cloud ║"
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
        print_purple("Active Ollama Model:"),
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

    // Synthesize & play speech audio asynchronously
    if let Ok(audio_bytes) = tts.synthesize(text).await {
        let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
    }
}
