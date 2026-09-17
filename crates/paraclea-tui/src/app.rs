//! Paraclea TUI Application State & Event Loop.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use paraclea_core::{
    audio::AudioPlayer,
    bible::BibleReader,
    config::Config,
    crossref::CrossReferenceLinker,
    dendrite::{Dendrite, DendriteStore},
    library::LibraryEngine,
    mesh::ReticulumEngine,
    ollama::{ChatMessage as OllamaChatMessage, OllamaClient},
    persona::PersonaManager,
    pocket_tts::PocketTtsEngine,
    qdrant::QdrantClient,
};

use crate::modals::{render_help_modal, render_input_modal, render_list_picker_modal};
use crate::theme::AppTheme;
use crate::views::{
    bible::{render_bible_view, BibleViewState},
    chat::{render_chat_view, ChatMessage},
    crossref::{render_crossref_view, CrossrefViewState},
    doctor::render_doctor_view,
    galaxy::{GalaxyState, GalaxyView},
    library::{render_library_view, LibraryViewState},
    mesh::{render_mesh_view, MeshViewState},
};

pub const COMMAND_PALETTE: &[(&str, &str)] = &[
    ("/help", "Show help and keyboard shortcuts"),
    ("/theme", "Cycle color themes (5 available)"),
    ("/language", "Pick Bible language (66 languages)"),
    ("/version", "Pick Bible translation version"),
    ("/bible", "Open Bible reader / interactive scripture"),
    ("/compare", "Compare translations side-by-side"),
    ("/library", "Browse offline book library"),
    ("/galaxy", "Open 3D celestial knowledge galaxy"),
    ("/memory", "Inspect Dendrite knowledge graph"),
    ("/mesh", "Reticulum mesh network status"),
    ("/model", "Switch active Ollama AI model"),
    ("/doctor", "System health diagnostics & doctor"),
    ("/backup", "1-Click AES-256-GCM encrypted USB backup"),
    ("/crossref", "Cross-reference lookup"),
    ("/clear", "Clear chat history"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Chat = 0,
    Bible = 1,
    Library = 2,
    Crossref = 3,
    Galaxy = 4,
    Mesh = 5,
    Doctor = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveFocus {
    Sidebar,
    MainViewport,
    PromptInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    CommandPalette,
    LanguagePicker,
    TranslationPicker,
    ModelPicker,
    BackupPrompt,
}

pub enum StreamEvent {
    Token(String),
    Done,
    Error(String),
}

pub struct App {
    pub cfg: Config,
    pub config_path: PathBuf,
    pub theme: AppTheme,
    pub active_tab: ActiveTab,
    pub active_focus: ActiveFocus,
    pub is_sidebar_open: bool,
    pub active_modal: ActiveModal,

    // Core Engines
    pub bible_reader: Option<BibleReader>,
    pub library_engine: LibraryEngine,
    pub dendrite_graph: Arc<Dendrite>,
    pub crossref_linker: CrossReferenceLinker,
    pub mesh_engine: Option<ReticulumEngine>,
    pub ollama: OllamaClient,
    pub qdrant: QdrantClient,
    pub persona: PersonaManager,
    pub pocket_tts: PocketTtsEngine,

    // System Health & Doctor state
    pub ollama_online: bool,
    pub qdrant_online: bool,
    pub bible_lang_count: usize,
    pub bible_version_count: usize,
    pub selected_language_code: Option<String>,
    pub selected_language_name: Option<String>,

    // Views State
    pub chat_history: Vec<ChatMessage>,
    pub streaming_text: String,
    pub is_streaming: bool,
    pub is_speaking: bool,
    pub chat_scroll: usize,

    pub bible_state: BibleViewState,
    pub bible_books: Vec<String>,
    pub bible_verses: Vec<(usize, String)>,
    pub bible_comparison: Vec<(String, Vec<(usize, String)>)>,

    pub library_state: LibraryViewState,
    pub library_categories: Vec<String>,
    pub library_books: Vec<String>,
    pub library_chapter_title: String,
    pub library_chapter_content: String,

    pub crossref_state: CrossrefViewState,
    pub mesh_state: MeshViewState,
    pub galaxy_state: GalaxyState,

    // Prompt & Modals
    pub input_buffer: String,
    pub input_history: Vec<String>,
    pub history_idx: Option<usize>,

    pub modal_filter: String,
    pub modal_items: Vec<String>,
    pub modal_selected_idx: usize,
    pub command_palette_items: Vec<(&'static str, &'static str)>,
    pub input_modal_buffer: String,

    pub backup_status: Option<String>,
    pub rx_stream: mpsc::UnboundedReceiver<StreamEvent>,
    pub tx_stream: mpsc::UnboundedSender<StreamEvent>,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let (tx_stream, rx_stream) = mpsc::unbounded_channel();
        let config_path = Config::find_or_default_config_path();

        let bible_reader = BibleReader::load_auto().ok();
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let library_dir = PathBuf::from(&home_dir).join(".paraclea/library");
        let library_engine = LibraryEngine::new(library_dir);

        let dendrite_db_path = PathBuf::from(&home_dir).join(".paraclea/dendrite.db");
        let dendrite_store = DendriteStore::open(&dendrite_db_path).ok().map(Arc::new);
        let dendrite_graph = Arc::new(Dendrite::new());
        if let Some(ref store) = dendrite_store {
            let _ = store.load_all(&dendrite_graph);
        }
        let crossref_linker = CrossReferenceLinker::new(Arc::clone(&dendrite_graph), dendrite_store);
        let mesh_engine = ReticulumEngine::new().ok();
        let ollama = OllamaClient::new(&cfg.model.ollama.url, &cfg.model.ollama.model)
            .unwrap_or_else(|_| OllamaClient::new("http://localhost:11434", "ministral-3:3b").unwrap());
        let qdrant = QdrantClient::new(&cfg.vector_db.qdrant_url)
            .unwrap_or_else(|_| QdrantClient::new("http://localhost:6333").unwrap());
        let persona = PersonaManager::new(&cfg.persona.dir)
            .unwrap_or_else(|_| PersonaManager::new("persona").unwrap());
        let pocket_tts = PocketTtsEngine::new(
            &cfg.voice.pocket_tts_url,
            &cfg.voice.pocket_tts_voice,
            Some(&cfg.voice.pocket_tts_cli),
        ).unwrap_or_else(|_| PocketTtsEngine::new("http://localhost:8000", "alba", None).unwrap());

        let bible_books: Vec<String> = if let Some(ref reader) = bible_reader {
            reader.books.iter().map(|b| b.name.clone()).collect()
        } else {
            vec!["Genesis".to_string(), "Exodus".to_string(), "Matthew".to_string(), "John".to_string(), "Revelation".to_string()]
        };

        let initial_verses = if let Some(ref reader) = bible_reader {
            reader.read_chapter("Genesis", 1).unwrap_or_default()
        } else {
            Vec::new()
        };

        let library_categories = library_engine.list_categories();
        let initial_books: Vec<String> = library_engine
            .list_books(library_categories.first().map(|s| s.as_str()))
            .iter()
            .map(|b| b.title.clone())
            .collect();

        let (initial_ch_title, initial_ch_content) = if let Some(first_b) = initial_books.first() {
            if let Some((_, ch)) = library_engine.read_chapter(first_b, 1) {
                (ch.title.clone(), ch.content.clone())
            } else {
                ("Chapter 1".to_string(), "No content available.".to_string())
            }
        } else {
            ("Chapter 1".to_string(), "No library books ingested yet.".to_string())
        };

        let languages = BibleReader::list_languages();
        let bible_lang_count = languages.len();
        let mut bible_version_count = 0;
        for lang in &languages {
            let trans = BibleReader::list_translations_for_lang(&lang.code);
            bible_version_count += trans.len();
        }
        if bible_version_count == 0 {
            bible_version_count = 1;
        }

        let fallback_reader = BibleReader::from_json_str("[]").unwrap();
        let galaxy_state = GalaxyState::new(
            bible_reader.as_ref().unwrap_or(&fallback_reader),
            &library_engine,
        );

        let initial_theme = AppTheme::from_name(&cfg.theme).unwrap_or(AppTheme::RoyalByzantium);

        Self {
            cfg,
            config_path,
            theme: initial_theme,
            active_tab: ActiveTab::Chat,
            active_focus: ActiveFocus::PromptInput,
            is_sidebar_open: true,
            active_modal: ActiveModal::None,

            bible_reader,
            library_engine,
            dendrite_graph,
            crossref_linker,
            mesh_engine,
            ollama,
            qdrant,
            persona,
            pocket_tts,

            ollama_online: false,
            qdrant_online: false,
            bible_lang_count,
            bible_version_count,
            selected_language_code: Some("eng".to_string()),
            selected_language_name: Some("English".to_string()),

            chat_history: Vec::new(),
            streaming_text: String::new(),
            is_streaming: false,
            is_speaking: false,
            chat_scroll: 0,

            bible_state: BibleViewState::default(),
            bible_books,
            bible_verses: initial_verses,
            bible_comparison: Vec::new(),

            library_state: LibraryViewState::default(),
            library_categories,
            library_books: initial_books,
            library_chapter_title: initial_ch_title,
            library_chapter_content: initial_ch_content,

            crossref_state: CrossrefViewState::default(),
            mesh_state: MeshViewState::default(),
            galaxy_state,

            input_buffer: String::new(),
            input_history: Vec::new(),
            history_idx: None,

            modal_filter: String::new(),
            modal_items: Vec::new(),
            modal_selected_idx: 0,
            command_palette_items: COMMAND_PALETTE.to_vec(),
            input_modal_buffer: String::new(),

            backup_status: None,
            rx_stream,
            tx_stream,
        }
    }

    pub async fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            // Process streaming events non-blocking
            while let Ok(event) = self.rx_stream.try_recv() {
                match event {
                    StreamEvent::Token(tok) => {
                        self.streaming_text.push_str(&tok);
                    }
                    StreamEvent::Done => {
                        self.is_streaming = false;
                        let now_str = chrono::Local::now().format("%H:%M").to_string();
                        self.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: self.streaming_text.clone(),
                            thinking: None,
                            timestamp: now_str,
                        });
                        let spoken_text = self.streaming_text.clone();
                        self.streaming_text.clear();

                        // Async TTS Voice Playback
                        let tts = self.pocket_tts.clone();
                        tokio::spawn(async move {
                            if let Ok(audio_bytes) = tts.synthesize(&spoken_text).await {
                                let _ = AudioPlayer::play_wav_bytes(&audio_bytes);
                            }
                        });
                    }
                    StreamEvent::Error(err) => {
                        self.is_streaming = false;
                        self.streaming_text.clear();
                        self.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: format!("⚠️ Ollama Error: {}", err),
                            thinking: None,
                            timestamp: chrono::Local::now().format("%H:%M").to_string(),
                        });
                    }
                }
            }

            // Update galaxy simulation clock only when active
            if self.active_tab == ActiveTab::Galaxy {
                self.galaxy_state.update(0.033);
            }

            // Draw 30 FPS Frame
            terminal.draw(|f| self.render(f))?;

            // Event Poll (33ms = ~30 FPS)
            if event::poll(Duration::from_millis(33))? {
                let evt = event::read()?;
                match evt {
                    Event::Key(key) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                            break;
                        }
                        if self.handle_key_event(key.code, key.modifiers).await? {
                            break;
                        }
                    }
                    Event::Mouse(mouse) if self.active_tab == ActiveTab::Galaxy => {
                        use crossterm::event::{MouseButton, MouseEventKind};
                        match mouse.kind {
                            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                                let dx = mouse.column as i32 - self.galaxy_state.last_mouse_x.map_or(mouse.column as i32, |x| x);
                                let dy = mouse.row as i32 - self.galaxy_state.last_mouse_y.map_or(mouse.row as i32, |y| y);
                                self.galaxy_state.camera.rotate_yaw(dx as f32 * 0.005);
                                self.galaxy_state.camera.rotate_pitch(-dy as f32 * 0.003);
                            }
                            MouseEventKind::Up(MouseButton::Left) => {
                                self.galaxy_state.last_mouse_x = None;
                                self.galaxy_state.last_mouse_y = None;
                            }
                            _ => {}
                        }
                        self.galaxy_state.last_mouse_x = Some(mouse.column as i32);
                        self.galaxy_state.last_mouse_y = Some(mouse.row as i32);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub async fn handle_key_event(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        // Global Shortcuts
        if mods.contains(KeyModifiers::CONTROL) {
            match code {
                KeyCode::Char('t') => {
                    self.theme = self.theme.next();
                    self.cfg.theme = self.theme.to_str().to_string();
                    let _ = self.cfg.save(&self.config_path);
                    return Ok(false);
                }
                KeyCode::Char('b') => {
                    self.is_sidebar_open = !self.is_sidebar_open;
                    return Ok(false);
                }
                KeyCode::Char('p') => {
                    self.open_translation_picker();
                    return Ok(false);
                }
                KeyCode::Char('m') => {
                    self.open_model_picker().await;
                    return Ok(false);
                }
                KeyCode::Char('u') => {
                    self.trigger_encrypted_backup();
                    return Ok(false);
                }
                _ => {}
            }
        }

        // Modal Active Handling
        if self.active_modal != ActiveModal::None {
            if self.active_modal == ActiveModal::BackupPrompt {
                match code {
                    KeyCode::Esc => {
                        self.active_modal = ActiveModal::None;
                        self.input_modal_buffer.clear();
                    }
                    KeyCode::Backspace => {
                        self.input_modal_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        self.input_modal_buffer.push(c);
                    }
                    KeyCode::Enter => {
                        let pass = self.input_modal_buffer.clone();
                        self.input_modal_buffer.clear();
                        self.active_modal = ActiveModal::None;
                        self.execute_encrypted_backup(&pass);
                    }
                    _ => {}
                }
                return Ok(false);
            }

            if self.active_modal == ActiveModal::CommandPalette {
                match code {
                    KeyCode::Esc => {
                        self.active_modal = ActiveModal::None;
                        self.modal_filter.clear();
                        self.input_buffer.clear();
                    }
                    KeyCode::Up => {
                        if self.modal_selected_idx > 0 {
                            self.modal_selected_idx -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.modal_selected_idx + 1 < self.command_palette_items.len() {
                            self.modal_selected_idx += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if self.modal_filter.len() <= 1 {
                            self.active_modal = ActiveModal::None;
                            self.modal_filter.clear();
                            self.input_buffer.clear();
                        } else {
                            self.modal_filter.pop();
                            self.filter_command_palette();
                        }
                    }
                    KeyCode::Char(c) => {
                        self.modal_filter.push(c);
                        self.filter_command_palette();
                    }
                    KeyCode::Enter => {
                        self.apply_command_palette_selection().await;
                    }
                    _ => {}
                }
                return Ok(false);
            }

            match code {
                KeyCode::Esc => {
                    self.active_modal = ActiveModal::None;
                    self.modal_filter.clear();
                }
                KeyCode::Up => {
                    if self.modal_selected_idx > 0 {
                        self.modal_selected_idx -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.modal_selected_idx + 1 < self.modal_items.len() {
                        self.modal_selected_idx += 1;
                    }
                }
                KeyCode::Backspace => {
                    self.modal_filter.pop();
                    self.filter_modal_items();
                }
                KeyCode::Char(c) => {
                    self.modal_filter.push(c);
                    self.filter_modal_items();
                }
                KeyCode::Enter => {
                    self.apply_modal_selection();
                }
                _ => {}
            }
            return Ok(false);
        }

        // Global Tab Navigation
        if code == KeyCode::Tab {
            self.active_focus = match self.active_focus {
                ActiveFocus::Sidebar => ActiveFocus::MainViewport,
                ActiveFocus::MainViewport => ActiveFocus::PromptInput,
                ActiveFocus::PromptInput => ActiveFocus::Sidebar,
            };
            return Ok(false);
        }

        // Quick Numeric Tab Switching when not focused on prompt
        if self.active_focus != ActiveFocus::PromptInput {
            match code {
                KeyCode::Char('1') => { self.active_tab = ActiveTab::Chat; return Ok(false); }
                KeyCode::Char('2') => { self.active_tab = ActiveTab::Bible; return Ok(false); }
                KeyCode::Char('3') => { self.active_tab = ActiveTab::Library; return Ok(false); }
                KeyCode::Char('4') => { self.active_tab = ActiveTab::Crossref; return Ok(false); }
                KeyCode::Char('5') => { self.active_tab = ActiveTab::Galaxy; return Ok(false); }
                KeyCode::Char('6') => { self.active_tab = ActiveTab::Mesh; return Ok(false); }
                KeyCode::Char('7') => {
                    self.active_tab = ActiveTab::Doctor;
                    self.refresh_doctor_status().await;
                    return Ok(false);
                }
                KeyCode::Char('/') => {
                    self.active_focus = ActiveFocus::PromptInput;
                    self.open_command_palette();
                    return Ok(false);
                }
                KeyCode::Char('?') => {
                    self.active_modal = ActiveModal::Help;
                    return Ok(false);
                }
                _ => {}
            }
        }

        // Focus Specific Navigation
        match self.active_focus {
            ActiveFocus::PromptInput => match code {
                KeyCode::Char('/') if self.input_buffer.is_empty() => {
                    self.open_command_palette();
                }
                KeyCode::Enter => {
                    let input = self.input_buffer.trim().to_string();
                    if !input.is_empty() {
                        self.input_history.push(input.clone());
                        self.input_buffer.clear();
                        self.history_idx = None;
                        self.process_command(&input).await;
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Up => {
                    if !self.input_history.is_empty() {
                        let next_idx = match self.history_idx {
                            Some(i) => i.saturating_sub(1),
                            None => self.input_history.len() - 1,
                        };
                        self.history_idx = Some(next_idx);
                        self.input_buffer = self.input_history[next_idx].clone();
                    }
                }
                KeyCode::Down => {
                    if let Some(i) = self.history_idx {
                        if i + 1 < self.input_history.len() {
                            let next_idx = i + 1;
                            self.history_idx = Some(next_idx);
                            self.input_buffer = self.input_history[next_idx].clone();
                        } else {
                            self.history_idx = None;
                            self.input_buffer.clear();
                        }
                    }
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            },
            ActiveFocus::MainViewport => match self.active_tab {
                ActiveTab::Chat => match code {
                    KeyCode::Up => self.chat_scroll = self.chat_scroll.saturating_sub(1),
                    KeyCode::Down => {
                        let total_lines: usize = self.chat_history.iter().map(|m| m.content.lines().count() + 3).sum();
                        let max_scroll = total_lines.saturating_sub(10);
                        if self.chat_scroll < max_scroll {
                            self.chat_scroll += 1;
                        }
                    }
                    KeyCode::PageUp => self.chat_scroll = self.chat_scroll.saturating_sub(5),
                    KeyCode::PageDown => {
                        let total_lines: usize = self.chat_history.iter().map(|m| m.content.lines().count() + 3).sum();
                        let max_scroll = total_lines.saturating_sub(10);
                        self.chat_scroll = (self.chat_scroll + 5).min(max_scroll);
                    }
                    KeyCode::Home => self.chat_scroll = 0,
                    KeyCode::End => {
                        let total_lines: usize = self.chat_history.iter().map(|m| m.content.lines().count() + 3).sum();
                        self.chat_scroll = total_lines.saturating_sub(10);
                    }
                    _ => {}
                },
                ActiveTab::Bible => match code {
                    KeyCode::PageUp => {
                        self.bible_state.scroll = self.bible_state.scroll.saturating_sub(5);
                    }
                    KeyCode::PageDown => {
                        let max_scroll = self.bible_verses.len().saturating_sub(1);
                        self.bible_state.scroll = (self.bible_state.scroll + 5).min(max_scroll);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.bible_state.selected_book_idx > 0 {
                            self.bible_state.selected_book_idx -= 1;
                            self.bible_state.scroll = 0;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.bible_state.selected_book_idx + 1 < self.bible_books.len() {
                            self.bible_state.selected_book_idx += 1;
                            self.bible_state.scroll = 0;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if self.bible_state.selected_chapter > 1 {
                            self.bible_state.selected_chapter -= 1;
                            self.bible_state.scroll = 0;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let max_chapters = self.bible_reader.as_ref().and_then(|r| {
                            let bname = self.bible_books.get(self.bible_state.selected_book_idx)?;
                            r.get_chapter_count(bname)
                        }).unwrap_or(150);
                        if self.bible_state.selected_chapter < max_chapters {
                            self.bible_state.selected_chapter += 1;
                            self.bible_state.scroll = 0;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Char('c') => {
                        self.bible_state.compare_mode = !self.bible_state.compare_mode;
                        self.load_active_bible_chapter();
                    }
                    KeyCode::Home => self.bible_state.scroll = 0,
                    KeyCode::End => {
                        let max_scroll = self.bible_verses.len().saturating_sub(1);
                        self.bible_state.scroll = max_scroll;
                    }
                    _ => {}
                },
                ActiveTab::Library => match code {
                    KeyCode::Up => {
                        if self.library_state.selected_book_idx > 0 {
                            self.library_state.selected_book_idx -= 1;
                            self.load_active_library_chapter();
                        }
                    }
                    KeyCode::Down => {
                        if self.library_state.selected_book_idx + 1 < self.library_books.len() {
                            self.library_state.selected_book_idx += 1;
                            self.load_active_library_chapter();
                        }
                    }
                    KeyCode::PageUp => self.library_state.scroll = self.library_state.scroll.saturating_sub(5),
                    KeyCode::PageDown => {
                        let max_scroll = self.library_chapter_content.lines().count().saturating_sub(1);
                        self.library_state.scroll = (self.library_state.scroll + 5).min(max_scroll);
                    }
                    KeyCode::Home => self.library_state.scroll = 0,
                    KeyCode::End => {
                        let max_scroll = self.library_chapter_content.lines().count().saturating_sub(1);
                        self.library_state.scroll = max_scroll;
                    }
                    KeyCode::Left if self.library_state.selected_category_idx > 0 => {
                        self.library_state.selected_category_idx -= 1;
                        self.refresh_library_category();
                    }
                    KeyCode::Right if self.library_state.selected_category_idx + 1 < self.library_categories.len() => {
                        self.library_state.selected_category_idx += 1;
                        self.refresh_library_category();
                    }
                    _ => {}
                },
                ActiveTab::Galaxy => {
                    match self.galaxy_state.handle_key(code, mods) {
                        crate::views::galaxy::GalaxyAction::Handled => return Ok(false),
                        crate::views::galaxy::GalaxyAction::Inspect(node) => {
                            match node.entity_type {
                                crate::galaxy::physics::EntityType::Sun => {
                                    self.active_tab = ActiveTab::Bible;
                                    self.load_active_bible_chapter();
                                }
                                crate::galaxy::physics::EntityType::Planet => {
                                    self.active_tab = ActiveTab::Bible;
                                    self.selected_language_code = Some(node.short_code.to_lowercase());
                                    self.selected_language_name = Some(node.name.clone());
                                    self.open_translation_picker();
                                }
                                crate::galaxy::physics::EntityType::Moon => {
                                    self.bible_state.active_translation = node.short_code.to_uppercase();
                                    self.active_tab = ActiveTab::Bible;
                                    self.load_active_bible_chapter();
                                }
                                crate::galaxy::physics::EntityType::Asteroid => {
                                    self.active_tab = ActiveTab::Library;
                                    let cat_name = node.id.replace("cat_", "");
                                    if let Some(pos) = self.library_categories.iter().position(|c| c.eq_ignore_ascii_case(&cat_name)) {
                                        self.library_state.selected_category_idx = pos;
                                        self.refresh_library_category();
                                    } else if let Some(pos) = self.library_books.iter().position(|b| node.name.starts_with(b)) {
                                        self.library_state.selected_book_idx = pos;
                                        self.load_active_library_chapter();
                                    }
                                }
                            }
                            return Ok(false);
                        }
                        crate::views::galaxy::GalaxyAction::None => {}
                    }
                }
                _ => {}
            },
            ActiveFocus::Sidebar => match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let current = self.active_tab as usize;
                    if current > 0 {
                        self.active_tab = match current - 1 {
                            0 => ActiveTab::Chat,
                            1 => ActiveTab::Bible,
                            2 => ActiveTab::Library,
                            3 => ActiveTab::Crossref,
                            4 => ActiveTab::Galaxy,
                            5 => ActiveTab::Mesh,
                            _ => ActiveTab::Doctor,
                        };
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let current = self.active_tab as usize;
                    if current < 6 {
                        self.active_tab = match current + 1 {
                            1 => ActiveTab::Bible,
                            2 => ActiveTab::Library,
                            3 => ActiveTab::Crossref,
                            4 => ActiveTab::Galaxy,
                            5 => ActiveTab::Mesh,
                            6 => ActiveTab::Doctor,
                            _ => ActiveTab::Chat,
                        };
                    }
                }
                KeyCode::Enter => self.active_focus = ActiveFocus::MainViewport,
                _ => {}
            },
        }

        Ok(false)
    }

    fn render(&self, f: &mut Frame) {
        let size = f.area();

        // 1. Top Status Header
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top Header / Tabs
                Constraint::Min(10),   // Viewport + Sidebar
                Constraint::Length(3), // Bottom Prompt Input
            ])
            .split(size);

        self.render_header(f, main_chunks[0]);

        // 2. Middle Row: Sidebar + Active Viewport
        if self.is_sidebar_open {
            let mid_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(28), Constraint::Min(40)])
                .split(main_chunks[1]);

            self.render_sidebar(f, mid_chunks[0]);
            self.render_main_viewport(f, mid_chunks[1]);
        } else {
            self.render_main_viewport(f, main_chunks[1]);
        }

        // 3. Bottom Prompt Input
        self.render_input_bar(f, main_chunks[2]);

        // 4. Overlays & Modals
        match self.active_modal {
            ActiveModal::Help => render_help_modal(f, size, &self.theme),
            ActiveModal::CommandPalette => {
                crate::modals::render_command_palette_modal(
                    f, size, &self.command_palette_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
                );
            }
            ActiveModal::LanguagePicker => render_list_picker_modal(
                f, size, "🌐 Select Scripture Language (66 Available)",
                &self.modal_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
            ),
            ActiveModal::TranslationPicker => render_list_picker_modal(
                f, size, "📖 Select Scripture Translation",
                &self.modal_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
            ),
            ActiveModal::ModelPicker => render_list_picker_modal(
                f, size, "🤖 Select Active Ollama AI Model",
                &self.modal_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
            ),
            ActiveModal::BackupPrompt => render_input_modal(
                f, size, "🔒 Enter Backup Encryption Passphrase",
                "Passphrase", &self.input_modal_buffer, true, &self.theme,
            ),
            _ => {}
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let tabs = vec![
            (ActiveTab::Chat, "[1] Chat"),
            (ActiveTab::Bible, "[2] Bible"),
            (ActiveTab::Library, "[3] Library"),
            (ActiveTab::Crossref, "[4] Cross-Ref"),
            (ActiveTab::Galaxy, "[5] Galaxy"),
            (ActiveTab::Mesh, "[6] Mesh"),
            (ActiveTab::Doctor, "[7] Doctor"),
        ];

        let mut spans = Vec::new();
        spans.push(Span::raw(" "));
        for (tab, label) in tabs {
            if self.active_tab == tab {
                spans.push(Span::styled(format!(" {} ", label), self.theme.tab_active()));
            } else {
                spans.push(Span::styled(format!(" {} ", label), self.theme.tab_inactive()));
            }
            spans.push(Span::raw(" "));
        }

        let model_label = format!(" Model: {} ", self.cfg.model.ollama.model);
        let trans_label = format!(" Trans: {} ", self.bible_state.active_translation);
        let theme_label = format!(" Theme: {} ", self.theme.name());

        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(model_label, self.theme.header_badge()));
        spans.push(Span::raw("•"));
        spans.push(Span::styled(trans_label, self.theme.header_title()));
        spans.push(Span::raw("•"));
        spans.push(Span::styled(theme_label, Style::default().fg(Color::DarkGray)));

        let block = Block::default()
            .title(" 🕊️ PARACLEA SCHOLAR — ADVANCED OFFLINE ASSISTANT ")
            .borders(Borders::ALL)
            .border_type(self.theme.border_type())
            .border_style(self.theme.border_focused());

        let p_left = Paragraph::new(Line::from(spans)).block(block);
        f.render_widget(p_left, area);
    }

    fn render_sidebar(&self, f: &mut Frame, area: Rect) {
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📖 Scripture Engine", self.theme.header_title()),
        ])));
        items.push(ListItem::new(format!("  • Active: {}", self.bible_state.active_translation)));
        items.push(ListItem::new("  • Available: 160 across 66 langs".to_string()));
        items.push(ListItem::new(""));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("🌌 Paraclea Galaxy", self.theme.header_title()),
        ])));
        items.push(ListItem::new(format!("  • Celestial Nodes: {}", self.galaxy_state.system.nodes.len())).style(Style::default().fg(Color::Rgb(255, 215, 0))));
        items.push(ListItem::new(""));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📚 Multi-Category Library", self.theme.header_title()),
        ])));
        for cat in &self.library_categories {
            items.push(ListItem::new(format!("  • [{}]", cat.to_uppercase())).style(Style::default().fg(Color::Cyan)));
        }
        items.push(ListItem::new(""));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("🧬 Knowledge Graph", self.theme.header_title()),
        ])));
        items.push(ListItem::new(format!("  • Dendrite Nodes: {}", self.dendrite_graph.len())));
        items.push(ListItem::new(""));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("🕸 Reticulum Mesh", self.theme.header_title()),
        ])));
        let mesh_status = if self.mesh_engine.is_some() { "ONLINE (Ad-hoc)" } else { "STANDBY" };
        items.push(ListItem::new(format!("  • Interface: {}", mesh_status)).style(Style::default().fg(Color::Green)));

        let block = Block::default()
            .title(" 🧭 Navigation & Decks ")
            .borders(Borders::ALL)
            .border_type(self.theme.border_type())
            .border_style(if self.active_focus == ActiveFocus::Sidebar { self.theme.border_focused() } else { self.theme.border_normal() });

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn render_main_viewport(&self, f: &mut Frame, area: Rect) {
        match self.active_tab {
            ActiveTab::Chat => render_chat_view(
                f, area, &self.chat_history, &self.streaming_text,
                self.is_streaming, self.is_speaking, self.chat_scroll, &self.theme,
            ),
            ActiveTab::Bible => {
                let comp_refs: Vec<(&str, Vec<(usize, String)>)> = self.bible_comparison.iter()
                    .map(|(tag, v)| (tag.as_str(), v.clone()))
                    .collect();
                render_bible_view(
                    f, area, &self.bible_state, &self.bible_books, &self.bible_verses, &comp_refs, &self.theme,
                );
            }
            ActiveTab::Library => render_library_view(
                f, area, &self.library_state, &self.library_categories, &self.library_books,
                &self.library_chapter_title, &self.library_chapter_content, &self.theme,
            ),
            ActiveTab::Crossref => {
                let nodes: Vec<(String, String, String, String)> = self.dendrite_graph.all().into_iter().map(|n| {
                    (n.id, n.title, n.content, n.node_type.as_str().to_string())
                }).collect();
                render_crossref_view(f, area, &self.crossref_state, &nodes, &self.theme);
            }
            ActiveTab::Galaxy => {
                f.render_widget(GalaxyView::new(&self.galaxy_state, self.theme), area);
            }
            ActiveTab::Mesh => {
                let status = self.mesh_engine.as_ref().map(|m| m.status()).unwrap_or_else(|| "Reticulum Mesh Standby".to_string());
                let id_hash = self.mesh_engine.as_ref().and_then(|m| m.identity_hash.as_deref());
                let mailbox: Vec<(String, String, String, String)> = self.mesh_engine.as_ref().map(|m| {
                    m.read_mailbox().into_iter().map(|msg| (msg.timestamp, msg.sender, msg.recipient, msg.content)).collect()
                }).unwrap_or_default();
                render_mesh_view(f, area, &self.mesh_state, &status, id_hash, &[], &mailbox, &self.theme);
            }
            ActiveTab::Doctor => render_doctor_view(
                f, area, self.ollama_online, &self.cfg.model.ollama.model, self.qdrant_online,
                self.dendrite_graph.len(), self.bible_lang_count, self.bible_version_count, self.backup_status.as_deref(), &self.theme,
            ),
        }
    }

    fn render_input_bar(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" ⌨️ Command & Study Prompt (Type '/' for commands, '?' for Help) ")
            .borders(Borders::ALL)
            .border_type(self.theme.border_type())
            .border_style(if self.active_focus == ActiveFocus::PromptInput { self.theme.border_focused() } else { self.theme.border_normal() });

        let prompt_line = Line::from(vec![
            Span::styled(" Paraclea > ", self.theme.user_prompt()),
            Span::styled(&self.input_buffer, Style::default().fg(Color::White)),
            Span::styled("█", self.theme.header_title()),
        ]);

        let p = Paragraph::new(prompt_line).block(block);
        f.render_widget(p, area);
    }

    pub async fn process_command(&mut self, input: &str) {
        let now_str = chrono::Local::now().format("%H:%M").to_string();
        self.chat_history.push(ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
            thinking: None,
            timestamp: now_str,
        });

        let (cmd, args) = match input.split_once(char::is_whitespace) {
            Some((c, a)) => (c.to_lowercase(), a.trim()),
            None => (input.to_lowercase(), ""),
        };

        match cmd.as_str() {
            "/help" | "?" => {
                self.active_modal = ActiveModal::Help;
            }
            "/theme" => {
                self.theme = self.theme.next();
                self.cfg.theme = self.theme.to_str().to_string();
                let _ = self.cfg.save(&self.config_path);
            }
            "/language" | "/lang" => {
                self.open_language_picker();
            }
            "/version" | "/translation" => {
                self.open_translation_picker();
            }
            "/bible" | "/read" => {
                self.active_tab = ActiveTab::Bible;
                if !args.is_empty() {
                    // Navigate to specified book/chapter
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    if let Some(pos) = self.bible_books.iter().position(|b| b.eq_ignore_ascii_case(parts[0])) {
                        self.bible_state.selected_book_idx = pos;
                    }
                    if parts.len() > 1 {
                        if let Ok(ch) = parts[1].parse::<usize>() {
                            self.bible_state.selected_chapter = ch;
                        }
                    }
                }
                self.load_active_bible_chapter();
            }
            "/compare" => {
                self.active_tab = ActiveTab::Bible;
                self.bible_state.compare_mode = !self.bible_state.compare_mode;
                self.load_active_bible_chapter();
            }
            "/library" | "/books" => {
                self.active_tab = ActiveTab::Library;
                if !args.is_empty() {
                    if let Some(pos) = self.library_categories.iter().position(|c| c.eq_ignore_ascii_case(args)) {
                        self.library_state.selected_category_idx = pos;
                        self.refresh_library_category();
                    }
                }
            }
            "/galaxy" => {
                self.active_tab = ActiveTab::Galaxy;
            }
            "/memory" => {
                self.active_tab = ActiveTab::Crossref;
            }
            "/mesh" => {
                self.active_tab = ActiveTab::Mesh;
            }
            "/model" => {
                self.open_model_picker().await;
            }
            "/doctor" => {
                self.active_tab = ActiveTab::Doctor;
                self.refresh_doctor_status().await;
            }
            "/backup" => {
                self.trigger_encrypted_backup();
            }
            "/crossref" => {
                self.active_tab = ActiveTab::Crossref;
            }
            "/clear" => {
                self.chat_history.clear();
            }
            _ => {
                // Dispatch AI Streaming Prompt to Ollama
                self.active_tab = ActiveTab::Chat;
                self.is_streaming = true;
                let tx = self.tx_stream.clone();
                let model = self.cfg.model.ollama.model.clone();
                let ollama = self.ollama.clone();
                let persona_prompt = self.persona.build_system_prompt();
                let query = input.to_string();

                tokio::spawn(async move {
                    let msgs = vec![
                        OllamaChatMessage { role: "system".to_string(), content: persona_prompt },
                        OllamaChatMessage { role: "user".to_string(), content: query },
                    ];
                    let tx_token = tx.clone();
                    let res = ollama.chat_with_model_stream(&model, msgs, move |tok| {
                        let _ = tx_token.send(StreamEvent::Token(tok.to_string()));
                    }).await;

                    match res {
                        Ok(_) => { let _ = tx.send(StreamEvent::Done); }
                        Err(e) => { let _ = tx.send(StreamEvent::Error(e.to_string())); }
                    }
                });
            }
        }
    }

    fn load_active_bible_chapter(&mut self) {
        let active_tag = self.bible_state.active_translation.clone();
        if let Some(target_file) = paraclea_core::bible::find_json_bible_file(&active_tag) {
            if let Ok(new_reader) = BibleReader::load_primary(&target_file) {
                let books: Vec<String> = new_reader.books.iter().map(|b| b.name.clone()).collect();
                if !books.is_empty() {
                    self.bible_books = books;
                }
                self.bible_reader = Some(new_reader);
            }
        }

        if let Some(ref reader) = self.bible_reader {
            let book_name = self.bible_books.get(self.bible_state.selected_book_idx).cloned().unwrap_or_else(|| "Genesis".to_string());
            let ch = self.bible_state.selected_chapter;
            let verses = reader.read_translation_chapter(&active_tag, &book_name, ch).unwrap_or_default();
            self.bible_verses = verses.clone();

            if self.bible_state.compare_mode {
                let mut comparisons = Vec::new();
                comparisons.push((active_tag.clone(), verses.clone()));
                let candidate_tags = ["KJV", "WEB", "BSB", "ASV", "BBE"];
                for tag in candidate_tags {
                    if !tag.eq_ignore_ascii_case(&active_tag) && comparisons.len() < 3 {
                        let v = reader.read_translation_chapter(tag, &book_name, ch).unwrap_or_default();
                        if !v.is_empty() {
                            comparisons.push((tag.to_string(), v));
                        }
                    }
                }
                self.bible_comparison = comparisons;
            } else {
                self.bible_comparison.clear();
            }
        }
    }

    fn refresh_library_category(&mut self) {
        if let Some(cat) = self.library_categories.get(self.library_state.selected_category_idx) {
            let books = self.library_engine.list_books(Some(cat));
            self.library_books = books.iter().map(|b| b.title.clone()).collect();
            self.library_state.selected_book_idx = 0;
            self.load_active_library_chapter();
        }
    }

    fn load_active_library_chapter(&mut self) {
        if let Some(b_title) = self.library_books.get(self.library_state.selected_book_idx) {
            if let Some((_, ch)) = self.library_engine.read_chapter(b_title, self.library_state.selected_chapter) {
                self.library_chapter_title = ch.title.clone();
                self.library_chapter_content = ch.content.clone();
            }
        }
    }

    pub async fn refresh_doctor_status(&mut self) {
        self.ollama_online = self.ollama.health_check().await.unwrap_or(false);
        self.qdrant_online = self.qdrant.health_check().await;
        let languages = BibleReader::list_languages();
        self.bible_lang_count = languages.len();
        let mut total_ver = 0;
        for lang in &languages {
            total_ver += BibleReader::list_translations_for_lang(&lang.code).len();
        }
        self.bible_version_count = total_ver.max(1);
    }

    pub fn open_command_palette(&mut self) {
        self.command_palette_items = COMMAND_PALETTE.to_vec();
        self.modal_selected_idx = 0;
        self.modal_filter = "/".to_string();
        self.active_modal = ActiveModal::CommandPalette;
    }

    pub fn filter_command_palette(&mut self) {
        let q = self.modal_filter.to_lowercase();
        self.command_palette_items = COMMAND_PALETTE
            .iter()
            .copied()
            .filter(|(cmd, desc)| cmd.to_lowercase().contains(&q) || desc.to_lowercase().contains(&q))
            .collect();
        if self.modal_selected_idx >= self.command_palette_items.len() {
            self.modal_selected_idx = 0;
        }
    }

    pub async fn apply_command_palette_selection(&mut self) {
        if let Some(&(cmd, _)) = self.command_palette_items.get(self.modal_selected_idx) {
            self.active_modal = ActiveModal::None;
            self.modal_filter.clear();
            self.input_buffer.clear();
            self.process_command(cmd).await;
        } else {
            self.active_modal = ActiveModal::None;
            self.modal_filter.clear();
            self.input_buffer.clear();
        }
    }

    pub fn open_language_picker(&mut self) {
        let languages = BibleReader::list_languages();
        self.modal_items = languages
            .iter()
            .map(|lang| format!("{} ({})", lang.name, lang.code))
            .collect();
        self.modal_selected_idx = 0;
        self.modal_filter.clear();
        self.active_modal = ActiveModal::LanguagePicker;
    }

    pub fn open_translation_picker(&mut self) {
        let lang_code = self.selected_language_code.as_deref().unwrap_or("eng");
        let trans = BibleReader::list_translations_for_lang(lang_code);
        let lang_name = self.selected_language_name.as_deref().unwrap_or("English");
        let mut items: Vec<String> = trans
            .iter()
            .map(|t| format!("{} - {} ({})", t.tag, t.name, lang_name))
            .collect();
        if items.is_empty() {
            let all_langs = BibleReader::list_languages();
            for lang in &all_langs {
                for t in BibleReader::list_translations_for_lang(&lang.code) {
                    items.push(format!("{} - {} ({})", t.tag, t.name, lang.name));
                }
            }
        }
        if items.is_empty() {
            items = vec![
                "KJV - King James Version (English)".to_string(),
                "BSB - Berean Standard Bible (English)".to_string(),
                "WEB - World English Bible (English)".to_string(),
            ];
        }
        self.modal_items = items;
        self.modal_selected_idx = 0;
        self.modal_filter.clear();
        self.active_modal = ActiveModal::TranslationPicker;
    }

    pub async fn open_model_picker(&mut self) {
        let models = self.ollama.fetch_available_models().await;
        if models.is_empty() {
            self.modal_items = vec!["ministral-3:3b".to_string(), "ornith-1.5:9b".to_string()];
        } else {
            self.modal_items = models.into_iter().map(|m| m.name).collect();
        }
        self.modal_selected_idx = 0;
        self.modal_filter.clear();
        self.active_modal = ActiveModal::ModelPicker;
    }

    pub fn filter_modal_items(&mut self) {
        let q = self.modal_filter.trim().to_lowercase();
        if q.is_empty() { return; }
        // 1. Exact tag match
        if let Some(pos) = self.modal_items.iter().position(|i| {
            let first_token = i.split_whitespace().next().unwrap_or("").to_lowercase();
            first_token == q
        }) {
            self.modal_selected_idx = pos;
            return;
        }
        // 2. Prefix match
        if let Some(pos) = self.modal_items.iter().position(|i| i.to_lowercase().starts_with(&q)) {
            self.modal_selected_idx = pos;
            return;
        }
        // 3. Substring match
        if let Some(pos) = self.modal_items.iter().position(|i| i.to_lowercase().contains(&q)) {
            self.modal_selected_idx = pos;
        }
    }

    pub fn apply_modal_selection(&mut self) {
        match self.active_modal {
            ActiveModal::LanguagePicker => {
                if let Some(sel) = self.modal_items.get(self.modal_selected_idx).cloned() {
                    if let Some(start) = sel.rfind('(') {
                        if let Some(end) = sel.rfind(')') {
                            let code = sel[start + 1..end].trim().to_string();
                            let name = sel[..start].trim().to_string();
                            self.selected_language_code = Some(code);
                            self.selected_language_name = Some(name);
                        }
                    }
                    self.open_translation_picker();
                    return;
                }
            }
            ActiveModal::TranslationPicker => {
                if let Some(sel) = self.modal_items.get(self.modal_selected_idx) {
                    let tag = sel.split_whitespace().next().unwrap_or("KJV").to_uppercase();
                    self.bible_state.active_translation = tag;
                    self.load_active_bible_chapter();
                }
            }
            ActiveModal::ModelPicker => {
                if let Some(sel) = self.modal_items.get(self.modal_selected_idx) {
                    self.cfg.model.ollama.model = sel.clone();
                    let _ = self.cfg.save(&self.config_path);
                }
            }
            _ => {}
        }
        self.active_modal = ActiveModal::None;
    }

    pub fn trigger_encrypted_backup(&mut self) {
        self.input_modal_buffer.clear();
        self.active_modal = ActiveModal::BackupPrompt;
    }

    pub fn execute_encrypted_backup(&mut self, passkey: &str) {
        let trimmed_key = passkey.trim();
        if trimmed_key.is_empty() {
            self.backup_status = Some("⚠️ Backup cancelled: Passphrase is required for encryption.".to_string());
            return;
        }

        if let Ok(home) = std::env::var("HOME") {
            let db_path = PathBuf::from(&home).join(".paraclea/dendrite.db");
            let mut target_dir = PathBuf::from(&home).join(".paraclea/backups");

            // Auto-detect mounted USB flash drive
            let user_name = std::env::var("USER").unwrap_or_default();
            let candidate_media_dirs = vec![
                format!("/media/{}", user_name),
                format!("/run/media/{}", user_name),
                "/media".to_string(),
            ];
            for m_dir_str in candidate_media_dirs {
                let media_dir = PathBuf::from(m_dir_str);
                if media_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&media_dir) {
                        for e in entries.flatten() {
                            if e.path().is_dir() {
                                target_dir = e.path();
                                break;
                            }
                        }
                    }
                }
            }

            let _ = std::fs::create_dir_all(&target_dir);
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let backup_file = target_dir.join(format!("paraclea_backup_{}.enc", ts));

            if db_path.exists() {
                match paraclea_core::backup::EncryptedBackup::create_backup(&db_path, &backup_file, trimmed_key) {
                    Ok(bytes) => {
                        self.backup_status = Some(format!("✓ AES-256-GCM Backup Saved: {:?} ({} bytes)", backup_file, bytes));
                        return;
                    }
                    Err(e) => {
                        self.backup_status = Some(format!("⚠️ Backup failed: {}", e));
                        return;
                    }
                }
            }
        }
        self.backup_status = Some("⚠️ No dendrite.db found to backup yet.".to_string());
    }
}
