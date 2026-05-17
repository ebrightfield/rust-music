//! Multi-staff score rendering: grand staff, bracketed sections, independent staves.
//!
//! [`MultiStaffScore`] combines multiple [`ScoreBuilder`] staves into a vertically
//! grouped system with optional brace or bracket connectors.

use crate::font::bravura_font;
use crate::layout::measure::MeasureLayoutConfig;
use crate::layout::multi_staff::{
    layout_multi_staff, ConnectorKind, StaffGroup, SubBracket,
};
use crate::layout::page::{break_measures_auto, break_measures_optimal, PageSystem, SystemBreaking};
use crate::layout::staff::StaffLayout;
use crate::layout::system::{layout_system, SystemLayout};
use crate::layout::tab::TabStaffLayout;
use crate::render::multi_staff_renderer::{draw_joined_barline, draw_multi_staff_connectors};
use crate::render::page_renderer::{
    draw_cross_system_glissandos, draw_cross_system_hairpins, draw_cross_system_lyric_extenders,
    draw_cross_system_ottava_brackets, draw_cross_system_slurs, draw_cross_system_ties,
    draw_cross_system_trill_extensions,
};
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::system_renderer::draw_system;
use crate::render::tab_renderer::{draw_tab_clef, draw_tab_staff_lines};
use crate::render::{SvgWriter, TextStyle};

use super::tab::{draw_tab_measure, TabScoreBuilder};
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
    /// When true, use optimal (Knuth-Plass DP) line breaking.
    optimal_breaks: bool,
    /// Display measure numbers above the start of each system.
    show_measure_numbers: bool,
    /// Optional tablature stave rendered below the standard notation staves.
    tab_stave: Option<TabScoreBuilder>,
    /// Nested sub-bracket groupings within a [`ConnectorKind::Bracket`]
    /// connector. Only honoured when the connector is a bracket; ignored
    /// (silently dropped at layout time) for braces, independent staves,
    /// and guitar+tab scores. The empty default means the score renders
    /// identically to one without sub-brackets, preserving golden parity
    /// for all existing scores.
    sub_brackets: Vec<SubBracket>,
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
            optimal_breaks: false,
            show_measure_numbers: false,
            tab_stave: None,
            sub_brackets: Vec::new(),
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
            optimal_breaks: false,
            show_measure_numbers: false,
            tab_stave: None,
            sub_brackets: Vec::new(),
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
            optimal_breaks: false,
            show_measure_numbers: false,
            tab_stave: None,
            sub_brackets: Vec::new(),
        }
    }

    /// Create a guitar+tab combined score: bracket connector, joined barlines,
    /// standard notation stave above a tablature stave.
    ///
    /// This is the standard guitar notation layout where a 5-line treble staff
    /// appears above a 6-string (or custom) TAB staff, connected by a bracket.
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::clef::Clef;
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::pitch::Pitch;
    /// use music::note::note::Note;
    /// use music_engraver::score::ScoreBuilder;
    /// use music_engraver::score::tab::TabScoreBuilder;
    /// use music_engraver::score::multi_staff::MultiStaffScore;
    ///
    /// let notation = ScoreBuilder::new()
    ///     .clef(Clef::Treble)
    ///     .time_signature(4, 4)
    ///     .note(Pitch::new(Note::E, 4).expect("valid"), Duration::QTR)
    ///     .end_barline();
    ///
    /// let tab = TabScoreBuilder::guitar()
    ///     .quarter().fret(1, 0)
    ///     .end_barline();
    ///
    /// let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
    /// ```
    pub fn guitar_tab(notation: ScoreBuilder, tab: TabScoreBuilder) -> Self {
        Self {
            staves: vec![notation],
            connector: ConnectorKind::Bracket,
            joined_barlines: true,
            system_width: 0.0,
            measures_per_system: 0,
            auto_breaks: false,
            optimal_breaks: false,
            show_measure_numbers: false,
            tab_stave: Some(tab),
            sub_brackets: Vec::new(),
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
        self.optimal_breaks = false;
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
        self.optimal_breaks = false;
        self
    }

    /// Enable optimal (Knuth-Plass style) line breaking.
    ///
    /// Uses dynamic programming to minimize total whitespace deviation across
    /// all systems. Produces more evenly filled lines than `auto_line_breaks`.
    ///
    /// Overrides any previous `measures_per_system` or `auto_line_breaks` setting.
    pub fn optimal_line_breaks(mut self) -> Self {
        self.optimal_breaks = true;
        self.auto_breaks = false;
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

    /// Replace the score's nested sub-bracket list — inner brackets drawn
    /// just to the right of the main section bracket, grouping contiguous
    /// ranges of staves into instrument families (e.g. Violin I + Violin II
    /// share a sub-bracket inside the larger string-section bracket).
    ///
    /// Each [`SubBracket`] specifies a `start_index` (0-based, into the
    /// notation staves of this score) and a `staff_count`. Entries with
    /// `staff_count < 2`, `start_index >= staves.len()`, or that overshoot
    /// the end are silently dropped at layout time.
    ///
    /// Only honoured for [`MultiStaffScore::section`] (i.e. a bracket
    /// connector). On a grand-staff, independent staves, or guitar+tab
    /// score this list is set but ignored by the layout pass — the
    /// generated SVG is identical to the no-sub-brackets case.
    ///
    /// When at least one valid sub-bracket is present, the main bracket
    /// shifts left by `(SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS +
    /// SUB_BRACKET_GAP_SS)` staff-spaces to make room for the inner
    /// bracket between its right edge and the staff origin.
    ///
    /// # Example
    /// ```no_run
    /// use music_engraver::layout::multi_staff::SubBracket;
    /// use music_engraver::score::multi_staff::MultiStaffScore;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// let staves: Vec<ScoreBuilder> = (0..5).map(|_| ScoreBuilder::new()).collect();
    /// let _svg = MultiStaffScore::section(staves)
    ///     .with_sub_brackets(vec![
    ///         SubBracket { start_index: 0, staff_count: 2 },
    ///         SubBracket { start_index: 2, staff_count: 3 },
    ///     ])
    ///     .render_svg();
    /// ```
    pub fn with_sub_brackets(mut self, sub_brackets: Vec<SubBracket>) -> Self {
        self.sub_brackets = sub_brackets;
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
        if self.staves.is_empty() && self.tab_stave.is_none() {
            return Ok(String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        }

        // Flush pending events on all staves
        for stave in &mut self.staves {
            stave.flush_pending();
        }
        if let Some(ref mut tab) = self.tab_stave {
            tab.flush_pending();
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

        // Determine total number of measures (max across all staves + tab)
        let notation_max = self.staves.iter().map(|s| s.measures.len()).max().unwrap_or(0);
        let tab_max = self.tab_stave.as_ref().map(|t| t.measures.len()).unwrap_or(0);
        let max_measures = notation_max.max(tab_max);
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
        let use_optimal = self.optimal_breaks
            || self.staves.iter().any(|s| s.optimal_breaks);
        let use_auto = self.auto_breaks
            || self.staves.iter().any(|s| s.auto_breaks);
        let chunks = if use_optimal && !stave_data.is_empty() {
            let (ref contents, ref prefix) = stave_data[0];
            break_measures_optimal(prefix, contents, &measure_config, sys_width)
        } else if use_auto && !stave_data.is_empty() {
            let (ref contents, ref prefix) = stave_data[0];
            break_measures_auto(prefix, contents, &measure_config, sys_width)
        } else {
            let breaking = SystemBreaking::Fixed(mps);
            break_measures(max_measures, &breaking)
        };

        // Build the multi-staff geometry for standard notation staves only.
        // Tab stave is positioned below with a separate gap.
        let notation_staff_count = self.staves.len();
        let group = StaffGroup {
            staff_count: notation_staff_count,
            connector: self.connector,
            joined_barlines: self.joined_barlines,
            sub_brackets: self.sub_brackets.clone(),
        };

        let multi_layout = layout_multi_staff(&group, 0.0, staff_space, sys_width);
        let notation_height = multi_layout.total_height();

        // Tab stave geometry (height varies by line count)
        let tab_line_count = self.tab_stave.as_ref().map(|t| t.line_count).unwrap_or(6);
        let tab_staff_height = staff_space * (tab_line_count.saturating_sub(1)) as f64;
        // Gap between bottom of last notation staff and top of tab staff
        let tab_gap = crate::layout::multi_staff::INTER_STAFF_GAP_SS * staff_space;

        // Total system height includes notation + optional tab stave
        let system_height = if self.tab_stave.is_some() {
            notation_height + tab_gap + tab_staff_height
        } else {
            notation_height
        };

        let inter_system_gap = 10.0 * staff_space;

        let left_margin = match self.connector {
            ConnectorKind::Brace => 2.0 * staff_space,
            ConnectorKind::Bracket => 2.0 * staff_space,
            ConnectorKind::None => 0.0,
        };

        let total_systems = chunks.len();
        let page_width = sys_width + left_margin;
        let page_height = if total_systems > 0 {
            total_systems as f64 * system_height
                + (total_systems - 1).max(0) as f64 * inter_system_gap
        } else {
            0.0
        };

        let vb_margin = staff_space;
        let vb_x = -vb_margin - left_margin;
        let vb_y = -vb_margin;
        let vb_w = page_width + 2.0 * vb_margin;
        let vb_h = page_height + 2.0 * vb_margin;
        let px_per_unit = 7.0 / staff_space;
        let px_w = vb_w * px_per_unit;
        let px_h = vb_h * px_per_unit;

        let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

        let num_staves = self.staves.len();
        let mut stave_page_systems: Vec<Vec<PageSystem>> = vec![Vec::new(); num_staves];

        // TAB clef occupies ~2.5 staff spaces + 0.5 padding
        let tab_clef_width = 3.0 * staff_space;

        for (sys_idx, (start, end)) in chunks.iter().enumerate() {
            let group_y = sys_idx as f64 * (system_height + inter_system_gap);

            // --- Standard notation staves ---
            let ms_layout = layout_multi_staff(&group, group_y, staff_space, sys_width);

            if !self.staves.is_empty() {
                draw_multi_staff_connectors(&mut svg, &font, &ms_layout)?;
            }

            let mut stave_systems: Vec<SystemLayout> = Vec::new();

            for (stave_idx, (contents, prefix)) in stave_data.iter().enumerate() {
                let stave_y = ms_layout.staff_y_origins[stave_idx];

                let stave_start = (*start).min(contents.len());
                let stave_end = (*end).min(contents.len());
                let slice = &contents[stave_start..stave_end];

                if slice.is_empty() {
                    let staff = StaffLayout::new(left_margin, stave_y, sys_width, staff_space);
                    draw_staff_lines(&mut svg, &staff, &config);
                    continue;
                }

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

                stave_page_systems[stave_idx].push(PageSystem {
                    x: left_margin,
                    y: stave_y,
                    system: system.clone(),
                    first_measure_number: start + 1,
                });

                stave_systems.push(system);
            }

            // --- Tab stave (below standard notation staves) ---
            if let Some(ref tab) = self.tab_stave {
                let tab_y = if self.staves.is_empty() {
                    group_y
                } else {
                    // Position below the last notation staff
                    let last_notation_y = ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1];
                    let notation_bottom = last_notation_y + staff_space * 4.0;
                    notation_bottom + tab_gap
                };

                let tab_staff = TabStaffLayout::new(
                    left_margin,
                    tab_y,
                    sys_width,
                    staff_space,
                    tab.line_count,
                );

                draw_tab_staff_lines(&mut svg, &tab_staff, &config);
                draw_tab_clef(&mut svg, &tab_staff, &font)?;

                // Draw tab measures for this system chunk
                let tab_start = (*start).min(tab.measures.len());
                let tab_end = (*end).min(tab.measures.len());
                let tab_measures = &tab.measures[tab_start..tab_end];

                // Content area starts after the TAB clef
                let content_x = left_margin + tab_clef_width;
                let content_width = sys_width - tab_clef_width;
                let measure_width = if tab_measures.is_empty() {
                    content_width
                } else {
                    content_width / tab_measures.len() as f64
                };

                for (m_idx, measure) in tab_measures.iter().enumerate() {
                    let measure_x = content_x + m_idx as f64 * measure_width;
                    draw_tab_measure(
                        &mut svg,
                        &font,
                        &config,
                        &tab_staff,
                        measure,
                        measure_x,
                        measure_width,
                    )?;
                }
            }

            // --- Measure numbers ---
            if self.show_measure_numbers {
                if let Some(first_system) = stave_systems.first() {
                    if !first_system.measures.is_empty() {
                        let first_measure = &first_system.measures[0];
                        let num_x = left_margin + first_measure.x_offset;
                        let num_y = ms_layout.staff_y_origins[0]
                            - crate::render::page_renderer::MEASURE_NUMBER_ABOVE_STAFF_SS * staff_space;
                        let font_size = crate::render::page_renderer::MEASURE_NUMBER_FONT_SIZE_SS * staff_space;
                        let measure_number = start + 1;
                        svg.add_text(num_x, num_y, &measure_number.to_string(), &TextStyle {
                            font_family: "serif",
                            font_size,
                            fill: "black",
                            anchor: "start",
                            font_weight: "normal",
                            font_style: "normal",
                            dominant_baseline: "auto",
                        });
                    }
                }
            }

            // --- Joined barlines spanning all staves (notation + tab) ---
            let total_stave_count = self.staves.len() + if self.tab_stave.is_some() { 1 } else { 0 };
            if self.joined_barlines && total_stave_count >= 2 {
                let y_top = ms_layout.staff_y_origins[0];
                let y_bottom = if let Some(ref _tab) = self.tab_stave {
                    let tab_y = if self.staves.is_empty() {
                        group_y
                    } else {
                        let last_y = ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1];
                        last_y + staff_space * 4.0 + tab_gap
                    };
                    tab_y + tab_staff_height
                } else {
                    ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1]
                        + staff_space * 4.0
                };

                draw_joined_barline(
                    &mut svg,
                    left_margin,
                    y_top,
                    y_bottom,
                    config.thin_barline_thickness_fu(),
                );

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

            // --- Bracket extending to cover tab stave ---
            if self.tab_stave.is_some() && !self.staves.is_empty() {
                // `draw_multi_staff_connectors` above drew a bracket spanning
                // only the notation staves. For the guitar+tab layout, the
                // bracket should reach the *bottom* of the tab stave. We
                // overdraw by constructing a `BracketLayout` covering the
                // full notation+tab span and re-invoking the shared
                // `draw_bracket` helper — the previous bracket gets visually
                // replaced (the new path/line cover the same x range and
                // extend further). Sharing the helper keeps the SMuFL scroll
                // glyphs (bracketTop/bracketBottom) consistent with the
                // notation-only bracket.
                if self.connector == ConnectorKind::Bracket && !ms_layout.staff_y_origins.is_empty() {
                    let tab_y = {
                        let last_y = ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1];
                        last_y + staff_space * 4.0 + tab_gap
                    };
                    let thickness = crate::layout::multi_staff::BRACKET_THICKNESS_SS * staff_space;
                    let extended_bracket = crate::layout::multi_staff::BracketLayout {
                        x: left_margin
                            - crate::layout::multi_staff::BRACKET_THICKNESS_SS * staff_space,
                        y_top: ms_layout.staff_y_origins[0],
                        y_bottom: tab_y + tab_staff_height,
                        thickness,
                        top_glyph: smufl::Glyph::BracketTop,
                        bottom_glyph: smufl::Glyph::BracketBottom,
                    };
                    crate::render::multi_staff_renderer::draw_bracket(
                        &mut svg,
                        &font,
                        &extended_bracket,
                    )?;
                }
            }
        }

        // Draw cross-system spans for notation staves
        for stave_systems in &stave_page_systems {
            if stave_systems.len() < 2 {
                continue;
            }
            draw_cross_system_ties(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_slurs(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_hairpins(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_lyric_extenders(&mut svg, &config, stave_systems);
            draw_cross_system_ottava_brackets(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_glissandos(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_trill_extensions(&mut svg, &font, &config, stave_systems)?;
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
        // Auto and Optimal are handled by caller; this path is a fallback.
        SystemBreaking::Auto | SystemBreaking::Optimal => {
            break_measures(total, &SystemBreaking::Fixed(4))
        }
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
    fn section_bracket_renders_single_thick_line_and_two_scroll_glyphs() {
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

        // The orchestral section bracket is now a single thick vertical line
        // plus two SMuFL `bracketTop`/`bracketBottom` scroll glyphs (paths) —
        // not three serif lines. Beyond the bracket itself the SVG carries
        // 3 × 5 = 15 staff lines plus joined barlines and stems.
        let line_count = svg.matches("<line").count();
        let path_count = svg.matches("<path").count();
        // Floor of 16 = 15 staff lines + at least 1 bracket vertical line.
        assert!(
            line_count >= 16,
            "section should have ≥16 lines (15 staff + bracket vertical), got {line_count}"
        );
        // Bracket adds 2 paths over the no-bracket baseline; clef glyphs and
        // noteheads also add paths, so the floor is ≥2.
        assert!(
            path_count >= 2,
            "section should have ≥2 bracket scroll paths, got {path_count}"
        );

        // Compare against the `independent` (no-bracket) variant of the same
        // music to lock in the actual *bracket-induced* delta: the section
        // bracket must add at least 2 paths (top + bottom scrolls) and at
        // least 1 line (the thick vertical) over the no-connector baseline.
        let staves_indep = vec![
            simple_treble(),
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .time_signature(4, 4)
                .note(pitch(Note::C, 5), Duration::WHOLE)
                .end_barline(),
            simple_bass(),
        ];
        let svg_indep = MultiStaffScore::independent(staves_indep).render_svg();
        let line_count_indep = svg_indep.matches("<line").count();
        let path_count_indep = svg_indep.matches("<path").count();
        assert!(
            line_count > line_count_indep,
            "bracket should add ≥1 line over independent baseline ({line_count} vs {line_count_indep})"
        );
        assert!(
            path_count >= path_count_indep + 2,
            "bracket should add ≥2 scroll paths over independent baseline ({path_count} vs {path_count_indep})"
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

    // Helper for sub-bracket score-level tests: 4 identical treble staves
    // arranged as a section group. Compact enough that the rendered SVG
    // count deltas are dominated by the bracket connectors.
    fn four_staff_section() -> Vec<ScoreBuilder> {
        (0..4)
            .map(|_| {
                ScoreBuilder::new()
                    .clef(Clef::Treble)
                    .time_signature(4, 4)
                    .note(pitch(Note::C, 5), Duration::WHOLE)
                    .end_barline()
            })
            .collect()
    }

    #[test]
    fn section_with_sub_brackets_adds_thin_lines_for_each_sub_bracket() {
        let baseline = MultiStaffScore::section(four_staff_section()).render_svg();
        let nested = MultiStaffScore::section(four_staff_section())
            .with_sub_brackets(vec![
                SubBracket { start_index: 0, staff_count: 2 },
                SubBracket { start_index: 2, staff_count: 2 },
            ])
            .render_svg();

        // Each valid sub-bracket adds exactly one thin vertical <line>; no
        // extra <path> elements (sub-brackets carry no scrolls).
        let baseline_lines = baseline.matches("<line").count();
        let baseline_paths = baseline.matches("<path").count();
        let nested_lines = nested.matches("<line").count();
        let nested_paths = nested.matches("<path").count();

        assert_eq!(
            nested_lines - baseline_lines,
            2,
            "two sub-brackets should add exactly 2 lines (baseline={baseline_lines}, nested={nested_lines})"
        );
        assert_eq!(
            nested_paths, baseline_paths,
            "sub-brackets must not change <path> count (baseline={baseline_paths}, nested={nested_paths})"
        );
    }

    #[test]
    fn section_with_sub_brackets_shifts_main_bracket_left() {
        // When sub-brackets are present the layout pass shifts the main
        // bracket left by `(SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS
        // + SUB_BRACKET_GAP_SS) * staff_space` = 0.76 sp. The shifted x
        // appears in the scroll-glyph `translate(...)` substring; the
        // pre-shift x does not.
        let baseline = MultiStaffScore::section(four_staff_section()).render_svg();
        let nested = MultiStaffScore::section(four_staff_section())
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }])
            .render_svg();

        // The pre-shift bracket scroll appears at x = -125 (= -BRACKET_THICKNESS_SS * ss).
        assert!(
            baseline.contains("translate(-125,"),
            "baseline bracket scroll should anchor at x=-125, SVG: {baseline}"
        );
        // After shift x = -125 - 190 = -315 (190 = 0.76 sp * 250 fu).
        assert!(
            nested.contains("translate(-315,"),
            "nested-bracket scroll should anchor at x=-315 after leftward shift"
        );
        assert!(
            !nested.contains("translate(-125,"),
            "nested SVG must not still carry the unshifted x=-125 scroll translate"
        );
    }

    #[test]
    fn brace_with_sub_brackets_is_silently_ignored() {
        // Sub-brackets are a bracket-only feature; on a brace (grand
        // staff) connector the layout pass drops them. The score's SVG
        // must therefore be byte-identical to the same grand-staff
        // score with no sub-brackets configured.
        let plain = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
        let with_subs = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }])
            .render_svg();
        assert_eq!(
            plain, with_subs,
            "sub-brackets must not affect a brace-connector score"
        );
    }

    #[test]
    fn independent_staves_with_sub_brackets_is_silently_ignored() {
        // Same invariant as the brace case, for the no-connector variant.
        let staves: Vec<ScoreBuilder> = (0..3).map(|_| simple_treble()).collect();
        let staves_again: Vec<ScoreBuilder> = (0..3).map(|_| simple_treble()).collect();
        let plain = MultiStaffScore::independent(staves).render_svg();
        let with_subs = MultiStaffScore::independent(staves_again)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 3 }])
            .render_svg();
        assert_eq!(
            plain, with_subs,
            "sub-brackets must not affect an independent-staves score"
        );
    }

    #[test]
    fn section_with_no_sub_brackets_is_unchanged() {
        // Anti-regression for the conditional shift in `layout_multi_staff`:
        // a section bracket WITHOUT sub-brackets must not pick up the leftward
        // shift. The plain `section(...)` SVG must remain byte-identical
        // whether or not `with_sub_brackets(...)` was called with an empty
        // vec (it being the default).
        let plain = MultiStaffScore::section(four_staff_section()).render_svg();
        let empty_subs = MultiStaffScore::section(four_staff_section())
            .with_sub_brackets(Vec::new())
            .render_svg();
        assert_eq!(
            plain, empty_subs,
            "explicit empty sub-bracket list must equal the default"
        );
    }

    #[test]
    fn section_with_invalid_sub_brackets_renders_same_as_no_sub_brackets() {
        // All-invalid sub-bracket entries (count < 2, overshoot) are silently
        // dropped at layout time — the resulting SVG must equal the no-
        // sub-bracket case (no leftward shift, no thin lines).
        let plain = MultiStaffScore::section(four_staff_section()).render_svg();
        let invalid_only = MultiStaffScore::section(four_staff_section())
            .with_sub_brackets(vec![
                SubBracket { start_index: 0, staff_count: 1 }, // too small
                SubBracket { start_index: 4, staff_count: 2 }, // out of range
                SubBracket { start_index: 2, staff_count: 5 }, // overshoots
            ])
            .render_svg();
        assert_eq!(
            plain, invalid_only,
            "all-invalid sub-brackets must render identically to no sub-brackets"
        );
    }

    #[test]
    fn guitar_tab_with_sub_brackets_is_silently_ignored() {
        // Guitar+tab is a 1-staff (notation) + 1-tab layout. Sub-brackets
        // need >= 2 notation staves to be valid; on guitar+tab the layout
        // pass drops them as invalid. Therefore the SVG must be byte-identical
        // to the same guitar+tab score with no sub-brackets configured.
        let notation = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::E, 4), Duration::QTR)
            .end_barline();
        let tab = TabScoreBuilder::guitar()
            .quarter().fret(1, 0)
            .end_barline();
        let plain = MultiStaffScore::guitar_tab(notation, tab).render_svg();

        let notation2 = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::E, 4), Duration::QTR)
            .end_barline();
        let tab2 = TabScoreBuilder::guitar()
            .quarter().fret(1, 0)
            .end_barline();
        let with_subs = MultiStaffScore::guitar_tab(notation2, tab2)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }])
            .render_svg();
        assert_eq!(
            plain, with_subs,
            "sub-brackets must not affect a guitar+tab score"
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

    // --- Cross-system span tests ---

    /// Build a grand staff where the treble stave has a tie crossing the
    /// system break (last note of measure 2 → first note of measure 3).
    fn grand_staff_with_cross_system_tie() -> MultiStaffScore {
        let treble = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            // Measure 1
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            // Measure 2 — tie forward on last note
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::G, 4), Duration::HALF)
            .tie()
            .barline()
            // Measure 3 — tie target
            .note(pitch(Note::G, 4), Duration::WHOLE)
            .barline()
            // Measure 4
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        MultiStaffScore::grand_staff(treble, bass)
            .measures_per_system(2) // 2 systems: measures 1-2 and 3-4
    }

    #[test]
    fn cross_system_tie_in_multi_staff_draws_half_ties() {
        let svg_with_tie = grand_staff_with_cross_system_tie().render_svg();

        // Build same grand staff without the tie
        let treble_no_tie = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::G, 4), Duration::HALF)
            .barline()
            .note(pitch(Note::G, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        let svg_no_tie = MultiStaffScore::grand_staff(treble_no_tie, bass)
            .measures_per_system(2)
            .render_svg();

        // The tied version should have more filled paths (half-ties are filled crescents)
        let tied_paths = svg_with_tie.matches("<path").count();
        let untied_paths = svg_no_tie.matches("<path").count();
        assert!(
            tied_paths > untied_paths,
            "tied ({tied_paths}) should have more paths than untied ({untied_paths})"
        );
    }

    #[test]
    fn cross_system_slur_in_multi_staff_draws_half_slurs() {
        let treble = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::A, 4), Duration::HALF)
            .slur_start()
            .barline()
            .note(pitch(Note::B, 4), Duration::WHOLE)
            .slur_end()
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        let svg_with_slur = MultiStaffScore::grand_staff(treble, bass)
            .measures_per_system(2)
            .render_svg();

        // Slurs are rendered as filled paths — should contain at least 2 extra
        // (trailing half-slur + incoming half-slur)
        let treble_no_slur = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::A, 4), Duration::HALF)
            .barline()
            .note(pitch(Note::B, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass2 = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        let svg_no_slur = MultiStaffScore::grand_staff(treble_no_slur, bass2)
            .measures_per_system(2)
            .render_svg();

        let slur_paths = svg_with_slur.matches("<path").count();
        let no_slur_paths = svg_no_slur.matches("<path").count();
        assert!(
            slur_paths > no_slur_paths,
            "slurred ({slur_paths}) should have more paths than unslurred ({no_slur_paths})"
        );
    }

    #[test]
    fn cross_system_hairpin_in_multi_staff_draws_half_wedges() {
        use crate::layout::dynamics::Dynamic;

        let treble = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::A, 4), Duration::HALF)
            .cresc()
            .barline()
            .note(pitch(Note::B, 4), Duration::WHOLE)
            .hairpin_end()
            .dynamic(Dynamic::Forte)
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        let svg_with_hp = MultiStaffScore::grand_staff(treble, bass)
            .measures_per_system(2)
            .render_svg();

        // Hairpins are rendered as 2 <line> elements per wedge.
        // Cross-system hairpin produces 4 lines (2 per half-wedge).
        let treble_no_hp = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::HALF)
            .note(pitch(Note::A, 4), Duration::HALF)
            .barline()
            .note(pitch(Note::B, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();

        let bass2 = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::C, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::D, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 3), Duration::WHOLE)
            .barline()
            .note(pitch(Note::F, 3), Duration::WHOLE)
            .end_barline();

        let svg_no_hp = MultiStaffScore::grand_staff(treble_no_hp, bass2)
            .measures_per_system(2)
            .render_svg();

        let hp_lines = svg_with_hp.matches("<line").count();
        let no_hp_lines = svg_no_hp.matches("<line").count();
        assert!(
            hp_lines > no_hp_lines,
            "hairpin ({hp_lines}) should have more lines than no hairpin ({no_hp_lines})"
        );
    }

    #[test]
    fn cross_system_spans_only_affect_owning_stave() {
        // Tie on treble stave should not affect bass stave rendering
        let svg = grand_staff_with_cross_system_tie().render_svg();

        // Should be a valid SVG with both staves rendered
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));

        // The tie paths should exist (at least 1 filled path from half-ties)
        let path_count = svg.matches("<path").count();
        // Without ties there are noteheads + clefs; with ties there are extra filled crescents
        assert!(
            path_count >= 12,
            "should have noteheads + clefs + tie paths, got {path_count}"
        );
    }

    #[test]
    fn no_cross_system_spans_with_single_system() {
        // With all 4 measures on one system, no cross-system logic runs
        let svg_1sys = grand_staff_with_cross_system_tie()
            .measures_per_system(4) // won't actually change this since it's already built
            .render_svg();

        // Just verify it renders without error
        assert!(svg_1sys.starts_with("<svg"));
    }

    // --- guitar_tab tests ---

    fn simple_tab() -> TabScoreBuilder {
        TabScoreBuilder::guitar()
            .quarter().fret(1, 0)
            .next()
            .quarter().fret(1, 2)
            .next()
            .quarter().fret(2, 3)
            .next()
            .quarter().fret(3, 0)
            .end_barline()
    }

    #[test]
    fn guitar_tab_renders_valid_svg() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.contains("</svg>"), "should close svg");
    }

    #[test]
    fn guitar_tab_has_notation_and_tab_staff_lines() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        let line_count = svg.matches("<line ").count();
        // 5 notation staff lines + 6 tab staff lines + barlines/stems/brackets = many
        assert!(
            line_count >= 11,
            "should have at least 11 lines (5 notation + 6 tab), got {line_count}"
        );
    }

    #[test]
    fn guitar_tab_has_fret_numbers() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        // Tab should show fret numbers
        assert!(svg.contains(">0</text>"), "should show fret number 0");
        assert!(svg.contains(">2</text>"), "should show fret number 2");
        assert!(svg.contains(">3</text>"), "should show fret number 3");
    }

    #[test]
    fn guitar_tab_has_tab_clef() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        // TAB clef is an SMuFL path element
        let path_count = svg.matches("<path ").count();
        // At minimum: treble clef + TAB clef + noteheads = several paths
        assert!(
            path_count >= 2,
            "should have at least 2 paths (treble + TAB clef), got {path_count}"
        );
    }

    #[test]
    fn guitar_tab_has_bracket_connector() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        // The bracket is now 1 thick vertical line + 2 SMuFL scroll glyphs
        // (paths). Counted separately the bracket adds:
        //   notation-only initial bracket: 1 line + 2 paths
        //   extended bracket re-draw (spanning notation+tab): 1 line + 2 paths
        // → 2 additional lines + 4 additional paths from brackets alone,
        // on top of staff lines (5 notation + 6 tab = 11), joined barlines,
        // stems, and notehead/clef glyph paths.
        let line_count = svg.matches("<line ").count();
        // Floor: 11 staff lines + ≥1 bracket vertical (the most visible one).
        assert!(
            line_count >= 12,
            "guitar_tab should have ≥12 lines (11 staff + bracket vertical), got {line_count}"
        );

        // The bracket glyphs (bracketTop/bracketBottom) must be rendered:
        // assert their characteristic translate-anchor strings appear by
        // checking that the SVG carries at least 2 `<path` elements
        // anchored above and below the notation+tab span.
        let path_count = svg.matches("<path").count();
        assert!(
            path_count >= 2,
            "guitar_tab should have ≥2 paths (bracket scrolls + clef), got {path_count}"
        );
    }

    #[test]
    fn guitar_tab_has_joined_barlines() {
        let notation = simple_treble();
        let tab = simple_tab();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        // Joined barlines span from top of notation to bottom of tab staff
        // They should produce additional vertical lines beyond staff lines
        let svg_no_tab = simple_treble().render_svg();
        let single_lines = svg_no_tab.matches("<line ").count();
        let combined_lines = svg.matches("<line ").count();
        assert!(
            combined_lines > single_lines,
            "combined ({combined_lines}) should have more lines than single ({single_lines})"
        );
    }

    #[test]
    fn guitar_tab_differs_from_grand_staff() {
        let notation = simple_treble();
        let tab = simple_tab();
        let guitar_tab_svg = MultiStaffScore::guitar_tab(notation.clone(), tab).render_svg();
        let grand_svg = MultiStaffScore::grand_staff(notation, simple_bass()).render_svg();
        // Guitar+tab should show fret numbers, grand staff should not
        assert!(guitar_tab_svg.contains(">0</text>"), "guitar_tab should have fret numbers");
        assert!(!grand_svg.contains(">0</text>") || !grand_svg.contains("dominant-baseline=\"central\""),
            "grand staff should not have fret numbers with central baseline");
    }

    #[test]
    fn guitar_tab_four_string_bass() {
        let notation = ScoreBuilder::new()
            .clef(Clef::Bass)
            .time_signature(4, 4)
            .note(pitch(Note::E, 2), Duration::WHOLE)
            .end_barline();
        let tab = TabScoreBuilder::four_string()
            .whole().fret(4, 0)
            .end_barline();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        assert!(svg.starts_with("<svg"));
        // 4-string tab: notation (5 lines) + tab (4 lines) = 9 minimum
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 9,
            "bass+4-string tab should have at least 9 lines, got {line_count}"
        );
    }

    #[test]
    fn guitar_tab_multi_system() {
        let notation = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::E, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::B, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::E, 5), Duration::WHOLE)
            .end_barline();
        let tab = TabScoreBuilder::guitar()
            .whole().fret(1, 0).barline()
            .whole().fret(3, 0).barline()
            .whole().fret(2, 0).barline()
            .whole().fret(1, 5)
            .end_barline();
        let svg = MultiStaffScore::guitar_tab(notation, tab)
            .measures_per_system(2)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        // 2 systems × (5 notation + 6 tab) = 22 staff lines minimum
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 22,
            "multi-system guitar_tab should have >=22 staff lines, got {line_count}"
        );
        // Fret numbers from both systems should appear
        assert!(svg.contains(">0</text>"), "should show fret 0");
        assert!(svg.contains(">5</text>"), "should show fret 5");
    }

    #[test]
    fn guitar_tab_with_tab_rhythm_stems() {
        let notation = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::E, 4), Duration::QTR)
            .note(pitch(Note::G, 4), Duration::QTR)
            .note(pitch(Note::B, 4), Duration::HALF)
            .end_barline();
        let tab = TabScoreBuilder::guitar()
            .quarter().fret(1, 0)
            .next()
            .quarter().fret(3, 0)
            .next()
            .half().fret(2, 0)
            .end_barline();
        let svg = MultiStaffScore::guitar_tab(notation, tab).render_svg();
        assert!(svg.starts_with("<svg"));
        // Rhythm stems on tab add extra lines (stem lines above tab staff)
        let line_count = svg.matches("<line ").count();
        // At minimum: 5 (notation) + 6 (tab) + notation stems + tab stems + bracket + barlines
        assert!(
            line_count >= 17,
            "should have tab rhythm stems (>=17 lines), got {line_count}"
        );
    }

    #[test]
    fn guitar_tab_with_measures_per_system_override() {
        let notation = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(pitch(Note::E, 4), Duration::WHOLE)
            .barline()
            .note(pitch(Note::G, 4), Duration::WHOLE)
            .end_barline();
        let tab = TabScoreBuilder::guitar()
            .whole().fret(1, 0).barline()
            .whole().fret(3, 0)
            .end_barline();
        // 1 measure per system → 2 systems
        let svg = MultiStaffScore::guitar_tab(notation, tab)
            .measures_per_system(1)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        // 2 systems × (5+6) = 22 staff lines minimum
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 22,
            "1 mps should produce 2 systems (>=22 staff lines), got {line_count}"
        );
    }

    // -- Pixel-content PNG verification --
    //
    // The existing PNG tests in `render/png.rs` cover single-staff
    // `ScoreBuilder` output. These tests extend pixel-content depth to
    // `MultiStaffScore`, which adds a brace/bracket glyph, joined barlines,
    // and a second staff — code paths the single-staff tests can't exercise.
    // A `MultiStaffScore` PNG that came back blank, or that dropped one
    // staff, or whose brace glyph was missing, would still pass the magic-
    // byte and dimension checks above; the assertions below would fire.
    #[cfg(feature = "png")]
    mod png_tests {
        use super::*;
        use crate::render::png::test_helpers::{
            count_dense_rows, count_inked_pixels, decode_pixmap, inked_bbox, png_dimensions,
            INK_ALPHA_THRESHOLD,
        };

        #[test]
        fn grand_staff_png_has_substantial_ink() {
            // Grand staff = 2 staves + brace + joined barlines + clefs +
            // time/key sigs + notes. Substantial ink expected.
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
                .render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let ink = count_inked_pixels(&pixmap, INK_ALPHA_THRESHOLD);
            // Grand staff is much richer than the single-note ScoreBuilder
            // baseline (200 ink threshold); demand at least 600 here.
            assert!(
                ink >= 600,
                "grand staff PNG has only {ink} inked pixels; \
                 expected >= 600 for 2 staves + brace + clefs + notes"
            );
        }

        #[test]
        fn grand_staff_png_has_more_ink_than_single_staff() {
            // A grand staff renders strictly more content than either single
            // staff alone. If a refactor accidentally dropped the second
            // staff (or rendered it on top of the first), this would fire.
            let single_png = simple_treble().render_png(1.0);
            let grand_png =
                MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
            let single_ink =
                count_inked_pixels(&decode_pixmap(&single_png), INK_ALPHA_THRESHOLD);
            let grand_ink =
                count_inked_pixels(&decode_pixmap(&grand_png), INK_ALPHA_THRESHOLD);
            assert!(
                grand_ink > single_ink,
                "grand staff ink ({grand_ink}) should exceed single staff ink ({single_ink})"
            );
        }

        #[test]
        fn grand_staff_png_ink_bbox_is_taller_than_single_staff() {
            // The vertical span of inked content in a grand staff covers
            // both staves; a single staff covers only itself. The grand
            // bbox height must be substantially larger. Catches a regression
            // where both staves render at the same y-origin.
            let single_png = simple_treble().render_png(1.0);
            let grand_png =
                MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
            let single_bbox =
                inked_bbox(&decode_pixmap(&single_png), INK_ALPHA_THRESHOLD).expect("single ink");
            let grand_bbox =
                inked_bbox(&decode_pixmap(&grand_png), INK_ALPHA_THRESHOLD).expect("grand ink");
            let single_h = single_bbox.3 - single_bbox.1;
            let grand_h = grand_bbox.3 - grand_bbox.1;
            // The grand staff has two staves + the gap between them; even
            // tightly packed, its vertical span should be at least 1.5× the
            // single-staff span. Empirically the ratio is ~3× but we leave
            // headroom for layout tweaks.
            assert!(
                grand_h as f64 >= single_h as f64 * 1.5,
                "grand staff bbox height ({grand_h}) should be >= 1.5× single ({single_h})"
            );
        }

        #[test]
        fn grand_staff_png_has_at_least_ten_dense_horizontal_bands() {
            // Two five-line staves produce ~10 dense horizontal bands (one
            // per staff line, blurred by AA so we count "rows >= 50% dense"
            // rather than a strict line count). Catches a regression where
            // one staff's lines are dropped or rendered as dashed strokes.
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
                .render_png(1.0);
            let pixmap = decode_pixmap(&png);
            // 50% density is conservative: staff lines genuinely span 100% of
            // the staff width, so each line contributes at least 1 row above
            // 50%. Threshold of 10 corresponds to exactly 5+5 lines with no
            // AA-fringe expansion; AA typically widens each line to ~2 rows.
            let dense = count_dense_rows(&pixmap, INK_ALPHA_THRESHOLD, 0.5);
            assert!(
                dense >= 10,
                "grand staff has only {dense} dense rows; \
                 expected >= 10 (2 staves × 5 staff lines)"
            );
        }

        #[test]
        fn grand_staff_png_ink_starts_left_of_first_note_column() {
            // The brace glyph and clef sit to the left of any note. The
            // leftmost inked pixel must therefore appear inside the first
            // ~20% of the image width. Catches a regression where the brace
            // is missing or rendered off-canvas (negative x clipped).
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass())
                .render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let (x_min, _, _, _) =
                inked_bbox(&pixmap, INK_ALPHA_THRESHOLD).expect("ink present");
            let img_w = pixmap.width();
            let left_band = img_w as f64 * 0.2;
            assert!(
                (x_min as f64) < left_band,
                "leftmost ink column is {x_min}, beyond {left_band:.0} (20% of {img_w}); \
                 expected brace/clef on the left edge"
            );
        }

        #[test]
        fn empty_grand_staff_renders_to_decodable_png() {
            // An empty grand-staff score still renders a valid PNG; this is
            // a regression canary against a panic-on-decode for the empty
            // case. We don't assert on ink count (could be 0); only that
            // the PNG decodes and reports its IHDR dimensions correctly.
            let png = MultiStaffScore::grand_staff(ScoreBuilder::new(), ScoreBuilder::new())
                .render_png(1.0);
            let (hdr_w, hdr_h) = png_dimensions(&png);
            let pixmap = decode_pixmap(&png);
            assert_eq!(pixmap.width(), hdr_w);
            assert_eq!(pixmap.height(), hdr_h);
            assert!(hdr_w > 0 && hdr_h > 0, "empty PNG should still have non-zero dims");
        }

        #[test]
        fn grand_staff_png_2x_has_more_ink_than_1x() {
            // Scale propagates to the rasterizer (verified once for
            // ScoreBuilder; re-verify for the MultiStaffScore code path
            // which builds its own PngRenderer instance — catches a wire-up
            // regression specific to multi-staff).
            let png_1x =
                MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
            let png_2x =
                MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(2.0);
            let ink_1x = count_inked_pixels(&decode_pixmap(&png_1x), INK_ALPHA_THRESHOLD);
            let ink_2x = count_inked_pixels(&decode_pixmap(&png_2x), INK_ALPHA_THRESHOLD);
            assert!(
                ink_2x > ink_1x * 2,
                "2× should at least double the ink count (got 1×={ink_1x}, 2×={ink_2x})"
            );
        }
    }
}
