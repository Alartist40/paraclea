//! Knowledge Graph & Cross-Reference View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub struct CrossrefViewState {
    pub selected_idx: usize,
    pub query: String,
    pub scroll: usize,
}

impl Default for CrossrefViewState {
    fn default() -> Self {
        Self {
            selected_idx: 0,
            query: String::new(),
            scroll: 0,
        }
    }
}

pub fn render_crossref_view(
    f: &mut Frame,
    area: Rect,
    state: &CrossrefViewState,
    nodes: &[(String, String, String, String)], // (id, title, content, type)
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(40)])
        .split(area);

    // Left List: Knowledge Nodes & Cross-References
    let list_items: Vec<ListItem> = nodes
        .iter()
        .enumerate()
        .map(|(idx, (_id, title, _content, ntype))| {
            if idx == state.selected_idx {
                ListItem::new(format!(" ▶ [{}] {}", ntype, title)).style(theme.highlight_item())
            } else {
                ListItem::new(format!("   [{}] {}", ntype, title)).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let list_block = Block::default()
        .title(" 🧬 Knowledge Nodes & Cross-Refs ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let list = List::new(list_items).block(list_block);
    f.render_widget(list, chunks[0]);

    // Right Details View
    let mut lines = Vec::new();
    if let Some((_id, title, content, ntype)) = nodes.get(state.selected_idx) {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!("   === {} [{}] ===", title, ntype), theme.header_title()),
        ]));
        lines.push(Line::from(""));

        for line in content.lines() {
            if line.contains("[[") && line.contains("]]") {
                lines.push(Line::from(vec![
                    Span::styled(format!("   🔗 {}", line), theme.header_badge()),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(format!("   {}", line), theme.scripture_text()),
                ]));
            }
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("   No cross-reference nodes created yet.", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   Create one in prompt via: /crossref Genesis 1:1 <-> Natural Philosophy Ch 1 Study Notes", theme.header_badge()),
        ]));
    }

    let detail_block = Block::default()
        .title(" 🔍 Node Association & Linkage Details ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let p = Paragraph::new(lines)
        .block(detail_block)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll as u16, 0));

    f.render_widget(p, chunks[1]);
}
