//! Theme system and visual presets for Paraclea TUI.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    RoyalByzantium,    // Paraclea signature: Gold & Regal Purple
    CrimsonCodex,      // Deep Red & Cream
    CyberScholar,      // Neon Cyan & Electric Magenta
    EmeraldMatrix,     // Mint green & Obsidian
    CelestialMidnight, // Deep Starlight Indigo & Silver
}

impl AppTheme {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "RoyalByzantium" => Some(Self::RoyalByzantium),
            "CrimsonCodex" => Some(Self::CrimsonCodex),
            "CyberScholar" => Some(Self::CyberScholar),
            "EmeraldMatrix" => Some(Self::EmeraldMatrix),
            "CelestialMidnight" => Some(Self::CelestialMidnight),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            AppTheme::RoyalByzantium => "RoyalByzantium",
            AppTheme::CrimsonCodex => "CrimsonCodex",
            AppTheme::CyberScholar => "CyberScholar",
            AppTheme::EmeraldMatrix => "EmeraldMatrix",
            AppTheme::CelestialMidnight => "CelestialMidnight",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::RoyalByzantium => "Royal Byzantium (Gold & Purple)",
            AppTheme::CrimsonCodex => "Crimson Codex (Red & Cream)",
            AppTheme::CyberScholar => "Cyber Scholar (Cyan & Magenta)",
            AppTheme::EmeraldMatrix => "Emerald Matrix (Green)",
            AppTheme::CelestialMidnight => "Celestial Midnight (Indigo & Silver)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AppTheme::RoyalByzantium => AppTheme::CrimsonCodex,
            AppTheme::CrimsonCodex => AppTheme::CyberScholar,
            AppTheme::CyberScholar => AppTheme::EmeraldMatrix,
            AppTheme::EmeraldMatrix => AppTheme::CelestialMidnight,
            AppTheme::CelestialMidnight => AppTheme::RoyalByzantium,
        }
    }

    pub fn border_type(&self) -> BorderType {
        BorderType::Rounded
    }

    pub fn primary(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(255, 215, 0),     // Gold
            AppTheme::CrimsonCodex => Color::Rgb(180, 30, 40),       // Crimson
            AppTheme::CyberScholar => Color::Rgb(0, 229, 255),       // Cyan
            AppTheme::EmeraldMatrix => Color::Rgb(0, 255, 127),      // Mint
            AppTheme::CelestialMidnight => Color::Rgb(100, 181, 246), // Starlight Cyan
        }
    }

    pub fn secondary(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(177, 74, 237),    // Purple
            AppTheme::CrimsonCodex => Color::Rgb(255, 240, 220),     // Cream
            AppTheme::CyberScholar => Color::Rgb(255, 0, 127),       // Magenta
            AppTheme::EmeraldMatrix => Color::Rgb(0, 150, 70),       // Dark Green
            AppTheme::CelestialMidnight => Color::Rgb(192, 192, 192),// Silver
        }
    }

    pub fn accent(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(255, 140, 0),     // Regal Orange
            AppTheme::CrimsonCodex => Color::Rgb(220, 80, 60),       // Burnt Sienna
            AppTheme::CyberScholar => Color::Rgb(180, 0, 255),       // Electric Violet
            AppTheme::EmeraldMatrix => Color::Rgb(0, 210, 230),      // Matrix Teal
            AppTheme::CelestialMidnight => Color::Rgb(186, 104, 200),// Nebula Orchid
        }
    }

    pub fn text(&self) -> Color {
        match self {
            AppTheme::RoyalByzantium => Color::Rgb(240, 235, 255),
            AppTheme::CrimsonCodex => Color::Rgb(255, 235, 220),
            AppTheme::CyberScholar => Color::Rgb(220, 245, 255),
            AppTheme::EmeraldMatrix => Color::Rgb(220, 255, 220),
            AppTheme::CelestialMidnight => Color::Rgb(230, 240, 255),
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
            AppTheme::CrimsonCodex => Style::default().fg(Color::Rgb(255, 235, 220)),
            AppTheme::CyberScholar => Style::default().fg(Color::Rgb(220, 245, 255)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(220, 255, 220)),
            AppTheme::CelestialMidnight => Style::default().fg(Color::Rgb(230, 240, 255)),
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
            AppTheme::CrimsonCodex => Style::default().fg(Color::Rgb(200, 120, 100)).add_modifier(Modifier::ITALIC),
            AppTheme::CyberScholar => Style::default().fg(Color::Rgb(220, 100, 180)).add_modifier(Modifier::ITALIC),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(100, 180, 100)).add_modifier(Modifier::ITALIC),
            AppTheme::CelestialMidnight => Style::default().fg(Color::Rgb(130, 170, 230)).add_modifier(Modifier::ITALIC),
        }
    }

    pub fn highlight_item(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn ascii_char_style(&self, ch: char) -> Style {
        match ch {
            '@' => Style::default().fg(self.primary()).add_modifier(Modifier::BOLD),
            '#' | '%' => Style::default().fg(self.primary()),
            '+' | '*' => Style::default().fg(self.secondary()).add_modifier(Modifier::BOLD),
            '=' | '-' => Style::default().fg(self.accent()),
            ':' | '.' | ' ' => match self {
                AppTheme::RoyalByzantium => Style::default().fg(Color::Rgb(55, 45, 75)),
                AppTheme::CrimsonCodex => Style::default().fg(Color::Rgb(65, 30, 35)),
                AppTheme::CyberScholar => Style::default().fg(Color::Rgb(30, 50, 70)),
                AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(25, 55, 35)),
                AppTheme::CelestialMidnight => Style::default().fg(Color::Rgb(35, 45, 70)),
            },
            _ => Style::default().fg(self.text()),
        }
    }
}
