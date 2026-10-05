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
//!     .note(Pitch::new(Note::D, 4), Duration::QTR)
//!     .note(Pitch::new(Note::E, 4), Duration::QTR)
//!     .note(Pitch::new(Note::Fis, 4), Duration::QTR)
//!     .note(Pitch::new(Note::G, 4), Duration::QTR)
//!     .barline()
//!     .rest(Duration::WHOLE)
//!     .end_barline()
//!     .render_svg();
//! ```

mod event;
mod groups;
pub mod guitar;
pub mod multi_staff;
pub mod tab;
pub use groups::GroupSpanError;

#[cfg(test)]
use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
#[cfg(test)]
use music::notation::rhythm::duration::DurationKind;
use music::note::pitch::Pitch;
#[cfg(test)]
use music::note::spelling::Accidental;

use crate::font::bravura_font;
use crate::layout::analysis_bracket::AnalysisBracketSpec;
#[cfg(test)]
use crate::layout::accidental::ResolvedAccidental;
use crate::layout::accidental::{AccidentalDisplay, AccidentalPolicy};
use crate::layout::arpeggio::ArpeggioDirection;
use crate::layout::articulation::{Articulation, ArticulationMark};
use crate::layout::bar_number::MeasureNumbering;
use crate::layout::barline::BarlineStyle;
use crate::layout::breath::BreathMark;
use crate::layout::dynamics::DynamicMark;
use crate::layout::glissando::GlissandoStyle;
use crate::layout::grace::GraceNotes;
use crate::layout::group::{BeamSpec, GroupMark, TupletSpec};
use crate::layout::hairpin::{HairpinType, NientePlacement};
use crate::layout::key_signature::KeySignature;
use crate::layout::line_break::{LineBreakPlan, LineBreakRequest};
use crate::layout::lyric::{LyricStyle, LyricSyllable, VerseLyric};
use crate::layout::measure::{MeasureLayoutConfig, NoteAnnotations, NoteSize, StemVisibility};
use crate::layout::measure_meta::{LineBreak, MeasureLength, MeasureMeta};
use crate::layout::navigation::NavigationSign;
use crate::layout::ornament::Ornament;
use crate::layout::ottava::OttavaKind;
use crate::layout::page::{layout_page, PageLayout, PageLayoutConfig, SystemBreaking};
use crate::layout::pedal::PedalMark;
use crate::layout::placement::Placement;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::stem::StemDirection;
use crate::layout::system::{
    ClefChange, ClefChangePlacement, ClefKind, MeasureContent, MeasureEvent, SystemPrefix,
};
use crate::layout::tempo::TempoMark;
use crate::layout::text_script::TextScript;
use crate::layout::text_spanner::TextSpanner;
use crate::layout::time_signature::{TimeSignature, TimeSignatureKind};
use crate::layout::tremolo::TremoloCount;
use crate::layout::trill_extension::TrillWiggleSpeed;
use crate::layout::volta::{VoltaAnnotation, VoltaHooks};
use crate::render::page_renderer::draw_page;

#[cfg(test)]
use event::{
    convert_event, duration_kind_to_log2, note_altered_in_key, note_key, resolve_accidental,
    should_show_accidental, AccidentalTracker,
};
use event::{convert_resolved_event, measure_timeline, resolve_measure_accidentals, ScoreEvent};

/// Measure-level timing directives in force when a measure was closed.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MeasureTiming {
    /// Pickup length declared with [`ScoreBuilder::partial`].
    partial: Option<MeasureLength>,
    /// Nominal length set with [`ScoreBuilder::measure_length`].
    length_override: Option<MeasureLength>,
    /// Whether any part of the measure was in cadenza mode.
    cadenza: bool,
}

/// A completed measure: voiced events, barline style, optional volta
/// annotation, and its timing directives.
#[derive(Clone, Debug)]
pub(crate) struct CompletedMeasure {
    pub(crate) events: Vec<(u8, ScoreEvent)>,
    pub(crate) barline: BarlineStyle,
    pub(crate) volta: Option<VoltaAnnotation>,
    pub(crate) timing: MeasureTiming,
}

/// A score whose structure cannot be engraved as written.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ScoreStructureError {
    /// A time-signature change was entered after the first event of a
    /// measure. Meter changes take effect at a measure start; close the
    /// measure with a barline first.
    #[error("time signature change in measure {measure} (0-based) is not at the measure start")]
    MidMeasureTimeSignatureChange {
        /// 0-based index of the offending measure.
        measure: usize,
    },
}

/// Force the voice stem direction (even voices up, odd voices down) on a
/// multi-voice measure's note or chord that has no direction yet — neither a
/// requested one nor its beam's. This follows standard engraving convention
/// for two-voice writing on a single staff.
fn force_stem_direction(event: &mut MeasureEvent, voice: usize) {
    let dir = groups::voice_stem_direction(voice);
    match event {
        MeasureEvent::Note(note) => {
            note.stem_direction.get_or_insert(dir);
        }
        MeasureEvent::Chord(chord) => {
            chord.stem_direction.get_or_insert(dir);
        }
        // Span marks, barlines, spacers, structural changes and rests have no stem.
        MeasureEvent::GroupMark(_)
        | MeasureEvent::Rest(_)
        | MeasureEvent::MultiMeasureRest { .. }
        | MeasureEvent::Spacer(_)
        | MeasureEvent::Barline(_)
        | MeasureEvent::ClefChange(_)
        | MeasureEvent::TimeSignature(_) => {}
    }
}

/// The most recent rhythmic event of `events` (optionally of one voice),
/// skipping zero-duration structural events.
fn last_rhythmic_event(
    events: &mut [(u8, ScoreEvent)],
    voice: Option<u8>,
) -> Option<&mut ScoreEvent> {
    events
        .iter_mut()
        .rev()
        .filter(|(v, _)| voice.is_none_or(|wanted| *v == wanted))
        .map(|(_, event)| event)
        .find(|event| {
            !event.is_structural()
                && !matches!(event, ScoreEvent::GroupMark(_) | ScoreEvent::Barline(_))
        })
}

/// Annotations of the most recent rhythmic event when it is a note, chord or
/// rest. Groups and multi-measure rests carry no per-event annotations.
fn last_annotations_mut(events: &mut [(u8, ScoreEvent)]) -> Option<&mut NoteAnnotations> {
    match last_rhythmic_event(events, None)? {
        ScoreEvent::Note { annotations, .. }
        | ScoreEvent::Chord { annotations, .. }
        | ScoreEvent::Rest { annotations, .. } => Some(annotations),
        _ => None,
    }
}

/// One verse occupies at most one slot on a note; entering it again replaces
/// that verse without erasing the other verses.
fn set_verse(
    annotations: &mut NoteAnnotations,
    verse: u16,
    syllable: LyricSyllable,
    style: LyricStyle,
) {
    let entry = VerseLyric {
        verse,
        syllable,
        style,
    };
    if let Some(existing) = annotations
        .lyrics
        .iter_mut()
        .find(|lyric| lyric.verse == verse)
    {
        *existing = entry;
    } else {
        annotations.lyrics.push(entry);
        annotations.lyrics.sort_by_key(|lyric| lyric.verse);
    }
}

/// Hide the stems (and with them the flags) of every note and chord in a
/// `MeasureEvent`, for a stemless score.
fn hide_stems(event: &mut MeasureEvent) {
    match event {
        MeasureEvent::Note(note) => note.annotations.stem = StemVisibility::Hidden,
        MeasureEvent::Chord(chord) => chord.annotations.stem = StemVisibility::Hidden,
        _ => {}
    }
}

/// Builder for constructing a score from `music` crate types and rendering to SVG.
///
/// Events are grouped into measures delimited by `barline()` / `end_barline()` calls.
/// The builder handles conversion of `Pitch` → staff position, `Duration` → log2 + dots,
/// and accidental resolution against the key signature.
#[derive(Clone, Debug)]
#[must_use = "a ScoreBuilder does nothing until .render_svg() or .try_render_svg() is called"]
pub struct ScoreBuilder {
    /// Clef at the start of the score.
    clef: ClefKind,
    key_sig: KeySignature,
    /// Meter declared at the start of the score (`None` = unmetered).
    time_signature: Option<TimeSignature>,
    /// Number of the first measure that is not a pickup.
    first_measure_number: i32,
    /// Pickup length declared for the measure in progress.
    pending_partial: Option<MeasureLength>,
    /// Measure-length override in force (LilyPond `Timing.measureLength`);
    /// cleared by meter changes and [`Self::reset_measure_length`].
    length_override: Option<MeasureLength>,
    /// Cadenza mode is on.
    cadenza: bool,
    /// Cadenza mode was on at some point of the measure in progress.
    cadenza_in_measure: bool,
    /// Events accumulated for the current (in-progress) measure.
    /// Each entry is `(voice_index, event)` where voice 0 is the primary voice.
    current_events: Vec<(u8, ScoreEvent)>,
    /// Completed measures.
    pub(crate) measures: Vec<CompletedMeasure>,
    /// Active voice index (0 = primary). Voice 0 stems follow auto-detection;
    /// when multiple voices are present, voice 0 forces stems up, voice 1 forces
    /// stems down.
    current_voice: u8,
    /// Per-verse associated voice, independent of the voice currently being entered.
    lyric_voices: Vec<(u16, u8)>,
    /// Measures per system (for line breaking). 0 = auto (4 per system).
    pub(crate) measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    pub(crate) system_width: f64,
    /// Use automatic width-based line breaking instead of fixed measures per system.
    pub(crate) auto_breaks: bool,
    /// Use optimal (Knuth-Plass style DP) line breaking instead of greedy.
    pub(crate) optimal_breaks: bool,
    /// Break systems only where `system_break()` was called.
    pub(crate) explicit_breaks: bool,
    /// `system_break()` / `no_break()` calls, in call order.
    line_break_requests: Vec<LineBreakRequest>,
    /// Which measures print their number.
    pub(crate) measure_numbering: MeasureNumbering,
    /// Whether we are currently inside a volta bracket region.
    in_volta: bool,
    /// Text label for the current volta bracket (set on `.volta_start()`).
    volta_text: Option<String>,
    /// Whether `.volta_end()` was called on the current measure (consumed at barline).
    volta_ending: bool,
    /// Side of the staff for dynamics, hairpins and dynamic text spanners
    /// entered from now on (`\dynamicUp` / `\dynamicDown`).
    dynamics_placement: Placement,
    /// Text marks entered before the first event; drawn above that event.
    leading_text_marks: Vec<TextScript>,
    /// How accidentals carry within a measure on this staff.
    accidental_policy: AccidentalPolicy,
    /// Whether every note and chord is engraved without stem and flags.
    stemless: bool,
}

impl ScoreBuilder {
    /// Create a new score builder with default settings (treble clef, no key/time signature).
    pub fn new() -> Self {
        Self {
            clef: ClefKind::Treble,
            key_sig: KeySignature::Open,
            time_signature: None,
            first_measure_number: 1,
            pending_partial: None,
            length_override: None,
            cadenza: false,
            cadenza_in_measure: false,
            current_events: Vec::new(),
            measures: Vec::new(),
            current_voice: 0,
            lyric_voices: Vec::new(),
            measures_per_system: 4,
            system_width: 0.0,
            auto_breaks: false,
            optimal_breaks: false,
            explicit_breaks: false,
            line_break_requests: Vec::new(),
            measure_numbering: MeasureNumbering::Hidden,
            in_volta: false,
            volta_text: None,
            volta_ending: false,
            dynamics_placement: Placement::Below,
            leading_text_marks: Vec::new(),
            accidental_policy: AccidentalPolicy::Default,
            stemless: false,
        }
    }

    /// Annotations of the most recent note, chord or rest (the target of
    /// every annotation builder that applies to rests).
    fn last_annotations_mut(&mut self) -> Option<&mut NoteAnnotations> {
        last_annotations_mut(&mut self.current_events)
    }

    /// Annotations of the most recent event when it is a note or chord (the
    /// target of pitch- and stem-bound builders such as ties, slurs, lyrics
    /// and ornaments, which do not apply to rests).
    fn last_pitched_annotations_mut(&mut self) -> Option<&mut NoteAnnotations> {
        match last_rhythmic_event(&mut self.current_events, None)? {
            ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. } => {
                Some(annotations)
            }
            _ => None,
        }
    }

    /// Set the clef at the start of the score.
    ///
    /// Once content has begun (any event entered or measure closed), this
    /// is a clef change: identical to [`Self::clef_change`]. It never
    /// rewrites earlier measures.
    pub fn clef(mut self, clef: Clef) -> Self {
        if self.has_content() {
            return self.clef_change(clef);
        }
        self.clef = ClefKind::from_clef(&clef);
        self
    }

    /// Change the clef at this point of the score.
    ///
    /// Every later pitch (in every voice) whose onset is at or after this
    /// point is placed in the new clef. The change is drawn as a change-size
    /// clef: mid-measure just before the next event; at the start of a
    /// measure before the preceding barline (LilyPond's default); at the start
    /// of a system in the system prefix, with a courtesy clef ending the
    /// previous system.
    pub fn clef_change(self, clef: Clef) -> Self {
        self.push_clef_change(clef, ClefChangePlacement::BeforeBarline)
    }

    /// Change the clef like [`Self::clef_change`], but draw a change at the
    /// start of a measure after the barline instead of before it.
    pub fn clef_change_after_barline(self, clef: Clef) -> Self {
        self.push_clef_change(clef, ClefChangePlacement::AfterBarline)
    }

    fn push_clef_change(mut self, clef: Clef, placement: ClefChangePlacement) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::ClefChange(ClefChange {
                clef: ClefKind::from_clef(&clef),
                placement,
            }),
        ));
        self
    }

    /// Whether any event has been entered or any measure closed.
    fn has_content(&self) -> bool {
        !self.measures.is_empty() || !self.current_events.is_empty()
    }

    /// Set the key signature.
    pub fn key_signature(mut self, key_sig: KeySignature) -> Self {
        self.key_sig = key_sig;
        self
    }

    /// Set the time signature (numerator, denominator) at the start of the
    /// score. Once content has begun this is a printed meter change, identical
    /// to [`Self::time_signature_change`].
    pub fn time_signature(self, numerator: u8, denominator: u8) -> Self {
        self.declare_meter(
            TimeSignatureKind::Numeric {
                numerator,
                denominator,
            },
            true,
        )
    }

    /// Set the time signature to common time (C symbol = 4/4) at the start of
    /// the score. Once content has begun this is a meter change, identical to
    /// [`Self::common_time_change`].
    pub fn common_time(self) -> Self {
        self.declare_meter(TimeSignatureKind::Common, true)
    }

    /// Set the time signature to cut time / alla breve (₵ symbol = 2/2) at the
    /// start of the score. Once content has begun this is a meter change,
    /// identical to [`Self::cut_time_change`].
    pub fn cut_time(self) -> Self {
        self.declare_meter(TimeSignatureKind::CutCommon, true)
    }

    /// Set an unprinted meter at the start of the score: measures keep its
    /// nominal length (bar structure, numbering) but no time signature is
    /// drawn (LilyPond `\omit Staff.TimeSignature`). Once content has begun
    /// this is a hidden meter change, identical to
    /// [`Self::hidden_time_signature_change`].
    pub fn hidden_time_signature(self, numerator: u8, denominator: u8) -> Self {
        self.declare_meter(
            TimeSignatureKind::Numeric {
                numerator,
                denominator,
            },
            false,
        )
    }

    /// Change the meter to `numerator/denominator`, printed after the barline
    /// that starts the new measure (or in the next system's prefix, with a
    /// courtesy signature ending the previous system).
    ///
    /// Must be entered at the start of a measure, before its first event;
    /// otherwise rendering fails with
    /// [`ScoreStructureError::MidMeasureTimeSignatureChange`]. Clears any
    /// [`Self::measure_length`] override.
    pub fn time_signature_change(self, numerator: u8, denominator: u8) -> Self {
        self.push_meter_change(
            TimeSignatureKind::Numeric {
                numerator,
                denominator,
            },
            true,
        )
    }

    /// Change the meter to common time (C), like [`Self::time_signature_change`].
    pub fn common_time_change(self) -> Self {
        self.push_meter_change(TimeSignatureKind::Common, true)
    }

    /// Change the meter to cut time (₵), like [`Self::time_signature_change`].
    pub fn cut_time_change(self) -> Self {
        self.push_meter_change(TimeSignatureKind::CutCommon, true)
    }

    /// Change the meter without printing it (LilyPond `\once \omit
    /// TimeSignature \time n/d`): nothing is drawn, but the new measure's
    /// nominal length follows the meter. Same placement rule as
    /// [`Self::time_signature_change`].
    pub fn hidden_time_signature_change(self, numerator: u8, denominator: u8) -> Self {
        self.push_meter_change(
            TimeSignatureKind::Numeric {
                numerator,
                denominator,
            },
            false,
        )
    }

    fn declare_meter(mut self, kind: TimeSignatureKind, visible: bool) -> Self {
        if self.has_content() {
            return self.push_meter_change(kind, visible);
        }
        self.time_signature = Some(TimeSignature { kind, visible });
        self.length_override = None;
        self
    }

    fn push_meter_change(mut self, kind: TimeSignatureKind, visible: bool) -> Self {
        self.length_override = None;
        self.current_events.push((
            self.current_voice,
            ScoreEvent::TimeSignatureChange(TimeSignature { kind, visible }),
        ));
        self
    }

    /// Declare the measure in progress a pickup (anacrusis) of `length`, like
    /// LilyPond's `\partial`. Accepts a [`Duration`] or any
    /// [`MeasureLength`] (e.g. `MeasureLength::new(11, 16)` for
    /// `\partial 16*11`).
    ///
    /// The pickup's nominal length is `length`; it does not advance the bar
    /// count, so a score-initial pickup is bar 0 (one before
    /// [`Self::first_measure_number`]) and a mid-score pickup shares the number
    /// of the bar before it. Pickups never print a bar number.
    pub fn partial(mut self, length: impl Into<MeasureLength>) -> Self {
        self.pending_partial = Some(length.into());
        self
    }

    /// Set the nominal length of the measure in progress and of later
    /// measures to `numerator/denominator` of a whole note without printing a
    /// meter change (LilyPond `\set Timing.measureLength`), e.g. a 9/8 bar
    /// under 4/4. In force until the next meter change or
    /// [`Self::reset_measure_length`].
    pub fn measure_length(mut self, numerator: u64, denominator: u64) -> Self {
        self.length_override = Some(MeasureLength::new(numerator, denominator));
        self
    }

    /// Return to the meter's own measure length (LilyPond `\unset
    /// Timing.measureLength`), from the measure in progress on.
    pub fn reset_measure_length(mut self) -> Self {
        self.length_override = None;
        self
    }

    /// Enter cadenza mode (LilyPond `\cadenzaOn`): measures that are in
    /// cadenza mode at any point carry no nominal length. Barlines stay
    /// explicit, as everywhere in this builder.
    pub fn cadenza_on(mut self) -> Self {
        self.cadenza = true;
        self.cadenza_in_measure = true;
        self
    }

    /// Leave cadenza mode (LilyPond `\cadenzaOff`).
    pub fn cadenza_off(mut self) -> Self {
        self.cadenza = false;
        self
    }

    /// Set the number of the first measure that is not a pickup (default 1;
    /// LilyPond `currentBarNumber`).
    pub fn first_measure_number(mut self, number: i32) -> Self {
        self.first_measure_number = number;
        self
    }

    /// Set the number of measures per system for line breaking.
    ///
    /// Explicit [`system_break`](Self::system_break)s still apply; the count
    /// restarts after each one.
    pub fn measures_per_system(mut self, n: usize) -> Self {
        self.measures_per_system = n;
        self.auto_breaks = false;
        self.explicit_breaks = false;
        self
    }

    /// Enable automatic line breaking based on natural measure widths.
    ///
    /// When enabled, measures are greedily packed onto each system until
    /// their combined natural width exceeds the system width. This produces
    /// more balanced output than a fixed measures-per-system count, especially
    /// when measures vary in density (e.g. whole notes vs. sixteenth-note runs).
    ///
    /// Overrides any previous `measures_per_system` setting.
    pub fn auto_line_breaks(mut self) -> Self {
        self.auto_breaks = true;
        self.optimal_breaks = false;
        self.explicit_breaks = false;
        self
    }

    /// Enable optimal (Knuth-Plass style) line breaking.
    ///
    /// Uses dynamic programming to minimize total whitespace deviation across
    /// all systems, producing more evenly filled lines than the greedy
    /// `auto_line_breaks`. Especially useful for scores with varying measure
    /// densities where greedy packing leaves some systems sparse while
    /// cramming others.
    ///
    /// Overrides any previous `measures_per_system` or `auto_line_breaks` setting.
    pub fn optimal_line_breaks(mut self) -> Self {
        self.optimal_breaks = true;
        self.auto_breaks = false;
        self.explicit_breaks = false;
        self
    }

    /// Break systems only where [`system_break`](Self::system_break) was
    /// called (LilyPond's `line-break-permission ##f`). Everything between
    /// two explicit breaks shares one system, compressed to the system width
    /// even when it overflows.
    ///
    /// Overrides any previous `measures_per_system`, `auto_line_breaks`, or
    /// `optimal_line_breaks` setting.
    pub fn explicit_line_breaks(mut self) -> Self {
        self.explicit_breaks = true;
        self.auto_breaks = false;
        self.optimal_breaks = false;
        self
    }

    /// Force a system break at the current position (LilyPond `\break`).
    ///
    /// The position is the end of the primary voice (voice 0) as entered so
    /// far:
    /// - right after `barline()` (or any measure-closing call), or before the
    ///   first event of a measure, the break falls on that barline;
    /// - after the last event of a measure, it falls on the measure's
    ///   closing barline;
    /// - inside a measure, the measure is split across two systems at that
    ///   onset. The first piece ends with the inline barline at that point
    ///   (see [`inline_barline`](Self::inline_barline); an invisible one when
    ///   there is none). The logical measure keeps one measure number and
    ///   one accidental state across the break.
    ///
    /// Every breaking mode honors explicit breaks. A break before the first
    /// measure or after the final barline has no effect.
    pub fn system_break(self) -> Self {
        self.request_line_break(LineBreak::Force)
    }

    /// Forbid a system break at the current position (LilyPond `\noBreak`),
    /// positioned like [`system_break`](Self::system_break). Measures joined
    /// by forbidden breaks stay on one system in every breaking mode. Inside a
    /// measure there is nothing to forbid: systems break inside a measure
    /// only at an explicit `system_break()`. The last call at a position wins.
    pub fn no_break(self) -> Self {
        self.request_line_break(LineBreak::Forbid)
    }

    fn request_line_break(mut self, kind: LineBreak) -> Self {
        let position = self
            .current_events
            .iter()
            .filter(|(voice, _)| *voice == 0)
            .count();
        self.line_break_requests.push(LineBreakRequest {
            measure: self.measures.len(),
            position,
            kind,
        });
        self
    }

    /// Insert a barline inside the current measure (LilyPond `\bar` between
    /// bar checks, e.g. the dashed `\bar "!"` subdividing a long bar or the
    /// invisible `\bar ""` marking a break point).
    ///
    /// The barline takes no musical time: it does not end the measure,
    /// advance the measure number, or reset accidentals. It belongs to the
    /// primary voice (voice 0) and follows the voice-0 events entered so far,
    /// whatever voice is active. A [`BarlineStyle::Invisible`] barline takes
    /// no space; a following [`system_break`](Self::system_break) breaks the
    /// system there.
    pub fn inline_barline(mut self, style: BarlineStyle) -> Self {
        self.current_events.push((0, ScoreEvent::Barline(style)));
        self
    }

    /// Set the system width in font design units.
    /// If not set (or 0), a default of 40 staff spaces is used.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Choose which measures print their number above the staff (default
    /// [`MeasureNumbering::Hidden`]). Numbers sit where a measure begins: at
    /// its opening barline, or after the prefix when it opens a system.
    pub fn measure_numbering(mut self, numbering: MeasureNumbering) -> Self {
        self.measure_numbering = numbering;
        self
    }

    /// Add a note to the current measure.
    pub fn note(mut self, pitch: Pitch, duration: Duration) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::Note {
                pitch,
                duration,
                annotations: NoteAnnotations::default(),
            },
        ));
        self
    }

    /// Add a note whose accidental follows an explicit [`AccidentalDisplay`]
    /// policy instead of automatic resolution.
    ///
    /// [`AccidentalDisplay::Force`] always engraves the plain accidental (a
    /// natural on an unaltered letter included); [`AccidentalDisplay::Cautionary`]
    /// always engraves it in parentheses. Either way the pitch's alteration
    /// becomes the measure's accidental state for its letter and octave.
    /// [`AccidentalDisplay::Auto`] is identical to [`Self::note`].
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::note::Note;
    /// use music::note::pitch::Pitch;
    /// use music_engraver::layout::accidental::AccidentalDisplay;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// let svg = ScoreBuilder::new()
    ///     .note(Pitch::new(Note::Fis, 4), Duration::QTR)
    ///     // Restate the sharp the measure already carries, in parentheses.
    ///     .note_with_accidental(Pitch::new(Note::Fis, 4), Duration::QTR, AccidentalDisplay::Cautionary)
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn note_with_accidental(
        self,
        pitch: Pitch,
        duration: Duration,
        display: AccidentalDisplay,
    ) -> Self {
        self.note_annotated(
            pitch,
            duration,
            NoteAnnotations {
                accidental_displays: vec![display],
                ..NoteAnnotations::default()
            },
        )
    }

    /// Mark the most recently added note or chord as tied forward to the next
    /// note at the same pitch. The tie curve is drawn connecting this note to
    /// the next note of the same staff position within the same system.
    ///
    /// Must be called immediately after `.note()` or `.chord()`. Ties bind
    /// pitches, so this has no effect when the most recent event is a rest.
    pub fn tie(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.tie_forward = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the start of a slur.
    ///
    /// The slur curve is drawn from this note to the next note/chord that has
    /// `slur_end()` called on it, within the same system. The curve direction
    /// is determined by the stem direction of the start note. Slurs bind
    /// notes, so this has no effect when the most recent event is a rest.
    pub fn slur_start(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.slur_start = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a slur.
    ///
    /// Pairs with a preceding `slur_start()` call. The slur is drawn between
    /// the most recent `slur_start` note and this note. No effect when the
    /// most recent event is a rest.
    pub fn slur_end(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.slur_end = true;
        }
        self
    }

    /// Attach a dynamic to the most recently added note, chord or rest
    /// (LilyPond `r4\p` included).
    ///
    /// Accepts a [`Dynamic`](crate::layout::dynamics::Dynamic) glyph or a
    /// [`CustomDynamic`](crate::layout::dynamics::CustomDynamic) such as "più p".
    /// The dynamic is centered on the event, below the staff unless
    /// [`Self::dynamics_placement`] put dynamics above.
    pub fn dynamic(mut self, mark: impl Into<DynamicMark>) -> Self {
        let placement = self.dynamics_placement;
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.dynamic = Some(mark.into());
            annotations.dynamics_placement = placement;
        }
        self
    }

    /// Attach a dynamic on an explicit side of the staff, overriding
    /// [`Self::dynamics_placement`] for this one event (LilyPond `^\p` /
    /// `_\p`). A hairpin starting on the same event follows it.
    pub fn dynamic_placed(mut self, mark: impl Into<DynamicMark>, placement: Placement) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.dynamic = Some(mark.into());
            annotations.dynamics_placement = placement;
        }
        self
    }

    /// Place every dynamic, hairpin and dynamic text spanner
    /// ([`Self::cresc_text`] and friends) entered from now on above or below
    /// the staff (LilyPond `\dynamicUp` / `\dynamicDown`). The default is
    /// below.
    pub fn dynamics_placement(mut self, placement: Placement) -> Self {
        self.dynamics_placement = placement;
        self
    }

    /// Mark the start of a hairpin (crescendo or decrescendo wedge) at the
    /// most recently added note, chord or rest. The wedge extends from this
    /// event to the next event with `hairpin_end()`, on the side set by
    /// [`Self::dynamics_placement`].
    pub fn hairpin_start(mut self, kind: HairpinType) -> Self {
        let placement = self.dynamics_placement;
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.hairpin_start = Some(kind);
            if annotations.dynamic.is_none() {
                annotations.dynamics_placement = placement;
            }
        }
        self
    }

    /// Mark the most recently added note, chord or rest as the end of a
    /// hairpin wedge (LilyPond `r8\!` included).
    pub fn hairpin_end(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.hairpin_end = true;
        }
        self
    }

    /// Convenience: start a crescendo at the most recent note, chord or rest.
    pub fn cresc(self) -> Self {
        self.hairpin_start(HairpinType::Crescendo)
    }

    /// Convenience: start a decrescendo at the most recent note, chord or rest.
    pub fn decresc(self) -> Self {
        self.hairpin_start(HairpinType::Decrescendo)
    }

    /// Flag the most recently added note, chord or rest as the start of a
    /// **dashed** hairpin wedge. Use after a `hairpin_start` / `cresc()` /
    /// `decresc()` call on the same note to switch the rendered wedge from
    /// solid to dashed lines.
    ///
    /// Engraved convention uses dashed wedges for "soft" or implied crescendi
    /// and for modern-notation continuation markings (independent of the
    /// `cresc. - - -` dashed-text variant, which uses dashed text rather than
    /// a wedge). The flag attaches to the start note; the wedge style spans
    /// the whole hairpin including the trailing half on the source system of
    /// a cross-system wedge. The incoming half on the next system is always
    /// dashed regardless of this flag.
    ///
    /// Must be called alongside the `hairpin_start` call on the same event.
    /// Has no visible effect if that event has no `hairpin_start` set.
    pub fn hairpin_dashed(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.hairpin_dashed = true;
        }
        self
    }

    /// Attach a niente "o" circle to the hairpin starting at the most recently
    /// added note, chord or rest, at the [`NientePlacement`] tip of the wedge.
    ///
    /// Use after a `hairpin_start` / `cresc()` / `decresc()` call on the same
    /// note. The circle marks "to/from silence":
    /// - [`NientePlacement::ClosedEnd`] — circle at the pointy tip
    ///   ("al niente" / "dal niente", standard convention)
    /// - [`NientePlacement::OpenEnd`] — circle at the wide tip (rare modern
    ///   variant, Lachenmann / Sciarrino)
    ///
    /// Combines with [`Self::hairpin_dashed`]: the wedge dashes but the
    /// circle stays solid per engraved convention. Cross-system hairpins
    /// place the circle on whichever half (trailing on the source system or
    /// incoming on the target system) contains the anchor tip.
    ///
    /// Must be called alongside the `hairpin_start` call on the same event.
    /// Has no visible effect if that event has no `hairpin_start` set.
    pub fn hairpin_niente_start(mut self, placement: NientePlacement) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.hairpin_niente = Some(placement);
        }
        self
    }

    /// Convenience: attach a closed-end niente "o" (the common
    /// "al niente" / "dal niente" convention) to the hairpin starting at the
    /// most recent note, chord or rest.
    ///
    /// Equivalent to `hairpin_niente_start(NientePlacement::ClosedEnd)`. Use
    /// `hairpin_niente_start(NientePlacement::OpenEnd)` for the rarer
    /// modern-notation variant.
    pub fn hairpin_niente(self) -> Self {
        self.hairpin_niente_start(NientePlacement::ClosedEnd)
    }

    /// Start a text spanner at the most recently added note, chord or rest:
    /// a label ("rit.", "cresc.", "dim") followed by a dashed, solid or no
    /// line running to the event marked with
    /// [`text_spanner_end`](Self::text_spanner_end) (LilyPond
    /// `\startTextSpan`). Spanners continue across system breaks: the label
    /// stays on the first system and later systems carry the line alone.
    pub fn text_spanner_start(mut self, spanner: TextSpanner) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.text_spanner_start = Some(spanner);
        }
        self
    }

    /// End the open text spanner at the most recently added note, chord or
    /// rest (LilyPond `\stopTextSpan`). The line stops just before it.
    pub fn text_spanner_end(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.text_spanner_end = true;
        }
        self
    }

    /// Convenience: start a dashed "cresc." text spanner, on the side set by
    /// [`Self::dynamics_placement`]. End it with
    /// [`text_spanner_end`](Self::text_spanner_end).
    pub fn cresc_text(self) -> Self {
        let spanner = TextSpanner::cresc().placed(self.dynamics_placement);
        self.text_spanner_start(spanner)
    }

    /// Convenience: start a dashed "decresc." text spanner, on the side set
    /// by [`Self::dynamics_placement`].
    pub fn decresc_text(self) -> Self {
        let spanner = TextSpanner::decresc().placed(self.dynamics_placement);
        self.text_spanner_start(spanner)
    }

    /// Convenience: start a dashed "dim." text spanner, on the side set by
    /// [`Self::dynamics_placement`].
    pub fn dim_text(self) -> Self {
        let spanner = TextSpanner::dim().placed(self.dynamics_placement);
        self.text_spanner_start(spanner)
    }

    /// Attach a rehearsal mark above the staff at the most recently added
    /// note, chord or rest, centered on it.
    ///
    /// `text` is the mark content (e.g. "A", "B", "1", "12").
    /// `style` controls the enclosure (boxed or plain).
    pub fn rehearsal_mark(mut self, text: impl Into<String>, style: RehearsalStyle) -> Self {
        let mark = Some((text.into(), style));
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.rehearsal_mark = mark;
        }
        self
    }

    /// Attach a tempo marking to the most recently added note, chord or rest
    /// (a tempo on a leading rest is common).
    ///
    /// The mark is drawn above the staff, left-aligned with the event. See
    /// [`TempoMark`] for words, metronome marks ("(♩. = c. 58-56)"),
    /// note = note equations and stacked text.
    pub fn tempo(mut self, mark: TempoMark) -> Self {
        let m = Some(mark);
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.tempo_mark = m;
        }
        self
    }

    /// Attach a text script to the most recently added note, chord or rest
    /// (LilyPond `^\markup` / `_\markup`): e.g. italic "a tempo" above,
    /// italic "dolce" below, or a label "a)" above a rest. Several scripts on
    /// one event stack outward on their side of the staff.
    pub fn text_script(mut self, script: TextScript) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.text_scripts.push(script);
        }
        self
    }

    /// Attach a mark to the barline at the current moment (LilyPond
    /// `\textMark` / `\textEndMark` at a bar line), e.g.
    /// `TextScript::glyph(Glyph::FermataAbove, Placement::Above)` over a
    /// final barline, or a small event number.
    ///
    /// Called after `.barline()` (or `.end_barline()`), the mark is aligned
    /// on that barline; called before it, on the barline that follows the
    /// most recent voice-0 event. The script's [`TextAlign`] is relative to
    /// the barline: `Center` for a `\textMark`, `Right` for a `\textEndMark`
    /// (so a mark on the final barline stays inside the system). A mark
    /// entered before the score's first event is drawn above that first
    /// event instead.
    ///
    /// [`TextAlign`]: crate::layout::text_script::TextAlign
    pub fn text_mark(mut self, mark: TextScript) -> Self {
        let events = if self.current_events.is_empty() {
            self.measures.last_mut().map(|measure| &mut measure.events)
        } else {
            Some(&mut self.current_events)
        };
        let target = events
            .and_then(|events| last_rhythmic_event(events, Some(0)))
            .and_then(|event| match event {
                ScoreEvent::Note { annotations, .. }
                | ScoreEvent::Chord { annotations, .. }
                | ScoreEvent::Rest { annotations, .. } => Some(annotations),
                _ => None,
            });
        match target {
            Some(annotations) => annotations.text_marks.push(mark),
            None => self.leading_text_marks.push(mark),
        }
        self
    }

    /// Attach a verse-1 upright syllable to the latest note or chord.
    /// This uses the same numbered-verse model as [`Self::lyric_verse`].
    pub fn lyric(self, syllable: LyricSyllable) -> Self {
        self.lyric_verse(1, syllable, LyricStyle::Upright)
    }

    /// Attach a syllable to the given numbered verse (1-based). Different
    /// verses on the same event coexist and have independent continuations.
    /// By default the latest pitched event is used, regardless of voice entry
    /// order. [`Self::lyric_associated_voice`] selects a voice for this verse.
    pub fn lyric_verse(mut self, verse: u16, syllable: LyricSyllable, style: LyricStyle) -> Self {
        assert!(verse > 0, "lyric verse numbers start at 1");
        let voice = self
            .lyric_voices
            .iter()
            .find(|(v, _)| *v == verse)
            .map(|(_, voice)| *voice);
        let target = if let Some(voice) = voice {
            last_rhythmic_event(&mut self.current_events, Some(voice)).and_then(|event| match event
            {
                ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. } => {
                    Some(annotations)
                }
                _ => None,
            })
        } else {
            self.last_pitched_annotations_mut()
        };
        if let Some(annotations) = target {
            set_verse(annotations, verse, syllable, style);
        }
        self
    }

    /// Associate this verse with a particular staff voice for future lyric
    /// calls, including after barlines and system breaks. Switch it again when
    /// the source underlay changes voice (`associatedVoice`).
    pub fn lyric_associated_voice(mut self, verse: u16, voice: u8) -> Self {
        assert!(verse > 0, "lyric verse numbers start at 1");
        if let Some((_, associated)) = self.lyric_voices.iter_mut().find(|(v, _)| *v == verse) {
            *associated = voice;
        } else {
            self.lyric_voices.push((verse, voice));
        }
        self
    }

    /// Attach a syllable explicitly to the most recent note/chord of `voice`,
    /// without changing this verse's persistent voice association.
    pub fn lyric_verse_on_voice(
        mut self,
        voice: u8,
        verse: u16,
        syllable: LyricSyllable,
        style: LyricStyle,
    ) -> Self {
        assert!(verse > 0, "lyric verse numbers start at 1");
        if let Some(ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }) =
            last_rhythmic_event(&mut self.current_events, Some(voice))
        {
            set_verse(annotations, verse, syllable, style);
        }
        self
    }

    /// Attach an articulation (staccato, tenuto, accent, marcato, staccatissimo,
    /// any member of the fermata family — standard, long/short, very-long/
    /// very-short, Henze long/short — a bow stroke (up-bow / down-bow), or a
    /// combined-articulation glyph: accent-staccato, marcato-staccato,
    /// tenuto-staccato (portato), or tenuto-accent) to the most recently
    /// added note, chord or rest.
    ///
    /// Placement (above/below) is determined automatically from stem direction.
    /// Fermatas and bow strokes are placed above rests and most noteheads.
    /// Combined articulations follow the standard stem-opposite rule.
    pub fn articulation(self, artic: Articulation) -> Self {
        self.articulation_mark(artic.into())
    }

    /// Attach a built-in or custom articulation mark to the most recently
    /// added note, chord or rest. [`ArticulationMark::custom`] accepts a SMuFL
    /// glyph; [`ArticulationMark::broad_mark`] draws a line-and-block mark.
    /// Marks can be parenthesized or forced above/below. On each side they
    /// stack outward alongside other marks on the same event.
    pub fn articulation_mark(mut self, mark: ArticulationMark) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.articulations.push(mark);
        }
        self
    }

    /// Attach an ornament (trill, mordent, turn, etc.) to the most recently
    /// added note or chord.
    ///
    /// Ornaments are placed above the staff, centered on the note. Unlike
    /// articulations, ornaments do not flip based on stem direction.
    /// No-op if the last event was a rest.
    pub fn ornament(mut self, orn: Ornament) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(orn);
        }
        self
    }

    /// Attach a trill ornament with a wavy-line extension to the most recently
    /// added note or chord. The wiggle continues from the "tr" glyph to the
    /// next note in the system, indicating a sustained trill across the
    /// originating note's full duration.
    ///
    /// Equivalent to `.ornament(Ornament::Trill)` plus the `trill_extension`
    /// annotation flag. No-op if the last event was a rest. If there is no
    /// following note in the same system, only the "tr" glyph is drawn (the
    /// extension silently disappears — convention is that the trill simply
    /// ends with the note).
    pub fn trill_with_extension(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
        }
        self
    }

    /// Attach a bracket-form trill (trill + wavy-line extension + vertical
    /// hook(s) at the start, end, or both ends of the wiggle) to the most
    /// recently added note or chord. The bracket form makes the precise range
    /// of a sustained trill unambiguous — Behind Bars: "where the duration of
    /// a trill must be precisely defined, the wavy line is bracketed at one
    /// or both ends."
    ///
    /// Equivalent to `.trill_with_extension()` plus a `trill_bracket`
    /// annotation. No-op if the last event was a rest. The three flags
    /// (`ornament == Trill`, `trill_extension == true`, `trill_bracket ==
    /// Some(...)`) are coupled at the API surface so users can't request a
    /// bracket on something that isn't a trill-with-extension.
    ///
    /// For cross-system trills, a `Both` bracket places the start hook on
    /// the source system (with the outgoing wiggle) and the end hook on the
    /// target system (with the incoming wiggle), so the bracket frames the
    /// trill's semantic range rather than the per-system wiggle fragments.
    pub fn trill_with_extension_bracketed(
        mut self,
        side: crate::layout::trill_bracket::TrillBracketSide,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_bracket = Some(side);
        }
        self
    }

    /// Attach a bracket-form trill with explicit hook direction and length.
    ///
    /// Same effect as [`trill_with_extension_bracketed`](Self::trill_with_extension_bracketed)
    /// plus user-specified overrides for the hook's vertical direction and
    /// length. `direction = HookDirection::Down` is the conventional default
    /// (hook points back toward the staff); pass `HookDirection::Up` when the
    /// trill is rendered below the staff. `length_ss` is the hook length in
    /// staff spaces and replaces the default ~0.75ss; reasonable values are
    /// 0.5..=1.0.
    ///
    /// No-op if the last event was a rest. Sets all four coupled annotations:
    /// `ornament == Trill`, `trill_extension == true`, `trill_bracket == Some(side)`,
    /// `trill_bracket_direction == Some(direction)`, `trill_bracket_length_ss
    /// == Some(length_ss)`.
    pub fn trill_with_extension_bracketed_custom(
        mut self,
        side: crate::layout::trill_bracket::TrillBracketSide,
        direction: crate::layout::trill_bracket::HookDirection,
        length_ss: f64,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_bracket = Some(side);
            annotations.trill_bracket_direction = Some(direction);
            annotations.trill_bracket_length_ss = Some(length_ss);
        }
        self
    }

    /// Attach a bracket-form trill using a [`TrillBracketOptions`] bundle.
    ///
    /// Ergonomic alternative to
    /// [`trill_with_extension_bracketed_custom`](Self::trill_with_extension_bracketed_custom)
    /// when the caller wants to override only one of the two custom knobs
    /// (direction or length) without restating the other. Any optional
    /// field left as `None` in the options falls back to the renderer's
    /// conventional default (`HookDirection::Down`, ~0.75 staff-space length).
    ///
    /// Concretely, `trill_with_extension_bracketed_with_options(TrillBracketOptions::new(side))`
    /// is byte-equivalent to `trill_with_extension_bracketed(side)`, and
    /// `trill_with_extension_bracketed_with_options(opts)` with both optional
    /// fields populated is byte-equivalent to the matching
    /// `trill_with_extension_bracketed_custom` call.
    ///
    /// To bracket a compound precomposed trill, set
    /// [`TrillBracketOptions::ornament`](crate::layout::trill_bracket::TrillBracketOptions::ornament)
    /// to `Some(Ornament::TrillWithMordent)` via `.with_ornament(...)`. The
    /// renderer drives the wiggle's start past the full compound glyph (not
    /// just the "tr" prefix) and the bracket hooks anchor at the wiggle's
    /// edges, so the mordent suffix remains visually intact.
    ///
    /// No-op if the last event was a rest. The ornament must satisfy
    /// [`Ornament::supports_trill_extension`]; passing an unsupported
    /// ornament (e.g. `ShortTrill`, `Mordent`, a turn) makes the renderer
    /// silently drop the extension and the bracket, leaving only the
    /// ornament glyph itself.
    pub fn trill_with_extension_bracketed_with_options(
        mut self,
        opts: crate::layout::trill_bracket::TrillBracketOptions,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            // `None` ornament collapses to `Trill` here (not at the renderer)
            // because the annotation field is the source of truth for the
            // glyph + trill-extension-eligibility check downstream; keeping
            // it explicit keeps the renderer logic glyph-agnostic.
            annotations.ornament = Some(opts.ornament.unwrap_or(Ornament::Trill));
            annotations.trill_extension = true;
            annotations.trill_bracket = Some(opts.side);
            // `None` here means "use the renderer's default" — we explicitly
            // do NOT collapse `None` to `Some(default)` so a future tweak to
            // the default propagates without API churn.
            annotations.trill_bracket_direction = opts.direction;
            annotations.trill_bracket_length_ss = opts.length_ss;
            // Explicit wiggle termination length. `None` lets the wiggle
            // run to the next note or the system edge; `Some(L)` mirrors
            // `trill_with_extension_length_ss(L)` exactly (the renderer
            // reads this field independently of the bracket, so a bracketed
            // trill with an explicit length anchors its end hook at the
            // shortened terminus).
            annotations.trill_extension_length_ss = opts.extension_length_ss;
        }
        self
    }

    /// Attach a trill ornament with a speed-variant wavy-line extension to
    /// the most recently added note or chord. The wiggle is tiled with the
    /// chosen SMuFL `wiggleTrill*` glyph: faster variants pack the wiggle
    /// more densely (visually communicating a faster trill), slower variants
    /// spread it out.
    ///
    /// Equivalent to `.trill_with_extension()` but using
    /// `speed.to_glyph()` instead of the default `wiggleTrill`. The three
    /// flags (`ornament == Trill`, `trill_extension == true`,
    /// `trill_wiggle_speed == Some(speed)`) are coupled at the API surface so
    /// users can't request a speed on something that isn't a
    /// trill-with-extension. No-op if the last event was a rest.
    ///
    /// For cross-system trills, the same speed glyph is used on both the
    /// trailing wiggle (system N) and the incoming wiggle (system N+1) so
    /// the wavy line reads as one continuous mark of consistent density
    /// across the line break.
    pub fn trill_with_extension_speed(mut self, speed: TrillWiggleSpeed) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_wiggle_speed = Some(speed);
        }
        self
    }

    /// Attach a speed-variant trill extension using a
    /// [`TrillExtensionSpeedOptions`](crate::layout::trill_extension::TrillExtensionSpeedOptions)
    /// bundle.
    ///
    /// Ergonomic alternative to
    /// [`trill_with_extension_speed`](Self::trill_with_extension_speed) when
    /// the caller also wants to override the ornament glyph (for example,
    /// pairing a precomposed `TrillWithMordent` with a non-default wiggle
    /// speed). The bare `trill_with_extension_speed(speed)` builder hardcodes
    /// `Ornament::Trill`, so without this options-based variant a user
    /// wanting "compound trill at fast wiggle speed" had no API to reach the
    /// combination short of hand-constructing annotations.
    ///
    /// Concretely:
    /// `trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(speed))`
    /// is byte-equivalent to `trill_with_extension_speed(speed)`.
    /// `trill_with_extension_speed_with_options(opts.with_ornament(Ornament::TrillWithMordent))`
    /// renders a compound trill glyph followed by the chosen-speed wiggle —
    /// the wiggle starts past the full compound glyph (not just the "tr"
    /// prefix), matching the existing renderer behavior for
    /// `trill_with_mordent_with_extension`.
    ///
    /// No-op if the last event was a rest. The ornament must satisfy
    /// [`Ornament::supports_trill_extension`]; passing an unsupported
    /// ornament (e.g. `ShortTrill`, `Mordent`, a turn) makes the renderer
    /// silently drop the extension, leaving only the ornament glyph itself.
    pub fn trill_with_extension_speed_with_options(
        mut self,
        opts: crate::layout::trill_extension::TrillExtensionSpeedOptions,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            // `None` ornament collapses to `Trill` here (not at the renderer)
            // so the annotation field remains the single source of truth for
            // the glyph + trill-extension-eligibility check downstream. This
            // matches the convention established by
            // `trill_with_extension_bracketed_with_options`.
            annotations.ornament = Some(opts.ornament.unwrap_or(Ornament::Trill));
            annotations.trill_extension = true;
            annotations.trill_wiggle_speed = Some(opts.speed);
            // Explicit wiggle termination length. `None` leaves the field
            // unset (renderer uses the natural span); `Some(L)` mirrors
            // `trill_with_extension_length_ss(L)` byte-for-byte at the
            // standard speed, and clamps a non-standard-speed wiggle to
            // the requested length when both knobs are set.
            annotations.trill_extension_length_ss = opts.extension_length_ss;
        }
        self
    }

    /// Attach a trill ornament with an *explicit-length* wavy-line extension
    /// to the most recently added note or chord. The wiggle terminates after
    /// the requested number of staff spaces past the trill glyph, regardless
    /// of where the next note sits or whether the trilled note is the last
    /// in its system. Use this when the trill should visually "run out"
    /// before the next note (sub-note granularity) — e.g. a trill on a half
    /// note that the player should release midway through the held duration.
    ///
    /// The clamp is one-sided: if `length_ss` is larger than the available
    /// natural span (to the next note's left edge or to the system's right
    /// edge), the wiggle is shortened to the natural span. A non-positive
    /// `length_ss` produces no wiggle at all (the "tr" glyph is still
    /// drawn), matching the renderer's silent fail-safe for spans too short
    /// to fit a single tile.
    ///
    /// Sets three flags: `ornament == Some(Ornament::Trill)`,
    /// `trill_extension == true`, `trill_extension_length_ss ==
    /// Some(length_ss)`. No-op if the last event was a rest.
    ///
    /// **Cross-system interaction:** a positive explicit length disables
    /// cross-system propagation — the trill terminates within its source
    /// system at the requested point, even if the trilled note happens to
    /// be the last note in its system. The convention is that an explicit
    /// length specifies a definite endpoint, while the default
    /// "extend to next note" behavior is the only path that ever produces
    /// cross-system wavy lines.
    pub fn trill_with_extension_length_ss(mut self, length_ss: f64) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_extension_length_ss = Some(length_ss);
        }
        self
    }

    /// Attach a trill ornament with a *note-anchored* wavy-line extension to
    /// the most recently added note or chord. The wiggle terminates at the
    /// note `note_offset` positions past the trilled note in the system's
    /// flat note sequence — `note_offset = 1` is the immediately following
    /// note (byte-equivalent to [`trill_with_extension`](Self::trill_with_extension)),
    /// `note_offset = 2` is the note after that, and so on. Use this when
    /// the trill should visibly hold across one or more intervening notes
    /// before releasing into a specific later note, without having to
    /// compute the staff-space distance by hand the way
    /// [`trill_with_extension_length_ss`](Self::trill_with_extension_length_ss)
    /// requires.
    ///
    /// An offset that walks past the end of the system falls back to the
    /// "extend to system right edge" behavior (same as a trilled last
    /// note). Cross-system propagation is reserved for the *natural*
    /// last-note case — an explicit offset that overshoots terminates at
    /// the system edge but does NOT continue into the next system, because
    /// an explicit offset is a definite anchor request, not a
    /// "let it flow" signal.
    ///
    /// `note_offset = 0` is degenerate (the target is the trilled note
    /// itself) and produces no wiggle, matching the renderer's fail-safe
    /// for spans too short to fit one tile.
    ///
    /// Sets three flags: `ornament == Some(Ornament::Trill)`,
    /// `trill_extension == true`,
    /// `trill_extension_to_note_offset == Some(note_offset)`. No-op if the
    /// last event was a rest.
    ///
    /// **Interaction with [`trill_with_extension_length_ss`](Self::trill_with_extension_length_ss):**
    /// the explicit length wins when both are set (the length field is
    /// the more specific termination — an exact staff-space distance vs.
    /// a "stretch to note N" hint). The to-note offset still travels with
    /// the annotation, so a future caller can read it back, but it has no
    /// effect on the rendered wiggle while a positive length is in force.
    pub fn trill_with_extension_to(mut self, note_offset: usize) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_extension_to_note_offset = Some(note_offset);
        }
        self
    }

    /// Attach a precomposed trill-with-mordent ornament + wavy-line extension
    /// to the most recently added note or chord. The compound glyph
    /// (`OrnamentPrecompTrillWithMordent`) reads as "trill, then a mordent
    /// at the end"; with an extension the wiggle starts past the full
    /// compound glyph (not just the "tr" prefix), so the mordent suffix
    /// remains visually intact. Common in Baroque keyboard music.
    ///
    /// Sets both flags: `ornament == Some(Ornament::TrillWithMordent)` and
    /// `trill_extension == true`. No-op if the last event was a rest. If
    /// there is no following note in the same system, only the compound
    /// glyph is drawn (the extension silently disappears — same convention
    /// as `trill_with_extension`).
    ///
    /// Bracket and speed modifiers attached separately (`trill_bracket`,
    /// `trill_wiggle_speed`) are honored on this ornament too — they are
    /// properties of the wiggle, not the prefix glyph. The non-options
    /// bracket builders (`trill_with_extension_bracketed`,
    /// `..._bracketed_custom`) hardcode `Ornament::Trill`; for a
    /// bracket-form compound trill use
    /// [`trill_with_extension_bracketed_with_options`](Self::trill_with_extension_bracketed_with_options)
    /// with `.with_ornament(Ornament::TrillWithMordent)`.
    pub fn trill_with_mordent_with_extension(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::TrillWithMordent);
            annotations.trill_extension = true;
        }
        self
    }

    /// Attach a trill-with-extension annotation using a
    /// [`TrillExtensionFullOptions`](crate::layout::trill_options::TrillExtensionFullOptions)
    /// bundle — the unified counterpart of
    /// [`trill_with_extension_bracketed_with_options`](Self::trill_with_extension_bracketed_with_options)
    /// and
    /// [`trill_with_extension_speed_with_options`](Self::trill_with_extension_speed_with_options).
    ///
    /// The two existing options-based builders cover bracket-only and
    /// speed-only configurations; this builder covers their union, so a
    /// caller wanting *bracket + speed* together (e.g. a `Both`-bracketed
    /// `Slow` wiggle on a precomposed `TrillWithMordent`) can express the
    /// combination in a single call rather than hand-constructing the
    /// underlying annotation fields. Every option is independent.
    ///
    /// Field semantics:
    /// - `opts.bracket` — `None` skips the bracket entirely. When `Some`,
    ///   the bracket flag is set and `opts.bracket_direction` /
    ///   `opts.bracket_length_ss` are forwarded directly (still `None` if
    ///   the caller did not override them, so the renderer's defaults apply
    ///   downstream).
    /// - `opts.speed` — `None` leaves `trill_wiggle_speed` unset (renderer
    ///   uses the standard wiggle); `Some(speed)` selects a SMuFL
    ///   `wiggleTrill*` variant.
    /// - `opts.ornament` — `None` collapses to [`Ornament::Trill`] at the
    ///   annotation field (kept consistent with the other options-based
    ///   builders so the renderer's `supports_trill_extension()` check sees
    ///   the canonical glyph). `Some(Ornament::TrillWithMordent)` selects
    ///   the precomposed compound; other ornaments that don't satisfy
    ///   [`Ornament::supports_trill_extension`] make the renderer's
    ///   collector silently drop the extension and the bracket.
    /// - `opts.length_ss` — `None` (the default) lets the wiggle extend to
    ///   the next note or to the system's right edge. `Some(length_ss)`
    ///   clamps the wiggle to terminate `length_ss` staff spaces past its
    ///   natural start, mirroring
    ///   [`trill_with_extension_length_ss`](Self::trill_with_extension_length_ss).
    ///   Overruns are clamped to the natural span; non-positive values
    ///   suppress the wiggle. A positive explicit length disables
    ///   cross-system propagation for this trill.
    ///
    /// Byte-equivalence guarantees:
    /// - `trill_with_extension_full_options(TrillExtensionFullOptions::new())`
    ///   is byte-equivalent to [`trill_with_extension`](Self::trill_with_extension).
    /// - `trill_with_extension_full_options(bracket_opts.into())` is
    ///   byte-equivalent to
    ///   `trill_with_extension_bracketed_with_options(bracket_opts)` for any
    ///   `bracket_opts: TrillBracketOptions`.
    /// - `trill_with_extension_full_options(speed_opts.into())` is
    ///   byte-equivalent to
    ///   `trill_with_extension_speed_with_options(speed_opts)` for any
    ///   `speed_opts: TrillExtensionSpeedOptions`.
    /// - `trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(L))`
    ///   is byte-equivalent to
    ///   [`trill_with_extension_length_ss(L)`](Self::trill_with_extension_length_ss)
    ///   for any `L: f64`.
    ///
    /// No-op if the last event was a rest.
    pub fn trill_with_extension_full_options(
        mut self,
        opts: crate::layout::trill_options::TrillExtensionFullOptions,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            // `None` ornament collapses to `Trill` at the builder layer (not
            // the renderer) so the annotation field stays the single source
            // of truth for glyph + extension-eligibility — matches the
            // convention established by the two single-purpose options
            // builders.
            annotations.ornament = Some(opts.ornament.unwrap_or(Ornament::Trill));
            annotations.trill_extension = true;
            // `None` bracket means "no bracket" — clear the side; the
            // direction/length overrides are tied to whether a bracket is
            // set, so they propagate downstream where the renderer would
            // ignore them in the bracket-absent case anyway.
            annotations.trill_bracket = opts.bracket;
            annotations.trill_bracket_direction = opts.bracket_direction;
            annotations.trill_bracket_length_ss = opts.bracket_length_ss;
            // `None` speed leaves the field unset — the renderer's existing
            // `unwrap_or_default()` picks `Standard`, matching the bare
            // `trill_with_extension()` output.
            annotations.trill_wiggle_speed = opts.speed;
            // `None` length_ss leaves the field unset so the renderer falls
            // back to the natural-span behavior; `Some(L)` mirrors
            // `trill_with_extension_length_ss(L)` byte-for-byte. The
            // renderer's existing clamping (overruns to natural span,
            // non-positive → no wiggle, cross-system suppression for
            // positive lengths) handles all edge cases without further
            // intervention here.
            annotations.trill_extension_length_ss = opts.length_ss;
            // `None` ramp leaves the multi-speed path disengaged — the
            // renderer falls back to single-speed wiggle using
            // `trill_wiggle_speed`. `Some(spec)` populates the annotation;
            // the renderer's wiggle dispatch checks for this field before
            // selecting a path. The spec is stored raw (no validation) —
            // a degenerate spec produces `None` at draw time and the
            // renderer's existing "no wiggle" fail-safe handles it.
            annotations.trill_speed_ramp = opts.speed_ramp;
        }
        self
    }

    /// Attach a multi-speed trill extension using a [`TrillSpeedRamp`] and
    /// region count. Convenience wrapper for the most common multi-speed
    /// configuration: a single ornament (Trill), no bracket, no explicit
    /// length, and the multi-speed renderer path engaged via a freshly
    /// constructed [`TrillSpeedRampSpec`].
    ///
    /// Byte-equivalent to:
    /// ```ignore
    /// trill_with_extension_full_options(
    ///     TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(ramp, region_count)
    /// )
    /// ```
    ///
    /// This is the multi-speed counterpart of
    /// [`trill_with_extension_speed`](Self::trill_with_extension_speed). The
    /// single-speed builder hardcodes `Ornament::Trill` and accepts a single
    /// `TrillWiggleSpeed`; this one accepts a ramp (constant or linear) plus
    /// a region count. Callers wanting to combine a ramp with a bracket, an
    /// alternative ornament, or an explicit length should use
    /// [`trill_with_extension_full_options`](Self::trill_with_extension_full_options)
    /// with the full options bundle directly.
    ///
    /// A degenerate spec — `region_count == 0`, or `Linear` with
    /// `region_count == 1` — produces no wiggle at draw time, matching the
    /// renderer's existing fail-safe. The annotation fields still get set;
    /// the dispatch silently falls through to "no wiggle, only the trill
    /// glyph drawn." No-op if the last event was a rest.
    pub fn trill_with_extension_speed_ramp(
        mut self,
        ramp: crate::layout::trill_extension::TrillSpeedRamp,
        region_count: usize,
    ) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_speed_ramp = Some(
                crate::layout::trill_extension::TrillSpeedRampSpec::new(ramp, region_count),
            );
        }
        self
    }

    /// Attach a navigation sign (segno, coda) to the most recently added note,
    /// chord or rest. The sign glyph is placed above the staff, centered on
    /// the event.
    pub fn navigation_sign(mut self, sign: NavigationSign) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.navigation_sign = Some(sign);
        }
        self
    }

    /// Start an ottava bracket (8va, 8vb, 15ma, 15mb) at the most recently
    /// added note or chord. The bracket extends until `.ottava_end()` is called.
    ///
    /// No-op if the last event was a rest.
    pub fn ottava_start(mut self, kind: OttavaKind) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ottava_start = Some(kind);
        }
        self
    }

    /// End an ottava bracket at the most recently added note or chord.
    ///
    /// No-op if the last event was a rest.
    pub fn ottava_end(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.ottava_end = true;
        }
        self
    }

    /// Open an analysis bracket at the most recently added note, chord or
    /// rest. Beamed and tuplet members are ordinary events and can anchor it.
    /// A barline does not clear an open bracket; close it on a later event.
    pub fn analysis_bracket_start(mut self, spec: AnalysisBracketSpec) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.analysis_bracket_start = Some(spec);
        }
        self
    }

    /// Close the latest open analysis bracket on this voice at the most
    /// recently added note, chord or rest.
    pub fn analysis_bracket_end(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.analysis_bracket_end = true;
        }
        self
    }

    /// Attach a pedal-down ("Ped.") marking to the most recently added note,
    /// chord or rest. The SMuFL "keyboardPedalPed" glyph is placed below the
    /// staff, well below dynamics, text scripts and lyrics.
    pub fn pedal_down(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.pedal = Some(PedalMark::Down);
        }
        self
    }

    /// Attach a pedal-up ("*") marking to the most recently added note,
    /// chord or rest. The SMuFL "keyboardPedalUp" glyph is placed below the
    /// staff at the same vertical position as pedal-down markings.
    pub fn pedal_up(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.pedal = Some(PedalMark::Up);
        }
        self
    }

    /// Attach a half-pedal marking to the most recently added note, chord or
    /// rest.
    /// Half-pedaling is a partial sustain pedal depression that retains some
    /// resonance while clearing accumulated overtones. The SMuFL
    /// "keyboardPedalHalf" glyph is placed below the staff at the same
    /// vertical position as the standard pedal markings.
    pub fn pedal_half(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.pedal = Some(PedalMark::Half);
        }
        self
    }

    /// Attach a sostenuto-pedal ("Sost.") marking to the most recently added
    /// note, chord or rest.
    ///
    /// Sostenuto is the middle pedal on a grand piano: it sustains only the
    /// notes already held when depressed. The SMuFL "keyboardPedalSost" glyph
    /// is placed below the staff at the same vertical position as the
    /// sustain pedal markings.
    pub fn pedal_sost(mut self) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.pedal = Some(PedalMark::Sost);
        }
        self
    }

    /// Attach tremolo slashes (1–3) to the most recently added note or chord.
    ///
    /// The slashes are drawn across the stem of the note/chord. Single slash =
    /// eighth-note subdivision, double = sixteenth, triple = thirty-second.
    /// No-op if the last event was a rest.
    pub fn tremolo(mut self, count: TremoloCount) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.tremolo = Some(count);
        }
        self
    }

    /// Attach an arpeggio (rolled chord) marking to the most recently added
    /// note or chord. A wavy vertical line is drawn to the left of the
    /// noteheads, indicating the chord should be played as a roll.
    ///
    /// `ArpeggioDirection::Up` (default, low to high) uses the ArpeggiatoUp glyph;
    /// `ArpeggioDirection::Down` uses ArpeggiatoDown.
    ///
    /// No-op if the last event was a rest.
    pub fn arpeggio(mut self, direction: ArpeggioDirection) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.arpeggio = Some(direction);
        }
        self
    }

    /// Attach a breath mark (comma, tick, or caesura) to the most recently
    /// added note, chord or rest. The mark is placed above the staff, to the
    /// right of the event, indicating a brief pause or lift.
    pub fn breath_mark(mut self, mark: BreathMark) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.breath_mark = Some(mark);
            annotations.breath_mark_parenthesized = false;
        }
        self
    }

    /// Attach a breath mark enclosed in parentheses to the most recently
    /// added note or chord (e.g. an editorial "(,)"), placed like
    /// [`Self::breath_mark`].
    ///
    /// No-op if the last event was a rest.
    pub fn parenthesized_breath_mark(mut self, mark: BreathMark) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.breath_mark = Some(mark);
            annotations.breath_mark_parenthesized = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the start of a glissando
    /// line to the next note. The diagonal line is drawn between the two notes
    /// during system rendering. No-op if the last event is a rest.
    pub fn glissando(mut self, style: GlissandoStyle) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.glissando_start = Some(style);
        }
        self
    }

    /// Engrave grace notes before the most recently added note or chord.
    ///
    /// The group is drawn small ([`crate::layout::grace::GRACE_NOTE_SCALE`])
    /// from real noteheads, stems, flags, and accidentals: two or more
    /// eighths (or shorter) are beamed, an acciaccatura slashes its first
    /// stem, and [`GraceNotes::slur`] adds a slur from the first grace note
    /// to the principal. Grace accidentals are resolved through the measure's
    /// accidental state just before the principal's onset, and the group's
    /// width is reserved before the principal. Replaces any grace notes
    /// already attached. No-op if the last event was a rest.
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::clef::Clef;
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::note::Note;
    /// use music::note::pitch::Pitch;
    /// use music_engraver::layout::grace::{GraceNoteKind, GraceNotes};
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// // LilyPond: \grace { aes,8[ ees,8] } c8.
    /// let svg = ScoreBuilder::new()
    ///     .clef(Clef::Bass)
    ///     .note(Pitch::new(Note::C, 3), Duration::new(music::notation::rhythm::duration::DurationKind::Eighth, 1))
    ///     .grace_notes(
    ///         GraceNotes::new(GraceNoteKind::Appoggiatura)
    ///             .note(Pitch::new(Note::Aes, 2), Duration::EIGHTH)
    ///             .note(Pitch::new(Note::Ees, 2), Duration::EIGHTH),
    ///     )
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn grace_notes(mut self, graces: GraceNotes) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.grace_notes = Some(graces);
        }
        self
    }

    /// Enclose every notehead of the most recently added note or chord, with
    /// its accidental, in parentheses (LilyPond `\parenthesize`). The
    /// parentheses' width is reserved. No-op if the last event was a rest.
    pub fn parenthesize(mut self) -> Self {
        match self
            .current_events
            .iter_mut()
            .rev()
            .find(|(_, event)| !matches!(event, ScoreEvent::GroupMark(_)))
        {
            Some((_, ScoreEvent::Note { annotations, .. })) => {
                annotations.parenthesized_noteheads = vec![true];
            }
            Some((
                _,
                ScoreEvent::Chord {
                    pitches,
                    annotations,
                    ..
                },
            )) => {
                annotations.parenthesized_noteheads = vec![true; pitches.len()];
            }
            _ => {}
        }
        self
    }

    /// Enclose the augmentation dots of the most recently added note or chord
    /// in parentheses (LilyPond `Dots.parenthesized`). No-op if the last
    /// event was a rest.
    pub fn parenthesize_dots(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.parenthesized_dots = true;
        }
        self
    }

    /// Engrave the most recently added note or chord at `size` (e.g.
    /// [`NoteSize::Cue`] for a small-note layer, LilyPond `\tiny`): its
    /// noteheads, accidentals, dots, stem, flags, and their spacing rods all
    /// scale. No-op if the last event was a rest.
    pub fn note_size(mut self, size: NoteSize) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.size = size;
        }
        self
    }

    /// Omit the stem and flags of the most recently added note or chord
    /// (LilyPond `\once \omit Stem`). No-op if the last event was a rest.
    pub fn hide_stem(mut self) -> Self {
        if let Some(annotations) = self.last_pitched_annotations_mut() {
            annotations.stem = StemVisibility::Hidden;
        }
        self
    }

    /// Engrave every note and chord of the score without stems or flags
    /// (LilyPond `\omit Stem` + `\omit Flag` for the whole staff): stemless
    /// formulas and chord series. Durations still drive notehead shapes and
    /// spacing.
    pub fn stemless(mut self) -> Self {
        self.stemless = true;
        self
    }

    /// Choose how accidentals carry within a measure for this staff (see
    /// [`AccidentalPolicy`]); the default keeps an alteration in force until
    /// the barline, [`AccidentalPolicy::Forget`] judges every note against
    /// the key signature alone (LilyPond `\accidentalStyle forget`).
    pub fn accidental_policy(mut self, policy: AccidentalPolicy) -> Self {
        self.accidental_policy = policy;
        self
    }

    /// Attach a chord symbol above the staff at the most recently added note,
    /// chord or rest (e.g. "Cmaj7", "Am", "G7", "F#dim"). Chord symbols are
    /// rendered in bold above the staff, centered on the event.
    pub fn chord_symbol(mut self, symbol: impl Into<String>) -> Self {
        let s = Some(symbol.into());
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.chord_symbol = s;
        }
        self
    }

    /// Add a chord (multiple simultaneous pitches) to the current measure.
    ///
    /// All notes in the chord share the same duration. Noteheads that are a
    /// second apart are automatically offset to avoid collision.
    pub fn chord(mut self, pitches: Vec<Pitch>, duration: Duration) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::Chord {
                pitches,
                duration,
                annotations: NoteAnnotations::default(),
            },
        ));
        self
    }

    /// Add a chord with one [`AccidentalDisplay`] policy per pitch.
    ///
    /// `displays` is parallel to `pitches`; missing entries are
    /// [`AccidentalDisplay::Auto`]. Engraved accidentals, parenthesized ones
    /// included, stack into non-colliding columns left of the chord.
    pub fn chord_with_accidentals(
        self,
        pitches: Vec<Pitch>,
        duration: Duration,
        displays: Vec<AccidentalDisplay>,
    ) -> Self {
        self.chord_annotated(
            pitches,
            duration,
            NoteAnnotations {
                accidental_displays: displays,
                ..NoteAnnotations::default()
            },
        )
    }

    pub(crate) fn note_annotated(
        mut self,
        pitch: Pitch,
        duration: Duration,
        annotations: NoteAnnotations,
    ) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::Note {
                pitch,
                duration,
                annotations,
            },
        ));
        self
    }

    pub(crate) fn chord_annotated(
        mut self,
        pitches: Vec<Pitch>,
        duration: Duration,
        annotations: NoteAnnotations,
    ) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::Chord {
                pitches,
                duration,
                annotations,
            },
        ));
        self
    }

    /// Open a beam span in the current voice: every note, chord, and rest
    /// added until the matching [`Self::end_beam`] is beamed together, with
    /// one automatically chosen stem direction. Members are ordinary events,
    /// so every annotation builder (tie, slur, dynamic, lyric, …) applies to
    /// them as usual. A beam may cross a barline, and may overlap or sit
    /// inside a tuplet span. Rests are covered by the beam; notes and chords
    /// must be eighths or shorter.
    ///
    /// Invalid spans (a beam begun inside an open beam, a beam never ended,
    /// an unbeamable member, …) make the render entry points return
    /// [`crate::error::EngraverError::Group`].
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::pitch::Pitch;
    /// use music::note::note::Note;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// let svg = ScoreBuilder::new()
    ///     .begin_beam()
    ///     .note(Pitch::new(Note::E, 4), Duration::EIGHTH)
    ///     .slur_start()
    ///     .rest(Duration::EIGHTH)
    ///     .chord(vec![Pitch::new(Note::G, 4), Pitch::new(Note::B, 4)], Duration::EIGHTH)
    ///     .slur_end()
    ///     .end_beam()
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn begin_beam(self) -> Self {
        self.begin_beam_with(BeamSpec::default())
    }

    /// Open a beam span with explicit options: a forced stem direction for
    /// every member (`\stemUp` / `\stemDown`) or secondary-beam subdivision
    /// (`subdivideBeams`). See [`Self::begin_beam`].
    pub fn begin_beam_with(self, spec: BeamSpec) -> Self {
        self.push_group_mark(GroupMark::BeamStart {
            spec,
            continued: false,
        })
    }

    /// Close the beam span opened by [`Self::begin_beam`] in the current voice.
    pub fn end_beam(self) -> Self {
        self.push_group_mark(GroupMark::BeamEnd { continues: false })
    }

    /// Open a tuplet span in the current voice: the notes, chords, and rests
    /// added until the matching [`Self::end_tuplet`] are performed at
    /// `spec.number : spec.in_time_of` and engraved with the spec's number,
    /// bracket, and placement. A tuplet is not beamed by itself — open a beam
    /// span inside (or around) it for that — and may nest inside another
    /// tuplet or cross a barline. Members may mix durations and kinds.
    ///
    /// # Example
    /// ```no_run
    /// use music::notation::rhythm::duration::Duration;
    /// use music::note::pitch::Pitch;
    /// use music::note::note::Note;
    /// use music_engraver::layout::group::TupletSpec;
    /// use music_engraver::score::ScoreBuilder;
    ///
    /// // `\tuplet 3/2 { r8 b8 b8 }`: an unbeamed triplet with a rest member.
    /// let svg = ScoreBuilder::new()
    ///     .begin_tuplet(TupletSpec::new(3, 2))
    ///     .rest(Duration::EIGHTH)
    ///     .note(Pitch::new(Note::B, 4), Duration::EIGHTH)
    ///     .note(Pitch::new(Note::B, 4), Duration::EIGHTH)
    ///     .end_tuplet()
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn begin_tuplet(self, spec: TupletSpec) -> Self {
        self.push_group_mark(GroupMark::TupletStart {
            spec,
            continued: false,
        })
    }

    /// Close the innermost tuplet span open in the current voice.
    pub fn end_tuplet(self) -> Self {
        self.push_group_mark(GroupMark::TupletEnd { continues: false })
    }

    fn push_group_mark(mut self, mark: GroupMark) -> Self {
        self.current_events
            .push((self.current_voice, ScoreEvent::GroupMark(mark)));
        self
    }

    /// Force the stem direction of the most recently added note or chord
    /// (`\stemUp` / `\stemDown` on one event). Inside a beam, the beam's
    /// direction comes from its spec first, then from its first member with a
    /// forced direction.
    pub fn stem_direction(mut self, direction: StemDirection) -> Self {
        if let Some(annotations) = self.last_annotations_mut() {
            annotations.stem_direction = Some(direction);
        }
        self
    }

    /// Add a beam group: sugar for [`Self::begin_beam`], one
    /// [`Self::note`] per entry, and [`Self::end_beam`]. Every note must be an
    /// eighth or shorter.
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
    ///         (Pitch::new(Note::E, 4), Duration::EIGHTH),
    ///         (Pitch::new(Note::F, 4), Duration::EIGHTH),
    ///         (Pitch::new(Note::G, 4), Duration::EIGHTH),
    ///         (Pitch::new(Note::A, 4), Duration::EIGHTH),
    ///     ])
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn beam_group(self, notes: Vec<(Pitch, Duration)>) -> Self {
        self.grouped_notes(None, true, Self::auto_display(notes))
    }

    /// Add a beam group whose notes each carry an [`AccidentalDisplay`] policy.
    ///
    /// Identical to [`Self::beam_group`] for [`AccidentalDisplay::Auto`] members.
    pub fn beam_group_with_accidentals(
        self,
        notes: Vec<(Pitch, Duration, AccidentalDisplay)>,
    ) -> Self {
        self.grouped_notes(None, true, notes)
    }

    /// Add a tuplet whose number is also its performed ratio's normal-note
    /// count (`tuplet_number : tuplet_number`, i.e. written durations keep
    /// their spacing). Sugar for [`Self::tuplet_ratio`]; callers that know the
    /// performed ratio use that instead.
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
    ///         (Pitch::new(Note::E, 4), Duration::EIGHTH),
    ///         (Pitch::new(Note::F, 4), Duration::EIGHTH),
    ///         (Pitch::new(Note::G, 4), Duration::EIGHTH),
    ///     ])
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn tuplet(self, tuplet_number: u32, notes: Vec<(Pitch, Duration)>) -> Self {
        self.tuplet_ratio(tuplet_number, tuplet_number, notes)
    }

    /// Add a `tuplet_number : in_time_of` tuplet of notes: sugar for
    /// [`Self::begin_tuplet`] with [`TupletSpec::new`], the notes, and
    /// [`Self::end_tuplet`]. When there are at least two notes and all are
    /// eighths or shorter, they are also beamed together (a beam span inside
    /// the tuplet); otherwise they stay unbeamed. Use the span API for any
    /// other combination (rests, chords, partial beams, custom appearance).
    pub fn tuplet_ratio(
        self,
        tuplet_number: u32,
        in_time_of: u32,
        notes: Vec<(Pitch, Duration)>,
    ) -> Self {
        self.tuplet_ratio_with_accidentals(tuplet_number, in_time_of, Self::auto_display(notes))
    }

    /// Add a `tuplet_number:in_time_of` tuplet whose notes each carry an
    /// [`AccidentalDisplay`] policy.
    ///
    /// Identical to [`Self::tuplet_ratio`] for [`AccidentalDisplay::Auto`] members.
    pub fn tuplet_ratio_with_accidentals(
        self,
        tuplet_number: u32,
        in_time_of: u32,
        notes: Vec<(Pitch, Duration, AccidentalDisplay)>,
    ) -> Self {
        let beamed = notes.len() >= 2
            && notes
                .iter()
                .all(|(_, duration, _)| event::duration_kind_to_log2(duration.kind()) >= 3);
        self.grouped_notes(
            Some(TupletSpec::new(tuplet_number, in_time_of)),
            beamed,
            notes,
        )
    }

    fn auto_display(notes: Vec<(Pitch, Duration)>) -> Vec<(Pitch, Duration, AccidentalDisplay)> {
        notes
            .into_iter()
            .map(|(pitch, duration)| (pitch, duration, AccidentalDisplay::Auto))
            .collect()
    }

    /// Notes inside an optional tuplet span and, when `beamed`, a beam span.
    fn grouped_notes(
        mut self,
        tuplet: Option<TupletSpec>,
        beamed: bool,
        notes: Vec<(Pitch, Duration, AccidentalDisplay)>,
    ) -> Self {
        if let Some(spec) = tuplet {
            self = self.begin_tuplet(spec);
        }
        if beamed {
            self = self.begin_beam();
        }
        for (pitch, duration, display) in notes {
            self = self.note_with_accidental(pitch, duration, display);
        }
        if beamed {
            self = self.end_beam();
        }
        if tuplet.is_some() {
            self = self.end_tuplet();
        }
        self
    }

    /// Add a rest to the current measure.
    pub fn rest(mut self, duration: Duration) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::Rest {
                duration,
                annotations: NoteAnnotations::default(),
            },
        ));
        self
    }

    /// Add an invisible spacer (LilyPond `s`): it takes `duration` and
    /// horizontal room like a rest but draws nothing. A measure holding only
    /// spacers renders as an empty bar, at least
    /// [`MeasureLayoutConfig::empty_measure_min_width`] wide.
    pub fn spacer(mut self, duration: Duration) -> Self {
        self.current_events
            .push((self.current_voice, ScoreEvent::Spacer { duration }));
        self
    }

    /// Add a multi-measure rest filling the current measure.
    ///
    /// Renders as a single measure-shaped frame containing an H-bar (thick
    /// horizontal bar with vertical serifs at each end, centered on the middle
    /// staff line) and a count number above it. `count` is the number of
    /// consecutive measures of rest this frame represents — engraved parts use
    /// this convention to compress empty passages.
    ///
    /// Conventionally a multi-measure rest is the only event in its measure;
    /// adding notes or other rests alongside it produces undefined visual
    /// output. Call `.barline()` (or `.end_barline()`) immediately after.
    ///
    /// A count of `0` is treated identically to `1` by the renderer — the
    /// label simply reads "0", which is meaningless but harmless. Callers
    /// should pass `count >= 1`.
    pub fn multi_measure_rest(mut self, count: u32) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::MultiMeasureRest {
                count,
                style: crate::layout::multi_measure_rest::MultiMeasureRestStyle::HBar,
            },
        ));
        self
    }

    /// Add a multi-measure rest in the older "church-rest" style.
    ///
    /// For small counts (1–4) the renderer draws combinations of SMuFL whole
    /// and breve rest glyphs on the staff instead of the modern H-bar:
    /// - **1**: a single whole rest hanging from the 4th line.
    /// - **2**: a single breve (double-whole) rest sitting on the 3rd line.
    /// - **3**: a breve + whole rest, side by side.
    /// - **4**: two breve rests side by side.
    ///
    /// A bold count number is drawn above the staff (matching the H-bar
    /// convention) so a reader can scan the count without distinguishing
    /// styles.
    ///
    /// For counts greater than [`CHURCH_REST_MAX_COUNT`](crate::layout::multi_measure_rest::CHURCH_REST_MAX_COUNT)
    /// the renderer silently falls back to the H-bar style — the church-rest
    /// cluster would otherwise span too wide and become ambiguous.
    pub fn multi_measure_rest_church(mut self, count: u32) -> Self {
        self.current_events.push((
            self.current_voice,
            ScoreEvent::MultiMeasureRest {
                count,
                style: crate::layout::multi_measure_rest::MultiMeasureRestStyle::Church,
            },
        ));
        self
    }

    /// Mark the start of a volta bracket (1st/2nd ending) on the current measure.
    ///
    /// `text` is the label displayed at the left side of the bracket (e.g. "1.", "2.",
    /// "1.–3."). Call before writing notes for the first measure of the ending.
    /// The bracket continues over subsequent measures until `.volta_end()` is called.
    ///
    /// A volta bracket has a left hook + text on the start measure, a top line on
    /// continuation measures, and a right hook on the end measure (the one where
    /// `.volta_end()` is called before the barline).
    pub fn volta_start(mut self, text: &str) -> Self {
        self.in_volta = true;
        self.volta_text = Some(text.to_string());
        self
    }

    /// Mark the current measure as the last measure of a volta bracket.
    ///
    /// The bracket's right hook is drawn at the end of this measure. Call before
    /// the barline that ends this measure.
    pub fn volta_end(mut self) -> Self {
        self.volta_ending = true;
        self
    }

    /// Resolve the volta annotation for the current measure being flushed.
    fn resolve_volta(&mut self) -> Option<VoltaAnnotation> {
        let has_text = self.volta_text.is_some();
        let ending = std::mem::take(&mut self.volta_ending);

        if has_text && ending {
            // Single-measure volta — both hooks + text
            let text = self.volta_text.take();
            self.in_volta = false;
            Some(VoltaAnnotation {
                text,
                hooks: VoltaHooks::Both,
            })
        } else if has_text {
            // First measure of multi-measure volta — left hook + text, open right
            let text = self.volta_text.take();
            Some(VoltaAnnotation {
                text,
                hooks: VoltaHooks::LeftOnly,
            })
        } else if self.in_volta && ending {
            // Last measure of multi-measure volta — right hook, close bracket
            self.in_volta = false;
            Some(VoltaAnnotation {
                text: None,
                hooks: VoltaHooks::RightOnly,
            })
        } else if self.in_volta {
            // Middle measure of multi-measure volta — top line only
            Some(VoltaAnnotation {
                text: None,
                hooks: VoltaHooks::Neither,
            })
        } else {
            // Not in a volta
            None
        }
    }

    /// End the current measure with a single barline and start a new one.
    /// Resets the active voice to 0.
    pub fn barline(mut self) -> Self {
        self.close_measure(BarlineStyle::Single);
        self
    }

    /// End the current measure with a final (double) barline.
    /// Typically called at the end of the piece. Resets the active voice to 0.
    pub fn end_barline(mut self) -> Self {
        self.close_measure(BarlineStyle::Final);
        self
    }

    /// End the current measure with a specific barline style.
    /// Resets the active voice to 0.
    ///
    /// [`BarlineStyle::Invisible`] closes the measure without drawing a
    /// barline; as the last call it ends the piece with no final barline,
    /// e.g. on an incomplete bar.
    pub fn barline_style(mut self, style: BarlineStyle) -> Self {
        self.close_measure(style);
        self
    }

    /// Close the measure in progress with `barline`, recording its events,
    /// volta, and timing directives, and start the next one in voice 0.
    fn close_measure(&mut self, barline: BarlineStyle) {
        let events = std::mem::take(&mut self.current_events);
        let volta = self.resolve_volta();
        let timing = MeasureTiming {
            partial: self.pending_partial.take(),
            length_override: self.length_override,
            cadenza: std::mem::replace(&mut self.cadenza_in_measure, self.cadenza),
        };
        self.measures.push(CompletedMeasure {
            events,
            barline,
            volta,
            timing,
        });
        self.current_voice = 0;
    }

    /// Flush any pending events as a final measure if not already flushed.
    pub(crate) fn flush_pending(&mut self) {
        if !self.current_events.is_empty() {
            self.close_measure(BarlineStyle::Final);
        }
        self.current_voice = 0;
    }

    /// Convert accumulated `ScoreEvent`s into `MeasureContent`s suitable for layout.
    ///
    /// Each measure's accidentals are resolved staff-wide in musical order
    /// across all voices (see `resolve_measure_accidentals`), against the key
    /// signature and the measure's own accidental state, which resets at
    /// every barline. When multiple voices are present, voice 0 goes in
    /// `events` and voices 1+ go in `additional_voices`. Multi-voice measures
    /// force stem directions: voice 0 = stems up, voice 1 = stems down.
    ///
    /// Clef and meter state is carried in score order. Every pitch is placed
    /// in the clef in force at its onset: the latest clef change, in any
    /// voice, at or before it. Structural changes travel in the primary
    /// voice's events at their onset; hidden meter changes leave no event but
    /// set the measure's [`MeasureMeta`].
    pub(crate) fn build_measure_contents(
        &self,
    ) -> Result<Vec<MeasureContent>, ScoreStructureError> {
        let mut clef = self.clef;
        let mut meter = self.time_signature.clone();
        let mut next_number = self.first_measure_number;
        let mut open_tuplets: Vec<Vec<TupletSpec>> = Vec::new();
        let mut contents = Vec::with_capacity(self.measures.len());
        for (index, measure) in self.measures.iter().enumerate() {
            let voiced_events = &measure.events;
            let timeline = measure_timeline(voiced_events, &open_tuplets);

            // Clef changes in onset order (builder order among equal onsets).
            let measure_start_clef = clef;
            let mut clef_changes: Vec<(event::Onset, ClefKind)> = Vec::new();
            for ((_, event), &onset) in voiced_events.iter().zip(&timeline.onsets) {
                match event {
                    ScoreEvent::ClefChange(change) => clef_changes.push((onset, change.clef)),
                    ScoreEvent::TimeSignatureChange(time_signature) => {
                        if onset != 0 {
                            return Err(ScoreStructureError::MidMeasureTimeSignatureChange {
                                measure: index,
                            });
                        }
                        meter = Some(time_signature.clone());
                    }
                    _ => {}
                }
            }
            clef_changes.sort_by_key(|&(onset, _)| onset);
            let clef_at = |onset: event::Onset| {
                clef_changes
                    .iter()
                    .take_while(|&&(change_onset, _)| change_onset <= onset)
                    .last()
                    .map_or(measure_start_clef, |&(_, kind)| kind)
            };
            if let Some(&(_, last)) = clef_changes.last() {
                clef = last;
            }

            // Determine the maximum voice index in this measure.
            let max_voice = voiced_events.iter().map(|(v, _)| *v).max().unwrap_or(0);
            let is_multi_voice = max_voice > 0;

            let resolved = resolve_measure_accidentals(
                voiced_events,
                &self.key_sig,
                &open_tuplets,
                self.accidental_policy,
            );
            groups::advance_open_tuplets(&mut open_tuplets, voiced_events);
            let mut accidentals = resolved.iter();

            // Separate events by voice. Structural changes entered in another
            // voice join the primary voice before its first event at or after
            // their onset.
            let mut voice_buckets: Vec<Vec<MeasureEvent>> =
                (0..=max_voice).map(|_| Vec::new()).collect();
            let mut primary_onsets: Vec<event::Onset> = Vec::new();
            let mut foreign_changes: Vec<(event::Onset, MeasureEvent)> = Vec::new();
            for ((voice, event), &onset) in voiced_events.iter().zip(&timeline.onsets) {
                if let ScoreEvent::TimeSignatureChange(time_signature) = event {
                    if !time_signature.visible {
                        continue;
                    }
                }
                let staff_clef = clef_at(onset).to_clef();
                let mut me = convert_resolved_event(event, &staff_clef, &mut accidentals);
                if event.is_structural() && *voice != 0 {
                    foreign_changes.push((onset, me));
                    continue;
                }
                if is_multi_voice {
                    force_stem_direction(&mut me, usize::from(*voice));
                }
                if self.stemless {
                    hide_stems(&mut me);
                }
                if *voice == 0 {
                    primary_onsets.push(onset);
                }
                voice_buckets[*voice as usize].push(me);
            }
            for (onset, change) in foreign_changes.into_iter().rev() {
                let at = primary_onsets
                    .iter()
                    .position(|&primary| primary >= onset)
                    .unwrap_or(primary_onsets.len());
                voice_buckets[0].insert(at, change);
                primary_onsets.insert(at, onset);
            }

            let anacrusis = measure.timing.partial.is_some();
            let number = if anacrusis {
                next_number - 1
            } else {
                next_number += 1;
                next_number - 1
            };
            let nominal_length = if measure.timing.cadenza {
                None
            } else {
                measure
                    .timing
                    .partial
                    .or(measure.timing.length_override)
                    .or_else(|| meter.as_ref().map(|meter| meter.kind.measure_length()))
            };
            let meta = MeasureMeta {
                number,
                meter: meter.as_ref().map(|meter| meter.kind.clone()),
                meter_visible: meter.as_ref().is_some_and(|meter| meter.visible),
                nominal_length,
                actual_length: timeline.to_length(timeline.length),
                anacrusis,
                ..MeasureMeta::default()
            };

            let primary = voice_buckets.remove(0);
            contents.push(MeasureContent {
                events: primary,
                barline: measure.barline,
                volta: measure.volta.clone(),
                additional_voices: voice_buckets,
                meta,
            });
        }
        groups::finish_group_spans(&mut contents);
        self.attach_leading_text_marks(&mut contents);
        Ok(contents)
    }

    pub(crate) fn validate_group_spans(&self) -> Result<(), GroupSpanError> {
        groups::validate_group_spans(&self.measures)
    }

    /// Draw text marks entered before any event above the score's first
    /// note, chord or rest.
    fn attach_leading_text_marks(&self, contents: &mut [MeasureContent]) {
        if self.leading_text_marks.is_empty() {
            return;
        }
        let first = contents
            .iter_mut()
            .flat_map(|content| content.events.iter_mut())
            .find_map(|event| match event {
                MeasureEvent::Note(n) => Some(&mut n.annotations),
                MeasureEvent::Chord(c) => Some(&mut c.annotations),
                MeasureEvent::Rest(r) => Some(&mut r.annotations),
                _ => None,
            });
        if let Some(annotations) = first {
            annotations
                .text_scripts
                .splice(0..0, self.leading_text_marks.iter().cloned());
        }
    }

    /// Switch the active voice for subsequent events.
    ///
    /// Voice 0 is the primary (default) voice. Voice 1 is the secondary voice.
    /// When a measure contains events from more than one voice, stem directions
    /// are forced: voice 0 stems up, voice 1 stems down. Voices 2+ default to
    /// stems up (even voices up, odd voices down).
    ///
    /// Call `voice(0)` to return to the primary voice. The voice resets to 0
    /// at each barline automatically.
    ///
    /// # Example
    /// ```no_run
    /// # use music_engraver::score::ScoreBuilder;
    /// # use music::notation::clef::Clef;
    /// # use music::notation::rhythm::duration::Duration;
    /// # use music::note::pitch::Pitch;
    /// # use music::note::note::Note;
    /// let svg = ScoreBuilder::new()
    ///     .clef(Clef::Treble)
    ///     .time_signature(4, 4)
    ///     // Voice 0: melody (stems up)
    ///     .note(Pitch::new(Note::E, 5), Duration::HALF)
    ///     .note(Pitch::new(Note::D, 5), Duration::HALF)
    ///     // Voice 1: bass (stems down)
    ///     .voice(1)
    ///     .note(Pitch::new(Note::C, 4), Duration::WHOLE)
    ///     .voice(0) // back to primary
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn voice(mut self, voice_index: u8) -> Self {
        self.current_voice = voice_index;
        self
    }

    /// Build the prefix of the score's first system before any change: the
    /// initial clef, key, and (printed) initial meter. Each system's actual
    /// prefix is derived from it with
    /// [`system_start_prefix`](crate::layout::system::system_start_prefix).
    pub(crate) fn build_prefix(&self) -> SystemPrefix {
        let time_signature = self
            .time_signature
            .as_ref()
            .filter(|meter| meter.visible)
            .map(|meter| meter.kind.clone());
        SystemPrefix::new(&self.clef.to_clef(), self.key_sig.clone(), time_signature)
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

    /// The system-breaking policy selected on this builder.
    pub(crate) fn system_breaking(&self) -> SystemBreaking {
        if self.explicit_breaks {
            SystemBreaking::Explicit
        } else if self.optimal_breaks {
            SystemBreaking::Optimal
        } else if self.auto_breaks {
            SystemBreaking::Auto
        } else {
            SystemBreaking::Fixed(self.effective_measures_per_system())
        }
    }

    /// This builder's `system_break()` / `no_break()` requests resolved
    /// against its logical measures (from
    /// [`build_measure_contents`](Self::build_measure_contents)).
    pub(crate) fn line_break_plan(&self, logical: &[MeasureContent]) -> LineBreakPlan {
        LineBreakPlan::from_requests(&self.line_break_requests, logical)
    }

    /// Flush any pending measure and lay the score out as a page: logical
    /// measures with this builder's explicit line breaks applied, broken
    /// into systems by its breaking policy. `None` when there are no measures.
    pub(crate) fn page_layout(
        &mut self,
        staff_space: f64,
    ) -> Result<Option<PageLayout>, ScoreStructureError> {
        self.flush_pending();
        if self.measures.is_empty() {
            return Ok(None);
        }

        let logical = self.build_measure_contents()?;
        let measure_contents = self.line_break_plan(&logical).apply(logical);
        let prefix = self.build_prefix();
        let measure_config = MeasureLayoutConfig::from_staff_space(staff_space);
        let sys_width = self.effective_system_width(staff_space);
        let mut page_config = PageLayoutConfig::new(staff_space, sys_width);
        page_config.measure_numbering = self.measure_numbering;

        Ok(Some(layout_page(
            &prefix,
            &measure_contents,
            &measure_config,
            &page_config,
            &self.system_breaking(),
        )))
    }

    /// Render the score to an SVG string, returning an error if font operations fail.
    ///
    /// Flushes any pending events as a final measure (with `Final` barline)
    /// if no explicit end barline was provided; to end without one, close the
    /// last measure with `barline_style(BarlineStyle::Invisible)`.
    #[must_use = "the SVG string is returned but not used"]
    pub fn try_render_svg(mut self) -> Result<String, crate::error::EngraverError> {
        self.flush_pending();
        self.validate_group_spans()?;
        let font = bravura_font();
        let config = font.engraving_config();
        let Some(page_layout) = self.page_layout(config.staff_space)? else {
            return Ok(String::from(
                "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            ));
        };
        Ok(draw_page(&font, &config, &page_layout)?.to_svg())
    }

    /// Render the score to an SVG string.
    ///
    /// Convenience wrapper around [`try_render_svg`](Self::try_render_svg) that
    /// panics on errors. For error handling, use `try_render_svg` instead.
    ///
    /// # Panics
    ///
    /// Panics on malformed beam or tuplet spans (see
    /// [`GroupSpanError`]) and if font glyph lookup fails. The latter cannot
    /// happen with the bundled Bravura font because all SMuFL glyph names used
    /// by the layout engine are present in Bravura's metadata, and the font
    /// data is compiled in via `include_bytes!`.
    #[must_use = "the SVG string is returned but not used"]
    pub fn render_svg(self) -> String {
        self.try_render_svg()
            .unwrap_or_else(|error| panic!("score failed to render: {error}"))
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

    /// Render the score to PNG at the given scale factor and write it to `path`.
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

impl Default for ScoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_accidentals;
#[cfg(test)]
mod tests_analysis_bracket;
#[cfg(test)]
mod tests_barlines;
#[cfg(test)]
mod tests_breve;
#[cfg(test)]
mod tests_c_clefs;
#[cfg(test)]
mod tests_modus_novus_struct;
#[cfg(test)]
mod tests_lyrics_verses;
#[cfg(test)]
mod tests_rest_marks;
#[cfg(test)]
mod tests_structure;
#[cfg(test)]
mod tests_written_octave;
