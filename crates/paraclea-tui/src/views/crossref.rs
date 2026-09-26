//! Knowledge Graph & Cross-Reference View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

#[derive(Default)]
pub struct CrossrefViewState {
    pub selected_idx: usize,
    pub query: String,
    pub scroll: usize,
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
    let list_items: Vec<ListItem> = if nodes.is_empty() {
        vec![ListItem::new("   No knowledge nodes stored").style(Style::default().fg(theme.text()))]
    } else {
        nodes
            .iter()
            .enumerate()
            .map(|(idx, (_id, title, _content, ntype))| {
                if idx == state.selected_idx {
                    ListItem::new(format!(" ▶ [{}] {}", ntype, title)).style(theme.highlight_item())
                } else {
                    ListItem::new(format!("   [{}] {}", ntype, title)).style(Style::default().fg(theme.text()))
                }
            })
            .collect()
    };

    let list_block = Block::default()
        .title(" 🧬 Knowledge Nodes & Cross-Refs ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));
    let list = List::new(list_items).block(list_block);
    let mut list_state = ListState::default();
    if !nodes.is_empty() {
        list_state.select(Some(state.selected_idx));
    }
    f.render_stateful_widget(list, chunks[0], &mut list_state);

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
            Span::styled("   No cross-reference nodes created yet.", Style::default().fg(theme.text())),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   Create one in prompt via: /crossref Genesis 1:1 <-> Natural Philosophy Ch 1 Study Notes", theme.header_badge()),
        ]));
    }

    let detail_block = Block::default()
        .title(" 🔍 Node Association & Linkage Details ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let p = Paragraph::new(lines)
        .block(detail_block)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll as u16, 0));

    f.render_widget(p, chunks[1]);
}
