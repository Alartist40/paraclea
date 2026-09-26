//! AI Chat & Commentary View for Paraclea TUI.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::ascii_art::{render_ascii_line, PARACLEA_BANNER};
use crate::theme::AppTheme;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub timestamp: String,
}

pub fn build_chat_lines(
    history: &[ChatMessage],
    streaming_text: &str,
    is_streaming: bool,
    width: u16,
    theme: &AppTheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if history.is_empty() && !is_streaming {
        lines.push(Line::from(""));
        let indent = (width.saturating_sub(60)) as usize / 2;
        for banner_line in PARACLEA_BANNER.lines() {
            lines.push(render_ascii_line(banner_line, indent, theme));
        }
        lines.push(Line::from(""));
        let pad = " ".repeat(indent.max(2));
        lines.push(Line::from(vec![
            Span::raw(pad.clone()),
            Span::styled("PARACLEA SCHOLAR AI — Multi-Lingual Scripture & Knowledge Assistant", theme.header_title()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw(pad.clone()),
            Span::styled("• Type any question or scripture reference to begin study.", Style::default().fg(Color::Rgb(180, 180, 200))),
        ]));
        lines.push(Line::from(vec![
            Span::raw(pad.clone()),
            Span::styled("• Type ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("/bible", theme.header_title()),
            Span::styled(" or ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("/compare", theme.header_title()),
            Span::styled(" to read & cross-examine across available translations.", Style::default().fg(Color::Rgb(180, 180, 200))),
        ]));
        lines.push(Line::from(vec![
            Span::raw(pad.clone()),
            Span::styled("• Type ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("/library", theme.header_title()),
            Span::styled(" to explore Psychology, Survival, Medical, EGW, and Philosophy.", Style::default().fg(Color::Rgb(180, 180, 200))),
        ]));
        lines.push(Line::from(vec![
            Span::raw(pad),
            Span::styled("• Press ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("1 – 7", theme.header_badge()),
            Span::styled(" to switch decks ([1]Chat [2]Bible [3]Library [4]Crossref [5]Galaxy [6]Mesh [7]Doctor) | ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("i / Enter", theme.header_title()),
            Span::styled(": Focus Prompt | ", Style::default().fg(Color::Rgb(180, 180, 200))),
            Span::styled("Tab", theme.header_title()),
            Span::styled(": Cycle Focus.", Style::default().fg(Color::Rgb(180, 180, 200))),
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
                Span::styled(format!("   {}", msg.content), Style::default().fg(theme.text())),
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
                if line_content.starts_with('>') || line_content.contains("Genesis") || line_content.contains("John") || line_content.contains("Revelation") {
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
            Span::styled(" 🕊️ Paraclea [Generating... Press Esc to Cancel]", theme.assistant_header()),
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

    lines
}

pub fn compute_chat_total_rows(lines: &[Line], inner_width: usize) -> usize {
    let mut total_rows = 0;
    for line in lines {
        let line_len: usize = line.spans.iter().map(|s| s.content.width()).sum();
        let rows = if line_len == 0 { 1 } else { line_len.div_ceil(inner_width) };
        total_rows += rows;
    }
    total_rows
}

/// Maximum scroll offset (bottom anchor) for a chat view of the given size.
/// Key/mouse handlers use this so their clamps match what the renderer draws.
pub fn chat_max_scroll(
    history: &[ChatMessage],
    streaming_text: &str,
    is_streaming: bool,
    area_width: u16,
    area_height: u16,
    theme: &AppTheme,
) -> usize {
    let lines = build_chat_lines(history, streaming_text, is_streaming, area_width, theme);
    let inner_width = area_width.saturating_sub(2).max(1) as usize;
    let inner_height = area_height.saturating_sub(2) as usize;
    compute_chat_total_rows(&lines, inner_width).saturating_sub(inner_height)
}

#[allow(clippy::too_many_arguments)]
pub fn render_chat_view(
    f: &mut Frame,
    area: Rect,
    history: &[ChatMessage],
    streaming_text: &str,
    is_streaming: bool,
    is_speaking: bool,
    scroll: usize,
    auto_scroll: bool,
    theme: &AppTheme,
) {
    let lines = build_chat_lines(history, streaming_text, is_streaming, area.width, theme);
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let inner_height = area.height.saturating_sub(2) as usize;

    let total_rows = compute_chat_total_rows(&lines, inner_width);
    let max_scroll = total_rows.saturating_sub(inner_height);
    let effective_scroll = if auto_scroll {
        max_scroll
    } else {
        scroll.min(max_scroll)
    };

    let status_str = if is_speaking { " 🔊 Speaking Voice Output... " } else { "" };
    let block = Block::default()
        .title(format!(" 💬 Conversation & AI Study Commentary{} ", status_str))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((effective_scroll as u16, 0));

    f.render_widget(p, area);
}
