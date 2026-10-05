use crate::layout::accidental::{layout_accidental_columns, AccidentalDisplay, ResolvedAccidental};
use crate::layout::analysis_bracket::AnalysisBracketSpec;
use crate::layout::arpeggio::ArpeggioDirection;
use crate::layout::articulation::ArticulationMark;
use crate::layout::barline::BarlineStyle;
use crate::layout::breath::BreathMark;
use crate::layout::clef::{ClefLayout, ClefSize};
use crate::layout::dynamics::DynamicMark;
use crate::layout::glissando::GlissandoStyle;
use crate::layout::grace::{grace_group_extent, grace_stem_direction, GraceGroup, GraceNotes};
use crate::layout::group::{GroupMark, TupletSpec};
use crate::layout::hairpin::{HairpinType, NientePlacement};
use crate::layout::key_signature::KeySignature;
use crate::layout::lyric::{VerseLyric, LYRIC_FONT_SIZE_SS};
use crate::layout::navigation::NavigationSign;
use crate::layout::ornament::Ornament;
use crate::layout::ottava::OttavaKind;
use crate::layout::pedal::PedalMark;
use crate::layout::placement::Placement;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::staff::StaffPosition;
use crate::layout::stem::StemDirection;
use crate::layout::tempo::TempoMark;
use crate::layout::text_script::TextScript;
use crate::layout::text_spanner::TextSpanner;
use crate::layout::time_signature::TimeSignatureKind;
use crate::layout::tremolo::TremoloCount;
use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};
use crate::layout::trill_extension::{TrillSpeedRampSpec, TrillWiggleSpeed};

/// Semantic shape of a notehead, resolved to a duration-specific SMuFL glyph.
///
/// `Normal` is the default. The other styles are reusable notation semantics:
/// they are not guitar-renderer overlays, so they retain ordinary stems, beams,
/// tuplets, dots, accidentals, and span attachment geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoteheadStyle {
    /// Conventional oval notehead.
    #[default]
    Normal,
    /// Diamond harmonic notehead.
    Diamond,
    /// X notehead for dead or string-percussion attacks.
    X,
    /// Circled-X notehead for body percussion.
    CircleX,
    /// Rhythmic slash notehead.
    Slash,
    /// Square notehead for fretboard percussion.
    Square,
}

impl NoteheadStyle {
    /// Resolve this shape for `duration_log2` (`-1` breve, `0` whole, `1` half,
    /// `2+` filled).
    ///
    /// Every style uses its SMuFL double-whole variant for breves except
    /// [`Self::Square`]: SMuFL defines no double-whole square percussion head,
    /// so a square breve falls back to the open `NoteheadSquareWhite` shared
    /// with whole and half notes (its breve duration still drives spacing and
    /// stem suppression).
    pub fn glyph(self, duration_log2: i8) -> smufl::Glyph {
        use smufl::Glyph;
        match (self, duration_log2) {
            (Self::Normal, ..=-1) => Glyph::NoteheadDoubleWhole,
            (Self::Normal, 0) => Glyph::NoteheadWhole,
            (Self::Normal, 1) => Glyph::NoteheadHalf,
            (Self::Normal, _) => Glyph::NoteheadBlack,
            (Self::Diamond, ..=-1) => Glyph::NoteheadDiamondDoubleWhole,
            (Self::Diamond, 0) => Glyph::NoteheadDiamondWhole,
            (Self::Diamond, 1) => Glyph::NoteheadDiamondHalf,
            (Self::Diamond, _) => Glyph::NoteheadDiamondBlack,
            (Self::X, ..=-1) => Glyph::NoteheadXDoubleWhole,
            (Self::X, 0) => Glyph::NoteheadXWhole,
            (Self::X, 1) => Glyph::NoteheadXHalf,
            (Self::X, _) => Glyph::NoteheadXBlack,
            (Self::CircleX, ..=-1) => Glyph::NoteheadCircleXDoubleWhole,
            (Self::CircleX, 0) => Glyph::NoteheadCircleXWhole,
            (Self::CircleX, 1) => Glyph::NoteheadCircleXHalf,
            (Self::CircleX, _) => Glyph::NoteheadCircleX,
            (Self::Slash, ..=-1) => Glyph::NoteheadSlashWhiteDoubleWhole,
            (Self::Slash, 0) => Glyph::NoteheadSlashWhiteWhole,
            (Self::Slash, 1) => Glyph::NoteheadSlashWhiteHalf,
            (Self::Slash, _) => Glyph::NoteheadSlashVerticalEnds,
            (Self::Square, ..=1) => Glyph::NoteheadSquareWhite,
            (Self::Square, _) => Glyph::NoteheadSquareBlack,
        }
    }
}

/// Engraved size of a note or chord.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NoteSize {
    /// Ordinary size.
    #[default]
    Normal,
    /// Small (cue) notes, LilyPond `\tiny` / font size −2: noteheads,
    /// accidentals, dots, stems, flags, and their spacing rods all shrink by
    /// [`NoteSize::scale`].
    Cue,
}

impl NoteSize {
    /// Glyph and spacing scale factor: 1 for normal notes, `2^(−2/6)` ≈ 0.794
    /// for cue notes.
    pub fn scale(self) -> f64 {
        match self {
            Self::Normal => 1.0,
            Self::Cue => CUE_NOTE_SCALE,
        }
    }
}

/// Size of [`NoteSize::Cue`] notes: LilyPond font size −2, `2^(−2/6)`.
pub const CUE_NOTE_SCALE: f64 = 0.793_700_525_984_1;

/// Whether a note or chord's stem (and with it its flags) is engraved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StemVisibility {
    /// Engrave the stem and flags its duration calls for.
    #[default]
    Visible,
    /// Engrave neither stem nor flags (LilyPond `\omit Stem` / `\omit
    /// Flag`): stemless formulas, chord series, and imagined notes.
    Hidden,
}

/// Articulation and expression annotations attached to a note, chord or rest.
///
/// These fields are shared between [`NoteEvent`], [`ChordEvent`] and
/// [`RestEvent`], covering ties, slurs, hairpins, dynamics, rehearsal marks,
/// tempo marks, text scripts and text spanners. On rests only the
/// pitch-independent fields apply (see [`RestEvent::annotations`]).
/// All fields default to "no annotation" (`false` / `None`).
#[derive(Clone, Debug, Default)]
pub struct NoteAnnotations {
    /// Destination stave for an event of a continuous cross-staff voice.
    /// Ignored by ordinary single-staff scores.
    pub on_staff: Option<usize>,
    /// Identity within the cross-staff voice, used to join laid-out anchors.
    pub(crate) cross_staff_id: Option<usize>,
    /// Notehead styles parallel to the note/chord's pitches. An empty vector
    /// means [`NoteheadStyle::Normal`] for every pitch.
    pub notehead_styles: Vec<NoteheadStyle>,
    /// Whether each resolved notehead is enclosed by real SMuFL notehead
    /// parentheses (LilyPond `\parenthesize`). The enclosure also takes in the
    /// notehead's own accidental. Entries are parallel to pitches; missing
    /// entries are false.
    pub parenthesized_noteheads: Vec<bool>,
    /// Engraved size of the whole note/chord (noteheads, accidentals, dots,
    /// stem, flags, and their spacing rods).
    pub size: NoteSize,
    /// Whether the stem and flags are engraved.
    pub stem: StemVisibility,
    /// Whether the augmentation dots are enclosed in parentheses (LilyPond
    /// `Dots.parenthesized`).
    pub parenthesized_dots: bool,
    /// Accidental display policies parallel to the note/chord's pitches. An
    /// empty vector (or a missing entry) means [`AccidentalDisplay::Auto`].
    pub accidental_displays: Vec<AccidentalDisplay>,
    /// Stem direction requested for this note/chord (`\stemUp` /
    /// `\stemDown`). Score conversion copies it into the event's
    /// `stem_direction`; `None` leaves the direction to the enclosing beam,
    /// the voice, or the staff position.
    pub stem_direction: Option<StemDirection>,
    /// Whether this event is a fixed-position, unpitched semantic head.
    ///
    /// Unpitched heads bypass key-signature accidental resolution entirely:
    /// they neither display accidentals nor mutate the measure tracker.
    pub unpitched: bool,
    /// Whether this note/chord is tied forward to the next note at the same
    /// staff position. The tie curve is drawn by the system renderer after
    /// all measures are laid out.
    pub tie_forward: bool,
    /// Optional dynamic marking (a SMuFL dynamic such as pp, mf, ff, or a
    /// custom words-plus-glyph dynamic such as "più p"), centered on this
    /// event on the side given by `dynamics_placement`.
    pub dynamic: Option<DynamicMark>,
    /// Side of the staff for this event's dynamic and for the hairpin that
    /// starts here (LilyPond `\dynamicUp` / `\dynamicDown`). Defaults to
    /// below.
    pub dynamics_placement: Placement,
    /// Whether this note/chord is the start of a slur (curved line to a following note).
    /// The slur curve is drawn by the system renderer after all measures are laid out.
    pub slur_start: bool,
    /// Whether this note/chord is the end of a slur.
    pub slur_end: bool,
    /// Whether this note/chord is the start of a hairpin (crescendo/decrescendo wedge).
    /// The hairpin is drawn by the system renderer after all measures are laid out.
    pub hairpin_start: Option<HairpinType>,
    /// Whether this note/chord is the end of a hairpin wedge.
    pub hairpin_end: bool,
    /// Whether the hairpin starting at this note/chord should be drawn with
    /// dashed wedge lines instead of solid. Has no effect unless
    /// `hairpin_start` is `Some`. Engraved convention uses dashed wedges for
    /// "soft" or implied crescendi and for modern-notation continuation
    /// markings independent of the `cresc. - - -` text variant.
    ///
    /// The flag lives on the start side because the wedge is a single visual
    /// object owned by the start note; the end note carries no styling.
    /// Cross-system hairpins propagate the dashed style to the trailing
    /// half on the source system; the incoming half on the next system is
    /// always dashed regardless of this flag (engraved convention for
    /// cross-system continuations).
    pub hairpin_dashed: bool,
    /// Optional niente "o" circle on the hairpin starting at this note/chord.
    /// `None` for a plain hairpin; `Some(ClosedEnd)` for the standard
    /// engraving convention ("al niente" / "dal niente" — circle at the
    /// pointy tip); `Some(OpenEnd)` for the rarer modern-notation variant
    /// (circle at the wide tip).
    ///
    /// Has no effect unless `hairpin_start` is `Some`. Combines freely with
    /// `hairpin_dashed`: the wedge lines dash but the niente "o" stays solid
    /// per engraved convention (the circle is a definite symbol).
    ///
    /// Cross-system hairpins propagate the niente to whichever half (trailing
    /// on the source system or incoming on the target system) contains the
    /// anchor tip. For a crescendo with `ClosedEnd`, the niente sits at the
    /// closed left tip → trailing half on the source system. For a
    /// decrescendo with `ClosedEnd`, the niente sits at the closed right tip
    /// → incoming half on the target system. `OpenEnd` flips both rules.
    pub hairpin_niente: Option<NientePlacement>,
    /// Optional rehearsal mark displayed above the staff at this note/chord's position.
    /// Tuple of (text content, enclosure style).
    pub rehearsal_mark: Option<(String, RehearsalStyle)>,
    /// Optional tempo marking displayed above the staff at this note/chord's position.
    pub tempo_mark: Option<TempoMark>,
    /// Free text scripts above or below the staff (LilyPond `^\markup` /
    /// `_\markup`), e.g. italic "dolce" below or "a)" above. Scripts on
    /// the same side stack outward in order.
    pub text_scripts: Vec<TextScript>,
    /// Marks centered on the following barline (LilyPond `\textMark` /
    /// `\textEndMark`). If no barline follows, drawn at the event's right edge.
    pub text_marks: Vec<TextScript>,
    /// Articulation-like marks (staccato, tenuto, accent, fermata, caller
    /// glyphs, …) placed near the notehead or rest.
    pub articulations: Vec<ArticulationMark>,
    /// Written grace notes, converted to `grace_group` before layout.
    pub grace_notes: Option<GraceNotes>,
    /// Resolved grace group with staff positions and accidentals.
    pub grace_group: Option<GraceGroup>,
    /// Numbered lyric verses at this event, each with its own continuation and font style.
    pub lyrics: Vec<VerseLyric>,
    /// Optional chord symbol displayed above the staff (e.g. "Cmaj7", "Am").
    pub chord_symbol: Option<String>,
    /// Optional ornament marking (trill, mordent, turn, etc.) placed above the staff.
    pub ornament: Option<Ornament>,
    /// Optional navigation sign (segno, coda) placed above the staff.
    pub navigation_sign: Option<NavigationSign>,
    /// Whether this note/chord starts an ottava bracket (8va, 8vb, etc.).
    /// The bracket extends from this note to the note with `ottava_end = true`.
    pub ottava_start: Option<OttavaKind>,
    /// Whether this note/chord ends an ottava bracket.
    pub ottava_end: bool,
    /// Optional pedal marking (Ped. down or * up) placed below the staff.
    pub pedal: Option<PedalMark>,
    /// Optional tremolo slashes (1–3) drawn on the stem of this note/chord.
    pub tremolo: Option<TremoloCount>,
    /// Optional arpeggio (rolled chord) wavy line drawn to the left of the
    /// chord noteheads. Primarily used on chords but valid on single notes.
    pub arpeggio: Option<ArpeggioDirection>,
    /// Optional breath mark (comma, tick, or caesura) placed above the staff
    /// to the right of this note/chord, indicating a brief pause or lift.
    pub breath_mark: Option<BreathMark>,
    /// Whether the breath mark is enclosed in parentheses.
    pub breath_mark_parenthesized: bool,
    /// Whether this note/chord starts a glissando line to the next note.
    /// The diagonal line is drawn by the system renderer after all measures
    /// are laid out.
    pub glissando_start: Option<GlissandoStyle>,
    /// Whether this trill ornament has a wavy-line extension that continues
    /// to the next note. Has no effect unless `ornament` is `Some(Ornament::Trill)`;
    /// the renderer skips the extension cleanly when the ornament isn't a trill.
    /// The wavy line is drawn by the system renderer after all measures are
    /// laid out, tiling the SMuFL `wiggleTrill` segment between the trill
    /// glyph and the next note.
    pub trill_extension: bool,
    /// Optional bracket form for the trill extension: a vertical hook capping
    /// the start, end, or both ends of the wavy line. Has no effect unless
    /// both `ornament == Some(Ornament::Trill)` and `trill_extension == true`
    /// — a bracket without a wiggle to bracket is silently dropped. For
    /// cross-system trills, a `Both` bracket places the start hook on the
    /// source system (with the outgoing wiggle) and the end hook on the
    /// target system (with the incoming wiggle), so the bracket frames the
    /// trill's true semantic range rather than the per-system wiggle
    /// fragments.
    pub trill_bracket: Option<TrillBracketSide>,
    /// Optional override for the direction in which trill bracket hooks
    /// extend from the wiggle baseline. Has no effect unless `trill_bracket`
    /// is `Some`. `None` selects the conventional `Down` direction (hook
    /// points back toward the staff for trills sitting above the staff).
    /// Set to `Some(HookDirection::Up)` for the rare case of a trill rendered
    /// below the staff where the hook should still point back toward the
    /// affected notes.
    pub trill_bracket_direction: Option<HookDirection>,
    /// Optional override for trill bracket hook length, in staff spaces.
    /// Has no effect unless `trill_bracket` is `Some`. `None` selects the
    /// default of ~0.75 staff spaces. Behind Bars shows hooks ranging from
    /// roughly 0.5 to 1.0 staff spaces depending on the surrounding density;
    /// callers can opt into a thinner or chunkier hook here without changing
    /// the engraving config globally.
    pub trill_bracket_length_ss: Option<f64>,
    /// Optional speed/density variant for the trill wavy-line extension.
    /// Has no effect unless `trill_extension == true`. `None` selects the
    /// neutral `Standard` wiggle (Bravura's `wiggleTrill` glyph). Choosing
    /// a faster or slower variant communicates trill speed visually while
    /// keeping the gesture's meaning unchanged.
    pub trill_wiggle_speed: Option<TrillWiggleSpeed>,
    /// Optional explicit termination length for the trill wavy-line
    /// extension, in staff spaces. Has no effect unless `trill_extension ==
    /// true`. `None` (the default) lets the wiggle extend to the next note
    /// (within-system) or to the system's right edge (cross-system) per the
    /// usual convention. `Some(length_ss)` clamps the wiggle so it terminates
    /// no later than `length_ss` staff spaces past its natural start —
    /// useful when a trill should visually "run out" before the next note
    /// (e.g. a trill on a half note where the trill is intended to release
    /// partway through the held duration).
    ///
    /// Clamping is one-sided: if the requested length is larger than the
    /// natural span, the natural span wins (no overrun past the next note
    /// or the system edge). When set to a positive value, the wiggle never
    /// propagates across a system break — even if the natural span would
    /// have extended to the system edge — because the explicit length
    /// already specifies a definite endpoint.
    ///
    /// Non-positive values produce no wiggle (the renderer's same
    /// fail-safe as for spans too short to fit one tile).
    pub trill_extension_length_ss: Option<f64>,
    /// Optional explicit end-anchor for the trill wavy-line extension,
    /// expressed as a positive note offset (in the system's flat note
    /// sequence) from the trilled note. Has no effect unless
    /// `trill_extension == true`. `None` (the default) terminates the
    /// wiggle at the *immediately following* note (offset = 1 implicitly)
    /// or at the system's right edge if this is the last note. `Some(n)`
    /// with `n >= 1` terminates the wiggle at the note `n` positions after
    /// the trilled note — useful when the trill should visibly hold across
    /// one or more intervening notes before releasing into a specific
    /// successor.
    ///
    /// Offsets that walk past the end of the system fall back to the
    /// "extend to system right edge" behavior (same as a trilled last
    /// note), and the trill propagates across the system break only when
    /// `n` walks past the *last* note (i.e. the natural last-note case);
    /// any other walk-past-end terminates at the system edge without
    /// cross-system continuation.
    ///
    /// `Some(0)` is rejected at the renderer's fail-safe as no wiggle (a
    /// zero-offset target is the trilled note itself, so start_x ==
    /// end_x — same suppression as a non-positive `trill_extension_length_ss`).
    ///
    /// Independent of [`trill_extension_length_ss`](Self::trill_extension_length_ss):
    /// when both are set, the explicit length wins (the length field is
    /// the more specific termination). This matches the documented
    /// "explicit length specifies a definite endpoint" semantic of the
    /// length field; the to-note offset is a softer "stretch to note N"
    /// hint that yields to the explicit length when both are present.
    pub trill_extension_to_note_offset: Option<usize>,
    /// Optional multi-speed ramp spec for the trill wavy-line extension.
    /// Has no effect unless `trill_extension == true`. `None` (the default)
    /// renders a single-speed wiggle using `trill_wiggle_speed` (or the
    /// `Standard` default when that is also `None`). `Some(spec)` engages
    /// the multi-speed renderer path: the system renderer evenly partitions
    /// the wiggle's span into `spec.region_count` regions and tiles each
    /// region with the speed produced by `spec.ramp.synthesize_regions`.
    ///
    /// When a ramp is present, it *supersedes* `trill_wiggle_speed` for
    /// glyph selection — the speed field's value is ignored by the
    /// renderer's wiggle path (the bracket/length fields continue to apply
    /// as normal). Both fields are permitted to coexist on the annotation
    /// so widening a single-speed annotation by layering a ramp does not
    /// force the caller to first clear the speed field.
    ///
    /// A degenerate spec (e.g. `region_count == 0`, or `Linear` with
    /// `region_count == 1`) makes `synthesize_regions` return `None` at
    /// draw time, and the renderer falls back to no wiggle — same
    /// fail-safe as for spans too short to tile.
    pub trill_speed_ramp: Option<TrillSpeedRampSpec>,
    /// Text spanner starting at this event: a label ("rit.", "cresc.",
    /// "dim") followed by a dashed, solid or no line running to the next
    /// event with `text_spanner_end`, continuing across system breaks.
    pub text_spanner_start: Option<TextSpanner>,
    /// Whether this event ends the text spanner opened by a preceding
    /// `text_spanner_start`.
    pub text_spanner_end: bool,
    /// Start a horizontal analysis bracket on this exact event, even inside a beam/tuplet.
    pub analysis_bracket_start: Option<AnalysisBracketSpec>,
    /// End the most recently opened horizontal analysis bracket on this event.
    pub analysis_bracket_end: bool,
}

/// A chord (multiple simultaneous notes) to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct ChordEvent {
    /// Staff positions of notes in the chord (bottom line = 0), in any order.
    /// Will be sorted during layout.
    pub staff_positions: Vec<i8>,
    /// Log2 of the duration denominator: -1=breve, 0=whole, 1=half, 2=quarter, 3=eighth, etc.
    /// All notes in a chord share the same duration.
    pub duration_log2: i8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
    /// Resolved accidentals to display, parallel to `staff_positions`.
    /// `None` entries mean no accidental for that note.
    pub accidentals: Vec<Option<ResolvedAccidental>>,
    /// Stem direction override. `None` uses auto-detection based on chord extent.
    pub stem_direction: Option<StemDirection>,
    /// Articulation/expression annotations (ties, dynamics, slurs, hairpins, etc.).
    pub annotations: NoteAnnotations,
}

/// A musical event within a measure that occupies horizontal space.
#[derive(Clone, Debug)]
pub enum MeasureElement {
    /// Clef (typically only at the start of the first measure or on clef change).
    /// Uses `ClefLayout` which wraps `music::Clef` info without requiring Clone on Clef.
    Clef(ClefLayout),
    /// Key signature.
    KeySignature(KeySignature),
    /// Time signature.
    TimeSignature(TimeSignatureKind),
    /// A note event: staff position, notehead kind, stem direction, flag count, dot count,
    /// optional accidental glyph (already resolved to SMuFL glyph).
    Note(NoteEvent),
    /// A rest event: log2 duration (-1=breve, 0=whole, 1=half, 2=quarter, etc.), dot count.
    Rest(RestEvent),
    /// A chord (multiple simultaneous notes).
    Chord(ChordEvent),
    /// A zero-width beam or tuplet span boundary. Members are the ordinary
    /// notes, chords, and rests between a start and its matching end; see
    /// [`crate::layout::group`].
    GroupMark(GroupMark),
    /// Multi-measure rest: H-bar (default) or church-rest cluster spanning
    /// the measure width with a count number. `count` is the number of
    /// measures of rest; `style` controls the visual depiction.
    MultiMeasureRest {
        /// Number of consecutive measures of rest.
        count: u32,
        /// Visual style (H-bar or church-rest).
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle,
    },
    /// Invisible rhythmic placeholder (a LilyPond spacer `s`): takes time
    /// and space like a rest but draws nothing. A measure holding only
    /// spacers renders as an empty bar.
    Spacer(SpacerEvent),
    /// A closing or inline barline; an inline barline neither ends the
    /// logical measure nor resets its accidental state.
    Barline(BarlineStyle),
}

/// A single note to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct NoteEvent {
    /// Staff position (bottom line = 0).
    pub staff_position: i8,
    /// Log2 of the duration denominator: -1=breve, 0=whole, 1=half, 2=quarter, 3=eighth, etc.
    pub duration_log2: i8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
    /// Accidental to display (if any), already resolved against the key
    /// signature, the measure's accidental state, and the display policy.
    pub accidental: Option<ResolvedAccidental>,
    /// Stem direction override. `None` uses auto-detection.
    pub stem_direction: Option<StemDirection>,
    /// Articulation/expression annotations (ties, dynamics, slurs, hairpins, etc.).
    pub annotations: NoteAnnotations,
}

/// A rest to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct RestEvent {
    /// Log2 of the duration denominator: -1=breve, 0=whole, 1=half, 2=quarter, etc.
    pub duration_log2: i8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
    /// Marks attached to the rest: dynamics, hairpin and text-spanner
    /// endpoints, tempo and rehearsal marks, text scripts, fermatas (and
    /// other articulations, drawn above the staff), breath marks. Pitch- and
    /// stem-bound fields (ties, slurs, lyrics, ornaments, grace notes,
    /// tremolos, glissandi, noteheads, accidentals) do not apply to rests.
    pub annotations: NoteAnnotations,
}

/// An invisible rhythmic placeholder within a measure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpacerEvent {
    /// Log2 of the duration denominator: -1=breve, 0=whole, 1=half, 2=quarter, etc.
    pub duration_log2: i8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
}

/// A positioned element within a laid-out measure.
///
/// Each element's horizontal extent is decomposed into a Gourlay
/// **rod** (incompressible: notehead + accidental cluster + dot cluster +
/// minimum padding) and a **spring** (compressible: the duration-driven rest
/// length). The natural-layout `width` is always `rod + spring`. The system
/// layer compresses/extends a system to a target width by scaling springs only
/// and leaving rods fixed (see `layout::system`).
#[derive(Clone, Debug)]
pub struct PositionedElement {
    /// X-coordinate (in font design units) from the start of the measure.
    pub x: f64,
    /// The element.
    pub element: MeasureElement,
    /// Advance width of this element at natural layout (`rod + spring`),
    /// in font design units.
    pub width: f64,
    /// Incompressible rod width (in font design units). Prefix elements
    /// (clef/key/time), barlines, and multi-measure rests are fully rod
    /// (`spring == 0`).
    pub rod: f64,
    /// Compressible spring width (in font design units). Scaled by the system
    /// layer to fit a target width. Invariant: `width == rod + spring`.
    pub spring: f64,
}

/// The result of laying out a measure: elements with assigned x-positions.
#[derive(Clone, Debug)]
pub struct MeasureLayout {
    /// Positioned elements in left-to-right order.
    pub elements: Vec<PositionedElement>,
    /// Total width of the measure at natural layout, in font design units
    /// (`total_rod + total_spring`).
    pub total_width: f64,
    /// Sum of all element rods (incompressible).
    pub total_rod: f64,
    /// Sum of all element springs (compressible). Invariant:
    /// `total_width == total_rod + total_spring`.
    pub total_spring: f64,
}

impl MeasureLayout {
    /// The measure's closing barline: its last `Barline` element. Courtesy
    /// elements at a system end may follow it.
    fn closing_barline(&self) -> Option<&PositionedElement> {
        self.elements
            .iter()
            .rev()
            .find(|element| matches!(element.element, MeasureElement::Barline(_)))
    }

    /// Measure-relative x of the closing barline element (the measure's
    /// right edge when it has none).
    pub fn closing_barline_x(&self) -> f64 {
        self.closing_barline()
            .map_or(self.total_width, |element| element.x)
    }

    /// Measure-relative right edge of the closing barline element (the
    /// measure's right edge when it has none). Equals `total_width` unless
    /// courtesy elements follow the barline.
    pub fn closing_barline_end(&self) -> f64 {
        self.closing_barline()
            .map_or(self.total_width, |element| element.x + element.width)
    }
}

/// Configuration for measure layout.
#[derive(Clone, Debug)]
pub struct MeasureLayoutConfig {
    /// Staff space in font design units. Glyph metrics (clef bounding boxes,
    /// time-signature advances) are given in staff spaces and scale by it.
    pub staff_space: f64,
    /// Space reserved left of a full-size (system-start) clef, between the
    /// start of the staff and the clef's ink.
    pub clef_left_margin: f64,
    /// Space reserved left of a change-size clef, after the preceding element.
    pub clef_change_margin: f64,
    /// Padding after a clef's ink before the next element.
    pub clef_padding: f64,
    /// Width allocated per key signature accidental.
    pub key_sig_accidental_width: f64,
    /// Padding after key signature.
    pub key_sig_padding: f64,
    /// Padding after time signature before first note.
    pub time_sig_padding: f64,
    /// Space reserved left of a time signature that follows a barline (a
    /// meter change, or a courtesy signature ending a system).
    pub time_sig_change_margin: f64,
    /// Width for a barline.
    pub barline_width: f64,
    /// Minimum rhythmic width of a measure that holds only spacers (an empty
    /// bar), in font design units.
    pub empty_measure_min_width: f64,
    /// Gourlay spacing exponent `c` in the spring rest length `k · duration^c`.
    /// Default 0.6 (Gould/Gourlay empirical range 0.5–0.7). Larger `c` widens
    /// the gap between long and short notes.
    pub spacing_exponent: f64,
    /// Gourlay spring constant `k` (in font design units): the rest length of
    /// the spring following a note of the shortest duration in the measure
    /// (where `duration == 1`).
    pub spring_constant: f64,
    /// Incompressible rod estimate for one notehead (in font design units).
    pub notehead_rod: f64,
    /// Additional rod width when an event carries one plain accidental
    /// (in font design units).
    pub accidental_rod: f64,
    /// Further rod width when an accidental is parenthesized, covering both
    /// SMuFL accidental parentheses (in font design units).
    pub accidental_parens_rod: f64,
    /// Horizontal gap between stacked accidental columns of one chord
    /// (in font design units).
    pub accidental_column_gap: f64,
    /// Additional rod width per augmentation dot (in font design units).
    pub dot_rod: f64,
    /// Further rod width when a note's dots are parenthesized (in font
    /// design units).
    pub dot_parens_rod: f64,
    /// Rod width of one notehead parenthesis (in font design units); a
    /// parenthesized note reserves one before and one after its noteheads.
    pub notehead_parens_rod: f64,
    /// Minimum padding included in every rhythmic rod (in font design units).
    pub min_rod_padding: f64,
}

impl MeasureLayoutConfig {
    /// Default config using a staff space value (typically from EngravingConfig).
    pub fn from_staff_space(ss: f64) -> Self {
        Self {
            staff_space: ss,
            clef_left_margin: 1.0 * ss,
            clef_change_margin: 0.5 * ss,
            clef_padding: 0.5 * ss,
            key_sig_accidental_width: 1.0 * ss,
            key_sig_padding: 0.75 * ss,
            time_sig_padding: 0.75 * ss,
            time_sig_change_margin: 0.5 * ss,
            barline_width: 0.5 * ss,
            empty_measure_min_width: 4.0 * ss,
            spacing_exponent: 0.6,
            // Phase 4 calibration (see `examples/spacing_calibration.rs` and the
            // 2026-08-06 progress entry): k = 1.0·ss. Matching the legacy
            // power-of-ratio model's natural widths — the plan's original
            // churn-minimizing target — was found to be both unachievable and
            // undesirable: it needs k ≈ 0.02·ss for uniform rhythms (springs
            // ~0, collapsing the model to fixed-width spacing) and a *negative*
            // k for accidental-heavy measures, because the legacy model gave
            // accidentals no room at all. k = 1.0·ss instead keeps the realized
            // long:short advance ratio inside the engraving-practice band.
            spring_constant: 1.0 * ss,
            // Notehead advance estimate matches the value used elsewhere
            // (e.g. `layout/glissando.rs`): ~1.18 staff spaces.
            notehead_rod: 1.18 * ss,
            accidental_rod: 1.0 * ss,
            // Bravura's accidentalParensLeft/Right each advance 0.564 ss.
            accidental_parens_rod: 1.128 * ss,
            accidental_column_gap: crate::layout::accidental::ACCIDENTAL_COLUMN_GAP_SS * ss,
            dot_rod: 0.35 * ss,
            dot_parens_rod: crate::layout::dot::DOT_PARENTHESES_EXTRA_SS * ss,
            // Bravura's noteheadParenthesisLeft/Right each advance 0.436 ss.
            notehead_parens_rod: 0.436 * ss,
            min_rod_padding: 0.3 * ss,
        }
    }
}

/// Compute the Gourlay spring rest length for a note of the given duration.
///
/// The shortest written note in the measure has duration `1.0`; a note twice
/// as long has duration `2.0`, etc. Tuplet time scaling is applied by the
/// caller so an explicit ratio changes a tuplet member's advance relative to
/// ordinary events without changing the tuplet's internal proportions.
fn spring_rest_length(
    duration_log2: i8,
    shortest_log2: i8,
    spring_constant: f64,
    spacing_exponent: f64,
) -> f64 {
    let steps = f64::from(shortest_log2) - f64::from(duration_log2);
    let duration = 2.0_f64.powf(steps);
    spring_constant * duration.powf(spacing_exponent)
}

/// Compute the incompressible rod width for a rhythmic event whose engraved
/// accidentals extend `accidental_extent` to the left of its noteheads.
fn event_rod(accidental_extent: f64, dots: u8, config: &MeasureLayoutConfig) -> f64 {
    config.min_rod_padding + config.notehead_rod + accidental_extent + dots as f64 * config.dot_rod
}

/// Rod of a note or chord at its annotated size: padding, notehead, dots
/// (with their parentheses), the closing notehead parenthesis, and an inner
/// accidental extent (`0.0` unless it is a later beam/tuplet member).
fn note_rod(
    accidental_extent: f64,
    dots: u8,
    annotations: &NoteAnnotations,
    config: &MeasureLayoutConfig,
) -> f64 {
    let scale = annotations.size.scale();
    let mut rod = config.min_rod_padding
        + config.notehead_rod * scale
        + accidental_extent
        + dots as f64 * config.dot_rod * scale;
    if dots > 0 && annotations.parenthesized_dots {
        rod += config.dot_parens_rod * scale;
    }
    if annotations.parenthesized_noteheads.contains(&true) {
        rod += config.notehead_parens_rod * scale;
    }
    rod
}

/// Estimated rod width of one engraved accidental, including its parentheses.
fn accidental_rod_width(accidental: ResolvedAccidental, config: &MeasureLayoutConfig) -> f64 {
    if accidental.parenthesized {
        config.accidental_rod + config.accidental_parens_rod
    } else {
        config.accidental_rod
    }
}

/// Estimated leftward extent of the accidentals engraved on one notehead
/// column: a lone accidental's rod width (parentheses included), or the
/// stacked chord accidental columns' extent when several are engraved.
///
/// `accidentals` is parallel to `staff_positions`.
fn accidental_left_extent(
    staff_positions: &[StaffPosition],
    accidentals: &[Option<ResolvedAccidental>],
    config: &MeasureLayoutConfig,
) -> f64 {
    let mut engraved =
        staff_positions
            .iter()
            .zip(accidentals)
            .filter_map(|(&position, accidental)| {
                accidental.map(|accidental| (position, accidental_rod_width(accidental, config)))
            });
    let Some(first) = engraved.next() else {
        return 0.0;
    };
    let Some(second) = engraved.next() else {
        return first.1;
    };
    let mut stacked = vec![first, second];
    stacked.extend(engraved);
    layout_accidental_columns(&stacked, config.accidental_column_gap).extent
}

/// Estimated extent of everything a note or chord engraves left of its
/// notehead column, at its annotated size: accidentals, the opening notehead
/// parenthesis, and a preceding grace group (whose stems follow
/// `stem_direction`, the principal's forced direction if any).
fn annotated_left_extent(
    staff_positions: &[StaffPosition],
    accidentals: &[Option<ResolvedAccidental>],
    annotations: &NoteAnnotations,
    stem_direction: Option<StemDirection>,
    config: &MeasureLayoutConfig,
) -> f64 {
    let scale = annotations.size.scale();
    let mut extent = accidental_left_extent(staff_positions, accidentals, config) * scale;
    if annotations.parenthesized_noteheads.contains(&true) {
        extent += config.notehead_parens_rod * scale;
    }
    if let Some(group) = &annotations.grace_group {
        extent += grace_group_extent(
            group,
            grace_stem_direction(stem_direction),
            config.staff_space,
        );
    }
    extent
}

fn note_left_extent(note: &NoteEvent, config: &MeasureLayoutConfig) -> f64 {
    annotated_left_extent(
        std::slice::from_ref(&note.staff_position),
        std::slice::from_ref(&note.accidental),
        &note.annotations,
        note.stem_direction,
        config,
    )
}

/// Space reserved before an element: its accidental, grace and parenthesis
/// extent, or a clef's margin (zero for rests and bare notes).
fn element_left_extent(element: &MeasureElement, config: &MeasureLayoutConfig) -> f64 {
    match element {
        MeasureElement::Note(note) => note_left_extent(note, config),
        MeasureElement::Chord(chord) => annotated_left_extent(
            &chord.staff_positions,
            &chord.accidentals,
            &chord.annotations,
            chord.stem_direction,
            config,
        ),
        MeasureElement::Clef(clef) => match clef.size {
            ClefSize::Full => config.clef_left_margin,
            ClefSize::Change => config.clef_change_margin,
        },
        _ => 0.0,
    }
}

/// Syllables on a pitched event. Text and its width never reserve space on
/// rests or structural span marks.
fn event_lyrics(element: &MeasureElement) -> Option<(&[VerseLyric], f64)> {
    match element {
        MeasureElement::Note(note) => {
            Some((&note.annotations.lyrics, note.annotations.size.scale()))
        }
        MeasureElement::Chord(chord) => {
            Some((&chord.annotations.lyrics, chord.annotations.size.scale()))
        }
        _ => None,
    }
}

/// Lay out a sequence of measure elements with horizontal positions.
///
/// Non-rhythmic elements (clef, key sig, time sig, barline) are fully
/// incompressible (all rod, no spring). Rhythmic elements (notes, rests,
/// chords) decompose into a Gourlay rod (notehead + accidental + dot +
/// padding) and a duration-driven spring; a member of an open tuplet span
/// scales its spring by the open tuplets' ratios. Beam and tuplet span marks
/// ([`MeasureElement::GroupMark`]) take no width, so span members are spaced
/// exactly like standalone events and each keeps its own x. The system layer
/// later scales springs only to fit a target width.
///
/// Accidentals sit left of their notehead column, so each event's accidental
/// extent (stacked columns and parentheses included) is reserved as an
/// incompressible gap *before* the event — including a measure-initial event,
/// whose accidentals would otherwise cross the preceding barline.

pub fn layout_measure(elements: &[MeasureElement], config: &MeasureLayoutConfig) -> MeasureLayout {
    let mut positioned = Vec::with_capacity(elements.len());
    let mut x = 0.0;
    let mut reserved_leading_gaps = 0.0;
    let mut previous: Option<&MeasureElement> = None;

    // Find the shortest written duration. Tuplet ratios scale their springs
    // below, preserving both the established Gourlay baseline and performed
    // n:in-the-time-of duration relative to ordinary events.
    let shortest_log2 = elements
        .iter()
        .filter_map(|element| crate::layout::group::member_duration(element).map(|(log2, _)| log2))
        .max()
        .unwrap_or(2);

    let spring = |duration_log2: i8, time_scale: f64| {
        spring_rest_length(
            duration_log2,
            shortest_log2,
            config.spring_constant,
            config.spacing_exponent,
        ) * time_scale.powf(config.spacing_exponent)
    };

    // Tuplet spans open at this point of the measure, outermost first.
    let mut open_tuplets: Vec<TupletSpec> = Vec::new();
    for elem in elements {
        let time_scale = crate::layout::group::tuplet_time_scale(&open_tuplets);
        // Each arm yields (rod, spring, trailing_padding). Prefix elements use
        // trailing padding (e.g. clef_padding) that sits outside the element's
        // own width; rhythmic elements fold all spacing into rod + spring
        // except their leading accidental extent, reserved before them below.
        let (rod, spr, trailing) = match elem {
            // The clef's origin sits at the element x (after its leading
            // margin); its rod is the inked width right of the origin.
            MeasureElement::Clef(clef) => (
                clef.ink_box().x_right * config.staff_space,
                0.0,
                config.clef_padding,
            ),
            MeasureElement::KeySignature(key) => {
                let count = match key {
                    KeySignature::Sharps(n) | KeySignature::Flats(n) => *n as f64,
                    KeySignature::Open => 0.0,
                };
                let w = count * config.key_sig_accidental_width;
                let trailing = if w > 0.0 { config.key_sig_padding } else { 0.0 };
                (w, 0.0, trailing)
            }
            MeasureElement::TimeSignature(kind) => (
                kind.width_ss() * config.staff_space,
                0.0,
                config.time_sig_padding,
            ),
            MeasureElement::Note(n) => (
                note_rod(0.0, n.dots, &n.annotations, config),
                spring(n.duration_log2, time_scale),
                0.0,
            ),
            MeasureElement::Rest(r) => {
                // A rest has no notehead/accidental, but reuse the notehead rod
                // as the glyph-extent estimate; dots still apply.
                (
                    event_rod(0.0, r.dots, config),
                    spring(r.duration_log2, time_scale),
                    0.0,
                )
            }
            // A spacer occupies exactly the room of a rest of its duration,
            // so an empty bar is as wide as the same bar holding a rest.
            MeasureElement::Spacer(spacer) => (
                event_rod(0.0, spacer.dots, config),
                spring(spacer.duration_log2, 1.0),
                0.0,
            ),
            MeasureElement::Chord(c) => {
                // A chord shares one stem column (one notehead rod); its stacked
                // accidental columns are its leading accidental extent.
                (
                    note_rod(0.0, c.dots, &c.annotations, config),
                    spring(c.duration_log2, time_scale),
                    0.0,
                )
            }
            MeasureElement::GroupMark(mark) => {
                match mark {
                    GroupMark::TupletStart { spec, .. } => open_tuplets.push(*spec),
                    GroupMark::TupletEnd { .. } => {
                        open_tuplets.pop();
                    }
                    GroupMark::BeamStart { .. } | GroupMark::BeamEnd { .. } => {}
                }
                (0.0, 0.0, 0.0)
            }
            MeasureElement::MultiMeasureRest { .. } => {
                // Occupies the full rhythmic width of the measure as an
                // incompressible block; the renderer draws the H-bar (or
                // church-rest cluster) spanning to the barline. Use whole-note
                // (longest) spring length as the block allocation, but treat it
                // as rod so it neither compresses nor stretches.
                (event_rod(0.0, 0, config) + spring(0, 1.0), 0.0, 0.0)
            }
            // An invisible barline marks a position (a break point or an
            // unmarked end) without taking any space.
            MeasureElement::Barline(style) if !style.is_visible() => (0.0, 0.0, 0.0),
            MeasureElement::Barline(_) => (config.barline_width, 0.0, 0.0),
        };

        let mut leading = element_left_extent(elem, config)
            + match (elem, previous) {
                (MeasureElement::TimeSignature(_), None | Some(MeasureElement::Barline(_))) => {
                    config.time_sig_change_margin
                }
                _ => 0.0,
            };
        if let Some((lyrics, scale)) = event_lyrics(elem) {
            // A hard gap is needed only between consecutive pitched events
            // that actually print syllables on the same numbered verse. Use
            // the same serif width estimates as text scripts; this keeps the
            // rod incompressible when a system is justified or squeezed.
            let previous_rhythm = positioned.iter().rev().find(|p: &&PositionedElement| {
                matches!(
                    p.element,
                    MeasureElement::Note(_)
                        | MeasureElement::Chord(_)
                        | MeasureElement::Rest(_)
                        | MeasureElement::Spacer(_)
                        | MeasureElement::Barline(_)
                )
            });
            if let Some((previous, (prior_lyrics, prior_scale))) =
                previous_rhythm.and_then(|p| event_lyrics(&p.element).map(|lyrics| (p, lyrics)))
            {
                let prior_center = previous.x + config.notehead_rod * prior_scale * 0.5;
                // Springs can shrink to zero during system justification;
                // do not count the previous event's natural spring as
                // collision clearance that must remain incompressible.
                let this_center = x + leading - previous.spring + config.notehead_rod * scale * 0.5;
                let mut extra_gap = 0.0_f64;
                for lyric in lyrics.iter().filter(|lyric| !lyric.syllable.skip) {
                    if let Some(prior) = prior_lyrics
                        .iter()
                        .find(|prior| prior.verse == lyric.verse && !prior.syllable.skip)
                    {
                        let font_size = LYRIC_FONT_SIZE_SS * config.staff_space;
                        let width_before = crate::layout::text_script::estimate_text_width(
                            &prior.syllable.text,
                            font_size,
                            prior.style,
                        );
                        let width_after = crate::layout::text_script::estimate_text_width(
                            &lyric.syllable.text,
                            font_size,
                            lyric.style,
                        );
                        let separation =
                            (width_before + width_after) * 0.5 + 0.5 * config.staff_space;
                        extra_gap = extra_gap.max(prior_center + separation - this_center);
                    }
                }
                leading += extra_gap.max(0.0);
            }
        }
        previous = Some(elem);
        x += leading;
        reserved_leading_gaps += leading;
        let width = rod + spr;
        positioned.push(PositionedElement {
            x,
            element: elem.clone(),
            width,
            rod,
            spring: spr,
        });
        x += width + trailing;
    }

    // A bar holding nothing visible (only spacers) keeps a minimum rhythmic
    // width, so an empty measure still reads as a bar rather than collapsing
    // to a sliver between two barlines. The deficit widens the last spacer's
    // rod and shifts whatever follows it.
    let spacer_only = positioned
        .iter()
        .any(|p| matches!(p.element, MeasureElement::Spacer(_)))
        && !positioned.iter().any(|p| {
            matches!(
                p.element,
                MeasureElement::Note(_)
                    | MeasureElement::Rest(_)
                    | MeasureElement::Chord(_)
                    | MeasureElement::MultiMeasureRest { .. }
            )
        });
    if spacer_only {
        let spacer_width: f64 = positioned
            .iter()
            .filter(|p| matches!(p.element, MeasureElement::Spacer(_)))
            .map(|p| p.width)
            .sum();
        let deficit = config.empty_measure_min_width - spacer_width;
        if deficit > 0.0 {
            let last = positioned
                .iter()
                .rposition(|p| matches!(p.element, MeasureElement::Spacer(_)))
                .expect("spacer_only implies a spacer");
            positioned[last].rod += deficit;
            positioned[last].width += deficit;
            for p in &mut positioned[last + 1..] {
                p.x += deficit;
            }
            x += deficit;
        }
    }

    let total_rod: f64 = positioned.iter().map(|p| p.rod).sum::<f64>()
        + positioned
            .iter()
            .zip(elements.iter())
            .map(|(_p, e)| trailing_padding(e, config))
            .sum::<f64>()
        + reserved_leading_gaps;
    let total_spring: f64 = positioned.iter().map(|p| p.spring).sum();

    MeasureLayout {
        total_width: x,
        total_rod,
        total_spring,
        elements: positioned,
    }
}

/// Trailing padding that sits after a prefix element but outside its `width`
/// (clef/key-sig/time-sig). Counted as incompressible rod at the measure level.
pub(crate) fn trailing_padding(elem: &MeasureElement, config: &MeasureLayoutConfig) -> f64 {
    match elem {
        MeasureElement::Clef(_) => config.clef_padding,
        MeasureElement::KeySignature(key) => {
            let count = match key {
                KeySignature::Sharps(n) | KeySignature::Flats(n) => *n as f64,
                KeySignature::Open => 0.0,
            };
            if count * config.key_sig_accidental_width > 0.0 {
                config.key_sig_padding
            } else {
                0.0
            }
        }
        MeasureElement::TimeSignature(_) => config.time_sig_padding,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::notation::clef::Clef;

    fn test_config() -> MeasureLayoutConfig {
        MeasureLayoutConfig::from_staff_space(250.0)
    }

    #[test]
    fn empty_measure() {
        let layout = layout_measure(&[], &test_config());
        assert!(layout.elements.is_empty());
        assert!((layout.total_width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_quarter_note() {
        let cfg = test_config();
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        })];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 1);
        assert!((layout.elements[0].x - 0.0).abs() < f64::EPSILON);
        // Single note: it is the shortest, so duration = 1.0 and spring = k.
        // Width = rod + spring; rod is the bare notehead rod (no accidental/dots).
        let el = &layout.elements[0];
        let expected_rod = event_rod(0.0, 0, &cfg);
        let expected_spring = cfg.spring_constant;
        assert!((el.rod - expected_rod).abs() < f64::EPSILON, "rod");
        assert!((el.spring - expected_spring).abs() < f64::EPSILON, "spring");
        assert!(
            (el.width - (el.rod + el.spring)).abs() < f64::EPSILON,
            "width must equal rod + spring"
        );
    }

    #[test]
    fn clef_then_note() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 2);
        // The clef origin sits after its left margin; the note follows the
        // clef's ink (Bravura gClef: 2.684 ss right of the origin) plus padding.
        assert!((layout.elements[0].x - cfg.clef_left_margin).abs() < f64::EPSILON);
        let expected_x = cfg.clef_left_margin + 2.684 * cfg.staff_space + cfg.clef_padding;
        assert!(
            (layout.elements[1].x - expected_x).abs() < 1e-6,
            "note should start after clef, got {} expected {}",
            layout.elements[1].x,
            expected_x,
        );
    }

    #[test]
    fn full_prefix_ordering() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Sharps(3)),
            MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 5);
        // Each element's x should be strictly greater than or equal to the previous
        for i in 1..layout.elements.len() {
            assert!(
                layout.elements[i].x >= layout.elements[i - 1].x,
                "element {} x={} should be >= element {} x={}",
                i,
                layout.elements[i].x,
                i - 1,
                layout.elements[i - 1].x,
            );
        }
        // Total width should be positive
        assert!(layout.total_width > 0.0);
    }

    #[test]
    fn proportional_spacing_half_vs_quarter() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 1, // half note
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 2, // quarter note
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Half note (log2=1) should get more space than quarter (log2=2)
        assert!(
            layout.elements[0].width > layout.elements[1].width,
            "half note (w={}) should be wider than quarter (w={})",
            layout.elements[0].width,
            layout.elements[1].width,
        );
    }

    #[test]
    fn proportional_spacing_whole_vs_eighth() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 0, // whole note
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 3, // eighth note
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // The spring is the duration-driven part. A whole note is 8x an eighth
        // in duration, so its spring is 8^c times larger (c = 0.6 → ~3.48x).
        // Rods are equal (same bare notehead), so the spring ratio is the clean
        // measure of duration proportionality.
        let spring_ratio = layout.elements[0].spring / layout.elements[1].spring;
        assert!(
            spring_ratio > 3.0,
            "whole-note spring should be >3x an eighth's, got ratio {}",
            spring_ratio,
        );
        // Rods identical (no accidental, no dots, single notehead each).
        assert!((layout.elements[0].rod - layout.elements[1].rod).abs() < f64::EPSILON);
    }

    #[test]
    fn equal_durations_get_equal_spacing() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 0,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 8,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert!((layout.elements[0].width - layout.elements[1].width).abs() < f64::EPSILON);
        assert!((layout.elements[1].width - layout.elements[2].width).abs() < f64::EPSILON);
        // Second note starts at first note's x + width
        assert!((layout.elements[1].x - layout.elements[0].width).abs() < f64::EPSILON,);
    }

    #[test]
    fn key_sig_open_adds_no_space() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::KeySignature(KeySignature::Open),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Open key signature should not push the note forward
        assert!(
            (layout.elements[1].x - 0.0).abs() < f64::EPSILON,
            "note should start at x=0 after open key sig, got {}",
            layout.elements[1].x,
        );
    }

    #[test]
    fn barline_at_end() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 2);
        // Barline starts after note's width
        assert!(layout.elements[1].x > 0.0);
        assert!((layout.elements[1].width - cfg.barline_width).abs() < f64::EPSILON);
    }

    #[test]
    fn total_width_is_sum_of_all() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Last element's x + width should equal total_width
        let last = layout.elements.last().unwrap();
        assert!(
            (last.x + last.width - layout.total_width).abs() < f64::EPSILON,
            "total_width {} should equal last element end {}",
            layout.total_width,
            last.x + last.width,
        );
    }

    #[test]
    fn rest_uses_proportional_spacing() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Rest(RestEvent {
                duration_log2: 1, // half rest
                dots: 0,
                annotations: Default::default(),
            }),
            MeasureElement::Rest(RestEvent {
                duration_log2: 2, // quarter rest
                dots: 0,
                annotations: Default::default(),
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert!(
            layout.elements[0].width > layout.elements[1].width,
            "half rest should be wider than quarter rest"
        );
    }

    #[test]
    fn spring_rest_length_shortest_note_is_constant() {
        // The shortest note (duration == 1.0) has spring == k.
        let s = spring_rest_length(3, 3, 1000.0, 0.6);
        assert!((s - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spring_rest_length_double_duration_scales_by_two_to_c() {
        // A note twice as long → duration 2.0 → spring = k · 2^c.
        let s = spring_rest_length(2, 3, 1000.0, 0.6);
        let expected = 1000.0 * 2.0_f64.powf(0.6);
        assert!((s - expected).abs() < 1e-9);
    }

    #[test]
    fn spring_rest_length_quadruple_duration() {
        // Four times as long → duration 4.0 → spring = k · 4^c.
        let s = spring_rest_length(1, 3, 1000.0, 0.6);
        let expected = 1000.0 * 4.0_f64.powf(0.6);
        assert!((s - expected).abs() < 1e-9);
    }

    #[test]
    fn width_equals_rod_plus_spring_for_every_element() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Sharps(2)),
            MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 3,
                denominator: 4,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 1,
                accidental: Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalSharp)),
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Rest(RestEvent {
                duration_log2: 3,
                dots: 0,
                annotations: Default::default(),
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        for el in &layout.elements {
            assert!(
                (el.width - (el.rod + el.spring)).abs() < f64::EPSILON,
                "width {} != rod {} + spring {}",
                el.width,
                el.rod,
                el.spring,
            );
        }
        // total_width == total_rod + total_spring
        assert!(
            (layout.total_width - (layout.total_rod + layout.total_spring)).abs() < 1e-6,
            "total_width {} != total_rod {} + total_spring {}",
            layout.total_width,
            layout.total_rod,
            layout.total_spring,
        );
    }

    #[test]
    fn prefix_and_barline_are_pure_rod() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Flats(2)),
            MeasureElement::TimeSignature(TimeSignatureKind::Common),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        for el in &layout.elements {
            assert!(
                el.spring.abs() < f64::EPSILON,
                "prefix/barline element should have zero spring, got {}",
                el.spring,
            );
        }
        assert!(layout.total_spring.abs() < f64::EPSILON);
    }

    #[test]
    fn accidental_reserves_space_before_the_note_and_dots_widen_its_rod() {
        let cfg = test_config();
        let plain = layout_measure(
            &[MeasureElement::Note(NoteEvent {
                staff_position: 0,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            })],
            &cfg,
        );
        let adorned = layout_measure(
            &[MeasureElement::Note(NoteEvent {
                staff_position: 0,
                duration_log2: 2,
                dots: 2,
                accidental: Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalFlat)),
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            })],
            &cfg,
        );
        // Same duration (and it is the only/shortest note in each) → same spring.
        assert!((plain.elements[0].spring - adorned.elements[0].spring).abs() < f64::EPSILON);
        // The accidental sits left of the notehead: the note starts after it…
        assert_eq!(plain.elements[0].x, 0.0);
        assert_eq!(adorned.elements[0].x, cfg.accidental_rod);
        // …while only the dots widen the note's own rod.
        assert!(
            ((adorned.elements[0].rod - plain.elements[0].rod) - 2.0 * cfg.dot_rod).abs()
                < f64::EPSILON,
        );
        // Both are incompressible measure rod.
        let expected_delta = cfg.accidental_rod + 2.0 * cfg.dot_rod;
        assert!(((adorned.total_rod - plain.total_rod) - expected_delta).abs() < 1e-9);
        assert!(((adorned.total_width - plain.total_width) - expected_delta).abs() < 1e-9);
    }

    #[test]
    fn multiple_key_sig_accidentals_widen() {
        let cfg = test_config();
        let layout3 = layout_measure(
            &[
                MeasureElement::KeySignature(KeySignature::Sharps(3)),
                MeasureElement::Barline(BarlineStyle::Single),
            ],
            &cfg,
        );
        let layout5 = layout_measure(
            &[
                MeasureElement::KeySignature(KeySignature::Sharps(5)),
                MeasureElement::Barline(BarlineStyle::Single),
            ],
            &cfg,
        );
        assert!(
            layout5.total_width > layout3.total_width,
            "5 sharps measure should be wider than 3 sharps"
        );
    }

    #[test]
    fn config_from_staff_space_scales() {
        let cfg1 = MeasureLayoutConfig::from_staff_space(100.0);
        let cfg2 = MeasureLayoutConfig::from_staff_space(200.0);
        assert!((cfg2.clef_left_margin - 2.0 * cfg1.clef_left_margin).abs() < f64::EPSILON);
        assert!((cfg2.spring_constant - 2.0 * cfg1.spring_constant).abs() < f64::EPSILON);
        assert!((cfg2.notehead_rod - 2.0 * cfg1.notehead_rod).abs() < f64::EPSILON);
        // c is dimensionless and does not scale with staff space.
        assert!((cfg2.spacing_exponent - cfg1.spacing_exponent).abs() < f64::EPSILON);
    }

    // ---- Phase 4 calibration locks ----
    //
    // These pin the two properties the calibration sweep
    // (`examples/spacing_calibration.rs`) established. See the
    // 2026-08-06 entry in docs/ENGRAVER-PROGRESS.md for the rationale.

    #[test]
    fn calibrated_defaults_are_the_locked_values() {
        // c = 0.6 is the port plan's §6 midpoint of the 0.5-0.7 empirical
        // range; k = 1.0·ss. Changing either shifts every spacing-sensitive
        // golden, so the values are asserted rather than left implicit.
        let cfg = MeasureLayoutConfig::from_staff_space(250.0);
        assert!((cfg.spacing_exponent - 0.6).abs() < f64::EPSILON);
        assert!((cfg.spring_constant - 250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rod_alone_prevents_notehead_collision_at_the_spring_floor() {
        // The calibration sweep's key structural finding: because every
        // rhythmic rod includes `min_rod_padding + notehead_rod`, adjacent
        // notehead centers stay at least `min_rod_padding` further apart than
        // one notehead is wide — even at the `s = 0` spring floor, the worst
        // case the system layer can produce. Collision avoidance therefore
        // does not depend on the tuning of c or k.
        let cfg = MeasureLayoutConfig::from_staff_space(250.0);
        let per_event_rod = event_rod(0.0, 0, &cfg);
        assert!(
            per_event_rod > cfg.notehead_rod,
            "rod ({per_event_rod}) must exceed notehead width ({}) so fully \
             collapsed springs still cannot collide",
            cfg.notehead_rod
        );
        assert!(
            (per_event_rod - cfg.notehead_rod - cfg.min_rod_padding).abs() < f64::EPSILON,
            "the collision margin is exactly min_rod_padding"
        );
    }

    #[test]
    fn longer_notes_get_sublinear_extra_advance() {
        // Proportionality sanity: a half note in a measure whose shortest note
        // is an eighth (4x the duration) must take more room than the eighth,
        // but far less than 4x — strict proportionality reads badly and wastes
        // width. At the locked defaults the realized ratio is ~1.5x.
        let cfg = MeasureLayoutConfig::from_staff_space(250.0);
        let note = |duration_log2| {
            MeasureElement::Note(NoteEvent {
                staff_position: 0,
                duration_log2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            })
        };
        let layout = layout_measure(&[note(1), note(3)], &cfg);
        let ratio = layout.elements[0].width / layout.elements[1].width;
        assert!(
            ratio > 1.0 && ratio < 2.5,
            "half:eighth advance ratio {ratio} outside the engraving-practice band"
        );
    }

    #[test]
    fn breve_advance_is_one_duration_doubling_beyond_whole() {
        // A breve lasts twice a whole note. With identical rods, its spring
        // must be the spacing model's duration function evaluated one doubling
        // further, pushing the following event right by exactly the extra
        // spring — for notes, rests, chords, and tuplet members alike.
        let cfg = test_config();
        fn note(duration_log2: i8) -> MeasureElement {
            MeasureElement::Note(NoteEvent {
                staff_position: 2,
                duration_log2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            })
        }
        fn single(element: MeasureElement) -> (Vec<MeasureElement>, usize) {
            (vec![element], 0)
        }
        let makers: [fn(i8) -> (Vec<MeasureElement>, usize); 4] = [
            |duration_log2| single(note(duration_log2)),
            |duration_log2| {
                single(MeasureElement::Rest(RestEvent {
                    duration_log2,
                    dots: 0,
                    annotations: NoteAnnotations::default(),
                }))
            },
            |duration_log2| {
                single(MeasureElement::Chord(ChordEvent {
                    staff_positions: vec![2, 4, 6],
                    duration_log2,
                    dots: 0,
                    accidentals: vec![None; 3],
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }))
            },
            |duration_log2| {
                (
                    vec![
                        MeasureElement::GroupMark(GroupMark::TupletStart {
                            spec: TupletSpec::new(3, 2),
                            continued: false,
                        }),
                        note(duration_log2),
                        MeasureElement::GroupMark(GroupMark::TupletEnd { continues: false }),
                    ],
                    1,
                )
            },
        ];
        let doubling = 2.0_f64.powf(cfg.spacing_exponent);
        for make in makers {
            let layout_with = |duration_log2| {
                let (mut elements, event) = make(duration_log2);
                elements.push(note(2));
                let next = elements.len() - 1;
                (layout_measure(&elements, &cfg), event, next)
            };
            let (breve, event, next) = layout_with(-1);
            let (whole, ..) = layout_with(0);
            let (breve_event, whole_event) = (&breve.elements[event], &whole.elements[event]);
            assert!((breve_event.rod - whole_event.rod).abs() < 1e-9);
            assert!(
                (breve_event.spring / whole_event.spring - doubling).abs() < 1e-9,
                "breve spring {} must be whole spring {} times 2^exponent",
                breve_event.spring,
                whole_event.spring
            );
            let breve_next_x = breve.elements[next].x;
            let whole_next_x = whole.elements[next].x;
            assert!(breve_next_x > whole_next_x);
            assert!(
                ((breve_next_x - whole_next_x) - (breve_event.spring - whole_event.spring)).abs()
                    < 1e-9
            );
        }
        let breve_note = layout_measure(&[note(-1), note(2)], &cfg);
        let expected = spring_rest_length(-1, 2, cfg.spring_constant, cfg.spacing_exponent);
        assert!((breve_note.elements[0].spring - expected).abs() < 1e-9);
    }

    #[test]
    fn semantic_notehead_styles_resolve_to_exact_duration_glyphs() {
        use smufl::Glyph;

        let cases = [
            (
                NoteheadStyle::Normal,
                [
                    Glyph::NoteheadDoubleWhole,
                    Glyph::NoteheadWhole,
                    Glyph::NoteheadHalf,
                    Glyph::NoteheadBlack,
                ],
            ),
            (
                NoteheadStyle::Diamond,
                [
                    Glyph::NoteheadDiamondDoubleWhole,
                    Glyph::NoteheadDiamondWhole,
                    Glyph::NoteheadDiamondHalf,
                    Glyph::NoteheadDiamondBlack,
                ],
            ),
            (
                NoteheadStyle::X,
                [
                    Glyph::NoteheadXDoubleWhole,
                    Glyph::NoteheadXWhole,
                    Glyph::NoteheadXHalf,
                    Glyph::NoteheadXBlack,
                ],
            ),
            (
                NoteheadStyle::CircleX,
                [
                    Glyph::NoteheadCircleXDoubleWhole,
                    Glyph::NoteheadCircleXWhole,
                    Glyph::NoteheadCircleXHalf,
                    Glyph::NoteheadCircleX,
                ],
            ),
            (
                NoteheadStyle::Slash,
                [
                    Glyph::NoteheadSlashWhiteDoubleWhole,
                    Glyph::NoteheadSlashWhiteWhole,
                    Glyph::NoteheadSlashWhiteHalf,
                    Glyph::NoteheadSlashVerticalEnds,
                ],
            ),
            (
                NoteheadStyle::Square,
                [
                    // SMuFL has no double-whole square percussion head.
                    Glyph::NoteheadSquareWhite,
                    Glyph::NoteheadSquareWhite,
                    Glyph::NoteheadSquareWhite,
                    Glyph::NoteheadSquareBlack,
                ],
            ),
        ];
        for (style, [breve, whole, half, filled]) in cases {
            assert_eq!(style.glyph(-1), breve);
            assert_eq!(style.glyph(0), whole);
            assert_eq!(style.glyph(1), half);
            assert_eq!(style.glyph(2), filled);
            assert_eq!(style.glyph(7), filled);
        }
    }
}
