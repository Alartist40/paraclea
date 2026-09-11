//! Paraclea TUI — Terminal Graphical Interface Library.

pub mod app;
pub mod modals;
pub mod terminal;
pub mod theme;
pub mod views {
    pub mod bible;
    pub mod chat;
    pub mod crossref;
    pub mod doctor;
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
        assert_eq!(t5, AppTheme::RoyalByzantium);
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

        // Switch to Bible tab via F2
        let _ = app.handle_key_event(KeyCode::F(2), KeyModifiers::NONE).await;
        assert_eq!(app.active_tab, ActiveTab::Bible);

        // Switch to Library tab via F3
        let _ = app.handle_key_event(KeyCode::F(3), KeyModifiers::NONE).await;
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


