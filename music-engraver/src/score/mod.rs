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
pub mod tab;

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
#[cfg(test)]
use music::notation::rhythm::duration::DurationKind;
use music::note::pitch::Pitch;
#[cfg(test)]
use music::note::spelling::Accidental;

use crate::font::bravura_font;
use crate::layout::arpeggio::ArpeggioDirection;
use crate::layout::articulation::Articulation;
use crate::layout::barline::BarlineStyle;
use crate::layout::breath::BreathMark;
use crate::layout::grace::GraceNoteKind;
use crate::layout::hairpin::HairpinType;
use crate::layout::dynamics::Dynamic;
use crate::layout::glissando::GlissandoStyle;
use crate::layout::key_signature::KeySignature;
use crate::layout::lyric::LyricSyllable;
use crate::layout::navigation::NavigationSign;
use crate::layout::ornament::Ornament;
use crate::layout::ottava::OttavaKind;
use crate::layout::pedal::PedalMark;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::tempo::TempoMark;
use crate::layout::tremolo::TremoloCount;
use crate::layout::trill_extension::TrillWiggleSpeed;
use crate::layout::measure::{MeasureLayoutConfig, NoteAnnotations};
use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
use crate::layout::system::{ClefKind, MeasureContent, MeasureEvent, SystemPrefix};
use crate::layout::time_signature::TimeSignatureKind;
use crate::layout::volta::{VoltaAnnotation, VoltaHooks};
use crate::render::page_renderer::draw_page;

use event::{AccidentalTracker, ScoreEvent, convert_event};
#[cfg(test)]
use event::{
    duration_kind_to_log2, effective_accidental,
    note_altered_in_key, note_key, resolve_accidental, should_show_accidental,
};

/// A completed measure: voiced events, barline style, and optional volta annotation.
pub(crate) type CompletedMeasure = (Vec<(u8, ScoreEvent)>, BarlineStyle, Option<VoltaAnnotation>);

/// Force stem direction on a `MeasureEvent` based on voice index.
///
/// Voice 0 (and even voices) get stems up; voice 1 (and odd voices) get stems down.
/// This follows standard engraving convention for two-voice writing on a single staff.
fn force_stem_direction(event: &mut MeasureEvent, voice: u8) {
    use crate::layout::stem::StemDirection;
    let dir = if voice.is_multiple_of(2) {
        StemDirection::Up
    } else {
        StemDirection::Down
    };
    match event {
        MeasureEvent::Note(n) => n.stem_direction = Some(dir),
        MeasureEvent::Chord(c) => c.stem_direction = Some(dir),
        MeasureEvent::BeamGroup(bg) => bg.stem_direction = Some(dir),
        MeasureEvent::TupletGroup(tg) => tg.beam_group.stem_direction = Some(dir),
        // Multi-measure rests and ordinary rests have no stem to flip.
        MeasureEvent::Rest(_) | MeasureEvent::MultiMeasureRest { .. } => {}
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
    clef: ClefKind,
    key_sig: KeySignature,
    time_sig: Option<(u8, u8)>,
    /// Override for time signature display style (Common/CutCommon).
    /// When None and time_sig is Some, uses Numeric display.
    time_sig_kind: Option<TimeSignatureKind>,
    /// Events accumulated for the current (in-progress) measure.
    /// Each entry is `(voice_index, event)` where voice 0 is the primary voice.
    current_events: Vec<(u8, ScoreEvent)>,
    /// Completed measures.
    pub(crate) measures: Vec<CompletedMeasure>,
    /// Active voice index (0 = primary). Voice 0 stems follow auto-detection;
    /// when multiple voices are present, voice 0 forces stems up, voice 1 forces
    /// stems down.
    current_voice: u8,
    /// Measures per system (for line breaking). 0 = auto (4 per system).
    pub(crate) measures_per_system: usize,
    /// System width in font design units. 0 = auto.
    pub(crate) system_width: f64,
    /// Use automatic width-based line breaking instead of fixed measures per system.
    pub(crate) auto_breaks: bool,
    /// Use optimal (Knuth-Plass style DP) line breaking instead of greedy.
    pub(crate) optimal_breaks: bool,
    /// Display measure numbers above the start of each system.
    pub(crate) show_measure_numbers: bool,
    /// Whether we are currently inside a volta bracket region.
    in_volta: bool,
    /// Text label for the current volta bracket (set on `.volta_start()`).
    volta_text: Option<String>,
    /// Whether `.volta_end()` was called on the current measure (consumed at barline).
    volta_ending: bool,
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
            current_voice: 0,
            measures_per_system: 4,
            system_width: 0.0,
            auto_breaks: false,
            optimal_breaks: false,
            show_measure_numbers: false,
            in_volta: false,
            volta_text: None,
            volta_ending: false,
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
        self.auto_breaks = false;
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
        self
    }

    /// Set the system width in font design units.
    /// If not set (or 0), a default of 40 staff spaces is used.
    pub fn system_width_fu(mut self, width: f64) -> Self {
        self.system_width = width;
        self
    }

    /// Show measure numbers above the start of each system.
    ///
    /// When enabled, each system displays the 1-based measure number of its
    /// first bar above the staff, left-aligned with the start of the note
    /// content (after the prefix: clef, key/time signature).
    pub fn show_measure_numbers(mut self) -> Self {
        self.show_measure_numbers = true;
        self
    }

    /// Add a note to the current measure.
    pub fn note(mut self, pitch: Pitch, duration: Duration) -> Self {
        self.current_events.push((self.current_voice, ScoreEvent::Note { pitch, duration, annotations: NoteAnnotations::default() }));
        self
    }

    /// Mark the most recently added note as tied forward to the next note at the
    /// same pitch. The tie curve is drawn connecting this note to the next note
    /// of the same staff position within the same system.
    ///
    /// Must be called immediately after `.note()`. Has no effect if the last event
    /// is not a note.
    pub fn tie(mut self) -> Self {
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
            annotations.slur_start = true;
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a slur.
    ///
    /// Pairs with a preceding `slur_start()` call. The slur is drawn between
    /// the most recent `slur_start` note and this note.
    pub fn slur_end(mut self) -> Self {
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
            annotations.dynamic = Some(dyn_mark);
        }
        self
    }

    /// Mark the start of a hairpin (crescendo or decrescendo wedge) at the most
    /// recently added note or chord. The wedge extends from this note to the
    /// next note/chord with `hairpin_end()`.
    pub fn hairpin_start(mut self, kind: HairpinType) -> Self {
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
            annotations.hairpin_start = Some(kind);
        }
        self
    }

    /// Mark the most recently added note or chord as the end of a hairpin wedge.
    pub fn hairpin_end(mut self) -> Self {
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
        if let Some((_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. })) = self.current_events.last_mut() {
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
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.lyric = Some(syllable);
        }
        self
    }

    /// Attach an articulation (staccato, tenuto, accent, marcato, staccatissimo,
    /// or any member of the fermata family — standard, long/short, very-long/
    /// very-short, Henze long/short) to the most recently added note or chord.
    ///
    /// Placement (above/below) is determined automatically from stem direction.
    /// All fermata variants are always placed above. No-op if the last event
    /// was a rest.
    pub fn articulation(mut self, artic: Articulation) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.articulations.push(artic);
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.ornament = Some(Ornament::Trill);
            annotations.trill_extension = true;
            annotations.trill_wiggle_speed = Some(speed);
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.ornament = Some(Ornament::TrillWithMordent);
            annotations.trill_extension = true;
        }
        self
    }

    /// Attach a navigation sign (segno, coda) to the most recently added note
    /// or chord. The sign glyph is placed above the staff, centered on the note.
    ///
    /// No-op if the last event was a rest.
    pub fn navigation_sign(mut self, sign: NavigationSign) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.navigation_sign = Some(sign);
        }
        self
    }

    /// Start an ottava bracket (8va, 8vb, 15ma, 15mb) at the most recently
    /// added note or chord. The bracket extends until `.ottava_end()` is called.
    ///
    /// No-op if the last event was a rest.
    pub fn ottava_start(mut self, kind: OttavaKind) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.ottava_start = Some(kind);
        }
        self
    }

    /// End an ottava bracket at the most recently added note or chord.
    ///
    /// No-op if the last event was a rest.
    pub fn ottava_end(mut self) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.ottava_end = true;
        }
        self
    }

    /// Attach a pedal-down ("Ped.") marking to the most recently added note or chord.
    ///
    /// The SMuFL "keyboardPedalPed" glyph is placed below the staff, well below
    /// dynamics, expression text, and lyrics. No-op if the last event was a rest.
    pub fn pedal_down(mut self) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.pedal = Some(PedalMark::Down);
        }
        self
    }

    /// Attach a pedal-up ("*") marking to the most recently added note or chord.
    ///
    /// The SMuFL "keyboardPedalUp" glyph is placed below the staff at the same
    /// vertical position as pedal-down markings. No-op if the last event was a rest.
    pub fn pedal_up(mut self) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.pedal = Some(PedalMark::Up);
        }
        self
    }

    /// Attach tremolo slashes (1–3) to the most recently added note or chord.
    ///
    /// The slashes are drawn across the stem of the note/chord. Single slash =
    /// eighth-note subdivision, double = sixteenth, triple = thirty-second.
    /// No-op if the last event was a rest.
    pub fn tremolo(mut self, count: TremoloCount) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
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
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.arpeggio = Some(direction);
        }
        self
    }

    /// Attach a breath mark (comma, tick, or caesura) to the most recently
    /// added note or chord. The mark is placed above the staff, to the right
    /// of the notehead, indicating a brief pause or lift before the next note.
    ///
    /// No-op if the last event was a rest.
    pub fn breath_mark(mut self, mark: BreathMark) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.breath_mark = Some(mark);
        }
        self
    }

    /// Mark the most recently added note or chord as the start of a glissando
    /// line to the next note. The diagonal line is drawn between the two notes
    /// during system rendering. No-op if the last event is a rest.
    pub fn glissando(mut self, style: GlissandoStyle) -> Self {
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.glissando_start = Some(style);
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
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.grace_note = Some((staff_pos, kind));
        }
        self
    }

    /// Attach a grace note to the most recently added note or chord, drawing a
    /// connecting slur from the grace to the principal note.
    ///
    /// This is the canonical engraving for acciaccatura and is also common
    /// for appoggiatura. The slur arcs away from the principal note's stem
    /// (stem-up → slur under, stem-down → slur over).
    ///
    /// No-op if the last event is a rest (grace notes attach to pitched events).
    pub fn grace_note_slur(mut self, pitch: Pitch, kind: GraceNoteKind) -> Self {
        use crate::layout::note_placement::pitch_to_staff_position;
        let clef = self.clef.to_clef();
        let staff_pos = pitch_to_staff_position(&pitch, &clef);
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.grace_note = Some((staff_pos, kind));
            annotations.grace_note_slur = true;
        }
        self
    }

    /// Attach a chord symbol above the staff at the most recently added note
    /// or chord (e.g. "Cmaj7", "Am", "G7", "F#dim").
    ///
    /// Chord symbols are rendered in bold above the staff, centered on the
    /// note/chord they apply to. No-op if the last event was a rest.
    pub fn chord_symbol(mut self, symbol: impl Into<String>) -> Self {
        let s = Some(symbol.into());
        if let Some(
            (_, ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. }),
        ) = self.current_events.last_mut()
        {
            annotations.chord_symbol = s;
        }
        self
    }

    /// Add a chord (multiple simultaneous pitches) to the current measure.
    ///
    /// All notes in the chord share the same duration. Noteheads that are a
    /// second apart are automatically offset to avoid collision.
    pub fn chord(mut self, pitches: Vec<Pitch>, duration: Duration) -> Self {
        self.current_events.push((self.current_voice, ScoreEvent::Chord { pitches, duration, annotations: NoteAnnotations::default() }));
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
        self.current_events.push((self.current_voice, ScoreEvent::BeamGroup { notes }));
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
        self.current_events.push((self.current_voice, ScoreEvent::TupletGroup { notes, tuplet_number }));
        self
    }

    /// Add a rest to the current measure.
    pub fn rest(mut self, duration: Duration) -> Self {
        self.current_events.push((self.current_voice, ScoreEvent::Rest { duration }));
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
        let events = std::mem::take(&mut self.current_events);
        let volta = self.resolve_volta();
        self.measures.push((events, BarlineStyle::Single, volta));
        self.current_voice = 0;
        self
    }

    /// End the current measure with a final (double) barline.
    /// Typically called at the end of the piece. Resets the active voice to 0.
    pub fn end_barline(mut self) -> Self {
        let events = std::mem::take(&mut self.current_events);
        let volta = self.resolve_volta();
        self.measures.push((events, BarlineStyle::Final, volta));
        self.current_voice = 0;
        self
    }

    /// End the current measure with a specific barline style.
    /// Resets the active voice to 0.
    pub fn barline_style(mut self, style: BarlineStyle) -> Self {
        let events = std::mem::take(&mut self.current_events);
        let volta = self.resolve_volta();
        self.measures.push((events, style, volta));
        self.current_voice = 0;
        self
    }

    /// Flush any pending events as a final measure if not already flushed.
    pub(crate) fn flush_pending(&mut self) {
        if !self.current_events.is_empty() {
            let events = std::mem::take(&mut self.current_events);
            let volta = self.resolve_volta();
            self.measures.push((events, BarlineStyle::Final, volta));
        }
        self.current_voice = 0;
    }

    /// Convert accumulated `ScoreEvent`s into `MeasureContent`s suitable for layout.
    ///
    /// Each measure's accidentals are tracked independently (courtesy naturals,
    /// suppression of redundant accidentals within a measure). When multiple
    /// voices are present, voice 0 goes in `events` and voices 1+ go in
    /// `additional_voices`. Multi-voice measures force stem directions:
    /// voice 0 = stems up, voice 1 = stems down.
    pub(crate) fn build_measure_contents(&self) -> Vec<MeasureContent> {
        let clef = self.clef.to_clef();
        self.measures
            .iter()
            .map(|(voiced_events, barline, volta)| {
                // Determine the maximum voice index in this measure.
                let max_voice = voiced_events.iter().map(|(v, _)| *v).max().unwrap_or(0);
                let is_multi_voice = max_voice > 0;

                let mut seen: AccidentalTracker = HashMap::new();

                // Separate events by voice.
                let mut voice_buckets: Vec<Vec<MeasureEvent>> =
                    (0..=max_voice).map(|_| Vec::new()).collect();

                for (voice, event) in voiced_events {
                    let mut me = convert_event(event, &clef, &self.key_sig, Some(&mut seen));
                    if is_multi_voice {
                        force_stem_direction(&mut me, *voice);
                    }
                    voice_buckets[*voice as usize].push(me);
                }

                let primary = voice_buckets.remove(0);
                MeasureContent {
                    events: primary,
                    barline: *barline,
                    volta: volta.clone(),
                    additional_voices: voice_buckets,
                }
            })
            .collect()
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
    ///     .note(Pitch::new(Note::E, 5).unwrap(), Duration::HALF)
    ///     .note(Pitch::new(Note::D, 5).unwrap(), Duration::HALF)
    ///     // Voice 1: bass (stems down)
    ///     .voice(1)
    ///     .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
    ///     .voice(0) // back to primary
    ///     .end_barline()
    ///     .render_svg();
    /// ```
    pub fn voice(mut self, voice_index: u8) -> Self {
        self.current_voice = voice_index;
        self
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
        let mut page_config = PageLayoutConfig::new(staff_space, sys_width);
        page_config.show_measure_numbers = self.show_measure_numbers;
        let breaking = if self.optimal_breaks {
            SystemBreaking::Optimal
        } else if self.auto_breaks {
            SystemBreaking::Auto
        } else {
            SystemBreaking::Fixed(self.effective_measures_per_system())
        };

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
