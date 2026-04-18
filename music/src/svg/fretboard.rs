//! Fretboard SVG diagrams.
//!
//! Generates chord diagrams and fretboard visualizations.

use std::collections::HashSet;

use crate::fretboard::fretboard_shape::FretboardShape;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};
use crate::svg::theme::SvgTheme;
use crate::svg::util::SvgBuilder;

/// Orientation of the fretboard diagram.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Orientation {
    /// Standard chord diagram orientation (nut at top, frets going down).
    #[default]
    Vertical,
    /// Tab-like orientation (strings horizontal, frets vertical).
    Horizontal,
}

/// Configuration for fretboard diagrams.
#[derive(Clone, Debug)]
pub struct FretboardConfig {
    /// Number of frets to display.
    pub num_frets: u8,
    /// Starting fret number (0 = open position with nut).
    pub start_fret: u8,
    /// Whether to show fret numbers.
    pub show_fret_numbers: bool,
    /// Whether to show string names at the bottom.
    pub show_string_names: bool,
    /// Diagram orientation.
    pub orientation: Orientation,
    /// Optional title above the diagram.
    pub title: Option<String>,
    /// Custom string names (defaults to E A D G B E for 6 strings).
    pub string_names: Option<Vec<String>>,
    /// Color theme.
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

/// A position on the fretboard.
#[derive(Clone, Debug, PartialEq)]
pub enum FretPosition {
    /// A fretted note on a specific string and fret.
    Fretted { string: u8, fret: u8 },
    /// An open string.
    Open { string: u8 },
    /// A muted string (shown with X).
    Muted { string: u8 },
}

impl FretPosition {
    /// Get the string number for this position.
    pub fn string(&self) -> u8 {
        match self {
            FretPosition::Fretted { string, .. } => *string,
            FretPosition::Open { string } => *string,
            FretPosition::Muted { string } => *string,
        }
    }
}

/// A barre (index finger bar) across multiple strings.
#[derive(Clone, Debug, PartialEq)]
pub struct Barre {
    /// The fret where the barre is placed.
    pub fret: u8,
    /// Starting string (lower string number, closer to low E).
    pub from_string: u8,
    /// Ending string (higher string number, closer to high E).
    pub to_string: u8,
}

impl Barre {
    /// Create a new barre.
    pub fn new(fret: u8, from_string: u8, to_string: u8) -> Self {
        Self {
            fret,
            from_string: from_string.min(to_string),
            to_string: from_string.max(to_string),
        }
    }

    /// Create a full barre across all strings at a fret.
    pub fn full(fret: u8, num_strings: u8) -> Self {
        Self {
            fret,
            from_string: 0,
            to_string: num_strings.saturating_sub(1),
        }
    }
}

/// Finger numbering for a position (1 = index, 2 = middle, 3 = ring, 4 = pinky, T = thumb).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Finger {
    /// Index finger (1).
    Index,
    /// Middle finger (2).
    Middle,
    /// Ring finger (3).
    Ring,
    /// Pinky finger (4).
    Pinky,
    /// Thumb (T).
    Thumb,
}

impl Finger {
    /// Get the display label for this finger.
    pub fn label(&self) -> &'static str {
        match self {
            Finger::Index => "1",
            Finger::Middle => "2",
            Finger::Ring => "3",
            Finger::Pinky => "4",
            Finger::Thumb => "T",
        }
    }
}

/// Builder for fretboard SVG diagrams.
///
/// # Example
/// ```
/// use music::svg::fretboard::{FretboardBuilder, FretPosition};
///
/// // Create an open C chord diagram
/// let svg = FretboardBuilder::new()
///     .position(FretPosition::Muted { string: 0 })
///     .position(FretPosition::Fretted { string: 1, fret: 3 })
///     .position(FretPosition::Fretted { string: 2, fret: 2 })
///     .position(FretPosition::Open { string: 3 })
///     .position(FretPosition::Fretted { string: 4, fret: 1 })
///     .position(FretPosition::Open { string: 5 })
///     .title("C Major")
///     .build();
///
/// assert!(svg.starts_with("<svg"));
/// ```
pub struct FretboardBuilder<'a> {
    shape: Option<&'a FretboardShape<'a>>,
    positions: Vec<FretPosition>,
    root_positions: HashSet<(u8, u8)>,
    barres: Vec<Barre>,
    fingerings: std::collections::HashMap<(u8, u8), Finger>,
    config: FretboardConfig,
}

impl<'a> Default for FretboardBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FretboardBuilder<'a> {
    /// Create a new fretboard builder with default configuration.
    pub fn new() -> Self {
        Self {
            shape: None,
            positions: Vec::new(),
            root_positions: HashSet::new(),
            barres: Vec::new(),
            fingerings: std::collections::HashMap::new(),
            config: FretboardConfig::default(),
        }
    }

    /// Create diagram from a FretboardShape.
    pub fn from_shape(mut self, shape: &'a FretboardShape<'a>) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Add a single position to the diagram.
    pub fn position(mut self, pos: FretPosition) -> Self {
        self.positions.push(pos);
        self
    }

    /// Add multiple positions to the diagram.
    pub fn positions(mut self, positions: impl IntoIterator<Item = FretPosition>) -> Self {
        self.positions.extend(positions);
        self
    }

    /// Mark a position as a root note (will be highlighted differently).
    pub fn root_at(mut self, string: u8, fret: u8) -> Self {
        self.root_positions.insert((string, fret));
        self
    }

    /// Set the starting fret (0 for open position with nut).
    pub fn start_fret(mut self, fret: u8) -> Self {
        self.config.start_fret = fret;
        self
    }

    /// Set the number of frets to display.
    pub fn num_frets(mut self, n: u8) -> Self {
        self.config.num_frets = n;
        self
    }

    /// Set the diagram orientation.
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.config.orientation = o;
        self
    }

    /// Set a title for the diagram.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// Set the color theme.
    pub fn theme(mut self, theme: SvgTheme) -> Self {
        self.config.theme = theme;
        self
    }

    /// Whether to show fret numbers.
    pub fn show_fret_numbers(mut self, show: bool) -> Self {
        self.config.show_fret_numbers = show;
        self
    }

    /// Whether to show string names.
    pub fn show_string_names(mut self, show: bool) -> Self {
        self.config.show_string_names = show;
        self
    }

    /// Set custom string names.
    pub fn string_names(mut self, names: Vec<String>) -> Self {
        self.config.string_names = Some(names);
        self
    }

    /// Add a barre across multiple strings at a fret.
    ///
    /// # Example
    /// ```
    /// use music::svg::fretboard::{FretboardBuilder, Barre};
    ///
    /// // F major barre chord
    /// let svg = FretboardBuilder::new()
    ///     .barre(Barre::new(1, 0, 5))  // Full barre at fret 1
    ///     .build();
    /// ```
    pub fn barre(mut self, b: Barre) -> Self {
        self.barres.push(b);
        self
    }

    /// Add a barre from fret and string range.
    pub fn barre_at(mut self, fret: u8, from_string: u8, to_string: u8) -> Self {
        self.barres.push(Barre::new(fret, from_string, to_string));
        self
    }

    /// Add a full barre across all strings at a fret.
    pub fn full_barre(mut self, fret: u8, num_strings: u8) -> Self {
        self.barres.push(Barre::full(fret, num_strings));
        self
    }

    /// Set fingering for a position.
    ///
    /// The finger number will be displayed inside the dot.
    pub fn finger_at(mut self, string: u8, fret: u8, finger: Finger) -> Self {
        self.fingerings.insert((string, fret), finger);
        self
    }

    /// Build the SVG string.
    pub fn build(self) -> String {
        if let Some(shape) = self.shape {
            fretboard_shape_svg(
                shape,
                &self.root_positions,
                &self.barres,
                &self.fingerings,
                &self.config,
            )
        } else {
            fretboard_positions_svg(
                &self.positions,
                &self.root_positions,
                &self.barres,
                &self.fingerings,
                &self.config,
            )
        }
    }
}

/// Generate SVG from a FretboardShape.
pub fn fretboard_shape_svg(
    shape: &FretboardShape,
    root_positions: &HashSet<(u8, u8)>,
    barres: &[Barre],
    fingerings: &std::collections::HashMap<(u8, u8), Finger>,
    config: &FretboardConfig,
) -> String {
    let positions: Vec<FretPosition> = shape
        .fretted_notes
        .iter()
        .enumerate()
        .map(|(string, note)| {
            let string = string as u8;
            match note {
                FrettedNote::Muted { .. } => FretPosition::Muted { string },
                FrettedNote::Sounded(SoundedNote { fret: 0, .. }) => FretPosition::Open { string },
                FrettedNote::Sounded(SoundedNote { fret, .. }) => {
                    FretPosition::Fretted { string, fret: *fret }
                }
            }
        })
        .collect();

    // Auto-calculate start_fret if not at open position
    let mut adjusted_config = config.clone();
    if config.start_fret == 0 {
        // Check if shape needs different start fret
        let min_fret = positions
            .iter()
            .filter_map(|p| match p {
                FretPosition::Fretted { fret, .. } if *fret > 0 => Some(*fret),
                _ => None,
            })
            .min()
            .unwrap_or(0);

        let max_fret = positions
            .iter()
            .filter_map(|p| match p {
                FretPosition::Fretted { fret, .. } => Some(*fret),
                _ => None,
            })
            .max()
            .unwrap_or(0);

        // If lowest fret is above the default range, adjust start_fret
        if min_fret > config.num_frets && !shape.contains_open_strings() {
            adjusted_config.start_fret = min_fret.saturating_sub(1);
            // Ensure we show enough frets
            if max_fret > adjusted_config.start_fret + adjusted_config.num_frets {
                adjusted_config.num_frets = max_fret - adjusted_config.start_fret + 1;
            }
        }
    }

    fretboard_positions_svg(&positions, root_positions, barres, fingerings, &adjusted_config)
}

/// Translate an absolute fret number into the index of the visible fret space
/// (1-based) within the diagram window defined by `config.start_fret` and
/// `config.num_frets`. Returns `None` if the fret is below the visible window,
/// so callers can skip it cleanly. Values greater than `config.num_frets` are
/// returned and expected to be filtered by the caller's upper-bound check.
///
/// When `start_fret == 0`, the leftmost (top) diagram line is the nut and the
/// first visible space is fret 1. When `start_fret > 0`, the leftmost line
/// represents the wire between fret `start_fret` and `start_fret + 1`, so the
/// first visible space is `start_fret + 1`.
fn relative_fret(fret: u8, config: &FretboardConfig) -> Option<u8> {
    if fret < config.start_fret + 1 && config.start_fret > 0 {
        // Note sits at or below the hidden boundary.
        return None;
    }
    Some(fret.saturating_sub(config.start_fret))
}

/// Generate SVG from raw fret positions.
pub fn fretboard_positions_svg(
    positions: &[FretPosition],
    root_positions: &HashSet<(u8, u8)>,
    barres: &[Barre],
    fingerings: &std::collections::HashMap<(u8, u8), Finger>,
    config: &FretboardConfig,
) -> String {
    let num_strings = positions
        .iter()
        .map(|p| p.string())
        .max()
        .map(|s| s + 1)
        .unwrap_or(6);

    match config.orientation {
        Orientation::Vertical => {
            draw_vertical_fretboard(positions, root_positions, barres, fingerings, num_strings, config)
        }
        Orientation::Horizontal => {
            draw_horizontal_fretboard(positions, root_positions, barres, fingerings, num_strings, config)
        }
    }
}

fn draw_vertical_fretboard(
    positions: &[FretPosition],
    root_positions: &HashSet<(u8, u8)>,
    barres: &[Barre],
    fingerings: &std::collections::HashMap<(u8, u8), Finger>,
    num_strings: u8,
    config: &FretboardConfig,
) -> String {
    let string_spacing = 30u32;
    let fret_spacing = 40u32;
    let margin_top = if config.title.is_some() { 50 } else { 30 };
    let margin_left = if config.show_fret_numbers && config.start_fret > 0 {
        40
    } else {
        20
    };
    let margin_right = 30u32;
    let margin_bottom = if config.show_string_names { 40 } else { 20 };
    let dot_radius = 10u32;

    let width = margin_left + (num_strings as u32 - 1) * string_spacing + margin_right;
    let height = margin_top + config.num_frets as u32 * fret_spacing + margin_bottom;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 20.0, title, "title-text");
    }

    // Nut (thick line at top if starting at fret 0)
    if config.start_fret == 0 {
        let x1 = margin_left as f64;
        let x2 = (margin_left + (num_strings as u32 - 1) * string_spacing) as f64;
        svg.line(
            x1,
            margin_top as f64,
            x2,
            margin_top as f64,
            config.theme.stroke_color,
            4.0,
            1.0,
        );
    }

    // Frets (horizontal lines)
    for fret in 0..=config.num_frets {
        if fret == 0 && config.start_fret == 0 {
            continue; // Skip if nut already drawn
        }
        let y = (margin_top + fret as u32 * fret_spacing) as f64;
        let x1 = margin_left as f64;
        let x2 = (margin_left + (num_strings as u32 - 1) * string_spacing) as f64;
        svg.line(x1, y, x2, y, config.theme.stroke_color, 1.0, 1.0);
    }

    // Strings (vertical lines)
    for string in 0..num_strings {
        let x = (margin_left + string as u32 * string_spacing) as f64;
        let y1 = margin_top as f64;
        let y2 = (margin_top + config.num_frets as u32 * fret_spacing) as f64;
        svg.line(x, y1, x, y2, config.theme.stroke_color, 1.0, 1.0);
    }

    // Fret number (if not at open position)
    if config.show_fret_numbers && config.start_fret > 0 {
        svg.text(
            (margin_left - 20) as f64,
            (margin_top + fret_spacing / 2) as f64,
            &config.start_fret.to_string(),
            "fret-text",
        );
    }

    // String names
    if config.show_string_names {
        let default_names: Vec<String> = vec!["E", "A", "D", "G", "B", "E"]
            .into_iter()
            .take(num_strings as usize)
            .map(String::from)
            .collect();
        let names = config.string_names.as_ref().unwrap_or(&default_names);

        for (i, name) in names.iter().enumerate() {
            let x = (margin_left + i as u32 * string_spacing) as f64;
            let y = (margin_top + config.num_frets as u32 * fret_spacing + 25) as f64;
            svg.text(x, y, name, "string-text");
        }
    }

    // Draw fret markers (dots at frets 3, 5, 7, 9, 12, 15, etc.)
    let marker_frets = [3, 5, 7, 9, 15, 17, 19, 21];
    let double_marker_frets = [12, 24];
    let fretboard_width = (num_strings as u32 - 1) * string_spacing;
    let center_x = margin_left as f64 + fretboard_width as f64 / 2.0;

    for &fret in &marker_frets {
        if fret > config.start_fret && fret <= config.start_fret + config.num_frets {
            let relative_fret = fret - config.start_fret;
            let y = margin_top as f64 + (relative_fret as f64 - 0.5) * fret_spacing as f64;
            svg.circle(center_x, y, 4, config.theme.inactive_color, "none", 0.0);
        }
    }

    for &fret in &double_marker_frets {
        if fret > config.start_fret && fret <= config.start_fret + config.num_frets {
            let relative_fret = fret - config.start_fret;
            let y = margin_top as f64 + (relative_fret as f64 - 0.5) * fret_spacing as f64;
            // Two dots, offset from center
            let offset = fretboard_width as f64 * 0.25;
            svg.circle(center_x - offset, y, 4, config.theme.inactive_color, "none", 0.0);
            svg.circle(center_x + offset, y, 4, config.theme.inactive_color, "none", 0.0);
        }
    }

    // Collect positions covered by barres (to avoid drawing dots for them)
    let mut barre_positions: HashSet<(u8, u8)> = HashSet::new();
    for barre in barres {
        for string in barre.from_string..=barre.to_string {
            barre_positions.insert((string, barre.fret));
        }
    }

    // Draw barres first (before dots so dots can overlay)
    for barre in barres {
        let Some(relative_fret) = relative_fret(barre.fret, config) else { continue };

        if relative_fret >= 1 && relative_fret <= config.num_frets {
            let x1 = (margin_left + barre.from_string as u32 * string_spacing) as f64;
            let x2 = (margin_left + barre.to_string as u32 * string_spacing) as f64;
            let y = margin_top as f64
                + (relative_fret as f64 * fret_spacing as f64)
                - (fret_spacing as f64 / 2.0);

            // Draw barre as a rounded rectangle
            let barre_height = 12.0;
            svg.rect(
                x1 - 5.0,
                y - barre_height / 2.0,
                x2 - x1 + 10.0,
                barre_height,
                config.theme.highlight_color,
                config.theme.stroke_color,
                1.0,
            );
        }
    }

    // Draw positions
    for pos in positions {
        match pos {
            FretPosition::Muted { string } => {
                let x = (margin_left + *string as u32 * string_spacing) as f64;
                let y = (margin_top - 15) as f64;
                svg.text_colored(x, y, "×", "string-text", config.theme.text_color);
            }
            FretPosition::Open { string } => {
                let x = (margin_left + *string as u32 * string_spacing) as f64;
                let y = (margin_top - 12) as f64;
                svg.circle(x, y, 6, "none", config.theme.stroke_color, 1.5);
            }
            FretPosition::Fretted { string, fret } => {
                // Skip if covered by a barre
                if barre_positions.contains(&(*string, *fret)) {
                    continue;
                }

                // Only draw if within visible range
                let Some(relative_fret) = relative_fret(*fret, config) else { continue };
                if relative_fret >= 1 && relative_fret <= config.num_frets {
                    let x = (margin_left + *string as u32 * string_spacing) as f64;
                    let y = margin_top as f64
                        + (relative_fret as f64 * fret_spacing as f64)
                        - (fret_spacing as f64 / 2.0);

                    let is_root = root_positions.contains(&(*string, *fret));
                    let fill = if is_root {
                        config.theme.root_color
                    } else {
                        config.theme.highlight_color
                    };

                    svg.circle(x, y, dot_radius, fill, config.theme.stroke_color, 1.0);

                    // Draw finger number if specified
                    if let Some(finger) = fingerings.get(&(*string, *fret)) {
                        svg.text_colored(x, y + 4.0, finger.label(), "note-text", "#ffffff");
                    }
                }
            }
        }
    }

    svg.build()
}

fn draw_horizontal_fretboard(
    positions: &[FretPosition],
    root_positions: &HashSet<(u8, u8)>,
    barres: &[Barre],
    fingerings: &std::collections::HashMap<(u8, u8), Finger>,
    num_strings: u8,
    config: &FretboardConfig,
) -> String {
    let string_spacing = 25u32;
    let fret_spacing = 45u32;
    let margin_top = if config.title.is_some() { 50 } else { 30 };
    let margin_left = if config.show_string_names { 40 } else { 20 };
    let margin_right = 30u32;
    let margin_bottom = if config.show_fret_numbers { 35 } else { 20 };
    let dot_radius = 9u32;

    let width = margin_left + config.num_frets as u32 * fret_spacing + margin_right;
    let height = margin_top + (num_strings as u32 - 1) * string_spacing + margin_bottom;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 20.0, title, "title-text");
    }

    // Nut (thick line at left if starting at fret 0)
    if config.start_fret == 0 {
        let y1 = margin_top as f64;
        let y2 = (margin_top + (num_strings as u32 - 1) * string_spacing) as f64;
        svg.line(
            margin_left as f64,
            y1,
            margin_left as f64,
            y2,
            config.theme.stroke_color,
            4.0,
            1.0,
        );
    }

    // Frets (vertical lines)
    for fret in 0..=config.num_frets {
        if fret == 0 && config.start_fret == 0 {
            continue; // Skip if nut already drawn
        }
        let x = (margin_left + fret as u32 * fret_spacing) as f64;
        let y1 = margin_top as f64;
        let y2 = (margin_top + (num_strings as u32 - 1) * string_spacing) as f64;
        svg.line(x, y1, x, y2, config.theme.stroke_color, 1.0, 1.0);
    }

    // Strings (horizontal lines) - note: in horizontal view, string 0 is at bottom
    for string in 0..num_strings {
        let y = (margin_top + (num_strings - 1 - string) as u32 * string_spacing) as f64;
        let x1 = margin_left as f64;
        let x2 = (margin_left + config.num_frets as u32 * fret_spacing) as f64;
        svg.line(x1, y, x2, y, config.theme.stroke_color, 1.0, 1.0);
    }

    // String names (on the left)
    if config.show_string_names {
        let default_names: Vec<String> = vec!["E", "A", "D", "G", "B", "E"]
            .into_iter()
            .take(num_strings as usize)
            .map(String::from)
            .collect();
        let names = config.string_names.as_ref().unwrap_or(&default_names);

        for (i, name) in names.iter().enumerate() {
            let y = (margin_top + (num_strings - 1 - i as u8) as u32 * string_spacing) as f64;
            svg.text((margin_left - 20) as f64, y, name, "string-text");
        }
    }

    // Fret numbers
    if config.show_fret_numbers {
        for fret in 1..=config.num_frets {
            let actual_fret = config.start_fret + fret;
            let x = margin_left as f64 + (fret as f64 - 0.5) * fret_spacing as f64;
            let y = (margin_top + (num_strings as u32 - 1) * string_spacing + 20) as f64;
            svg.text(x, y, &actual_fret.to_string(), "fret-text");
        }
    }

    // Collect positions covered by barres
    let mut barre_positions: HashSet<(u8, u8)> = HashSet::new();
    for barre in barres {
        for string in barre.from_string..=barre.to_string {
            barre_positions.insert((string, barre.fret));
        }
    }

    // Draw barres (horizontal bar in horizontal view means vertical rectangle)
    for barre in barres {
        let Some(relative_fret) = relative_fret(barre.fret, config) else { continue };

        if relative_fret >= 1 && relative_fret <= config.num_frets {
            let x = margin_left as f64 + (relative_fret as f64 - 0.5) * fret_spacing as f64;
            let y1 = (margin_top + (num_strings - 1 - barre.to_string) as u32 * string_spacing) as f64;
            let y2 = (margin_top + (num_strings - 1 - barre.from_string) as u32 * string_spacing) as f64;

            // Draw barre as a rounded rectangle (vertical in horizontal layout)
            let barre_width = 12.0;
            svg.rect(
                x - barre_width / 2.0,
                y1 - 5.0,
                barre_width,
                y2 - y1 + 10.0,
                config.theme.highlight_color,
                config.theme.stroke_color,
                1.0,
            );
        }
    }

    // Draw positions
    for pos in positions {
        match pos {
            FretPosition::Muted { string } => {
                let y = (margin_top + (num_strings - 1 - *string) as u32 * string_spacing) as f64;
                let x = (margin_left - 10) as f64;
                svg.text_colored(x, y, "×", "string-text", config.theme.text_color);
            }
            FretPosition::Open { string } => {
                // Place the open-string indicator straddling the nut with the
                // bulk of the circle to its left (players read open strings as
                // living "before" the nut, not in the first fret space).
                let y = (margin_top + (num_strings - 1 - *string) as u32 * string_spacing) as f64;
                let x = margin_left as f64 - 3.0;
                svg.circle(x, y, 5, "none", config.theme.stroke_color, 1.5);
            }
            FretPosition::Fretted { string, fret } => {
                // Skip if covered by barre
                if barre_positions.contains(&(*string, *fret)) {
                    continue;
                }

                let Some(relative_fret) = relative_fret(*fret, config) else { continue };

                if relative_fret >= 1 && relative_fret <= config.num_frets {
                    let x = margin_left as f64
                        + (relative_fret as f64 - 0.5) * fret_spacing as f64;
                    let y =
                        (margin_top + (num_strings - 1 - *string) as u32 * string_spacing) as f64;

                    let is_root = root_positions.contains(&(*string, *fret));
                    let fill = if is_root {
                        config.theme.root_color
                    } else {
                        config.theme.highlight_color
                    };

                    svg.circle(x, y, dot_radius, fill, config.theme.stroke_color, 1.0);

                    // Draw finger number if specified
                    if let Some(finger) = fingerings.get(&(*string, *fret)) {
                        svg.text_colored(x, y + 4.0, finger.label(), "note-text", "#ffffff");
                    }
                }
            }
        }
    }

    svg.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fretboard::STD_6STR_GTR;

    #[test]
    fn test_fretboard_builder_basic() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_fretboard_builder_open_c_chord() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Muted { string: 0 })
            .position(FretPosition::Fretted { string: 1, fret: 3 })
            .position(FretPosition::Fretted { string: 2, fret: 2 })
            .position(FretPosition::Open { string: 3 })
            .position(FretPosition::Fretted { string: 4, fret: 1 })
            .position(FretPosition::Open { string: 5 })
            .title("C Major")
            .build();

        assert!(svg.contains("C Major"));
        assert!(svg.contains("×")); // Muted string
    }

    #[test]
    fn test_fretboard_muted_string() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Muted { string: 0 })
            .build();

        assert!(svg.contains("×"));
    }

    #[test]
    fn test_fretboard_open_string() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Open { string: 0 })
            .build();

        // Open string is drawn as an unfilled circle near the nut
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_fretboard_with_nut() {
        let svg = FretboardBuilder::new()
            .start_fret(0)
            .position(FretPosition::Open { string: 0 })
            .build();

        // Nut should be drawn with stroke-width="4"
        assert!(svg.contains("stroke-width=\"4\""));
    }

    #[test]
    fn test_fretboard_without_nut() {
        let svg = FretboardBuilder::new()
            .start_fret(5)
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        // Should contain fret number
        assert!(svg.contains(">5<"));
    }

    #[test]
    fn test_fretboard_root_positions() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 1, fret: 3 })
            .root_at(1, 3)
            .build();

        // Root should use root_color
        assert!(svg.contains(SvgTheme::default().root_color));
    }

    #[test]
    fn test_fretboard_horizontal_orientation() {
        let svg = FretboardBuilder::new()
            .orientation(Orientation::Horizontal)
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        assert!(svg.starts_with("<svg"));
        // Horizontal orientation has different layout
        // Strings should be horizontal, so y values should span the string range
    }

    #[test]
    fn test_fretboard_custom_string_names() {
        let svg = FretboardBuilder::new()
            .string_names(vec![
                "B".to_string(),
                "E".to_string(),
                "A".to_string(),
                "D".to_string(),
                "G".to_string(),
                "B".to_string(),
                "E".to_string(),
            ])
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        assert!(svg.contains(">B<"));
    }

    #[test]
    fn test_fretboard_from_shape() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        let svg = FretboardBuilder::new()
            .from_shape(&shape)
            .title("C Major")
            .build();

        assert!(svg.contains("C Major"));
        assert!(svg.contains("×")); // First string is muted
    }

    #[test]
    fn test_fretboard_from_shape_auto_start_fret() {
        // Barre chord at 5th position
        let shape = FretboardShape::from_string("5-7-7-6-5-5", &STD_6STR_GTR).unwrap();
        let svg = FretboardBuilder::new()
            .from_shape(&shape)
            .title("A Major Barre")
            .build();

        assert!(svg.contains("A Major Barre"));
        // Should auto-adjust start_fret
    }

    #[test]
    fn test_fretboard_dimensions() {
        let svg = FretboardBuilder::new()
            .num_frets(4)
            .position(FretPosition::Fretted { string: 0, fret: 1 })
            .build();

        assert!(svg.contains("viewBox"));
    }

    #[test]
    fn test_fretboard_theme() {
        let svg = FretboardBuilder::new()
            .theme(SvgTheme::dark())
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        assert!(svg.contains(SvgTheme::dark().highlight_color));
    }

    #[test]
    fn test_fretboard_hide_fret_numbers() {
        let svg = FretboardBuilder::new()
            .start_fret(5)
            .show_fret_numbers(false)
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        // Should not contain fret number "5" as standalone text element
        // When fret numbers are hidden, the left margin is smaller
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn test_fretboard_hide_string_names() {
        let svg = FretboardBuilder::new()
            .show_string_names(false)
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .build();

        // Should not contain "E", "A", "D", "G", "B" as string labels
        // When string names are hidden, the bottom margin is smaller
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_horizontal_fretboard_layout() {
        let svg = FretboardBuilder::new()
            .orientation(Orientation::Horizontal)
            .num_frets(5)
            .position(FretPosition::Fretted { string: 0, fret: 1 })
            .position(FretPosition::Fretted { string: 2, fret: 3 })
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_fret_markers_displayed() {
        // Test that fret markers (dots at positions 3, 5, 7, 9, 12) are shown
        let svg = FretboardBuilder::new()
            .start_fret(0)
            .num_frets(12)
            .position(FretPosition::Fretted { string: 0, fret: 1 })
            .build();

        // There should be multiple small circles for fret markers
        // Count circles with radius 4 (marker dots)
        let marker_count = svg.matches("r=\"4\"").count();
        assert!(marker_count > 0, "Should have fret markers");
    }

    #[test]
    fn test_barre_chord() {
        // F major barre chord: 1-3-3-2-1-1 with barre at fret 1
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 0, fret: 1 })
            .position(FretPosition::Fretted { string: 1, fret: 3 })
            .position(FretPosition::Fretted { string: 2, fret: 3 })
            .position(FretPosition::Fretted { string: 3, fret: 2 })
            .position(FretPosition::Fretted { string: 4, fret: 1 })
            .position(FretPosition::Fretted { string: 5, fret: 1 })
            .barre(Barre::new(1, 0, 5))  // Barre across strings 0-5 at fret 1
            .title("F Major")
            .build();

        assert!(svg.contains("F Major"));
        // Barre should be drawn as rect
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_barre_at_method() {
        // start_fret(4) means the first visible space is fret 5, so a barre
        // at fret 5 lands in the first visible space and renders.
        let svg = FretboardBuilder::new()
            .barre_at(5, 0, 5)
            .start_fret(4)
            .build();

        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_full_barre_method() {
        let svg = FretboardBuilder::new()
            .full_barre(3, 6)  // Full barre at 3rd fret across 6 strings
            .build();

        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_barre_horizontal_orientation() {
        let svg = FretboardBuilder::new()
            .orientation(Orientation::Horizontal)
            .barre(Barre::new(1, 0, 5))
            .build();

        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_finger_numbering() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 1, fret: 3 })
            .position(FretPosition::Fretted { string: 2, fret: 2 })
            .position(FretPosition::Fretted { string: 4, fret: 1 })
            .finger_at(1, 3, Finger::Ring)     // Ring finger on string 1, fret 3
            .finger_at(2, 2, Finger::Middle)   // Middle finger on string 2, fret 2
            .finger_at(4, 1, Finger::Index)    // Index finger on string 4, fret 1
            .title("C Major with Fingering")
            .build();

        assert!(svg.contains("C Major with Fingering"));
        // Finger numbers should be displayed as text
        assert!(svg.contains(">1<")); // Index
        assert!(svg.contains(">2<")); // Middle
        assert!(svg.contains(">3<")); // Ring
    }

    #[test]
    fn test_finger_numbering_horizontal() {
        let svg = FretboardBuilder::new()
            .orientation(Orientation::Horizontal)
            .position(FretPosition::Fretted { string: 0, fret: 5 })
            .finger_at(0, 5, Finger::Pinky)
            .build();

        assert!(svg.contains(">4<")); // Pinky
    }

    #[test]
    fn test_thumb_finger() {
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 0, fret: 2 })
            .finger_at(0, 2, Finger::Thumb)
            .build();

        assert!(svg.contains(">T<")); // Thumb
    }

    #[test]
    fn test_barre_prevents_individual_dots() {
        // When a barre covers positions, those positions shouldn't be drawn as separate dots
        let svg = FretboardBuilder::new()
            .position(FretPosition::Fretted { string: 0, fret: 1 })
            .position(FretPosition::Fretted { string: 1, fret: 1 })
            .position(FretPosition::Fretted { string: 2, fret: 1 })
            .barre(Barre::new(1, 0, 2))  // Barre covers strings 0-2 at fret 1
            .build();

        // Should have rect for barre
        assert!(svg.contains("<rect"));
        // Count circles - should not have individual dots for the barre positions
        // Only other potential circles are for open strings or other non-barre notes
    }

    #[test]
    fn test_barre_new_normalizes_order() {
        // Barre::new should work regardless of from/to order
        let barre1 = Barre::new(5, 0, 5);
        let barre2 = Barre::new(5, 5, 0);  // Reversed order

        assert_eq!(barre1.from_string, 0);
        assert_eq!(barre1.to_string, 5);
        assert_eq!(barre2.from_string, 0);
        assert_eq!(barre2.to_string, 5);
    }
}
