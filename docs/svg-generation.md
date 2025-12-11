# Design Document: Pure Rust SVG Generation for Music Theory Diagrams

## Overview

This document outlines the design for a pure Rust SVG generation module that creates music theory diagrams without external dependencies. The module will generate pitch circle diagrams, fretboard diagrams, and related visualizations directly as SVG strings.

This is a port of the Python `svg_tools.py` design to idiomatic Rust, adapted to work with the existing `rust-music` type system (`Pc`, `PcSet`, `FretboardShape`, etc.).

## Motivation

### Why Native SVG Generation?

1. **Zero dependencies**: Only Rust standard library
2. **Fast**: No subprocess calls, no intermediate files, no runtime overhead
3. **Portable**: Compiles to any target Rust supports (WASM, native, embedded)
4. **Embeddable**: SVG strings can be saved to files, embedded in HTML, served via API
5. **Type-safe**: Leverage Rust's type system to prevent invalid diagram states

### Use Cases

- CLI tool that outputs SVG files
- Web server (Axum/Actix) serving diagram endpoints
- WASM module for browser-based music theory apps
- Static site generation for music education content
- Integration with notation software

## Module Structure

```
music/src/
└── svg/
    ├── mod.rs              # Module exports, shared types
    ├── pitch_circle.rs     # Chromatic circle diagrams
    ├── fretboard.rs        # Fretboard/chord diagrams
    ├── interval.rs         # Interval diagrams
    ├── theme.rs            # Color themes and styling
    └── util.rs             # SVG primitives, math helpers
```

## Core Types

### Configuration Structs

```rust
/// Theme for consistent styling across diagrams
#[derive(Clone, Debug)]
pub struct SvgTheme {
    pub highlight_color: &'static str,
    pub root_color: &'static str,
    pub background_color: &'static str,
    pub stroke_color: &'static str,
    pub text_color: &'static str,
    pub inactive_color: &'static str,
}

impl Default for SvgTheme {
    fn default() -> Self {
        Self {
            highlight_color: "#4a90d9",
            root_color: "#2d5a87",
            background_color: "#ffffff",
            stroke_color: "#333333",
            text_color: "#333333",
            inactive_color: "#cccccc",
        }
    }
}

/// Configuration for pitch circle diagrams
#[derive(Clone, Debug)]
pub struct PitchCircleConfig {
    pub radius: u32,
    pub show_intervals: bool,
    pub title: Option<String>,
    pub note_labels: NoteLabels,
    pub theme: SvgTheme,
}

impl Default for PitchCircleConfig {
    fn default() -> Self {
        Self {
            radius: 80,
            show_intervals: false,
            title: None,
            note_labels: NoteLabels::Mixed,
            theme: SvgTheme::default(),
        }
    }
}

/// How to label the 12 pitch classes
#[derive(Clone, Debug)]
pub enum NoteLabels {
    Sharps,      // C, C#, D, D#, E, F, F#, G, G#, A, A#, B
    Flats,       // C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B
    Mixed,       // C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B
    PitchClass,  // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    Custom([&'static str; 12]),
}

/// Configuration for fretboard diagrams
#[derive(Clone, Debug)]
pub struct FretboardConfig {
    pub num_frets: u8,
    pub start_fret: u8,
    pub show_fret_numbers: bool,
    pub show_string_names: bool,
    pub orientation: Orientation,
    pub title: Option<String>,
    pub string_names: Option<Vec<String>>,
    pub theme: SvgTheme,
}

impl Default for FretboardConfig {
    fn default() -> Self {
        Self {
            num_frets: 5,
            start_fret: 0,
            show_fret_numbers: true,
            show_string_names: true,
            orientation: Orientation::Vertical,
            title: None,
            string_names: None,
            theme: SvgTheme::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Orientation {
    #[default]
    Vertical,   // Standard chord diagram
    Horizontal, // Tab-like orientation
}
```

### Builder Pattern (Ergonomic API)

```rust
/// Builder for pitch circle SVGs
pub struct PitchCircleBuilder {
    pitch_set: HashSet<Pc>,
    root: Option<Pc>,
    config: PitchCircleConfig,
}

impl PitchCircleBuilder {
    pub fn new() -> Self {
        Self {
            pitch_set: HashSet::new(),
            root: None,
            config: PitchCircleConfig::default(),
        }
    }

    pub fn pitches(mut self, pcs: impl IntoIterator<Item = Pc>) -> Self {
        self.pitch_set = pcs.into_iter().collect();
        self
    }

    pub fn from_pc_set(mut self, pc_set: &PcSet) -> Self {
        self.pitch_set = pc_set.iter().cloned().collect();
        self
    }

    pub fn root(mut self, pc: Pc) -> Self {
        self.root = Some(pc);
        self
    }

    pub fn radius(mut self, r: u32) -> Self {
        self.config.radius = r;
        self
    }

    pub fn show_intervals(mut self, show: bool) -> Self {
        self.config.show_intervals = show;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    pub fn theme(mut self, theme: SvgTheme) -> Self {
        self.config.theme = theme;
        self
    }

    pub fn build(self) -> String {
        pitch_circle_svg(&self.pitch_set, self.root, &self.config)
    }
}

/// Builder for fretboard SVGs
pub struct FretboardBuilder<'a> {
    shape: Option<&'a FretboardShape<'a>>,
    positions: Vec<FretPosition>,
    root_positions: HashSet<(u8, u8)>,
    config: FretboardConfig,
}

#[derive(Clone, Debug)]
pub enum FretPosition {
    Fretted { string: u8, fret: u8 },
    Open { string: u8 },
    Muted { string: u8 },
}

impl<'a> FretboardBuilder<'a> {
    pub fn new() -> Self {
        Self {
            shape: None,
            positions: Vec::new(),
            root_positions: HashSet::new(),
            config: FretboardConfig::default(),
        }
    }

    pub fn from_shape(mut self, shape: &'a FretboardShape<'a>) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn position(mut self, pos: FretPosition) -> Self {
        self.positions.push(pos);
        self
    }

    pub fn positions(mut self, positions: impl IntoIterator<Item = FretPosition>) -> Self {
        self.positions.extend(positions);
        self
    }

    pub fn root_at(mut self, string: u8, fret: u8) -> Self {
        self.root_positions.insert((string, fret));
        self
    }

    pub fn start_fret(mut self, fret: u8) -> Self {
        self.config.start_fret = fret;
        self
    }

    pub fn num_frets(mut self, n: u8) -> Self {
        self.config.num_frets = n;
        self
    }

    pub fn orientation(mut self, o: Orientation) -> Self {
        self.config.orientation = o;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    pub fn build(self) -> String {
        if let Some(shape) = self.shape {
            fretboard_shape_svg(shape, &self.root_positions, &self.config)
        } else {
            fretboard_positions_svg(&self.positions, &self.root_positions, &self.config)
        }
    }
}
```

## Core Functions

### Pitch Circle

```rust
/// Generate SVG for a chromatic pitch circle
pub fn pitch_circle_svg(
    pitch_set: &HashSet<Pc>,
    root: Option<Pc>,
    config: &PitchCircleConfig,
) -> String {
    let mut svg = SvgBuilder::new(config.radius * 2 + 80, config.radius * 2 + 80);

    let cx = config.radius + 40;
    let cy = config.radius + 40;

    // Add title if present
    if let Some(ref title) = config.title {
        svg.text(cx, 18, title, "title-text");
    }

    // Draw background circle
    svg.circle(cx, cy, config.radius, "none", &config.theme.inactive_color, 1);

    // Draw interval lines if requested
    if config.show_intervals && pitch_set.len() > 1 {
        svg.group_start("interval-lines");
        let sorted: Vec<_> = pitch_set.iter().sorted().collect();
        for window in sorted.windows(2) {
            let (x1, y1) = pc_to_coords(window[0], cx, cy, config.radius);
            let (x2, y2) = pc_to_coords(window[1], cx, cy, config.radius);
            svg.line(x1, y1, x2, y2, &config.theme.highlight_color, 2, 0.5);
        }
        // Close the loop
        let (x1, y1) = pc_to_coords(sorted.last().unwrap(), cx, cy, config.radius);
        let (x2, y2) = pc_to_coords(sorted.first().unwrap(), cx, cy, config.radius);
        svg.line(x1, y1, x2, y2, &config.theme.highlight_color, 2, 0.5);
        svg.group_end();
    }

    // Draw pitch class dots
    for pc in PcIter::default() {
        let (x, y) = pc_to_coords(&pc, cx, cy, config.radius);
        let is_root = root.map_or(false, |r| r == pc);
        let is_highlighted = pitch_set.contains(&pc);

        let fill = if is_root && is_highlighted {
            &config.theme.root_color
        } else if is_highlighted {
            &config.theme.highlight_color
        } else {
            &config.theme.background_color
        };

        let text_color = if is_highlighted {
            "#ffffff"
        } else {
            &config.theme.text_color
        };

        svg.circle(x, y, 14, fill, &config.theme.stroke_color, 1.5);
        svg.text(x, y + 4, &pc_label(&pc, &config.note_labels), "note-text")
            .fill(text_color);
    }

    svg.build()
}

fn pc_to_coords(pc: &Pc, cx: u32, cy: u32, radius: u32) -> (f64, f64) {
    let angle = (u8::from(pc) as f64 * 30.0 - 90.0).to_radians();
    let x = cx as f64 + radius as f64 * angle.cos();
    let y = cy as f64 + radius as f64 * angle.sin();
    (x, y)
}
```

### Fretboard Diagram

```rust
/// Generate SVG from a FretboardShape
pub fn fretboard_shape_svg(
    shape: &FretboardShape,
    root_positions: &HashSet<(u8, u8)>,
    config: &FretboardConfig,
) -> String {
    let positions: Vec<FretPosition> = shape.fretted_notes
        .iter()
        .enumerate()
        .map(|(string, note)| {
            let string = string as u8;
            match note {
                FrettedNote::Muted { .. } => FretPosition::Muted { string },
                FrettedNote::Sounded(SoundedNote { fret: 0, .. }) => FretPosition::Open { string },
                FrettedNote::Sounded(SoundedNote { fret, .. }) => FretPosition::Fretted { string, fret: *fret },
            }
        })
        .collect();

    fretboard_positions_svg(&positions, root_positions, config)
}

/// Generate SVG from raw positions
pub fn fretboard_positions_svg(
    positions: &[FretPosition],
    root_positions: &HashSet<(u8, u8)>,
    config: &FretboardConfig,
) -> String {
    let num_strings = positions.iter()
        .map(|p| match p {
            FretPosition::Fretted { string, .. } |
            FretPosition::Open { string } |
            FretPosition::Muted { string } => *string
        })
        .max()
        .map(|s| s + 1)
        .unwrap_or(6);

    match config.orientation {
        Orientation::Vertical => draw_vertical_fretboard(positions, root_positions, num_strings, config),
        Orientation::Horizontal => draw_horizontal_fretboard(positions, root_positions, num_strings, config),
    }
}

fn draw_vertical_fretboard(
    positions: &[FretPosition],
    root_positions: &HashSet<(u8, u8)>,
    num_strings: u8,
    config: &FretboardConfig,
) -> String {
    let string_spacing = 30u32;
    let fret_spacing = 40u32;
    let margin_top = if config.title.is_some() { 50 } else { 30 };
    let margin_left = if config.show_fret_numbers { 40 } else { 20 };
    let margin_right = 30u32;
    let margin_bottom = 30u32;
    let dot_radius = 10u32;

    let width = margin_left + (num_strings as u32 - 1) * string_spacing + margin_right;
    let height = margin_top + config.num_frets as u32 * fret_spacing + margin_bottom;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text(width / 2, 20, title, "title-text");
    }

    // Nut (thick line at top if starting at fret 0)
    if config.start_fret == 0 {
        let x1 = margin_left;
        let x2 = margin_left + (num_strings as u32 - 1) * string_spacing;
        svg.line(x1, margin_top, x2, margin_top, &config.theme.stroke_color, 4, 1.0);
    }

    // Frets
    for fret in 0..=config.num_frets {
        if fret == 0 && config.start_fret == 0 { continue; } // Skip if nut drawn
        let y = margin_top + fret as u32 * fret_spacing;
        let x1 = margin_left;
        let x2 = margin_left + (num_strings as u32 - 1) * string_spacing;
        svg.line(x1, y, x2, y, &config.theme.stroke_color, 1, 1.0);
    }

    // Strings
    for string in 0..num_strings {
        let x = margin_left + string as u32 * string_spacing;
        let y1 = margin_top;
        let y2 = margin_top + config.num_frets as u32 * fret_spacing;
        svg.line(x, y1, x, y2, &config.theme.stroke_color, 1, 1.0);
    }

    // Fret number (if not open position)
    if config.show_fret_numbers && config.start_fret > 0 {
        svg.text(margin_left - 15, margin_top + fret_spacing / 2 + 4,
                 &config.start_fret.to_string(), "fret-text");
    }

    // String names
    if config.show_string_names {
        let names = config.string_names.as_ref()
            .map(|v| v.clone())
            .unwrap_or_else(|| vec!["E", "A", "D", "G", "B", "E"]
                .into_iter().take(num_strings as usize).map(String::from).collect());

        for (i, name) in names.iter().enumerate() {
            let x = margin_left + i as u32 * string_spacing;
            let y = margin_top + config.num_frets as u32 * fret_spacing + 20;
            svg.text(x, y, name, "string-text");
        }
    }

    // Draw positions
    for pos in positions {
        match pos {
            FretPosition::Muted { string } => {
                let x = margin_left + *string as u32 * string_spacing;
                let y = margin_top - 15;
                svg.text(x, y, "×", "string-text");
            }
            FretPosition::Open { string } => {
                let x = margin_left + *string as u32 * string_spacing;
                let y = margin_top - 12;
                svg.circle(x, y, 6, "none", &config.theme.stroke_color, 1.5);
            }
            FretPosition::Fretted { string, fret } => {
                let relative_fret = fret - config.start_fret;
                if relative_fret > 0 && relative_fret <= config.num_frets {
                    let x = margin_left + *string as u32 * string_spacing;
                    let y = margin_top + (relative_fret as u32 * fret_spacing) - (fret_spacing / 2);
                    let is_root = root_positions.contains(&(*string, *fret));
                    let fill = if is_root {
                        &config.theme.root_color
                    } else {
                        &config.theme.highlight_color
                    };
                    svg.circle(x, y, dot_radius, fill, &config.theme.stroke_color, 1);
                }
            }
        }
    }

    svg.build()
}
```

### SVG Builder (Internal)

```rust
/// Low-level SVG string builder
struct SvgBuilder {
    parts: Vec<String>,
    width: u32,
    height: u32,
}

impl SvgBuilder {
    fn new(width: u32, height: u32) -> Self {
        let mut parts = Vec::new();
        parts.push(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            width, height, width, height
        ));
        parts.push(r#"  <style>
    .note-text { font-family: Arial, sans-serif; font-size: 11px; font-weight: bold; text-anchor: middle; }
    .title-text { font-family: Arial, sans-serif; font-size: 14px; font-weight: bold; text-anchor: middle; }
    .fret-text { font-family: Arial, sans-serif; font-size: 10px; text-anchor: middle; }
    .string-text { font-family: Arial, sans-serif; font-size: 11px; font-weight: bold; text-anchor: middle; }
  </style>"#.to_string());

        Self { parts, width, height }
    }

    fn circle(&mut self, cx: u32, cy: u32, r: u32, fill: &str, stroke: &str, stroke_width: u32) -> &mut Self {
        self.parts.push(format!(
            r#"  <circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            cx, cy, r, fill, stroke, stroke_width
        ));
        self
    }

    fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, stroke: &str, stroke_width: u32, opacity: f32) -> &mut Self {
        self.parts.push(format!(
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-opacity="{}"/>"#,
            x1, y1, x2, y2, stroke, stroke_width, opacity
        ));
        self
    }

    fn text(&mut self, x: u32, y: u32, content: &str, class: &str) -> &mut Self {
        self.parts.push(format!(
            r#"  <text x="{}" y="{}" class="{}">{}</text>"#,
            x, y, class, content
        ));
        self
    }

    fn group_start(&mut self, class: &str) -> &mut Self {
        self.parts.push(format!(r#"  <g class="{}">"#, class));
        self
    }

    fn group_end(&mut self) -> &mut Self {
        self.parts.push("  </g>".to_string());
        self
    }

    fn build(mut self) -> String {
        self.parts.push("</svg>".to_string());
        self.parts.join("\n")
    }
}
```

## Integration with Existing Types

### Trait Extensions

```rust
/// Extension trait for types that can be rendered as pitch circles
pub trait ToPitchCircleSvg {
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder;
}

impl ToPitchCircleSvg for PcSet {
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder {
        PitchCircleBuilder::new().from_pc_set(self)
    }
}

impl ToPitchCircleSvg for NoteSet {
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder {
        let pc_set = PcSet::from(self);
        PitchCircleBuilder::new().from_pc_set(&pc_set)
    }
}

/// Extension trait for types that can be rendered as fretboard diagrams
pub trait ToFretboardSvg<'a> {
    fn to_fretboard_svg(&'a self) -> FretboardBuilder<'a>;
}

impl<'a> ToFretboardSvg<'a> for FretboardShape<'a> {
    fn to_fretboard_svg(&'a self) -> FretboardBuilder<'a> {
        FretboardBuilder::new().from_shape(self)
    }
}
```

### Usage Examples

```rust
use music::svg::{PitchCircleBuilder, FretboardBuilder, ToPitchCircleSvg, ToFretboardSvg};
use music::{Pc, PcSet, FretboardShape, STD_6STR_GTR};

// Method 1: Builder pattern directly
let svg = PitchCircleBuilder::new()
    .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])  // C major triad
    .root(Pc::Pc0)
    .title("C Major Triad")
    .show_intervals(true)
    .build();

// Method 2: From existing types via trait
let c_major_scale = PcSet::new(vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11]);
let svg = c_major_scale
    .to_pitch_circle_svg()
    .root(Pc::Pc0)
    .title("C Major Scale")
    .build();

// Method 3: Fretboard from shape
let shape: FretboardShape = /* ... */;
let svg = shape
    .to_fretboard_svg()
    .root_at(0, 5)  // A on low E string
    .root_at(2, 7)  // A on D string
    .title("A Minor Pentatonic")
    .build();

// Save to file
std::fs::write("diagram.svg", svg)?;
```

## Utility Functions

```rust
/// Save SVG string to a file
pub fn save_svg(svg: &str, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::write(path, svg)
}

/// Convert SVG to a data URI for HTML embedding
pub fn svg_to_data_uri(svg: &str) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let encoded = STANDARD.encode(svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", encoded)
}

/// Generate an HTML page with embedded SVG
pub fn svg_to_html(svg: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body style="display:flex;justify-content:center;padding:20px;">
{}
</body>
</html>"#, title, svg)
}
```

## Theme Presets

```rust
impl SvgTheme {
    pub fn dark() -> Self {
        Self {
            highlight_color: "#5dade2",
            root_color: "#2e86ab",
            background_color: "#1a1a2e",
            stroke_color: "#666666",
            text_color: "#cccccc",
            inactive_color: "#333333",
        }
    }

    pub fn print() -> Self {
        Self {
            highlight_color: "#000000",
            root_color: "#000000",
            background_color: "#ffffff",
            stroke_color: "#000000",
            text_color: "#000000",
            inactive_color: "#999999",
        }
    }

    pub fn colorful() -> Self {
        Self {
            highlight_color: "#e74c3c",
            root_color: "#c0392b",
            background_color: "#ffffff",
            stroke_color: "#2c3e50",
            text_color: "#2c3e50",
            inactive_color: "#bdc3c7",
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_circle_contains_all_notes() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .build();

        for label in ["C", "D", "E", "F", "G", "A", "B"] {
            assert!(svg.contains(label), "Missing note: {}", label);
        }
    }

    #[test]
    fn pitch_circle_valid_svg() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("xmlns"));
    }

    #[test]
    fn fretboard_dimensions() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .num_frets(5)
            .build();

        assert!(svg.contains("viewBox"));
    }

    #[test]
    fn fretboard_muted_string() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Muted { string: 0 })
            .build();

        assert!(svg.contains("×"));
    }
}
```

### Integration Tests

```rust
#[test]
fn roundtrip_pc_set_to_svg() {
    let pc_set = PcSet::new(vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11]);
    let svg = pc_set.to_pitch_circle_svg().build();

    // Should be valid SVG
    assert!(svg.starts_with("<svg"));

    // Should highlight correct number of notes
    let highlight_count = svg.matches("#4a90d9").count();
    assert_eq!(highlight_count, 7); // 7 notes in major scale
}
```

## Implementation Plan

### Phase 1: Core Module
- [ ] Create `music/src/svg/mod.rs` with module structure
- [ ] Implement `SvgBuilder` for low-level SVG generation
- [ ] Implement `pitch_circle_svg()` function
- [ ] Implement `PitchCircleBuilder`
- [ ] Add unit tests

### Phase 2: Fretboard Diagrams
- [ ] Implement `fretboard_positions_svg()` for vertical orientation
- [ ] Implement horizontal orientation
- [ ] Implement `fretboard_shape_svg()` integration
- [ ] Implement `FretboardBuilder`
- [ ] Add unit tests

### Phase 3: Integration
- [ ] Implement `ToPitchCircleSvg` trait
- [ ] Implement `ToFretboardSvg` trait
- [ ] Add theme presets
- [ ] Add utility functions (`save_svg`, `svg_to_data_uri`)
- [ ] Integration tests

### Phase 4: Polish
- [ ] Interval diagrams
- [ ] Fret markers (dots at frets 3, 5, 7, 9, 12)
- [ ] Barre notation
- [ ] Finger numbering
- [ ] Documentation and examples

## Open Questions

1. **Should we support `#[derive(Svg)]` macro for custom types?**
   - Could auto-generate diagram methods for user-defined chord/scale types

2. **WASM considerations?**
   - Current design should work fine, but may want to add `wasm-bindgen` exports

3. **Should SVG generation be a separate crate?**
   - Could be `music-svg` that depends on `music`
   - Keeps core library lean for users who don't need visualization

4. **Interactive SVG features?**
   - Could add `onclick` handlers for web use
   - Would require JS integration

## References

- SVG Specification: https://www.w3.org/TR/SVG2/
- Python implementation: `pitch_set_lib/typesetting_tools/svg_tools.py`
- Existing Rust types: `music/src/note/pitch_class.rs`, `music/src/fretboard/`
