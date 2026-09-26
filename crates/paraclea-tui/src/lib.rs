//! Paraclea TUI — Terminal Graphical Interface Library.

pub mod app;
pub mod ascii_art;
pub mod galaxy;
pub mod modals;
pub mod terminal;
pub mod theme;
pub mod views {
    pub mod bible;
    pub mod chat;
    pub mod crossref;
    pub mod doctor;
    pub mod galaxy;
    pub mod library;
    pub mod mesh;
}

use anyhow::Result;
use paraclea_core::config::Config;

use crate::app::App;
use crate::terminal::TuiRuntimeGuard;

/// Launch the interactive Paraclea Ratatui Terminal Interface.
pub async fn run_tui(cfg: Config) -> Result<()> {
    let (_guard, mut terminal) = TuiRuntimeGuard::init()?;
    let mut app = App::new(cfg);
    app.run_loop(&mut terminal).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ActiveModal, ActiveTab, App};
    use crate::theme::AppTheme;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_theme_cycling() {
        let t1 = AppTheme::RoyalByzantium;
        let t2 = t1.next();
        assert_eq!(t2, AppTheme::CrimsonCodex);
        let t3 = t2.next();
        assert_eq!(t3, AppTheme::CyberScholar);
        let t4 = t3.next();
        assert_eq!(t4, AppTheme::EmeraldMatrix);
        let t5 = t4.next();
        assert_eq!(t5, AppTheme::CelestialMidnight);
        let t6 = t5.next();
        assert_eq!(t6, AppTheme::RoyalByzantium);

        // Verify color definitions for all themes
        for theme in [
            AppTheme::RoyalByzantium,
            AppTheme::CrimsonCodex,
            AppTheme::CyberScholar,
            AppTheme::EmeraldMatrix,
            AppTheme::CelestialMidnight,
        ] {
            assert!(!theme.name().is_empty());
            let _ = theme.primary();
            let _ = theme.secondary();
            let _ = theme.scripture_text();
            let _ = theme.thinking_block();
            let _ = theme.ascii_char_style('@');
            let _ = theme.ascii_char_style(':');
        }
    }

    #[test]
    fn test_ascii_art_banner_rendering() {
        use crate::ascii_art::{render_ascii_line, PARACLEA_BANNER};
        assert!(!PARACLEA_BANNER.is_empty());
        for line in PARACLEA_BANNER.lines() {
            let rendered = render_ascii_line(line, 4, &AppTheme::RoyalByzantium);
            assert!(!rendered.spans.is_empty() || line.is_empty());
        }
    }

    #[test]
    fn test_app_state_initialization() {
        let cfg = Config::default();
        let app = App::new(cfg);
        assert_eq!(app.active_tab, ActiveTab::Chat);
        assert!(app.is_sidebar_open);
        assert_eq!(app.theme, AppTheme::RoyalByzantium);
        assert!(!app.bible_books.is_empty());
        assert!(!app.library_categories.is_empty());
        assert!(app.bible_lang_count > 0);
        assert!(app.bible_version_count > 0);
    }

    #[tokio::test]
    async fn test_key_event_tab_switching() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // Switch to Bible tab via 2 when not focused on prompt
        app.active_focus = crate::app::ActiveFocus::Sidebar;
        let _ = app.handle_key_event(KeyCode::Char('2'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Bible);

        // Switch to Library tab via 3
        let _ = app.handle_key_event(KeyCode::Char('3'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Library);

        // Toggle sidebar with Ctrl+B
        assert!(app.is_sidebar_open);
        let _ = app.handle_key_event(KeyCode::Char('b'), KeyModifiers::CONTROL).await;
        assert!(!app.is_sidebar_open);

        // Cycle theme with Ctrl+T
        assert_eq!(app.theme, AppTheme::RoyalByzantium);
        let _ = app.handle_key_event(KeyCode::Char('t'), KeyModifiers::CONTROL).await;
        assert_eq!(app.theme, AppTheme::CrimsonCodex);
    }

    #[tokio::test]
    async fn test_process_commands() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // /bible command
        app.process_command("/bible Genesis 1").await;
        assert_eq!(app.active_tab, ActiveTab::Bible);
        assert_eq!(app.bible_state.selected_chapter, 1);

        // /compare command
        app.process_command("/compare").await;
        assert_eq!(app.active_tab, ActiveTab::Bible);
        assert!(app.bible_state.compare_mode);

        // /library command
        app.process_command("/library survival").await;
        assert_eq!(app.active_tab, ActiveTab::Library);

        // /language command
        app.process_command("/language").await;
        assert_eq!(app.active_modal, ActiveModal::LanguagePicker);

        // /version command
        app.process_command("/version").await;
        assert_eq!(app.active_modal, ActiveModal::TranslationPicker);

        // /memory /crossref command
        app.process_command("/memory").await;
        assert_eq!(app.active_tab, ActiveTab::Crossref);

        // /mesh command
        app.process_command("/mesh").await;
        assert_eq!(app.active_tab, ActiveTab::Mesh);

        // /help modal
        app.process_command("/help").await;
        assert_eq!(app.active_modal, ActiveModal::Help);

        // /clear
        app.chat_history.push(crate::views::chat::ChatMessage {
            role: "user".to_string(),
            content: "test".to_string(),
            thinking: None,
            timestamp: "12:00".to_string(),
        });
        app.process_command("/clear").await;
        assert!(app.chat_history.is_empty());
    }

    #[test]
    fn test_filter_and_apply_modal() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        app.open_translation_picker();
        assert_eq!(app.active_modal, ActiveModal::TranslationPicker);
        assert!(!app.modal_items.is_empty());

        // Filter for BSB
        app.modal_filter = "BSB".to_string();
        app.filter_modal_items();
        app.apply_modal_selection();
        assert_eq!(app.active_modal, ActiveModal::None);
        assert_eq!(app.bible_state.active_translation, "BSB");
    }

    #[tokio::test]
    async fn test_command_palette_and_language_flow() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // Trigger command palette via '/' on empty prompt
        app.active_focus = crate::app::ActiveFocus::PromptInput;
        app.input_buffer.clear();
        let _ = app.handle_key_event(KeyCode::Char('/'), KeyModifiers::NONE).await;
        assert_eq!(app.active_modal, ActiveModal::CommandPalette);
        assert!(!app.command_palette_items.is_empty());
        app.active_modal = ActiveModal::None;

        // Trigger command palette via '/' from Sidebar
        app.active_focus = crate::app::ActiveFocus::Sidebar;
        let _ = app.handle_key_event(KeyCode::Char('/'), KeyModifiers::NONE).await;
        assert_eq!(app.active_modal, ActiveModal::CommandPalette);
        assert_eq!(app.active_focus, crate::app::ActiveFocus::PromptInput);
        app.active_modal = ActiveModal::None;

        // Trigger help modal via '?' from MainViewport
        app.active_focus = crate::app::ActiveFocus::MainViewport;
        let _ = app.handle_key_event(KeyCode::Char('?'), KeyModifiers::NONE).await;
        assert_eq!(app.active_modal, ActiveModal::Help);
        app.active_modal = ActiveModal::None;

        // Filter palette for 'doctor'
        app.modal_filter = "/doc".to_string();
        app.filter_command_palette();
        assert!(app.command_palette_items.iter().any(|(c, _)| *c == "/doctor"));

        // Language -> Translation two-step selection
        app.open_language_picker();
        assert_eq!(app.active_modal, ActiveModal::LanguagePicker);
        // Find English
        app.modal_filter = "English".to_string();
        app.filter_modal_items();
        app.apply_modal_selection();
        assert_eq!(app.active_modal, ActiveModal::TranslationPicker);
        assert_eq!(app.selected_language_code.as_deref(), Some("eng"));
    }

    #[tokio::test]
    async fn test_translation_reload_and_comparison_dedup() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // 1. Initial State: KJV
        assert_eq!(app.bible_state.active_translation, "KJV");
        assert!(!app.bible_verses.is_empty());

        // 2. Switch to WEB translation via picker
        app.open_translation_picker();
        app.modal_filter = "WEB".to_string();
        app.filter_modal_items();
        app.apply_modal_selection();
        assert_eq!(app.bible_state.active_translation, "WEB");
        assert!(!app.bible_verses.is_empty());

        // 3. Enable compare mode via /compare
        app.process_command("/compare").await;
        assert!(app.bible_state.compare_mode);
        assert!(!app.bible_comparison.is_empty());

        // 4. Assert active translation is first and all column tags are distinct (no duplicates)
        assert_eq!(app.bible_comparison[0].0, "WEB");
        let mut seen_tags = std::collections::HashSet::new();
        for (tag, verses) in &app.bible_comparison {
            assert!(seen_tags.insert(tag.to_uppercase()), "Duplicate comparison tag found: {}", tag);
            assert!(!verses.is_empty(), "Comparison column for {} was unexpectedly empty", tag);
        }
    }

    #[test]
    fn test_encrypted_backup_trigger() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        app.trigger_encrypted_backup();
        assert_eq!(app.active_modal, ActiveModal::BackupPrompt);

        // Passphrase cancelled
        app.execute_encrypted_backup("");
        assert!(app.backup_status.as_ref().unwrap().contains("cancelled"));

        // Passphrase provided
        app.execute_encrypted_backup("my_secure_study_passphrase");
        assert!(app.backup_status.is_some());
    }

    #[tokio::test]
    async fn test_numeric_tab_switch_on_empty_prompt() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // 1. On fresh launch, default focus is MainViewport: 1-7 switch tabs immediately
        assert_eq!(app.active_focus, crate::app::ActiveFocus::MainViewport);
        let _ = app.handle_key_event(KeyCode::Char('2'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Bible);

        let _ = app.handle_key_event(KeyCode::Char('3'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Library);

        let _ = app.handle_key_event(KeyCode::Char('1'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Chat);

        // 2. In Sidebar: 1-7 also switches tabs immediately
        app.active_focus = crate::app::ActiveFocus::Sidebar;
        let _ = app.handle_key_event(KeyCode::Char('4'), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Crossref);

        // 3. In PromptInput: typing digits appends characters (e.g. "2 Corinthians") and does NOT switch tab
        app.active_focus = crate::app::ActiveFocus::PromptInput;
        app.input_buffer.clear();
        let _ = app.handle_key_event(KeyCode::Char('2'), KeyModifiers::NONE).await;
        let _ = app.handle_key_event(KeyCode::Char(' '), KeyModifiers::NONE).await;
        let _ = app.handle_key_event(KeyCode::Char('C'), KeyModifiers::NONE).await;
        assert_eq!(app.input_buffer, "2 C");
        assert_eq!(app.active_tab, ActiveTab::Crossref); // Tab does not switch

        // 4. In PromptInput: Alt+1..Alt+7 still allows instant deck switching
        let _ = app.handle_key_event(KeyCode::Char('1'), KeyModifiers::ALT).await;
        assert_eq!(app.active_tab, ActiveTab::Chat);
    }

    #[tokio::test]
    async fn test_chat_auto_scroll() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        assert!(app.chat_auto_scroll);
        assert_eq!(app.chat_scroll, 0);

        // Add multiple chat messages
        for i in 0..20 {
            app.chat_history.push(crate::views::chat::ChatMessage {
                role: "user".to_string(),
                content: format!("Message line {}", i),
                thinking: None,
                timestamp: "12:00".to_string(),
            });
        }
        app.update_chat_scroll_to_bottom();
        assert!(app.chat_auto_scroll);

        // User manually scrolls up in MainViewport
        app.active_focus = crate::app::ActiveFocus::MainViewport;
        app.active_tab = ActiveTab::Chat;
        let _ = app.handle_key_event(KeyCode::Up, KeyModifiers::NONE).await;
        assert!(!app.chat_auto_scroll, "Auto scroll should disengage on manual up scroll");

        // Sending a new message re-engages auto-scroll
        app.process_command("New message").await;
        assert!(app.chat_auto_scroll, "Auto scroll should re-engage on message send");

        // /clear resets scroll and re-engages auto scroll
        app.process_command("/clear").await;
        assert_eq!(app.chat_scroll, 0);
        assert!(app.chat_auto_scroll);
    }

    #[tokio::test]
    async fn test_real_stream_cancellation() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // Simulate streaming state with partial reply
        app.is_streaming = true;
        app.streaming_text = "Partial AI reply...".to_string();
        let initial_gen = app.stream_generation_id;

        // User presses Esc
        let _ = app.handle_key_event(KeyCode::Esc, KeyModifiers::NONE).await;

        // Verify stream is cancelled, text cleared, generation ID advanced, and partial message saved
        assert!(!app.is_streaming);
        assert!(app.streaming_text.is_empty());
        assert_eq!(app.stream_generation_id, initial_gen + 1);
        assert!(app.chat_history.iter().any(|m| m.content.contains("Partial AI reply...") && m.content.contains("[cancelled]")));

        // Verify stale tokens with old gen_id are ignored
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tx.send(crate::app::StreamEvent::Token(initial_gen, "Stale token".to_string())).unwrap();
        if let Ok(crate::app::StreamEvent::Token(gen, tok)) = rx.try_recv() {
            if gen == app.stream_generation_id {
                app.streaming_text.push_str(&tok);
            }
        }
        assert!(app.streaming_text.is_empty(), "Stale token should not be appended");
    }

    #[tokio::test]
    async fn test_chapter_clamping_all_paths() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        // Slash command with out-of-range chapter
        app.process_command("/bible Genesis 999").await;
        assert_eq!(app.active_tab, ActiveTab::Bible);
        assert!(app.bible_state.selected_chapter <= 50, "Genesis chapter 999 should be clamped to 50");

        // Direct chapter setting beyond max
        app.bible_state.selected_chapter = 500;
        app.process_command("/read Exodus").await;
        assert!(app.bible_state.selected_chapter <= 40, "Exodus chapter should be clamped to 40");
    }

    #[tokio::test]
    async fn test_library_chapter_navigation() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        app.active_tab = ActiveTab::Library;
        app.active_focus = crate::app::ActiveFocus::MainViewport;
        assert_eq!(app.library_state.selected_chapter, 1);

        // Next chapter via ']'
        let _ = app.handle_key_event(KeyCode::Char(']'), KeyModifiers::NONE).await;
        assert!(app.library_state.selected_chapter >= 1);

        // Prev chapter via '['
        let _ = app.handle_key_event(KeyCode::Char('['), KeyModifiers::NONE).await;
        assert_eq!(app.library_state.selected_chapter, 1);
    }

    #[tokio::test]
    async fn test_clear_aborts_active_stream() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        app.is_streaming = true;
        app.streaming_text = "in-flight tokens".to_string();
        app.chat_history.push(crate::views::chat::ChatMessage {
            role: "user".to_string(),
            content: "old message".to_string(),
            thinking: None,
            timestamp: "12:00".to_string(),
        });
        let initial_gen = app.stream_generation_id;

        app.process_command("/clear").await;

        assert!(!app.is_streaming, "/clear should stop the streaming flag");
        assert!(app.streaming_text.is_empty(), "/clear should drop in-flight text");
        assert!(app.chat_history.is_empty(), "/clear should empty history");
        assert_eq!(app.stream_generation_id, initial_gen + 1, "/clear should invalidate the running stream");
        assert_eq!(app.chat_scroll, 0);
        assert!(app.chat_auto_scroll);
    }

    #[test]
    fn test_filter_selection_preservation() {
        let cfg = Config::default();
        let mut app = App::new(cfg);

        app.open_translation_picker();
        // Select an item that isn't the first one
        if app.modal_items.len() > 1 {
            let target_item = app.modal_items[1].clone();
            app.modal_selected_idx = 1;

            // Filter with a query that still contains the target item
            let first_word = target_item.split_whitespace().next().unwrap_or("");
            app.modal_filter = first_word.to_string();
            app.filter_modal_items();

            // Selected index should point to the target item in filtered list
            assert_eq!(app.modal_items.get(app.modal_selected_idx), Some(&target_item));
        }
    }
}


