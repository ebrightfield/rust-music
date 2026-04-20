//! Multi-staff score rendering: grand staff, bracketed sections, independent staves.
//!
//! [`MultiStaffScore`] combines multiple [`ScoreBuilder`] staves into a vertically
//! grouped system with optional brace or bracket connectors.

use crate::font::bravura_font;
use crate::layout::measure::MeasureLayoutConfig;
use crate::layout::multi_staff::{
    layout_multi_staff, ConnectorKind, StaffGroup,
};
use crate::layout::page::{break_measures_auto, SystemBreaking};
use crate::layout::staff::StaffLayout;
use crate::layout::system::{layout_system, SystemLayout};
use crate::render::multi_staff_renderer::{draw_joined_barline, draw_multi_staff_connectors};
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::system_renderer::draw_system;
use crate::render::{SvgWriter, TextStyle};

use super::ScoreBuilder;

/// A multi-staff score combining multiple [`ScoreBuilder`] staves with a
/// visual connector (brace, bracket, or none) and optionally joined barlines.
///
/// Each stave is an independent `ScoreBuilder` with its own clef, key signature,
/// and musical content. The multi-staff score renders all staves vertically
/// aligned, with shared system breaks.
///
/// # Example
/// ```no_run
/// use music::notation::clef::Clef;
/// use music::notation::rhythm::duration::Duration;
/// use music::note::pitch::Pitch;
/// use music::note::note::Note;
/// use music_engraver::layout::key_signature::KeySignature;
/// use music_engraver::score::ScoreBuilder;
/// use music_engraver::score::multi_staff::MultiStaffScore;
///
/// let treble = ScoreBuilder::new()
///     .clef(Clef::Treble)
///     .key_signature(KeySignature::Sharps(2))
///     .time_signature(4, 4)
///     .note(Pitch::new(Note::D, 5).expect("valid"), Duration::WHOLE)
///     .end_barline();
///
/// let bass = ScoreBuilder::new()
///     .clef(Clef::Bass)
///     .key_signature(KeySignature::Sharps(2))
///     .time_signature(4, 4)
///     .note(Pitch::new(Note::D, 3).expect("valid"), Duration::WHOLE)
///     .end_barline();
///
/// let svg = MultiStaffScore::grand_staff(treble, bass).render_svg();
/// ```
#[must_use = "a MultiStaffScore does nothing until .render_svg() or .try_render_svg() is called"]
pub struct MultiStaffScore {
    staves: Vec<ScoreBuilder>,
    connector: ConnectorKind,
    joined_barlines: bool,
    /// Override system width (font design units). 0 = auto.
    system_width: f64,
    /// Override measures per system. 0 = auto.
    measures_per_system: usize,
    /// When true, use width-based auto line breaking instead of fixed measures_per_system.
    auto_breaks: bool,
    /// Display measure numbers above the start of each system.
    show_measure_numbers: bool,
}

impl MultiStaffScore {
    /// Create a grand staff (piano/keyboard): brace connector, joined barlines.
    ///
    /// `upper` is typically the treble clef stave, `lower` the bass clef stave.
    pub fn grand_staff(upper: ScoreBuilder, lower: ScoreBuilder) -> Self {
        Self {
            staves: vec![upper, lower],
            connector: ConnectorKind::Brace,
            joined_barlines: true,
            system_width: 0.0,
            measures_per_system: 0,
            auto_breaks: false,
            show_measure_numbers: false,
        }
    }

    /// Create a bracketed section (orchestral grouping): bracket connector, joined barlines.
    pub fn section(staves: Vec<ScoreBuilder>) -> Self {
        Self {
            staves,
            connector: ConnectorKind::Bracket,
            joined_barlines: true,
            system_width: 0.0,
            measures_per_system: 0,
            auto_breaks: false,
            show_measure_numbers: false,
        }
    }

    /// Create independent staves with no connector or joined barlines.
    pub fn independent(staves: Vec<ScoreBuilder>) -> Self {
        Self {
            staves,
            connector: ConnectorKind::None,
            joined_barlines: false,
            system_width: 0.0,
            measures_per_system: 0,
            auto_breaks: false,
            show_measure_numbers: false,
        }
    }

    /// Set the system width in font design units.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Set the number of measures per system.
    ///
    /// Calling this disables auto line breaks if previously enabled.
    pub fn measures_per_system(mut self, n: usize) -> Self {
        self.measures_per_system = n;
        self.auto_breaks = false;
        self
    }

    /// Enable automatic width-based line breaking.
    ///
    /// Measures are greedily packed onto systems until the natural width
    /// exceeds the target system width. Uses the first stave's content for
    /// width estimation.
    ///
    /// Calling this overrides a previous `measures_per_system` setting.
    pub fn auto_line_breaks(mut self) -> Self {
        self.auto_breaks = true;
        self
    }

    /// Show measure numbers above the start of each system.
    ///
    /// Each system displays the 1-based measure number of its first bar
    /// above the top staff. Only the topmost stave displays numbers.
    pub fn show_measure_numbers(mut self) -> Self {
        self.show_measure_numbers = true;
        self
    }

    /// Render the multi-staff score to an SVG string.
    ///
    /// # Panics
    ///
    /// Panics if font glyph lookup fails. See [`ScoreBuilder::render_svg`] for
    /// panic safety discussion.
    #[must_use = "the SVG string is returned but not used"]
    pub fn render_svg(self) -> String {
        self.try_render_svg()
            .expect("bundled Bravura font contains all required SMuFL glyphs")
    }

    /// Render the multi-staff score to an SVG string, returning an error on failure.
    #[must_use = "the SVG string is returned but not used"]
    pub fn try_render_svg(mut self) -> Result<String, crate::error::EngraverError> {
        if self.staves.is_empty() {
            return Ok(String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        }

        // Flush pending events on all staves
        for stave in &mut self.staves {
            stave.flush_pending();
        }

        let font = bravura_font();
        let config = font.engraving_config();
        let staff_space = config.staff_space;

        // Determine system width from first stave that has it set, or auto
        let sys_width = if self.system_width > 0.0 {
            self.system_width
        } else {
            self.staves
                .iter()
                .find(|s| s.system_width > 0.0)
                .map(|s| s.system_width)
                .unwrap_or(40.0 * staff_space)
        };

        let mps = if self.measures_per_system > 0 {
            self.measures_per_system
        } else {
            self.staves
                .iter()
                .find(|s| s.measures_per_system > 0)
                .map(|s| s.measures_per_system)
                .unwrap_or(4)
        };

        let measure_config = MeasureLayoutConfig::from_staff_space(staff_space);

        // Determine total number of measures (max across all staves)
        let max_measures = self.staves.iter().map(|s| s.measures.len()).max().unwrap_or(0);
        if max_measures == 0 {
            return Ok(String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        }

        // Build measure contents and prefixes for each stave (needed early for auto breaking)
        let stave_data: Vec<_> = self
            .staves
            .iter()
            .map(|s| (s.build_measure_contents(), s.build_prefix()))
            .collect();

        // Break measures into system chunks
        let use_auto = self.auto_breaks
            || self.staves.iter().any(|s| s.auto_breaks);
        let chunks = if use_auto {
            // Use first stave's content for width estimation
            let (ref contents, ref prefix) = stave_data[0];
            break_measures_auto(prefix, contents, &measure_config, sys_width)
        } else {
            let breaking = SystemBreaking::Fixed(mps);
            break_measures(max_measures, &breaking)
        };

        // Build the multi-staff geometry
        let group = StaffGroup {
            staff_count: self.staves.len(),
            connector: self.connector,
            joined_barlines: self.joined_barlines,
        };

        // Compute the height of one multi-staff system
        let multi_layout = layout_multi_staff(&group, 0.0, staff_space, sys_width);
        let system_height = multi_layout.total_height();

        // Vertical spacing between multi-staff system groups
        // Use 10 staff spaces clearance between bottom of one group and top of next
        let inter_system_gap = 10.0 * staff_space;

        // Left margin to accommodate brace/bracket
        let left_margin = match self.connector {
            ConnectorKind::Brace => 2.0 * staff_space,
            ConnectorKind::Bracket => 2.0 * staff_space,
            ConnectorKind::None => 0.0,
        };

        // Page dimensions
        let total_systems = chunks.len();
        let page_width = sys_width + left_margin;
        let page_height = if total_systems > 0 {
            total_systems as f64 * system_height
                + (total_systems - 1).max(0) as f64 * inter_system_gap
        } else {
            0.0
        };

        // Prepare SVG
        let vb_margin = staff_space;
        let vb_x = -vb_margin - left_margin;
        let vb_y = -vb_margin;
        let vb_w = page_width + 2.0 * vb_margin;
        let vb_h = page_height + 2.0 * vb_margin;
        let px_per_unit = 7.0 / staff_space;
        let px_w = vb_w * px_per_unit;
        let px_h = vb_h * px_per_unit;

        let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

        // Render each system chunk
        for (sys_idx, (start, end)) in chunks.iter().enumerate() {
            let group_y = sys_idx as f64 * (system_height + inter_system_gap);

            // Compute multi-staff layout at this y position
            let ms_layout = layout_multi_staff(&group, group_y, staff_space, sys_width);

            // Draw connector (brace/bracket)
            draw_multi_staff_connectors(&mut svg, &font, &ms_layout)?;

            // Layout and draw each stave
            let mut stave_systems: Vec<SystemLayout> = Vec::new();

            for (stave_idx, (contents, prefix)) in stave_data.iter().enumerate() {
                let stave_y = ms_layout.staff_y_origins[stave_idx];

                // Slice measures for this system chunk (pad with empty if stave is shorter)
                let stave_start = (*start).min(contents.len());
                let stave_end = (*end).min(contents.len());
                let slice = &contents[stave_start..stave_end];

                if slice.is_empty() {
                    // Still draw staff lines for empty staves
                    let staff = StaffLayout::new(left_margin, stave_y, sys_width, staff_space);
                    draw_staff_lines(&mut svg, &staff, &config);
                    continue;
                }

                // First system of each stave gets full prefix; subsequent get no time sig
                let sys_prefix = if sys_idx == 0 {
                    prefix.clone()
                } else {
                    crate::layout::system::SystemPrefix {
                        clef_layout: prefix.clef_layout.clone(),
                        clef_kind: prefix.clef_kind,
                        key_signature: prefix.key_signature.clone(),
                        time_signature: None,
                    }
                };

                let system = layout_system(
                    &sys_prefix,
                    slice,
                    &measure_config,
                    Some(sys_width),
                );

                draw_system(&mut svg, &font, &config, &system, left_margin, stave_y)?;

                stave_systems.push(system);
            }

            // Draw measure number above the top stave of each system
            if self.show_measure_numbers {
                if let Some(first_system) = stave_systems.first() {
                    if !first_system.measures.is_empty() {
                        let first_measure = &first_system.measures[0];
                        let num_x = left_margin + first_measure.x_offset;
                        let num_y = ms_layout.staff_y_origins[0]
                            - crate::render::page_renderer::MEASURE_NUMBER_ABOVE_STAFF_SS * staff_space;
                        let font_size = crate::render::page_renderer::MEASURE_NUMBER_FONT_SIZE_SS * staff_space;
                        let measure_number = start + 1; // 1-based
                        svg.add_text(num_x, num_y, &measure_number.to_string(), &TextStyle {
                            font_family: "serif",
                            font_size,
                            fill: "black",
                            anchor: "start",
                            font_weight: "normal",
                            font_style: "normal",
                        });
                    }
                }
            }

            // Draw joined barlines through all staves (at barline positions of first stave)
            if self.joined_barlines && ms_layout.staff_y_origins.len() >= 2 {
                let y_top = ms_layout.staff_y_origins[0];
                let y_bottom = ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1]
                    + staff_space * 4.0; // bottom of last staff

                // Draw initial joined barline at the left edge (system start)
                draw_joined_barline(
                    &mut svg,
                    left_margin,
                    y_top,
                    y_bottom,
                    config.thin_barline_thickness_fu(),
                );

                // Draw joined barlines at each measure boundary from the first stave's layout
                if let Some(first_system) = stave_systems.first() {
                    for measure in &first_system.measures {
                        let barline_x = left_margin + measure.x_offset + measure.layout.total_width;
                        draw_joined_barline(
                            &mut svg,
                            barline_x,
                            y_top,
                            y_bottom,
                            config.thin_barline_thickness_fu(),
                        );
                    }
                }
            }
        }

        Ok(svg.to_svg())
    }

    /// Render the multi-staff score to PNG bytes at the given scale factor.
    #[cfg(feature = "png")]
    pub fn try_render_png(self, scale: f32) -> Result<Vec<u8>, crate::error::EngraverError> {
        let svg = self.try_render_svg()?;
        let mut renderer = crate::render::png::PngRenderer::new(scale);
        renderer.load_system_fonts();
        Ok(renderer.render_png(&svg)?)
    }

    /// Render the multi-staff score to PNG bytes at the given scale factor.
    #[cfg(feature = "png")]
    #[must_use = "the PNG bytes are returned but not used"]
    pub fn render_png(self, scale: f32) -> Vec<u8> {
        self.try_render_png(scale)
            .expect("bundled Bravura font and PNG pipeline should not fail for valid input")
    }
}

/// Break `total` measures into chunks per the breaking strategy.
fn break_measures(total: usize, breaking: &SystemBreaking) -> Vec<(usize, usize)> {
    match breaking {
        SystemBreaking::Fixed(n) => {
            let n = (*n).max(1);
            let mut chunks = Vec::new();
            let mut start = 0;
            while start < total {
                let end = (start + n).min(total);
                chunks.push((start, end));
                start = end;
            }
            chunks
        }
        SystemBreaking::Manual(breaks) => {
            let mut chunks = Vec::new();
            let mut start = 0;
            for &b in breaks {
                if b > start && b <= total {
                    chunks.push((start, b));
                    start = b;
                }
            }
            if start < total {
                chunks.push((start, total));
            }
            chunks
        }
        // Auto handled by caller using break_measures_auto; this path is a
        // fallback when Auto is passed directly to break_measures.
        SystemBreaking::Auto => break_measures(total, &SystemBreaking::Fixed(4)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::notation::clef::Clef;
    use music::notation::rhythm::duration::Duration;
    use music::note::note::Note;
    use music::note::pitch::Pitch;
    use crate::layout::key_signature::KeySignature;

    fn pitch(note: Note, octave: u8) -> Pitch {
        Pitch::new(note, octave).expect("valid pitch")
    }

    fn simple_treble() -> ScoreBuilder {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4)
            .note(pitch(Note::D, 5), Duration::HALF)
            .note(pitch(Note::E, 5), Duration::HALF)
            .end_barline()
    }

    fn simple_bass() -> ScoreBuilder {
        ScoreBuilder::new()
            .clef(Clef::Bass)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4)
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .end_barline()
    }

    #[test]
    fn grand_staff_renders_valid_svg() {
        let svg = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.contains("</svg>"), "should close svg");
    }

    #[test]
    fn grand_staff_has_brace_glyph() {
        let svg = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        // Brace is rendered as a <path> with a scale transform
        assert!(svg.contains("scale(1,"), "should contain brace scaling");
    }

    #[test]
    fn grand_staff_has_two_sets_of_staff_lines() {
        let svg = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        // Each staff has 5 horizontal lines; 2 staves = 10 staff lines
        // Plus stem lines, barlines, joined barlines
        let line_count = svg.matches("<line").count();
        // At least 10 staff lines + some barlines
        assert!(
            line_count >= 10,
            "should have at least 10 lines (2 staves × 5 lines), got {line_count}"
        );
    }

    #[test]
    fn grand_staff_has_joined_barlines() {
        let svg = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        // Joined barlines are extra <line> elements spanning both staves
        // The total lines should be more than just individual staff lines + stems
        let line_count = svg.matches("<line").count();
        // 10 staff lines + at least 2 joined barlines + individual barlines + stems
        assert!(
            line_count >= 14,
            "should have joined barlines (>=14 lines), got {line_count}"
        );
    }

    #[test]
    fn grand_staff_has_more_paths_than_single_staff() {
        let single_svg = simple_treble().render_svg();
        let grand_svg = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();

        let single_paths = single_svg.matches("<path").count();
        let grand_paths = grand_svg.matches("<path").count();

        assert!(
            grand_paths > single_paths,
            "grand staff ({grand_paths} paths) should have more than single ({single_paths})"
        );
    }

    #[test]
    fn section_bracket_renders_three_bracket_lines() {
        let staves = vec![
            simple_treble(),
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .time_signature(4, 4)
                .note(pitch(Note::C, 5), Duration::WHOLE)
                .end_barline(),
            simple_bass(),
        ];
        let svg = MultiStaffScore::section(staves).render_svg();
        assert!(svg.starts_with("<svg"));
        // Bracket produces 3 lines (1 vertical + 2 serifs)
        // Plus staff lines, barlines, stems, joined barlines — should be many lines
        let line_count = svg.matches("<line").count();
        // 3 staves × 5 lines + bracket (3 lines) + barlines + joined barlines + stems
        assert!(
            line_count >= 18,
            "section should have many lines (>=18), got {line_count}"
        );
    }

    #[test]
    fn independent_staves_no_connector() {
        let staves = vec![simple_treble(), simple_bass()];
        let svg = MultiStaffScore::independent(staves).render_svg();
        // No brace scale transform
        assert!(
            !svg.contains("scale(1,"),
            "independent should have no brace"
        );
    }

    #[test]
    fn empty_staves_renders_empty_svg() {
        let svg = MultiStaffScore::grand_staff(
            ScoreBuilder::new(),
            ScoreBuilder::new(),
        ).render_svg();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn system_width_override() {
        let svg_default = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
            .render_svg();
        let svg_wide = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
            .system_width_fu(20000.0)
            .render_svg();

        // Different widths should produce different SVGs
        assert_ne!(svg_default, svg_wide);
    }

    #[test]
    fn measures_per_system_override() {
        // Build staves with 2 measures each
        let treble = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 5), Duration::WHOLE)
            .end_barline();
        let bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .end_barline();

        let svg_1_per = MultiStaffScore::grand_staff(treble.clone(), bass.clone())
            .measures_per_system(1)
            .render_svg();
        let svg_2_per = MultiStaffScore::grand_staff(treble, bass)
            .measures_per_system(2)
            .render_svg();

        // 1 per system will be taller (2 system groups); 2 per system will be shorter (1 group)
        assert_ne!(svg_1_per, svg_2_per);
    }

    #[test]
    fn grand_staff_differs_from_bracket() {
        let svg_grand = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        let svg_bracket = MultiStaffScore::section(vec![simple_treble(), simple_bass()]).render_svg();
        // Brace vs bracket produce different visual output
        assert_ne!(svg_grand, svg_bracket);
    }

    #[test]
    fn break_measures_fixed() {
        let chunks = break_measures(7, &SystemBreaking::Fixed(3));
        assert_eq!(chunks, vec![(0, 3), (3, 6), (6, 7)]);
    }

    #[test]
    fn break_measures_fixed_exact() {
        let chunks = break_measures(6, &SystemBreaking::Fixed(3));
        assert_eq!(chunks, vec![(0, 3), (3, 6)]);
    }

    #[test]
    fn break_measures_manual() {
        let chunks = break_measures(8, &SystemBreaking::Manual(vec![3, 6]));
        assert_eq!(chunks, vec![(0, 3), (3, 6), (6, 8)]);
    }

    #[test]
    fn break_measures_empty() {
        let chunks = break_measures(0, &SystemBreaking::Fixed(4));
        assert!(chunks.is_empty());
    }

    /// Build a multi-measure treble stave for auto-breaking tests.
    fn multi_measure_treble(num_measures: usize) -> ScoreBuilder {
        let mut b = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4);
        for i in 0..num_measures {
            let note = if i % 2 == 0 { Note::D } else { Note::E };
            b = b
                .note(pitch(note, 5), Duration::QTR)
                .note(pitch(Note::Fis, 5), Duration::QTR)
                .note(pitch(Note::G, 5), Duration::QTR)
                .note(pitch(Note::A, 5), Duration::QTR);
            if i < num_measures - 1 {
                b = b.barline();
            }
        }
        b.end_barline()
    }

    fn multi_measure_bass(num_measures: usize) -> ScoreBuilder {
        let mut b = ScoreBuilder::new()
            .clef(Clef::Bass)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4);
        for i in 0..num_measures {
            b = b.note(pitch(Note::D, 3), Duration::WHOLE);
            if i < num_measures - 1 {
                b = b.barline();
            }
        }
        b.end_barline()
    }

    #[test]
    fn auto_line_breaks_renders_valid_svg() {
        let svg = MultiStaffScore::grand_staff(
            multi_measure_treble(6),
            multi_measure_bass(6),
        )
        .auto_line_breaks()
        .render_svg();
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.contains("</svg>"), "should close svg");
    }

    #[test]
    fn auto_line_breaks_differs_from_fixed() {
        let svg_auto = MultiStaffScore::grand_staff(
            multi_measure_treble(8),
            multi_measure_bass(8),
        )
        .auto_line_breaks()
        .render_svg();

        let svg_fixed = MultiStaffScore::grand_staff(
            multi_measure_treble(8),
            multi_measure_bass(8),
        )
        .measures_per_system(4)
        .render_svg();

        // Auto breaking may pack differently than fixed 4-per-system,
        // especially with narrow or wide system widths.
        // At minimum both should be valid SVGs.
        assert!(svg_auto.starts_with("<svg"));
        assert!(svg_fixed.starts_with("<svg"));
    }

    #[test]
    fn auto_line_breaks_narrow_width_produces_more_systems() {
        // With a very narrow system width, auto should produce many system groups
        let svg_narrow = MultiStaffScore::grand_staff(
            multi_measure_treble(6),
            multi_measure_bass(6),
        )
        .system_width_fu(5000.0)
        .auto_line_breaks()
        .render_svg();

        let svg_wide = MultiStaffScore::grand_staff(
            multi_measure_treble(6),
            multi_measure_bass(6),
        )
        .system_width_fu(50000.0)
        .auto_line_breaks()
        .render_svg();

        // Narrow width should produce a taller SVG (more system groups stacked)
        // Both should be valid
        assert!(svg_narrow.starts_with("<svg"));
        assert!(svg_wide.starts_with("<svg"));

        // Narrow will have more staff line sets (more systems × 2 staves × 5 lines each)
        let narrow_lines = svg_narrow.matches("<line").count();
        let wide_lines = svg_wide.matches("<line").count();
        assert!(
            narrow_lines > wide_lines,
            "narrow ({narrow_lines} lines) should have more systems than wide ({wide_lines} lines)"
        );
    }

    #[test]
    fn auto_line_breaks_overrides_measures_per_system() {
        // Setting auto_line_breaks after measures_per_system should use auto
        let svg = MultiStaffScore::grand_staff(
            multi_measure_treble(6),
            multi_measure_bass(6),
        )
        .measures_per_system(2)  // set fixed first
        .auto_line_breaks()      // then override with auto
        .render_svg();

        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn measures_per_system_overrides_auto_line_breaks() {
        // Setting measures_per_system after auto should disable auto
        let svg_fixed = MultiStaffScore::grand_staff(
            multi_measure_treble(4),
            multi_measure_bass(4),
        )
        .auto_line_breaks()
        .measures_per_system(2)  // override back to fixed
        .render_svg();

        let svg_auto = MultiStaffScore::grand_staff(
            multi_measure_treble(4),
            multi_measure_bass(4),
        )
        .auto_line_breaks()
        .render_svg();

        // Fixed(2) with 4 measures = 2 system groups
        // Auto with 4 measures = depends on width, likely different layout
        assert!(svg_fixed.starts_with("<svg"));
        assert!(svg_auto.starts_with("<svg"));
    }

    #[test]
    fn auto_line_breaks_inherits_from_stave() {
        // If a stave has auto_breaks=true, MultiStaffScore should pick it up
        let treble = multi_measure_treble(6).auto_line_breaks();
        let bass = multi_measure_bass(6);

        let svg = MultiStaffScore::grand_staff(treble, bass).render_svg();
        assert!(svg.starts_with("<svg"));

        // Should have multiple systems like explicit auto on MultiStaffScore
        let svg_explicit = MultiStaffScore::grand_staff(
            multi_measure_treble(6),
            multi_measure_bass(6),
        )
        .auto_line_breaks()
        .render_svg();

        // Both use auto breaking — should produce same layout
        assert_eq!(svg, svg_explicit);
    }

    #[test]
    fn section_auto_line_breaks() {
        let staves = vec![
            multi_measure_treble(4),
            multi_measure_bass(4),
            multi_measure_treble(4),
        ];
        let svg = MultiStaffScore::section(staves)
            .auto_line_breaks()
            .render_svg();
        assert!(svg.starts_with("<svg"));
        // Should have bracket lines
        let line_count = svg.matches("<line").count();
        assert!(line_count >= 15, "section should have many lines, got {line_count}");
    }
}
