//! Modal Dialogs for Paraclea TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
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
    let modal_area = centered_rect(65, 75, area);
    f.render_widget(Clear, modal_area);

    let text = vec![
        Line::from(vec![
            Span::styled("PARACLEA SCHOLAR TUI — KEYBINDINGS & CHEATSHEET", theme.header_title()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab / Shift+Tab  ", theme.header_badge()),
            Span::styled("Cycle active pane (Main Viewport ↔ Prompt ↔ Sidebar)", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("1 – 7            ", theme.header_badge()),
            Span::styled("Switch decks in Viewport/Sidebar (or Alt+1–7 in Prompt)", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("i / Enter / /    ", theme.header_badge()),
            Span::styled("Focus bottom command & study prompt", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Esc              ", theme.header_badge()),
            Span::styled("Cancel streaming or return focus to Viewport", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + B         ", theme.header_badge()),
            Span::styled("Toggle left sidebar visibility", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + T         ", theme.header_badge()),
            Span::styled("Cycle color themes (5 presets)", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + P         ", theme.header_badge()),
            Span::styled("Quick Translation & Language Picker modal", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + M         ", theme.header_badge()),
            Span::styled("Active Ollama Model Selector modal", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + U         ", theme.header_badge()),
            Span::styled("Trigger 1-Click Encrypted USB Backup", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("/                ", theme.header_badge()),
            Span::styled("Focus prompt & open Slash Command Palette", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Up / Down / PgUp ", theme.header_badge()),
            Span::styled("Scroll active viewport / navigate lists & history", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("Esc              ", theme.header_badge()),
            Span::styled("Close active modal, cancel generation, or return focus to Chat", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [Esc] or [Enter] to dismiss this dialog", Style::default().fg(Color::Rgb(160, 160, 180)).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let block = Block::default()
        .title(" 💡 Keyboard Shortcuts & Help ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

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
    let modal_area = centered_rect(65, 70, area);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(modal_area);

    // Search bar
    let search_title = if filter.is_empty() {
        format!(" {} ({} items) ", title, items.len())
    } else {
        format!(" {} (Matches: {}) ", title, items.len())
    };

    let search_block = Block::default()
        .title(search_title)
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let search_line = Line::from(vec![
        Span::styled(" Filter > ", theme.header_badge()),
        Span::styled(filter, Style::default().fg(theme.text()).add_modifier(Modifier::BOLD)),
        Span::styled("█", theme.header_title()),
    ]);
    let search_p = Paragraph::new(search_line).block(search_block);
    f.render_widget(search_p, chunks[0]);

    // List of items
    let list_items: Vec<ListItem> = if items.is_empty() {
        vec![ListItem::new("   No matching items found.").style(Style::default().fg(Color::Rgb(160, 160, 180)).add_modifier(Modifier::ITALIC))]
    } else {
        items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                if idx == selected_idx {
                    ListItem::new(format!(" ▶ {}", item)).style(theme.highlight_item())
                } else {
                    ListItem::new(format!("   {}", item)).style(Style::default().fg(theme.text()))
                }
            })
            .collect()
    };

    let list_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let list = List::new(list_items).block(list_block);
    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(selected_idx));
    }
    f.render_stateful_widget(list, chunks[1], &mut state);
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
            Span::styled(display_value, Style::default().fg(theme.text())),
            Span::styled("█", theme.header_title()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [Enter] to Confirm, [Esc] to Cancel", Style::default().fg(Color::Rgb(160, 160, 180)).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

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
    let modal_area = centered_rect(65, 65, area);
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
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let search_line = Line::from(vec![
        Span::styled(" Command > ", theme.header_badge()),
        Span::styled(filter, Style::default().fg(theme.text()).add_modifier(Modifier::BOLD)),
        Span::styled("█", theme.header_title()),
    ]);
    let search_p = Paragraph::new(search_line).block(search_block);
    f.render_widget(search_p, chunks[0]);

    // List of commands
    let list_items: Vec<ListItem> = if commands.is_empty() {
        vec![ListItem::new("   No matching commands found.").style(Style::default().fg(Color::Rgb(160, 160, 180)).add_modifier(Modifier::ITALIC))]
    } else {
        commands
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
                        Span::styled(format!("— {}", desc), Style::default().fg(theme.text())),
                    ]))
                }
            })
            .collect()
    };

    let list_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));

    let list = List::new(list_items).block(list_block);
    let mut state = ListState::default();
    if !commands.is_empty() {
        state.select(Some(selected_idx));
    }
    f.render_stateful_widget(list, chunks[1], &mut state);
}



