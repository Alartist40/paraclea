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
        assert_eq!(t2, AppTheme::MonasteryAmber);
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
            AppTheme::MonasteryAmber,
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
        assert_eq!(app.theme, AppTheme::MonasteryAmber);
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
}


