# Paraclea TUI Walkthrough

## Current State
- **Tests**: 35/35 passed (4 mazzaroth + 1 cli + 10 core + 20 tui)
- **Clippy**: 0 warnings (`cargo clippy --workspace --all-targets -- -D warnings`)
- **Binary**: `~/.local/bin/paraclea` and `~/.cargo/bin/paraclea` (install both — PATH order varies by shell)
- **Crates**: paraclea-core, paraclea-cli, paraclea-gui, paraclea-tui, mazzaroth
- **Launch**: bare `paraclea` starts the TUI; `paraclea --repl` starts the line REPL

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

The app boots with `active_focus = MainViewport`, so pressing `1`-`7` switches decks **immediately on launch**. While typing in the prompt, digits are just text — `2 Corinthians`, `3 John`, and `1689 London Baptist` all type normally. `Alt+1`-`Alt+7` switches decks from anywhere, including from prompt focus and with a modal open (the modal closes on switch).

### Command Palette & Help

Pressing `/` with an empty prompt (or from any non-prompt focus) opens the modal command palette; `?` opens the help modal. Both intercept input until dismissed with Esc/Enter/`q`.

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
            Style::default().fg(Color::Rgb(115, 120, 150)) // unfocused (WCAG AA)
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

## Galaxy View (Mazzaroth Engine)

The galaxy visualization is powered by **Mazzaroth** — a standalone, database-agnostic 3D celestial renderer extracted from Paraclea.

### Controls
- `WASD` or arrow keys: rotate camera (yaw/pitch)
- `+`/`-`: zoom in/out
- Mouse wheel: zoom in (up) / out (down)
- `Space`: toggle pause (freezes both camera spin AND orbital simulation)
- `]` / `[`: cycle through celestial nodes (Tab/Shift+Tab are consumed by the global focus cycle)
- `Enter`: inspect selected node (jumps to relevant view)
- `R`: reset camera
- `Mouse drag`: rotate camera

### Node Types
- **Sun**: Central core (Holy Scripture)
- **Planet**: Language systems orbiting the sun
- **Moon**: Bible translations orbiting their parent language
- **Asteroid**: Library knowledge decks, dust particles

### Mazzaroth Architecture

```
crates/mazzaroth/
├── src/
│   ├── lib.rs        — Public API re-exports
│   ├── physics.rs    — Two-pass hierarchical orbital simulation (O(1) parent lookup)
│   ├── renderer.rs   — 3D→2D projection, painter's algorithm, 350-star parallax
│   ├── schema.rs     — DatabaseSchema trait (the generic interface)
│   ├── builder.rs    — Generic GalaxyBuilder (schema → GalaxySystem)
│   └── theme.rs      — GalaxyTheme trait + 5 preset themes
```

### DatabaseSchema Trait

Any program can implement this to get a 3D galaxy:

```rust
pub trait DatabaseSchema {
    fn root_name(&self) -> &str;
    fn categories(&self) -> Vec<Category>;
    fn items_for_category(&self, category_id: &str) -> Vec<Item>;
    // Optional overrides with defaults:
    fn root_id(&self) -> &str { "root" }
    fn outer_decks(&self) -> Vec<Deck> { vec![] }
    fn core_dust_config(&self) -> DustConfig { DustConfig { count: 80, scatter: 1.5 } }
}
```

### Paraclea Integration

`ParacleaSchema` bridges `BibleReader` and `LibraryEngine` into Mazzaroth:

```rust
impl DatabaseSchema for ParacleaSchema {
    fn categories(&self) -> Vec<Category> {
        // BibleReader::list_languages() → Category
    }
    fn items_for_category(&self, cat_id: &str) -> Vec<Item> {
        // BibleReader::list_translations_for_lang() → Item
    }
}
```

---

## Cross-Platform Compatibility

Paraclea runs on **Linux (x86_64/ARM64), macOS, and Windows**.

### Platform Helpers (`paraclea-core/src/lib.rs`)

```rust
pub fn home_dir() -> PathBuf    // dirs::home_dir() fallback to "."
pub fn temp_dir() -> PathBuf    // std::env::temp_dir()
pub fn shell_command() -> Command  // "cmd" on Windows, "sh" on Unix
pub fn shell_arg() -> &'static str // "/C" on Windows, "-c" on Unix
```

### What's Platform-Specific

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Audio playback | `aplay`/`paplay`/`pw-play` | `afplay` | `PowerShell SoundPlayer` |
| Shell commands | `sh -c` | `sh -c` | `cmd /C` |
| Home directory | `$HOME` | `$HOME` | `USERPROFILE` |
| USB backup paths | `/media/<user>`, `/run/media/<user>` | `/Volumes` | `D:\`..`Z:\` |
| Temp directory | `/tmp` | `/private/var/folders/...` | `%TEMP%` |

### What's Universal (No Changes Needed)

- Pure Rust — no `unsafe`, no C dependencies, no SIMD
- All data in JSON/YAML/SQLite — architecture-neutral formats
- `ratatui` + `crossterm` — cross-platform terminal UI
- `rusqlite` bundled — compiles SQLite from C source on all platforms
- `aes-gcm`, `pbkdf2`, `sha2` — pure Rust crypto
- `dirs` crate — cross-platform directory resolution

---

## UI/UX Overhaul (v0.9.0)

### Contrast System

Three theme hooks guarantee visibility regardless of terminal background:

```rust
theme.bg()        // root frame fill (app.rs render — drawn first, full area)
theme.panel_bg()  // every Block in the crate (26/26) — header, sidebar, views, modals
theme.input_bg()  // prompt input bar
```

No element relies on the terminal's default background anymore. `Color::DarkGray` was eliminated entirely; unfocused borders use `rgb(115,120,150)` and inactive tabs use `rgb(150,150,175)` so everything clears WCAG AA.

### Scroll Architecture

The renderer computes the bottom anchor once per frame and publishes it to input handlers:

```rust
// chat.rs
pub fn chat_max_scroll(history, streaming_text, is_streaming, w, h, theme) -> usize

// app.rs
chat_max_scroll: Cell<usize>,          // set in render_main_viewport
Up/PageUp/ScrollUp   → sync to bottom first (if auto), then decrement
Down/PageDown/ScrollDown → clamp to cell value, re-engage at bottom
```

Because the bound comes from the same `build_chat_lines` + unicode-width math the renderer uses, keyboard, mouse, and render always agree — no more hardcoded `total_lines - 10` heuristics.

### Stream Lifecycle

```
prompt → cancel_active_stream() → gen_id++ → tokio::spawn(AbortHandle stored)
                                              │
Esc anywhere → abort() + gen_id++ ────────────┤ (stale Token/Done/Error dropped)
second prompt → cancel first ─────────────────┤ (no channel interleave)
/clear → cancel + clear streaming_text ───────┘ (no ghost resurrection)
Done (current gen) → push to chat_history; partial on cancel tagged [cancelled]
```

---

## Keyboard Reference

| Key | Context | Action |
|-----|---------|--------|
| `Tab` / `Shift+Tab` | No modal open | Cycle focus: Sidebar → MainViewport → PromptInput |
| `1`-`7` | MainViewport / Sidebar | Jump directly to deck (works on launch) |
| `Alt+1`-`Alt+7` | Global | Jump to deck from anywhere (closes open modal) |
| `Ctrl+T` | Global | Cycle theme (persists to config) |
| `Ctrl+B` | Global | Toggle sidebar |
| `Ctrl+P` | Global | Open translation picker |
| `Ctrl+M` | Global | Open model picker |
| `Ctrl+U` | Global | Trigger encrypted backup (status via toast) |
| `/` | Empty prompt / non-prompt focus | Open command palette |
| `?` | Empty prompt / non-prompt focus | Open help modal |
| `i` / `Enter` | MainViewport | Focus prompt input |
| `Esc` | While streaming | Cancel generation (keeps partial reply) |
| `Esc` | MainViewport (not streaming) | Focus prompt input |
| `Esc` | Modal open | Close modal |
| `Up`/`Down`/`PageUp`/`PageDown` | Chat | Scroll (disengages auto-scroll) |
| `Home`/`End` | Chat | Top / bottom (End re-engages auto-scroll) |
| `j`/`k`, `↑`/`↓` | Bible / Library | Navigate books |
| `h`/`l`, `←`/`→` | Bible | Navigate chapters |
| `c` | Bible | Toggle compare mode |
| `[`/`]`, `p`/`n` | Library | Previous / next chapter |
| `Enter` | Galaxy | Inspect selected node |
| `Space` | Galaxy | Pause/resume spin + simulation |
| Mouse wheel | Chat / Bible / Library | Scroll (clamped; bottom re-engages auto-scroll) |
| Mouse wheel | Galaxy | Zoom in / out |
| Left-drag | Galaxy | Rotate camera |

---

## Verification
```bash
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```
