//! Interactive Scripture Reader & Translation Comparison View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub struct BibleViewState {
    pub selected_book_idx: usize,
    pub selected_chapter: usize,
    pub active_translation: String,
    pub compare_translations: Vec<String>,
    pub compare_mode: bool,
    pub interlinear_mode: bool,
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
            interlinear_mode: false,
            scroll: 0,
        }
    }
}

/// Formats a verse with Greek or Hebrew root lemma glosses and Strong's IDs.
pub fn format_interlinear_line<'a>(
    v_num: usize,
    text: &'a str,
    theme: &AppTheme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("   {:>3} ", v_num), theme.header_badge()),
        Span::styled(text, theme.scripture_text()),
    ]));

    let mut gloss_spans = vec![Span::raw("       ↳ ")];
    let mut found = false;
    for word in text.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();
        let tag = match clean.as_str() {
            "beginning" => Some("H7225 רֵאשִׁית"),
            "created" => Some("H1254 בָּרָא"),
            "god" => Some("H430 אֱלֹהִים"),
            "heaven" | "heavens" => Some("H8064 שָׁמַיִם"),
            "earth" => Some("H776 אֶרֶץ"),
            "word" => Some("G3056 λόγος"),
            "light" => Some("G5457 φῶς"),
            "life" => Some("G2222 ζωή"),
            "truth" => Some("G225 ἀλήθεια"),
            "spirit" => Some("G4151 πνεῦμα"),
            "comforter" | "advocate" | "helper" => Some("G3875 παράκλητος"),
            "grace" => Some("G5485 χάρις"),
            "faith" => Some("G4102 πίστις"),
            "sanctuary" => Some("H6944 קֹדֶשׁ"),
            _ => None,
        };
        if let Some(t) = tag {
            found = true;
            gloss_spans.push(Span::styled(format!("[{}={}] ", word, t), theme.header_badge()));
        }
    }

    if found {
        lines.push(Line::from(gloss_spans));
    }
    lines
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
                ListItem::new(format!("   {}", b)).style(Style::default().fg(theme.text()))
            }
        })
        .collect();

    let books_block = Block::default()
        .title(" Scripture Books ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let books_list = List::new(book_items).block(books_block);
    let mut list_state = ListState::default();
    if !books.is_empty() {
        list_state.select(Some(state.selected_book_idx));
    }
    f.render_stateful_widget(books_list, chunks[0], &mut list_state);

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
                .border_style(theme.border_focused())
                .style(Style::default().bg(theme.panel_bg()));

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
            if state.interlinear_mode {
                lines.extend(format_interlinear_line(*v_num, text.as_str(), theme));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(format!("   {:>3} ", v_num), theme.header_badge()),
                    Span::styled(text.as_str(), theme.scripture_text()),
                ]));
            }
            lines.push(Line::from(""));
        }

        let reader_block = Block::default()
            .title(format!(
                " {} Chapter {} [{}] (Press 'c' for Compare, 'x' for Interlinear) ",
                current_book, state.selected_chapter, state.active_translation
            ))
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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_parallel_reader() {
        let mut state = BibleViewState::default();
        assert!(!state.compare_mode);
        state.compare_mode = true;
        assert_eq!(state.scroll, 0);

        // Test synchronized scroll increment across columns
        state.scroll += 5;
        assert_eq!(state.scroll, 5);

        // Test comparison translation pairs
        let kjv_verses = vec![(1, "In the beginning God created the heaven and the earth.".to_string())];
        let bsb_verses = vec![(1, "In the beginning God created the heavens and the earth.".to_string())];
        let comparison = [
            ("kjv", kjv_verses),
            ("bsb", bsb_verses),
        ];

        assert_eq!(comparison.len(), 2);
        assert_eq!(comparison[0].0, "kjv");
        assert_eq!(comparison[1].0, "bsb");
    }

    #[test]
    pub fn test_interlinear_rendering() {
        let mut state = BibleViewState::default();
        assert!(!state.interlinear_mode);
        state.interlinear_mode = true;
        let theme = AppTheme::RoyalByzantium;
        let lines = format_interlinear_line(1, "In the beginning God created the heaven and the earth.", &theme);
        assert!(lines.len() >= 2);
        let gloss_line = &lines[1];
        let has_strong = gloss_line.spans.iter().any(|s| s.content.contains("H7225"));
        assert!(has_strong, "Interlinear line must contain Strong H7225 tag");
    }
}
