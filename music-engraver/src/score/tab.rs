//! High-level API for engraving tablature scores.
//!
//! [`TabScoreBuilder`] provides a fluent interface for building guitar/bass/ukulele
//! tablature and rendering to SVG. Fret numbers are positioned on string lines
//! with proportional spacing and automatic system breaking.
//!
//! # Example
//! ```no_run
//! use music_engraver::score::tab::TabScoreBuilder;
//!
//! let svg = TabScoreBuilder::guitar()
//!     .fret(1, 0)   // open high E
//!     .fret(2, 1)   // B string, fret 1
//!     .fret(3, 0)   // open G
//!     .barline()
//!     .fret(4, 2)   // D string, fret 2
//!     .fret(5, 3)   // A string, fret 3
//!     .end_barline()
//!     .render_svg();
//! ```

use crate::font::{bravura_font, EngravingConfig, MusicFont};
use crate::layout::bar_number::{
    layout_bar_numbers, BarNumberSlot, MeasureNumbering, BAR_NUMBER_ABOVE_STAFF_SS,
    BAR_NUMBER_FONT_SIZE_SS,
};
use crate::layout::barline::BarlineStyle;
use crate::layout::tab::{layout_fret_number, layout_muted_string, TabStaffLayout};
use crate::layout::tab_beam::{layout_tab_beam_group, TabBeamedNote};
use crate::layout::tab_hammer::{layout_tab_legato, LegatoKind};
use crate::layout::tab_harmonic::layout_tab_harmonic;
use crate::layout::tab_let_ring::{layout_tab_let_ring, layout_tab_let_ring_dash};
use crate::layout::tab_palm_mute::{layout_tab_palm_mute, layout_tab_palm_mute_dash};
use crate::layout::tab_rhythm::layout_tab_rhythm;
use crate::layout::tab_slide::layout_tab_slide;
use crate::layout::tab_vibrato::{layout_tab_vibrato, VibratoKind};
use crate::render::bar_number_renderer::draw_bar_numbers;
use crate::render::tab_beam_renderer::draw_tab_beam_group;
use crate::render::tab_hammer_renderer::draw_tab_legato;
use crate::render::tab_harmonic_renderer::draw_tab_harmonic;
use crate::render::tab_let_ring_renderer::{draw_tab_let_ring, draw_tab_let_ring_dash};
use crate::render::tab_palm_mute_renderer::{draw_tab_palm_mute, draw_tab_palm_mute_dash};
use crate::render::tab_renderer::{draw_fret_number, draw_tab_clef, draw_tab_staff_lines};
use crate::render::tab_rhythm_renderer::draw_tab_rhythm;
use crate::render::tab_slide_renderer::draw_tab_slide;
use crate::render::tab_vibrato_renderer::draw_tab_vibrato;
use crate::render::SvgWriter;

/// A single event in a tab measure.
#[derive(Clone, Debug)]
pub(crate) enum TabEvent {
    /// One or more fret numbers played simultaneously.
    /// Each entry is (string_number, fret_number) where string is 1-based.
    /// `duration_log2`: optional rhythm (0=whole, 1=half, 2=quarter, 3=eighth, etc.)
    /// `slide_out`: when true, draw a slide line from this event to the next.
    /// `legato_out`: when Some, draw a hammer-on/pull-off arc to the next event.
    /// `vibrato`: when Some, draw a wavy vibrato line above the fret number.
    /// `harmonic`: when true, draw a natural harmonic indicator (○) above the fret number.
    /// `palm_mute`: when true, draw "P.M." text above the fret number(s).
    /// `let_ring`: when true, draw "let ring" text above the fret number(s).
    /// `muted_strings`: strings displayed as "x" (dead/muted) instead of fret numbers.
    Fret {
        frets: Vec<(u8, u8)>,
        duration_log2: Option<i8>,
        slide_out: bool,
        legato_out: Option<LegatoKind>,
        vibrato: Option<VibratoKind>,
        harmonic: bool,
        palm_mute: bool,
        let_ring: bool,
        muted_strings: Vec<u8>,
    },
    /// A rest (blank space — no fret numbers).
    /// `duration_log2`: optional rhythm for rest stem display.
    Rest { duration_log2: Option<i8> },
    /// A beam group: multiple fret events connected by beam lines above the staff.
    /// Each sub-event is (frets, duration_log2).
    BeamGroup { events: Vec<(Vec<(u8, u8)>, i8)> },
}

/// A completed tab measure: events + ending barline style.
#[derive(Clone, Debug)]
pub(crate) struct TabMeasure {
    pub(crate) events: Vec<TabEvent>,
    pub(crate) barline: BarlineStyle,
}

/// Builder for constructing tablature scores and rendering to SVG.
///
/// Events are grouped into measures delimited by `barline()` / `end_barline()`.
/// Each event is either a fret number (or simultaneous chord of fret numbers)
/// or a rest. Events are spaced equally within a measure.
#[derive(Clone, Debug)]
#[must_use = "a TabScoreBuilder does nothing until .render_svg() is called"]
pub struct TabScoreBuilder {
    pub(crate) line_count: u8,
    /// Accumulated fret entries for the current in-progress multi-string event.
    /// `.fret()` pushes here; `.fret()` on a different beat or `.rest()` flushes.
    current_frets: Vec<(u8, u8)>,
    /// Pending duration for the next event (set by `.duration()`).
    pending_duration: Option<i8>,
    /// Events accumulated for the current in-progress measure.
    current_events: Vec<TabEvent>,
    /// Completed measures.
    pub(crate) measures: Vec<TabMeasure>,
    /// Measures per system. 0 = use default (4).
    measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    system_width: f64,
    /// Which measures print their number.
    measure_numbering: MeasureNumbering,
    /// When true, we are accumulating events for a beam group.
    in_beam_group: bool,
    /// Accumulated beam group sub-events: (frets, duration_log2).
    beam_group_events: Vec<(Vec<(u8, u8)>, i8)>,
    /// When true, the next flushed Fret event gets `slide_out = true`.
    pending_slide: bool,
    /// When Some, the next flushed Fret event gets `legato_out` set.
    pending_legato: Option<LegatoKind>,
    /// When Some, the next flushed Fret event gets `vibrato` set.
    pending_vibrato: Option<VibratoKind>,
    /// When true, the next flushed Fret event gets `harmonic = true`.
    pending_harmonic: bool,
    /// When true, the next flushed Fret event gets `palm_mute = true`.
    pending_palm_mute: bool,
    /// When true, the next flushed Fret event gets `let_ring = true`.
    pending_let_ring: bool,
    /// Accumulated muted string numbers for the current event.
    current_muted: Vec<u8>,
}

impl TabScoreBuilder {
    /// Create a new tab score builder with a custom number of strings.
    pub fn new(line_count: u8) -> Self {
        Self {
            line_count,
            current_frets: Vec::new(),
            pending_duration: None,
            current_events: Vec::new(),
            measures: Vec::new(),
            measures_per_system: 4,
            system_width: 0.0,
            measure_numbering: MeasureNumbering::Hidden,
            in_beam_group: false,
            beam_group_events: Vec::new(),
            pending_slide: false,
            pending_legato: None,
            pending_vibrato: None,
            pending_harmonic: false,
            pending_palm_mute: false,
            pending_let_ring: false,
            current_muted: Vec::new(),
        }
    }

    /// Create a standard 6-string guitar tab.
    pub fn guitar() -> Self {
        Self::new(6)
    }

    /// Create a 4-string tab (bass guitar, ukulele).
    pub fn four_string() -> Self {
        Self::new(4)
    }

    /// Set the number of measures per system.
    pub fn measures_per_system(mut self, n: usize) -> Self {
        self.measures_per_system = n;
        self
    }

    /// Set the system width in font design units.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Choose which measures print their number (1-based) above the staff.
    pub fn measure_numbering(mut self, numbering: MeasureNumbering) -> Self {
        self.measure_numbering = numbering;
        self
    }

    /// Flush any pending fret entries (including muted strings) as a single event.
    fn flush_frets(&mut self) {
        if !self.current_frets.is_empty() || !self.current_muted.is_empty() {
            if self.in_beam_group {
                self.flush_beam_frets();
            } else {
                let frets = std::mem::take(&mut self.current_frets);
                let duration_log2 = self.pending_duration.take();
                let slide_out = std::mem::take(&mut self.pending_slide);
                let legato_out = self.pending_legato.take();
                let vibrato = self.pending_vibrato.take();
                let harmonic = std::mem::take(&mut self.pending_harmonic);
                let palm_mute = std::mem::take(&mut self.pending_palm_mute);
                let let_ring = std::mem::take(&mut self.pending_let_ring);
                let muted_strings = std::mem::take(&mut self.current_muted);
                self.current_events.push(TabEvent::Fret {
                    frets,
                    duration_log2,
                    slide_out,
                    legato_out,
                    vibrato,
                    harmonic,
                    palm_mute,
                    let_ring,
                    muted_strings,
                });
            }
        }
    }

    /// Add a single fret number. Consecutive `.fret()` calls on different
    /// strings before a `.next()` or `.rest()` are grouped into a chord
    /// (simultaneous fret numbers on multiple strings).
    ///
    /// `string` is 1-based (1 = highest pitch = bottom line in TAB).
    /// `fret` is the fret number (0 = open string).
    pub fn fret(mut self, string: u8, fret: u8) -> Self {
        self.current_frets.push((string, fret));
        self
    }

    /// End the current beat and start a new one.
    ///
    /// This flushes any accumulated `.fret()` calls as a single simultaneous
    /// event and prepares for the next beat. If you want each `.fret()` call
    /// to be its own beat, call `.next()` between them or just use `.fret()`
    /// without `.next()` for chords.
    pub fn next(mut self) -> Self {
        self.flush_frets();
        self
    }

    /// Set the duration for the next event (fret or rest).
    ///
    /// `duration_log2`: -1=breve, 0=whole, 1=half, 2=quarter, 3=eighth, 4=sixteenth, etc.
    /// The duration is consumed by the next `.fret()` flush or `.rest()` call.
    /// When set, a rhythm stem (and flag for eighths and shorter) is drawn
    /// above the tab staff.
    pub fn duration(mut self, duration_log2: i8) -> Self {
        self.pending_duration = Some(duration_log2);
        self
    }

    /// Convenience: set duration to quarter note (duration_log2 = 2).
    pub fn quarter(self) -> Self {
        self.duration(2)
    }

    /// Convenience: set duration to eighth note (duration_log2 = 3).
    pub fn eighth(self) -> Self {
        self.duration(3)
    }

    /// Convenience: set duration to half note (duration_log2 = 1).
    pub fn half(self) -> Self {
        self.duration(1)
    }

    /// Convenience: set duration to whole note (duration_log2 = 0).
    pub fn whole(self) -> Self {
        self.duration(0)
    }

    /// Begin accumulating events for a beam group.
    ///
    /// All `.fret()` calls between `.beam_start()` and `.beam_end()` are
    /// collected and rendered with connecting beam lines above the tab staff
    /// instead of individual flags. Each sub-event must have a duration set
    /// via `.eighth()`, `.duration()`, etc. Sub-events are separated by `.next()`.
    pub fn beam_start(mut self) -> Self {
        self.flush_frets();
        self.in_beam_group = true;
        self.beam_group_events.clear();
        self
    }

    /// End the beam group and flush it as a single `BeamGroup` event.
    ///
    /// The accumulated sub-events are connected by horizontal beam lines.
    /// Requires at least 2 sub-events with eighth-or-shorter durations.
    pub fn beam_end(mut self) -> Self {
        self.flush_beam_frets();
        let events = std::mem::take(&mut self.beam_group_events);
        self.in_beam_group = false;
        if !events.is_empty() {
            self.current_events.push(TabEvent::BeamGroup { events });
        }
        self
    }

    /// Flush current frets into the beam group accumulator.
    fn flush_beam_frets(&mut self) {
        if !self.current_frets.is_empty() {
            let frets = std::mem::take(&mut self.current_frets);
            let dur = self.pending_duration.take().unwrap_or(3); // default to eighth
            self.beam_group_events.push((frets, dur));
        }
        // Muted strings not supported in beam groups — clear to avoid stale data
        self.current_muted.clear();
    }

    /// Mark the most recently added fret event (or the next one to be flushed)
    /// for a slide into the following event. A diagonal line will be drawn from
    /// this event's fret position(s) to the next event's matching string(s).
    ///
    /// If called before any `.fret()`, or after a `.rest()`, this is a no-op
    /// (slides only apply to fret events).
    pub fn slide(mut self) -> Self {
        if !self.current_frets.is_empty() {
            // Frets are still accumulating — mark the pending flush for slide
            self.pending_slide = true;
        } else if let Some(TabEvent::Fret { slide_out, .. }) = self.current_events.last_mut() {
            // Frets already flushed — mark the last event
            *slide_out = true;
        }
        self
    }

    /// Mark the current fret event for a hammer-on arc to the next event.
    ///
    /// A curved arc with "H" is drawn from this fret event to the next
    /// fret event on each matching string. Works like `.slide()` but
    /// renders a curve instead of a diagonal line.
    pub fn hammer(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_legato = Some(LegatoKind::HammerOn);
        } else if let Some(TabEvent::Fret { legato_out, .. }) = self.current_events.last_mut() {
            *legato_out = Some(LegatoKind::HammerOn);
        }
        self
    }

    /// Mark the current fret event for a pull-off arc to the next event.
    ///
    /// A curved arc with "P" is drawn from this fret event to the next
    /// fret event on each matching string. Works like `.slide()` but
    /// renders a curve instead of a diagonal line.
    pub fn pull(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_legato = Some(LegatoKind::PullOff);
        } else if let Some(TabEvent::Fret { legato_out, .. }) = self.current_events.last_mut() {
            *legato_out = Some(LegatoKind::PullOff);
        }
        self
    }

    /// Mark the current fret event for a vibrato wavy line above the fret
    /// number(s). Standard vibrato has a moderate wave amplitude.
    pub fn vibrato(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_vibrato = Some(VibratoKind::Normal);
        } else if let Some(TabEvent::Fret { vibrato, .. }) = self.current_events.last_mut() {
            *vibrato = Some(VibratoKind::Normal);
        }
        self
    }

    /// Mark the current fret event for a wide vibrato wavy line above the
    /// fret number(s). Wide vibrato has a taller wave amplitude, indicating
    /// a more exaggerated pitch oscillation.
    pub fn wide_vibrato(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_vibrato = Some(VibratoKind::Wide);
        } else if let Some(TabEvent::Fret { vibrato, .. }) = self.current_events.last_mut() {
            *vibrato = Some(VibratoKind::Wide);
        }
        self
    }

    /// Mark the current fret event as a natural harmonic.
    ///
    /// A small circle (○) is drawn above the fret number(s), indicating
    /// the string should be lightly touched at that fret position to
    /// produce a harmonic overtone. Common harmonic frets: 5, 7, 12.
    pub fn harmonic(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_harmonic = true;
        } else if let Some(TabEvent::Fret { harmonic, .. }) = self.current_events.last_mut() {
            *harmonic = true;
        }
        self
    }

    /// Mark the current fret event for palm muting.
    ///
    /// "P.M." text is drawn above the fret number(s). When consecutive events
    /// are palm-muted, a dashed continuation line is drawn between them.
    pub fn palm_mute(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_palm_mute = true;
        } else if let Some(TabEvent::Fret { palm_mute, .. }) = self.current_events.last_mut() {
            *palm_mute = true;
        }
        self
    }

    /// Mark the current fret event for "let ring" sustain.
    ///
    /// "let ring" text is drawn above the fret number(s). When consecutive events
    /// are marked let ring, a dashed continuation line is drawn between them.
    pub fn let_ring(mut self) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_let_ring = true;
        } else if let Some(TabEvent::Fret { let_ring, .. }) = self.current_events.last_mut() {
            *let_ring = true;
        }
        self
    }

    /// Add a muted/dead string marker ("x") on the given string.
    ///
    /// Muted strings are grouped with fret numbers into the same beat event.
    /// Multiple `.mute()` calls before `.next()` or `.rest()` add multiple
    /// muted strings to the same event. Commonly used for percussive muted
    /// strums in guitar tab.
    ///
    /// `string` is 1-based (1 = highest pitch = bottom line in TAB).
    pub fn mute(mut self, string: u8) -> Self {
        self.current_muted.push(string);
        self
    }

    /// Add a rest (empty beat with no fret numbers).
    pub fn rest(mut self) -> Self {
        self.flush_frets();
        let duration_log2 = self.pending_duration.take();
        self.current_events.push(TabEvent::Rest { duration_log2 });
        self
    }

    /// End the current measure with a single barline.
    pub fn barline(mut self) -> Self {
        self.flush_frets();
        let events = std::mem::take(&mut self.current_events);
        self.measures.push(TabMeasure {
            events,
            barline: BarlineStyle::Single,
        });
        self
    }

    /// End the current measure with a final (double) barline.
    pub fn end_barline(mut self) -> Self {
        self.flush_frets();
        let events = std::mem::take(&mut self.current_events);
        self.measures.push(TabMeasure {
            events,
            barline: BarlineStyle::Final,
        });
        self
    }

    /// End the current measure with a specific barline style.
    pub fn barline_style(mut self, style: BarlineStyle) -> Self {
        self.flush_frets();
        let events = std::mem::take(&mut self.current_events);
        self.measures.push(TabMeasure {
            events,
            barline: style,
        });
        self
    }

    /// Flush any pending events as a final measure.
    pub(crate) fn flush_pending(&mut self) {
        self.flush_frets();
        if !self.current_events.is_empty() {
            let events = std::mem::take(&mut self.current_events);
            self.measures.push(TabMeasure {
                events,
                barline: BarlineStyle::Final,
            });
        }
    }

    /// Effective measures per system.
    fn effective_mps(&self) -> usize {
        if self.measures_per_system == 0 {
            4
        } else {
            self.measures_per_system
        }
    }

    /// Render the tab score to an SVG string.
    ///
    /// # Panics
    ///
    /// Panics if the bundled Bravura font lacks the TAB clef glyph (unreachable
    /// in normal operation since Bravura contains all required SMuFL glyphs).
    #[must_use = "the SVG string is returned but not used"]
    pub fn render_svg(self) -> String {
        self.try_render_svg()
            .expect("bundled Bravura font contains all required SMuFL glyphs")
    }

    /// Render the tab score to an SVG string, returning an error on failure.
    #[must_use = "the SVG string is returned but not used"]
    pub fn try_render_svg(mut self) -> Result<String, crate::error::EngraverError> {
        self.flush_pending();

        if self.measures.is_empty() {
            return Ok(String::from(
                "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            ));
        }

        let font = bravura_font();
        let config = font.engraving_config();
        let staff_space = config.staff_space;

        let sys_width = if self.system_width > 0.0 {
            self.system_width
        } else {
            40.0 * staff_space
        };

        let mps = self.effective_mps();

        // Break measures into system chunks
        let chunks = break_tab_measures(self.measures.len(), mps);

        // TAB clef occupies ~2.5 staff spaces + 0.5 padding
        let clef_width = 3.0 * staff_space;

        // Compute page dimensions
        let tab_staff_height = staff_space * (self.line_count.saturating_sub(1)) as f64;
        let inter_system_gap = 4.0 * staff_space;
        let total_systems = chunks.len();
        let page_height = if total_systems > 0 {
            total_systems as f64 * tab_staff_height
                + (total_systems.saturating_sub(1)) as f64 * inter_system_gap
        } else {
            0.0
        };

        let vb_margin = staff_space;
        // Bar numbers sit above the first staff: keep them inside the box.
        let top_margin = if self.measure_numbering == MeasureNumbering::Hidden {
            vb_margin
        } else {
            (BAR_NUMBER_ABOVE_STAFF_SS + BAR_NUMBER_FONT_SIZE_SS) * staff_space
        };
        let vb_x = -vb_margin;
        let vb_y = -top_margin;
        let vb_w = sys_width + 2.0 * vb_margin;
        let vb_h = page_height + top_margin + vb_margin;
        let px_per_unit = 7.0 / staff_space;
        let px_w = vb_w * px_per_unit;
        let px_h = vb_h * px_per_unit;

        let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

        for (sys_idx, (start, end)) in chunks.iter().enumerate() {
            let sys_y = sys_idx as f64 * (tab_staff_height + inter_system_gap);

            let tab_staff =
                TabStaffLayout::new(0.0, sys_y, sys_width, staff_space, self.line_count);

            // Draw staff lines
            draw_tab_staff_lines(&mut svg, &tab_staff, &config);

            // Draw TAB clef
            draw_tab_clef(&mut svg, &tab_staff, &font)?;

            // Layout and draw measures for this system
            let measures_in_system = &self.measures[*start..*end];
            let content_width = sys_width - clef_width;
            let measure_width = if measures_in_system.is_empty() {
                content_width
            } else {
                content_width / measures_in_system.len() as f64
            };

            // Bar numbers: each measure begins at its left barline (after the
            // TAB clef for the system's first measure).
            let numbers = layout_bar_numbers(
                self.measure_numbering,
                (*start..*end).map(|index| BarNumberSlot {
                    number: i32::try_from(index + 1).unwrap_or(i32::MAX),
                    x: clef_width + (index - start) as f64 * measure_width,
                    system_start: index == *start,
                    numbered: true,
                }),
                sys_y,
                staff_space,
            );
            draw_bar_numbers(&mut svg, &numbers, staff_space);

            for (m_idx, measure) in measures_in_system.iter().enumerate() {
                let measure_x = clef_width + m_idx as f64 * measure_width;

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

        Ok(svg.to_svg())
    }

    /// Render the tab score to a PNG byte vector at the given scale factor.
    ///
    /// Scale 1.0 maps SVG user units 1:1 to pixels; 2.0 produces a 2× image.
    ///
    /// # Panics
    /// Panics if the bundled Bravura font lacks the TAB clef glyph (unreachable
    /// in normal operation since Bravura contains all required SMuFL glyphs).
    #[cfg(feature = "png")]
    #[must_use = "the PNG bytes are returned but not used"]
    pub fn render_png(self, scale: f32) -> Vec<u8> {
        self.try_render_png(scale)
            .expect("bundled Bravura font and PNG pipeline should not fail for valid input")
    }

    /// Render the tab score to a PNG byte vector, returning an error on failure.
    #[cfg(feature = "png")]
    pub fn try_render_png(self, scale: f32) -> Result<Vec<u8>, crate::error::EngraverError> {
        let svg = self.try_render_svg()?;
        let mut renderer = crate::render::png::PngRenderer::new(scale);
        renderer.load_system_fonts();
        Ok(renderer.render_png(&svg)?)
    }

    /// Render the tab score to PNG at the given scale factor and write it to
    /// `path`.
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

impl Default for TabScoreBuilder {
    fn default() -> Self {
        Self::guitar()
    }
}

/// Break N measures into system chunks of `mps` measures each.
fn break_tab_measures(total: usize, mps: usize) -> Vec<(usize, usize)> {
    let mps = mps.max(1);
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < total {
        let end = (start + mps).min(total);
        chunks.push((start, end));
        start = end;
    }
    chunks
}

/// Draw a single tab measure: fret numbers at evenly spaced positions + barline.
pub(crate) fn draw_tab_measure(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    tab_staff: &TabStaffLayout,
    measure: &TabMeasure,
    measure_x: f64,
    measure_width: f64,
) -> Result<(), crate::font::FontError> {
    let event_count = measure.events.len();
    if event_count == 0 {
        // Still draw barline at the end
        draw_measure_barline(
            svg,
            font,
            config,
            tab_staff,
            measure_x + measure_width,
            &measure.barline,
        )?;
        return Ok(());
    }

    // Padding before first event and after last event
    let padding = measure_width * 0.08;
    let usable_width = measure_width - 2.0 * padding;

    // Compute x positions for events (evenly spaced)
    let spacing = if event_count > 1 {
        usable_width / (event_count - 1) as f64
    } else {
        0.0
    };

    let stem_width = config.stem_thickness_fu();

    for (e_idx, event) in measure.events.iter().enumerate() {
        let event_x = if event_count == 1 {
            measure_x + padding + usable_width / 2.0
        } else {
            measure_x + padding + e_idx as f64 * spacing
        };

        match event {
            TabEvent::Fret {
                frets,
                duration_log2,
                muted_strings,
                ..
            } => {
                for &(string, fret) in frets {
                    let layout = layout_fret_number(tab_staff, string, fret, event_x);
                    draw_fret_number(svg, &layout);
                }
                for &string in muted_strings {
                    let layout = layout_muted_string(tab_staff, string, event_x);
                    draw_fret_number(svg, &layout);
                }
                // Draw rhythm stem + flag above the staff if duration is set
                if let Some(dur) = duration_log2 {
                    if let Some(rhythm_layout) =
                        layout_tab_rhythm(tab_staff, event_x, *dur, stem_width)
                    {
                        draw_tab_rhythm(svg, &rhythm_layout, font)?;
                    }
                }
            }
            TabEvent::Rest { duration_log2 } => {
                // Rests with duration get a rhythm stem (stem-only, no fret numbers)
                if let Some(dur) = duration_log2 {
                    if let Some(rhythm_layout) =
                        layout_tab_rhythm(tab_staff, event_x, *dur, stem_width)
                    {
                        draw_tab_rhythm(svg, &rhythm_layout, font)?;
                    }
                }
            }
            TabEvent::BeamGroup {
                events: beam_events,
            } => {
                draw_tab_beam_group_event(
                    svg,
                    config,
                    tab_staff,
                    beam_events,
                    event_x,
                    spacing.max(usable_width / (event_count.max(1)) as f64),
                    stem_width,
                );
            }
        }
    }

    // Second pass: draw slide lines between consecutive fret events
    let slide_stroke = config.stem_thickness_fu();
    for i in 0..event_count.saturating_sub(1) {
        if let TabEvent::Fret {
            frets: src_frets,
            slide_out: true,
            ..
        } = &measure.events[i]
        {
            // Find the next Fret event (skip rests)
            if let Some(target_idx) =
                (i + 1..event_count).find(|&j| matches!(&measure.events[j], TabEvent::Fret { .. }))
            {
                if let TabEvent::Fret {
                    frets: tgt_frets, ..
                } = &measure.events[target_idx]
                {
                    let src_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + i as f64 * spacing
                    };
                    let tgt_x = measure_x + padding + target_idx as f64 * spacing;

                    // Draw a slide line for each string that appears in both events
                    for &(src_str, _) in src_frets {
                        if tgt_frets.iter().any(|&(ts, _)| ts == src_str) {
                            if let Some(slide_layout) =
                                layout_tab_slide(tab_staff, src_str, src_x, tgt_x, slide_stroke)
                            {
                                draw_tab_slide(svg, &slide_layout);
                            }
                        }
                    }
                }
            }
        }
    }

    // Third pass: draw vibrato, harmonics, and sustained-technique labels.
    for (e_idx, event) in measure.events.iter().enumerate() {
        if let TabEvent::Fret {
            frets,
            vibrato,
            harmonic,
            palm_mute,
            let_ring,
            ..
        } = event
        {
            let event_x = if event_count == 1 {
                measure_x + padding + usable_width / 2.0
            } else {
                measure_x + padding + e_idx as f64 * spacing
            };

            let technique_stroke = config.stem_thickness_fu();

            // Vibrato wavy line
            if let Some(kind) = vibrato {
                for &(string, _) in frets {
                    let vib_layout =
                        layout_tab_vibrato(tab_staff, string, event_x, *kind, technique_stroke);
                    draw_tab_vibrato(svg, &vib_layout);
                }
            }

            // Natural harmonic indicator (small ○ above fret number)
            if *harmonic {
                for &(string, _) in frets {
                    let harm_layout = layout_tab_harmonic(tab_staff, string, event_x);
                    draw_tab_harmonic(svg, &harm_layout, font)?;
                }
            }

            // Palm mute "P.M." text above the staff
            if *palm_mute {
                let pm_layout = layout_tab_palm_mute(tab_staff, event_x);
                draw_tab_palm_mute(svg, &pm_layout);
            }

            // "let ring" text above the staff (higher than P.M.)
            if *let_ring {
                let lr_layout = layout_tab_let_ring(tab_staff, event_x);
                draw_tab_let_ring(svg, &lr_layout);
            }
        }
    }

    // Palm mute pass: draw dashed continuation lines between consecutive palm-muted events
    {
        let technique_stroke = config.stem_thickness_fu();
        let mut pm_start: Option<usize> = None;

        for (e_idx, event) in measure.events.iter().enumerate() {
            let is_pm = matches!(
                event,
                TabEvent::Fret {
                    palm_mute: true,
                    ..
                }
            );

            if is_pm && pm_start.is_none() {
                pm_start = Some(e_idx);
            }

            // If we hit a non-PM event or the end of the events, close the current PM span
            if (!is_pm || e_idx == event_count - 1) && pm_start.is_some() {
                let start_idx = pm_start.unwrap();
                let end_idx = if is_pm { e_idx } else { e_idx - 1 };

                // Draw a dashed line if the PM span covers more than one event
                if end_idx > start_idx {
                    let start_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + start_idx as f64 * spacing
                    };
                    let end_x = measure_x + padding + end_idx as f64 * spacing;

                    if let Some(dash_layout) =
                        layout_tab_palm_mute_dash(tab_staff, start_x, end_x, technique_stroke)
                    {
                        draw_tab_palm_mute_dash(svg, &dash_layout);
                    }
                }

                if !is_pm {
                    pm_start = None;
                }
            }
        }
    }

    // Let ring pass: draw dashed continuation lines between consecutive let-ring events
    {
        let technique_stroke = config.stem_thickness_fu();
        let mut lr_start: Option<usize> = None;

        for (e_idx, event) in measure.events.iter().enumerate() {
            let is_lr = matches!(event, TabEvent::Fret { let_ring: true, .. });

            if is_lr && lr_start.is_none() {
                lr_start = Some(e_idx);
            }

            if (!is_lr || e_idx == event_count - 1) && lr_start.is_some() {
                let start_idx = lr_start.unwrap();
                let end_idx = if is_lr { e_idx } else { e_idx - 1 };

                if end_idx > start_idx {
                    let start_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + start_idx as f64 * spacing
                    };
                    let end_x = measure_x + padding + end_idx as f64 * spacing;

                    if let Some(dash_layout) =
                        layout_tab_let_ring_dash(tab_staff, start_x, end_x, technique_stroke)
                    {
                        draw_tab_let_ring_dash(svg, &dash_layout);
                    }
                }

                if !is_lr {
                    lr_start = None;
                }
            }
        }
    }

    // Fourth pass: draw hammer-on/pull-off arcs between consecutive fret events
    for i in 0..event_count.saturating_sub(1) {
        if let TabEvent::Fret {
            frets: src_frets,
            legato_out: Some(kind),
            ..
        } = &measure.events[i]
        {
            if let Some(target_idx) =
                (i + 1..event_count).find(|&j| matches!(&measure.events[j], TabEvent::Fret { .. }))
            {
                if let TabEvent::Fret {
                    frets: tgt_frets, ..
                } = &measure.events[target_idx]
                {
                    let src_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + i as f64 * spacing
                    };
                    let tgt_x = measure_x + padding + target_idx as f64 * spacing;

                    for &(src_str, _) in src_frets {
                        if tgt_frets.iter().any(|&(ts, _)| ts == src_str) {
                            if let Some(legato_layout) = layout_tab_legato(
                                tab_staff,
                                src_str,
                                src_x,
                                tgt_x,
                                *kind,
                                slide_stroke,
                            ) {
                                draw_tab_legato(svg, &legato_layout);
                            }
                        }
                    }
                }
            }
        }
    }

    // Barline at the right edge of the measure
    draw_measure_barline(
        svg,
        font,
        config,
        tab_staff,
        measure_x + measure_width,
        &measure.barline,
    )?;

    Ok(())
}

/// Draw a beam group event: fret numbers + beamed rhythm stems above the staff.
///
/// Sub-events are spaced evenly within the allocated `group_width` starting
/// at `start_x`. Each sub-event's fret numbers are drawn, then beam layout
/// and rendering connect the stems.
fn draw_tab_beam_group_event(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    tab_staff: &TabStaffLayout,
    events: &[(Vec<(u8, u8)>, i8)],
    start_x: f64,
    group_width: f64,
    stem_width: f64,
) {
    if events.is_empty() {
        return;
    }

    // Compute x positions for sub-events within the beam group
    let sub_count = events.len();
    let sub_spacing = if sub_count > 1 {
        group_width / (sub_count - 1) as f64
    } else {
        0.0
    };

    let mut beam_notes = Vec::with_capacity(sub_count);

    for (i, (frets, dur)) in events.iter().enumerate() {
        let x = if sub_count == 1 {
            start_x
        } else {
            start_x + i as f64 * sub_spacing
        };

        // Draw fret numbers
        for &(string, fret) in frets {
            let layout = layout_fret_number(tab_staff, string, fret, x);
            draw_fret_number(svg, &layout);
        }

        beam_notes.push(TabBeamedNote {
            x,
            duration_log2: *dur,
        });
    }

    // Layout and draw beams
    let beam_thickness = config.beam_thickness_fu();
    let beam_gap = config.beam_spacing_fu();

    if let Some(beam_layout) =
        layout_tab_beam_group(tab_staff, &beam_notes, stem_width, beam_thickness, beam_gap)
    {
        draw_tab_beam_group(svg, &beam_layout);
    }
}

/// Draw a barline at the given x position spanning the tab staff.
///
/// Draws thin/thick vertical lines directly rather than going through
/// `StaffLayout`-based barline functions, since tab staves have variable
/// line counts and don't use `StaffLayout`.
pub(crate) fn draw_measure_barline(
    svg: &mut SvgWriter,
    _font: &MusicFont,
    config: &EngravingConfig,
    tab_staff: &TabStaffLayout,
    x: f64,
    style: &BarlineStyle,
) -> Result<(), crate::font::FontError> {
    let y_top = tab_staff.y_origin;
    let y_bottom = tab_staff.bottom_y();
    let thin = config.thin_barline_thickness_fu();
    let thick = config.to_font_units(config.thick_barline_thickness);
    let sep = config.to_font_units(config.barline_separation);

    match style {
        BarlineStyle::Single => {
            svg.add_line(x, y_top, x, y_bottom, "black", thin);
        }
        BarlineStyle::Double => {
            svg.add_line(x, y_top, x, y_bottom, "black", thin);
            svg.add_line(x + sep, y_top, x + sep, y_bottom, "black", thin);
        }
        BarlineStyle::Final => {
            svg.add_line(x, y_top, x, y_bottom, "black", thin);
            svg.add_line(x + sep, y_top, x + sep, y_bottom, "black", thick);
        }
        BarlineStyle::StartRepeat => {
            svg.add_line(x, y_top, x, y_bottom, "black", thick);
            svg.add_line(
                x + sep + thick / 2.0,
                y_top,
                x + sep + thick / 2.0,
                y_bottom,
                "black",
                thin,
            );
        }
        BarlineStyle::EndRepeat => {
            svg.add_line(x, y_top, x, y_bottom, "black", thin);
            svg.add_line(x + sep, y_top, x + sep, y_bottom, "black", thick);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tab_produces_minimal_svg() {
        let svg = TabScoreBuilder::guitar().render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn guitar_tab_has_six_staff_lines() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        // 6 staff lines + barline lines (Final = thin + thick = 2 extra)
        let total_lines = svg.matches("<line ").count();
        assert!(
            total_lines >= 6,
            "guitar tab should have at least 6 staff lines, got {total_lines}"
        );
        // Verify exactly 6 staff lines by checking they span the full width
        // (staff lines are horizontal; barlines are vertical)
    }

    #[test]
    fn four_string_tab_has_four_lines() {
        let svg = TabScoreBuilder::four_string()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        // 4 staff lines + barline lines
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 4,
            "4-string tab should have at least 4 staff lines, got {line_count}"
        );
    }

    #[test]
    fn fret_number_appears_in_svg() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">5</text>"), "fret number 5 should appear");
    }

    #[test]
    fn two_digit_fret_number_appears() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 12)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">12</text>"), "fret 12 should appear");
    }

    #[test]
    fn open_string_shows_zero() {
        let svg = TabScoreBuilder::guitar()
            .fret(3, 0)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">0</text>"), "open string should show 0");
    }

    #[test]
    fn chord_shows_multiple_fret_numbers() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .fret(2, 1)
            .fret(3, 0)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">0</text>"), "should show open string");
        assert!(svg.contains(">1</text>"), "should show fret 1");
        // 3 fret numbers → 3 text + 3 rect elements
        assert_eq!(
            svg.matches("<text ").count(),
            3,
            "chord should show 3 fret numbers"
        );
    }

    #[test]
    fn next_separates_beats() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .next()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">2</text>"));
        assert_eq!(svg.matches("<text ").count(), 2);
    }

    #[test]
    fn rest_produces_no_fret_numbers() {
        let svg = TabScoreBuilder::guitar().rest().end_barline().render_svg();
        // No fret number text (but staff lines + clef still present)
        assert!(!svg.contains(">0</text>"));
        assert!(!svg.contains(">5</text>"));
    }

    #[test]
    fn tab_clef_appears() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        assert!(svg.contains("<path "), "TAB clef path should appear");
    }

    #[test]
    fn multiple_measures_produce_barlines() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .barline()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        // At least 2 barline line elements beyond the 6 staff lines
        let total_lines = svg.matches("<line ").count();
        assert!(
            total_lines > 6,
            "multiple measures should add barline lines beyond staff lines, got {total_lines}"
        );
    }

    #[test]
    fn multi_system_tab() {
        let svg = TabScoreBuilder::guitar()
            .measures_per_system(2)
            .fret(1, 0)
            .barline()
            .fret(1, 2)
            .barline()
            .fret(1, 3)
            .barline()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // 4 measures / 2 per system = 2 systems
        // Each system has 6 staff lines + barline lines
        let total_lines = svg.matches("<line ").count();
        assert!(
            total_lines >= 12,
            "2 systems should have at least 12 staff lines, got {total_lines}"
        );
        // Both systems should have TAB clef paths
        assert!(
            svg.matches("<path ").count() >= 2,
            "should have at least 2 TAB clef paths"
        );
    }

    #[test]
    fn system_width_override() {
        let narrow = TabScoreBuilder::guitar()
            .system_width_fu(3000.0)
            .fret(1, 0)
            .end_barline()
            .render_svg();
        let wide = TabScoreBuilder::guitar()
            .system_width_fu(8000.0)
            .fret(1, 0)
            .end_barline()
            .render_svg();
        assert_ne!(
            narrow, wide,
            "different widths should produce different SVGs"
        );
    }

    #[test]
    fn measure_numbers_shown_when_enabled() {
        let svg = TabScoreBuilder::guitar()
            .measure_numbering(MeasureNumbering::SystemStart)
            .fret(1, 0)
            .barline()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">1</text>"), "should show measure number 1");
    }

    #[test]
    fn measure_numbers_not_shown_by_default() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .barline()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        // The only text elements should be fret numbers, not measure numbers
        // Fret numbers are "0" and "2"; measure number "1" should not appear
        // unless show_measure_numbers is enabled.
        // (We can't easily distinguish, so just check for expected fret texts)
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">2</text>"));
    }

    #[test]
    fn barline_style_respects_repeat() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .barline_style(BarlineStyle::StartRepeat)
            .fret(1, 2)
            .barline_style(BarlineStyle::EndRepeat)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        // Repeat barlines produce more line elements than single barlines
        let total_lines = svg.matches("<line ").count();
        assert!(
            total_lines > 6,
            "repeat barlines should add lines, got {total_lines}"
        );
    }

    #[test]
    fn default_is_guitar() {
        let default_svg = TabScoreBuilder::default()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        let guitar_svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        assert_eq!(
            default_svg, guitar_svg,
            "default should be guitar (6-string)"
        );
    }

    #[test]
    fn flush_pending_auto_closes_measure() {
        let svg = TabScoreBuilder::guitar().fret(1, 0).fret(2, 1).render_svg();
        // Even without explicit barline, pending events are flushed
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">1</text>"));
    }

    #[test]
    fn multi_system_has_two_staff_groups() {
        let svg = TabScoreBuilder::guitar()
            .measures_per_system(1)
            .fret(1, 0)
            .barline()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        // 2 systems × 6 staff lines = 12
        let staff_lines = svg.matches("<line ").count();
        assert!(
            staff_lines >= 12,
            "2 systems should have at least 12 staff lines, got {staff_lines}"
        );
    }

    #[test]
    fn break_tab_measures_splits_correctly() {
        let chunks = break_tab_measures(7, 3);
        assert_eq!(chunks, vec![(0, 3), (3, 6), (6, 7)]);
    }

    #[test]
    fn break_tab_measures_single_system() {
        let chunks = break_tab_measures(3, 4);
        assert_eq!(chunks, vec![(0, 3)]);
    }

    #[test]
    fn break_tab_measures_empty() {
        let chunks = break_tab_measures(0, 4);
        assert!(chunks.is_empty());
    }

    #[test]
    fn duration_adds_rhythm_stem() {
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // Should have a stem line above the staff (in addition to 6 staff lines + barline)
        let line_count = svg.matches("<line ").count();
        // 6 staff lines + 2 barline lines (final) + 1 rhythm stem = 9
        assert!(
            line_count >= 9,
            "quarter note rhythm stem should add a line, got {line_count}"
        );
    }

    #[test]
    fn eighth_duration_adds_stem_and_flag() {
        let svg = TabScoreBuilder::guitar()
            .eighth()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // Should have a flag path in addition to the TAB clef path
        let path_count = svg.matches("<path ").count();
        assert!(
            path_count >= 2,
            "eighth note should have TAB clef + flag path, got {path_count}"
        );
    }

    #[test]
    fn whole_note_no_stem() {
        let svg = TabScoreBuilder::guitar()
            .whole()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // Whole notes have no stem: only 6 staff lines + barline lines
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 8,
            "whole note should have no rhythm stem: 6 staff + 2 barline = 8, got {line_count}"
        );
    }

    #[test]
    fn no_duration_no_stem() {
        let svg_no_dur = TabScoreBuilder::guitar()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        let svg_with_dur = TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        assert_ne!(
            svg_no_dur, svg_with_dur,
            "fret without duration should differ from fret with duration"
        );
        // Without duration: no rhythm stem line beyond staff+barline
        let lines_no_dur = svg_no_dur.matches("<line ").count();
        let lines_with_dur = svg_with_dur.matches("<line ").count();
        assert!(
            lines_with_dur > lines_no_dur,
            "duration should add a stem line ({lines_with_dur} vs {lines_no_dur})"
        );
    }

    #[test]
    fn duration_on_rest_draws_stem() {
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .rest()
            .end_barline()
            .render_svg();
        // Rest with duration should draw a stem but no fret numbers
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 9,
            "rest with duration should have rhythm stem, got {line_count}"
        );
        // No fret number text
        assert!(!svg.contains(">0</text>"));
    }

    #[test]
    fn half_note_stem_no_flag() {
        let svg = TabScoreBuilder::guitar()
            .half()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // Half note: stem but no flag — only 1 path (TAB clef)
        let path_count = svg.matches("<path ").count();
        assert_eq!(
            path_count, 1,
            "half note should have TAB clef only (no flag), got {path_count}"
        );
        // But should have a rhythm stem line
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 9,
            "half note should have rhythm stem, got {line_count}"
        );
    }

    #[test]
    fn multiple_events_with_mixed_durations() {
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .next()
            .fret(1, 3) // no duration — no stem
            .end_barline()
            .render_svg();
        // 2 stems (quarter + eighth), 1 flag (eighth), 3 fret numbers
        let text_count = svg.matches("<text ").count();
        assert_eq!(text_count, 3, "should have 3 fret numbers");
        // 6 staff lines + 2 barline + 2 stems = 10
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 10,
            "should have 6 staff + 2 barline + 2 rhythm stems = 10, got {line_count}"
        );
        // 1 TAB clef + 1 eighth flag = 2 paths
        let path_count = svg.matches("<path ").count();
        assert_eq!(
            path_count, 2,
            "should have TAB clef + eighth flag = 2 paths, got {path_count}"
        );
    }

    #[test]
    fn duration_consumed_per_event() {
        // .duration() only applies to the next flushed event, not subsequent ones
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 0) // gets quarter duration
            .next()
            .fret(1, 2) // no duration (was consumed)
            .end_barline()
            .render_svg();
        // Only 1 rhythm stem (for the quarter)
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 9,
            "only first event should have rhythm stem: 6 staff + 2 barline + 1 stem = 9, got {line_count}"
        );
    }

    #[test]
    fn try_render_svg_returns_ok() {
        let result = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .try_render_svg();
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn different_strings_produce_different_positions() {
        let svg1 = TabScoreBuilder::guitar()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        let svg6 = TabScoreBuilder::guitar()
            .fret(6, 5)
            .end_barline()
            .render_svg();
        assert_ne!(
            svg1, svg6,
            "fret 5 on string 1 vs string 6 should have different y positions"
        );
    }

    // ---- Beam group tests ----

    #[test]
    fn beam_group_two_eighths_has_polygon() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // Beam groups use polygons instead of flag paths
        assert!(
            svg.contains("<polygon "),
            "beam group should produce beam polygon"
        );
        // 2 fret numbers
        assert_eq!(svg.matches(">0</text>").count(), 1, "should show fret 0");
        assert_eq!(svg.matches(">2</text>").count(), 1, "should show fret 2");
    }

    #[test]
    fn beam_group_differs_from_individual_eighths() {
        let beamed = TabScoreBuilder::guitar()
            .beam_start()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        let individual = TabScoreBuilder::guitar()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .end_barline()
            .render_svg();
        assert_ne!(
            beamed, individual,
            "beamed should differ from individual flagged eighths"
        );
        // Individual has flag paths; beamed has polygons
        assert!(beamed.contains("<polygon "), "beamed uses polygons");
        assert!(
            !individual.contains("<polygon "),
            "individual uses flags, not polygons"
        );
    }

    #[test]
    fn beam_group_four_sixteenths_has_two_polygons() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .duration(4)
            .fret(1, 0)
            .next()
            .duration(4)
            .fret(1, 2)
            .next()
            .duration(4)
            .fret(1, 3)
            .next()
            .duration(4)
            .fret(1, 5)
            .beam_end()
            .end_barline()
            .render_svg();
        let polygon_count = svg.matches("<polygon ").count();
        assert_eq!(
            polygon_count, 2,
            "4 sixteenths should have 2 beam polygons (primary + secondary), got {polygon_count}"
        );
        // 4 fret numbers
        assert_eq!(
            svg.matches("<text ").count(),
            4,
            "should have 4 fret numbers"
        );
    }

    #[test]
    fn beam_group_has_stems_as_lines() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .next()
            .eighth()
            .fret(1, 3)
            .beam_end()
            .end_barline()
            .render_svg();
        let line_count = svg.matches("<line ").count();
        // 6 staff lines + 2 barline (Final) + 3 beam stems = 11
        assert_eq!(
            line_count, 11,
            "3-note beam group: 6 staff + 2 barline + 3 stems = 11, got {line_count}"
        );
    }

    #[test]
    fn beam_group_with_chord() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .eighth()
            .fret(1, 0)
            .fret(2, 1) // chord: two strings on same beat
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // 3 fret numbers (0, 1, 2)
        assert_eq!(
            svg.matches("<text ").count(),
            3,
            "should have 3 fret numbers"
        );
        assert!(svg.contains("<polygon "), "should have beam polygon");
    }

    #[test]
    fn beam_group_default_duration_is_eighth() {
        // If no duration is set in beam group, defaults to eighth
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .fret(1, 0)
            .next()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // Should still produce a beam (default eighth = beamable)
        assert!(
            svg.contains("<polygon "),
            "beam group with default duration should produce beam polygon"
        );
    }

    #[test]
    fn beam_group_no_flag_paths() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // Only 1 path: TAB clef. No flag paths since beams replace flags.
        let path_count = svg.matches("<path ").count();
        assert_eq!(
            path_count, 1,
            "beam group should have only TAB clef path (no flags), got {path_count}"
        );
    }

    // ---- Slide tests ----

    #[test]
    fn slide_adds_line_between_frets() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .slide()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // 6 staff lines + 2 barline (Final) + 1 slide line = 9
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 9,
            "slide should add 1 extra line: 6 staff + 2 barline + 1 slide = 9, got {line_count}"
        );
    }

    #[test]
    fn no_slide_without_slide_call() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // 6 staff lines + 2 barline = 8 (no slide line)
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 8,
            "without slide: 6 staff + 2 barline = 8, got {line_count}"
        );
    }

    #[test]
    fn slide_differs_from_no_slide() {
        let with_slide = TabScoreBuilder::guitar()
            .fret(1, 5)
            .slide()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        let without_slide = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        assert_ne!(with_slide, without_slide, "slide should change SVG output");
    }

    #[test]
    fn slide_on_chord_draws_lines_for_matching_strings() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .fret(2, 5)
            .slide()
            .next()
            .fret(1, 7)
            .fret(2, 7)
            .end_barline()
            .render_svg();
        // 6 staff + 2 barline + 2 slide lines (one per matching string) = 10
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 10,
            "chord slide on 2 strings: 6 staff + 2 barline + 2 slides = 10, got {line_count}"
        );
    }

    #[test]
    fn slide_only_on_matching_strings() {
        // Source has strings 1,2; target only has string 1 → only 1 slide
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .fret(2, 5)
            .slide()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // 6 staff + 2 barline + 1 slide (string 1 only) = 9
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 9,
            "slide only on matching string: 6+2+1=9, got {line_count}"
        );
    }

    #[test]
    fn multiple_consecutive_slides() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 3)
            .slide()
            .next()
            .fret(1, 5)
            .slide()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // 6 staff + 2 barline + 2 slide lines = 10
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 10,
            "two consecutive slides: 6+2+2=10, got {line_count}"
        );
    }

    #[test]
    fn slide_on_rest_target_skips_to_next_fret() {
        // Slide should skip rest and find the next fret event
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .slide()
            .next()
            .rest()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // Should still draw the slide line (skipping the rest)
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 9,
            "slide should skip rest target: 6+2+1=9, got {line_count}"
        );
    }

    #[test]
    fn slide_after_already_flushed_event() {
        // Call .slide() after .next() — should mark the already-flushed event
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .slide()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // The .slide() after .next() marks the last flushed event
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 9,
            "slide after flush: 6+2+1=9, got {line_count}"
        );
    }

    #[test]
    fn beam_group_mixed_with_non_beamed() {
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 5)
            .next() // individual quarter
            .beam_start()
            .eighth()
            .fret(1, 0)
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // Should have: 1 quarter stem + 2 beam stems = 3 rhythm stems
        // Plus 6 staff lines + 2 barline = 8. Total: 11
        let line_count = svg.matches("<line ").count();
        assert_eq!(
            line_count, 11,
            "quarter + 2-note beam: 6 staff + 2 barline + 3 stems = 11, got {line_count}"
        );
        // 1 beam polygon + 1 TAB clef path = 2 SVG elements with polygon/path
        assert_eq!(svg.matches("<polygon ").count(), 1, "1 beam polygon");
        assert_eq!(
            svg.matches("<path ").count(),
            1,
            "1 TAB clef path (no flags on beamed)"
        );
    }

    // --- Hammer-on / Pull-off tests ---

    #[test]
    fn hammer_on_adds_arc_path_and_h_label() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .hammer()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // 1 TAB clef path + 1 arc path = 2 paths
        assert_eq!(
            svg.matches("<path ").count(),
            2,
            "should have TAB clef + 1 hammer arc path"
        );
        assert!(svg.contains(">H</text>"), "should show 'H' label");
    }

    #[test]
    fn pull_off_adds_arc_path_and_p_label() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .pull()
            .next()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        assert_eq!(
            svg.matches("<path ").count(),
            2,
            "should have TAB clef + 1 pull-off arc path"
        );
        assert!(svg.contains(">P</text>"), "should show 'P' label");
    }

    #[test]
    fn no_legato_without_method_call() {
        let svg_no = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        assert_eq!(
            svg_no.matches("<path ").count(),
            1,
            "without hammer/pull, only TAB clef path"
        );
        assert!(!svg_no.contains(">H</text>"));
        assert!(!svg_no.contains(">P</text>"));
    }

    #[test]
    fn hammer_differs_from_pull() {
        let svg_h = TabScoreBuilder::guitar()
            .fret(1, 5)
            .hammer()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        let svg_p = TabScoreBuilder::guitar()
            .fret(1, 5)
            .pull()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        assert_ne!(svg_h, svg_p, "hammer and pull should produce different SVG");
    }

    #[test]
    fn hammer_on_arc_is_unfilled() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .hammer()
            .next()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("fill=\"none\""),
            "arc should be stroke-only (unfilled)"
        );
    }

    #[test]
    fn consecutive_hammer_pull_chain() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .hammer()
            .next()
            .fret(1, 7)
            .pull()
            .next()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2 arcs = 3 paths
        assert_eq!(
            svg.matches("<path ").count(),
            3,
            "chain of hammer + pull should produce 2 arc paths + 1 TAB clef"
        );
        assert!(svg.contains(">H</text>"), "should show H");
        assert!(svg.contains(">P</text>"), "should show P");
    }

    #[test]
    fn hammer_on_rest_skips_no_target() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .hammer()
            .next()
            .rest()
            .end_barline()
            .render_svg();
        // No arc because the next event is a rest, not a fret
        assert_eq!(
            svg.matches("<path ").count(),
            1,
            "hammer before rest should not produce an arc"
        );
    }

    #[test]
    fn legato_with_chord_draws_arc_per_matching_string() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .fret(2, 5)
            .hammer()
            .next()
            .fret(1, 7)
            .fret(2, 7)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2 arcs (one per string) = 3 paths
        assert_eq!(
            svg.matches("<path ").count(),
            3,
            "chord hammer should produce 1 arc per matching string + TAB clef"
        );
    }

    // ── vibrato ───────────────────────────────────────────────

    #[test]
    fn vibrato_adds_wavy_path() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .vibrato()
            .end_barline()
            .render_svg();
        // Vibrato is a stroke path with Q commands
        assert!(
            svg.contains(" Q"),
            "vibrato should produce quadratic Bézier path"
        );
        assert!(
            svg.contains("fill=\"none\""),
            "vibrato path should have no fill"
        );
    }

    #[test]
    fn wide_vibrato_differs_from_normal() {
        let normal = TabScoreBuilder::guitar()
            .fret(1, 5)
            .vibrato()
            .end_barline()
            .render_svg();
        let wide = TabScoreBuilder::guitar()
            .fret(1, 5)
            .wide_vibrato()
            .end_barline()
            .render_svg();
        assert_ne!(normal, wide, "wide vibrato should differ from normal");
    }

    #[test]
    fn vibrato_on_rest_is_noop() {
        let with_vib = TabScoreBuilder::guitar().rest().end_barline().render_svg();
        // rest() flushes frets first — calling vibrato after rest should be no-op
        // since there's no Fret event to modify
        let without = TabScoreBuilder::guitar().rest().end_barline().render_svg();
        assert_eq!(with_vib, without);
    }

    #[test]
    fn vibrato_without_call_produces_no_wave() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // Should not contain vibrato wavy line (no Q commands from wave)
        // Note: Q could appear from other paths, but without vibrato
        // the path count should differ
        let with = TabScoreBuilder::guitar()
            .fret(1, 5)
            .vibrato()
            .end_barline()
            .render_svg();
        assert_ne!(svg, with, "vibrato should change the output");
    }

    #[test]
    fn vibrato_on_chord_draws_per_string() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .fret(2, 7)
            .vibrato()
            .end_barline()
            .render_svg();
        // Should have 2 vibrato paths (one per string)
        let wave_count = svg.matches("fill=\"none\" stroke=\"black\"").count();
        assert!(
            wave_count >= 2,
            "chord vibrato should draw one wave per string, got {}",
            wave_count
        );
    }

    // ── harmonic ──────────────────────────────────────────────

    #[test]
    fn harmonic_adds_extra_path() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 12)
            .harmonic()
            .end_barline()
            .render_svg();
        // 1 TAB clef + 1 harmonic glyph = 2 paths
        assert_eq!(
            svg.matches("<path ").count(),
            2,
            "should have TAB clef + harmonic indicator = 2 paths"
        );
    }

    #[test]
    fn no_harmonic_without_method_call() {
        let svg_with = TabScoreBuilder::guitar()
            .fret(1, 12)
            .harmonic()
            .end_barline()
            .render_svg();
        let svg_without = TabScoreBuilder::guitar()
            .fret(1, 12)
            .end_barline()
            .render_svg();
        assert_ne!(svg_with, svg_without, "harmonic should change the output");
    }

    #[test]
    fn harmonic_on_chord_draws_per_string() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 12)
            .fret(2, 12)
            .harmonic()
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2 harmonic glyphs = 3 paths
        assert_eq!(
            svg.matches("<path ").count(),
            3,
            "chord harmonic: 1 TAB + 2 indicators = 3 paths"
        );
    }

    #[test]
    fn harmonic_has_scale_transform() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .harmonic()
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("scale(0.6)"),
            "harmonic glyph should be scaled down"
        );
    }

    #[test]
    fn harmonic_after_flush_marks_last_event() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 12)
            .next()
            .harmonic()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // .harmonic() after .next() marks the already-flushed event
        // 1 TAB clef + 1 harmonic = 2 paths
        assert_eq!(
            svg.matches("<path ").count(),
            2,
            "harmonic after flush should mark the previous event"
        );
    }

    #[test]
    fn harmonic_with_vibrato_both_rendered() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 12)
            .harmonic()
            .vibrato()
            .end_barline()
            .render_svg();
        // Should have both harmonic glyph (filled) and vibrato wave (unfilled)
        assert!(svg.contains("scale(0.6)"), "harmonic present");
        assert!(svg.contains("fill=\"none\""), "vibrato wave present");
    }

    // --- Palm mute tests ---

    #[test]
    fn palm_mute_adds_pm_text() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .palm_mute()
            .end_barline()
            .render_svg();
        assert!(svg.contains("P.M."), "palm mute should add P.M. text");
    }

    #[test]
    fn no_palm_mute_without_method_call() {
        let svg_with = TabScoreBuilder::guitar()
            .fret(1, 0)
            .palm_mute()
            .end_barline()
            .render_svg();
        let svg_without = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        assert_ne!(svg_with, svg_without, "palm mute should change the output");
    }

    #[test]
    fn palm_mute_text_is_italic() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .palm_mute()
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("font-style=\"italic\""),
            "P.M. text should be italic"
        );
    }

    #[test]
    fn consecutive_palm_mutes_produce_dashed_line() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .palm_mute()
            .next()
            .fret(1, 0)
            .palm_mute()
            .next()
            .fret(1, 0)
            .palm_mute()
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("stroke-dasharray"),
            "consecutive palm mutes should produce a dashed continuation line"
        );
    }

    #[test]
    fn single_palm_mute_no_dashed_line() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .palm_mute()
            .next()
            .fret(1, 3)
            .end_barline()
            .render_svg();
        assert!(
            !svg.contains("stroke-dasharray"),
            "single palm mute event should not produce a dashed line"
        );
    }

    #[test]
    fn palm_mute_after_flush_marks_last_event() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .palm_mute()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // .palm_mute() after .next() marks the already-flushed event
        assert!(
            svg.contains("P.M."),
            "palm_mute after flush should mark the previous event"
        );
    }

    #[test]
    fn let_ring_adds_text() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .let_ring()
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("let ring"),
            "let ring should add 'let ring' text"
        );
    }

    #[test]
    fn no_let_ring_without_method_call() {
        let svg_with = TabScoreBuilder::guitar()
            .fret(1, 0)
            .let_ring()
            .end_barline()
            .render_svg();
        let svg_without = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        assert_ne!(svg_with, svg_without, "let ring should change the output");
    }

    #[test]
    fn let_ring_text_is_italic() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .let_ring()
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("font-style=\"italic\""),
            "let ring text should be italic"
        );
    }

    #[test]
    fn consecutive_let_rings_produce_dashed_line() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .let_ring()
            .next()
            .fret(1, 3)
            .let_ring()
            .next()
            .fret(1, 5)
            .let_ring()
            .end_barline()
            .render_svg();
        // Dashed continuation line from consecutive let ring events
        let dash_count = svg.matches("stroke-dasharray").count();
        assert!(
            dash_count >= 1,
            "consecutive let ring events should produce dashed continuation line(s), found {dash_count}"
        );
    }

    #[test]
    fn single_let_ring_no_dashed_line() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .let_ring()
            .next()
            .fret(1, 3)
            .end_barline()
            .render_svg();
        // Only one let ring event — no dashed line
        assert!(
            !svg.contains("stroke-dasharray"),
            "single let ring event should not produce a dashed line"
        );
    }

    #[test]
    fn let_ring_after_flush_marks_last_event() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5)
            .next()
            .let_ring()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        assert!(
            svg.contains("let ring"),
            "let_ring after flush should mark the previous event"
        );
    }

    #[test]
    fn mute_renders_x_text() {
        let svg = TabScoreBuilder::guitar()
            .mute(1)
            .mute(2)
            .mute(3)
            .end_barline()
            .render_svg();
        // Three muted strings should produce 3 "x" text elements
        let x_count = svg.matches(">x</text>").count();
        assert_eq!(x_count, 3, "3 muted strings should produce 3 'x' texts");
    }

    #[test]
    fn mute_mixed_with_frets() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .mute(5)
            .mute(6)
            .end_barline()
            .render_svg();
        // Should have fret "0" and two "x" texts
        assert!(svg.contains(">0</text>"), "should render fret number 0");
        let x_count = svg.matches(">x</text>").count();
        assert_eq!(x_count, 2, "2 muted strings should produce 2 'x' texts");
    }

    #[test]
    fn mute_only_no_frets() {
        let svg = TabScoreBuilder::guitar()
            .mute(1)
            .mute(2)
            .mute(3)
            .mute(4)
            .mute(5)
            .mute(6)
            .end_barline()
            .render_svg();
        let x_count = svg.matches(">x</text>").count();
        assert_eq!(x_count, 6, "all-mute event should show 6 x markers");
    }

    #[test]
    fn mute_differs_from_no_mute() {
        let base = TabScoreBuilder::guitar()
            .fret(1, 0)
            .end_barline()
            .render_svg();
        let with_mute = TabScoreBuilder::guitar()
            .fret(1, 0)
            .mute(6)
            .end_barline()
            .render_svg();
        assert_ne!(base, with_mute, "muted string should change SVG output");
    }

    #[test]
    fn mute_with_rhythm_stem() {
        let svg = TabScoreBuilder::guitar()
            .quarter()
            .mute(1)
            .mute(2)
            .end_barline()
            .render_svg();
        // Should have 2 "x" texts and a rhythm stem line
        let x_count = svg.matches(">x</text>").count();
        assert_eq!(x_count, 2);
        // Rhythm stem produces at least one <line element beyond staff lines
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count > 6,
            "should have staff lines + rhythm stem, got {line_count}"
        );
    }

    #[test]
    fn mute_across_beats_independent() {
        let svg = TabScoreBuilder::guitar()
            .mute(1)
            .next()
            .mute(6)
            .end_barline()
            .render_svg();
        let x_count = svg.matches(">x</text>").count();
        assert_eq!(x_count, 2, "two separate beats with muted strings");
    }

    #[cfg(feature = "png")]
    mod png_tests {
        use super::*;
        use crate::render::png::test_helpers::{
            count_dense_rows, count_inked_pixels, decode_pixmap, inked_bbox, png_dimensions,
            INK_ALPHA_THRESHOLD,
        };

        #[test]
        fn tab_render_png_produces_valid_png() {
            let png = TabScoreBuilder::guitar()
                .fret(1, 0)
                .fret(2, 1)
                .barline()
                .fret(3, 2)
                .end_barline()
                .render_png(1.0);
            // PNG magic bytes
            assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
            let (w, h) = png_dimensions(&png);
            assert!(w > 50, "width should be reasonable, got {w}");
            assert!(h > 20, "height should be reasonable, got {h}");
        }

        #[test]
        fn tab_render_png_2x_is_larger_than_1x() {
            let builder = || {
                TabScoreBuilder::guitar()
                    .fret(1, 5)
                    .fret(2, 7)
                    .end_barline()
            };
            let png_1x = builder().render_png(1.0);
            let png_2x = builder().render_png(2.0);
            let (w1, h1) = png_dimensions(&png_1x);
            let (w2, h2) = png_dimensions(&png_2x);
            assert!(w2 > w1, "2x width ({w2}) should be larger than 1x ({w1})");
            assert!(h2 > h1, "2x height ({h2}) should be larger than 1x ({h1})");
        }

        #[test]
        fn tab_try_render_png_matches_render_png() {
            let builder = || TabScoreBuilder::guitar().fret(1, 3).end_barline();
            let try_result = builder().try_render_png(1.0).expect("should succeed");
            let direct = builder().render_png(1.0);
            assert_eq!(try_result.len(), direct.len());
        }

        #[test]
        fn tab_render_png_nonzero_for_complex_score() {
            let png = TabScoreBuilder::guitar()
                .quarter()
                .fret(1, 0)
                .fret(2, 2)
                .fret(3, 2)
                .next()
                .eighth()
                .fret(1, 3)
                .next()
                .fret(1, 5)
                .barline()
                .rest()
                .next()
                .fret(6, 0)
                .end_barline()
                .render_png(1.5);
            assert!(!png.is_empty());
            let (w, h) = png_dimensions(&png);
            assert!(
                w > 100,
                "complex tab PNG width should be substantial, got {w}"
            );
            assert!(
                h > 30,
                "complex tab PNG height should be substantial, got {h}"
            );
        }

        // -- Pixel-content verification --
        //
        // Tab staves have 6 strings (vs 5 staff lines for standard notation),
        // a TAB clef glyph (vs the much smaller G/F clef), and fret numbers
        // rendered as text (vs notehead glyphs). A regression like dropping
        // a string line, rendering the TAB clef as a wrong glyph, or losing
        // text fret numbers would slip past magic-byte and dimension checks
        // but show up here as ink-density or layout anomalies.

        /// A small but content-rich tab score: 6 chord notes across two
        /// measures, both barlines, fret numbers on multiple strings.
        fn rich_tab_score() -> TabScoreBuilder {
            TabScoreBuilder::guitar()
                .quarter()
                .fret(1, 0)
                .fret(2, 2)
                .fret(3, 2)
                .next()
                .fret(1, 3)
                .next()
                .fret(1, 5)
                .barline()
                .fret(6, 0)
                .next()
                .fret(5, 2)
                .next()
                .fret(4, 0)
                .end_barline()
        }

        /// A near-empty tab score: TAB clef + end barline only. No fret
        /// numbers; the only ink is from the staff lines and clef glyph.
        fn sparse_tab_score() -> TabScoreBuilder {
            TabScoreBuilder::guitar().end_barline()
        }

        #[test]
        fn tab_png_has_substantial_ink() {
            let png = rich_tab_score().render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let ink = count_inked_pixels(&pixmap, INK_ALPHA_THRESHOLD);
            // 6 strings × full-width staff lines alone yield several hundred
            // inked pixels; clef + fret numbers + barlines push well beyond.
            assert!(
                ink >= 400,
                "tab PNG has only {ink} inked pixels; expected >= 400 for \
                 6 strings + TAB clef + fret numbers + barlines"
            );
        }

        #[test]
        fn tab_png_has_at_least_six_dense_horizontal_bands() {
            // Standard guitar tab has 6 strings drawn as 6 horizontal lines
            // spanning the full measure width. Each contributes at least one
            // row above the 50% density threshold (AA may widen each to ~2
            // rows). Catches a regression where a string line is dropped or
            // rendered as dashed strokes.
            let png = rich_tab_score().render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let dense = count_dense_rows(&pixmap, INK_ALPHA_THRESHOLD, 0.5);
            assert!(
                dense >= 6,
                "tab has only {dense} dense rows; expected >= 6 (six strings)"
            );
        }

        #[test]
        fn tab_png_has_more_dense_rows_than_standard_staff() {
            // 6-string tab should produce more dense horizontal bands than a
            // 5-line standard staff (sanity vs the previous test, but with a
            // built-in baseline comparison — catches a regression that
            // accidentally collapsed the tab staff to 5 strings).
            use crate::score::ScoreBuilder as NotationScoreBuilder;
            use music::notation::clef::Clef;
            use music::notation::rhythm::duration::Duration;
            use music::note::note::Note;
            use music::note::pitch::Pitch;

            let standard_png = NotationScoreBuilder::new()
                .clef(Clef::Treble)
                .note(Pitch::new(Note::C, 4), Duration::QTR)
                .end_barline()
                .render_png(1.0);
            let tab_png = rich_tab_score().render_png(1.0);
            let standard_dense =
                count_dense_rows(&decode_pixmap(&standard_png), INK_ALPHA_THRESHOLD, 0.5);
            let tab_dense = count_dense_rows(&decode_pixmap(&tab_png), INK_ALPHA_THRESHOLD, 0.5);
            assert!(
                tab_dense > standard_dense,
                "tab dense rows ({tab_dense}) should exceed standard staff ({standard_dense})"
            );
        }

        #[test]
        fn tab_png_with_fret_numbers_has_more_ink_than_sparse_tab() {
            // Fret-number text is drawn into the same vertical region as the
            // staff lines; counting total ink is the cleanest way to verify
            // the text actually rendered. If a font-loading failure ever
            // rendered fret digits as invisible, the sparse vs rich
            // difference would shrink to zero.
            let sparse_png = sparse_tab_score().render_png(1.0);
            let rich_png = rich_tab_score().render_png(1.0);
            let sparse_ink = count_inked_pixels(&decode_pixmap(&sparse_png), INK_ALPHA_THRESHOLD);
            let rich_ink = count_inked_pixels(&decode_pixmap(&rich_png), INK_ALPHA_THRESHOLD);
            assert!(
                rich_ink > sparse_ink,
                "tab with fret numbers ({rich_ink}) should have more ink than \
                 empty tab ({sparse_ink})"
            );
            // And the difference should be substantial — at least 100 extra
            // inked pixels for 6 fret digits + extra barline.
            assert!(
                rich_ink - sparse_ink >= 100,
                "fret-number ink contribution is only {} pixels; expected >= 100",
                rich_ink - sparse_ink
            );
        }

        #[test]
        fn tab_png_ink_bbox_spans_most_of_width() {
            // TAB clef sits on the left, end barline on the right; inked
            // content must span most of the image width. Catches a bug
            // where (e.g.) all content rendered into a single column or
            // the clef was clipped off-canvas.
            let png = rich_tab_score().render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let bbox = inked_bbox(&pixmap, INK_ALPHA_THRESHOLD).expect("tab PNG should have ink");
            let bbox_width = bbox.2 - bbox.0;
            let img_w = pixmap.width();
            let span_fraction = bbox_width as f64 / img_w as f64;
            assert!(
                span_fraction > 0.5,
                "tab ink bbox width {bbox_width} is only {:.1}% of image width {img_w}; \
                 expected staff to span >50%",
                100.0 * span_fraction
            );
        }

        #[test]
        fn tab_png_is_mostly_transparent_background() {
            // The tab staff occupies a horizontal band in the middle of a
            // taller page. Background dominance regression canary (mirrors
            // the standard-notation test in render::png::tests).
            let png = rich_tab_score().render_png(1.0);
            let pixmap = decode_pixmap(&png);
            let total = (pixmap.width() * pixmap.height()) as usize;
            let transparent = pixmap.pixels().iter().filter(|p| p.alpha() == 0).count();
            let transparent_fraction = transparent as f64 / total as f64;
            assert!(
                transparent_fraction > 0.5,
                "expected tab background to dominate, but only {transparent}/{total} \
                 ({:.1}%) pixels are fully transparent",
                100.0 * transparent_fraction
            );
        }

        #[test]
        fn tab_png_2x_scale_increases_ink() {
            // Verifies the scale parameter is wired through the tab-specific
            // PNG path (it builds its own PngRenderer inside `try_render_png`).
            let png_1x = rich_tab_score().render_png(1.0);
            let png_2x = rich_tab_score().render_png(2.0);
            let ink_1x = count_inked_pixels(&decode_pixmap(&png_1x), INK_ALPHA_THRESHOLD);
            let ink_2x = count_inked_pixels(&decode_pixmap(&png_2x), INK_ALPHA_THRESHOLD);
            assert!(
                ink_2x > ink_1x * 2,
                "tab 2× should at least double ink (1×={ink_1x}, 2×={ink_2x})"
            );
        }
    }
}
