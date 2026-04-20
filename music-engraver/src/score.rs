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

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::pitch::Pitch;
use music::note::spelling::{Accidental, Spelling};

use crate::font::bravura_font;
use crate::layout::accidental::accidental_glyph;
use crate::layout::barline::BarlineStyle;
use crate::layout::hairpin::HairpinType;
use crate::layout::dynamics::Dynamic;
use crate::layout::key_signature::KeySignature;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::tempo::TempoMark;
use crate::layout::measure::{BeamGroupEvent, ChordEvent, MeasureLayoutConfig, NoteEvent, RestEvent, TupletGroupEvent};
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
use crate::layout::system::{ClefKind, MeasureContent, MeasureEvent, SystemPrefix};
use crate::layout::time_signature::TimeSignatureKind;
use crate::render::page_renderer::draw_page;

/// Convert a `DurationKind` to the log2 representation used by the layout engine.
///
/// Layout uses: 0=whole, 1=half, 2=quarter, 3=eighth, etc.
/// Breve (double whole) maps to 0 as well since the layout engine doesn't
/// distinguish breves from wholes for spacing purposes.
fn duration_kind_to_log2(kind: DurationKind) -> u8 {
    match kind {
        DurationKind::Breve => 0,
        DurationKind::Whole => 0,
        DurationKind::Half => 1,
        DurationKind::Qtr => 2,
        DurationKind::Eighth => 3,
        DurationKind::Sixteenth => 4,
        DurationKind::ThirtySecond => 5,
        DurationKind::SixtyFourth => 6,
        DurationKind::OneTwentyEighth => 7,
    }
}

/// Resolve whether an accidental should be displayed for a given pitch in a key signature.
///
/// Suppresses accidentals that are redundant with the key signature (e.g., F# in D major).
/// Shows naturals that cancel key-signature alterations (e.g., F♮ in D major).
/// Double sharps/flats are always shown since they never appear in key signatures.
/// Does not track within-measure accidental state — each note is resolved independently.
#[cfg(test)]
fn should_show_accidental(pitch: &Pitch, key_sig: &KeySignature) -> Option<smufl::Glyph> {
    resolve_accidental(pitch, key_sig, None)
}

/// Key for tracking accidentals within a measure: (diatonic letter index 0–6, octave).
/// Uses `i32::from(&Letter)` since `Letter` doesn't implement `Hash`/`Eq`.
type NoteKey = (i32, u8);

/// Map tracking which accidental was last shown for each note (letter+octave) in a measure.
type AccidentalTracker = HashMap<NoteKey, Accidental>;

fn note_key(pitch: &Pitch) -> NoteKey {
    let spelling = Spelling::from(&pitch.note);
    (i32::from(&spelling.letter), pitch.octave)
}

/// The effective accidental state for a note: what accidental applies to this letter+octave.
///
/// For tracking purposes: Natural on an unaltered note = None (no accidental in effect),
/// Natural on an altered note = Natural (cancelling), Sharp/Flat/Double = themselves.
fn effective_accidental(pitch: &Pitch, key_sig: &KeySignature) -> Option<Accidental> {
    let spelling = Spelling::from(&pitch.note);
    let acc = spelling.acc;
    let altered = note_altered_in_key(spelling.letter, key_sig);

    match acc {
        Accidental::Natural => {
            if altered {
                Some(Accidental::Natural)
            } else {
                None
            }
        }
        _ => Some(acc),
    }
}

/// Resolve whether an accidental should be displayed, with optional within-measure tracking.
///
/// `seen_in_measure`: if Some, maps note identity (letter+octave) to the last accidental
/// shown for that note in this measure. Suppresses repeated accidentals and shows courtesy
/// naturals when a previous accidental in the measure is cancelled.
fn resolve_accidental(
    pitch: &Pitch,
    key_sig: &KeySignature,
    seen_in_measure: Option<&AccidentalTracker>,
) -> Option<smufl::Glyph> {
    let spelling = Spelling::from(&pitch.note);
    let acc = spelling.acc;
    let altered = note_altered_in_key(spelling.letter, key_sig);
    let key = note_key(pitch);

    // Check within-measure tracking
    if let Some(seen) = seen_in_measure {
        if let Some(&prev_acc) = seen.get(&key) {
            let current_effective = effective_accidental(pitch, key_sig);
            if current_effective == Some(prev_acc) {
                // Same accidental already displayed — suppress
                return None;
            }
            // Different accidental — show it, including naturals cancelling
            // a previous accidental shown within this measure
            if acc == Accidental::Natural {
                return accidental_glyph(Accidental::Natural, true);
            }
        }
    }

    match acc {
        Accidental::Natural => {
            if altered {
                accidental_glyph(Accidental::Natural, true)
            } else {
                // Courtesy natural: if a previous note in this measure had an accidental
                // on the same letter+octave, show a natural to clarify
                if let Some(seen) = seen_in_measure {
                    if seen.contains_key(&key) {
                        return accidental_glyph(Accidental::Natural, true);
                    }
                }
                None
            }
        }
        Accidental::Sharp => {
            if altered && matches!(key_sig, KeySignature::Sharps(_)) {
                None
            } else {
                accidental_glyph(Accidental::Sharp, false)
            }
        }
        Accidental::Flat => {
            if altered && matches!(key_sig, KeySignature::Flats(_)) {
                None
            } else {
                accidental_glyph(Accidental::Flat, false)
            }
        }
        Accidental::DoubleSharp | Accidental::DoubleFlat => accidental_glyph(acc, false),
    }
}

/// Check whether a letter name is altered (has a sharp or flat) in the given key signature.
fn note_altered_in_key(letter: music::note::spelling::Letter, key_sig: &KeySignature) -> bool {
    use music::note::spelling::Letter;

    let sharp_order = [
        Letter::F,
        Letter::C,
        Letter::G,
        Letter::D,
        Letter::A,
        Letter::E,
        Letter::B,
    ];
    let flat_order = [
        Letter::B,
        Letter::E,
        Letter::A,
        Letter::D,
        Letter::G,
        Letter::C,
        Letter::F,
    ];

    match key_sig {
        KeySignature::Open => false,
        KeySignature::Sharps(n) => {
            let n = (*n as usize).min(7);
            sharp_order[..n].contains(&letter)
        }
        KeySignature::Flats(n) => {
            let n = (*n as usize).min(7);
            flat_order[..n].contains(&letter)
        }
    }
}

/// An event being accumulated in the current measure.
#[derive(Clone, Debug)]
enum ScoreEvent {
    Note { pitch: Pitch, duration: Duration, tie_forward: bool, dynamic: Option<Dynamic>, slur_start: bool, slur_end: bool, hairpin_start: Option<HairpinType>, hairpin_end: bool, rehearsal_mark: Option<(String, RehearsalStyle)>, tempo_mark: Option<TempoMark>, expression: Option<String> },
    Rest { duration: Duration },
    Chord { pitches: Vec<Pitch>, duration: Duration, tie_forward: bool, dynamic: Option<Dynamic>, slur_start: bool, slur_end: bool, hairpin_start: Option<HairpinType>, hairpin_end: bool, rehearsal_mark: Option<(String, RehearsalStyle)>, tempo_mark: Option<TempoMark>, expression: Option<String> },
    BeamGroup { notes: Vec<(Pitch, Duration)> },
    TupletGroup { notes: Vec<(Pitch, Duration)>, tuplet_number: u32 },
}

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
                    .map(|e| self.convert_event_tracked(e, &clef, &mut seen))
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

    /// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
    /// Does not track within-measure accidental state (each note resolved independently).
    #[cfg(test)]
    fn convert_event(&self, event: &ScoreEvent, clef: &Clef) -> MeasureEvent {
        match event {
            ScoreEvent::Note { pitch, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
                let staff_pos = pitch_to_staff_position(pitch, clef);
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();
                let acc = should_show_accidental(pitch, &self.key_sig);

                MeasureEvent::Note(NoteEvent {
                    staff_position: staff_pos,
                    duration_log2: log2,
                    dots,
                    accidental: acc,
                    stem_direction: None,
                    tie_forward: *tie_forward,
                    dynamic: *dynamic,
                    slur_start: *slur_start,
                    slur_end: *slur_end,
                    hairpin_start: *hairpin_start,
                    hairpin_end: *hairpin_end,
                    rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
                })
            }
            ScoreEvent::Rest { duration } => {
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();

                MeasureEvent::Rest(RestEvent {
                    duration_log2: log2,
                    dots,
                })
            }
            ScoreEvent::Chord { pitches, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();
                let staff_positions: Vec<i8> = pitches
                    .iter()
                    .map(|p| pitch_to_staff_position(p, clef))
                    .collect();
                let accidentals: Vec<Option<smufl::Glyph>> = pitches
                    .iter()
                    .map(|p| should_show_accidental(p, &self.key_sig))
                    .collect();

                MeasureEvent::Chord(ChordEvent {
                    staff_positions,
                    duration_log2: log2,
                    dots,
                    accidentals,
                    stem_direction: None,
                    tie_forward: *tie_forward,
                    dynamic: *dynamic,
                    slur_start: *slur_start,
                    slur_end: *slur_end,
                    hairpin_start: *hairpin_start,
                    hairpin_end: *hairpin_end,
                    rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
                })
            }
            ScoreEvent::BeamGroup { notes } => {
                let note_events: Vec<NoteEvent> = notes
                    .iter()
                    .map(|(pitch, duration)| {
                        let staff_pos = pitch_to_staff_position(pitch, clef);
                        let log2 = duration_kind_to_log2(duration.kind());
                        let dots = duration.num_dots();
                        let acc = should_show_accidental(pitch, &self.key_sig);
                        NoteEvent {
                            staff_position: staff_pos,
                            duration_log2: log2,
                            dots,
                            accidental: acc,
                            stem_direction: None,
                            tie_forward: false,
                            dynamic: None,
                            slur_start: false,
                            slur_end: false,
                            hairpin_start: None,
                            hairpin_end: false,
                            rehearsal_mark: None, tempo_mark: None, expression: None,
                        }
                    })
                    .collect();
                MeasureEvent::BeamGroup(BeamGroupEvent {
                    notes: note_events,
                    stem_direction: None,
                })
            }
            ScoreEvent::TupletGroup { notes, tuplet_number } => {
                let note_events: Vec<NoteEvent> = notes
                    .iter()
                    .map(|(pitch, duration)| {
                        let staff_pos = pitch_to_staff_position(pitch, clef);
                        let log2 = duration_kind_to_log2(duration.kind());
                        let dots = duration.num_dots();
                        let acc = should_show_accidental(pitch, &self.key_sig);
                        NoteEvent {
                            staff_position: staff_pos,
                            duration_log2: log2,
                            dots,
                            accidental: acc,
                            stem_direction: None,
                            tie_forward: false,
                            dynamic: None,
                            slur_start: false,
                            slur_end: false,
                            hairpin_start: None,
                            hairpin_end: false,
                            rehearsal_mark: None, tempo_mark: None, expression: None,
                        }
                    })
                    .collect();
                MeasureEvent::TupletGroup(TupletGroupEvent {
                    beam_group: BeamGroupEvent {
                        notes: note_events,
                        stem_direction: None,
                    },
                    tuplet_number: *tuplet_number,
                })
            }
        }
    }

    /// Convert a `ScoreEvent` into a `MeasureEvent`, tracking accidentals within the measure.
    ///
    /// `seen` maps note identity (letter+octave) to the accidental last displayed for that
    /// note in this measure. Suppresses redundant accidentals and shows courtesy naturals.
    /// Resets at each measure boundary (caller provides a fresh map per measure).
    fn convert_event_tracked(
        &self,
        event: &ScoreEvent,
        clef: &Clef,
        seen: &mut AccidentalTracker,
    ) -> MeasureEvent {
        match event {
            ScoreEvent::Note { pitch, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
                let staff_pos = pitch_to_staff_position(pitch, clef);
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();
                let acc = resolve_accidental(pitch, &self.key_sig, Some(seen));

                // Update tracking: record what accidental state this note establishes
                let key = note_key(pitch);
                if let Some(eff) = effective_accidental(pitch, &self.key_sig) {
                    seen.insert(key, eff);
                } else {
                    seen.remove(&key);
                }

                MeasureEvent::Note(NoteEvent {
                    staff_position: staff_pos,
                    duration_log2: log2,
                    dots,
                    accidental: acc,
                    stem_direction: None,
                    tie_forward: *tie_forward,
                    dynamic: *dynamic,
                    slur_start: *slur_start,
                    slur_end: *slur_end,
                    hairpin_start: *hairpin_start,
                    hairpin_end: *hairpin_end,
                    rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
                })
            }
            ScoreEvent::Rest { duration } => {
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();

                MeasureEvent::Rest(RestEvent {
                    duration_log2: log2,
                    dots,
                })
            }
            ScoreEvent::Chord { pitches, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
                let log2 = duration_kind_to_log2(duration.kind());
                let dots = duration.num_dots();
                let staff_positions: Vec<i8> = pitches
                    .iter()
                    .map(|p| pitch_to_staff_position(p, clef))
                    .collect();
                let accidentals: Vec<Option<smufl::Glyph>> = pitches
                    .iter()
                    .map(|p| {
                        let acc = resolve_accidental(p, &self.key_sig, Some(seen));
                        // Update tracking for each note in the chord
                        let key = note_key(p);
                        if let Some(eff) = effective_accidental(p, &self.key_sig) {
                            seen.insert(key, eff);
                        } else {
                            seen.remove(&key);
                        }
                        acc
                    })
                    .collect();

                MeasureEvent::Chord(ChordEvent {
                    staff_positions,
                    duration_log2: log2,
                    dots,
                    accidentals,
                    stem_direction: None,
                    tie_forward: *tie_forward,
                    dynamic: *dynamic,
                    slur_start: *slur_start,
                    slur_end: *slur_end,
                    hairpin_start: *hairpin_start,
                    hairpin_end: *hairpin_end,
                    rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
                })
            }
            ScoreEvent::BeamGroup { notes } => {
                let note_events: Vec<NoteEvent> = notes
                    .iter()
                    .map(|(pitch, duration)| {
                        let staff_pos = pitch_to_staff_position(pitch, clef);
                        let log2 = duration_kind_to_log2(duration.kind());
                        let dots = duration.num_dots();
                        let acc = resolve_accidental(pitch, &self.key_sig, Some(seen));

                        // Update tracking
                        let key = note_key(pitch);
                        if let Some(eff) = effective_accidental(pitch, &self.key_sig) {
                            seen.insert(key, eff);
                        } else {
                            seen.remove(&key);
                        }

                        NoteEvent {
                            staff_position: staff_pos,
                            duration_log2: log2,
                            dots,
                            accidental: acc,
                            stem_direction: None,
                            tie_forward: false,
                            dynamic: None,
                            slur_start: false,
                            slur_end: false,
                            hairpin_start: None,
                            hairpin_end: false,
                            rehearsal_mark: None, tempo_mark: None, expression: None,
                        }
                    })
                    .collect();
                MeasureEvent::BeamGroup(BeamGroupEvent {
                    notes: note_events,
                    stem_direction: None,
                })
            }
            ScoreEvent::TupletGroup { notes, tuplet_number } => {
                let note_events: Vec<NoteEvent> = notes
                    .iter()
                    .map(|(pitch, duration)| {
                        let staff_pos = pitch_to_staff_position(pitch, clef);
                        let log2 = duration_kind_to_log2(duration.kind());
                        let dots = duration.num_dots();
                        let acc = resolve_accidental(pitch, &self.key_sig, Some(seen));

                        let key = note_key(pitch);
                        if let Some(eff) = effective_accidental(pitch, &self.key_sig) {
                            seen.insert(key, eff);
                        } else {
                            seen.remove(&key);
                        }

                        NoteEvent {
                            staff_position: staff_pos,
                            duration_log2: log2,
                            dots,
                            accidental: acc,
                            stem_direction: None,
                            tie_forward: false,
                            dynamic: None,
                            slur_start: false,
                            slur_end: false,
                            hairpin_start: None,
                            hairpin_end: false,
                            rehearsal_mark: None, tempo_mark: None, expression: None,
                        }
                    })
                    .collect();
                MeasureEvent::TupletGroup(TupletGroupEvent {
                    beam_group: BeamGroupEvent {
                        notes: note_events,
                        stem_direction: None,
                    },
                    tuplet_number: *tuplet_number,
                })
            }
        }
    }
}

impl Default for ScoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::note::Note;

    // --- duration_kind_to_log2 ---

    #[test]
    fn log2_whole_is_0() {
        assert_eq!(duration_kind_to_log2(DurationKind::Whole), 0);
    }

    #[test]
    fn log2_quarter_is_2() {
        assert_eq!(duration_kind_to_log2(DurationKind::Qtr), 2);
    }

    #[test]
    fn log2_eighth_is_3() {
        assert_eq!(duration_kind_to_log2(DurationKind::Eighth), 3);
    }

    #[test]
    fn log2_sixteenth_is_4() {
        assert_eq!(duration_kind_to_log2(DurationKind::Sixteenth), 4);
    }

    #[test]
    fn log2_128th_is_7() {
        assert_eq!(duration_kind_to_log2(DurationKind::OneTwentyEighth), 7);
    }

    #[test]
    fn log2_breve_is_0() {
        assert_eq!(duration_kind_to_log2(DurationKind::Breve), 0);
    }

    // --- note_altered_in_key ---

    #[test]
    fn f_altered_in_one_sharp() {
        assert!(note_altered_in_key(
            music::note::spelling::Letter::F,
            &KeySignature::Sharps(1)
        ));
    }

    #[test]
    fn c_not_altered_in_one_sharp() {
        assert!(!note_altered_in_key(
            music::note::spelling::Letter::C,
            &KeySignature::Sharps(1)
        ));
    }

    #[test]
    fn c_altered_in_two_sharps() {
        assert!(note_altered_in_key(
            music::note::spelling::Letter::C,
            &KeySignature::Sharps(2)
        ));
    }

    #[test]
    fn b_altered_in_one_flat() {
        assert!(note_altered_in_key(
            music::note::spelling::Letter::B,
            &KeySignature::Flats(1)
        ));
    }

    #[test]
    fn b_not_altered_in_open_key() {
        assert!(!note_altered_in_key(
            music::note::spelling::Letter::B,
            &KeySignature::Open
        ));
    }

    #[test]
    fn all_altered_in_seven_sharps() {
        use music::note::spelling::Letter;
        for letter in [
            Letter::C,
            Letter::D,
            Letter::E,
            Letter::F,
            Letter::G,
            Letter::A,
            Letter::B,
        ] {
            assert!(note_altered_in_key(letter, &KeySignature::Sharps(7)));
        }
    }

    // --- should_show_accidental ---

    #[test]
    fn sharp_note_shows_sharp_glyph() {
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn sharp_note_suppressed_when_in_sharp_key() {
        // F# in D major (2 sharps) — F is sharped in the key sig, so the
        // accidental is redundant and suppressed.
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, None);
    }

    #[test]
    fn natural_note_in_sharp_key_shows_natural() {
        // F natural in D major — F is sharped in key sig, so we show a natural
        let pitch = Pitch::new(Note::F, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalNatural));
    }

    #[test]
    fn natural_note_in_open_key_no_accidental() {
        let pitch = Pitch::new(Note::C, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, None);
    }

    #[test]
    fn flat_note_shows_flat() {
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn sharp_note_shown_in_flat_key() {
        // F# in Bb major (2 flats) — F is not flatted, so sharp is shown
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn sharp_note_shown_when_not_in_key() {
        // G# in D major (2 sharps: F#, C#) — G is not altered, so show the sharp
        let pitch = Pitch::new(Note::Gis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn flat_note_suppressed_when_in_flat_key() {
        // Bb in Bb major (2 flats: Bb, Eb) — B is flatted in key sig, suppress
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, None);
    }

    #[test]
    fn flat_note_shown_in_sharp_key() {
        // Bb in G major (1 sharp) — B is not altered, show the flat
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(1));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn flat_note_shown_when_not_in_key() {
        // Ab in Bb major (2 flats: Bb, Eb) — A is not flatted, show the flat
        let pitch = Pitch::new(Note::Aes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn double_sharp_shows_double_sharp() {
        let pitch = Pitch::new(Note::Fisis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
    }

    #[test]
    fn double_sharp_shown_even_in_sharp_key() {
        // F## in D major — double sharps are never in key signatures
        let pitch = Pitch::new(Note::Fisis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
    }

    #[test]
    fn double_flat_shown_even_in_flat_key() {
        let pitch = Pitch::new(Note::Beses, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleFlat));
    }

    // --- ScoreBuilder ---

    #[test]
    fn empty_score_produces_svg() {
        let svg = ScoreBuilder::new().render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn single_note_score_produces_svg_with_path() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path"), "should contain glyph paths");
        assert!(svg.contains("<line"), "should contain staff lines");
    }

    #[test]
    fn score_with_key_sig_shows_accidental_paths() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
            .render_svg();
        // Key sig has 2 sharps → at least 2 extra paths beyond clef+notehead
        let path_count = svg.matches("<path").count();
        assert!(
            path_count >= 4,
            "expected >= 4 paths (clef + 2 key sig sharps + notehead), got {}",
            path_count
        );
    }

    #[test]
    fn score_with_time_sig() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(3, 4)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .render_svg();
        // Time sig numerals are paths
        let path_count = svg.matches("<path").count();
        assert!(
            path_count >= 5,
            "expected >= 5 paths (clef + 2 time sig digits + 3 noteheads), got {}",
            path_count
        );
    }

    #[test]
    fn multi_measure_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::WHOLE)
            .end_barline()
            .render_svg();
        assert!(svg.starts_with("<svg"));
        // Should have at least 2 barline elements (single + final)
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 7,
            "expected >= 7 lines (5 staff + barlines), got {}",
            line_count
        );
    }

    #[test]
    fn rest_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .render_svg();
        assert!(svg.contains("<path"), "rest should produce a path");
    }

    #[test]
    fn bass_clef_score() {
        let svg_treble = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 3).unwrap(), Duration::QTR)
            .render_svg();
        let svg_bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .note(Pitch::new(Note::C, 3).unwrap(), Duration::QTR)
            .render_svg();
        // Different clef → different clef glyph path data
        assert_ne!(svg_treble, svg_bass);
    }

    #[test]
    fn dotted_note_produces_dot_glyph() {
        let dotted_qtr = Duration::new(DurationKind::Qtr, 1);
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), dotted_qtr)
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef + notehead + dot = at least 3 paths
        assert!(
            path_count >= 3,
            "expected >= 3 paths for dotted note, got {}",
            path_count
        );
    }

    #[test]
    fn eighth_note_produces_flag() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH)
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef + notehead + flag = at least 3 paths
        assert!(
            path_count >= 3,
            "expected >= 3 paths for eighth note (clef+notehead+flag), got {}",
            path_count
        );
    }

    #[test]
    fn measures_per_system_affects_layout() {
        let build = || {
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .time_signature(4, 4)
                .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::D, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::E, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::F, 4).unwrap(), Duration::WHOLE)
                .end_barline()
        };

        let svg_2 = build().measures_per_system(2).render_svg();
        let svg_4 = build().measures_per_system(4).render_svg();

        // With 2 measures/system, 4 measures → 2 systems → 2 sets of staff lines (10 total)
        // With 4 measures/system, 4 measures → 1 system → 1 set of staff lines (5 total)
        let lines_2 = svg_2.matches("<line").count();
        let lines_4 = svg_4.matches("<line").count();
        assert!(
            lines_2 > lines_4,
            "2 measures/system ({} lines) should have more lines than 4/system ({} lines)",
            lines_2,
            lines_4
        );
    }

    #[test]
    fn natural_shown_when_cancelling_key_sig() {
        // In D major (F#, C#), an F natural should show a natural accidental
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef + 2 key sig sharps + notehead + natural accidental = 5
        assert!(
            path_count >= 5,
            "expected >= 5 paths (natural accidental shown), got {}",
            path_count
        );
    }

    #[test]
    fn convert_event_note_preserves_staff_position() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let pitch = Pitch::new(Note::B, 4).unwrap();
        let event = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                // B4 in treble: G4=pos 2, A4=pos 3, B4=pos 4 (middle line)
                assert_eq!(n.staff_position, 4);
                assert_eq!(n.duration_log2, 2);
                assert_eq!(n.dots, 0);
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_rest_preserves_duration() {
        let builder = ScoreBuilder::new();
        let dur = Duration::new(DurationKind::Half, 1); // dotted half
        let event = ScoreEvent::Rest { duration: dur };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Rest(r) => {
                assert_eq!(r.duration_log2, 1);
                assert_eq!(r.dots, 1);
            }
            _ => panic!("expected Rest event"),
        }
    }

    #[test]
    fn common_time_uses_common_symbol() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .common_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // Common time uses a single glyph (timeSigCommon), not two digit glyphs.
        // With numeric 4/4 we'd get 2 digit paths; with common we get 1 symbol path.
        let svg_numeric = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // The SVGs should differ because different glyphs are used
        assert_ne!(svg, svg_numeric);
    }

    #[test]
    fn cut_time_uses_cut_common_symbol() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .cut_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        let svg_common = ScoreBuilder::new()
            .clef(Clef::Treble)
            .common_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // Cut time and common time produce different SVGs
        assert_ne!(svg, svg_common);
    }

    #[test]
    fn cut_time_differs_from_numeric_2_2() {
        let svg_cut = ScoreBuilder::new()
            .clef(Clef::Treble)
            .cut_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        let svg_numeric = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(2, 2)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        assert_ne!(svg_cut, svg_numeric);
    }

    #[test]
    fn pending_events_auto_flushed_as_final_barline() {
        // Not calling barline() or end_barline() — events should still render
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path"));
    }

    #[test]
    fn try_render_svg_returns_ok_for_valid_input() {
        let result = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .try_render_svg();
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path"));
    }

    #[test]
    fn try_render_svg_empty_score_returns_ok() {
        let result = ScoreBuilder::new().try_render_svg();
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn try_render_svg_matches_render_svg() {
        let svg_try = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4)
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .barline()
            .rest(Duration::WHOLE)
            .end_barline()
            .try_render_svg()
            .unwrap();
        let svg_direct = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4)
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .barline()
            .rest(Duration::WHOLE)
            .end_barline()
            .render_svg();
        assert_eq!(svg_try, svg_direct);
    }

    // --- within-measure accidental tracking ---

    #[test]
    fn resolve_accidental_with_tracking_suppresses_repeated_sharp() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let pitch = Pitch::new(Note::Fis, 4).unwrap();

        // First occurrence: show sharp
        let first = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(first, Some(smufl::Glyph::AccidentalSharp));

        // Record it
        seen.insert(note_key(&pitch), Accidental::Sharp);

        // Second occurrence: suppress (same accidental already shown)
        let second = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(second, None, "repeated sharp should be suppressed");
    }

    #[test]
    fn resolve_accidental_with_tracking_suppresses_repeated_flat() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let pitch = Pitch::new(Note::Bes, 4).unwrap();

        let first = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(first, Some(smufl::Glyph::AccidentalFlat));
        seen.insert(note_key(&pitch), Accidental::Flat);

        let second = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(second, None, "repeated flat should be suppressed");
    }

    #[test]
    fn resolve_accidental_shows_courtesy_natural_after_sharp() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let sharp_pitch = Pitch::new(Note::Fis, 4).unwrap();
        let natural_pitch = Pitch::new(Note::F, 4).unwrap();

        // Show sharp on F#4
        seen.insert(note_key(&sharp_pitch), Accidental::Sharp);

        // F4 natural should show a courtesy natural
        let result = resolve_accidental(&natural_pitch, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalNatural),
            "natural should show after sharp on same letter+octave"
        );
    }

    #[test]
    fn resolve_accidental_shows_courtesy_natural_after_flat() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let flat_pitch = Pitch::new(Note::Bes, 4).unwrap();
        let natural_pitch = Pitch::new(Note::B, 4).unwrap();

        seen.insert(note_key(&flat_pitch), Accidental::Flat);

        let result = resolve_accidental(&natural_pitch, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalNatural),
            "natural should show after flat on same letter+octave"
        );
    }

    #[test]
    fn resolve_accidental_different_octaves_independent() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let fis4 = Pitch::new(Note::Fis, 4).unwrap();
        let fis5 = Pitch::new(Note::Fis, 5).unwrap();

        // Show sharp on F#4
        seen.insert(note_key(&fis4), Accidental::Sharp);

        // F#5 is a different octave — should still show sharp
        let result = resolve_accidental(&fis5, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalSharp),
            "sharp on different octave should not be suppressed"
        );
    }

    #[test]
    fn convert_event_tracked_suppresses_repeated_accidental() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let pitch = Pitch::new(Note::Fis, 4).unwrap();

        let ev1 = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let ev2 = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
        };

        let r1 = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);
        let r2 = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);

        match (&r1, &r2) {
            (MeasureEvent::Note(n1), MeasureEvent::Note(n2)) => {
                assert!(
                    n1.accidental.is_some(),
                    "first F#4 should show accidental"
                );
                assert!(
                    n2.accidental.is_none(),
                    "second F#4 should suppress accidental"
                );
            }
            _ => panic!("expected Note events"),
        }
    }

    #[test]
    fn convert_event_tracked_shows_natural_after_sharp() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        let ev_sharp = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let ev_natural = ScoreEvent::Note {
            pitch: Pitch::new(Note::F, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };

        let _ = builder.convert_event_tracked(&ev_sharp, &Clef::Treble, &mut seen);
        let r2 = builder.convert_event_tracked(&ev_natural, &Clef::Treble, &mut seen);

        match r2 {
            MeasureEvent::Note(n) => {
                assert_eq!(
                    n.accidental,
                    Some(smufl::Glyph::AccidentalNatural),
                    "F natural after F# should show courtesy natural"
                );
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn tracking_resets_between_measures_in_render() {
        // F#4 in measure 1, then F#4 in measure 2 — both should show sharp
        // (tracking resets at barline)
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR) // suppressed
            .barline()
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR) // should show (new measure)
            .end_barline()
            .render_svg();

        // Count accidental paths: measure 1 has 1 sharp, measure 2 has 1 sharp = 2 sharps
        // Each measure also has noteheads. Clef adds 1 path.
        // Without tracking: 3 sharps. With tracking: 2 sharps.
        let path_count = svg.matches("<path").count();
        // clef(1) + 3 noteheads + 2 sharps = 6 paths
        assert_eq!(
            path_count, 6,
            "expected 6 paths (1 clef + 3 noteheads + 2 sharps, with 1 suppressed), got {}",
            path_count
        );
    }

    #[test]
    fn tracking_within_measure_suppresses_duplicate() {
        // Two F#4 in one measure — second should not show sharp
        let svg_tracked = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // One F#4 alone for comparison
        let svg_single = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let paths_tracked = svg_tracked.matches("<path").count();
        let paths_single = svg_single.matches("<path").count();

        // Tracked: clef + 2 noteheads + 1 sharp = 4
        // Single: clef + 1 notehead + 1 sharp = 3
        // Difference should be exactly 1 (one extra notehead, no extra sharp)
        assert_eq!(
            paths_tracked - paths_single,
            1,
            "adding duplicate F# should add only 1 path (notehead), not 2 (notehead+sharp)"
        );
    }

    #[test]
    fn effective_accidental_natural_on_altered() {
        let pitch = Pitch::new(Note::F, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Sharps(1));
        assert_eq!(eff, Some(Accidental::Natural));
    }

    #[test]
    fn effective_accidental_natural_on_unaltered() {
        let pitch = Pitch::new(Note::C, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Open);
        assert_eq!(eff, None);
    }

    #[test]
    fn effective_accidental_sharp() {
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Open);
        assert_eq!(eff, Some(Accidental::Sharp));
    }

    #[test]
    fn note_key_same_letter_different_octave() {
        let p1 = Pitch::new(Note::C, 3).unwrap();
        let p2 = Pitch::new(Note::C, 5).unwrap();
        assert_ne!(note_key(&p1), note_key(&p2));
    }

    #[test]
    fn note_key_same_note_same_octave() {
        let p1 = Pitch::new(Note::C, 4).unwrap();
        let p2 = Pitch::new(Note::C, 4).unwrap();
        assert_eq!(note_key(&p1), note_key(&p2));
    }

    #[test]
    fn note_key_enharmonic_different() {
        // C# and Db are different letters
        let p1 = Pitch::new(Note::Cis, 4).unwrap();
        let p2 = Pitch::new(Note::Des, 4).unwrap();
        assert_ne!(note_key(&p1), note_key(&p2));
    }

    // --- chord support in ScoreBuilder ---

    #[test]
    fn chord_produces_multiple_noteheads() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef(1) + 3 noteheads = 4 paths minimum
        assert!(
            path_count >= 4,
            "expected >= 4 paths (clef + 3 noteheads), got {}",
            path_count
        );
    }

    #[test]
    fn chord_has_more_paths_than_single_note() {
        let svg_note = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .render_svg();
        let svg_chord = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .render_svg();
        let paths_note = svg_note.matches("<path").count();
        let paths_chord = svg_chord.matches("<path").count();
        assert!(
            paths_chord > paths_note,
            "chord ({} paths) should have more paths than single note ({} paths)",
            paths_chord,
            paths_note
        );
    }

    #[test]
    fn chord_with_accidentals_in_key() {
        // In D major (F#, C#): chord with F#4 and A4
        // F# is suppressed (in key sig), A has no accidental
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .chord(
                vec![
                    Pitch::new(Note::Fis, 4).unwrap(),
                    Pitch::new(Note::A, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef(1) + 2 key sig sharps + 2 noteheads + 0 accidentals = 5
        assert_eq!(
            path_count, 5,
            "expected 5 paths (clef + 2 key sharps + 2 noteheads, no extra accidentals), got {}",
            path_count
        );
    }

    #[test]
    fn chord_with_natural_accidental() {
        // In D major, chord with F♮4 and G4 — natural should appear on F
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .chord(
                vec![
                    Pitch::new(Note::F, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .render_svg();
        let path_count = svg.matches("<path").count();
        // clef(1) + 2 key sharps + 2 noteheads + 1 natural = 6
        assert_eq!(
            path_count, 6,
            "expected 6 paths (clef + 2 key sharps + 2 noteheads + 1 natural), got {}",
            path_count
        );
    }

    #[test]
    fn convert_event_chord_maps_pitches() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::E, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.staff_positions.len(), 2);
                // E4 in treble = pos 0, G4 = pos 2
                assert_eq!(c.staff_positions[0], 0);
                assert_eq!(c.staff_positions[1], 2);
                assert_eq!(c.duration_log2, 2);
                assert_eq!(c.dots, 0);
                assert_eq!(c.accidentals.len(), 2);
            }
            _ => panic!("expected Chord event"),
        }
    }

    #[test]
    fn convert_event_tracked_chord_suppresses_repeated_accidental() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        // First: single F#4 note
        let ev1 = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then: chord with F#4 (should be suppressed) and A4
        let ev2 = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::Fis, 4).unwrap(),
                Pitch::new(Note::A, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Chord(c) => {
                // F#4's accidental should be suppressed (already shown)
                assert_eq!(c.accidentals[0], None, "F#4 accidental should be suppressed");
                // A4 has no accidental in C major
                assert_eq!(c.accidentals[1], None, "A4 should have no accidental");
            }
            _ => panic!("expected Chord event"),
        }
    }

    // --- beam group ---

    #[test]
    fn beam_group_renders_svg_with_polygons() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .beam_group(vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
            ])
            .end_barline()
            .render_svg();

        assert!(svg.starts_with("<svg"));
        // 4 noteheads + clef + time sig digits
        let path_count = svg.matches("<path ").count();
        assert!(path_count >= 4, "at least 4 paths for noteheads, got {path_count}");
        // Beam polygons (1 primary for all eighth notes)
        let polygon_count = svg.matches("<polygon ").count();
        assert!(polygon_count >= 1, "at least 1 beam polygon, got {polygon_count}");
    }

    #[test]
    fn beam_group_differs_from_individual_eighth_notes() {
        let e4 = Pitch::new(Note::E, 4).unwrap();
        let f4 = Pitch::new(Note::F, 4).unwrap();

        let svg_beamed = ScoreBuilder::new()
            .clef(Clef::Treble)
            .beam_group(vec![
                (e4, Duration::EIGHTH),
                (f4, Duration::EIGHTH),
            ])
            .end_barline()
            .render_svg();

        let svg_flagged = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(e4, Duration::EIGHTH)
            .note(f4, Duration::EIGHTH)
            .end_barline()
            .render_svg();

        // Beamed version should have polygons, flagged should not
        assert!(svg_beamed.matches("<polygon ").count() >= 1, "beamed has polygons");
        assert_eq!(svg_flagged.matches("<polygon ").count(), 0, "flagged has no polygons");
    }

    #[test]
    fn beam_group_convert_event_produces_beam_group_measure_event() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::BeamGroup {
            notes: vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
            ],
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::BeamGroup(bg) => {
                assert_eq!(bg.notes.len(), 2);
                // E4 in treble: pos 0, G4: pos 2
                assert_eq!(bg.notes[0].staff_position, 0);
                assert_eq!(bg.notes[1].staff_position, 2);
                assert_eq!(bg.notes[0].duration_log2, 3);
                assert_eq!(bg.notes[1].duration_log2, 3);
                assert!(bg.stem_direction.is_none());
            }
            _ => panic!("expected BeamGroup event"),
        }
    }

    #[test]
    fn beam_group_tracked_accidentals() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        // First F#4 note (standalone)
        let ev1 = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then beam group with F#4 again — should suppress repeated accidental
        let ev2 = ScoreEvent::BeamGroup {
            notes: vec![
                (Pitch::new(Note::Fis, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
            ],
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::BeamGroup(bg) => {
                assert!(bg.notes[0].accidental.is_none(), "F#4 accidental suppressed");
                assert!(bg.notes[1].accidental.is_none(), "A4 has no accidental");
            }
            _ => panic!("expected BeamGroup event"),
        }
    }

    // --- tie support ---

    #[test]
    fn tie_produces_filled_path_in_svg() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .tie()
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // Tie should produce a filled path with stroke="none"
        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert!(tie_count >= 1, "expected at least 1 tie path, got {tie_count}");
        // Should contain Bézier curves
        assert!(svg.contains(" C"), "tie should have cubic Bézier curves");
    }

    #[test]
    fn tie_across_barline_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .tie()
            .barline()
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert!(tie_count >= 1, "tie across barline should produce a tie path");
    }

    #[test]
    fn no_tie_without_tie_call() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "no tie without .tie() call");
    }

    #[test]
    fn tie_on_rest_has_no_effect() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .tie() // Should have no effect since last event is a rest
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "tie after rest should have no effect");
    }

    #[test]
    fn tie_differs_from_untied() {
        let svg_tied = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .tie()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .end_barline()
            .render_svg();

        let svg_untied = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .end_barline()
            .render_svg();

        assert_ne!(svg_tied, svg_untied, "tied and untied should produce different SVGs");
        // Tied version should be longer (has tie path)
        assert!(
            svg_tied.len() > svg_untied.len(),
            "tied SVG should be larger"
        );
    }

    #[test]
    fn convert_event_preserves_tie_forward() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.tie_forward, "tie_forward should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    // --- chord tie support ---

    #[test]
    fn tie_after_chord_sets_tie_forward() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.tie_forward, "chord tie_forward should be preserved");
            }
            _ => panic!("expected Chord event"),
        }
    }

    #[test]
    fn chord_tie_produces_filled_paths() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .tie()
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        // 3 notes in the chord → 3 ties
        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 3, "expected 3 ties (one per chord note), got {tie_count}");
    }

    #[test]
    fn chord_tie_differs_from_untied_chord() {
        let build = || {
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .chord(
                    vec![
                        Pitch::new(Note::E, 4).unwrap(),
                        Pitch::new(Note::G, 4).unwrap(),
                    ],
                    Duration::HALF,
                )
        };

        let svg_tied = build()
            .tie()
            .chord(
                vec![
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        let svg_untied = build()
            .chord(
                vec![
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        assert_ne!(svg_tied, svg_untied, "tied chord should differ from untied");
        assert!(
            svg_tied.len() > svg_untied.len(),
            "tied chord SVG should be larger (contains tie paths)"
        );
    }

    #[test]
    fn chord_tie_no_effect_without_tie_call() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![Pitch::new(Note::C, 4).unwrap(), Pitch::new(Note::E, 4).unwrap()],
                Duration::HALF,
            )
            .chord(
                vec![Pitch::new(Note::C, 4).unwrap(), Pitch::new(Note::E, 4).unwrap()],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "no ties without .tie() call");
    }

    #[test]
    fn convert_event_tracked_chord_preserves_tie_forward() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.tie_forward, "tracked conversion should preserve tie_forward");
            }
            _ => panic!("expected Chord event"),
        }
    }

    // --- dynamics integration ---

    #[test]
    fn dynamic_on_note_produces_extra_path() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let paths_with = svg_with.matches("<path ").count();
        let paths_without = svg_without.matches("<path ").count();
        assert_eq!(
            paths_with,
            paths_without + 1,
            "dynamic should add exactly 1 path (the dynamic glyph)"
        );
    }

    #[test]
    fn different_dynamics_produce_different_svg() {
        let svg_p = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Piano)
            .render_svg();
        let svg_f = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .render_svg();
        assert_ne!(svg_p, svg_f, "p and f should produce different SVGs");
    }

    #[test]
    fn dynamic_on_chord_produces_extra_path() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .dynamic(Dynamic::Ff)
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .end_barline()
            .render_svg();

        let paths_with = svg_with.matches("<path ").count();
        let paths_without = svg_without.matches("<path ").count();
        assert_eq!(
            paths_with,
            paths_without + 1,
            "dynamic on chord should add exactly 1 path"
        );
    }

    #[test]
    fn dynamic_on_rest_has_no_effect() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .dynamic(Dynamic::Mf) // should be ignored — last event is a rest
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .end_barline()
            .render_svg();

        assert_eq!(svg_with, svg_without, "dynamic on rest should have no effect");
    }

    #[test]
    fn convert_event_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Pp),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.dynamic, Some(Dynamic::Pp), "dynamic should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Fff),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.dynamic, Some(Dynamic::Fff), "tracked conversion preserves dynamic");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_chord_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Sfz),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.dynamic, Some(Dynamic::Sfz), "chord dynamic should be preserved");
            }
            _ => panic!("expected Chord event"),
        }
    }

    #[test]
    fn multiple_dynamics_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Piano)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::B, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .note(Pitch::new(Note::D, 5).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // Should have clef + 2 time sig digits + 4 noteheads + 2 dynamics = 9 paths
        let path_count = svg.matches("<path ").count();
        assert_eq!(
            path_count, 9,
            "expected 9 paths (clef + 2 time digits + 4 noteheads + 2 dynamics), got {}",
            path_count
        );
    }

    // --- tuplet support in ScoreBuilder ---

    #[test]
    fn tuplet_renders_svg_with_bracket() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .tuplet(3, vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
            ])
            .end_barline()
            .render_svg();

        assert!(svg.starts_with("<svg"));
        // 3 noteheads + clef + 2 time sig digits + 1 tuplet number = 7 paths
        let path_count = svg.matches("<path ").count();
        assert_eq!(path_count, 7, "expected 7 paths, got {path_count}");
        // Should have beam polygon
        let polygon_count = svg.matches("<polygon ").count();
        assert!(polygon_count >= 1, "should have beam polygon(s)");
        // Should have bracket lines (hooks + bracket segments)
        let line_count = svg.matches("<line ").count();
        // 5 staff + 3 stems + 4 bracket/hooks + barline(s) = should be > 10
        assert!(line_count > 10, "expected > 10 lines (staff + stems + bracket), got {line_count}");
    }

    #[test]
    fn tuplet_differs_from_beam_group() {
        let notes = vec![
            (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
            (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
            (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
        ];

        let svg_beam = ScoreBuilder::new()
            .clef(Clef::Treble)
            .beam_group(notes.clone())
            .end_barline()
            .render_svg();

        let svg_tuplet = ScoreBuilder::new()
            .clef(Clef::Treble)
            .tuplet(3, notes)
            .end_barline()
            .render_svg();

        // Tuplet should have extra content (bracket + number)
        assert!(
            svg_tuplet.len() > svg_beam.len(),
            "tuplet SVG ({}) should be larger than beam group SVG ({})",
            svg_tuplet.len(),
            svg_beam.len()
        );

        // Tuplet has 1 extra path (number glyph)
        let paths_beam = svg_beam.matches("<path ").count();
        let paths_tuplet = svg_tuplet.matches("<path ").count();
        assert_eq!(paths_tuplet, paths_beam + 1, "tuplet adds 1 path for number glyph");
    }

    #[test]
    fn tuplet_convert_event_produces_tuplet_group() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::TupletGroup {
            notes: vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::B, 4).unwrap(), Duration::EIGHTH),
            ],
            tuplet_number: 3,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::TupletGroup(tg) => {
                assert_eq!(tg.tuplet_number, 3);
                assert_eq!(tg.beam_group.notes.len(), 3);
                assert_eq!(tg.beam_group.notes[0].staff_position, 0); // E4
                assert_eq!(tg.beam_group.notes[1].staff_position, 2); // G4
                assert_eq!(tg.beam_group.notes[2].staff_position, 4); // B4
                assert_eq!(tg.beam_group.notes[0].duration_log2, 3);
            }
            _ => panic!("expected TupletGroup event"),
        }
    }

    #[test]
    fn tuplet_tracked_accidentals() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        // First F#4 note (standalone)
        let ev1 = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then tuplet with F#4 again — should suppress repeated accidental
        let ev2 = ScoreEvent::TupletGroup {
            notes: vec![
                (Pitch::new(Note::Fis, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::C, 5).unwrap(), Duration::EIGHTH),
            ],
            tuplet_number: 3,
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::TupletGroup(tg) => {
                assert!(tg.beam_group.notes[0].accidental.is_none(), "F#4 accidental suppressed");
                assert!(tg.beam_group.notes[1].accidental.is_none(), "A4 has no accidental");
            }
            _ => panic!("expected TupletGroup event"),
        }
    }

    #[test]
    fn tuplet_quintuplet_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .tuplet(5, vec![
                (Pitch::new(Note::C, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::D, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::E, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::SIXTEENTH),
            ])
            .end_barline()
            .render_svg();

        assert!(svg.starts_with("<svg"));
        // 5 noteheads + clef + 1 tuplet number = 7 paths
        let path_count = svg.matches("<path ").count();
        assert_eq!(path_count, 7, "expected 7 paths, got {path_count}");
    }

    // --- slur tests ---

    #[test]
    fn slur_start_end_produces_filled_path() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .slur_start()
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .slur_end()
            .end_barline()
            .render_svg();

        // Should contain a filled slur path (Bézier curves)
        let filled_count = svg.matches(r#"stroke="none""#).count();
        assert!(filled_count >= 1, "expected at least 1 filled slur path, got {filled_count}");
        assert!(svg.contains(" C"), "slur should contain cubic Bézier commands");
    }

    #[test]
    fn slurred_differs_from_unslurred() {
        let slurred = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .slur_start()
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .slur_end()
            .end_barline()
            .render_svg();

        let unslurred = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        assert_ne!(slurred, unslurred, "slurred should differ from un-slurred");
    }

    #[test]
    fn slur_on_rest_is_noop() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .slur_start()
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // slur_start on a rest is a no-op; no slur_end anywhere, so no slur
        let filled_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(filled_count, 0, "slur_start on rest should be a no-op");
    }

    #[test]
    fn convert_event_preserves_slur_flags() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: true,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.slur_start, "slur_start should be preserved");
                assert!(!n.slur_end, "slur_end should be false");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_slur_flags() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::G, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: true,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(!n.slur_start, "slur_start should be false");
                assert!(n.slur_end, "slur_end should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn chord_slur_preserves_flags() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: true,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.slur_start, "chord slur_start should be preserved");
                assert!(!c.slur_end);
            }
            _ => panic!("expected Chord event"),
        }
    }

    // --- hairpin tests ---

    use crate::layout::hairpin::HairpinType;

    /// Helper to create a Pitch from a note name string and octave.
    fn p(name: &str, octave: u8) -> Pitch {
        let note = match name {
            "C" => Note::C,
            "D" => Note::D,
            "E" => Note::E,
            "F" => Note::F,
            "G" => Note::G,
            "A" => Note::A,
            "B" => Note::B,
            _ => panic!("unsupported note name: {name}"),
        };
        Pitch::new(note, octave).unwrap()
    }

    #[test]
    fn hairpin_adds_lines_to_svg() {
        let svg_hp = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("C", 4), Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR)
            .note(p("G", 4), Duration::QTR).hairpin_end()
            .rest(Duration::QTR)
            .end_barline()
            .render_svg();

        let svg_no = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("C", 4), Duration::QTR)
            .note(p("E", 4), Duration::QTR)
            .note(p("G", 4), Duration::QTR)
            .rest(Duration::QTR)
            .end_barline()
            .render_svg();

        let hp_lines = svg_hp.matches("<line ").count();
        let no_lines = svg_no.matches("<line ").count();
        assert_eq!(hp_lines, no_lines + 2, "hairpin adds 2 lines");
    }

    #[test]
    fn decresc_differs_from_cresc() {
        let svg_c = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("C", 4), Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
            .end_barline()
            .render_svg();

        let svg_d = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("C", 4), Duration::QTR).decresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
            .end_barline()
            .render_svg();

        assert_ne!(svg_c, svg_d, "cresc and decresc should differ");
    }

    #[test]
    fn hairpin_on_rest_is_noop() {
        let svg1 = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
            .end_barline()
            .render_svg();

        let svg2 = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR)
            .note(p("E", 4), Duration::QTR)
            .end_barline()
            .render_svg();

        // cresc() on a rest is a no-op, so hairpin_start is never set
        // hairpin_end without matching start produces no hairpin
        assert_eq!(svg1, svg2, "hairpin on rest should be no-op");
    }

    #[test]
    fn convert_event_preserves_hairpin_fields() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: Some(HairpinType::Crescendo),
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.hairpin_start, Some(HairpinType::Crescendo));
                assert!(!n.hairpin_end);
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_hairpin_fields() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: true,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let mut seen = HashMap::new();
        let result = builder.convert_event_tracked(&event, &clef, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.hairpin_start.is_none());
                assert!(n.hairpin_end);
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn chord_hairpin_preserved_in_convert() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Chord {
            pitches: vec![p("C", 4), p("E", 4)],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: Some(HairpinType::Decrescendo),
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.hairpin_start, Some(HairpinType::Decrescendo));
                assert!(!c.hairpin_end);
            }
            _ => panic!("expected Chord"),
        }
    }

    // --- Rehearsal mark integration tests ---

    #[test]
    fn rehearsal_mark_adds_text_element_to_svg() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::QTR)
            .rehearsal_mark("A", RehearsalStyle::Boxed)
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();
        // Boxed rehearsal mark produces a <text> element and a <rect> element
        assert!(svg.contains("<text "), "rehearsal mark should produce a <text> element");
        assert!(svg.contains("<rect "), "boxed rehearsal mark should produce a <rect> element");
        assert!(svg.contains(">A<"), "text content 'A' should appear in the SVG");
    }

    #[test]
    fn rehearsal_mark_on_chord_adds_text() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
            .rehearsal_mark("B", RehearsalStyle::Boxed)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">B<"), "chord rehearsal mark text should appear in SVG");
        assert!(svg.contains("<rect "), "boxed style should produce a rect");
    }

    #[test]
    fn plain_rehearsal_mark_has_text_but_no_rect() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("C", 5), Duration::WHOLE)
            .rehearsal_mark("C", RehearsalStyle::Plain)
            .end_barline()
            .render_svg();
        assert!(svg.contains(">C<"), "plain rehearsal mark text should appear");
        // Plain style should NOT have a rect (only boxed does)
        assert!(!svg.contains("<rect "), "plain rehearsal mark should not have a rect");
    }

    #[test]
    fn rehearsal_mark_on_rest_is_noop() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::WHOLE)
            .rehearsal_mark("X", RehearsalStyle::Boxed)
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::WHOLE)
            .end_barline()
            .render_svg();
        // rehearsal_mark after rest is a no-op
        assert_eq!(svg_with, svg_without, "rehearsal_mark on rest should have no effect");
    }

    #[test]
    fn note_with_rehearsal_differs_from_without() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("D", 5), Duration::QTR)
            .rehearsal_mark("1", RehearsalStyle::Boxed)
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("D", 5), Duration::QTR)
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();
        assert_ne!(svg_with, svg_without, "rehearsal mark should change the SVG output");
    }

    #[test]
    fn convert_event_preserves_rehearsal_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("A".to_string(), RehearsalStyle::Boxed)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.rehearsal_mark, Some(("A".to_string(), RehearsalStyle::Boxed)));
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_rehearsal_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("B".to_string(), RehearsalStyle::Plain)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let mut seen = HashMap::new();
        let result = builder.convert_event_tracked(&event, &clef, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.rehearsal_mark, Some(("B".to_string(), RehearsalStyle::Plain)));
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn chord_convert_preserves_rehearsal_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Chord {
            pitches: vec![p("C", 4), p("E", 4)],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("12".to_string(), RehearsalStyle::Boxed)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.rehearsal_mark, Some(("12".to_string(), RehearsalStyle::Boxed)));
            }
            _ => panic!("expected Chord"),
        }
    }

    // --- Tempo mark integration tests ---

    #[test]
    fn tempo_mark_adds_text_to_svg() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::QTR)
            .tempo(TempoMark::Text("Allegro".into()))
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();

        assert!(svg.contains(">Allegro<"), "tempo text should appear in SVG");
        assert!(svg.contains("bold"), "tempo text should be bold");
    }

    #[test]
    fn tempo_metronome_adds_bpm_to_svg() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("C", 4), Duration::QTR)
            .tempo(TempoMark::Metronome {
                note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            })
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();

        assert!(svg.contains("= 120"), "BPM should appear in SVG");
    }

    #[test]
    fn tempo_on_rest_is_noop() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::WHOLE)
            .tempo(TempoMark::Text("Andante".into()))
            .end_barline()
            .render_svg();

        // tempo() on rest should be ignored — "Andante" should NOT appear
        assert!(!svg_with.contains(">Andante<"), "tempo mark on rest should be ignored");
    }

    #[test]
    fn tempo_differs_from_no_tempo() {
        let with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::WHOLE)
            .tempo(TempoMark::Text("Vivace".into()))
            .end_barline()
            .render_svg();

        let without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::WHOLE)
            .end_barline()
            .render_svg();

        assert_ne!(with, without, "tempo mark should change SVG output");
    }

    #[test]
    fn convert_event_preserves_tempo_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Text("Largo".into())),
            expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.tempo_mark, Some(TempoMark::Text("Largo".into())));
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn tracked_convert_preserves_tempo_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let mut seen = HashMap::new();
        let event = ScoreEvent::Note {
            pitch: p("D", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Metronome {
                note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 60,
            }),
            expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.tempo_mark.is_some());
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn chord_convert_preserves_tempo_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Chord {
            pitches: vec![p("C", 4), p("E", 4)],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Text("Adagio".into())),
            expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.tempo_mark, Some(TempoMark::Text("Adagio".into())));
            }
            _ => panic!("expected Chord"),
        }
    }

    // --- Expression text integration tests ---

    #[test]
    fn expression_adds_italic_text_to_svg() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::QTR)
            .expression("dolce")
            .rest(Duration::new(DurationKind::Half, 1))
            .end_barline()
            .render_svg();

        assert!(svg.contains(">dolce<"), "expression text should appear in SVG");
        assert!(svg.contains("italic"), "expression text should be italic");
    }

    #[test]
    fn expression_on_rest_is_noop() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::WHOLE)
            .expression("legato")
            .end_barline()
            .render_svg();

        assert!(!svg.contains(">legato<"), "expression on rest should be ignored");
    }

    #[test]
    fn expression_differs_from_no_expression() {
        let with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::WHOLE)
            .expression("espressivo")
            .end_barline()
            .render_svg();

        let without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("E", 4), Duration::WHOLE)
            .end_barline()
            .render_svg();

        assert_ne!(with, without, "expression should change SVG output");
    }

    #[test]
    fn convert_event_preserves_expression() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: Some("cantabile".into()),
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.expression, Some("cantabile".into()));
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn chord_expression_produces_text() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
            .expression("sostenuto")
            .end_barline()
            .render_svg();

        assert!(svg.contains(">sostenuto<"), "chord expression should appear");
        assert!(svg.contains("italic"), "chord expression should be italic");
    }
}