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
pub mod multi_staff;

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
#[cfg(test)]
use music::notation::rhythm::duration::DurationKind;
use music::note::pitch::Pitch;
#[cfg(test)]
use music::note::spelling::Accidental;

use crate::font::bravura_font;
use crate::layout::articulation::Articulation;
use crate::layout::barline::BarlineStyle;
use crate::layout::grace::GraceNoteKind;
use crate::layout::hairpin::HairpinType;
use crate::layout::dynamics::Dynamic;
use crate::layout::key_signature::KeySignature;
use crate::layout::lyric::LyricSyllable;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::tempo::TempoMark;
use crate::layout::measure::{MeasureLayoutConfig, NoteAnnotations};
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
    pub(crate) measures: Vec<(Vec<ScoreEvent>, BarlineStyle)>,
    /// Measures per system (for line breaking). 0 = auto (4 per system).
    pub(crate) measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    pub(crate) system_width: f64,
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
        self.current_events.push(ScoreEvent::Note { pitch, duration, annotations: NoteAnnotations::default() });
        self
    }

    /// Mark the most recently added note as tied forward to the next note at the
    /// same pitch. The tie curve is drawn connecting this note to the next note
    /// of the same staff position within the same system.
    ///
    /// Must be called immediately after `.note()`. Has no effect if the last event
    /// is not a note.
    pub fn tie(mut self) -> Self {
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.tie_forward = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the start of a slur.
    ///
    /// The slur curve is drawn from this note to the next note/chord that has
    /// `slur_end()` called on it, within the same system. The curve direction
    /// is determined by the stem direction of the start note.
    pub fn slur_start(mut self) -> Self {
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.slur_start = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a slur.
    ///
    /// Pairs with a preceding `slur_start()` call. The slur is drawn between
    /// the most recent `slur_start` note and this note.
    pub fn slur_end(mut self) -> Self {
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.slur_end = true;
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
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.dynamic = Some(dyn_mark);
        }
        self
    }

    /// Mark the start of a hairpin (crescendo or decrescendo wedge) at the most
    /// recently added note or chord. The wedge extends from this note to the
    /// next note/chord with `hairpin_end()`.
    pub fn hairpin_start(mut self, kind: HairpinType) -> Self {
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.hairpin_start = Some(kind);
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a hairpin wedge.
    pub fn hairpin_end(mut self) -> Self {
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.hairpin_end = true;
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
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.rehearsal_mark = mark;
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
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.tempo_mark = m;
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
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) = self.current_events.last_mut() {
            annotations.expression = e;
        }
        self
    }

    /// Attach a lyric syllable to the most recently added note or chord.
    ///
    /// Lyrics are rendered in roman (upright) text below the staff, below
    /// dynamics and expression text. Use [`LyricSyllable::word`] for
    /// end-of-word syllables, [`LyricSyllable::with_hyphen`] for mid-word
    /// syllables (displays trailing hyphen), and [`LyricSyllable::with_extender`]
    /// for melismatic syllables (sustained across multiple notes).
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Has no effect
    /// if the last event is not a note or chord.
    pub fn lyric(mut self, syllable: LyricSyllable) -> Self {
        if let Some(
            ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. },
        ) = self.current_events.last_mut()
        {
            annotations.lyric = Some(syllable);
        }
        self
    }

    /// Attach an articulation (staccato, tenuto, accent, marcato, staccatissimo,
    /// fermata) to the most recently added note or chord.
    ///
    /// Placement (above/below) is determined automatically from stem direction.
    /// Fermata is always placed above. No-op if the last event was a rest.
    pub fn articulation(mut self, artic: Articulation) -> Self {
        if let Some(
            ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. },
        ) = self.current_events.last_mut()
        {
            annotations.articulation = Some(artic);
        }
        self
    }

    /// Attach a grace note to the most recently added note or chord.
    ///
    /// The grace note is rendered as a small composite glyph (notehead + stem +
    /// optional slash) to the left of the principal note. `pitch` is the grace
    /// note's pitch; `kind` selects acciaccatura (slashed) or appoggiatura.
    ///
    /// No-op if the last event is a rest (grace notes attach to pitched events).
    pub fn grace_note(mut self, pitch: Pitch, kind: GraceNoteKind) -> Self {
        use crate::layout::note_placement::pitch_to_staff_position;
        let clef = self.clef.to_clef();
        let staff_pos = pitch_to_staff_position(&pitch, &clef);
        if let Some(
            ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. },
        ) = self.current_events.last_mut()
        {
            annotations.grace_note = Some((staff_pos, kind));
        }
        self
    }

    /// Add a chord (multiple simultaneous pitches) to the current measure.
    ///
    /// All notes in the chord share the same duration. Noteheads that are a
    /// second apart are automatically offset to avoid collision.
    pub fn chord(mut self, pitches: Vec<Pitch>, duration: Duration) -> Self {
        self.current_events.push(ScoreEvent::Chord { pitches, duration, annotations: NoteAnnotations::default() });
        self
    }

    /// Add a beam group (multiple notes connected by beams) to the current measure.
    ///
    /// All notes must be eighth notes or shorter (duration_log2 >= 3).
    /// Stem direction is auto-detected from the group's staff positions.
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::pitch::Pitch;
    /// use music::note::note::Note;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// let svg = ScoreBuilder::new()
    ///     .beam_group(vec![
    ///         (Pitch::new(Note::E, 4).expect("valid pitch"), Duration::EIGHTH),
    ///         (Pitch::new(Note::F, 4).expect("valid pitch"), Duration::EIGHTH),
    ///         (Pitch::new(Note::G, 4).expect("valid pitch"), Duration::EIGHTH),
    ///         (Pitch::new(Note::A, 4).expect("valid pitch"), Duration::EIGHTH),
    ///     ])
    ///     .end_barline()
    ///     .render_svg();
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
    /// ```no_run
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::pitch::Pitch;
    /// use music::note::note::Note;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// let svg = ScoreBuilder::new()
    ///     .tuplet(3, vec![
    ///         (Pitch::new(Note::E, 4).expect("valid pitch"), Duration::EIGHTH),
    ///         (Pitch::new(Note::F, 4).expect("valid pitch"), Duration::EIGHTH),
    ///         (Pitch::new(Note::G, 4).expect("valid pitch"), Duration::EIGHTH),
    ///     ])
    ///     .end_barline()
    ///     .render_svg();
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

    /// Flush any pending events as a final measure if not already flushed.
    pub(crate) fn flush_pending(&mut self) {
        if !self.current_events.is_empty() {
            let events = std::mem::take(&mut self.current_events);
            self.measures.push((events, BarlineStyle::Final));
        }
    }

    /// Convert accumulated `ScoreEvent`s into `MeasureContent`s suitable for layout.
    ///
    /// Each measure's accidentals are tracked independently (courtesy naturals,
    /// suppression of redundant accidentals within a measure).
    pub(crate) fn build_measure_contents(&self) -> Vec<MeasureContent> {
        let clef = self.clef.to_clef();
        self.measures
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
            .collect()
    }

    /// Build the system prefix (clef, key sig, time sig) for this score's stave.
    pub(crate) fn build_prefix(&self) -> SystemPrefix {
        let clef = self.clef.to_clef();
        let time_sig_kind = match (&self.time_sig_kind, self.time_sig) {
            (Some(kind), _) => Some(kind.clone()),
            (None, Some((n, d))) => Some(TimeSignatureKind::Numeric {
                numerator: n,
                denominator: d,
            }),
            (None, None) => None,
        };
        SystemPrefix::new(&clef, self.key_sig.clone(), time_sig_kind)
    }

    /// Effective system width in font design units (auto = 40 staff spaces).
    pub(crate) fn effective_system_width(&self, staff_space: f64) -> f64 {
        if self.system_width > 0.0 {
            self.system_width
        } else {
            40.0 * staff_space
        }
    }

    /// Effective measures per system (auto = 4).
    pub(crate) fn effective_measures_per_system(&self) -> usize {
        if self.measures_per_system == 0 {
            4
        } else {
            self.measures_per_system
        }
    }

    /// Render the score to an SVG string, returning an error if font operations fail.
    ///
    /// Flushes any pending events as a final measure (with `Final` barline)
    /// if no explicit end barline was provided.
    #[must_use = "the SVG string is returned but not used"]
    pub fn try_render_svg(mut self) -> Result<String, crate::error::EngraverError> {
        self.flush_pending();

        if self.measures.is_empty() {
            return Ok(String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        }

        let font = bravura_font();
        let config = font.engraving_config();
        let staff_space = config.staff_space;

        let measure_contents = self.build_measure_contents();
        let prefix = self.build_prefix();
        let measure_config = MeasureLayoutConfig::from_staff_space(staff_space);
        let sys_width = self.effective_system_width(staff_space);
        let page_config = PageLayoutConfig::new(staff_space, sys_width);
        let breaking = SystemBreaking::Fixed(self.effective_measures_per_system());

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

    /// Render the score to PNG bytes at the given scale factor.
    ///
    /// Combines [`try_render_svg`](Self::try_render_svg) with
    /// [`PngRenderer`](crate::render::png::PngRenderer) in one call.
    /// System fonts are loaded so that text elements (rehearsal marks,
    /// tempo markings, expression text) render with proper serif/sans-serif
    /// fonts.
    ///
    /// Scale `1.0` produces the SVG's intrinsic pixel dimensions;
    /// `2.0` is suitable for retina/HiDPI output.
    #[cfg(feature = "png")]
    pub fn try_render_png(self, scale: f32) -> Result<Vec<u8>, crate::error::EngraverError> {
        let svg = self.try_render_svg()?;
        let mut renderer = crate::render::png::PngRenderer::new(scale);
        renderer.load_system_fonts();
        Ok(renderer.render_png(&svg)?)
    }

    /// Render the score to PNG bytes at the given scale factor.
    ///
    /// Convenience wrapper around [`try_render_png`](Self::try_render_png)
    /// that panics on error. See [`render_svg`](Self::render_svg) for panic
    /// safety discussion.
    #[cfg(feature = "png")]
    #[must_use = "the PNG bytes are returned but not used"]
    pub fn render_png(self, scale: f32) -> Vec<u8> {
        self.try_render_png(scale)
            .expect("bundled Bravura font and PNG pipeline should not fail for valid input")
    }
}

impl Default for ScoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests;
