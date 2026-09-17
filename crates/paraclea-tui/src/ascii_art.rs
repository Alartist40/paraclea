//! Paraclea ASCII brand art rendering with theme integration and span coalescing.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::theme::AppTheme;

pub const PARACLEA_BANNER: &str = include_str!("../../../ascii-art-paraclea.txt");

/// Render a single line of ASCII art into a ratatui Line with span coalescing for contiguous identical styles.
pub fn render_ascii_line(line: &str, indent: usize, theme: &AppTheme) -> Line<'static> {
    if line.is_empty() {
        return Line::from("");
    }

    let mut spans = Vec::new();
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent)));
    }

    let mut current_chunk = String::new();
    let mut current_style: Option<Style> = None;

    for ch in line.chars() {
        let style = theme.ascii_char_style(ch);
        match current_style {
            Some(s) if s == style => {
                current_chunk.push(ch);
            }
            Some(s) => {
                spans.push(Span::styled(std::mem::take(&mut current_chunk), s));
                current_chunk.push(ch);
                current_style = Some(style);
            }
            None => {
                current_chunk.push(ch);
                current_style = Some(style);
            }
        }
    }

    if !current_chunk.is_empty() {
        if let Some(s) = current_style {
            spans.push(Span::styled(current_chunk, s));
        }
    }

    Line::from(spans)
}
