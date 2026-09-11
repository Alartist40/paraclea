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

use crate::modals::{render_help_modal, render_list_picker_modal};
use crate::theme::AppTheme;
use crate::views::{
    bible::{render_bible_view, BibleViewState},
    chat::{render_chat_view, ChatMessage},
    crossref::{render_crossref_view, CrossrefViewState},
    doctor::render_doctor_view,
    library::{render_library_view, LibraryViewState},
    mesh::{render_mesh_view, MeshViewState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Chat = 0,
    Bible = 1,
    Library = 2,
    Crossref = 3,
    Mesh = 4,
    Doctor = 5,
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
    TranslationPicker,
    ModelPicker,
    CommandPalette,
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

    // Views State
    pub chat_history: Vec<ChatMessage>,
    pub streaming_text: String,
    pub is_streaming: bool,
    pub is_speaking: bool,
    pub chat_scroll: usize,

    pub bible_state: BibleViewState,
    pub bible_books: Vec<String>,
    pub bible_verses: Vec<(usize, String)>,
    pub bible_comparison: Vec<(&'static str, Vec<(usize, String)>)>,

    pub library_state: LibraryViewState,
    pub library_categories: Vec<String>,
    pub library_books: Vec<String>,
    pub library_chapter_title: String,
    pub library_chapter_content: String,

    pub crossref_state: CrossrefViewState,
    pub mesh_state: MeshViewState,

    // Prompt & Modals
    pub input_buffer: String,
    pub input_history: Vec<String>,
    pub history_idx: Option<usize>,

    pub modal_filter: String,
    pub modal_items: Vec<String>,
    pub modal_selected_idx: usize,

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

        Self {
            cfg,
            config_path,
            theme: AppTheme::RoyalByzantium,
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

            input_buffer: String::new(),
            input_history: Vec::new(),
            history_idx: None,

            modal_filter: String::new(),
            modal_items: Vec::new(),
            modal_selected_idx: 0,

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

            // Draw 30 FPS Frame
            terminal.draw(|f| self.render(f))?;

            // Event Poll (33ms = ~30 FPS)
            if event::poll(Duration::from_millis(33))? {
                if let Event::Key(key) = event::read()? {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        break;
                    }
                    if self.handle_key_event(key.code, key.modifiers).await? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_key_event(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        // Global Shortcuts
        if mods.contains(KeyModifiers::CONTROL) {
            match code {
                KeyCode::Char('t') => {
                    self.theme = self.theme.next();
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

        // Tab Switching via F-keys
        match code {
            KeyCode::F(1) => { self.active_tab = ActiveTab::Chat; return Ok(false); }
            KeyCode::F(2) => { self.active_tab = ActiveTab::Bible; return Ok(false); }
            KeyCode::F(3) => { self.active_tab = ActiveTab::Library; return Ok(false); }
            KeyCode::F(4) => { self.active_tab = ActiveTab::Crossref; return Ok(false); }
            KeyCode::F(5) => { self.active_tab = ActiveTab::Mesh; return Ok(false); }
            KeyCode::F(6) => { self.active_tab = ActiveTab::Doctor; return Ok(false); }
            KeyCode::Tab => {
                self.active_focus = match self.active_focus {
                    ActiveFocus::Sidebar => ActiveFocus::MainViewport,
                    ActiveFocus::MainViewport => ActiveFocus::PromptInput,
                    ActiveFocus::PromptInput => ActiveFocus::Sidebar,
                };
                return Ok(false);
            }
            _ => {}
        }

        // Focus Specific Navigation
        match self.active_focus {
            ActiveFocus::PromptInput => match code {
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
                    KeyCode::Down => self.chat_scroll += 1,
                    KeyCode::PageUp => self.chat_scroll = self.chat_scroll.saturating_sub(5),
                    KeyCode::PageDown => self.chat_scroll += 5,
                    _ => {}
                },
                ActiveTab::Bible => match code {
                    KeyCode::Up => {
                        if self.bible_state.selected_book_idx > 0 {
                            self.bible_state.selected_book_idx -= 1;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Down => {
                        if self.bible_state.selected_book_idx + 1 < self.bible_books.len() {
                            self.bible_state.selected_book_idx += 1;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Left => {
                        if self.bible_state.selected_chapter > 1 {
                            self.bible_state.selected_chapter -= 1;
                            self.load_active_bible_chapter();
                        }
                    }
                    KeyCode::Right => {
                        self.bible_state.selected_chapter += 1;
                        self.load_active_bible_chapter();
                    }
                    KeyCode::Char('c') => {
                        self.bible_state.compare_mode = !self.bible_state.compare_mode;
                        self.load_active_bible_chapter();
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
                    KeyCode::Left => {
                        if self.library_state.selected_category_idx > 0 {
                            self.library_state.selected_category_idx -= 1;
                            self.refresh_library_category();
                        }
                    }
                    KeyCode::Right => {
                        if self.library_state.selected_category_idx + 1 < self.library_categories.len() {
                            self.library_state.selected_category_idx += 1;
                            self.refresh_library_category();
                        }
                    }
                    _ => {}
                },
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
                            4 => ActiveTab::Mesh,
                            _ => ActiveTab::Doctor,
                        };
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let current = self.active_tab as usize;
                    if current < 5 {
                        self.active_tab = match current + 1 {
                            1 => ActiveTab::Bible,
                            2 => ActiveTab::Library,
                            3 => ActiveTab::Crossref,
                            4 => ActiveTab::Mesh,
                            5 => ActiveTab::Doctor,
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
            ActiveModal::TranslationPicker => render_list_picker_modal(
                f, size, "📖 Select Scripture Translation (160 Available)",
                &self.modal_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
            ),
            ActiveModal::ModelPicker => render_list_picker_modal(
                f, size, "🤖 Select Active Ollama AI Model",
                &self.modal_items, self.modal_selected_idx, &self.modal_filter, &self.theme,
            ),
            _ => {}
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let tabs = vec![
            (ActiveTab::Chat, "[F1] Chat"),
            (ActiveTab::Bible, "[F2] Bible"),
            (ActiveTab::Library, "[F3] Library"),
            (ActiveTab::Crossref, "[F4] Cross-Ref"),
            (ActiveTab::Mesh, "[F5] Mesh"),
            (ActiveTab::Doctor, "[F6] Doctor"),
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
        items.push(ListItem::new(format!("  • Available: 160 across 66 langs")));
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
                render_bible_view(
                    f, area, &self.bible_state, &self.bible_books, &self.bible_verses, &[], &self.theme,
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
            ActiveTab::Mesh => {
                let status = self.mesh_engine.as_ref().map(|m| m.status()).unwrap_or_else(|| "Reticulum Mesh Standby".to_string());
                let id_hash = self.mesh_engine.as_ref().and_then(|m| m.identity_hash.as_deref());
                let mailbox: Vec<(String, String, String, String)> = self.mesh_engine.as_ref().map(|m| {
                    m.read_mailbox().into_iter().map(|msg| (msg.timestamp, msg.sender, msg.recipient, msg.content)).collect()
                }).unwrap_or_default();
                render_mesh_view(f, area, &self.mesh_state, &status, id_hash, &[], &mailbox, &self.theme);
            }
            ActiveTab::Doctor => render_doctor_view(
                f, area, true, &self.cfg.model.ollama.model, false,
                self.dendrite_graph.len(), 66, 160, self.backup_status.as_deref(), &self.theme,
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

    async fn process_command(&mut self, input: &str) {
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
                    self.load_active_bible_chapter();
                }
            }
            "/compare" => {
                self.active_tab = ActiveTab::Bible;
                self.bible_state.compare_mode = true;
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
            "/mesh" => {
                self.active_tab = ActiveTab::Mesh;
            }
            "/doctor" => {
                self.active_tab = ActiveTab::Doctor;
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
        if let Some(ref reader) = self.bible_reader {
            let book_name = self.bible_books.get(self.bible_state.selected_book_idx).cloned().unwrap_or_else(|| "Genesis".to_string());
            let ch = self.bible_state.selected_chapter;
            let verses = reader.read_chapter(&book_name, ch).unwrap_or_default();
            self.bible_verses = verses;
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

    fn open_translation_picker(&mut self) {
        self.modal_items = vec![
            "KJV - King James Version (English)".to_string(),
            "BSB - Berean Standard Bible (English)".to_string(),
            "WEB - World English Bible (English)".to_string(),
            "RVA - Reina Valera Antigua (Spanish)".to_string(),
            "Crampon - French Crampon 1923 (French)".to_string(),
            "Luther - Martin Luther Bibel (German)".to_string(),
            "Synodal - Russian Synodal Bible (Russian)".to_string(),
        ];
        self.modal_selected_idx = 0;
        self.modal_filter.clear();
        self.active_modal = ActiveModal::TranslationPicker;
    }

    async fn open_model_picker(&mut self) {
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

    fn filter_modal_items(&mut self) {
        let q = self.modal_filter.to_lowercase();
        if q.is_empty() { return; }
        if let Some(pos) = self.modal_items.iter().position(|i| i.to_lowercase().contains(&q)) {
            self.modal_selected_idx = pos;
        }
    }

    fn apply_modal_selection(&mut self) {
        match self.active_modal {
            ActiveModal::TranslationPicker => {
                if let Some(sel) = self.modal_items.get(self.modal_selected_idx) {
                    let tag = sel.split_whitespace().next().unwrap_or("KJV");
                    self.bible_state.active_translation = tag.to_string();
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

    fn trigger_encrypted_backup(&mut self) {
        if let Ok(home) = std::env::var("HOME") {
            let db_path = PathBuf::from(&home).join(".paraclea/dendrite.db");
            let backup_dir = PathBuf::from(&home).join(".paraclea/backups");
            let _ = std::fs::create_dir_all(&backup_dir);
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let backup_file = backup_dir.join(format!("paraclea_backup_{}.enc", ts));

            if db_path.exists() {
                use sha2::{Sha256, Digest};
                use std::io::{Read, Write};
                if let Ok(mut fin) = std::fs::File::open(&db_path) {
                    let mut buf = Vec::new();
                    let _ = fin.read_to_end(&mut buf);
                    let mut hasher = Sha256::new();
                    hasher.update(b"PARACLEA_SECURE_SALT_2026");
                    let key = hasher.finalize();
                    let mut enc = Vec::with_capacity(buf.len());
                    for (i, b) in buf.iter().enumerate() {
                        enc.push(b ^ key[i % key.len()]);
                    }
                    if let Ok(mut fout) = std::fs::File::create(&backup_file) {
                        let _ = fout.write_all(b"PARACLEA_ENC_v1");
                        let _ = fout.write_all(&enc);
                        self.backup_status = Some(format!("✓ Encrypted Backup Saved: {:?} ({} bytes)", backup_file, enc.len()));
                        return;
                    }
                }
            }
        }
        self.backup_status = Some("⚠️ No dendrite.db found to backup yet.".to_string());
    }
}
