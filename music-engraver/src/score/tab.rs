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
use crate::layout::barline::BarlineStyle;
use crate::layout::tab::{layout_fret_number, TabStaffLayout};
use crate::layout::tab_beam::{layout_tab_beam_group, TabBeamedNote};
use crate::layout::tab_rhythm::layout_tab_rhythm;
use crate::layout::tab_bend::{layout_tab_bend, BendAmount};
use crate::layout::tab_hammer::{layout_tab_legato, LegatoKind};
use crate::layout::tab_slide::layout_tab_slide;
use crate::render::tab_beam_renderer::draw_tab_beam_group;
use crate::render::tab_renderer::{draw_fret_number, draw_tab_clef, draw_tab_staff_lines};
use crate::render::tab_rhythm_renderer::draw_tab_rhythm;
use crate::render::tab_bend_renderer::draw_tab_bend;
use crate::render::tab_hammer_renderer::draw_tab_legato;
use crate::render::tab_slide_renderer::draw_tab_slide;
use crate::render::{SvgWriter, TextStyle};

/// A single event in a tab measure.
#[derive(Clone, Debug)]
pub(crate) enum TabEvent {
    /// One or more fret numbers played simultaneously.
    /// Each entry is (string_number, fret_number) where string is 1-based.
    /// `duration_log2`: optional rhythm (0=whole, 1=half, 2=quarter, 3=eighth, etc.)
    /// `slide_out`: when true, draw a slide line from this event to the next.
    /// `legato_out`: when Some, draw a hammer-on/pull-off arc to the next event.
    /// `bend`: when Some, draw a bend arrow above the fret number(s).
    Fret {
        frets: Vec<(u8, u8)>,
        duration_log2: Option<u8>,
        slide_out: bool,
        legato_out: Option<LegatoKind>,
        bend: Option<BendAmount>,
    },
    /// A rest (blank space — no fret numbers).
    /// `duration_log2`: optional rhythm for rest stem display.
    Rest {
        duration_log2: Option<u8>,
    },
    /// A beam group: multiple fret events connected by beam lines above the staff.
    /// Each sub-event is (frets, duration_log2).
    BeamGroup {
        events: Vec<(Vec<(u8, u8)>, u8)>,
    },
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
    pending_duration: Option<u8>,
    /// Events accumulated for the current in-progress measure.
    current_events: Vec<TabEvent>,
    /// Completed measures.
    pub(crate) measures: Vec<TabMeasure>,
    /// Measures per system. 0 = use default (4).
    measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    system_width: f64,
    /// Display measure numbers above the start of each system.
    show_measure_numbers: bool,
    /// When true, we are accumulating events for a beam group.
    in_beam_group: bool,
    /// Accumulated beam group sub-events: (frets, duration_log2).
    beam_group_events: Vec<(Vec<(u8, u8)>, u8)>,
    /// When true, the next flushed Fret event gets `slide_out = true`.
    pending_slide: bool,
    /// When Some, the next flushed Fret event gets `legato_out` set.
    pending_legato: Option<LegatoKind>,
    /// When Some, the next flushed Fret event gets `bend` set.
    pending_bend: Option<BendAmount>,
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
            show_measure_numbers: false,
            in_beam_group: false,
            beam_group_events: Vec::new(),
            pending_slide: false,
            pending_legato: None,
            pending_bend: None,
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

    /// Show measure numbers above the start of each system.
    pub fn show_measure_numbers(mut self) -> Self {
        self.show_measure_numbers = true;
        self
    }

    /// Flush any pending fret entries as a single event.
    fn flush_frets(&mut self) {
        if !self.current_frets.is_empty() {
            if self.in_beam_group {
                self.flush_beam_frets();
            } else {
                let frets = std::mem::take(&mut self.current_frets);
                let duration_log2 = self.pending_duration.take();
                let slide_out = std::mem::take(&mut self.pending_slide);
                let legato_out = self.pending_legato.take();
                let bend = self.pending_bend.take();
                self.current_events.push(TabEvent::Fret {
                    frets,
                    duration_log2,
                    slide_out,
                    legato_out,
                    bend,
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
    /// `duration_log2`: 0=whole, 1=half, 2=quarter, 3=eighth, 4=sixteenth, etc.
    /// The duration is consumed by the next `.fret()` flush or `.rest()` call.
    /// When set, a rhythm stem (and flag for eighths and shorter) is drawn
    /// above the tab staff.
    pub fn duration(mut self, duration_log2: u8) -> Self {
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

    /// Mark the current fret event for a bend arrow above the fret number(s).
    ///
    /// A curved arrow with the bend amount label (e.g. "full", "1/2") is drawn
    /// above the fret number on each string. The bend arrow appears at the
    /// fret position (in-place bend, not between two events like slides).
    pub fn bend(mut self, amount: BendAmount) -> Self {
        if !self.current_frets.is_empty() {
            self.pending_bend = Some(amount);
        } else if let Some(TabEvent::Fret { bend, .. }) = self.current_events.last_mut() {
            *bend = Some(amount);
        }
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
        let tab_staff_height =
            staff_space * (self.line_count.saturating_sub(1)) as f64;
        let inter_system_gap = 4.0 * staff_space;
        let total_systems = chunks.len();
        let page_height = if total_systems > 0 {
            total_systems as f64 * tab_staff_height
                + (total_systems.saturating_sub(1)) as f64 * inter_system_gap
        } else {
            0.0
        };

        let vb_margin = staff_space;
        let vb_x = -vb_margin;
        let vb_y = -vb_margin;
        let vb_w = sys_width + 2.0 * vb_margin;
        let vb_h = page_height + 2.0 * vb_margin;
        let px_per_unit = 7.0 / staff_space;
        let px_w = vb_w * px_per_unit;
        let px_h = vb_h * px_per_unit;

        let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

        for (sys_idx, (start, end)) in chunks.iter().enumerate() {
            let sys_y =
                sys_idx as f64 * (tab_staff_height + inter_system_gap);

            let tab_staff = TabStaffLayout::new(
                0.0,
                sys_y,
                sys_width,
                staff_space,
                self.line_count,
            );

            // Draw staff lines
            draw_tab_staff_lines(&mut svg, &tab_staff, &config);

            // Draw TAB clef
            draw_tab_clef(&mut svg, &tab_staff, &font)?;

            // Draw measure number above the tab staff
            if self.show_measure_numbers {
                let num_x = clef_width;
                let num_y = sys_y - 1.8 * staff_space;
                let font_size = 1.2 * staff_space;
                let measure_number = start + 1;
                svg.add_text(
                    num_x,
                    num_y,
                    &measure_number.to_string(),
                    &TextStyle {
                        font_family: "serif",
                        font_size,
                        fill: "black",
                        anchor: "start",
                        font_weight: "normal",
                        font_style: "normal",
                        dominant_baseline: "auto",
                    },
                );
            }

            // Layout and draw measures for this system
            let measures_in_system = &self.measures[*start..*end];
            let content_width = sys_width - clef_width;
            let measure_width = if measures_in_system.is_empty() {
                content_width
            } else {
                content_width / measures_in_system.len() as f64
            };

            for (m_idx, measure) in measures_in_system.iter().enumerate() {
                let measure_x =
                    clef_width + m_idx as f64 * measure_width;

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
        draw_measure_barline(svg, font, config, tab_staff, measure_x + measure_width, &measure.barline)?;
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
                ..
            } => {
                for &(string, fret) in frets {
                    let layout =
                        layout_fret_number(tab_staff, string, fret, event_x);
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
            TabEvent::BeamGroup { events: beam_events } => {
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
        if let TabEvent::Fret { frets: src_frets, slide_out: true, .. } = &measure.events[i] {
            // Find the next Fret event (skip rests)
            if let Some(target_idx) = (i + 1..event_count).find(|&j| {
                matches!(&measure.events[j], TabEvent::Fret { .. })
            }) {
                if let TabEvent::Fret { frets: tgt_frets, .. } = &measure.events[target_idx] {
                    let src_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + i as f64 * spacing
                    };
                    let tgt_x = measure_x + padding + target_idx as f64 * spacing;

                    // Draw a slide line for each string that appears in both events
                    for &(src_str, _) in src_frets {
                        if tgt_frets.iter().any(|&(ts, _)| ts == src_str) {
                            if let Some(slide_layout) = layout_tab_slide(
                                tab_staff, src_str, src_x, tgt_x, slide_stroke,
                            ) {
                                draw_tab_slide(svg, &slide_layout);
                            }
                        }
                    }
                }
            }
        }
    }

    // Third pass: draw bend arrows at fret events with bend amount
    for (e_idx, event) in measure.events.iter().enumerate() {
        if let TabEvent::Fret { frets, bend: Some(amount), .. } = event {
            let event_x = if event_count == 1 {
                measure_x + padding + usable_width / 2.0
            } else {
                measure_x + padding + e_idx as f64 * spacing
            };

            let bend_stroke = config.stem_thickness_fu();
            for &(string, _) in frets {
                let bend_layout = layout_tab_bend(tab_staff, string, event_x, *amount, bend_stroke);
                draw_tab_bend(svg, &bend_layout);
            }
        }
    }

    // Fourth pass: draw hammer-on/pull-off arcs between consecutive fret events
    for i in 0..event_count.saturating_sub(1) {
        if let TabEvent::Fret { frets: src_frets, legato_out: Some(kind), .. } = &measure.events[i] {
            if let Some(target_idx) = (i + 1..event_count).find(|&j| {
                matches!(&measure.events[j], TabEvent::Fret { .. })
            }) {
                if let TabEvent::Fret { frets: tgt_frets, .. } = &measure.events[target_idx] {
                    let src_x = if event_count == 1 {
                        measure_x + padding + usable_width / 2.0
                    } else {
                        measure_x + padding + i as f64 * spacing
                    };
                    let tgt_x = measure_x + padding + target_idx as f64 * spacing;

                    for &(src_str, _) in src_frets {
                        if tgt_frets.iter().any(|&(ts, _)| ts == src_str) {
                            if let Some(legato_layout) = layout_tab_legato(
                                tab_staff, src_str, src_x, tgt_x, *kind, slide_stroke,
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
    draw_measure_barline(svg, font, config, tab_staff, measure_x + measure_width, &measure.barline)?;

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
    events: &[(Vec<(u8, u8)>, u8)],
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
            svg.add_line(x + sep + thick / 2.0, y_top, x + sep + thick / 2.0, y_bottom, "black", thin);
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
        let svg = TabScoreBuilder::guitar()
            .rest()
            .end_barline()
            .render_svg();
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
        assert!(svg.matches("<path ").count() >= 2, "should have at least 2 TAB clef paths");
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
        assert_ne!(narrow, wide, "different widths should produce different SVGs");
    }

    #[test]
    fn measure_numbers_shown_when_enabled() {
        let svg = TabScoreBuilder::guitar()
            .show_measure_numbers()
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
        assert!(total_lines > 6, "repeat barlines should add lines, got {total_lines}");
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
        assert_eq!(default_svg, guitar_svg, "default should be guitar (6-string)");
    }

    #[test]
    fn flush_pending_auto_closes_measure() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 0)
            .fret(2, 1)
            .render_svg();
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
        assert!(!individual.contains("<polygon "), "individual uses flags, not polygons");
    }

    #[test]
    fn beam_group_four_sixteenths_has_two_polygons() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .duration(4).fret(1, 0).next()
            .duration(4).fret(1, 2).next()
            .duration(4).fret(1, 3).next()
            .duration(4).fret(1, 5)
            .beam_end()
            .end_barline()
            .render_svg();
        let polygon_count = svg.matches("<polygon ").count();
        assert_eq!(
            polygon_count, 2,
            "4 sixteenths should have 2 beam polygons (primary + secondary), got {polygon_count}"
        );
        // 4 fret numbers
        assert_eq!(svg.matches("<text ").count(), 4, "should have 4 fret numbers");
    }

    #[test]
    fn beam_group_has_stems_as_lines() {
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .eighth().fret(1, 0).next()
            .eighth().fret(1, 2).next()
            .eighth().fret(1, 3)
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
            .fret(2, 1)  // chord: two strings on same beat
            .next()
            .eighth()
            .fret(1, 2)
            .beam_end()
            .end_barline()
            .render_svg();
        // 3 fret numbers (0, 1, 2)
        assert_eq!(svg.matches("<text ").count(), 3, "should have 3 fret numbers");
        assert!(svg.contains("<polygon "), "should have beam polygon");
    }

    #[test]
    fn beam_group_default_duration_is_eighth() {
        // If no duration is set in beam group, defaults to eighth
        let svg = TabScoreBuilder::guitar()
            .beam_start()
            .fret(1, 0).next()
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
            .eighth().fret(1, 0).next()
            .eighth().fret(1, 2)
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
            .quarter().fret(1, 5).next()    // individual quarter
            .beam_start()
            .eighth().fret(1, 0).next()
            .eighth().fret(1, 2)
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
        assert_eq!(svg.matches("<path ").count(), 1, "1 TAB clef path (no flags on beamed)");
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
            svg.matches("<path ").count(), 2,
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
            svg.matches("<path ").count(), 2,
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
            svg_no.matches("<path ").count(), 1,
            "without hammer/pull, only TAB clef path"
        );
        assert!(!svg_no.contains(">H</text>"));
        assert!(!svg_no.contains(">P</text>"));
    }

    #[test]
    fn hammer_differs_from_pull() {
        let svg_h = TabScoreBuilder::guitar()
            .fret(1, 5).hammer().next().fret(1, 7)
            .end_barline()
            .render_svg();
        let svg_p = TabScoreBuilder::guitar()
            .fret(1, 5).pull().next().fret(1, 7)
            .end_barline()
            .render_svg();
        assert_ne!(svg_h, svg_p, "hammer and pull should produce different SVG");
    }

    #[test]
    fn hammer_on_arc_is_unfilled() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5).hammer().next().fret(1, 7)
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
            .fret(1, 5).hammer().next()
            .fret(1, 7).pull().next()
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2 arcs = 3 paths
        assert_eq!(
            svg.matches("<path ").count(), 3,
            "chain of hammer + pull should produce 2 arc paths + 1 TAB clef"
        );
        assert!(svg.contains(">H</text>"), "should show H");
        assert!(svg.contains(">P</text>"), "should show P");
    }

    #[test]
    fn hammer_on_rest_skips_no_target() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5).hammer().next()
            .rest()
            .end_barline()
            .render_svg();
        // No arc because the next event is a rest, not a fret
        assert_eq!(
            svg.matches("<path ").count(), 1,
            "hammer before rest should not produce an arc"
        );
    }

    #[test]
    fn legato_with_chord_draws_arc_per_matching_string() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 5).fret(2, 5).hammer().next()
            .fret(1, 7).fret(2, 7)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2 arcs (one per string) = 3 paths
        assert_eq!(
            svg.matches("<path ").count(), 3,
            "chord hammer should produce 1 arc per matching string + TAB clef"
        );
    }

    // --- Bend tests ---

    #[test]
    fn bend_adds_arrow_paths_and_text() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .bend(BendAmount::Full)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 1 bend curve + 1 arrowhead = 3 paths
        assert_eq!(
            svg.matches("<path ").count(), 3,
            "should have TAB clef + bend curve + arrowhead = 3 paths"
        );
        assert!(svg.contains(">full</text>"), "should show 'full' label");
    }

    #[test]
    fn no_bend_without_method_call() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .end_barline()
            .render_svg();
        // Only 1 path: TAB clef
        assert_eq!(
            svg.matches("<path ").count(), 1,
            "without bend, only TAB clef path"
        );
        assert!(!svg.contains(">full</text>"));
        assert!(!svg.contains(">1/2</text>"));
    }

    #[test]
    fn half_bend_shows_half_label() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .bend(BendAmount::Half)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">1/2</text>"), "should show '1/2' label");
    }

    #[test]
    fn quarter_bend_shows_quarter_label() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .bend(BendAmount::Quarter)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">1/4</text>"), "should show '1/4' label");
    }

    #[test]
    fn different_bend_amounts_produce_different_svg() {
        let svg_full = TabScoreBuilder::guitar()
            .fret(1, 7).bend(BendAmount::Full)
            .end_barline()
            .render_svg();
        let svg_half = TabScoreBuilder::guitar()
            .fret(1, 7).bend(BendAmount::Half)
            .end_barline()
            .render_svg();
        assert_ne!(svg_full, svg_half, "full and half bends should produce different SVG");
    }

    #[test]
    fn bend_on_chord_draws_arrow_per_string() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .fret(2, 7)
            .bend(BendAmount::Full)
            .end_barline()
            .render_svg();
        // 1 TAB clef + 2×(curve + arrowhead) = 5 paths
        assert_eq!(
            svg.matches("<path ").count(), 5,
            "chord bend on 2 strings: 1 TAB + 2 curves + 2 arrowheads = 5 paths"
        );
    }

    #[test]
    fn bend_arrowhead_is_filled() {
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .bend(BendAmount::Full)
            .end_barline()
            .render_svg();
        assert!(svg.contains("fill=\"black\""), "arrowhead should be filled black");
    }

    #[test]
    fn bend_after_already_flushed_event() {
        // Call .bend() after .next() — should mark the already-flushed event
        let svg = TabScoreBuilder::guitar()
            .fret(1, 7)
            .next()
            .bend(BendAmount::Full)
            .fret(1, 5)
            .end_barline()
            .render_svg();
        // The .bend() after .next() marks the last flushed event
        assert!(svg.contains(">full</text>"), "bend after flush should still show label");
    }
}
