//! High-level API for engraving music from `music` crate types.
//!
//! The [`ScoreBuilder`] accepts `Pitch`es and `Duration`s from the `music` crate,
//! converts them into the internal layout representation, and renders to SVG.
//!
//! # Example
//! ```no_run
//! use music::notation::clef::Clef;
//! use music::notation::rhythm::duration::Duration;
//! use music::note::pitch::Pitch;
//! use music::note::note::Note;
//! use music_engraver::layout::key_signature::KeySignature;
//! use music_engraver::score::ScoreBuilder;
//!
//! let svg = ScoreBuilder::new()
//!     .clef(Clef::Treble)
//!     .key_signature(KeySignature::Sharps(2))
//!     .time_signature(4, 4)
//!     .note(Pitch::new(Note::D, 4).expect("valid pitch"), Duration::QTR)
//!     .note(Pitch::new(Note::E, 4).expect("valid pitch"), Duration::QTR)
//!     .note(Pitch::new(Note::Fis, 4).expect("valid pitch"), Duration::QTR)
//!     .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::QTR)
//!     .barline()
//!     .rest(Duration::WHOLE)
//!     .end_barline()
//!     .render_svg();
//! ```

mod event;

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
#[cfg(test)]
use music::notation::rhythm::duration::DurationKind;
use music::note::pitch::Pitch;
#[cfg(test)]
use music::note::spelling::Accidental;

use crate::font::bravura_font;
use crate::layout::barline::BarlineStyle;
use crate::layout::hairpin::HairpinType;
use crate::layout::dynamics::Dynamic;
use crate::layout::key_signature::KeySignature;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::tempo::TempoMark;
use crate::layout::measure::MeasureLayoutConfig;
use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
use crate::layout::system::{ClefKind, MeasureContent, MeasureEvent, SystemPrefix};
use crate::layout::time_signature::TimeSignatureKind;
use crate::render::page_renderer::draw_page;

use event::{AccidentalTracker, ScoreEvent, convert_event};
#[cfg(test)]
use event::{
    duration_kind_to_log2, effective_accidental,
    note_altered_in_key, note_key, resolve_accidental, should_show_accidental,
};

/// Builder for constructing a score from `music` crate types and rendering to SVG.
///
/// Events are grouped into measures delimited by `barline()` / `end_barline()` calls.
/// The builder handles conversion of `Pitch` → staff position, `Duration` → log2 + dots,
/// and accidental resolution against the key signature.
#[derive(Clone, Debug)]
#[must_use = "a ScoreBuilder does nothing until .render_svg() or .try_render_svg() is called"]
pub struct ScoreBuilder {
    clef: ClefKind,
    key_sig: KeySignature,
    time_sig: Option<(u8, u8)>,
    /// Override for time signature display style (Common/CutCommon).
    /// When None and time_sig is Some, uses Numeric display.
    time_sig_kind: Option<TimeSignatureKind>,
    /// Events accumulated for the current (in-progress) measure.
    current_events: Vec<ScoreEvent>,
    /// Completed measures.
    measures: Vec<(Vec<ScoreEvent>, BarlineStyle)>,
    /// Measures per system (for line breaking). 0 = auto (4 per system).
    measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    system_width: f64,
}

impl ScoreBuilder {
    /// Create a new score builder with default settings (treble clef, no key/time signature).
    pub fn new() -> Self {
        Self {
            clef: ClefKind::Treble,
            key_sig: KeySignature::Open,
            time_sig: None,
            time_sig_kind: None,
            current_events: Vec::new(),
            measures: Vec::new(),
            measures_per_system: 4,
            system_width: 0.0,
        }
    }

    /// Set the clef.
    pub fn clef(mut self, clef: Clef) -> Self {
        self.clef = ClefKind::from_clef(&clef);
        self
    }

    /// Set the key signature.
    pub fn key_signature(mut self, key_sig: KeySignature) -> Self {
        self.key_sig = key_sig;
        self
    }

    /// Set the time signature (numerator, denominator).
    pub fn time_signature(mut self, numerator: u8, denominator: u8) -> Self {
        self.time_sig = Some((numerator, denominator));
        self.time_sig_kind = None;
        self
    }

    /// Set the time signature to common time (C symbol = 4/4).
    pub fn common_time(mut self) -> Self {
        self.time_sig = Some((4, 4));
        self.time_sig_kind = Some(TimeSignatureKind::Common);
        self
    }

    /// Set the time signature to cut time / alla breve (₵ symbol = 2/2).
    pub fn cut_time(mut self) -> Self {
        self.time_sig = Some((2, 2));
        self.time_sig_kind = Some(TimeSignatureKind::CutCommon);
        self
    }

    /// Set the number of measures per system for line breaking.
    pub fn measures_per_system(mut self, n: usize) -> Self {
        self.measures_per_system = n;
        self
    }

    /// Set the system width in font design units.
    /// If not set (or 0), a default of 40 staff spaces is used.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Add a note to the current measure.
    pub fn note(mut self, pitch: Pitch, duration: Duration) -> Self {
        self.current_events.push(ScoreEvent::Note { pitch, duration, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None });
        self
    }

    /// Mark the most recently added note as tied forward to the next note at the
    /// same pitch. The tie curve is drawn connecting this note to the next note
    /// of the same staff position within the same system.
    ///
    /// Must be called immediately after `.note()`. Has no effect if the last event
    /// is not a note.
    pub fn tie(mut self) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { tie_forward, .. }) => *tie_forward = true,
            Some(ScoreEvent::Chord { tie_forward, .. }) => *tie_forward = true,
            _ => {}
        }
        self
    }

    /// Mark the most recently added note or chord as the start of a slur.
    ///
    /// The slur curve is drawn from this note to the next note/chord that has
    /// `slur_end()` called on it, within the same system. The curve direction
    /// is determined by the stem direction of the start note.
    pub fn slur_start(mut self) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { slur_start, .. }) => *slur_start = true,
            Some(ScoreEvent::Chord { slur_start, .. }) => *slur_start = true,
            _ => {}
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a slur.
    ///
    /// Pairs with a preceding `slur_start()` call. The slur is drawn between
    /// the most recent `slur_start` note and this note.
    pub fn slur_end(mut self) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { slur_end, .. }) => *slur_end = true,
            Some(ScoreEvent::Chord { slur_end, .. }) => *slur_end = true,
            _ => {}
        }
        self
    }

    /// Attach a dynamic marking (e.g. pp, mf, ff) to the most recently added
    /// note or chord. The dynamic is rendered below the staff, centered on
    /// the note it applies to.
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Has no effect
    /// if the last event is not a note or chord.
    pub fn dynamic(mut self, dyn_mark: Dynamic) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { dynamic, .. }) => *dynamic = Some(dyn_mark),
            Some(ScoreEvent::Chord { dynamic, .. }) => *dynamic = Some(dyn_mark),
            _ => {}
        }
        self
    }

    /// Mark the start of a hairpin (crescendo or decrescendo wedge) at the most
    /// recently added note or chord. The wedge extends from this note to the
    /// next note/chord with `hairpin_end()`.
    pub fn hairpin_start(mut self, kind: HairpinType) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { hairpin_start, .. }) => *hairpin_start = Some(kind),
            Some(ScoreEvent::Chord { hairpin_start, .. }) => *hairpin_start = Some(kind),
            _ => {}
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a hairpin wedge.
    pub fn hairpin_end(mut self) -> Self {
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { hairpin_end, .. }) => *hairpin_end = true,
            Some(ScoreEvent::Chord { hairpin_end, .. }) => *hairpin_end = true,
            _ => {}
        }
        self
    }

    /// Convenience: mark the start of a crescendo at the most recent note.
    pub fn cresc(self) -> Self {
        self.hairpin_start(HairpinType::Crescendo)
    }

    /// Convenience: mark the start of a decrescendo at the most recent note.
    pub fn decresc(self) -> Self {
        self.hairpin_start(HairpinType::Decrescendo)
    }

    /// Attach a rehearsal mark above the staff at the most recently added note
    /// or chord. The mark is rendered above the top staff line, centered on
    /// the note it applies to.
    ///
    /// `text` is the mark content (e.g. "A", "B", "1", "12").
    /// `style` controls the enclosure (boxed or plain).
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Has no effect
    /// if the last event is not a note or chord.
    pub fn rehearsal_mark(mut self, text: impl Into<String>, style: RehearsalStyle) -> Self {
        let mark = Some((text.into(), style));
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { rehearsal_mark, .. }) => *rehearsal_mark = mark,
            Some(ScoreEvent::Chord { rehearsal_mark, .. }) => *rehearsal_mark = mark,
            _ => {}
        }
        self
    }

    /// Attach a tempo marking to the most recently added note or chord.
    ///
    /// Tempo marks are rendered above the staff at the note's position.
    /// Accepts any [`TempoMark`] variant (text, metronome, or combined).
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Has no effect
    /// if the last event is not a note or chord.
    pub fn tempo(mut self, mark: TempoMark) -> Self {
        let m = Some(mark);
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { tempo_mark, .. }) => *tempo_mark = m,
            Some(ScoreEvent::Chord { tempo_mark, .. }) => *tempo_mark = m,
            _ => {}
        }
        self
    }

    /// Attach an expression text marking to the most recently added note or chord.
    ///
    /// Expression text is rendered in italic below the staff (e.g. "dolce",
    /// "espressivo", "legato", "cantabile").
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Has no effect
    /// if the last event is not a note or chord.
    pub fn expression(mut self, text: impl Into<String>) -> Self {
        let e = Some(text.into());
        match self.current_events.last_mut() {
            Some(ScoreEvent::Note { expression, .. }) => *expression = e,
            Some(ScoreEvent::Chord { expression, .. }) => *expression = e,
            _ => {}
        }
        self
    }

    /// Add a chord (multiple simultaneous pitches) to the current measure.
    ///
    /// All notes in the chord share the same duration. Noteheads that are a
    /// second apart are automatically offset to avoid collision.
    pub fn chord(mut self, pitches: Vec<Pitch>, duration: Duration) -> Self {
        self.current_events.push(ScoreEvent::Chord { pitches, duration, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None });
        self
    }

    /// Add a beam group (multiple notes connected by beams) to the current measure.
    ///
    /// All notes must be eighth notes or shorter (duration_log2 >= 3).
    /// Stem direction is auto-detected from the group's staff positions.
    ///
    /// # Example
    /// ```ignore
    /// builder.beam_group(vec![
    ///     (pitch_e4, Duration::EIGHTH),
    ///     (pitch_f4, Duration::EIGHTH),
    ///     (pitch_g4, Duration::EIGHTH),
    ///     (pitch_a4, Duration::EIGHTH),
    /// ])
    /// ```
    pub fn beam_group(mut self, notes: Vec<(Pitch, Duration)>) -> Self {
        self.current_events.push(ScoreEvent::BeamGroup { notes });
        self
    }

    /// Add a tuplet group to the current measure.
    ///
    /// Renders beamed notes with a tuplet bracket and number above or below.
    /// `tuplet_number` is the number to display (e.g. 3 for triplet, 5 for quintuplet).
    ///
    /// # Example
    /// ```ignore
    /// builder.tuplet(3, vec![
    ///     (pitch_e4, Duration::EIGHTH),
    ///     (pitch_f4, Duration::EIGHTH),
    ///     (pitch_g4, Duration::EIGHTH),
    /// ])
    /// ```
    pub fn tuplet(mut self, tuplet_number: u32, notes: Vec<(Pitch, Duration)>) -> Self {
        self.current_events.push(ScoreEvent::TupletGroup { notes, tuplet_number });
        self
    }

    /// Add a rest to the current measure.
    pub fn rest(mut self, duration: Duration) -> Self {
        self.current_events.push(ScoreEvent::Rest { duration });
        self
    }

    /// End the current measure with a single barline and start a new one.
    pub fn barline(mut self) -> Self {
        let events = std::mem::take(&mut self.current_events);
        self.measures.push((events, BarlineStyle::Single));
        self
    }

    /// End the current measure with a final (double) barline.
    /// Typically called at the end of the piece.
    pub fn end_barline(mut self) -> Self {
        let events = std::mem::take(&mut self.current_events);
        self.measures.push((events, BarlineStyle::Final));
        self
    }

    /// End the current measure with a specific barline style.
    pub fn barline_style(mut self, style: BarlineStyle) -> Self {
        let events = std::mem::take(&mut self.current_events);
        self.measures.push((events, style));
        self
    }

    /// Render the score to an SVG string, returning an error if font operations fail.
    ///
    /// Flushes any pending events as a final measure (with `Final` barline)
    /// if no explicit end barline was provided.
    #[must_use = "the SVG string is returned but not used"]
    pub fn try_render_svg(mut self) -> Result<String, crate::error::EngraverError> {
        // Flush any pending events
        if !self.current_events.is_empty() {
            let events = std::mem::take(&mut self.current_events);
            self.measures.push((events, BarlineStyle::Final));
        }

        if self.measures.is_empty() {
            return Ok(String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        }

        let font = bravura_font();
        let config = font.engraving_config();
        let staff_space = config.staff_space;

        let clef = self.clef.to_clef();

        // Convert ScoreEvents to MeasureContent, tracking accidentals within each measure
        let measure_contents: Vec<MeasureContent> = self
            .measures
            .iter()
            .map(|(events, barline)| {
                let mut seen: AccidentalTracker = HashMap::new();
                let measure_events: Vec<MeasureEvent> = events
                    .iter()
                    .map(|e| convert_event(e, &clef, &self.key_sig, Some(&mut seen)))
                    .collect();
                MeasureContent {
                    events: measure_events,
                    barline: *barline,
                }
            })
            .collect();

        // Build system prefix
        let time_sig_kind = match (&self.time_sig_kind, self.time_sig) {
            (Some(kind), _) => Some(kind.clone()),
            (None, Some((n, d))) => Some(TimeSignatureKind::Numeric {
                numerator: n,
                denominator: d,
            }),
            (None, None) => None,
        };

        let prefix = SystemPrefix::new(&clef, self.key_sig.clone(), time_sig_kind);
        let measure_config = MeasureLayoutConfig::from_staff_space(staff_space);

        let sys_width = if self.system_width > 0.0 {
            self.system_width
        } else {
            40.0 * staff_space
        };
        let page_config = PageLayoutConfig::new(staff_space, sys_width);

        let mps = if self.measures_per_system == 0 {
            4
        } else {
            self.measures_per_system
        };
        let breaking = SystemBreaking::Fixed(mps);

        let page_layout = layout_page(
            &prefix,
            &measure_contents,
            &measure_config,
            &page_config,
            &breaking,
        );

        Ok(draw_page(&font, &config, &page_layout)?.to_svg())
    }

    /// Render the score to an SVG string.
    ///
    /// Convenience wrapper around [`try_render_svg`](Self::try_render_svg) that
    /// panics on font errors. For error handling, use `try_render_svg` instead.
    ///
    /// # Panics
    ///
    /// Panics if font glyph lookup fails. This cannot happen with the bundled
    /// Bravura font because all SMuFL glyph names used by the layout engine are
    /// present in Bravura's metadata, and the font data is compiled in via
    /// `include_bytes!`. The only realistic failure path would be a corrupted
    /// binary or a future code change that requests a glyph not in the font.
    #[must_use = "the SVG string is returned but not used"]
    pub fn render_svg(self) -> String {
        self.try_render_svg()
            .expect("bundled Bravura font contains all required SMuFL glyphs")
    }

}

impl Default for ScoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests;
