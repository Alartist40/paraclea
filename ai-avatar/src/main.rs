//! Paraclea — Cute AI Companion Assistant & Self-Developing Engine (Rust)
//!
//! Pipeline:  User Input → Persona Assembly → Ollama LLM → Tool Executor → Pocket TTS → Audio Playback
//! Background: Async Heartbeat Loop for continuous reflection and memory consolidation.

mod audio;
mod heartbeat;
mod ollama;
mod persona;
mod pocket_tts;
mod tools;

use anyhow::Result;
use audio::AudioPlayer;
use heartbeat::HeartbeatLoop;
use ollama::{ChatMessage, OllamaClient};
use persona::PersonaManager;
use pocket_tts::PocketTtsEngine;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tools::ToolExecutor;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,paraclea=info".to_string()),
        )
        .with_target(false)
        .init();

    println!("====================================================");
    println!("🌸 PARACLEA — Cute AI Assistant & Self-Developing Companion");
    println!("====================================================\n");

    // 1. Initialize Persona Manager
    let persona_dir = std::env::var("PARACLEA_PERSONA_DIR").unwrap_or_else(|_| "persona".to_string());
    let persona = PersonaManager::new(&persona_dir)?;
    info!("Persona files loaded from './{}'", persona_dir);

    // 2. Initialize Ollama Client
    let ollama_endpoint = std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let default_model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    let ollama = OllamaClient::new(&ollama_endpoint, &default_model)?;

    print!("🔍 Checking Ollama server at {}... ", ollama_endpoint);
    io::stdout().flush()?;
    match ollama.health_check().await {
        Ok(true) => {
            println!("OK!");
            if let Ok(models) = ollama.list_models().await {
                info!("Available Ollama models: {:?}", models);
            }
        }
        _ => {
            println!("WARNING!");
            warn!("Ollama server is not responding at {}. Make sure 'ollama serve' is running.", ollama_endpoint);
        }
    }

    // 3. Initialize Pocket TTS Client
    let tts_endpoint = std::env::var("POCKET_TTS_ENDPOINT").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let tts_voice = std::env::var("POCKET_TTS_VOICE").unwrap_or_else(|_| "alba".to_string());
    let cli_path = std::env::var("POCKET_TTS_CLI")
        .unwrap_or_else(|_| "/home/xander/Documents/reference/pocket-tts/.venv/bin/pocket-tts".to_string());

    let pocket_tts = PocketTtsEngine::new(&tts_endpoint, &tts_voice, Some(&cli_path))?;

    print!("🔍 Checking Pocket TTS service at {}... ", tts_endpoint);
    io::stdout().flush()?;
    if pocket_tts.health_check().await {
        println!("OK! (HTTP Server active)");
    } else {
        println!("INFO");
        info!("Pocket TTS HTTP server not active. CLI fallback ready at '{}'.", cli_path);
    }

    // 4. Start Background Heartbeat Loop
    let shutdown = Arc::new(AtomicBool::new(false));
    let heartbeat_interval_mins = std::env::var("HEARTBEAT_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);

    let heartbeat = HeartbeatLoop::new(heartbeat_interval_mins, persona.clone(), ollama.clone());
    let shutdown_hb = shutdown.clone();
    tokio::spawn(async move {
        heartbeat.run(shutdown_hb).await;
    });

    // 5. Initialize Tool Executor
    let tool_executor = ToolExecutor::new(persona.clone());

    // 6. Interactive Chat Loop
    let mut history: Vec<ChatMessage> = Vec::new();
    println!("\n✨ Paraclea is ready! Type your message (or 'exit' / 'quit' to stop).\n");

    let stdin = io::stdin();
    loop {
        print!("You > ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        if stdin.read_line(&mut user_input).is_err() || user_input.trim().is_empty() {
            continue;
        }

        let input_str = user_input.trim();
        if input_str.eq_ignore_ascii_case("exit") || input_str.eq_ignore_ascii_case("quit") {
            println!("\nParaclea > Goodbye master! See you soon! (*^▽^*)\n");
            break;
        }

        // Log user turn to daily interaction log
        let _ = persona.append_daily_log(&format!("User: {}", input_str));

        // Build current prompt messages
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

        print!("Paraclea is thinking... ");
        io::stdout().flush()?;

        match ollama.chat(messages.clone()).await {
            Ok(response_text) => {
                print!("\r                         \r"); // clear "thinking..." line
                io::stdout().flush()?;

                // Check if response contains tool execution request
                if let Some(tool_call) = tool_executor.parse_tool_call(&response_text) {
                    info!("🛠️ Paraclea invoked tool: {}", tool_call.tool);
                    match tool_executor.execute(&tool_call) {
                        Ok(tool_result) => {
                            info!("Tool output: {}", tool_result);
                            // Feed tool result back to LLM for final response
                            let mut tool_messages = messages.clone();
                            tool_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: response_text,
                            });
                            tool_messages.push(ChatMessage {
                                role: "user".to_string(),
                                content: format!("[TOOL RESULT for {}]: {}", tool_call.tool, tool_result),
                            });

                            if let Ok(final_text) = ollama.chat(tool_messages).await {
                                display_and_speak(&final_text, &persona, &pocket_tts, &mut history, input_str).await;
                            }
                        }
                        Err(e) => {
                            error!("Tool execution error: {}", e);
                            display_and_speak(&response_text, &persona, &pocket_tts, &mut history, input_str).await;
                        }
                    }
                } else {
                    display_and_speak(&response_text, &persona, &pocket_tts, &mut history, input_str).await;
                }
            }
            Err(e) => {
                print!("\r                         \r");
                io::stdout().flush()?;
                println!("⚠️ Error reaching Ollama: {}\n", e);
            }
        }
    }

    shutdown.store(true, Ordering::SeqCst);
    Ok(())
}

async fn display_and_speak(
    text: &str,
    persona: &PersonaManager,
    tts: &PocketTtsEngine,
    history: &mut Vec<ChatMessage>,
    user_input: &str,
) {
    println!("Paraclea > {}\n", text);
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
        let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
    }
}
