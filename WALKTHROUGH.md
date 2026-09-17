# Paraclea TUI Walkthrough

## Current State
- **Tests**: 23/23 passed
- **Clippy**: 0 warnings
- **Binary**: `~/.local/bin/paraclea` and `~/.cargo/bin/paraclea`

---

## Tab Tile-Switching Architecture

The TUI uses a **3-tile layout** with a focus-cycling system. Pressing `Tab` moves keyboard focus between the three visible tiles. Each tile has its own interaction model and visual feedback.

### The Two Core Enums

```rust
// Which content view is shown in the main viewport
pub enum ActiveTab {
    Chat = 0,
    Bible = 1,
    Library = 2,
    Crossref = 3,
    Galaxy = 4,
    Mesh = 5,
    Doctor = 6,
}

// Which tile has keyboard focus (cycled with Tab)
pub enum ActiveFocus {
    Sidebar,
    MainViewport,
    PromptInput,
}
```

### The 3-Tile Layout

```
+----------------------------------------------+
|  Header / Tab Bar                    (3 row) |
+----------------------------------------------+
| Sidebar  |  Main Viewport                    |
| (28 col) |  (remaining, min 40)              |
|          |                                   |
|          |  Content renders here based on    |
|          |  ActiveTab (Chat, Bible, etc.)    |
+----------------------------------------------+
|  Prompt Input / Command Bar           (3 row)|
+----------------------------------------------+
```

### Tab Key Cycling

In `handle_key_event` (app.rs:517):

```rust
if code == KeyCode::Tab {
    self.active_focus = match self.active_focus {
        ActiveFocus::Sidebar => ActiveFocus::MainViewport,
        ActiveFocus::MainViewport => ActiveFocus::PromptInput,
        ActiveFocus::PromptInput => ActiveFocus::Sidebar,
    };
    return Ok(false);
}
```

The cycle is: **Sidebar -> MainViewport -> PromptInput -> Sidebar** (fixed circular pattern).

### Visual Focus Feedback

Each tile checks `active_focus` and applies different border styles:

```rust
// Focused tile gets bright primary color
.border_style(if self.active_focus == ActiveFocus::Sidebar {
    self.theme.border_focused()   // bright gold/cyan/etc
} else {
    self.theme.border_normal()    // dim gray-blue
})
```

The focused tile lights up. The other tiles dim. This gives instant visual feedback about which tile owns the keyboard.

### Per-Tile Input Dispatch

After Tab cycling, the code dispatches to the focused tile's handler:

```
Tab pressed?
  |
  v
ActiveFocus::PromptInput  -> text editing, command execution
ActiveFocus::MainViewport -> second dispatch on ActiveTab:
  |   Chat     -> scroll up/down, page up/down, home/end
  |   Bible    -> j/k book nav, h/l chapter nav, c compare
  |   Library  -> up/down book nav, left/right category nav
  |   Galaxy   -> delegates to GalaxyView::handle_key()
  |   ...
ActiveFocus::Sidebar      -> up/down changes ActiveTab, Enter moves to MainViewport
```

### Sidebar as Navigation Menu

When focus is on the Sidebar:
- `Up`/`Down` or `j`/`k` cycles through the 7 tabs
- `Enter` moves focus to MainViewport to interact with the selected tab
- The sidebar also shows system status (Ollama, Qdrant, Bible stats)

### Numeric Shortcuts

Pressing `1`-`7` (when focus is not on PromptInput) directly switches `ActiveTab` without cycling through the sidebar.

### Command Palette

Pressing `/` (when input is empty) or `?` opens a modal command palette that intercepts all input until dismissed with Esc.

---

## How to Replicate This Pattern

### Core Ingredients

1. **Two enums**: one for "which view is shown" (`ActiveTab`), one for "which tile has focus" (`ActiveFocus`).

2. **Layout**: Use `ratatui::Layout` to split the screen into N regions. Map each region to an `ActiveFocus` variant.

3. **Tab handler**: A single match statement that cycles `ActiveFocus` in a fixed order. Run it **before** focus-specific dispatch, **after** modal handling.

4. **Border highlighting**: Each tile renderer checks `active_focus == MyFocus::ThisTile` and applies bright vs dim border styles.

5. **Input dispatch**: A nested match: first on `ActiveFocus`, then on `ActiveTab` (for the main viewport tile). Each branch handles its own keys.

6. **Sidebar navigation**: When focus is on the sidebar, Up/Down changes `ActiveTab`, Enter moves focus to MainViewport.

### Minimal Skeleton

```rust
#[derive(Clone, Copy, PartialEq)]
enum Focus { Left, Center, Right }

#[derive(Clone, Copy, PartialEq)]
enum View { Dashboard, Settings, Logs }

struct App {
    focus: Focus,
    view: View,
}

impl App {
    fn handle_key(&mut self, key: KeyCode) {
        // 1. Tab cycles focus
        if key == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::Left => Focus::Center,
                Focus::Center => Focus::Right,
                Focus::Right => Focus::Left,
            };
            return;
        }

        // 2. Dispatch to focused tile
        match self.focus {
            Focus::Left => match key {
                KeyCode::Up => { /* change self.view */ }
                KeyCode::Enter => { self.focus = Focus::Center; }
                _ => {}
            },
            Focus::Center => match self.view {
                View::Dashboard => { /* dashboard keys */ }
                View::Settings => { /* settings keys */ }
                _ => {}
            },
            Focus::Right => { /* log scrolling */ }
        }
    }

    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(28),   // Left tile
                Constraint::Min(40),     // Center tile
                Constraint::Length(30),  // Right tile
            ])
            .split(f.area());

        // Each tile checks self.focus for border highlighting
        let left_border = if self.focus == Focus::Left {
            Style::default().fg(Color::Yellow)  // focused
        } else {
            Style::default().fg(Color::DarkGray) // unfocused
        };
        // ... render each tile with its border style
    }
}
```

### Key Design Decisions

- **Tab cycles, it doesn't toggle.** With 3 tiles, a cycle is natural. With 2, a toggle works. With 4+, a cycle still works.
- **Focus is separate from content.** `ActiveFocus` controls the keyboard. `ActiveTab` controls what's rendered. They're orthogonal.
- **Visual feedback is critical.** Without border highlighting, the user has no idea which tile owns the keyboard. Always dim unfocused tiles.
- **Modals intercept before Tab.** If a modal is open, Tab should not cycle focus. Check `active_modal != None` first and return early.
- **Sidebar acts as a menu.** When focus is on the sidebar, keys change `ActiveTab`. Enter "confirms" the selection and moves focus to the content tile.

---

## Theme System

5 themes with distinct color families:
- **RoyalByzantium**: Gold & Purple (default)
- **CrimsonCodex**: Deep Red & Cream
- **CyberScholar**: Cyan & Magenta
- **EmeraldMatrix**: Mint & Dark Green
- **CelestialMidnight**: Indigo & Silver

Theme persists across restarts via `Config.theme` field (YAML). Toggle with `Ctrl+T` or `/theme`.

### Theme Colors Per Element

| Element | Color Source |
|---------|-------------|
| Sun (The Word) | `theme.primary()` |
| Planets (Languages) | `theme.secondary()` |
| Flagship Moons | `theme.primary()` at 0.35 brightness |
| Secondary Moons | `theme.secondary()` at 0.20 brightness |
| Category Decks | `theme.accent()` |
| Core Dust | `theme.primary()` at 0.12 brightness |
| Background Stars | `theme.text()` at 0.08 brightness |

---

## Galaxy View

### Controls
- `WASD` or arrow keys: rotate camera (yaw/pitch)
- `+`/`-`: zoom in/out
- `Space`: toggle pause (freezes both camera spin AND orbital simulation)
- `Tab`/`[`: cycle through celestial nodes
- `Enter`: inspect selected node (jumps to relevant view)
- `R`: reset camera
- `Mouse drag`: rotate camera

### Node Types
- **Sun**: Central core (Holy Scripture)
- **Planet**: Language systems orbiting the sun
- **Moon**: Bible translations orbiting their parent language
- **Asteroid**: Library knowledge decks, dust particles

### Architecture
- `physics.rs`: Two-pass hierarchical orbital simulation (O(1) parent lookup via HashMap)
- `renderer.rs`: 3D→2D projection with XS=1.62 stretch, painter's algorithm, 350-star parallax background
- `data.rs`: Bible/library → galaxy binding (14 planets, ~84 moons, 80 core dust, 50 ambient dust)

---

## Keyboard Reference

| Key | Context | Action |
|-----|---------|--------|
| `Tab` | Any | Cycle focus between tiles |
| `1`-`7` | Non-input | Jump directly to tab |
| `Ctrl+T` | Global | Cycle theme |
| `Ctrl+B` | Global | Toggle sidebar |
| `Ctrl+P` | Global | Open translation picker |
| `Ctrl+M` | Global | Open model picker |
| `Ctrl+U` | Global | Trigger encrypted backup |
| `/` | Empty input | Open command palette |
| `?` | Non-input | Open help |
| `Esc` | Modal | Close modal |
| `Up`/`Down` | Chat | Scroll conversation |
| `PageUp`/`PageDown` | Chat/Bible/Library | Scroll by page |
| `Home`/`End` | Chat/Bible/Library | Jump to top/bottom |
| `j`/`k` | Bible/Library | Navigate books |
| `h`/`l` | Bible | Navigate chapters |
| `c` | Bible | Toggle compare mode |
| `Space` | Galaxy | Pause/resume simulation |

---

## Verification
```bash
cargo test --workspace && cargo clippy --workspace -- -D warnings
```
