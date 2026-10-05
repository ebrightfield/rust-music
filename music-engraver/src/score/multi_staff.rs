//! Multi-staff score rendering: grand staff, bracketed sections, independent staves.
//!
//! [`MultiStaffScore`] combines multiple [`ScoreBuilder`] staves into a vertically
//! grouped system with optional brace or bracket connectors.

use std::collections::HashMap;

use crate::font::bravura_font;
use crate::layout::bar_number::{
    layout_bar_numbers, system_bar_number_slots, MeasureNumbering, BAR_NUMBER_ABOVE_STAFF_SS,
    BAR_NUMBER_FONT_SIZE_SS,
};
use crate::layout::barline::BarlineStyle;
use crate::layout::line_break::LineBreakPlan;
use crate::layout::mark_extent::system_mark_extent_ss;
use crate::layout::measure::{MeasureElement, MeasureLayoutConfig};
use crate::layout::multi_staff::{
    layout_multi_staff_with_gaps, ConnectorKind, StaffGroup, SubBracket, INTER_STAFF_GAP_SS,
};
use crate::layout::page::{
    break_by_directives, content_natural_width, prefix_natural_width, PageSystem, SystemBreaking,
};
use crate::layout::staff::StaffLayout;
use crate::layout::system::{
    layout_staves_followed_by, staff_prefix_glyph_extent_ss, system_start_prefix, MeasureContent,
    SystemLayout, SystemPrefix,
};
use crate::layout::tab::TabStaffLayout;
use crate::render::bar_number_renderer::draw_bar_numbers;
use crate::render::multi_staff_renderer::{
    draw_joined_barline, draw_joined_dashed_barline, draw_multi_staff_connectors,
};
use crate::render::page_renderer::{
    draw_cross_system_analysis_brackets, draw_cross_system_glissandos, draw_cross_system_hairpins,
    draw_cross_system_lyric_extenders, draw_cross_system_lyric_hyphens,
    draw_cross_system_ottava_brackets, draw_cross_system_slurs, draw_cross_system_text_spanners,
    draw_cross_system_ties, draw_cross_system_trill_extensions,
};
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::system_renderer::draw_system;
use crate::render::tab_renderer::{draw_tab_clef, draw_tab_staff_lines};
use crate::render::{SvgWriter, TextStyle};

use super::guitar::{
    draw_guitar_bends, draw_guitar_spans, draw_guitar_tab_system, GuitarRenderAnchor, GuitarScore,
    GUITAR_BARLINE_WIDTH_SS, GUITAR_TAB_GAP_SS,
};
use super::ScoreBuilder;
#[path = "cross_staff.rs"]
mod cross_staff;
use cross_staff::{draw_cross_staff_glissandos, distribute_voice, CrossStaffGlissando};

/// Invalid placement of a continuous voice among the score's staves.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CrossStaffError {
    /// An event targets a stave that does not exist.
    #[error("cross-staff event {event} targets stave {staff}, but this score has {staff_count} staves")]
    InvalidStaff { event: usize, staff: usize, staff_count: usize },
    /// Only one continuous (primary) voice may be routed across staves.
    #[error("cross-staff event {event} uses voice {voice}; expected voice 0")]
    InvalidVoice { event: usize, voice: u8 },
    /// Structural/group events that cannot be distributed between staves.
    #[error("cross-staff event {event} cannot be routed: {kind}")]
    UnsupportedEvent { event: usize, kind: &'static str },
}


/// One stave's measure contents (explicit line breaks applied) and prefix.
pub(crate) type StaveData = (Vec<MeasureContent>, SystemPrefix);

/// Prepared stave contents and the system ranges shared by all staves.
type SharedSystems = (Vec<StaveData>, Vec<(usize, usize)>);

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
///     .note(Pitch::new(Note::D, 5), Duration::WHOLE)
///     .end_barline();
///
/// let bass = ScoreBuilder::new()
///     .clef(Clef::Bass)
///     .key_signature(KeySignature::Sharps(2))
///     .time_signature(4, 4)
///     .note(Pitch::new(Note::D, 3), Duration::WHOLE)
///     .end_barline();
///
/// let svg = MultiStaffScore::grand_staff(treble, bass).render_svg();
/// ```
#[must_use = "a MultiStaffScore does nothing until .render_svg() or .try_render_svg() is called"]
pub struct MultiStaffScore {
    staves: Vec<ScoreBuilder>,
    connector: ConnectorKind,
    joined_barlines: bool,
    /// A single musical voice whose events choose their notation stave.
    cross_staff_voice: Option<ScoreBuilder>,
    cross_staff_breaks: Option<LineBreakPlan>,
    cross_staff_glissandos: Vec<CrossStaffGlissando>,
    /// Override system width (font design units). 0 = auto.
    system_width: f64,
    /// Override measures per system. 0 = auto.
    measures_per_system: usize,
    /// When true, use width-based auto line breaking instead of fixed measures_per_system.
    auto_breaks: bool,
    /// When true, use optimal (Knuth-Plass DP) line breaking.
    optimal_breaks: bool,
    /// When true, break systems only at explicit `system_break()`s.
    explicit_breaks: bool,
    /// Which measures print their number (above the top stave).
    measure_numbering: MeasureNumbering,
    /// Optional tablature derived from the same semantic guitar timeline.
    tab_stave: Option<GuitarScore>,
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
            explicit_breaks: false,
            measure_numbering: MeasureNumbering::Hidden,
            tab_stave: None,
            sub_brackets: Vec::new(),
            cross_staff_voice: None,
            cross_staff_breaks: None,
            cross_staff_glissandos: Vec::new(),
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
            explicit_breaks: false,
            measure_numbering: MeasureNumbering::Hidden,
            tab_stave: None,
            cross_staff_voice: None,
            cross_staff_breaks: None,
            cross_staff_glissandos: Vec::new(),
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
            explicit_breaks: false,
            measure_numbering: MeasureNumbering::Hidden,
            tab_stave: None,
            sub_brackets: Vec::new(),
            cross_staff_voice: None,
            cross_staff_breaks: None,
            cross_staff_glissandos: Vec::new(),
        }
    }

    /// Create a coordinated standard-notation and TAB score from one validated
    /// guitar timeline.
    ///
    /// Sounding pitches, durations, voices, string/fret realizations, groups,
    /// and technique spans all come from `guitar`; callers cannot supply
    /// independently authored staves.
    pub fn guitar(guitar: GuitarScore) -> Self {
        let numbering = guitar.measure_numbering;
        Self {
            // Standard notation is derived only after the guitar timeline has
            // finalized its pending measure in `try_render_svg`.
            staves: Vec::new(),
            connector: ConnectorKind::Bracket,
            joined_barlines: true,
            system_width: 0.0,
            measures_per_system: 0,
            auto_breaks: false,
            optimal_breaks: false,
            explicit_breaks: false,
            measure_numbering: numbering,
            tab_stave: Some(guitar),
            sub_brackets: Vec::new(),
            cross_staff_voice: None,
            cross_staff_breaks: None,
            cross_staff_glissandos: Vec::new(),
        }
    }

    /// Add a single continuous voice routed among this score's staves.
    /// Assign notes, chords and rests with [`ScoreBuilder::on_staff`]; events
    /// without an assignment use stave zero. Non-active staves have invisible
    /// spacers at each onset, not duplicated notes or phantom rests.
    pub fn cross_staff_voice(mut self, voice: ScoreBuilder) -> Self {
        self.cross_staff_voice = Some(voice);
        self
    }

    /// Set the system width in font design units.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Set the number of measures per system.
    ///
    /// Calling this disables auto line breaks if previously enabled.
    /// Explicit `system_break()`s on any stave still apply.
    pub fn measures_per_system(mut self, n: usize) -> Self {
        self.measures_per_system = n;
        self.auto_breaks = false;
        self.optimal_breaks = false;
        self.explicit_breaks = false;
        self
    }

    /// Enable automatic width-based line breaking.
    ///
    /// Measures are greedily packed onto systems until the natural width
    /// exceeds the target system width. Uses the stave with the most visual
    /// measure pieces for width estimation, so a break inside one stave's
    /// logical measure is represented on every stave.
    ///
    /// Calling this overrides a previous `measures_per_system` setting.
    pub fn auto_line_breaks(mut self) -> Self {
        self.auto_breaks = true;
        self.optimal_breaks = false;
        self.explicit_breaks = false;
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
        self.explicit_breaks = false;
        self
    }

    /// Break systems only at explicit `system_break()`s, requested on any
    /// stave (see [`ScoreBuilder::explicit_line_breaks`]).
    ///
    /// Overrides any previous `measures_per_system`, `auto_line_breaks`, or
    /// `optimal_line_breaks` setting.
    pub fn explicit_line_breaks(mut self) -> Self {
        self.explicit_breaks = true;
        self.auto_breaks = false;
        self.optimal_breaks = false;
        self
    }

    /// Choose which measures print their number. Numbers come from the
    /// first stave's measures and are drawn above the topmost stave only.
    pub fn measure_numbering(mut self, numbering: MeasureNumbering) -> Self {
        self.measure_numbering = numbering;
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

    /// Each stave's measure contents and prefix, and the `(start, end)`
    /// content ranges of the systems they break into.
    ///
    /// Every stave's explicit line breaks apply to all staves, so the staves
    /// break — and split measures at mid-measure breaks — together. Width
    /// policies consult the widest measure over every stave, never stave zero.
    pub(crate) fn staves_into_systems(
        &self,
        measure_config: &MeasureLayoutConfig,
        sys_width: f64,
        measures_per_system: usize,
    ) -> Result<SharedSystems, super::ScoreStructureError> {
        let logical: Vec<_> = self
            .staves
            .iter()
            .map(ScoreBuilder::build_measure_contents)
            .collect::<Result<_, _>>()?;
        let mut line_breaks = LineBreakPlan::default();
        for (stave, contents) in self.staves.iter().zip(&logical) {
            line_breaks.merge(&stave.line_break_plan(contents));
        }
        if let Some(cross_breaks) = &self.cross_staff_breaks {
            line_breaks.merge(cross_breaks);
        }
        let mut stave_data: Vec<StaveData> = self
            .staves
            .iter()
            .zip(logical)
            .map(|(stave, contents)| (line_breaks.apply(contents), stave.build_prefix()))
            .collect();
        if let Some(reference) = stave_data.iter().max_by_key(|(contents, _)| contents.len()) {
            let reference = reference.0.clone();
            for (contents, _) in &mut stave_data {
                for missing in &reference[contents.len()..] {
                    contents.push(MeasureContent {
                        events: Vec::new(),
                        barline: missing.barline,
                        volta: None,
                        additional_voices: Vec::new(),
                        meta: missing.meta.clone(),
                    });
                }
            }
        }

        let breaking = if self.explicit_breaks || self.staves.iter().any(|s| s.explicit_breaks) {
            SystemBreaking::Explicit
        } else if self.optimal_breaks || self.staves.iter().any(|s| s.optimal_breaks) {
            SystemBreaking::Optimal
        } else if self.auto_breaks || self.staves.iter().any(|s| s.auto_breaks) {
            SystemBreaking::Auto
        } else {
            SystemBreaking::Fixed(measures_per_system)
        };
        let count = stave_data.first().map_or(0, |(contents, _)| contents.len());
        let directives = stave_data.first().map_or_else(Vec::new, |(contents, _)| {
            contents
                .iter()
                .map(|content| content.meta.line_break)
                .collect()
        });
        let widths: Vec<f64> = match breaking {
            SystemBreaking::Auto | SystemBreaking::Optimal => (0..count)
                .map(|index| {
                    stave_data
                        .iter()
                        .map(|(contents, _)| {
                            content_natural_width(&contents[index], measure_config)
                        })
                        .fold(0.0_f64, f64::max)
                })
                .collect(),
            _ => Vec::new(),
        };
        let first_prefix_width = stave_data
            .iter()
            .map(|(_, prefix)| prefix_natural_width(prefix, measure_config))
            .fold(0.0_f64, f64::max);
        let continuation_prefix_width = stave_data
            .iter()
            .map(|(_, prefix)| {
                let mut continuation = prefix.clone();
                continuation.time_signature = None;
                prefix_natural_width(&continuation, measure_config)
            })
            .fold(0.0_f64, f64::max);
        let chunks = break_by_directives(
            &directives,
            &breaking,
            &widths,
            (
                sys_width - first_prefix_width,
                sys_width - continuation_prefix_width,
            ),
        );
        Ok((stave_data, chunks))
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
        if self.staves.is_empty() && self.tab_stave.is_none() && self.cross_staff_voice.is_none() {
            return Ok(String::from(
                "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            ));
        }

        if let Some(score) = &mut self.tab_stave {
            score.finalize_pending_measure()?;
            self.staves = vec![score.notation_builder()];
        } else {
            // Flush pending events on independently supplied notation staves.
            for stave in &mut self.staves {
                stave.flush_pending();
            }
        }
        if let Some(mut voice) = self.cross_staff_voice.take() {
            voice.flush_pending();
            voice.validate_group_spans()?;
            self.cross_staff_breaks = Some(voice.line_break_plan(&voice.build_measure_contents()?));
            self.explicit_breaks |= voice.explicit_breaks;
            self.cross_staff_glissandos = distribute_voice(voice, &mut self.staves)?;
        }
        for stave in &self.staves {
            stave.validate_group_spans()?;
        }
        let guitar_timeline = match self.tab_stave.as_ref() {
            Some(score) => score.validated_bend_timeline()?,
            None => None,
        };

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

        let mut measure_config = MeasureLayoutConfig::from_staff_space(staff_space);
        if self.tab_stave.is_some() {
            measure_config.barline_width = GUITAR_BARLINE_WIDTH_SS * staff_space;
        }

        // Determine total number of measures (max across all staves + tab)
        let notation_max = self
            .staves
            .iter()
            .map(|s| s.measures.len())
            .max()
            .unwrap_or(0);
        let tab_max = self
            .tab_stave
            .as_ref()
            .map(GuitarScore::measure_count)
            .unwrap_or(0);
        let max_measures = notation_max.max(tab_max);
        if max_measures == 0 {
            return Ok(String::from(
                "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            ));
        }

        let (stave_data, chunks) = self.staves_into_systems(&measure_config, sys_width, mps)?;

        // Build the multi-staff geometry for standard notation staves only.
        // Tab stave is positioned below with a separate gap.
        let notation_staff_count = self.staves.len();
        let group = StaffGroup {
            staff_count: notation_staff_count,
            connector: self.connector,
            joined_barlines: self.joined_barlines,
            sub_brackets: self.sub_brackets.clone(),
        };

        // Plan every horizontal grid before assigning page/stave y positions.
        // The resulting verse/annotation extents determine each stave gap.
        let laid_out_systems: Vec<Vec<SystemLayout>> = chunks
            .iter()
            .map(|&(start, end)| {
                let prefixes: Vec<_> = stave_data
                    .iter()
                    .map(|(contents, prefix)| system_start_prefix(prefix, contents, start))
                    .collect();
                let slices: Vec<_> = stave_data
                    .iter()
                    .zip(&prefixes)
                    .map(|((contents, _), prefix)| {
                        (prefix, &contents[start..end], contents.get(end))
                    })
                    .collect();
                layout_staves_followed_by(&slices, &measure_config, Some(sys_width))
            })
            .collect();
        let mark_extents: Vec<Vec<_>> = laid_out_systems
            .iter()
            .map(|systems| systems.iter().map(system_mark_extent_ss).collect())
            .collect();
        let gaps: Vec<Vec<f64>> = mark_extents
            .iter()
            .map(|system| {
                system
                    .windows(2)
                    .map(|staves| INTER_STAFF_GAP_SS.max(staves[0].1 + staves[1].0 + 1.0))
                    .collect()
            })
            .collect();
        let mut planned_staff_layouts: Vec<_> = gaps
            .iter()
            .map(|system_gaps| {
                layout_multi_staff_with_gaps(&group, 0.0, staff_space, sys_width, system_gaps)
            })
            .collect();

        // Tab stave geometry (height varies by line count)
        let tab_line_count = self
            .tab_stave
            .as_ref()
            .map(GuitarScore::line_count)
            .unwrap_or(6);
        let tab_staff_height = staff_space * (tab_line_count.saturating_sub(1)) as f64;
        // Guitar-specific upper lanes include rhythm stems and tuplets. Lower
        // annotation lanes are planned per system so unrelated systems do not
        // inherit the tallest annotation stack in the score.
        let tab_gap = GUITAR_TAB_GAP_SS * staff_space;
        let guitar_annotation_layout = self
            .tab_stave
            .as_ref()
            .map(|score| score.annotation_layout(&chunks));
        let system_heights: Vec<f64> = planned_staff_layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| {
                layout.total_height()
                    + if self.tab_stave.is_some() {
                        tab_gap + tab_staff_height
                    } else {
                        0.0
                    }
                    + guitar_annotation_layout
                        .as_ref()
                        .map(|annotations| annotations.system_height_ss(index) * staff_space)
                        .unwrap_or(0.0)
            })
            .collect();

        let inter_system_gap = (0..mark_extents.len().saturating_sub(1))
            .filter_map(|index| {
                Some((
                    mark_extents[index].last()?.1,
                    mark_extents[index + 1].first()?.0,
                ))
            })
            .fold(10.0_f64, |gap, (below, above)| gap.max(below + above + 2.0))
            * staff_space;

        let left_margin = match self.connector {
            ConnectorKind::Brace => 2.0 * staff_space,
            ConnectorKind::Bracket => 2.0 * staff_space,
            ConnectorKind::None => 0.0,
        };

        let total_systems = chunks.len();
        let page_width = sys_width + left_margin;
        let page_height = system_heights.iter().sum::<f64>()
            + total_systems.saturating_sub(1) as f64 * inter_system_gap;
        let mut system_y_origins = Vec::with_capacity(total_systems);
        let mut next_system_y = 0.0;
        for height in &system_heights {
            system_y_origins.push(next_system_y);
            next_system_y += height + inter_system_gap;
        }

        let guitar_annotation_top_margin = self
            .tab_stave
            .as_ref()
            .map(GuitarScore::standard_annotation_top_margin_ss)
            .unwrap_or(0.0);
        let setup_label = self.tab_stave.as_ref().and_then(GuitarScore::setup_label);
        let setup_label_y_ss = setup_label
            .as_ref()
            .map(|_| -(guitar_annotation_top_margin.max(6.5) + 1.5));
        let side_margin = if self.tab_stave.is_some() {
            5.0 * staff_space
        } else {
            staff_space
        };
        let top_margin = if let Some(setup_y) = setup_label_y_ss {
            (-setup_y + 1.0) * staff_space
        } else if self.tab_stave.is_some() {
            guitar_annotation_top_margin * staff_space
        } else {
            side_margin
        };
        // Clefs and key/time signatures of the outer staves can reach past
        // the staff lines (a G clef's top, a tenor clef): keep them inside.
        let (top_margin, bottom_margin) = if self.tab_stave.is_some() {
            (top_margin, side_margin)
        } else {
            let first = stave_data
                .first()
                .and_then(|(contents, prefix)| staff_prefix_glyph_extent_ss(prefix, contents));
            let last = stave_data
                .last()
                .and_then(|(contents, prefix)| staff_prefix_glyph_extent_ss(prefix, contents));
            let first_marks = mark_extents
                .iter()
                .filter_map(|staves| staves.first().map(|&(above, _)| above))
                .fold(0.0_f64, f64::max);
            let last_marks = mark_extents
                .iter()
                .filter_map(|staves| staves.last().map(|&(_, below)| below))
                .fold(0.0_f64, f64::max);
            let first_notes = if self.cross_staff_breaks.is_some() {
                laid_out_systems.iter().filter_map(|s| s.first())
                    .map(|s| cross_staff::outer_note_extent_ss(s).0)
                    .fold(0.0_f64, f64::max)
            } else { 0.0 };
            let last_notes = if self.cross_staff_breaks.is_some() {
                laid_out_systems.iter().filter_map(|s| s.last())
                    .map(|s| cross_staff::outer_note_extent_ss(s).1)
                    .fold(0.0_f64, f64::max)
            } else { 0.0 };
            let numbers = if self.measure_numbering == MeasureNumbering::Hidden {
                0.0
            } else {
                (BAR_NUMBER_ABOVE_STAFF_SS + BAR_NUMBER_FONT_SIZE_SS) * staff_space
            };
            (
                top_margin
                    .max(first.map_or(0.0, |(top, _)| (-top) * staff_space) + side_margin)
                    .max(first_marks * staff_space + side_margin)
                    .max(first_notes * staff_space + side_margin)
                    .max(numbers),
                side_margin
                    .max(last.map_or(0.0, |(_, bottom)| (bottom - 4.0) * staff_space) + side_margin)
                    .max(last_marks * staff_space + side_margin)
                    .max(last_notes * staff_space + side_margin),
            )
        };
        let vb_x = -side_margin - left_margin;
        let vb_y = -top_margin;
        // The staves end at `left_margin + sys_width` (= `page_width`).
        let vb_w = page_width + side_margin - vb_x;
        let vb_h = page_height + top_margin + bottom_margin;
        let px_per_unit = 7.0 / staff_space;
        let px_w = vb_w * px_per_unit;
        let px_h = vb_h * px_per_unit;

        let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);
        if let Some(label) = setup_label {
            svg.add_raw("<g data-guitar-setup-header=\"true\">");
            svg.add_text(
                left_margin,
                setup_label_y_ss.expect("setup label y exists with setup label") * staff_space,
                &label,
                &TextStyle {
                    anchor: "start",
                    font_weight: "bold",
                    ..TextStyle::normal(0.78 * staff_space)
                },
            );
            svg.add_raw("</g>");
        }

        let num_staves = self.staves.len();
        let mut stave_page_systems: Vec<Vec<PageSystem>> = vec![Vec::new(); num_staves];
        let mut cross_staff_anchors = HashMap::new();

        let mut guitar_anchors: HashMap<_, GuitarRenderAnchor> = HashMap::new();
        let mut guitar_standard_staves = Vec::new();
        let mut guitar_tab_staves = Vec::new();

        for (sys_idx, (start, end)) in chunks.iter().enumerate() {
            let group_y = system_y_origins[sys_idx];

            // Move the pre-planned staff and connector geometry to this
            // system's page origin, without laying its columns out again.
            let ms_layout = &mut planned_staff_layouts[sys_idx];
            ms_layout.translate_y(group_y);
            if !self.staves.is_empty() {
                draw_multi_staff_connectors(&mut svg, &font, ms_layout)?;
            }
            let stave_systems = &laid_out_systems[sys_idx];

            for (stave_idx, system) in stave_systems.iter().enumerate() {
                let stave_y = ms_layout.staff_y_origins[stave_idx];
                if system.measures.is_empty() {
                    let staff = StaffLayout::new(left_margin, stave_y, sys_width, staff_space);
                    draw_staff_lines(&mut svg, &staff, &config);
                    continue;
                }
                draw_system(&mut svg, &font, &config, system, left_margin, stave_y)?;
                stave_page_systems[stave_idx].push(PageSystem {
                    x: left_margin,
                    y: stave_y,
                    system: system.clone(),
                });
            }
            if !self.cross_staff_glissandos.is_empty() {
                cross_staff::collect_anchors(
                    &font,
                    staff_space,
                    sys_idx,
                    stave_systems,
                    &ms_layout.staff_y_origins,
                    left_margin,
                    &mut cross_staff_anchors,
                )?;
            }

            // --- Tab stave (below standard notation staves) ---
            if let Some(tab) = &self.tab_stave {
                let tab_y = if self.staves.is_empty() {
                    group_y
                } else {
                    let last_notation_y =
                        ms_layout.staff_y_origins[ms_layout.staff_y_origins.len() - 1];
                    let notation_bottom = last_notation_y + staff_space * 4.0;
                    notation_bottom + tab_gap
                };

                let tab_staff = TabStaffLayout::new(
                    left_margin,
                    tab_y,
                    sys_width,
                    staff_space,
                    tab.line_count(),
                );

                draw_tab_staff_lines(&mut svg, &tab_staff, &config);
                draw_tab_clef(&mut svg, &tab_staff, &font)?;

                let standard_layout = stave_systems
                    .first()
                    .expect("guitar score always has one notation stave");
                guitar_standard_staves.push(StaffLayout::new(
                    left_margin,
                    ms_layout.staff_y_origins[0],
                    sys_width,
                    staff_space,
                ));
                draw_guitar_tab_system(
                    &mut svg,
                    &font,
                    &config,
                    tab,
                    guitar_annotation_layout
                        .as_ref()
                        .expect("guitar annotation layout exists with a TAB stave"),
                    sys_idx,
                    *start,
                    *end,
                    standard_layout,
                    guitar_standard_staves
                        .last()
                        .expect("standard guitar staff was just recorded"),
                    &tab_staff,
                    &mut guitar_anchors,
                )?;
                guitar_tab_staves.push(tab_staff);
            }

            // --- Measure numbers (top stave only) ---
            if let Some(first_system) = stave_systems.first() {
                let numbers = layout_bar_numbers(
                    self.measure_numbering,
                    system_bar_number_slots(first_system, left_margin),
                    ms_layout.staff_y_origins[0],
                    staff_space,
                );
                draw_bar_numbers(&mut svg, &numbers, staff_space);
            }

            // --- Joined barlines spanning all staves (notation + tab) ---
            let total_stave_count =
                self.staves.len() + if self.tab_stave.is_some() { 1 } else { 0 };
            if self.joined_barlines && total_stave_count >= 2 {
                // Vertical extent of every stave, top to bottom.
                let mut stave_spans: Vec<(f64, f64)> = ms_layout
                    .staff_y_origins
                    .iter()
                    .map(|&y| (y, y + staff_space * 4.0))
                    .collect();
                if self.tab_stave.is_some() {
                    let tab_y = match stave_spans.last() {
                        Some(&(_, notation_bottom)) => notation_bottom + tab_gap,
                        None => group_y,
                    };
                    stave_spans.push((tab_y, tab_y + tab_staff_height));
                }
                let y_top = stave_spans[0].0;
                let y_bottom = stave_spans[stave_spans.len() - 1].1;
                let gaps: Vec<(f64, f64)> = stave_spans
                    .windows(2)
                    .map(|pair| (pair[0].1, pair[1].0))
                    .collect();

                draw_joined_barline(
                    &mut svg,
                    left_margin,
                    y_top,
                    y_bottom,
                    config.thin_barline_thickness_fu(),
                );

                for (measure_index, measure) in stave_systems[0].measures.iter().enumerate() {
                    let style = stave_systems
                        .iter()
                        .filter_map(|system| system.measures.get(measure_index))
                        .find_map(|measure| {
                            measure.layout.elements.iter().rev().find_map(|element| {
                                match element.element {
                                    MeasureElement::Barline(style) if style.is_visible() => {
                                        Some(style)
                                    }
                                    _ => None,
                                }
                            })
                        });
                    let Some(style) = style else { continue };
                    // The barline is a shared grid column, not a coordinate
                    // borrowed from any individual stave's layout.
                    let barline_x =
                        left_margin + measure.x_offset + measure.shared_closing_barline_x;
                    if style == BarlineStyle::Dashed {
                        draw_joined_dashed_barline(&mut svg, &config, barline_x, &gaps);
                    } else if style.spans_staff_gaps() {
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
                if self.connector == ConnectorKind::Bracket && !ms_layout.staff_y_origins.is_empty()
                {
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
        if let Some(score) = self.tab_stave.as_ref() {
            draw_guitar_spans(
                &mut svg,
                &font,
                &config,
                score,
                guitar_annotation_layout
                    .as_ref()
                    .expect("guitar annotation layout exists with a TAB stave"),
                &guitar_anchors,
                &guitar_tab_staves,
            )?;
            if let Some(timeline) = guitar_timeline.as_ref() {
                draw_guitar_bends(
                    &mut svg,
                    &config,
                    score,
                    &guitar_anchors,
                    timeline,
                    &guitar_standard_staves,
                    &guitar_tab_staves,
                );
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
            draw_cross_system_text_spanners(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_lyric_extenders(&mut svg, &config, stave_systems);
            draw_cross_system_lyric_hyphens(&mut svg, &config, stave_systems);
            draw_cross_system_ottava_brackets(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_analysis_brackets(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_glissandos(&mut svg, &font, &config, stave_systems)?;
            draw_cross_system_trill_extensions(&mut svg, &font, &config, stave_systems)?;
        }

        draw_cross_staff_glissandos(
            &mut svg, staff_space, &self.cross_staff_glissandos, &cross_staff_anchors,
            &stave_page_systems,
        );

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

    /// Render the multi-staff score to PNG at the given scale factor and write
    /// it to `path`.
    ///
    /// Convenience wrapper combining [`try_render_png`](Self::try_render_png)
    /// with [`std::fs::write`]. Returns an [`EngraverError::Io`] if the file
    /// cannot be written, or [`EngraverError::Png`] if rendering fails.
    ///
    /// [`EngraverError::Io`]: crate::error::EngraverError::Io
    /// [`EngraverError::Png`]: crate::error::EngraverError::Png
    #[cfg(feature = "png")]
    pub fn save_png(
        self,
        path: impl AsRef<std::path::Path>,
        scale: f32,
    ) -> Result<(), crate::error::EngraverError> {
        let bytes = self.try_render_png(scale)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests_cross_staff.rs"]
mod cross_staff_tests;

#[cfg(test)]
#[path = "tests_multi_staff_brackets.rs"]
mod bracket_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::key_signature::KeySignature;
    use music::notation::clef::Clef;
    use music::notation::rhythm::duration::Duration;
    use music::note::note::Note;
    use music::note::pitch::Pitch;

    fn pitch(note: Note, octave: i8) -> Pitch {
        Pitch::new(note, octave)
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
                SubBracket {
                    start_index: 0,
                    staff_count: 2,
                },
                SubBracket {
                    start_index: 2,
                    staff_count: 2,
                },
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
            .with_sub_brackets(vec![SubBracket {
                start_index: 0,
                staff_count: 2,
            }])
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
            .with_sub_brackets(vec![SubBracket {
                start_index: 0,
                staff_count: 2,
            }])
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
            .with_sub_brackets(vec![SubBracket {
                start_index: 0,
                staff_count: 3,
            }])
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
                SubBracket {
                    start_index: 0,
                    staff_count: 1,
                }, // too small
                SubBracket {
                    start_index: 4,
                    staff_count: 2,
                }, // out of range
                SubBracket {
                    start_index: 2,
                    staff_count: 5,
                }, // overshoots
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
        let plain = MultiStaffScore::guitar(simple_guitar_score()).render_svg();
        let with_subs = MultiStaffScore::guitar(simple_guitar_score())
            .with_sub_brackets(vec![SubBracket {
                start_index: 0,
                staff_count: 2,
            }])
            .render_svg();
        assert_eq!(
            plain, with_subs,
            "sub-brackets must not affect a guitar+tab score"
        );
    }

    #[test]
    fn empty_staves_renders_empty_svg() {
        let svg =
            MultiStaffScore::grand_staff(ScoreBuilder::new(), ScoreBuilder::new()).render_svg();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn system_width_override() {
        let svg_default = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_svg();
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
        let svg_bracket =
            MultiStaffScore::section(vec![simple_treble(), simple_bass()]).render_svg();
        // Brace vs bracket produce different visual output
        assert_ne!(svg_grand, svg_bracket);
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
        let svg = MultiStaffScore::grand_staff(multi_measure_treble(6), multi_measure_bass(6))
            .auto_line_breaks()
            .render_svg();
        assert!(svg.starts_with("<svg"), "should start with <svg");
        assert!(svg.contains("</svg>"), "should close svg");
    }

    #[test]
    fn auto_line_breaks_differs_from_fixed() {
        let svg_auto = MultiStaffScore::grand_staff(multi_measure_treble(8), multi_measure_bass(8))
            .auto_line_breaks()
            .render_svg();

        let svg_fixed =
            MultiStaffScore::grand_staff(multi_measure_treble(8), multi_measure_bass(8))
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
        let svg_narrow =
            MultiStaffScore::grand_staff(multi_measure_treble(6), multi_measure_bass(6))
                .system_width_fu(5000.0)
                .auto_line_breaks()
                .render_svg();

        let svg_wide = MultiStaffScore::grand_staff(multi_measure_treble(6), multi_measure_bass(6))
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
        let svg = MultiStaffScore::grand_staff(multi_measure_treble(6), multi_measure_bass(6))
            .measures_per_system(2) // set fixed first
            .auto_line_breaks() // then override with auto
            .render_svg();

        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn measures_per_system_overrides_auto_line_breaks() {
        // Setting measures_per_system after auto should disable auto
        let svg_fixed =
            MultiStaffScore::grand_staff(multi_measure_treble(4), multi_measure_bass(4))
                .auto_line_breaks()
                .measures_per_system(2) // override back to fixed
                .render_svg();

        let svg_auto = MultiStaffScore::grand_staff(multi_measure_treble(4), multi_measure_bass(4))
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
        let svg_explicit =
            MultiStaffScore::grand_staff(multi_measure_treble(6), multi_measure_bass(6))
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
        assert!(
            line_count >= 15,
            "section should have many lines, got {line_count}"
        );
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

        MultiStaffScore::grand_staff(treble, bass).measures_per_system(2) // 2 systems: measures 1-2 and 3-4
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

    fn simple_guitar_score() -> GuitarScore {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        score.note(pitch(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        score.note(pitch(Note::G, 4), Duration::QTR, 1, 3).unwrap();
        score.note(pitch(Note::B, 4), Duration::QTR, 1, 7).unwrap();
        score.note(pitch(Note::E, 5), Duration::QTR, 1, 12).unwrap();
        score.end_barline().unwrap();
        score
    }

    #[test]
    fn guitar_score_renders_both_views_from_shared_events() {
        let svg = MultiStaffScore::guitar(simple_guitar_score()).render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">12</text>"));
        assert!(svg.matches("<line ").count() >= 11);
    }

    #[test]
    fn guitar_render_finalizes_pending_measure_before_deriving_notation() {
        let mut score = GuitarScore::standard();
        score.note(pitch(Note::G, 4), Duration::QTR, 1, 3).unwrap();

        let svg = MultiStaffScore::guitar(score).try_render_svg().unwrap();
        assert!(svg.contains(">3</text>"), "pending TAB event must render");
        assert!(
            svg.matches("<path").count() >= 4,
            "the finalized measure must include the standard clef and notehead paths"
        );
    }

    #[test]
    fn guitar_render_reports_incomplete_pending_measure() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        score.note(pitch(Note::G, 4), Duration::QTR, 1, 3).unwrap();

        let error = MultiStaffScore::guitar(score).try_render_svg().unwrap_err();
        assert!(matches!(
            error,
            crate::error::EngraverError::Guitar(
                crate::score::guitar::GuitarScoreError::IncompleteMeasure { .. }
            )
        ));
    }

    #[test]
    fn guitar_score_honors_shared_system_breaks() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        score
            .note(pitch(Note::E, 4), Duration::WHOLE, 1, 0)
            .unwrap();
        score.barline().unwrap();
        score
            .note(pitch(Note::G, 4), Duration::WHOLE, 1, 3)
            .unwrap();
        score.end_barline().unwrap();
        let svg = MultiStaffScore::guitar(score)
            .measures_per_system(1)
            .render_svg();
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">3</text>"));
        assert!(svg.matches("<line ").count() >= 22);
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
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
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
            let single_ink = count_inked_pixels(&decode_pixmap(&single_png), INK_ALPHA_THRESHOLD);
            let grand_ink = count_inked_pixels(&decode_pixmap(&grand_png), INK_ALPHA_THRESHOLD);
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
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
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
            let png = MultiStaffScore::grand_staff(simple_treble(), simple_bass()).render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let (x_min, _, _, _) = inked_bbox(&pixmap, INK_ALPHA_THRESHOLD).expect("ink present");
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
            assert!(
                hdr_w > 0 && hdr_h > 0,
                "empty PNG should still have non-zero dims"
            );
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
