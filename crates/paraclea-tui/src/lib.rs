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
    use crate::theme::AppTheme;

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
        assert_eq!(app.active_tab, app::ActiveTab::Chat);
        assert!(app.is_sidebar_open);
        assert_eq!(app.theme, AppTheme::RoyalByzantium);
        assert!(!app.bible_books.is_empty());
        assert!(!app.library_categories.is_empty());
    }
}

