//! AI Chat & Commentary View for Paraclea TUI.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub timestamp: String,
}

pub fn render_chat_view(
    f: &mut Frame,
    area: Rect,
    history: &[ChatMessage],
    streaming_text: &str,
    is_streaming: bool,
    is_speaking: bool,
    scroll: usize,
    theme: &AppTheme,
) {
    let mut lines = Vec::new();

    if history.is_empty() && !is_streaming {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("   ╒═════════════════════════════════════════════════════════════════════════════╕", theme.header_badge()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   │ ", theme.header_badge()),
            Span::styled("PARACLEA SCHOLAR AI — Multi-Lingual Scripture & Knowledge Assistant", theme.header_title()),
            Span::styled("     │", theme.header_badge()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ╘═════════════════════════════════════════════════════════════════════════════╛", theme.header_badge()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("   • Type any question or scripture reference to begin study.", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   • Type ", Style::default().fg(Color::Gray)),
            Span::styled("/bible", theme.header_title()),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("/compare", theme.header_title()),
            Span::styled(" to read & cross-examine across 160 translations.", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   • Type ", Style::default().fg(Color::Gray)),
            Span::styled("/library", theme.header_title()),
            Span::styled(" to explore Psychology, Survival, Medical, EGW, and Philosophy.", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   • Press ", Style::default().fg(Color::Gray)),
            Span::styled("F1 – F6", theme.header_badge()),
            Span::styled(" to switch between dedicated interactive workspace decks.", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));
    }

    for msg in history {
        lines.push(Line::from(""));
        if msg.role == "user" {
            lines.push(Line::from(vec![
                Span::styled(format!(" 👤 You [{}]", msg.timestamp), theme.user_prompt()),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("   {}", msg.content), Style::default().fg(Color::White)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!(" 🕊️ Paraclea [{}]", msg.timestamp), theme.assistant_header()),
            ]));

            if let Some(ref think) = msg.thinking {
                lines.push(Line::from(vec![
                    Span::styled(format!("   💭 Thinking: {}", think), theme.thinking_block()),
                ]));
            }

            for line_content in msg.content.lines() {
                if line_content.starts_with(">") || line_content.contains("Genesis") || line_content.contains("John") || line_content.contains("Revelation") {
                    lines.push(Line::from(vec![
                        Span::styled(format!("   ┃ {}", line_content), theme.scripture_card()),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(format!("   {}", line_content), theme.scripture_text()),
                    ]));
                }
            }
        }
    }

    if is_streaming {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" 🕊️ Paraclea [Generating...]", theme.assistant_header()),
        ]));
        for line_content in streaming_text.lines() {
            lines.push(Line::from(vec![
                Span::styled(format!("   {}", line_content), theme.scripture_text()),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("   █", theme.header_title()),
        ]));
    }

    let status_str = if is_speaking { " 🔊 Speaking Voice Output... " } else { "" };
    let block = Block::default()
        .title(format!(" 💬 Conversation & AI Study Commentary{} ", status_str))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    f.render_widget(p, area);
}
