//! Theme system and visual presets for Paraclea TUI.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    RoyalByzantium, // Paraclea signature: Gold & Regal Purple
    MonasteryAmber,  // Warm parchment ochre & sepia
    CyberScholar,    // Neon Cyan & Electric Magenta
    EmeraldMatrix,   // Mint green & Obsidian
}

impl AppTheme {
    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::RoyalByzantium => "Royal Byzantium (Gold & Purple)",
            AppTheme::MonasteryAmber => "Monastery Parchment (Amber)",
            AppTheme::CyberScholar => "Cyber Scholar (Cyan & Magenta)",
            AppTheme::EmeraldMatrix => "Emerald Matrix (Green)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AppTheme::RoyalByzantium => AppTheme::MonasteryAmber,
            AppTheme::MonasteryAmber => AppTheme::CyberScholar,
            AppTheme::CyberScholar => AppTheme::EmeraldMatrix,
            AppTheme::EmeraldMatrix => AppTheme::RoyalByzantium,
        }
    }

    pub fn border_type(&self) -> BorderType {
        BorderType::Rounded
    }

    pub fn primary(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(255, 215, 0),   // Gold
            AppTheme::MonasteryAmber => Color::Rgb(255, 191, 0),   // Amber
            AppTheme::CyberScholar => Color::Rgb(0, 229, 255),     // Cyan
            AppTheme::EmeraldMatrix => Color::Rgb(0, 255, 127),    // Mint
        }
    }

    pub fn secondary(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(177, 74, 237),  // Purple
            AppTheme::MonasteryAmber => Color::Rgb(212, 163, 115),  // Sepia
            AppTheme::CyberScholar => Color::Rgb(255, 0, 127),     // Magenta
            AppTheme::EmeraldMatrix => Color::Rgb(0, 150, 70),     // Dark Green
        }
    }

    pub fn header_title(&self) -> Style {
        Style::default()
            .fg(self.primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_badge(&self) -> Style {
        Style::default()
            .fg(self.secondary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_active(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_inactive(&self) -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn border_normal(&self) -> Style {
        Style::default().fg(Color::Rgb(60, 60, 80))
    }

    pub fn border_focused(&self) -> Style {
        Style::default().fg(self.primary())
    }

    pub fn scripture_card(&self) -> Style {
        Style::default().fg(self.primary())
    }

    pub fn scripture_text(&self) -> Style {
        match self {
            AppTheme::RoyalByzantium => Style::default().fg(Color::Rgb(240, 235, 255)),
            AppTheme::MonasteryAmber => Style::default().fg(Color::Rgb(255, 245, 220)),
            AppTheme::CyberScholar => Style::default().fg(Color::Rgb(220, 245, 255)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(220, 255, 220)),
        }
    }

    pub fn user_prompt(&self) -> Style {
        Style::default()
            .fg(self.secondary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn assistant_header(&self) -> Style {
        Style::default()
            .fg(self.primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn thinking_block(&self) -> Style {
        match self {
            AppTheme::RoyalByzantium => Style::default().fg(Color::Rgb(180, 140, 210)).add_modifier(Modifier::ITALIC),
            AppTheme::MonasteryAmber => Style::default().fg(Color::Rgb(200, 150, 70)).add_modifier(Modifier::ITALIC),
            AppTheme::CyberScholar => Style::default().fg(Color::Rgb(220, 100, 180)).add_modifier(Modifier::ITALIC),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(100, 180, 100)).add_modifier(Modifier::ITALIC),
        }
    }

    pub fn highlight_item(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.primary())
            .add_modifier(Modifier::BOLD)
    }
}
