//! System Doctor & Hardware Telemetry View for Paraclea TUI.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

pub fn render_doctor_view(
    f: &mut Frame,
    area: Rect,
    ollama_ok: bool,
    ollama_model: &str,
    qdrant_ok: bool,
    dendrite_count: usize,
    bible_langs: usize,
    bible_versions: usize,
    backup_status: Option<&str>,
    theme: &AppTheme,
) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("   ╔══════════════════════════════════════════════════════════════╗", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   ║     ", theme.header_badge()),
        Span::styled("PARACLEA AI ASSISTANT — SYSTEM DOCTOR & INTEGRITY", theme.header_title()),
        Span::styled("     ║", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   ╚══════════════════════════════════════════════════════════════╝", theme.header_badge()),
    ]));
    lines.push(Line::from(""));

    // Hardware
    lines.push(Line::from(vec![
        Span::styled("   💻 System Target & CPU Topology:", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • OS: "),
        Span::styled(std::env::consts::OS, Style::default().fg(Color::Cyan)),
        Span::raw("  • Architecture: "),
        Span::styled(std::env::consts::ARCH, Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));

    // Ollama Server
    lines.push(Line::from(vec![
        Span::styled("   🤖 Ollama LLM Inference Engine:", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Server Connectivity: "),
        if ollama_ok {
            Span::styled("ONLINE (http://localhost:11434)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("OFFLINE (http://localhost:11434)", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        },
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Active Selected Model: "),
        Span::styled(ollama_model, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Qdrant
    lines.push(Line::from(vec![
        Span::styled("   ⚡ Vector Database Engine:", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Qdrant Vector Service: "),
        if qdrant_ok {
            Span::styled("ONLINE", Style::default().fg(Color::Green))
        } else {
            Span::styled("STANDBY / LOCAL RAG", Style::default().fg(Color::Yellow))
        },
    ]));
    lines.push(Line::from(""));

    // Dendrite Knowledge Graph
    lines.push(Line::from(vec![
        Span::styled("   🧬 Dendrite v2 Knowledge Graph:", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • SQLite Database Storage: "),
        Span::styled("~/.paraclea/dendrite.db (ONLINE & HEALTHY)", Style::default().fg(Color::Green)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Stored Knowledge Nodes: "),
        Span::styled(format!("{}", dendrite_count), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Bible Database
    lines.push(Line::from(vec![
        Span::styled("   📚 Scripture & Multi-Category Library Database:", theme.header_badge()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Bible Languages Covered: "),
        Span::styled(format!("{}", bible_langs), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("  • Formatted Translations: "),
        Span::styled(format!("{}", bible_versions), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      • Non-Scripture Categories: "),
        Span::styled("5 Active (Psychology, Survival, Medical, EGW, Educational)", Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));

    // Backup
    if let Some(bk) = backup_status {
        lines.push(Line::from(vec![
            Span::styled(format!("   🔒 Backup Action: {}", bk), Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("   [Press Ctrl+U to trigger 1-Click Encrypted USB Backup]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
    ]));

    let block = Block::default()
        .title(" 🩺 System Diagnostic & Self-Healing Telemetry ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused());

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
