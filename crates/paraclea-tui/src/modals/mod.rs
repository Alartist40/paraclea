//! Modal Dialogs for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_help_modal(f: &mut Frame, area: Rect, theme: &AppTheme) {
    let modal_area = centered_rect(65, 70, area);
    f.render_widget(Clear, modal_area);

    let text = vec![
        Line::from(vec![
            Span::styled("PARACLEA SCHOLAR TUI — KEYBINDINGS & CHEATSHEET", theme.header_title()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab / Shift+Tab  ", theme.header_badge()),
            Span::raw("Cycle active pane (Sidebar ↔ Main Viewport ↔ Prompt)"),
        ]),
        Line::from(vec![
            Span::styled("1 – 7            ", theme.header_badge()),
            Span::raw("Switch tabs: [1]Chat [2]Bible [3]Library [4]Crossref [5]Galaxy [6]Mesh [7]Doctor"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + B         ", theme.header_badge()),
            Span::raw("Toggle left sidebar visibility"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + T         ", theme.header_badge()),
            Span::raw("Cycle color themes (Royal Byzantium, Monastery, Cyber, Emerald)"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + P         ", theme.header_badge()),
            Span::raw("Quick Translation & Language Picker modal (160 versions)"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + M         ", theme.header_badge()),
            Span::raw("Active Ollama Model Selector modal"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + U         ", theme.header_badge()),
            Span::raw("Trigger 1-Click Encrypted USB Backup"),
        ]),
        Line::from(vec![
            Span::styled("/                ", theme.header_badge()),
            Span::raw("Focus prompt & open Slash Command Palette"),
        ]),
        Line::from(vec![
            Span::styled("Up / Down / PgUp ", theme.header_badge()),
            Span::raw("Scroll active viewport / navigate lists & history"),
        ]),
        Line::from(vec![
            Span::styled("Esc              ", theme.header_badge()),
            Span::raw("Close active modal or return focus to Chat"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [Esc] or [Enter] to dismiss this dialog", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let block = Block::default()
        .title(" 💡 Keyboard Shortcuts & Help ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, modal_area);
}

pub fn render_list_picker_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[String],
    selected_idx: usize,
    filter: &str,
    theme: &AppTheme,
) {
    let modal_area = centered_rect(60, 65, area);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(modal_area);

    // Search bar
    let search_block = Block::default()
        .title(format!(" {} (Filter: '{}') ", title, filter))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let search_p = Paragraph::new(format!("Search > {}█", filter)).block(search_block);
    f.render_widget(search_p, chunks[0]);

    // List of items
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            if idx == selected_idx {
                ListItem::new(format!(" ▶ {}", item)).style(theme.highlight_item())
            } else {
                ListItem::new(format!("   {}", item)).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let list = List::new(list_items).block(list_block);
    f.render_widget(list, chunks[1]);
}

pub fn render_input_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    prompt_label: &str,
    value: &str,
    is_secret: bool,
    theme: &AppTheme,
) {
    let modal_area = centered_rect(55, 30, area);
    f.render_widget(Clear, modal_area);

    let display_value = if is_secret {
        "•".repeat(value.len())
    } else {
        value.to_string()
    };

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{}: ", prompt_label), theme.header_badge()),
            Span::styled(display_value, Style::default().fg(Color::White)),
            Span::styled("█", theme.header_title()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [Enter] to Confirm, [Esc] to Cancel", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, modal_area);
}

pub fn render_command_palette_modal(
    f: &mut Frame,
    area: Rect,
    commands: &[(&str, &str)],
    selected_idx: usize,
    filter: &str,
    theme: &AppTheme,
) {
    let modal_area = centered_rect(65, 60, area);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(modal_area);

    // Search bar
    let search_block = Block::default()
        .title(" ⚡ Paraclea Slash Commands (/ dropdown) ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());
    let search_p = Paragraph::new(format!("Command > {}█", filter)).block(search_block);
    f.render_widget(search_p, chunks[0]);

    // List of commands
    let list_items: Vec<ListItem> = commands
        .iter()
        .enumerate()
        .map(|(idx, (cmd, desc))| {
            if idx == selected_idx {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" ▶ {:<14} ", cmd), theme.highlight_item()),
                    Span::styled(format!("— {}", desc), Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD)),
                ]))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("   {:<14} ", cmd), theme.header_badge()),
                    Span::styled(format!("— {}", desc), Style::default().fg(Color::Rgb(180, 180, 200))),
                ]))
            }
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let list = List::new(list_items).block(list_block);
    f.render_widget(list, chunks[1]);
}


