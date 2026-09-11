//! Interactive Scripture Reader & Translation Comparison View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub struct BibleViewState {
    pub selected_book_idx: usize,
    pub selected_chapter: usize,
    pub active_translation: String,
    pub compare_translations: Vec<String>,
    pub compare_mode: bool,
    pub scroll: usize,
}

impl Default for BibleViewState {
    fn default() -> Self {
        Self {
            selected_book_idx: 0,
            selected_chapter: 1,
            active_translation: "KJV".to_string(),
            compare_translations: vec!["kjv".to_string(), "bsb".to_string(), "web".to_string()],
            compare_mode: false,
            scroll: 0,
        }
    }
}

pub fn render_bible_view(
    f: &mut Frame,
    area: Rect,
    state: &BibleViewState,
    books: &[String],
    verses: &[(usize, String)],
    comparison_verses: &[(&str, Vec<(usize, String)>)],
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(40)])
        .split(area);

    // Left Column: Book Selector List
    let book_items: Vec<ListItem> = books
        .iter()
        .enumerate()
        .map(|(idx, b)| {
            if idx == state.selected_book_idx {
                ListItem::new(format!(" ▶ {}", b)).style(theme.highlight_item())
            } else {
                ListItem::new(format!("   {}", b)).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let books_block = Block::default()
        .title(" 📖 Scripture Books ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let books_list = List::new(book_items).block(books_block);
    f.render_widget(books_list, chunks[0]);

    // Right Column: Scripture Text or Comparison Grid
    let current_book = books.get(state.selected_book_idx).cloned().unwrap_or_else(|| "Genesis".to_string());

    if state.compare_mode && !comparison_verses.is_empty() {
        // Multi-column side-by-side comparison
        let num_cols = comparison_verses.len().max(1);
        let col_constraints = vec![Constraint::Ratio(1, num_cols as u32); num_cols];
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(chunks[1]);

        for (col_idx, (trans_tag, col_v_list)) in comparison_verses.iter().enumerate() {
            let mut lines = Vec::new();
            for (v_num, text) in col_v_list {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:>3} ", v_num), theme.header_badge()),
                    Span::styled(text.as_str(), theme.scripture_text()),
                ]));
                lines.push(Line::from(""));
            }

            let col_block = Block::default()
                .title(format!(" {} (Ch {}) — {} ", current_book, state.selected_chapter, trans_tag.to_uppercase()))
                .borders(Borders::ALL)
                .border_type(theme.border_type())
                .border_style(theme.border_focused());

            let p = Paragraph::new(lines)
                .block(col_block)
                .wrap(Wrap { trim: true })
                .scroll((state.scroll as u16, 0));
            f.render_widget(p, col_chunks[col_idx]);
        }
    } else {
        // Standard Chapter Reader
        let mut lines = Vec::new();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!("   === {} Chapter {} ({}) ===", current_book, state.selected_chapter, state.active_translation), theme.header_title()),
        ]));
        lines.push(Line::from(""));

        for (v_num, text) in verses {
            lines.push(Line::from(vec![
                Span::styled(format!("   {:>3} ", v_num), theme.header_badge()),
                Span::styled(text.as_str(), theme.scripture_text()),
            ]));
            lines.push(Line::from(""));
        }

        let reader_block = Block::default()
            .title(format!(" 📜 {} Chapter {} [Translation: {}] (Press 'c' for Compare View) ", current_book, state.selected_chapter, state.active_translation))
            .borders(Borders::ALL)
            .border_type(theme.border_type())
            .border_style(theme.border_focused());

        let p = Paragraph::new(lines)
            .block(reader_block)
            .wrap(Wrap { trim: true })
            .scroll((state.scroll as u16, 0));

        f.render_widget(p, chunks[1]);
    }
}
