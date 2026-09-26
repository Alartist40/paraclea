//! Multi-Category Library Browser & Reader View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub struct LibraryViewState {
    pub selected_category_idx: usize,
    pub selected_book_idx: usize,
    pub selected_chapter: usize,
    pub scroll: usize,
}

impl Default for LibraryViewState {
    fn default() -> Self {
        Self {
            selected_category_idx: 0,
            selected_book_idx: 0,
            selected_chapter: 1,
            scroll: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_library_view(
    f: &mut Frame,
    area: Rect,
    state: &LibraryViewState,
    categories: &[String],
    books: &[String],
    chapter_title: &str,
    chapter_content: &str,
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(40)])
        .split(area);

    // Left Column: Category & Book List
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(chunks[0]);

    // Categories List
    let cat_items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(idx, cat)| {
            if idx == state.selected_category_idx {
                ListItem::new(format!(" ▶ [{}]", cat.to_uppercase())).style(theme.highlight_item())
            } else {
                ListItem::new(format!("   [{}]", cat.to_uppercase())).style(Style::default().fg(theme.secondary()))
            }
        })
        .collect();

    let cat_block = Block::default()
        .title(" 📚 Domain Categories ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));
    let cat_list = List::new(cat_items).block(cat_block);
    let mut cat_state = ListState::default();
    if !categories.is_empty() {
        cat_state.select(Some(state.selected_category_idx));
    }
    f.render_stateful_widget(cat_list, left_chunks[0], &mut cat_state);

    // Books in Category
    let book_items: Vec<ListItem> = if books.is_empty() {
        vec![ListItem::new("   No volumes in category").style(Style::default().fg(theme.text()))]
    } else {
        books
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                if idx == state.selected_book_idx {
                    ListItem::new(format!(" ▶ {}", b)).style(theme.highlight_item())
                } else {
                    ListItem::new(format!("   {}", b)).style(Style::default().fg(theme.text()))
                }
            })
            .collect()
    };

    let books_block = Block::default()
        .title(" 📖 Ingested Volumes ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));
    let books_list = List::new(book_items).block(books_block);
    let mut book_state = ListState::default();
    if !books.is_empty() {
        book_state.select(Some(state.selected_book_idx));
    }
    f.render_stateful_widget(books_list, left_chunks[1], &mut book_state);

    // Right Column: Chapter Reader View
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("   === {} (Chapter {}) ===", chapter_title, state.selected_chapter), theme.header_title()),
    ]));
    lines.push(Line::from(""));

    for line in chapter_content.lines() {
        lines.push(Line::from(vec![
            Span::styled(format!("   {}", line), theme.scripture_text()),
        ]));
    }

    let reader_block = Block::default()
        .title(format!(" 📑 Volume Reader — {} (Press 's' for AI Study Commentary) ", chapter_title))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let p = Paragraph::new(lines)
        .block(reader_block)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll as u16, 0));

    f.render_widget(p, chunks[1]);
}
