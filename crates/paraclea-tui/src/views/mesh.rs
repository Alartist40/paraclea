//! Reticulum Off-Grid Mesh Deck View for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

#[derive(Default)]
pub struct MeshViewState {
    pub selected_tab: usize, // 0: Status, 1: Peers, 2: Mailbox
    pub scroll: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn render_mesh_view(
    f: &mut Frame,
    area: Rect,
    _state: &MeshViewState,
    status_text: &str,
    identity_hash: Option<&str>,
    peers: &[String],
    messages: &[(String, String, String, String)], // (timestamp, sender, recipient, content)
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(10)])
        .split(area);

    // Top Header: Reticulum Network Identity & Mode
    let mut header_lines = Vec::new();
    header_lines.push(Line::from(vec![
        Span::styled(" 🕸 RETICULUM OFF-GRID MESH PROTOCOL (RNS) — PURE RUST / NATIVE STACK", theme.header_title()),
    ]));
    header_lines.push(Line::from(vec![
        Span::styled("   Local Identity Hash: ", theme.header_badge()),
        Span::styled(format!("<{}>", identity_hash.unwrap_or("unassigned_hash")), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  •  Mode: "),
        Span::styled("Ad-hoc P2P Off-Grid Node", Style::default().fg(Color::Green)),
    ]));

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let p_header = Paragraph::new(header_lines).block(header_block);
    f.render_widget(p_header, chunks[0]);

    // Bottom Split: Left Peers & Status, Right Mailbox Messages
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Left: Status & Peer Discovered Links
    let mut status_lines = Vec::new();
    for line in status_text.lines() {
        status_lines.push(Line::from(vec![
            Span::styled(format!("   {}", line), theme.scripture_text()),
        ]));
    }
    status_lines.push(Line::from(""));
    status_lines.push(Line::from(vec![
        Span::styled("   Discovered Mesh Peers:", theme.header_title()),
    ]));
    if peers.is_empty() {
        status_lines.push(Line::from(vec![
            Span::styled("     No active peer nodes detected in broadcast range.", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for p in peers {
            status_lines.push(Line::from(vec![
                Span::styled(format!("     • {}", p), Style::default().fg(Color::Cyan)),
            ]));
        }
    }

    let left_block = Block::default()
        .title(" 📡 Link Interface & Propagation Status ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let p_status = Paragraph::new(status_lines).block(left_block).wrap(Wrap { trim: true });
    f.render_widget(p_status, bottom_chunks[0]);

    // Right: Off-Grid Mailbox Inbox
    let mut mail_items = Vec::new();
    if messages.is_empty() {
        mail_items.push(ListItem::new("   No stored packets in local mailbox.").style(Style::default().fg(Color::DarkGray)));
    } else {
        for (ts, sender, recip, content) in messages {
            mail_items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" [{}] From: <{}> -> To: <{}>", ts, sender, recip), theme.header_badge()),
                ]),
                Line::from(vec![
                    Span::styled(format!("   {}", content), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
            ]));
        }
    }

    let mail_block = Block::default()
        .title(" 📬 Store-and-Forward Encrypted Mailbox ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let mail_list = List::new(mail_items).block(mail_block);
    f.render_widget(mail_list, bottom_chunks[1]);
}
