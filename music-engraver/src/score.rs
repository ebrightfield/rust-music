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
//!     .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
//!     .barline()
//!     .rest(Duration::WHOLE)
//!     .end_barline()
//!     .render_svg();
//! ```

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::pitch::Pitch;
use music::note::spelling::{Accidental, Spelling};

use crate::font::bravura_font;
use crate::layout::accidental::accidental_glyph;
use crate::layout::barline::BarlineStyle;
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{MeasureLayoutConfig, NoteEvent, RestEvent};
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
fn should_show_accidental(pitch: &Pitch, key_sig: &KeySignature) -> Option<smufl::Glyph> {
    let spelling = Spelling::from(&pitch.note);
    let acc = spelling.acc;
    let altered = note_altered_in_key(spelling.letter, key_sig);

    match acc {
        Accidental::Natural => {
            if altered {
                // Letter is sharped/flatted in key sig — show natural to cancel
                accidental_glyph(Accidental::Natural, true)
            } else {
                None
            }
        }
        Accidental::Sharp => {
            if altered && matches!(key_sig, KeySignature::Sharps(_)) {
                // Sharp is already in the key signature — suppress
                None
            } else {
                accidental_glyph(Accidental::Sharp, false)
            }
        }
        Accidental::Flat => {
            if altered && matches!(key_sig, KeySignature::Flats(_)) {
                // Flat is already in the key signature — suppress
                None
            } else {
                accidental_glyph(Accidental::Flat, false)
            }
        }
        // Double accidentals are never part of a key signature
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
    Note { pitch: Pitch, duration: Duration },
    Rest { duration: Duration },
}

/// Builder for constructing a score from `music` crate types and rendering to SVG.
///
/// Events are grouped into measures delimited by `barline()` / `end_barline()` calls.
/// The builder handles conversion of `Pitch` → staff position, `Duration` → log2 + dots,
/// and accidental resolution against the key signature.
#[derive(Clone, Debug)]
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
        self.current_events.push(ScoreEvent::Note { pitch, duration });
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

    /// Render the score to an SVG string.
    ///
    /// Flushes any pending events as a final measure (with `Final` barline)
    /// if no explicit end barline was provided.
    pub fn render_svg(mut self) -> String {
        // Flush any pending events
        if !self.current_events.is_empty() {
            let events = std::mem::take(&mut self.current_events);
            self.measures.push((events, BarlineStyle::Final));
        }

        if self.measures.is_empty() {
            return String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>");
        }

        let font = bravura_font();
        let config = font.engraving_config();
        let staff_space = config.staff_space;

        let clef = self.clef.to_clef();

        // Convert ScoreEvents to MeasureContent
        let measure_contents: Vec<MeasureContent> = self
            .measures
            .iter()
            .map(|(events, barline)| {
                let measure_events: Vec<MeasureEvent> = events
                    .iter()
                    .map(|e| self.convert_event(e, &clef))
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

        draw_page(&font, &config, &page_layout)
            .expect("font rendering should not fail for valid input")
            .to_svg()
    }

    /// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
    fn convert_event(&self, event: &ScoreEvent, clef: &Clef) -> MeasureEvent {
        match event {
            ScoreEvent::Note { pitch, duration } => {
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
            pitch: pitch.clone(),
            duration: Duration::QTR,
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
}
