//! LoRa RNode Packet Monitor & Signal Oscilloscope View for Paraclea TUI.
//!
//! Displays real-time RF signal metrics (RSSI, SNR), hop count telemetry,
//! and raw packet traces from connected Reticulum LoRa RNode interfaces.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::theme::AppTheme;

/// Telemetry metrics for a received or transmitted LoRa packet.
#[derive(Debug, Clone, PartialEq)]
pub struct PacketTelemetry {
    pub timestamp: String,
    pub packet_type: String, // "ANNOUNCE", "LINK", "DATA", "PROOF"
    pub rssi: i16,           // dBm (e.g. -85)
    pub snr: f32,            // dB (e.g. 7.5)
    pub hops: u8,
    pub sender_hash: String,
    pub payload_bytes: usize,
}

impl PacketTelemetry {
    pub fn new(
        timestamp: impl Into<String>,
        packet_type: impl Into<String>,
        rssi: i16,
        snr: f32,
        hops: u8,
        sender_hash: impl Into<String>,
        payload_bytes: usize,
    ) -> Self {
        Self {
            timestamp: timestamp.into(),
            packet_type: packet_type.into(),
            rssi,
            snr,
            hops,
            sender_hash: sender_hash.into(),
            payload_bytes,
        }
    }
}

/// Generates an ASCII bar for RSSI signal strength (-120 dBm to -40 dBm).
pub fn render_rssi_bar(rssi: i16) -> &'static str {
    if rssi >= -60 {
        "████████ (Excellent)"
    } else if rssi >= -75 {
        "██████░░ (Good)"
    } else if rssi >= -90 {
        "████░░░░ (Fair)"
    } else if rssi >= -105 {
        "██░░░░░░ (Weak)"
    } else {
        "█░░░░░░░ (Fringe)"
    }
}

/// Generates an ASCII bar for SNR (-10 dB to +15 dB).
pub fn render_snr_bar(snr: f32) -> &'static str {
    if snr >= 10.0 {
        "▲▲▲▲▲ (+Clean)"
    } else if snr >= 5.0 {
        "▲▲▲▲░ (+Nominal)"
    } else if snr >= 0.0 {
        "▲▲▲░░ (+Above Noise)"
    } else if snr >= -5.0 {
        "▲▲░░░ (-Sub-Noise)"
    } else {
        "▲░░░░ (-Extreme Deep)"
    }
}

/// Renders the complete Packet Monitor view.
pub fn render_packets_view(
    f: &mut Frame,
    area: Rect,
    packets: &[PacketTelemetry],
    interface_name: &str,
    theme: &AppTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Min(8),
        ])
        .split(area);

    // 1. Header: Interface State & RF Parameters
    let header_lines = vec![
        Line::from(vec![
            Span::styled(" 📡 LoRa RNode Packet Monitor & RF Oscilloscope ", theme.header_title()),
            Span::styled(format!(" [Interface: {}] ", interface_name), theme.header_badge()),
        ]),
        Line::from(vec![
            Span::styled("   RF Params: ", theme.header_badge()),
            Span::raw("915.000 MHz • BW 125 kHz • SF7 • CR 4/5 • TxPower 20 dBm • CRC ON"),
        ]),
    ];
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_focused())
        .style(Style::default().bg(theme.panel_bg()));
    f.render_widget(Paragraph::new(header_lines).block(header_block), chunks[0]);

    // 2. Oscilloscope / Real-Time RF Signal Meter
    let latest = packets.last();
    let (rssi_val, snr_val) = latest.map(|p| (p.rssi, p.snr)).unwrap_or((-92, 4.2));

    let scope_lines = vec![
        Line::from(vec![
            Span::styled("   RSSI (Signal Strength): ", theme.header_badge()),
            Span::styled(format!("{:>4} dBm  ", rssi_val), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(render_rssi_bar(rssi_val), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("   SNR  (Signal-to-Noise): ", theme.header_badge()),
            Span::styled(format!("{:>+5.1} dB   ", snr_val), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(render_snr_bar(snr_val), Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   Waveform Visualizer: ", theme.header_badge()),
            Span::styled("  ⡀⡄⡆⡇⣇⣗⣵⣶⣿⣷⣵⣗⣇⡇⡆⡄⡀⣀⣀⡀⡄⡆⡇⣇⣗⣵⣶⣿⣷⣵⣗⣇⡇⡆⡄⡀ ", Style::default().fg(Color::Yellow)),
        ]),
    ];
    let scope_block = Block::default()
        .title(" 📊 RF Carrier Signal Oscilloscope ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_normal())
        .style(Style::default().bg(theme.panel_bg()));
    f.render_widget(Paragraph::new(scope_lines).block(scope_block), chunks[1]);

    // 3. Live Packet Trace Log Table
    let mut trace_lines = Vec::new();
    trace_lines.push(Line::from(vec![
        Span::styled(
            format!(
                "   {:<10} {:<10} {:>8} {:>8} {:>6} {:<18} {:>8}",
                "TIME", "TYPE", "RSSI", "SNR", "HOPS", "SENDER", "PAYLOAD"
            ),
            theme.header_title(),
        ),
    ]));
    trace_lines.push(Line::from("   -----------------------------------------------------------------------------"));

    if packets.is_empty() {
        trace_lines.push(Line::from(vec![
            Span::styled("   Waiting for incoming RNode packets... (listening on 915.000 MHz)", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for p in packets.iter().rev().take(12) {
            let color = match p.packet_type.as_str() {
                "ANNOUNCE" => Color::Cyan,
                "LINK" => Color::Green,
                "DATA" => Color::Yellow,
                _ => Color::Magenta,
            };

            trace_lines.push(Line::from(vec![
                Span::raw(format!("   {:<10} ", p.timestamp)),
                Span::styled(format!("{:<10} ", p.packet_type), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:>5} dBm ", p.rssi)),
                Span::raw(format!("{:>+5.1} dB ", p.snr)),
                Span::raw(format!("{:>4}h  ", p.hops)),
                Span::styled(format!("<{}> ", &p.sender_hash[..p.sender_hash.len().min(14)]), Style::default().fg(Color::White)),
                Span::raw(format!("{:>5} B", p.payload_bytes)),
            ]));
        }
    }

    let log_block = Block::default()
        .title(" 📋 Raw Encrypted Packet Trace ")
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_normal())
        .style(Style::default().bg(theme.panel_bg()));
    f.render_widget(Paragraph::new(trace_lines).block(log_block).wrap(Wrap { trim: true }), chunks[2]);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_packet_telemetry() {
        let p = PacketTelemetry::new(
            "14:22:01",
            "ANNOUNCE",
            -82,
            6.5,
            1,
            "4a7bc9e120f3a5b2",
            144,
        );

        assert_eq!(p.packet_type, "ANNOUNCE");
        assert_eq!(p.rssi, -82);
        assert_eq!(p.hops, 1);

        let rssi_str = render_rssi_bar(p.rssi);
        assert!(rssi_str.contains("Fair") || rssi_str.contains("Good"));

        let snr_str = render_snr_bar(p.snr);
        assert!(snr_str.contains("Nominal") || snr_str.contains("Clean"));
    }
}
