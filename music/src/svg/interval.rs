//! Interval visualization diagrams.
//!
//! Generates SVG diagrams for visualizing musical intervals, including:
//! - Linear interval bars showing semitone distances
//! - Interval vectors (traditional set theory representation)
//! - Interval matrices showing relationships between pitch classes

use crate::note_collections::geometry::IntervalMatrix;
use crate::note_collections::PcSet;
use crate::svg::theme::SvgTheme;
use crate::svg::util::SvgBuilder;
use crate::Pc;

/// Configuration for interval diagrams.
#[derive(Clone, Debug)]
pub struct IntervalConfig {
    /// Optional title above the diagram.
    pub title: Option<String>,
    /// Color theme.
    pub theme: SvgTheme,
    /// Width of bars in linear diagrams.
    pub bar_width: u32,
    /// Height of bars in linear diagrams.
    pub bar_height: u32,
    /// Cell size for matrix diagrams.
    pub cell_size: u32,
}

impl Default for IntervalConfig {
    fn default() -> Self {
        Self {
            title: None,
            theme: SvgTheme::default(),
            bar_width: 30,
            bar_height: 150,
            cell_size: 35,
        }
    }
}

/// Builder for interval diagrams.
///
/// # Example
/// ```
/// use music::svg::interval::IntervalBuilder;
/// use music::Pc;
///
/// // Create an interval vector diagram for a major triad
/// let svg = IntervalBuilder::new()
///     .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
///     .title("Major Triad Intervals")
///     .build_vector();
///
/// assert!(svg.starts_with("<svg"));
/// ```
pub struct IntervalBuilder {
    pitch_set: Vec<Pc>,
    config: IntervalConfig,
}

impl Default for IntervalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IntervalBuilder {
    /// Create a new interval builder.
    pub fn new() -> Self {
        Self {
            pitch_set: Vec::new(),
            config: IntervalConfig::default(),
        }
    }

    /// Set the pitch classes to analyze.
    pub fn pitches(mut self, pcs: impl IntoIterator<Item = Pc>) -> Self {
        self.pitch_set = pcs.into_iter().collect();
        self
    }

    /// Create from a PcSet.
    pub fn from_pc_set(mut self, pc_set: &PcSet) -> Self {
        self.pitch_set = pc_set.iter().copied().collect();
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// Set the color theme.
    pub fn theme(mut self, theme: SvgTheme) -> Self {
        self.config.theme = theme;
        self
    }

    /// Set bar dimensions for linear diagrams.
    pub fn bar_size(mut self, width: u32, height: u32) -> Self {
        self.config.bar_width = width;
        self.config.bar_height = height;
        self
    }

    /// Set cell size for matrix diagrams.
    pub fn cell_size(mut self, size: u32) -> Self {
        self.config.cell_size = size;
        self
    }

    /// Build an interval vector bar chart.
    ///
    /// Shows a bar chart of interval class counts (ic 1-6 for reduced,
    /// or ic 0-11 for full).
    pub fn build_vector(self) -> String {
        let pc_set = PcSet::new(self.pitch_set);
        let matrix = IntervalMatrix::new(&pc_set);
        let vector = matrix.reduced_interval_vector();

        build_interval_vector_svg(&vector, &self.config)
    }

    /// Build a full interval vector bar chart (0-11 semitones).
    pub fn build_full_vector(self) -> String {
        let pc_set = PcSet::new(self.pitch_set);
        let matrix = IntervalMatrix::new(&pc_set);
        let vector = matrix.interval_vector();

        build_full_interval_vector_svg(&vector, &self.config)
    }

    /// Build an interval matrix grid.
    ///
    /// Shows a grid where each cell contains the interval between
    /// the row and column pitch classes.
    pub fn build_matrix(self) -> String {
        let pc_set = PcSet::new(self.pitch_set);
        let matrix = IntervalMatrix::new(&pc_set);

        build_interval_matrix_svg(&matrix, &self.config)
    }

    /// Build a linear interval diagram.
    ///
    /// Shows the intervals as arcs connecting adjacent notes on a line.
    pub fn build_linear(self) -> String {
        build_linear_interval_svg(&self.pitch_set, &self.config)
    }
}

/// Build SVG for a reduced interval vector (ic 1-6).
fn build_interval_vector_svg(vector: &[usize; 6], config: &IntervalConfig) -> String {
    let margin = 40u32;
    let title_height = if config.title.is_some() { 30 } else { 0 };
    let label_height = 25u32;
    let spacing = 10u32;

    let total_bars_width = 6 * config.bar_width + 5 * spacing;
    let width = margin * 2 + total_bars_width;
    let height = margin + title_height + config.bar_height + label_height;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 25.0, title, "title-text");
    }

    // Find max for scaling
    let max_val = *vector.iter().max().unwrap_or(&1).max(&1);
    let scale = config.bar_height as f64 / max_val as f64;

    // Draw bars
    let ic_labels = ["1", "2", "3", "4", "5", "6"];
    for (i, &count) in vector.iter().enumerate() {
        let x = margin + i as u32 * (config.bar_width + spacing);
        let bar_height = (count as f64 * scale) as u32;
        let y = margin + title_height + (config.bar_height - bar_height);

        // Bar
        svg.rect(
            x as f64,
            y as f64,
            config.bar_width as f64,
            bar_height as f64,
            config.theme.highlight_color,
            config.theme.stroke_color,
            1.0,
        );

        // Value on top of bar
        if count > 0 {
            svg.text(
                (x + config.bar_width / 2) as f64,
                (y - 5) as f64,
                &count.to_string(),
                "fret-text",
            );
        }

        // IC label below
        svg.text(
            (x + config.bar_width / 2) as f64,
            (margin + title_height + config.bar_height + 15) as f64,
            ic_labels[i],
            "string-text",
        );
    }

    svg.build()
}

/// Build SVG for a full interval vector (0-11 semitones).
fn build_full_interval_vector_svg(vector: &[usize; 12], config: &IntervalConfig) -> String {
    let margin = 30u32;
    let title_height = if config.title.is_some() { 30 } else { 0 };
    let label_height = 25u32;
    let bar_width = 25u32; // Narrower bars for 12 values
    let spacing = 5u32;

    let total_bars_width = 12 * bar_width + 11 * spacing;
    let width = margin * 2 + total_bars_width;
    let height = margin + title_height + config.bar_height + label_height;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 25.0, title, "title-text");
    }

    // Find max for scaling (exclude 0 which is always the set cardinality)
    let max_val = *vector[1..].iter().max().unwrap_or(&1).max(&1);
    let scale = config.bar_height as f64 / max_val as f64;

    // Draw bars (skip index 0 which is always cardinality)
    for (i, &count) in vector.iter().enumerate().skip(1) {
        let bar_index = i - 1;
        let x = margin + bar_index as u32 * (bar_width + spacing);
        let bar_height = (count as f64 * scale) as u32;
        let y = margin + title_height + (config.bar_height - bar_height);

        // Bar
        svg.rect(
            x as f64,
            y as f64,
            bar_width as f64,
            bar_height as f64,
            config.theme.highlight_color,
            config.theme.stroke_color,
            1.0,
        );

        // Value on top of bar
        if count > 0 {
            svg.text(
                (x + bar_width / 2) as f64,
                (y - 5) as f64,
                &count.to_string(),
                "fret-text",
            );
        }

        // Semitone label below
        svg.text(
            (x + bar_width / 2) as f64,
            (margin + title_height + config.bar_height + 15) as f64,
            &i.to_string(),
            "fret-text",
        );
    }

    svg.build()
}

/// Build SVG for an interval matrix grid.
fn build_interval_matrix_svg(matrix: &IntervalMatrix, config: &IntervalConfig) -> String {
    let dim = matrix.dimension();
    if dim == 0 {
        return SvgBuilder::new(100, 100).build();
    }

    let margin = 50u32;
    let title_height = if config.title.is_some() { 30 } else { 0 };
    let label_offset = 30u32;

    let grid_size = dim as u32 * config.cell_size;
    let width = margin * 2 + label_offset + grid_size;
    let height = margin + title_height + label_offset + grid_size;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 25.0, title, "title-text");
    }

    let grid_start_x = margin + label_offset;
    let grid_start_y = margin + title_height + label_offset;

    // Draw grid cells and values
    let pcs = matrix.pcs();
    for row in 0..dim {
        // Row label
        let pc_val: u8 = pcs.iter().nth(row).copied().map(|pc| pc.into()).unwrap_or(0);
        svg.text(
            (margin + label_offset / 2) as f64,
            (grid_start_y + row as u32 * config.cell_size + config.cell_size / 2) as f64,
            &pc_val.to_string(),
            "string-text",
        );

        for col in 0..dim {
            let x = grid_start_x + col as u32 * config.cell_size;
            let y = grid_start_y + row as u32 * config.cell_size;

            // Determine cell color based on interval
            let interval = matrix.get(row, col);
            let fill = if row == col {
                config.theme.inactive_color // Diagonal (unison)
            } else {
                config.theme.background_color
            };

            // Cell rectangle
            svg.rect(
                x as f64,
                y as f64,
                config.cell_size as f64,
                config.cell_size as f64,
                fill,
                config.theme.stroke_color,
                1.0,
            );

            // Interval value
            if let Some(ic) = interval {
                let ic_val: u8 = ic.into();
                svg.text(
                    (x + config.cell_size / 2) as f64,
                    (y + config.cell_size / 2 + 4) as f64,
                    &ic_val.to_string(),
                    "note-text",
                );
            }
        }
    }

    // Column labels
    for col in 0..dim {
        let pc_val: u8 = pcs.iter().nth(col).copied().map(|pc| pc.into()).unwrap_or(0);
        svg.text(
            (grid_start_x + col as u32 * config.cell_size + config.cell_size / 2) as f64,
            (margin + title_height + label_offset / 2) as f64,
            &pc_val.to_string(),
            "string-text",
        );
    }

    svg.build()
}

/// Build SVG for a linear interval diagram.
fn build_linear_interval_svg(pitches: &[Pc], config: &IntervalConfig) -> String {
    if pitches.is_empty() {
        return SvgBuilder::new(100, 100).build();
    }

    let margin = 40u32;
    let title_height = if config.title.is_some() { 35 } else { 0 };
    let arc_height = 60u32;
    let note_spacing = 50u32;
    let note_radius = 15u32;

    // Sort pitches for linear display
    let mut sorted_pitches: Vec<u8> = pitches.iter().map(|pc| (*pc).into()).collect();
    sorted_pitches.sort();

    let width = margin * 2 + (sorted_pitches.len() as u32 - 1).max(1) * note_spacing + note_radius * 2;
    let height = margin * 2 + title_height + arc_height + note_radius * 2;

    let mut svg = SvgBuilder::new(width, height);

    // Title
    if let Some(ref title) = config.title {
        svg.text((width / 2) as f64, 25.0, title, "title-text");
    }

    let baseline_y = (margin + title_height + arc_height + note_radius) as f64;

    // Draw arcs between adjacent notes
    for i in 0..sorted_pitches.len().saturating_sub(1) {
        let x1 = (margin + note_radius + i as u32 * note_spacing) as f64;
        let x2 = (margin + note_radius + (i + 1) as u32 * note_spacing) as f64;
        let interval = (sorted_pitches[i + 1] as i32 - sorted_pitches[i] as i32).rem_euclid(12) as u8;

        // Arc center and height proportional to interval
        let cx = (x1 + x2) / 2.0;
        let arc_h = (interval as f64 / 12.0 * arc_height as f64).max(15.0);

        // Draw arc as quadratic bezier path
        let path = format!(
            "M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}",
            x1,
            baseline_y - note_radius as f64,
            cx,
            baseline_y - note_radius as f64 - arc_h,
            x2,
            baseline_y - note_radius as f64
        );
        svg.path(&path, "none", config.theme.highlight_color, 2.0);

        // Interval label at arc apex
        svg.text(
            cx,
            baseline_y - note_radius as f64 - arc_h - 5.0,
            &interval.to_string(),
            "fret-text",
        );
    }

    // Draw note circles
    for (i, &pc) in sorted_pitches.iter().enumerate() {
        let x = (margin + note_radius + i as u32 * note_spacing) as f64;
        svg.circle(
            x,
            baseline_y,
            note_radius,
            config.theme.highlight_color,
            config.theme.stroke_color,
            1.0,
        );

        // Note label
        svg.text_colored(x, baseline_y + 4.0, &pc.to_string(), "note-text", "#ffffff");
    }

    svg.build()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_builder_basic() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .build_vector();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_interval_vector_diagram() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .title("Major Triad")
            .build_vector();

        assert!(svg.contains("Major Triad"));
        // Major triad has interval vector <001110>
        assert!(svg.contains("<rect")); // Should have bars
    }

    #[test]
    fn test_full_interval_vector() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .build_full_vector();

        assert!(svg.starts_with("<svg"));
        // Should show semitone counts 1-11
    }

    #[test]
    fn test_interval_matrix() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .title("Interval Matrix")
            .build_matrix();

        assert!(svg.contains("Interval Matrix"));
        assert!(svg.contains("<rect")); // Grid cells
    }

    #[test]
    fn test_linear_interval_diagram() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .title("Linear Intervals")
            .build_linear();

        assert!(svg.contains("Linear Intervals"));
        assert!(svg.contains("<path")); // Arcs
        assert!(svg.contains("<circle")); // Notes
    }

    #[test]
    fn test_empty_pitch_set() {
        let svg = IntervalBuilder::new().pitches([]).build_matrix();

        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn test_single_pitch() {
        let svg = IntervalBuilder::new().pitches([Pc::Pc0]).build_linear();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<circle")); // Single note
    }

    #[test]
    fn test_with_theme() {
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
            .theme(SvgTheme::dark())
            .build_vector();

        assert!(svg.contains(SvgTheme::dark().highlight_color));
    }

    #[test]
    fn test_from_pc_set() {
        let pc_set = PcSet::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
        let svg = IntervalBuilder::new()
            .from_pc_set(&pc_set)
            .build_vector();

        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn test_diminished_seventh_interval_vector() {
        // Dim7 chord has special symmetry: all intervals appear twice
        let svg = IntervalBuilder::new()
            .pitches([Pc::Pc0, Pc::Pc3, Pc::Pc6, Pc::Pc9])
            .title("Dim7 Intervals")
            .build_vector();

        assert!(svg.contains("Dim7 Intervals"));
    }

    #[test]
    fn test_chromatic_scale_intervals() {
        // Chromatic scale has all intervals
        let all_pcs: Vec<Pc> = (0u8..12).map(|i| Pc::from(i)).collect();
        let svg = IntervalBuilder::new()
            .pitches(all_pcs)
            .build_full_vector();

        assert!(svg.starts_with("<svg"));
    }
}
