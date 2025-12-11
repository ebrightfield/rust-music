//! Pitch circle (chromatic clock) SVG diagrams.

use std::collections::HashSet;

use crate::note::pitch_class::{Pc, PcIter};
use crate::note_collections::PcSet;
use crate::svg::theme::SvgTheme;
use crate::svg::util::{pc_to_coords, SvgBuilder};

/// How to label the 12 pitch classes on the circle.
#[derive(Clone, Debug)]
pub enum NoteLabels {
    /// C, C#, D, D#, E, F, F#, G, G#, A, A#, B
    Sharps,
    /// C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B
    Flats,
    /// C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B (common convention)
    Mixed,
    /// 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    PitchClass,
    /// Custom labels for all 12 positions
    Custom([&'static str; 12]),
}

impl Default for NoteLabels {
    fn default() -> Self {
        Self::Mixed
    }
}

impl NoteLabels {
    /// Get the label for a pitch class.
    pub fn label(&self, pc: &Pc) -> &'static str {
        let idx = u8::from(pc) as usize;
        match self {
            NoteLabels::Sharps => {
                ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"][idx]
            }
            NoteLabels::Flats => {
                ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"][idx]
            }
            NoteLabels::Mixed => {
                ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"][idx]
            }
            NoteLabels::PitchClass => {
                ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"][idx]
            }
            NoteLabels::Custom(labels) => labels[idx],
        }
    }
}

/// Configuration for pitch circle diagrams.
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

/// Builder for pitch circle SVGs.
///
/// # Example
/// ```
/// use music::svg::pitch_circle::PitchCircleBuilder;
/// use music::Pc;
///
/// let svg = PitchCircleBuilder::new()
///     .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])  // C major triad
///     .root(Pc::Pc0)
///     .title("C Major Triad")
///     .show_intervals(true)
///     .build();
///
/// assert!(svg.starts_with("<svg"));
/// ```
pub struct PitchCircleBuilder {
    pitch_set: HashSet<Pc>,
    root: Option<Pc>,
    config: PitchCircleConfig,
}

impl Default for PitchCircleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PitchCircleBuilder {
    /// Create a new pitch circle builder with default configuration.
    pub fn new() -> Self {
        Self {
            pitch_set: HashSet::new(),
            root: None,
            config: PitchCircleConfig::default(),
        }
    }

    /// Set the pitch classes to highlight from an iterator.
    pub fn pitches(mut self, pcs: impl IntoIterator<Item = Pc>) -> Self {
        self.pitch_set = pcs.into_iter().collect();
        self
    }

    /// Set the pitch classes from a PcSet.
    pub fn from_pc_set(mut self, pc_set: &PcSet) -> Self {
        self.pitch_set = pc_set.iter().cloned().collect();
        self
    }

    /// Set the root pitch class (will be highlighted differently).
    pub fn root(mut self, pc: Pc) -> Self {
        self.root = Some(pc);
        self
    }

    /// Set the radius of the circle.
    pub fn radius(mut self, r: u32) -> Self {
        self.config.radius = r;
        self
    }

    /// Whether to show interval lines connecting the pitch classes.
    pub fn show_intervals(mut self, show: bool) -> Self {
        self.config.show_intervals = show;
        self
    }

    /// Set the title shown above the diagram.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// Set the theme for styling.
    pub fn theme(mut self, theme: SvgTheme) -> Self {
        self.config.theme = theme;
        self
    }

    /// Set how to label the pitch classes.
    pub fn note_labels(mut self, labels: NoteLabels) -> Self {
        self.config.note_labels = labels;
        self
    }

    /// Build the SVG string.
    pub fn build(self) -> String {
        pitch_circle_svg(&self.pitch_set, self.root, &self.config)
    }
}

/// Generate SVG for a chromatic pitch circle.
pub fn pitch_circle_svg(
    pitch_set: &HashSet<Pc>,
    root: Option<Pc>,
    config: &PitchCircleConfig,
) -> String {
    let margin = 40u32;
    let title_offset = if config.title.is_some() { 25 } else { 0 };
    let width = config.radius * 2 + margin * 2;
    let height = config.radius * 2 + margin * 2 + title_offset;

    let mut svg = SvgBuilder::new(width, height);

    let cx = (width / 2) as f64;
    let cy = (config.radius + margin + title_offset) as f64;

    // Add title if present
    if let Some(ref title) = config.title {
        svg.text(cx, 18.0, title, "title-text");
    }

    // Draw background circle
    svg.circle(
        cx,
        cy,
        config.radius,
        "none",
        config.theme.inactive_color,
        1.0,
    );

    // Draw interval lines if requested
    if config.show_intervals && pitch_set.len() > 1 {
        svg.group_start("interval-lines");
        let mut sorted: Vec<_> = pitch_set.iter().collect();
        sorted.sort_by_key(|pc| u8::from(*pc));

        // Draw lines between adjacent pitch classes
        for window in sorted.windows(2) {
            let (x1, y1) = pc_to_coords(u8::from(window[0]), cx, cy, config.radius as f64);
            let (x2, y2) = pc_to_coords(u8::from(window[1]), cx, cy, config.radius as f64);
            svg.line(x1, y1, x2, y2, config.theme.highlight_color, 2.0, 0.5);
        }

        // Close the loop (connect last to first)
        if let (Some(first), Some(last)) = (sorted.first(), sorted.last()) {
            let (x1, y1) = pc_to_coords(u8::from(*last), cx, cy, config.radius as f64);
            let (x2, y2) = pc_to_coords(u8::from(*first), cx, cy, config.radius as f64);
            svg.line(x1, y1, x2, y2, config.theme.highlight_color, 2.0, 0.5);
        }
        svg.group_end();
    }

    // Draw pitch class dots
    for pc in PcIter::default() {
        let (x, y) = pc_to_coords(u8::from(&pc), cx, cy, config.radius as f64);
        let is_root = root.map_or(false, |r| r == pc);
        let is_highlighted = pitch_set.contains(&pc);

        let fill = if is_root && is_highlighted {
            config.theme.root_color
        } else if is_highlighted {
            config.theme.highlight_color
        } else {
            config.theme.background_color
        };

        let text_color = if is_highlighted {
            "#ffffff"
        } else {
            config.theme.text_color
        };

        svg.circle(x, y, 14, fill, config.theme.stroke_color, 1.5);
        svg.text_colored(x, y, config.note_labels.label(&pc), "note-text", text_color);
    }

    svg.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_circle_basic() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("xmlns"));
    }

    #[test]
    fn test_pitch_circle_contains_all_notes() {
        let svg = PitchCircleBuilder::new().pitches([Pc::Pc0]).build();

        // Check that all note labels are present
        for label in ["C", "D", "E", "F", "G", "A", "B"] {
            assert!(svg.contains(label), "Missing note: {}", label);
        }
    }

    #[test]
    fn test_pitch_circle_with_title() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .title("Test Title")
            .build();

        assert!(svg.contains("Test Title"));
    }

    #[test]
    fn test_pitch_circle_with_root() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .root(Pc::Pc0)
            .build();

        // Root should use root_color (#2d5a87)
        assert!(svg.contains("#2d5a87"));
    }

    #[test]
    fn test_pitch_circle_with_intervals() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .show_intervals(true)
            .build();

        // Should contain line elements
        assert!(svg.contains("<line"));
        assert!(svg.contains("interval-lines"));
    }

    #[test]
    fn test_pitch_circle_from_pc_set() {
        let pc_set = PcSet::new(vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11]);
        let svg = PitchCircleBuilder::new().from_pc_set(&pc_set).build();

        assert!(svg.starts_with("<svg"));
        // 7 notes should be highlighted
        let highlight_count = svg.matches(SvgTheme::default().highlight_color).count();
        assert_eq!(highlight_count, 7);
    }

    #[test]
    fn test_pitch_class_labels() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .note_labels(NoteLabels::PitchClass)
            .build();

        // Should contain numeric labels
        assert!(svg.contains(">0<"));
        assert!(svg.contains(">11<"));
    }

    #[test]
    fn test_sharp_labels() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .note_labels(NoteLabels::Sharps)
            .build();

        // Should contain sharp labels
        assert!(svg.contains("C#"));
        assert!(svg.contains("F#"));
    }

    #[test]
    fn test_flat_labels() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .note_labels(NoteLabels::Flats)
            .build();

        // Should contain flat labels
        assert!(svg.contains("Db"));
        assert!(svg.contains("Gb"));
    }

    #[test]
    fn test_different_themes() {
        // Dark theme
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .theme(SvgTheme::dark())
            .build();
        assert!(svg.contains("#5dade2")); // dark theme highlight color

        // Print theme
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .theme(SvgTheme::print())
            .build();
        assert!(svg.contains("#999999")); // print theme inactive color
    }

    #[test]
    fn test_custom_radius() {
        let svg = PitchCircleBuilder::new()
            .pitches([Pc::Pc0])
            .radius(120)
            .build();

        // Larger radius means larger viewBox
        assert!(svg.contains("viewBox=\"0 0 320"));
    }
}
