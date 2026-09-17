# Paraclea Galaxy — Theme Integration & Visual Contrast Overhaul

## Key Enhancements

### 1. Dynamic Theme-Aware Color Mapping (`renderer.rs` & `theme.rs`)
- **Sun (The Word)**: Luminous `theme.primary()` (gold, amber, cyan, mint, or starlight cyan) with radiant tri-glyph `✦ ✸ ✦` (brightness floor `0.70`).
- **Planets (Language Systems)**: Distinct `theme.secondary()` (purple, sepia, magenta, dark green, or solar gold, brightness floor `0.45`).
- **Flagship Moons**: Echo of the central star in `theme.primary()` (`0.35` floor).
- **Secondary Moons**: Echo of the parent planet in `theme.secondary()` (`0.20` floor).
- **Category Outer Decks**: `theme.accent()` (`0.40` floor) with orbiting particulate ring dust (`0.18` floor).
- **Core Dust**: `theme.primary()` subtle glow (`0.12` floor).
- **Background Starfield**: `theme.text()` starlight points (`0.08` floor).

### 2. Softened Distance-Alpha Curve
- Replaced harsh `(f - 0.45) * 1.55` curve with calibrated `alpha = max(0.12, (f - 0.35) * 1.2) * brightness`.
- Ensures distant celestial objects and moons behind the mid-plane remain visible with distinct depth rather than vanishing into fog.

### 3. Real-Time Theme Switching
- `AppTheme` passed dynamically to `render_galaxy` each frame.
- Cycling themes (via `t` or `/theme`) instantly restyles the entire 3D celestial sphere into the active palette.

---

## Verification
```bash
cargo test --workspace && cargo clippy --workspace -- -D warnings
```
- **Tests**: 22/22 passed.
- **Clippy**: 0 warnings.
- **Installed Binary**: Updated in `~/.local/bin/paraclea` and `~/.cargo/bin/paraclea`.
