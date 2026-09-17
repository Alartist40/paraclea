//! Galaxy View rendering and interactive camera controls.

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::galaxy::data::build_galaxy;
use crate::galaxy::physics::{EntityType, GalaxyNode, GalaxySystem};
use crate::galaxy::renderer::{Camera3D, GalaxyRenderer};
use crate::theme::AppTheme;
use paraclea_core::bible::BibleReader;
use paraclea_core::library::LibraryEngine;

pub struct GalaxyState {
    pub system: GalaxySystem,
    pub camera: Camera3D,
    pub selected_index: usize,
}

impl GalaxyState {
    pub fn new(reader: &BibleReader, library: &LibraryEngine) -> Self {
        let system = build_galaxy(reader, library);
        Self {
            system,
            camera: Camera3D::new(),
            selected_index: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.system.update(dt);
        if self.camera.auto_spin {
            self.camera.rotate_yaw(dt * 0.04);
        }
    }

    pub fn selected_node_id(&self) -> Option<&str> {
        self.system.nodes.get(self.selected_index).map(|n| n.id.as_str())
    }

    pub fn selected_node(&self) -> Option<&GalaxyNode> {
        self.system.nodes.get(self.selected_index)
    }

    pub fn next_selection(&mut self) {
        if !self.system.nodes.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.system.nodes.len();
        }
    }

    pub fn prev_selection(&mut self) {
        if !self.system.nodes.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.system.nodes.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> GalaxyAction {
        match code {
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                self.camera.rotate_yaw(-0.08);
                GalaxyAction::Handled
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                self.camera.rotate_yaw(0.08);
                GalaxyAction::Handled
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.camera.rotate_pitch(0.06);
                GalaxyAction::Handled
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.camera.rotate_pitch(-0.06);
                GalaxyAction::Handled
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.camera.zoom(-2.5);
                GalaxyAction::Handled
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.camera.zoom(2.5);
                GalaxyAction::Handled
            }
            KeyCode::Char(' ') => {
                self.camera.auto_spin = !self.camera.auto_spin;
                GalaxyAction::Handled
            }
            KeyCode::Tab | KeyCode::Char(']') => {
                self.next_selection();
                GalaxyAction::Handled
            }
            KeyCode::BackTab | KeyCode::Char('[') => {
                self.prev_selection();
                GalaxyAction::Handled
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.camera.reset();
                GalaxyAction::Handled
            }
            KeyCode::Enter => {
                if let Some(node) = self.selected_node() {
                    GalaxyAction::Inspect(node.clone())
                } else {
                    GalaxyAction::None
                }
            }
            _ => GalaxyAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GalaxyAction {
    None,
    Handled,
    Inspect(GalaxyNode),
}

pub struct GalaxyView<'a> {
    pub state: &'a GalaxyState,
    pub theme: AppTheme,
}

impl<'a> GalaxyView<'a> {
    pub fn new(state: &'a GalaxyState, theme: AppTheme) -> Self {
        Self { state, theme }
    }
}

impl<'a> Widget for GalaxyView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Length(3),
            ])
            .split(area);

        let canvas_area = chunks[0];
        let hud_area = chunks[1];

        // 1. Draw Galaxy Border Block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border_focused())
            .title(Span::styled(
                " 🌌 PARACLEA GALAXY ATLAS — GOLDEN RATIO 3D DISK ",
                self.theme.header_title(),
            ));

        let inner_canvas = block.inner(canvas_area);
        block.render(canvas_area, buf);

        // 2. Render 3D Celestial Bodies to Buffer
        let selected_id = self.state.selected_node_id();
        GalaxyRenderer::render_galaxy(
            &self.state.system,
            &self.state.camera,
            selected_id,
            inner_canvas,
            buf,
            self.theme,
        );

        // 3. Draw HUD and Telemetry
        let selected_node = self.state.system.nodes.get(self.state.selected_index);
        let node_info = if let Some(n) = selected_node {
            let type_str = match n.entity_type {
                EntityType::Sun => "☀️ CORE SCRIPTURE",
                EntityType::Planet => "🪐 LANGUAGE ANCHOR",
                EntityType::Moon => "🌕 TRANSLATION",
                EntityType::Asteroid => "☄️ KNOWLEDGE DECK",
            };
            format!(
                "Target: {} [{}] | Pos: ({:.1}, {:.1}, {:.1}) | Lum: {:.2}",
                n.name, type_str, n.pos[0], n.pos[1], n.pos[2], n.brightness
            )
        } else {
            "No Target Selected".to_string()
        };

        let spin_status = if self.state.camera.auto_spin { "ON" } else { "OFF" };
        let hud_text = vec![
            Line::from(vec![
                Span::styled(" [🔭 TARGET] ", Style::default().fg(self.theme.primary()).add_modifier(Modifier::BOLD)),
                Span::styled(node_info, Style::default().fg(Color::Rgb(220, 230, 255))),
            ]),
            Line::from(vec![
                Span::styled(" [🎮 CONTROLS] ", Style::default().fg(self.theme.secondary()).add_modifier(Modifier::BOLD)),
                Span::raw("←/→/↑/↓: Orbit | +/-: Zoom | Space: Auto-spin ["),
                Span::styled(spin_status, Style::default().fg(if self.state.camera.auto_spin { Color::Green } else { Color::DarkGray })),
                Span::raw("] | Tab/[: Cycle Node | R: Reset | Enter: Read"),
            ]),
        ];

        let hud_widget = Paragraph::new(hud_text)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_type(BorderType::Rounded)
                    .border_style(self.theme.border_normal()),
            );
        hud_widget.render(hud_area, buf);
    }
}
