//! Shared semantic timeline for coordinated standard guitar notation and tablature.

use std::collections::{BTreeMap, HashMap};

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use crate::error::EngraverError;
use crate::font::{EngravingConfig, MusicFont};
use crate::layout::barline::BarlineStyle;
use crate::layout::bar_number::MeasureNumbering;
use crate::layout::beam::beam_group_note_x_offsets;
use crate::layout::bend_gesture::{BendFragment, BendSegmentLayout, BendSegmentPhase, BendView};
use crate::layout::key_signature::KeySignature;
#[cfg(test)]
use crate::layout::measure::MeasureLayout;
use crate::layout::measure::{MeasureElement, NoteAnnotations, NoteheadStyle};
use crate::layout::measure_meta::LineBreak;
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::staff::StaffLayout;
use crate::layout::system::{ClefKind, SystemLayout};
use crate::layout::tab::{layout_fret_number, layout_muted_string, TabStaffLayout};
use crate::layout::tab_beam::{layout_tab_beam_group, TabBeamedNote};
use crate::layout::tab_hammer::{layout_tab_legato, LegatoKind};
use crate::layout::tab_harmonic::layout_tab_harmonic;
use crate::layout::tab_let_ring::{
    layout_tab_let_ring, layout_tab_let_ring_dash, LET_RING_DASH_OFFSET_SS,
};
use crate::layout::tab_palm_mute::{
    layout_tab_palm_mute, layout_tab_palm_mute_dash, PALM_MUTE_DASH_OFFSET_SS,
};
use crate::layout::tab_rhythm::layout_tab_rhythm;
use crate::layout::tab_slide::layout_tab_slide;
use crate::layout::tab_vibrato::{layout_tab_vibrato, VibratoKind};
use crate::render::bend_gesture_renderer::draw_bend_segment;
use crate::render::note_renderer::notehead_advance;
use crate::render::tab_beam_renderer::draw_tab_beam_group;
use crate::render::tab_hammer_renderer::draw_tab_legato;
use crate::render::tab_harmonic_renderer::draw_tab_harmonic;
use crate::render::tab_let_ring_renderer::{draw_tab_let_ring, draw_tab_let_ring_dash};
use crate::render::tab_palm_mute_renderer::{draw_tab_palm_mute, draw_tab_palm_mute_dash};
use crate::render::tab_renderer::draw_fret_number;
use crate::render::tab_rhythm_renderer::draw_tab_rhythm;
use crate::render::tab_slide_renderer::draw_tab_slide;
use crate::render::tab_vibrato_renderer::draw_tab_vibrato;
use crate::render::{RectStyle, SvgWriter, TextStyle};
use smufl::Glyph;

use super::multi_staff::MultiStaffScore;
use super::ScoreBuilder;

pub(crate) const GUITAR_TAB_GAP_SS: f64 = 14.5;
pub(crate) const GUITAR_BARLINE_WIDTH_SS: f64 = 0.8;
// The tallest annotation-lane occupant is a barre glyph (about 1.97 ss in
// bundled Bravura) drawn 0.5 ss above its lane line. A 2.7 ss pitch leaves a
// visible inter-lane gap while the 4.2 ss first-lane extent covers its label
// and lower edge.
const GUITAR_ANNOTATION_LANE_BASE_SS: f64 = 3.4;
const GUITAR_ANNOTATION_LANE_PITCH_SS: f64 = 2.7;
const GUITAR_ANNOTATION_FIRST_LANE_HEIGHT_SS: f64 = 4.2;
const GUITAR_LEGEND_ROW_HEIGHT_SS: f64 = 0.75;
const GUITAR_LEGEND_STRUM_SCALE: f64 = 0.3;

/// Stable identity for one event in a [`GuitarScore`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GuitarEventId(u64);

impl GuitarEventId {
    /// Numeric identity, useful when associating external score-local metadata.
    pub fn get(self) -> u64 {
        self.0
    }
}
/// A bend's sounding pitch, including a sub-semitone remainder.
///
/// `pitch` supplies the spelling and MIDI semitone; `cents` is the
/// additional 0–99 cents above it. The bend amount is derived from this
/// target and the source event's physical pitch, so the two cannot disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BendPitch {
    pitch: Pitch,
    cents: u8,
}

impl BendPitch {
    /// Construct a sounding bend pitch.
    pub fn new(pitch: Pitch, cents: u8) -> Result<Self, GuitarScoreError> {
        if cents > 99 {
            return Err(GuitarScoreError::InvalidBendCents { cents });
        }
        Ok(Self { pitch, cents })
    }

    /// Construct an exact semitone pitch with no microtonal remainder.
    pub fn exact(pitch: Pitch) -> Self {
        Self { pitch, cents: 0 }
    }

    /// Return the spelled semitone component.
    pub fn pitch(self) -> Pitch {
        self.pitch
    }

    /// Return the additional cents above [`Self::pitch`].
    pub fn cents(self) -> u8 {
        self.cents
    }

    fn total_cents(self) -> u16 {
        self.pitch.midi_note as u16 * 100 + self.cents as u16
    }
}

/// One exact point on the shared guitar-event timeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GuitarMoment {
    /// The onset of an existing event.
    Onset(GuitarEventId),
    /// A written offset strictly inside an existing event. Tuplet scaling is
    /// inherited from that event's group when moments are compared.
    After {
        event: GuitarEventId,
        offset: Duration,
    },
}

impl GuitarMoment {
    /// Address an event onset.
    pub fn onset(event: GuitarEventId) -> Self {
        Self::Onset(event)
    }

    /// Address a written duration strictly inside an event.
    pub fn after(event: GuitarEventId, offset: Duration) -> Self {
        Self::After { event, offset }
    }

    fn event(self) -> GuitarEventId {
        match self {
            Self::Onset(event) | Self::After { event, .. } => event,
        }
    }
}

/// Whether the source attack precedes the bend or sounds an already bent string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BendAttack {
    PickThenBend,
    PreBent,
}

/// Optional descent from a held bend target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BendRelease {
    start: GuitarMoment,
    end: GuitarMoment,
    target: BendPitch,
}

impl BendRelease {
    /// Define the held-target departure, release landing, and sounding target.
    pub fn new(start: GuitarMoment, end: GuitarMoment, target: BendPitch) -> Self {
        Self { start, end, target }
    }

    /// Return the moment at which the held target begins descending.
    pub fn start(self) -> GuitarMoment {
        self.start
    }

    /// Return the moment at which the release reaches its target.
    pub fn end(self) -> GuitarMoment {
        self.end
    }

    /// Return the sounding pitch at the end of the release.
    pub fn target(self) -> BendPitch {
        self.target
    }
}

/// A timed bend owned by one physical string on the shared guitar timeline.
///
/// The source remains the physical fretted pitch. `target` is the sounding
/// pitch reached at `arrival`; an optional release defines the held interval
/// and release landing, while reattacks identify later attacks of that same
/// held string/fret.
#[derive(Clone, Debug, PartialEq)]
pub struct BendGesture {
    source: GuitarEventId,
    string: u8,
    attack: BendAttack,
    onset: GuitarMoment,
    target: BendPitch,
    arrival: GuitarMoment,
    release: Option<BendRelease>,
    reattacks: Vec<GuitarEventId>,
}

impl BendGesture {
    /// Create a picked bend whose motion starts at the source onset.
    pub fn new(
        source: GuitarEventId,
        string: u8,
        target: BendPitch,
        arrival: GuitarMoment,
    ) -> Self {
        Self {
            source,
            string,
            attack: BendAttack::PickThenBend,
            onset: GuitarMoment::Onset(source),
            target,
            arrival,
            release: None,
            reattacks: Vec::new(),
        }
    }

    /// Create a bend already held at the source attack.
    pub fn pre_bend(source: GuitarEventId, string: u8, target: BendPitch) -> Self {
        Self {
            source,
            string,
            attack: BendAttack::PreBent,
            onset: GuitarMoment::Onset(source),
            target,
            arrival: GuitarMoment::Onset(source),
            release: None,
            reattacks: Vec::new(),
        }
    }

    /// Delay the start of a picked bend to a moment within the source event.
    pub fn starting_at(mut self, onset: GuitarMoment) -> Self {
        self.onset = onset;
        self
    }

    /// Add a held-target interval followed by a release.
    pub fn with_release(mut self, release: BendRelease) -> Self {
        self.release = Some(release);
        self
    }

    /// Record a later attack while the target remains held.
    pub fn reattacked_at(mut self, event: GuitarEventId) -> Self {
        self.reattacks.push(event);
        self
    }

    /// Return the source event.
    pub fn source(&self) -> GuitarEventId {
        self.source
    }

    /// Return the affected physical string.
    pub fn string(&self) -> u8 {
        self.string
    }

    /// Return whether the source is picked before or after reaching its target.
    pub fn attack(&self) -> BendAttack {
        self.attack
    }

    /// Return the beginning of upward motion.
    pub fn onset(&self) -> GuitarMoment {
        self.onset
    }

    /// Return the sounding arrival pitch.
    pub fn target(&self) -> BendPitch {
        self.target
    }

    /// Return the target-arrival moment.
    pub fn arrival(&self) -> GuitarMoment {
        self.arrival
    }

    /// Return the optional hold/release definition.
    pub fn release(&self) -> Option<BendRelease> {
        self.release
    }

    /// Return held-state reattack events in chronological order.
    pub fn reattacks(&self) -> &[GuitarEventId] {
        &self.reattacks
    }
}

/// A sounding pitch paired with its physical string/fret realization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrettedPitch {
    pub pitch: Pitch,
    pub string: u8,
    pub fret: u8,
}

impl FrettedPitch {
    pub fn new(pitch: Pitch, string: u8, fret: u8) -> Self {
        Self {
            pitch,
            string,
            fret,
        }
    }
}

/// Input for one rhythmic event inside a beam or tuplet group.
///
/// Every variant carries its complete semantic identity; grouped slashes,
/// dead attacks, and percussion therefore use the same timeline and renderer
/// pipeline as standalone events.
#[derive(Clone, Debug, PartialEq)]
pub enum GuitarEventSpec {
    /// One or more simultaneously realized sounding pitches.
    Pitched {
        /// Sounding pitches and physical string/fret realizations. The vector
        /// is validated as one physical chord, including distinct strings.
        pitches: Vec<FrettedPitch>,
        /// Exact written duration shared by every chord tone.
        duration: Duration,
    },
    /// One or more struck, fully damped strings.
    Dead {
        /// Distinct 1-based string numbers.
        strings: Vec<u8>,
        /// Exact written duration.
        duration: Duration,
    },
    /// Unvoiced rhythmic slash.
    Slash {
        /// Exact written duration.
        duration: Duration,
    },
    /// Unpitched guitar percussion.
    Percussion {
        /// Physical surface or material struck.
        target: PercussionTarget,
        /// Exact written duration.
        duration: Duration,
    },
}

impl GuitarEventSpec {
    /// Construct a grouped single-note event.
    pub fn pitched(pitch: Pitch, duration: Duration, string: u8, fret: u8) -> Self {
        Self::Pitched {
            pitches: vec![FrettedPitch::new(pitch, string, fret)],
            duration,
        }
    }

    /// Construct a grouped physically realized chord.
    ///
    /// Pitches are ordered by the caller and must target distinct strings.
    /// Tuning, capo, fret, and duplicate-string validation occurs when the
    /// event is added to a [`GuitarScore`].
    pub fn chord(pitches: Vec<FrettedPitch>, duration: Duration) -> Self {
        Self::Pitched { pitches, duration }
    }

    fn duration(&self) -> Duration {
        match self {
            Self::Pitched { duration, .. }
            | Self::Dead { duration, .. }
            | Self::Slash { duration }
            | Self::Percussion { duration, .. } => *duration,
        }
    }
}

/// Fretting-hand finger assigned to a played string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftHandFinger {
    One,
    Two,
    Three,
    Four,
}

impl LeftHandFinger {
    fn label(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
        }
    }
}

/// Direction of a plectrum stroke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickStroke {
    Down,
    Up,
}

/// Classical right-hand plucking finger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluckingFinger {
    Thumb,
    Index,
    Middle,
    Ring,
    Little,
}

/// Hand used for a tapped attack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TappingHand {
    /// Fretting-hand tap.
    Fretting,
    /// Picking-hand tap.
    Picking,
}

/// Production method for one notated harmonic.
///
/// Natural, artificial, and tapped harmonics are accepted only at integer-fret
/// nodes represented by offsets 4, 5, 7, 9, 12, 16, 19, 24, or 28. The
/// supported partials sound 12 semitones above at offset 12, 19 at offsets
/// 7/19, 24 at offsets 5/24, and 28 at offsets 4/9/16/28; the sounding pitch
/// is derived from that partial rather than from the raw touch-fret distance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarmonicKind {
    /// Open-string natural harmonic at the displayed physical fret/node.
    Natural,
    /// Stopped note touched at an explicit fret/node.
    Artificial {
        /// Physical touch fret or node, not the stopped fret.
        touch_fret: u8,
    },
    /// Picking-hand pinch harmonic; the node is intentionally unspecified.
    Pinch,
    /// Harmonic sounded by tapping an explicit fret/node.
    Tapped {
        /// Physical tap fret or node.
        touch_fret: u8,
        /// Hand performing the tap.
        hand: TappingHand,
    },
}

/// A harmonic's production method and explicit sounding pitch.
///
/// The event's [`FrettedPitch`] retains the stopped/displayed physical
/// realization while [`Self::sounding_pitch`] is projected to standard
/// notation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Harmonic {
    /// Physical production method.
    pub kind: HarmonicKind,
    /// Concert pitch heard from the harmonic.
    pub sounding_pitch: Pitch,
}

impl Harmonic {
    /// Describe a natural harmonic at the event's displayed node.
    pub fn natural(sounding_pitch: Pitch) -> Self {
        Self {
            kind: HarmonicKind::Natural,
            sounding_pitch,
        }
    }

    /// Describe an artificial harmonic with an explicit touch fret.
    pub fn artificial(sounding_pitch: Pitch, touch_fret: u8) -> Self {
        Self {
            kind: HarmonicKind::Artificial { touch_fret },
            sounding_pitch,
        }
    }

    /// Describe a pinch harmonic with an explicit sounding pitch.
    pub fn pinch(sounding_pitch: Pitch) -> Self {
        Self {
            kind: HarmonicKind::Pinch,
            sounding_pitch,
        }
    }

    /// Describe a tapped harmonic with its touch fret and performing hand.
    pub fn tapped(sounding_pitch: Pitch, touch_fret: u8, hand: TappingHand) -> Self {
        Self {
            kind: HarmonicKind::Tapped { touch_fret, hand },
            sounding_pitch,
        }
    }

    fn abbreviation(self) -> &'static str {
        match self.kind {
            HarmonicKind::Natural => "harm.",
            HarmonicKind::Artificial { .. } => "A.H.",
            HarmonicKind::Pinch => "P.H.",
            HarmonicKind::Tapped { .. } => "T.H.",
        }
    }
}

/// Direction of a chord-wide guitar strum.
///
/// This is intentionally distinct from [`PickStroke`], which describes a
/// plectrum's attack on one string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrumDirection {
    /// Sweep from lower-pitched strings toward higher-pitched strings.
    Down,
    /// Sweep from higher-pitched strings toward lower-pitched strings.
    Up,
}

/// Fretting-hand attack that sounds a note without a new right-hand stroke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftHandArticulation {
    HammerOn,
    PullOff,
}

/// Slap-family attack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlapTechnique {
    /// Thumb slap.
    Thumb,
    /// Finger pop.
    Pop,
}

/// Percussive guitar surface or material.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PercussionTarget {
    /// Guitar body or soundboard.
    Body,
    /// Fretboard surface.
    Fretboard,
    /// Damped strings used percussively.
    Strings,
}

/// Physical source of one guitar attack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackSource {
    /// Plectrum attack.
    Pick(PickStroke),
    /// Finger-plucked attack.
    Pluck(PluckingFinger),
    /// Fretting-hand-only attack.
    LeftHand(LeftHandArticulation),
    /// Tapped attack.
    Tap(TappingHand),
    /// Slap-family attack.
    Slap(SlapTechnique),
}

impl AttackSource {
    fn label(self) -> &'static str {
        match self {
            Self::Pick(PickStroke::Down) => "↓",
            Self::Pick(PickStroke::Up) => "↑",
            Self::Pluck(PluckingFinger::Thumb) => "p",
            Self::Pluck(PluckingFinger::Index) => "i",
            Self::Pluck(PluckingFinger::Middle) => "m",
            Self::Pluck(PluckingFinger::Ring) => "a",
            Self::Pluck(PluckingFinger::Little) => "c",
            Self::LeftHand(LeftHandArticulation::HammerOn) => "LH-H",
            Self::LeftHand(LeftHandArticulation::PullOff) => "LH-P",
            Self::Tap(TappingHand::Fretting) => "T(LH)",
            Self::Tap(TappingHand::Picking) => "T(RH)",
            Self::Slap(SlapTechnique::Thumb) => "slap",
            Self::Slap(SlapTechnique::Pop) => "pop",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Pick(PickStroke::Down) => "down-pick",
            Self::Pick(PickStroke::Up) => "up-pick",
            Self::Pluck(PluckingFinger::Thumb) => "pluck with thumb",
            Self::Pluck(PluckingFinger::Index) => "pluck with index",
            Self::Pluck(PluckingFinger::Middle) => "pluck with middle",
            Self::Pluck(PluckingFinger::Ring) => "pluck with ring",
            Self::Pluck(PluckingFinger::Little) => "pluck with little finger",
            Self::LeftHand(LeftHandArticulation::HammerOn) => "left-hand hammer-on",
            Self::LeftHand(LeftHandArticulation::PullOff) => "left-hand pull-off",
            Self::Tap(TappingHand::Fretting) => "fretting-hand tap",
            Self::Tap(TappingHand::Picking) => "picking-hand tap",
            Self::Slap(SlapTechnique::Thumb) => "thumb slap",
            Self::Slap(SlapTechnique::Pop) => "string pop",
        }
    }
}

/// Optional outline around score-local text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextEnclosure {
    Circle,
    Rectangle,
}

/// Vertical lane for event-anchored text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPlacement {
    Above,
    Below,
}

/// Text attached to a guitar event or explicit event range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuitarText {
    pub text: String,
    pub placement: TextPlacement,
    pub enclosure: Option<TextEnclosure>,
}

impl GuitarText {
    pub fn new(text: impl Into<String>, placement: TextPlacement) -> Self {
        Self {
            text: text.into(),
            placement,
            enclosure: None,
        }
    }

    pub fn enclosed(mut self, enclosure: TextEnclosure) -> Self {
        self.enclosure = Some(enclosure);
        self
    }
}

/// Direction indicated by a position shift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftDirection {
    Up,
    Down,
}

/// Classical guitar position, rendered as a Roman numeral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionLabel {
    pub fret: u8,
    pub shift: Option<ShiftDirection>,
}

impl PositionLabel {
    pub fn new(fret: u8) -> Self {
        Self { fret, shift: None }
    }

    pub fn shifted(mut self, direction: ShiftDirection) -> Self {
        self.shift = Some(direction);
        self
    }
}

/// Ordered, inclusive guitar-string range in 1-based high-to-low numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuitarStringRange {
    /// Highest-pitched included string number.
    pub first: u8,
    /// Lowest-pitched included string number.
    pub last: u8,
}

impl GuitarStringRange {
    /// Construct a contiguous range containing at least two strings.
    pub fn new(first: u8, last: u8) -> Result<Self, GuitarScoreError> {
        if first == 0 || first >= last {
            return Err(GuitarScoreError::InvalidStringRange { first, last });
        }
        Ok(Self { first, last })
    }

    fn contains(self, string: u8) -> bool {
        (self.first..=self.last).contains(&string)
    }
}

/// Whether a barre covers every string or a proper contiguous subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarreKind {
    /// Barre covering every string in the tuning.
    Full,
    /// Barre covering a proper contiguous subset of strings.
    Partial,
}

/// Physical barre held at one fret over a contiguous string range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuitarBarre {
    /// Full or partial coverage.
    pub kind: BarreKind,
    /// Capo-relative fret held by the finger.
    pub fret: u8,
    /// Inclusive strings held by the finger.
    pub strings: GuitarStringRange,
}

impl GuitarBarre {
    /// Construct a barre. Score attachment validates full/partial coverage
    /// against the score's actual string count.
    pub fn new(
        kind: BarreKind,
        fret: u8,
        first_string: u8,
        last_string: u8,
    ) -> Result<Self, GuitarScoreError> {
        if fret == 0 {
            return Err(GuitarScoreError::InvalidBarreFret);
        }
        Ok(Self {
            kind,
            fret,
            strings: GuitarStringRange::new(first_string, last_string)?,
        })
    }
}

/// Event-local guitar annotation. A string target permits independent
/// assignments on chord tones; `None` applies an attack to the whole event.
#[derive(Clone, Debug, PartialEq)]
pub enum GuitarAnnotation {
    /// Vibrato on every realized string in the event.
    Vibrato(VibratoKind),
    /// Harmonic applied to one physically realized string.
    Harmonic {
        /// One-based, high-to-low guitar string number.
        string: u8,
        /// Production method and concert sounding pitch.
        harmonic: Harmonic,
    },
    /// Parenthesized pitched attack on one string or the complete event.
    Ghost {
        /// `None` targets every pitched string in the event.
        string: Option<u8>,
    },
    /// Chord-wide strum direction.
    Strum(StrumDirection),
    /// Render a physically realized chord as one standard-notation comping
    /// slash while retaining its exact TAB frets.
    RhythmicSlash,
    /// Fretting-hand finger assigned to one pitched string.
    LeftHandFinger {
        /// One-based, high-to-low guitar string number.
        string: u8,
        /// Finger used on the target string.
        finger: LeftHandFinger,
    },
    /// Physical attack source for one string or the complete event.
    Attack {
        /// `None` applies to the complete event.
        string: Option<u8>,
        /// Physical attack source.
        source: AttackSource,
    },
    /// Harmony text placed above the standard notation.
    ChordSymbol(String),
    /// Score-local event text.
    Text(GuitarText),
    /// Classical-guitar position at the event.
    Position(PositionLabel),
}

/// TAB technique whose semantic range is delimited by stable event identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuitarSpanKind {
    Slide,
    HammerOn,
    PullOff,
    PalmMute,
    LetRing,
}

/// Non-string-specific annotation attached to an explicit event range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuitarSpanAnnotation {
    /// Classical-guitar position held over a range.
    Position(PositionLabel),
    /// Physically validated full or partial barre.
    Barre(GuitarBarre),
    /// Labeled analytical or practice cell.
    CellBracket {
        /// Cell label.
        label: String,
        /// Optional label enclosure.
        enclosure: Option<TextEnclosure>,
    },
    /// Free text spanning an event range.
    Text(GuitarText),
}

/// One entry in a generated score-local technique legend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TechniqueLegendEntry {
    /// Compact notated symbol.
    pub symbol: &'static str,
    /// Human-readable semantic description.
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GuitarPerformanceAnnotation {
    LeftHandFinger {
        string: u8,
        finger: LeftHandFinger,
    },
    Attack {
        string: Option<u8>,
        source: AttackSource,
    },
    Text(GuitarText),
    Position(PositionLabel),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GuitarEventAnnotations {
    pub(crate) vibrato: Option<VibratoKind>,
    pub(crate) harmonics: Vec<(u8, Harmonic)>,
    pub(crate) ghost_strings: Vec<u8>,
    pub(crate) strum: Option<StrumDirection>,
    pub(crate) chord_symbol: Option<String>,
    pub(crate) rhythmic_slash: bool,
    pub(crate) performance: Vec<GuitarPerformanceAnnotation>,
}

#[derive(Clone, Debug)]
pub(crate) enum GuitarEventKind {
    Pitched(Vec<FrettedPitch>),
    Dead(Vec<u8>),
    Slash,
    Percussion(PercussionTarget),
    Rest,
}

#[derive(Clone, Debug)]
pub(crate) struct GuitarEvent {
    pub(crate) id: GuitarEventId,
    pub(crate) duration: Duration,
    pub(crate) kind: GuitarEventKind,
    pub(crate) annotations: GuitarEventAnnotations,
}

impl GuitarEvent {
    pub(crate) fn strings(&self) -> impl Iterator<Item = u8> + '_ {
        let pitched = match &self.kind {
            GuitarEventKind::Pitched(notes) => Some(notes.as_slice()),
            _ => None,
        };
        let dead = match &self.kind {
            GuitarEventKind::Dead(strings) => Some(strings.as_slice()),
            _ => None,
        };
        pitched
            .into_iter()
            .flatten()
            .map(|note| note.string)
            .chain(dead.into_iter().flatten().copied())
    }
}

#[derive(Clone, Debug)]
pub(crate) enum GuitarGroup {
    Event(GuitarEvent),
    Beam(Vec<GuitarEvent>),
    Tuplet {
        number: u32,
        in_time_of: u32,
        events: Vec<GuitarEvent>,
    },
}

impl GuitarGroup {
    pub(crate) fn events(&self) -> &[GuitarEvent] {
        match self {
            Self::Event(event) => std::slice::from_ref(event),
            Self::Beam(events) | Self::Tuplet { events, .. } => events,
        }
    }

    fn effective_ticks(&self) -> Fraction {
        match self {
            Self::Event(event) => Fraction::new(event.duration.ticks() as u128, 1),
            Self::Beam(events) => Fraction::new(
                events
                    .iter()
                    .map(|event| event.duration.ticks() as u128)
                    .sum(),
                1,
            ),
            Self::Tuplet {
                number,
                in_time_of,
                events,
            } => Fraction::new(
                events
                    .iter()
                    .map(|event| event.duration.ticks() as u128)
                    .sum::<u128>()
                    * *in_time_of as u128,
                *number as u128,
            ),
        }
    }
}

/// A clef directive between two groups of one voice, with its exact onset
/// shared across all voices of the measure.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GuitarClefChange {
    voice: u8,
    after_group: usize,
    onset: Fraction,
    clef: ClefKind,
    after_barline: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct GuitarMeasure {
    pub(crate) voices: BTreeMap<u8, Vec<GuitarGroup>>,
    pub(crate) barline: BarlineStyle,
    /// Line-break permission after this measure.
    pub(crate) line_break: LineBreak,
    /// Meter change at the start of this measure.
    pub(crate) time_signature_change: Option<(u8, u8)>,
    pub(crate) clef_changes: Vec<GuitarClefChange>,
}

#[derive(Clone, Debug)]
pub(crate) enum StoredGuitarSpan {
    Technique { kind: GuitarSpanKind, string: u8 },
    Annotation(GuitarSpanAnnotation),
}

#[derive(Clone, Debug)]
pub(crate) struct GuitarSpan {
    pub(crate) annotation: StoredGuitarSpan,
    pub(crate) start: GuitarEventId,
    pub(crate) end: GuitarEventId,
}

/// A positive capo fret. TAB fret numbers remain relative to this capo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capo(u8);

impl Capo {
    /// Construct a capo. Fret zero is represented by `None` on [`GuitarScore`].
    pub fn new(fret: u8) -> Result<Self, GuitarScoreError> {
        if fret == 0 {
            return Err(GuitarScoreError::InvalidCapoFret);
        }
        Ok(Self(fret))
    }

    /// Return the positive capo fret.
    pub fn fret(self) -> u8 {
        self.0
    }
}

/// Visibility of score-start tuning metadata.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TuningDisplay {
    /// Do not emit a tuning label.
    #[default]
    Hidden,
    /// Emit exact open-string pitches in 1-based high-to-low order.
    Pitches,
    /// Emit the optional friendly name followed by exact open-string pitches.
    NameAndPitches,
}

/// Tuning ordered by guitar string number: index 0 is string 1 (highest).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuitarTuning {
    open_pitches: Vec<Pitch>,
    name: Option<String>,
}

impl GuitarTuning {
    /// Construct an unnamed tuning from exact open pitches ordered high to low.
    pub fn new(open_pitches_high_to_low: Vec<Pitch>) -> Result<Self, GuitarScoreError> {
        if open_pitches_high_to_low.is_empty() {
            return Err(GuitarScoreError::EmptyTuning);
        }
        Ok(Self {
            open_pitches: open_pitches_high_to_low,
            name: None,
        })
    }

    /// Construct a named tuning. The name is presentation metadata; exact
    /// open pitches remain the sole source of tuning semantics.
    pub fn named(
        name: impl Into<String>,
        open_pitches_high_to_low: Vec<Pitch>,
    ) -> Result<Self, GuitarScoreError> {
        if open_pitches_high_to_low.is_empty() {
            return Err(GuitarScoreError::EmptyTuning);
        }
        Ok(Self {
            open_pitches: open_pitches_high_to_low,
            name: Some(name.into()),
        })
    }

    /// Return six-string standard tuning, high E through low E.
    pub fn standard() -> Self {
        Self {
            open_pitches: vec![
                Pitch::new(Note::E, 4),
                Pitch::new(Note::B, 3),
                Pitch::new(Note::G, 3),
                Pitch::new(Note::D, 3),
                Pitch::new(Note::A, 2),
                Pitch::new(Note::E, 2),
            ],
            name: Some(String::from("Standard")),
        }
    }

    /// Return the number of physical strings.
    pub fn line_count(&self) -> u8 {
        self.open_pitches.len() as u8
    }

    /// Return exact open pitches ordered from string 1 downward.
    pub fn open_pitches(&self) -> &[Pitch] {
        &self.open_pitches
    }

    /// Return the optional presentation name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn pitch_at(&self, string: u8, fret: u8) -> Result<u8, GuitarScoreError> {
        let Some(open) = string
            .checked_sub(1)
            .and_then(|index| self.open_pitches.get(index as usize))
        else {
            return Err(GuitarScoreError::InvalidString {
                string,
                string_count: self.line_count(),
            });
        };
        open.midi_note
            .checked_add(fret)
            .filter(|midi| *midi <= 127)
            .ok_or(GuitarScoreError::FretOutOfMidiRange { string, fret })
    }
}
/// Named point in bend validation diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BendPhase {
    Onset,
    Arrival,
    ReleaseStart,
    ReleaseEnd,
    Reattack,
}

/// Validation failures in the shared guitar timeline.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GuitarScoreError {
    #[error("a guitar tuning must contain at least one string")]
    EmptyTuning,
    #[error("string {string} is outside 1..={string_count}")]
    InvalidString { string: u8, string_count: u8 },
    #[error("string range {first}..={last} must contain at least two ordered strings")]
    InvalidStringRange { first: u8, last: u8 },
    #[error("capo fret must be greater than zero")]
    InvalidCapoFret,
    #[error("capo fret {fret} puts string {string} outside the MIDI range")]
    CapoOutOfMidiRange { fret: u8, string: u8 },
    #[error("instrument setup cannot change after the first event")]
    SetupLocked,
    #[error("string {string} fret {fret} exceeds the MIDI pitch range")]
    FretOutOfMidiRange { string: u8, fret: u8 },
    #[error("{pitch} does not match string {string} fret {fret}; expected MIDI {expected_midi}")]
    PitchMismatch {
        pitch: Pitch,
        string: u8,
        fret: u8,
        expected_midi: u8,
    },
    #[error("string {string} appears more than once in one event")]
    DuplicateString { string: u8 },
    #[error("event group must contain at least one event")]
    EmptyGroup,
    #[error("tuplet ratio values must both be greater than zero")]
    InvalidTupletRatio,
    #[error("unknown guitar event id {0}")]
    UnknownEvent(u64),
    #[error("harmonic on event {event} targets unplayed string {string}")]
    HarmonicStringMismatch { event: u64, string: u8 },
    #[error("event {event} string {string} already has a harmonic")]
    DuplicateHarmonic { event: u64, string: u8 },
    #[error("event {event} string {string} already has a ghost annotation")]
    DuplicateGhost { event: u64, string: u8 },
    #[error("harmonic sounding pitch {sounding} is invalid for event {event} string {string}")]
    InvalidHarmonicPitch {
        event: u64,
        string: u8,
        sounding: Pitch,
    },
    #[error("event {event} is already displayed as a rhythmic slash")]
    DuplicateRhythmicSlash { event: u64 },
    #[error(
        "harmonic touch fret {touch_fret} is not a supported node above stopped fret {stopped_fret}"
    )]
    InvalidHarmonicTouch { stopped_fret: u8, touch_fret: u8 },
    #[error("annotation {annotation} is not valid for event {event}")]
    InvalidAnnotationTarget {
        event: u64,
        annotation: &'static str,
    },
    #[error("event {event} has incompatible annotations {first} and {second}")]
    IncompatibleAnnotations {
        event: u64,
        first: &'static str,
        second: &'static str,
    },
    #[error("strum annotation requires a chord, dead multi-string attack, or slash event")]
    InvalidStrumTarget,
    #[error("bend cents remainder {cents} is outside 0..=99")]
    InvalidBendCents { cents: u8 },
    #[error("bend source event {event} is not pitched on string {string}")]
    BendSourceStringMismatch { event: u64, string: u8 },
    #[error(
        "bend {phase:?} event {event} is in voice {actual_voice}, expected voice {source_voice}"
    )]
    BendVoiceMismatch {
        phase: BendPhase,
        event: u64,
        source_voice: u8,
        actual_voice: u8,
    },
    #[error("bend onset must fall within source event {event}")]
    BendOnsetOutsideSource { event: u64 },
    #[error(
        "bend {phase:?} offset {offset_ticks} is outside event {event} duration {duration_ticks}"
    )]
    InvalidBendOffset {
        phase: BendPhase,
        event: u64,
        offset_ticks: usize,
        duration_ticks: usize,
    },
    #[error("bend phases are out of order: {later:?} does not follow {earlier:?}")]
    InvalidBendOrder {
        earlier: BendPhase,
        later: BendPhase,
    },
    #[error("bend target {target_cents} cents must be above source {source_cents} cents")]
    InvalidBendTarget {
        source_cents: u16,
        target_cents: u16,
    },
    #[error(
        "bend release target {release_cents} is outside source {source_cents}..target {target_cents}"
    )]
    InvalidBendReleaseTarget {
        source_cents: u16,
        target_cents: u16,
        release_cents: u16,
    },
    #[error("bend reattack event {event} does not match string {string} fret {fret}")]
    BendReattackMismatch { event: u64, string: u8, fret: u8 },
    #[error(
        "bend event {event} re-frets string {string} from source fret {source_fret} to {event_fret}"
    )]
    BendStringRefret {
        event: u64,
        string: u8,
        source_fret: u8,
        event_fret: u8,
    },
    #[error("event {event} string {string} already has a bend gesture")]
    DuplicateBendGesture { event: u64, string: u8 },
    #[error("bend gesture overlaps another gesture on voice {voice} string {string}")]
    OverlappingBendGesture { voice: u8, string: u8 },
    #[error("span end {end} precedes span start {start}")]
    ReversedSpan { start: u64, end: u64 },
    #[error(
        "span endpoints {start} (voice {start_voice}) and {end} (voice {end_voice}) are in different voices"
    )]
    SpanVoiceMismatch {
        start: u64,
        end: u64,
        start_voice: u8,
        end_voice: u8,
    },
    #[error("barre fret must be greater than zero")]
    InvalidBarreFret,
    #[error("barre string {string} is outside 1..={string_count}")]
    BarreStringOutOfRange { string: u8, string_count: u8 },
    #[error("full/partial barre coverage does not match the score's {string_count} strings")]
    BarreKindMismatch { string_count: u8 },
    #[error("barre at fret {barre_fret} has no valid two-string start at event {event}")]
    BarreStartMismatch { event: u64, barre_fret: u8 },
    #[error("barre endpoint event {event} does not use a covered string")]
    BarreEndpointMismatch { event: u64 },
    #[error(
        "barre at fret {barre_fret} conflicts with event {event} string {string} fret {event_fret}"
    )]
    BarreFretMismatch {
        event: u64,
        string: u8,
        barre_fret: u8,
        event_fret: u8,
    },
    #[error("span endpoint {event} is not played on string {string}")]
    SpanStringMismatch { event: u64, string: u8 },
    #[error("event {event} has no string {string} for this annotation")]
    AnnotationStringMismatch { event: u64, string: u8 },
    #[error("voice {voice} occupies {actual_num}/{actual_den} ticks; expected {expected} ticks")]
    IncompleteMeasure {
        voice: u8,
        actual_num: u128,
        actual_den: u128,
        expected: u128,
    },
    #[error("the current measure is empty")]
    EmptyMeasure,
    #[error("a system break or no-break must directly follow a completed measure")]
    LineBreakNotAtBarline,
    #[error("a time signature change must precede the first event of its measure")]
    MidMeasureTimeSignatureChange,
}

/// One validated timeline that drives both standard notation and TAB.
#[derive(Clone, Debug)]
pub struct GuitarScore {
    pub(crate) tuning: GuitarTuning,
    capo: Option<Capo>,
    tuning_display: TuningDisplay,
    clef: ClefKind,
    key_signature: KeySignature,
    /// Meter at the start of the score.
    initial_time_signature: Option<(u8, u8)>,
    /// Meter in force for the measure in progress (validation).
    time_signature: Option<(u8, u8)>,
    /// Meter change entered at the start of the measure in progress.
    pending_time_signature_change: Option<(u8, u8)>,
    /// A meter change entered mid-measure, reported when the measure ends.
    pending_meter_error: Option<GuitarScoreError>,
    current_voice: u8,
    current_voices: BTreeMap<u8, Vec<GuitarGroup>>,
    pending_clef_changes: Vec<GuitarClefChange>,
    pub(crate) measure_numbering: MeasureNumbering,
    first_measure_number: i32,
    pub(crate) measures: Vec<GuitarMeasure>,
    pub(crate) spans: Vec<GuitarSpan>,
    pub(crate) bends: Vec<BendGesture>,
    show_technique_legend: bool,
    next_event_id: u64,
}

impl GuitarScore {
    pub fn new(tuning: GuitarTuning) -> Self {
        Self {
            tuning,
            capo: None,
            tuning_display: TuningDisplay::Hidden,
            clef: ClefKind::Treble8ba,
            key_signature: KeySignature::Open,
            initial_time_signature: None,
            time_signature: None,
            pending_time_signature_change: None,
            pending_meter_error: None,
            current_voice: 0,
            current_voices: BTreeMap::new(),
            pending_clef_changes: Vec::new(),
            measure_numbering: MeasureNumbering::Hidden,
            first_measure_number: 1,
            measures: Vec::new(),
            spans: Vec::new(),
            bends: Vec::new(),
            show_technique_legend: false,
            next_event_id: 1,
        }
    }

    pub fn standard() -> Self {
        Self::new(GuitarTuning::standard())
    }

    /// Set the score's initial clef before content; afterward insert a clef
    /// change at the current voice's onset without rewriting earlier notes.
    pub fn set_clef(&mut self, clef: Clef) -> &mut Self {
        if self.measures.is_empty()
            && self.pending_clef_changes.is_empty()
            && self.current_voices.values().all(Vec::is_empty)
        {
            self.clef = ClefKind::from_clef(&clef);
            return self;
        }
        self.clef_change(clef)
    }

    /// Insert a clef change at the current voice's onset. At a measure start
    /// its change-size clef precedes the previous barline.
    pub fn clef_change(&mut self, clef: Clef) -> &mut Self {
        self.push_clef_change(clef, false)
    }

    /// Like [`Self::clef_change`], but put a measure-start change after the
    /// barline (as explicitly requested by some scores).
    pub fn clef_change_after_barline(&mut self, clef: Clef) -> &mut Self {
        self.push_clef_change(clef, true)
    }

    fn push_clef_change(&mut self, clef: Clef, after_barline: bool) -> &mut Self {
        let groups = self.current_voices.get(&self.current_voice);
        self.pending_clef_changes.push(GuitarClefChange {
            voice: self.current_voice,
            after_group: groups.map_or(0, Vec::len),
            onset: groups.map_or(Fraction::default(), |groups| {
                groups.iter().fold(Fraction::default(), |sum, group| sum + group.effective_ticks())
            }),
            clef: ClefKind::from_clef(&clef),
            after_barline,
        });
        self
    }

    /// Set which guitar measures print their logical bar number.
    pub fn set_measure_numbering(&mut self, numbering: MeasureNumbering) -> &mut Self {
        self.measure_numbering = numbering;
        self
    }

    /// Set the number of the first non-pickup guitar measure.
    pub fn set_first_measure_number(&mut self, number: i32) -> &mut Self {
        self.first_measure_number = number;
        self
    }

    pub fn set_key_signature(&mut self, key_signature: KeySignature) -> &mut Self {
        self.key_signature = key_signature;
        self
    }

    /// Set the meter at the start of the score. Once events have been
    /// added this is a meter change, identical to
    /// [`Self::time_signature_change`]; a mid-measure call is reported as
    /// [`GuitarScoreError::MidMeasureTimeSignatureChange`] when the measure
    /// ends.
    pub fn set_time_signature(&mut self, numerator: u8, denominator: u8) -> &mut Self {
        if let Err(error) = self.time_signature_change(numerator, denominator) {
            self.pending_meter_error.get_or_insert(error);
        }
        self
    }

    /// Change the meter from the measure in progress on: measures are
    /// validated against it and the standard notation prints it after the
    /// barline (or in the next system's prefix). Must precede the first
    /// event of the measure.
    pub fn time_signature_change(
        &mut self,
        numerator: u8,
        denominator: u8,
    ) -> Result<&mut Self, GuitarScoreError> {
        if !self.current_voices.values().all(Vec::is_empty) {
            return Err(GuitarScoreError::MidMeasureTimeSignatureChange);
        }
        let meter = Some((numerator, denominator));
        if self.measures.is_empty() {
            self.initial_time_signature = meter;
        } else {
            self.pending_time_signature_change = meter;
        }
        self.time_signature = meter;
        Ok(self)
    }

    pub fn set_voice(&mut self, voice: u8) -> &mut Self {
        self.current_voice = voice;
        self
    }

    /// Set or clear the score's capo before any events are added.
    ///
    /// Open-string tuning remains concert pitch; validation and standard
    /// projection add the capo fret while TAB frets stay capo-relative.
    pub fn set_capo(&mut self, capo: Option<Capo>) -> Result<&mut Self, GuitarScoreError> {
        if self.next_event_id != 1 {
            return Err(GuitarScoreError::SetupLocked);
        }
        if let Some(capo) = capo {
            for (index, open) in self.tuning.open_pitches().iter().enumerate() {
                if open
                    .midi_note
                    .checked_add(capo.fret())
                    .is_none_or(|midi| midi > 127)
                {
                    return Err(GuitarScoreError::CapoOutOfMidiRange {
                        fret: capo.fret(),
                        string: index as u8 + 1,
                    });
                }
            }
        }
        self.capo = capo;
        Ok(self)
    }

    /// Select whether exact score-start tuning metadata is visible.
    pub fn set_tuning_display(&mut self, display: TuningDisplay) -> &mut Self {
        self.tuning_display = display;
        self
    }

    /// Return the active capo, if any.
    pub fn capo(&self) -> Option<Capo> {
        self.capo
    }

    pub(crate) fn setup_label(&self) -> Option<String> {
        if self.capo.is_none() && self.tuning_display == TuningDisplay::Hidden {
            return None;
        }
        let mut label = String::new();
        if let Some(capo) = self.capo {
            label.push_str("Capo ");
            label.push_str(&roman_numeral(capo.fret()));
        }
        if self.tuning_display != TuningDisplay::Hidden {
            if !label.is_empty() {
                label.push_str(" · ");
            }
            label.push_str("Tuning: ");
            if self.tuning_display == TuningDisplay::NameAndPitches {
                if let Some(name) = self.tuning.name() {
                    label.push_str(name);
                    label.push_str(" · ");
                }
            }
            for (index, pitch) in self.tuning.open_pitches().iter().enumerate() {
                if index > 0 {
                    label.push(' ');
                }
                use std::fmt::Write as _;
                let _ = write!(label, "{}={pitch}", index + 1);
            }
        }
        Some(label)
    }
    /// Include a legend generated from the attack symbols used in this score.
    pub fn show_technique_legend(&mut self) -> &mut Self {
        self.show_technique_legend = true;
        self
    }

    /// Generate deterministic legend entries for the physical assignments
    /// present in this score.
    pub fn technique_legend(&self) -> Vec<TechniqueLegendEntry> {
        let mut entries = Vec::new();
        let mut has_left_hand_fingering = false;
        for event in all_groups(&self.measures, &self.current_voices).flat_map(GuitarGroup::events)
        {
            for annotation in &event.annotations.performance {
                match annotation {
                    GuitarPerformanceAnnotation::LeftHandFinger { .. } => {
                        has_left_hand_fingering = true;
                    }
                    GuitarPerformanceAnnotation::Attack { source, .. } => {
                        push_legend_entry(&mut entries, source.label(), source.description());
                    }
                    GuitarPerformanceAnnotation::Text(_)
                    | GuitarPerformanceAnnotation::Position(_) => {}
                }
            }
            for (_, harmonic) in &event.annotations.harmonics {
                match harmonic.kind {
                    HarmonicKind::Natural => {
                        push_legend_entry(&mut entries, "harm.", "natural harmonic")
                    }
                    HarmonicKind::Artificial { .. } => {
                        push_legend_entry(&mut entries, "A.H.", "artificial harmonic")
                    }
                    HarmonicKind::Pinch => {
                        push_legend_entry(&mut entries, "P.H.", "pinch harmonic")
                    }
                    HarmonicKind::Tapped { hand, .. } => push_legend_entry(
                        &mut entries,
                        "T.H.",
                        match hand {
                            TappingHand::Fretting => "fretting-hand tapped harmonic",
                            TappingHand::Picking => "picking-hand tapped harmonic",
                        },
                    ),
                }
            }
            if !event.annotations.ghost_strings.is_empty() {
                push_legend_entry(&mut entries, "(note)", "pitched ghost attack");
            }
            if event.annotations.rhythmic_slash {
                push_legend_entry(&mut entries, "/", "rhythmic slash");
            }
            if let Some(direction) = event.annotations.strum {
                match direction {
                    StrumDirection::Down => push_legend_entry(&mut entries, "↧", "down strum"),
                    StrumDirection::Up => push_legend_entry(&mut entries, "↥", "up strum"),
                }
            }
            match &event.kind {
                GuitarEventKind::Dead(_) => {
                    push_legend_entry(&mut entries, "x", "dead/muted attack")
                }
                GuitarEventKind::Slash => push_legend_entry(&mut entries, "/", "rhythmic slash"),
                GuitarEventKind::Percussion(PercussionTarget::Body) => {
                    push_legend_entry(&mut entries, "golpe", "body percussion")
                }
                GuitarEventKind::Percussion(PercussionTarget::Fretboard) => {
                    push_legend_entry(&mut entries, "square", "fretboard percussion")
                }
                GuitarEventKind::Percussion(PercussionTarget::Strings) => {
                    push_legend_entry(&mut entries, "x-perc.", "string percussion")
                }
                GuitarEventKind::Pitched(_) | GuitarEventKind::Rest => {}
            }
        }
        if has_left_hand_fingering {
            entries.insert(
                0,
                TechniqueLegendEntry {
                    symbol: "①–④",
                    description: "left-hand finger",
                },
            );
        }
        entries
    }

    pub fn note(
        &mut self,
        pitch: Pitch,
        duration: Duration,
        string: u8,
        fret: u8,
    ) -> Result<GuitarEventId, GuitarScoreError> {
        self.chord(vec![FrettedPitch::new(pitch, string, fret)], duration)
    }

    pub fn chord(
        &mut self,
        pitches: Vec<FrettedPitch>,
        duration: Duration,
    ) -> Result<GuitarEventId, GuitarScoreError> {
        self.validate_pitches(&pitches)?;
        let id = self.allocate_id();
        self.push_group(GuitarGroup::Event(GuitarEvent {
            id,
            duration,
            kind: GuitarEventKind::Pitched(pitches),
            annotations: GuitarEventAnnotations::default(),
        }));
        Ok(id)
    }

    /// Add a fully damped, rhythmic string attack.
    pub fn dead(
        &mut self,
        strings: Vec<u8>,
        duration: Duration,
    ) -> Result<GuitarEventId, GuitarScoreError> {
        self.validate_strings(&strings)?;
        let id = self.allocate_id();
        self.push_group(GuitarGroup::Event(GuitarEvent {
            id,
            duration,
            kind: GuitarEventKind::Dead(strings),
            annotations: GuitarEventAnnotations::default(),
        }));
        Ok(id)
    }

    /// Add an unvoiced comping slash with exact rhythmic duration.
    pub fn slash(&mut self, duration: Duration) -> GuitarEventId {
        let id = self.allocate_id();
        self.push_group(GuitarGroup::Event(GuitarEvent {
            id,
            duration,
            kind: GuitarEventKind::Slash,
            annotations: GuitarEventAnnotations::default(),
        }));
        id
    }

    /// Add an unpitched rhythmic guitar-percussion event.
    pub fn percussion(&mut self, target: PercussionTarget, duration: Duration) -> GuitarEventId {
        let id = self.allocate_id();
        self.push_group(GuitarGroup::Event(GuitarEvent {
            id,
            duration,
            kind: GuitarEventKind::Percussion(target),
            annotations: GuitarEventAnnotations::default(),
        }));
        id
    }

    pub fn rest(&mut self, duration: Duration) -> GuitarEventId {
        let id = self.allocate_id();
        self.push_group(GuitarGroup::Event(GuitarEvent {
            id,
            duration,
            kind: GuitarEventKind::Rest,
            annotations: GuitarEventAnnotations::default(),
        }));
        id
    }

    /// Add heterogeneous pitched/dead/slash/percussion events to one beam.
    pub fn beam_group(
        &mut self,
        events: Vec<GuitarEventSpec>,
    ) -> Result<Vec<GuitarEventId>, GuitarScoreError> {
        self.add_group(events, None)
    }

    /// Add heterogeneous pitched/dead/slash/percussion events to an exact tuplet.
    pub fn tuplet(
        &mut self,
        number: u32,
        in_time_of: u32,
        events: Vec<GuitarEventSpec>,
    ) -> Result<Vec<GuitarEventId>, GuitarScoreError> {
        if number == 0 || in_time_of == 0 {
            return Err(GuitarScoreError::InvalidTupletRatio);
        }
        self.add_group(events, Some((number, in_time_of)))
    }
    /// Validate and attach one semantic bend gesture.
    ///
    /// All phase moments must already exist in this score. Validation uses
    /// exact per-voice timeline fractions, including tuplet scaling.
    pub fn bend(&mut self, gesture: BendGesture) -> Result<&mut Self, GuitarScoreError> {
        let timeline = self.event_timeline();
        let candidate = self.validate_bend(&gesture, &timeline)?;
        for existing in &self.bends {
            if existing.source == gesture.source && existing.string == gesture.string {
                return Err(GuitarScoreError::DuplicateBendGesture {
                    event: gesture.source.get(),
                    string: gesture.string,
                });
            }
            let existing = self.validate_bend(existing, &timeline)?;
            if existing.voice == candidate.voice
                && existing.string == candidate.string
                && candidate.start < existing.end
                && existing.start < candidate.end
            {
                return Err(GuitarScoreError::OverlappingBendGesture {
                    voice: candidate.voice,
                    string: candidate.string,
                });
            }
        }
        drop(timeline);
        self.bends.push(gesture);
        Ok(self)
    }

    /// Validate and attach one event-local guitar annotation.
    pub fn annotate(
        &mut self,
        event_id: GuitarEventId,
        annotation: GuitarAnnotation,
    ) -> Result<&mut Self, GuitarScoreError> {
        let event = self
            .event(event_id)
            .ok_or(GuitarScoreError::UnknownEvent(event_id.get()))?;

        let ghost_targets = match &annotation {
            GuitarAnnotation::Harmonic { string, harmonic } => {
                self.validate_harmonic(event, *string, *harmonic)?;
                if event
                    .annotations
                    .harmonics
                    .iter()
                    .any(|(candidate, _)| candidate == string)
                {
                    return Err(GuitarScoreError::DuplicateHarmonic {
                        event: event_id.get(),
                        string: *string,
                    });
                }
                if event.annotations.ghost_strings.contains(string) {
                    return Err(GuitarScoreError::IncompatibleAnnotations {
                        event: event_id.get(),
                        first: "harmonic",
                        second: "ghost",
                    });
                }
                None
            }
            GuitarAnnotation::RhythmicSlash => {
                if event.annotations.rhythmic_slash {
                    return Err(GuitarScoreError::DuplicateRhythmicSlash {
                        event: event_id.get(),
                    });
                }
                if !matches!(&event.kind, GuitarEventKind::Pitched(_)) {
                    return Err(GuitarScoreError::InvalidAnnotationTarget {
                        event: event_id.get(),
                        annotation: "rhythmic slash",
                    });
                }
                if !event.annotations.harmonics.is_empty()
                    || !event.annotations.ghost_strings.is_empty()
                    || self
                        .bends
                        .iter()
                        .any(|bend| bend_references_event(bend, event_id))
                {
                    return Err(GuitarScoreError::IncompatibleAnnotations {
                        event: event_id.get(),
                        first: "rhythmic slash",
                        second: "pitched technique",
                    });
                }
                None
            }
            GuitarAnnotation::Ghost { string } => {
                let GuitarEventKind::Pitched(notes) = &event.kind else {
                    return Err(GuitarScoreError::InvalidAnnotationTarget {
                        event: event_id.get(),
                        annotation: "ghost",
                    });
                };
                let targets = if let Some(string) = string {
                    if !notes.iter().any(|note| note.string == *string) {
                        return Err(GuitarScoreError::AnnotationStringMismatch {
                            event: event_id.get(),
                            string: *string,
                        });
                    }
                    vec![*string]
                } else {
                    notes.iter().map(|note| note.string).collect()
                };
                for string in &targets {
                    if event.annotations.ghost_strings.contains(string) {
                        return Err(GuitarScoreError::DuplicateGhost {
                            event: event_id.get(),
                            string: *string,
                        });
                    }
                    if event
                        .annotations
                        .harmonics
                        .iter()
                        .any(|(candidate, _)| candidate == string)
                    {
                        return Err(GuitarScoreError::IncompatibleAnnotations {
                            event: event_id.get(),
                            first: "ghost",
                            second: "harmonic",
                        });
                    }
                }
                if event.annotations.rhythmic_slash {
                    return Err(GuitarScoreError::IncompatibleAnnotations {
                        event: event_id.get(),
                        first: "ghost",
                        second: "rhythmic slash",
                    });
                }
                Some(targets)
            }
            GuitarAnnotation::Strum(_) => {
                let valid = match &event.kind {
                    GuitarEventKind::Pitched(notes) => notes.len() >= 2,
                    GuitarEventKind::Dead(strings) => strings.len() >= 2,
                    GuitarEventKind::Slash => true,
                    GuitarEventKind::Percussion(_) | GuitarEventKind::Rest => false,
                };
                if !valid {
                    return Err(GuitarScoreError::InvalidStrumTarget);
                }
                if event.annotations.strum.is_some() {
                    return Err(GuitarScoreError::IncompatibleAnnotations {
                        event: event_id.get(),
                        first: "strum",
                        second: "strum",
                    });
                }
                None
            }
            GuitarAnnotation::LeftHandFinger { string, .. } => {
                if !matches!(&event.kind, GuitarEventKind::Pitched(notes) if notes.iter().any(|note| note.string == *string))
                {
                    return Err(GuitarScoreError::AnnotationStringMismatch {
                        event: event_id.get(),
                        string: *string,
                    });
                }
                None
            }
            GuitarAnnotation::Attack { string, .. } => {
                if let Some(string) = string {
                    if !event.strings().any(|candidate| candidate == *string) {
                        return Err(GuitarScoreError::AnnotationStringMismatch {
                            event: event_id.get(),
                            string: *string,
                        });
                    }
                } else if matches!(
                    &event.kind,
                    GuitarEventKind::Rest | GuitarEventKind::Percussion(_)
                ) {
                    return Err(GuitarScoreError::InvalidAnnotationTarget {
                        event: event_id.get(),
                        annotation: "attack",
                    });
                }
                None
            }
            GuitarAnnotation::ChordSymbol(_)
                if !matches!(
                    &event.kind,
                    GuitarEventKind::Pitched(_) | GuitarEventKind::Slash
                ) =>
            {
                return Err(GuitarScoreError::InvalidAnnotationTarget {
                    event: event_id.get(),
                    annotation: "chord symbol",
                });
            }
            GuitarAnnotation::Vibrato(_) if event.strings().next().is_none() => {
                return Err(GuitarScoreError::InvalidAnnotationTarget {
                    event: event_id.get(),
                    annotation: "vibrato",
                });
            }
            GuitarAnnotation::Vibrato(_)
            | GuitarAnnotation::ChordSymbol(_)
            | GuitarAnnotation::Text(_)
            | GuitarAnnotation::Position(_) => None,
        };

        let event = self
            .event_mut(event_id)
            .expect("annotation validation established event existence");
        match annotation {
            GuitarAnnotation::Vibrato(kind) => event.annotations.vibrato = Some(kind),
            GuitarAnnotation::Harmonic { string, harmonic } => {
                event.annotations.harmonics.push((string, harmonic));
            }
            GuitarAnnotation::Ghost { .. } => {
                event
                    .annotations
                    .ghost_strings
                    .extend(ghost_targets.expect("ghost validation supplies targets"));
            }
            GuitarAnnotation::Strum(direction) => event.annotations.strum = Some(direction),
            GuitarAnnotation::RhythmicSlash => event.annotations.rhythmic_slash = true,
            GuitarAnnotation::LeftHandFinger { string, finger } => event
                .annotations
                .performance
                .push(GuitarPerformanceAnnotation::LeftHandFinger { string, finger }),
            GuitarAnnotation::Attack { string, source } => event
                .annotations
                .performance
                .push(GuitarPerformanceAnnotation::Attack { string, source }),
            GuitarAnnotation::ChordSymbol(symbol) => event.annotations.chord_symbol = Some(symbol),
            GuitarAnnotation::Text(text) => event
                .annotations
                .performance
                .push(GuitarPerformanceAnnotation::Text(text)),
            GuitarAnnotation::Position(position) => event
                .annotations
                .performance
                .push(GuitarPerformanceAnnotation::Position(position)),
        }
        Ok(self)
    }

    /// Attach a string-specific TAB technique to an explicit event range.
    pub fn span(
        &mut self,
        kind: GuitarSpanKind,
        start: GuitarEventId,
        end: GuitarEventId,
        string: u8,
    ) -> Result<&mut Self, GuitarScoreError> {
        self.validate_span_range(start, end)?;
        for id in [start, end] {
            let event = self
                .event(id)
                .expect("span validation established existence");
            if !event.strings().any(|candidate| candidate == string) {
                return Err(GuitarScoreError::SpanStringMismatch {
                    event: id.get(),
                    string,
                });
            }
        }
        self.spans.push(GuitarSpan {
            annotation: StoredGuitarSpan::Technique { kind, string },
            start,
            end,
        });
        Ok(self)
    }

    /// Attach a score-local annotation to an explicit event range. Rendering
    /// repeats the line on each system and places the label only at its start.
    pub fn annotation_span(
        &mut self,
        annotation: GuitarSpanAnnotation,
        start: GuitarEventId,
        end: GuitarEventId,
    ) -> Result<&mut Self, GuitarScoreError> {
        self.validate_span_range(start, end)?;
        if let GuitarSpanAnnotation::Barre(barre) = &annotation {
            self.validate_barre(*barre, start, end)?;
        }
        self.spans.push(GuitarSpan {
            annotation: StoredGuitarSpan::Annotation(annotation),
            start,
            end,
        });
        Ok(self)
    }

    pub fn end_measure(&mut self, barline: BarlineStyle) -> Result<&mut Self, GuitarScoreError> {
        if let Some(error) = self.pending_meter_error.take() {
            return Err(error);
        }
        if self.current_voices.values().all(Vec::is_empty) {
            return Err(GuitarScoreError::EmptyMeasure);
        }
        self.validate_current_measure()?;
        self.measures.push(GuitarMeasure {
            voices: std::mem::take(&mut self.current_voices),
            barline,
            line_break: LineBreak::Auto,
            time_signature_change: self.pending_time_signature_change.take(),
            clef_changes: std::mem::take(&mut self.pending_clef_changes),
        });
        self.current_voice = 0;
        Ok(self)
    }

    pub fn barline(&mut self) -> Result<&mut Self, GuitarScoreError> {
        self.end_measure(BarlineStyle::Single)
    }

    pub fn end_barline(&mut self) -> Result<&mut Self, GuitarScoreError> {
        self.end_measure(BarlineStyle::Final)
    }

    /// Force a system break after the most recently completed measure
    /// (LilyPond `\break` at a barline). Standard notation and TAB break
    /// together, in every breaking mode of [`MultiStaffScore`].
    ///
    /// # Errors
    ///
    /// [`GuitarScoreError::LineBreakNotAtBarline`] when the current measure
    /// already holds events or no measure has been completed yet.
    pub fn system_break(&mut self) -> Result<&mut Self, GuitarScoreError> {
        self.set_line_break(LineBreak::Force)
    }

    /// Forbid a system break after the most recently completed measure
    /// (LilyPond `\noBreak` at a barline).
    ///
    /// # Errors
    ///
    /// As for [`Self::system_break`].
    pub fn no_break(&mut self) -> Result<&mut Self, GuitarScoreError> {
        self.set_line_break(LineBreak::Forbid)
    }

    fn set_line_break(&mut self, kind: LineBreak) -> Result<&mut Self, GuitarScoreError> {
        if !self.current_voices.values().all(Vec::is_empty) {
            return Err(GuitarScoreError::LineBreakNotAtBarline);
        }
        self.measures
            .last_mut()
            .ok_or(GuitarScoreError::LineBreakNotAtBarline)?
            .line_break = kind;
        Ok(self)
    }

    pub fn try_render_svg(mut self) -> Result<String, EngraverError> {
        self.finalize_pending_measure()?;
        MultiStaffScore::guitar(self).try_render_svg()
    }

    pub(crate) fn finalize_pending_measure(&mut self) -> Result<(), GuitarScoreError> {
        if !self.current_voices.values().all(Vec::is_empty) {
            self.end_barline()?;
        }
        Ok(())
    }

    pub fn render_svg(self) -> String {
        self.try_render_svg()
            .expect("validated guitar score and bundled Bravura font should render")
    }

    pub(crate) fn line_count(&self) -> u8 {
        self.tuning.line_count()
    }

    pub(crate) fn measure_count(&self) -> usize {
        self.measures.len()
    }

    pub(crate) fn annotation_layout(&self, chunks: &[(usize, usize)]) -> GuitarAnnotationLayout {
        let timeline = self.event_timeline();
        let mut event_measures = HashMap::new();
        for (measure_index, measure) in self.measures.iter().enumerate() {
            for groups in measure.voices.values() {
                for group in groups {
                    for event in group.events() {
                        event_measures.insert(event.id, measure_index);
                    }
                }
            }
        }

        let mut candidates = self
            .spans
            .iter()
            .enumerate()
            .filter_map(|(span_index, span)| {
                if !matches!(&span.annotation, StoredGuitarSpan::Annotation(_)) {
                    return None;
                }
                let start = timeline.get(&span.start)?;
                let end = timeline.get(&span.end)?;
                Some((span_index, start.onset, end.onset))
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(span_index, start, end)| (*start, *end, *span_index));

        let mut lane_ends = Vec::<Fraction>::new();
        let mut span_lanes = vec![None; self.spans.len()];
        for (span_index, start, end) in candidates {
            let lane = lane_ends
                .iter()
                .position(|lane_end| *lane_end < start)
                .unwrap_or_else(|| {
                    lane_ends.push(end);
                    lane_ends.len() - 1
                });
            lane_ends[lane] = end;
            span_lanes[span_index] = Some(lane);
        }

        let mut span_lane_counts = vec![0usize; chunks.len()];
        for (span_index, span) in self.spans.iter().enumerate() {
            let Some(lane) = span_lanes[span_index] else {
                continue;
            };
            let Some((&start_measure, &end_measure)) = event_measures
                .get(&span.start)
                .zip(event_measures.get(&span.end))
            else {
                continue;
            };
            for (system_index, (measure_start, measure_end)) in chunks.iter().enumerate() {
                if *measure_start <= end_measure && start_measure < *measure_end {
                    span_lane_counts[system_index] = span_lane_counts[system_index].max(lane + 1);
                }
            }
        }

        let mut system_heights_ss = vec![0.0_f64; chunks.len()];
        for (system_index, (measure_start, measure_end)) in chunks.iter().enumerate() {
            let mut height = if span_lane_counts[system_index] > 0 {
                GUITAR_ANNOTATION_FIRST_LANE_HEIGHT_SS
                    + (span_lane_counts[system_index] - 1) as f64 * GUITAR_ANNOTATION_LANE_PITCH_SS
            } else {
                0.0
            };
            let local_start = (*measure_start).min(self.measures.len());
            let local_end = (*measure_end).min(self.measures.len());
            for measure in &self.measures[local_start..local_end] {
                for groups in measure.voices.values() {
                    for event in groups.iter().flat_map(GuitarGroup::events) {
                        let whole_event_attacks = event
                            .annotations
                            .performance
                            .iter()
                            .filter(|annotation| {
                                matches!(
                                    annotation,
                                    GuitarPerformanceAnnotation::Attack { string: None, .. }
                                )
                            })
                            .count();
                        if whole_event_attacks > 0 {
                            height = height.max(1.8 + (whole_event_attacks - 1) as f64 * 0.8);
                        }
                        for annotation in &event.annotations.performance {
                            match annotation {
                                GuitarPerformanceAnnotation::Text(text)
                                    if text.placement == TextPlacement::Below =>
                                {
                                    height = height.max(2.8);
                                }
                                GuitarPerformanceAnnotation::Position(_) => {
                                    height = height.max(3.7);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            system_heights_ss[system_index] = height;
        }

        let mut legend_y_ss = None;
        if self.show_technique_legend && !chunks.is_empty() {
            let legend_rows = self.technique_legend().len();
            if legend_rows > 0 {
                let y = system_heights_ss[0].max(0.3) + 0.7;
                legend_y_ss = Some(y);
                system_heights_ss[0] = y + legend_rows as f64 * GUITAR_LEGEND_ROW_HEIGHT_SS + 0.6;
            }
        }

        GuitarAnnotationLayout {
            span_lanes,
            system_heights_ss,
            legend_y_ss,
        }
    }

    pub(crate) fn standard_annotation_top_margin_ss(&self) -> f64 {
        let staff = StaffLayout::new(0.0, 0.0, 1.0, 1.0);
        let mut margin = 5.0_f64;
        for event in all_groups(&self.measures, &self.current_voices).flat_map(GuitarGroup::events)
        {
            let attack_count = event
                .annotations
                .performance
                .iter()
                .filter(|annotation| {
                    matches!(annotation, GuitarPerformanceAnnotation::Attack { .. })
                })
                .count();
            let label_count = event.annotations.harmonics.len() + attack_count;
            if label_count == 0 {
                continue;
            }
            let last_label_y =
                standard_event_label_y(self, &staff, event) - (label_count - 1) as f64 * 0.8;
            margin = margin.max(-last_label_y + 1.0);
        }
        margin
    }

    pub(crate) fn notation_builder(&self) -> ScoreBuilder {
        let mut score = ScoreBuilder::new()
            .clef(self.clef.to_clef())
            .first_measure_number(self.first_measure_number)
            .key_signature(self.key_signature.clone());
        if let Some((numerator, denominator)) = self.initial_time_signature {
            score = score.time_signature(numerator, denominator);
        }
        let mut active_clef = self.clef;
        for measure in &self.measures {
            if let Some((numerator, denominator)) = measure.time_signature_change {
                score = score.time_signature_change(numerator, denominator);
            }
            let changes = &measure.clef_changes;
            let max_voice = measure
                .voices
                .keys()
                .copied()
                .chain(changes.iter().map(|change| change.voice))
                .max()
                .unwrap_or(0);
            for voice in 0..=max_voice {
                score = score.voice(voice);
                let groups = measure.voices.get(&voice).map_or(&[][..], Vec::as_slice);
                let mut onset = Fraction::default();
                for (index, group) in groups.iter().enumerate() {
                    for change in changes
                        .iter()
                        .filter(|change| change.voice == voice && change.after_group == index)
                    {
                        score = if change.after_barline {
                            score.clef_change_after_barline(change.clef.to_clef())
                        } else {
                            score.clef_change(change.clef.to_clef())
                        };
                    }
                    // A clef directive applies at its onset across all voices,
                    // even when the voice carrying the directive is projected
                    // after this one into the ScoreBuilder.
                    let clef = changes
                        .iter()
                        .filter(|change| change.onset <= onset)
                        .max_by_key(|change| change.onset)
                        .map_or(active_clef, |change| change.clef);
                    score = match group {
                        GuitarGroup::Event(event) => {
                            add_event_to_score(score, event, clef, &self.bends)
                        }
                        GuitarGroup::Beam(events) => score.styled_beam_group(
                            events
                                .iter()
                                .map(|event| {
                                    (
                                        event_standard_written_pitches(event, clef, &self.bends),
                                        event.duration,
                                        event_note_annotations(event),
                                    )
                                })
                                .collect(),
                        ),
                        GuitarGroup::Tuplet {
                            number,
                            in_time_of,
                            events,
                        } => score.styled_tuplet_ratio(
                            *number,
                            *in_time_of,
                            events
                                .iter()
                                .map(|event| {
                                    (
                                        event_standard_written_pitches(event, clef, &self.bends),
                                        event.duration,
                                        event_note_annotations(event),
                                    )
                                })
                                .collect(),
                        ),
                    };
                    onset = onset + group.effective_ticks();
                }
                for change in changes
                    .iter()
                    .filter(|change| change.voice == voice && change.after_group == groups.len())
                {
                    score = if change.after_barline {
                        score.clef_change_after_barline(change.clef.to_clef())
                    } else {
                        score.clef_change(change.clef.to_clef())
                    };
                }
            }
            if let Some(change) = changes.iter().max_by_key(|change| change.onset) {
                active_clef = change.clef;
            }
            score = score.barline_style(measure.barline);
            score = match measure.line_break {
                LineBreak::Auto => score,
                LineBreak::Force => score.system_break(),
                LineBreak::Forbid => score.no_break(),
            };
        }
        score
    }

    fn validate_span_range(
        &self,
        start: GuitarEventId,
        end: GuitarEventId,
    ) -> Result<(), GuitarScoreError> {
        let timeline = self.event_timeline();
        let start_event = timeline
            .get(&start)
            .ok_or(GuitarScoreError::UnknownEvent(start.get()))?;
        let end_event = timeline
            .get(&end)
            .ok_or(GuitarScoreError::UnknownEvent(end.get()))?;
        if start_event.voice != end_event.voice {
            return Err(GuitarScoreError::SpanVoiceMismatch {
                start: start.get(),
                end: end.get(),
                start_voice: start_event.voice,
                end_voice: end_event.voice,
            });
        }
        if end_event.onset < start_event.onset {
            return Err(GuitarScoreError::ReversedSpan {
                start: start.get(),
                end: end.get(),
            });
        }
        Ok(())
    }

    pub(crate) fn validated_bend_timeline(
        &self,
    ) -> Result<Option<HashMap<GuitarEventId, TimelineEvent<'_>>>, GuitarScoreError> {
        if self.bends.is_empty() {
            return Ok(None);
        }
        let timeline = self.event_timeline();
        self.validate_bends_with_timeline(&timeline)?;
        Ok(Some(timeline))
    }

    fn validate_bends_with_timeline(
        &self,
        timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
    ) -> Result<(), GuitarScoreError> {
        let mut validated: Vec<ValidatedBend> = Vec::with_capacity(self.bends.len());
        for (index, gesture) in self.bends.iter().enumerate() {
            let current = self.validate_bend(gesture, timeline)?;
            for (other_index, other) in validated.iter().enumerate() {
                if self.bends[other_index].source == gesture.source
                    && self.bends[other_index].string == gesture.string
                {
                    return Err(GuitarScoreError::DuplicateBendGesture {
                        event: gesture.source.get(),
                        string: gesture.string,
                    });
                }
                if other.voice == current.voice
                    && other.string == current.string
                    && current.start < other.end
                    && other.start < current.end
                {
                    return Err(GuitarScoreError::OverlappingBendGesture {
                        voice: current.voice,
                        string: current.string,
                    });
                }
            }
            debug_assert_eq!(validated.len(), index);
            validated.push(current);
        }
        Ok(())
    }

    fn validate_bend(
        &self,
        gesture: &BendGesture,
        timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
    ) -> Result<ValidatedBend, GuitarScoreError> {
        let source = timeline
            .get(&gesture.source)
            .ok_or(GuitarScoreError::UnknownEvent(gesture.source.get()))?;
        let source_note = match &source.event.kind {
            GuitarEventKind::Pitched(notes) => notes
                .iter()
                .find(|note| note.string == gesture.string)
                .copied(),
            GuitarEventKind::Dead(_)
            | GuitarEventKind::Slash
            | GuitarEventKind::Percussion(_)
            | GuitarEventKind::Rest => None,
        }
        .ok_or(GuitarScoreError::BendSourceStringMismatch {
            event: gesture.source.get(),
            string: gesture.string,
        })?;
        if source.event.annotations.rhythmic_slash {
            return Err(GuitarScoreError::IncompatibleAnnotations {
                event: gesture.source.get(),
                first: "rhythmic slash",
                second: "bend",
            });
        }
        if source
            .event
            .annotations
            .harmonics
            .iter()
            .any(|(string, _)| *string == gesture.string)
        {
            return Err(GuitarScoreError::IncompatibleAnnotations {
                event: gesture.source.get(),
                first: "harmonic",
                second: "bend",
            });
        }

        let source_cents = source_note.pitch.midi_note as u16 * 100;
        let target_cents = gesture.target.total_cents();
        if target_cents <= source_cents || target_cents > 12_700 {
            return Err(GuitarScoreError::InvalidBendTarget {
                source_cents,
                target_cents,
            });
        }

        let onset = resolve_bend_moment(gesture.onset, BendPhase::Onset, source.voice, timeline)?;
        let arrival =
            resolve_bend_moment(gesture.arrival, BendPhase::Arrival, source.voice, timeline)?;
        match gesture.attack {
            BendAttack::PickThenBend => {
                if onset.time < source.onset || onset.time >= source.end {
                    return Err(GuitarScoreError::BendOnsetOutsideSource {
                        event: gesture.source.get(),
                    });
                }
                if arrival.time <= onset.time {
                    return Err(GuitarScoreError::InvalidBendOrder {
                        earlier: BendPhase::Onset,
                        later: BendPhase::Arrival,
                    });
                }
            }
            BendAttack::PreBent => {
                if onset.time != source.onset || arrival.time != source.onset {
                    return Err(GuitarScoreError::InvalidBendOrder {
                        earlier: BendPhase::Onset,
                        later: BendPhase::Arrival,
                    });
                }
            }
        }

        let release = if let Some(release) = gesture.release {
            let start = resolve_bend_moment(
                release.start,
                BendPhase::ReleaseStart,
                source.voice,
                timeline,
            )?;
            let end =
                resolve_bend_moment(release.end, BendPhase::ReleaseEnd, source.voice, timeline)?;
            if start.time < arrival.time {
                return Err(GuitarScoreError::InvalidBendOrder {
                    earlier: BendPhase::Arrival,
                    later: BendPhase::ReleaseStart,
                });
            }
            if end.time <= start.time {
                return Err(GuitarScoreError::InvalidBendOrder {
                    earlier: BendPhase::ReleaseStart,
                    later: BendPhase::ReleaseEnd,
                });
            }
            let release_cents = release.target.total_cents();
            if release_cents < source_cents || release_cents >= target_cents {
                return Err(GuitarScoreError::InvalidBendReleaseTarget {
                    source_cents,
                    target_cents,
                    release_cents,
                });
            }
            Some((start, end))
        } else {
            None
        };

        let mut previous_reattack = None;
        let mut gesture_end = timeline[&gesture.arrival.event()].end.max(arrival.time);
        for &reattack in &gesture.reattacks {
            let location = timeline
                .get(&reattack)
                .ok_or(GuitarScoreError::UnknownEvent(reattack.get()))?;
            if location.voice != source.voice {
                return Err(GuitarScoreError::BendVoiceMismatch {
                    phase: BendPhase::Reattack,
                    event: reattack.get(),
                    source_voice: source.voice,
                    actual_voice: location.voice,
                });
            }
            let matches_source = matches!(
                &location.event.kind,
                GuitarEventKind::Pitched(notes)
                    if notes.iter().any(|note| note.string == gesture.string
                        && note.fret == source_note.fret)
            );
            if !matches_source {
                return Err(GuitarScoreError::BendReattackMismatch {
                    event: reattack.get(),
                    string: gesture.string,
                    fret: source_note.fret,
                });
            }
            if location.onset < arrival.time
                || release
                    .as_ref()
                    .is_some_and(|(start, _)| location.onset >= start.time)
                || previous_reattack.is_some_and(|previous| location.onset <= previous)
            {
                return Err(GuitarScoreError::InvalidBendOrder {
                    earlier: BendPhase::Arrival,
                    later: BendPhase::Reattack,
                });
            }
            previous_reattack = Some(location.onset);
            gesture_end = gesture_end.max(location.end);
        }
        if let Some((_, end)) = release {
            gesture_end = end.time;
        }

        let phase_uses_event = |event: GuitarEventId| {
            event == gesture.onset.event()
                || event == gesture.arrival.event()
                || gesture.release.is_some_and(|release| {
                    event == release.start.event() || event == release.end.event()
                })
        };
        let refret = timeline
            .values()
            .filter_map(|location| {
                let participates = phase_uses_event(location.event.id)
                    || (location.onset >= onset.time && location.onset < gesture_end);
                if !participates {
                    return None;
                }
                let GuitarEventKind::Pitched(notes) = &location.event.kind else {
                    return None;
                };
                notes
                    .iter()
                    .find(|note| note.string == gesture.string && note.fret != source_note.fret)
                    .map(|note| (location, note))
            })
            .min_by_key(|(location, _)| (location.onset, location.event.id));
        if let Some((location, note)) = refret {
            return Err(GuitarScoreError::BendStringRefret {
                event: location.event.id.get(),
                string: gesture.string,
                source_fret: source_note.fret,
                event_fret: note.fret,
            });
        }

        Ok(ValidatedBend {
            voice: source.voice,
            string: gesture.string,
            start: onset.time,
            end: gesture_end,
        })
    }

    fn event_timeline(&self) -> HashMap<GuitarEventId, TimelineEvent<'_>> {
        let mut timeline = HashMap::new();
        let mut measure_start = Fraction::default();
        for measure in &self.measures {
            index_timeline_measure(&measure.voices, measure_start, &mut timeline);
            measure_start = measure_start + measure_effective_ticks(&measure.voices);
        }
        if !self.current_voices.is_empty() {
            index_timeline_measure(&self.current_voices, measure_start, &mut timeline);
        }
        timeline
    }

    fn add_group(
        &mut self,
        specs: Vec<GuitarEventSpec>,
        tuplet: Option<(u32, u32)>,
    ) -> Result<Vec<GuitarEventId>, GuitarScoreError> {
        if specs.is_empty() {
            return Err(GuitarScoreError::EmptyGroup);
        }
        let mut events = Vec::with_capacity(specs.len());
        let mut ids = Vec::with_capacity(specs.len());
        for spec in specs {
            let duration = spec.duration();
            let kind = match spec {
                GuitarEventSpec::Pitched { pitches, .. } => {
                    self.validate_pitches(&pitches)?;
                    GuitarEventKind::Pitched(pitches)
                }
                GuitarEventSpec::Dead { strings, .. } => {
                    self.validate_strings(&strings)?;
                    GuitarEventKind::Dead(strings)
                }
                GuitarEventSpec::Slash { .. } => GuitarEventKind::Slash,
                GuitarEventSpec::Percussion { target, .. } => GuitarEventKind::Percussion(target),
            };
            let id = self.allocate_id();
            ids.push(id);
            events.push(GuitarEvent {
                id,
                duration,
                kind,
                annotations: GuitarEventAnnotations::default(),
            });
        }
        let group = match tuplet {
            Some((number, in_time_of)) => GuitarGroup::Tuplet {
                number,
                in_time_of,
                events,
            },
            None => GuitarGroup::Beam(events),
        };
        self.push_group(group);
        Ok(ids)
    }

    fn effective_pitch_at(&self, string: u8, fret: u8) -> Result<u8, GuitarScoreError> {
        let open = self.tuning.pitch_at(string, 0)?;
        let capo = self.capo.map_or(0, Capo::fret);
        open.checked_add(capo)
            .and_then(|pitch| pitch.checked_add(fret))
            .filter(|pitch| *pitch <= 127)
            .ok_or(GuitarScoreError::FretOutOfMidiRange { string, fret })
    }

    fn harmonic_node_interval(offset: u8) -> Option<u8> {
        match offset {
            12 => Some(12),
            7 | 19 => Some(19),
            5 | 24 => Some(24),
            4 | 9 | 16 | 28 => Some(28),
            _ => None,
        }
    }

    fn validate_harmonic(
        &self,
        event: &GuitarEvent,
        string: u8,
        harmonic: Harmonic,
    ) -> Result<(), GuitarScoreError> {
        let GuitarEventKind::Pitched(notes) = &event.kind else {
            return Err(GuitarScoreError::HarmonicStringMismatch {
                event: event.id.get(),
                string,
            });
        };
        let Some(note) = notes.iter().find(|note| note.string == string) else {
            return Err(GuitarScoreError::HarmonicStringMismatch {
                event: event.id.get(),
                string,
            });
        };
        if event.annotations.rhythmic_slash {
            return Err(GuitarScoreError::IncompatibleAnnotations {
                event: event.id.get(),
                first: "harmonic",
                second: "rhythmic slash",
            });
        }
        if self
            .bends
            .iter()
            .any(|bend| bend.source == event.id && bend.string == string)
        {
            return Err(GuitarScoreError::IncompatibleAnnotations {
                event: event.id.get(),
                first: "harmonic",
                second: "bend",
            });
        }

        let expected_midi = match harmonic.kind {
            HarmonicKind::Natural => {
                let interval = Self::harmonic_node_interval(note.fret).ok_or(
                    GuitarScoreError::InvalidHarmonicTouch {
                        stopped_fret: 0,
                        touch_fret: note.fret,
                    },
                )?;
                self.effective_pitch_at(string, 0)?
                    .checked_add(interval)
                    .filter(|pitch| *pitch <= 127)
            }
            HarmonicKind::Artificial { touch_fret } | HarmonicKind::Tapped { touch_fret, .. } => {
                let offset = touch_fret.checked_sub(note.fret).ok_or(
                    GuitarScoreError::InvalidHarmonicTouch {
                        stopped_fret: note.fret,
                        touch_fret,
                    },
                )?;
                let interval = Self::harmonic_node_interval(offset).ok_or(
                    GuitarScoreError::InvalidHarmonicTouch {
                        stopped_fret: note.fret,
                        touch_fret,
                    },
                )?;
                note.pitch
                    .midi_note
                    .checked_add(interval)
                    .filter(|pitch| *pitch <= 127)
            }
            HarmonicKind::Pinch => (harmonic.sounding_pitch.midi_note > note.pitch.midi_note)
                .then_some(harmonic.sounding_pitch.midi_note),
        };
        if expected_midi != Some(harmonic.sounding_pitch.midi_note) {
            return Err(GuitarScoreError::InvalidHarmonicPitch {
                event: event.id.get(),
                string,
                sounding: harmonic.sounding_pitch,
            });
        }
        Ok(())
    }

    fn validate_barre(
        &self,
        barre: GuitarBarre,
        start: GuitarEventId,
        end: GuitarEventId,
    ) -> Result<(), GuitarScoreError> {
        let string_count = self.tuning.line_count();
        for string in [barre.strings.first, barre.strings.last] {
            if string == 0 || string > string_count {
                return Err(GuitarScoreError::BarreStringOutOfRange {
                    string,
                    string_count,
                });
            }
        }
        let covers_all = barre.strings.first == 1 && barre.strings.last == string_count;
        if matches!(barre.kind, BarreKind::Full) != covers_all {
            return Err(GuitarScoreError::BarreKindMismatch { string_count });
        }

        let timeline = self.event_timeline();
        let start_event = &timeline[&start];
        let end_event = &timeline[&end];
        let start_matches = match &start_event.event.kind {
            GuitarEventKind::Pitched(notes) => notes
                .iter()
                .filter(|note| barre.strings.contains(note.string) && note.fret == barre.fret)
                .count(),
            _ => 0,
        };
        if start_matches < 2 {
            return Err(GuitarScoreError::BarreStartMismatch {
                event: start.get(),
                barre_fret: barre.fret,
            });
        }
        for endpoint in [start_event.event, end_event.event] {
            let uses_covered_string = matches!(
                &endpoint.kind,
                GuitarEventKind::Pitched(notes)
                    if notes.iter().any(|note| barre.strings.contains(note.string))
            );
            if !uses_covered_string {
                return Err(GuitarScoreError::BarreEndpointMismatch {
                    event: endpoint.id.get(),
                });
            }
        }
        for location in timeline
            .values()
            .filter(|location| location.onset < end_event.end && location.end > start_event.onset)
        {
            if let GuitarEventKind::Pitched(notes) = &location.event.kind {
                if let Some(note) = notes
                    .iter()
                    .find(|note| barre.strings.contains(note.string) && note.fret < barre.fret)
                {
                    return Err(GuitarScoreError::BarreFretMismatch {
                        event: location.event.id.get(),
                        string: note.string,
                        barre_fret: barre.fret,
                        event_fret: note.fret,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_pitches(&self, pitches: &[FrettedPitch]) -> Result<(), GuitarScoreError> {
        if pitches.is_empty() {
            return Err(GuitarScoreError::EmptyGroup);
        }
        let mut seen = [false; 256];
        for pitch in pitches {
            let expected_midi = self.effective_pitch_at(pitch.string, pitch.fret)?;
            if pitch.pitch.midi_note != expected_midi {
                return Err(GuitarScoreError::PitchMismatch {
                    pitch: pitch.pitch,
                    string: pitch.string,
                    fret: pitch.fret,
                    expected_midi,
                });
            }
            if std::mem::replace(&mut seen[pitch.string as usize], true) {
                return Err(GuitarScoreError::DuplicateString {
                    string: pitch.string,
                });
            }
        }
        Ok(())
    }

    fn validate_strings(&self, strings: &[u8]) -> Result<(), GuitarScoreError> {
        if strings.is_empty() {
            return Err(GuitarScoreError::EmptyGroup);
        }
        let mut seen = [false; 256];
        for &string in strings {
            self.tuning.pitch_at(string, 0)?;
            if std::mem::replace(&mut seen[string as usize], true) {
                return Err(GuitarScoreError::DuplicateString { string });
            }
        }
        Ok(())
    }

    fn validate_current_measure(&self) -> Result<(), GuitarScoreError> {
        let Some((numerator, denominator)) = self.time_signature else {
            return Ok(());
        };
        let expected = 128u128 * numerator as u128 / denominator as u128;
        for (&voice, groups) in &self.current_voices {
            let actual = groups.iter().fold(Fraction::default(), |sum, group| {
                sum + group.effective_ticks()
            });
            if actual != Fraction::new(expected, 1) {
                return Err(GuitarScoreError::IncompleteMeasure {
                    voice,
                    actual_num: actual.num,
                    actual_den: actual.den,
                    expected,
                });
            }
        }
        Ok(())
    }

    fn allocate_id(&mut self) -> GuitarEventId {
        let id = GuitarEventId(self.next_event_id);
        self.next_event_id += 1;
        id
    }

    fn push_group(&mut self, group: GuitarGroup) {
        self.current_voices
            .entry(self.current_voice)
            .or_default()
            .push(group);
    }

    fn event(&self, id: GuitarEventId) -> Option<&GuitarEvent> {
        all_groups(&self.measures, &self.current_voices)
            .flat_map(GuitarGroup::events)
            .find(|event| event.id == id)
    }

    fn event_mut(&mut self, id: GuitarEventId) -> Option<&mut GuitarEvent> {
        for measure in &mut self.measures {
            for groups in measure.voices.values_mut() {
                if let Some(event) = find_event_mut(groups, id) {
                    return Some(event);
                }
            }
        }
        for groups in self.current_voices.values_mut() {
            if let Some(event) = find_event_mut(groups, id) {
                return Some(event);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TimelineEvent<'a> {
    event: &'a GuitarEvent,
    voice: u8,
    onset: Fraction,
    end: Fraction,
    scale: Fraction,
}

#[derive(Clone, Copy)]
struct ResolvedBendMoment {
    time: Fraction,
}

struct ValidatedBend {
    voice: u8,
    string: u8,
    start: Fraction,
    end: Fraction,
}

fn bend_references_event(gesture: &BendGesture, event: GuitarEventId) -> bool {
    gesture.source == event
        || gesture.onset.event() == event
        || gesture.arrival.event() == event
        || gesture.reattacks.contains(&event)
        || gesture
            .release
            .is_some_and(|release| release.start.event() == event || release.end.event() == event)
}

fn resolve_bend_moment(
    moment: GuitarMoment,
    phase: BendPhase,
    source_voice: u8,
    timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
) -> Result<ResolvedBendMoment, GuitarScoreError> {
    let event_id = moment.event();
    let location = timeline
        .get(&event_id)
        .ok_or(GuitarScoreError::UnknownEvent(event_id.get()))?;
    if location.voice != source_voice {
        return Err(GuitarScoreError::BendVoiceMismatch {
            phase,
            event: event_id.get(),
            source_voice,
            actual_voice: location.voice,
        });
    }
    let time = match moment {
        GuitarMoment::Onset(_) => location.onset,
        GuitarMoment::After { offset, .. } => {
            let offset_ticks = offset.ticks();
            let duration_ticks = location.event.duration.ticks();
            if offset_ticks == 0 || offset_ticks >= duration_ticks {
                return Err(GuitarScoreError::InvalidBendOffset {
                    phase,
                    event: event_id.get(),
                    offset_ticks,
                    duration_ticks,
                });
            }
            location.onset
                + Fraction::new(
                    offset_ticks as u128 * location.scale.num,
                    location.scale.den,
                )
        }
    };
    Ok(ResolvedBendMoment { time })
}

fn index_timeline_measure<'a>(
    voices: &'a BTreeMap<u8, Vec<GuitarGroup>>,
    measure_start: Fraction,
    timeline: &mut HashMap<GuitarEventId, TimelineEvent<'a>>,
) {
    for (&voice, groups) in voices {
        let mut group_onset = measure_start;
        for group in groups {
            let scale = match group {
                GuitarGroup::Tuplet {
                    number, in_time_of, ..
                } => Fraction::new(*in_time_of as u128, *number as u128),
                GuitarGroup::Event(_) | GuitarGroup::Beam(_) => Fraction::new(1, 1),
            };
            let mut event_onset = group_onset;
            for event in group.events() {
                let effective_duration =
                    Fraction::new(event.duration.ticks() as u128 * scale.num, scale.den);
                let end = event_onset + effective_duration;
                timeline.insert(
                    event.id,
                    TimelineEvent {
                        event,
                        voice,
                        onset: event_onset,
                        end,
                        scale,
                    },
                );
                event_onset = end;
            }
            group_onset = group_onset + group.effective_ticks();
        }
    }
}

fn measure_effective_ticks(voices: &BTreeMap<u8, Vec<GuitarGroup>>) -> Fraction {
    voices
        .values()
        .map(|groups| {
            groups.iter().fold(Fraction::default(), |sum, group| {
                sum + group.effective_ticks()
            })
        })
        .max()
        .unwrap_or_default()
}

fn pitched_notehead_style(event: &GuitarEvent, string: u8) -> NoteheadStyle {
    if event.annotations.rhythmic_slash {
        return NoteheadStyle::Slash;
    }
    if event
        .annotations
        .harmonics
        .iter()
        .any(|(candidate, _)| *candidate == string)
    {
        return NoteheadStyle::Diamond;
    }
    event
        .annotations
        .performance
        .iter()
        .find_map(|annotation| match annotation {
            GuitarPerformanceAnnotation::Attack {
                string: target,
                source: AttackSource::Slap(technique),
            } if target.is_none_or(|target| target == string) => Some(match technique {
                SlapTechnique::Thumb => NoteheadStyle::CircleX,
                SlapTechnique::Pop => NoteheadStyle::Diamond,
            }),
            _ => None,
        })
        .unwrap_or_default()
}

fn event_note_annotations(event: &GuitarEvent) -> NoteAnnotations {
    let mut annotations = NoteAnnotations {
        chord_symbol: event.annotations.chord_symbol.clone(),
        unpitched: event.annotations.rhythmic_slash
            || matches!(
                &event.kind,
                GuitarEventKind::Dead(_) | GuitarEventKind::Slash | GuitarEventKind::Percussion(_)
            ),
        ..NoteAnnotations::default()
    };
    match &event.kind {
        GuitarEventKind::Pitched(_) if event.annotations.rhythmic_slash => {
            annotations.notehead_styles.push(NoteheadStyle::Slash);
        }
        GuitarEventKind::Pitched(notes) => {
            let has_custom_head = !event.annotations.harmonics.is_empty()
                || !event.annotations.ghost_strings.is_empty()
                || event.annotations.performance.iter().any(|annotation| {
                    matches!(
                        annotation,
                        GuitarPerformanceAnnotation::Attack {
                            source: AttackSource::Slap(_),
                            ..
                        }
                    )
                });
            if has_custom_head {
                annotations.notehead_styles.reserve(notes.len());
                annotations.parenthesized_noteheads.reserve(notes.len());
                for note in notes {
                    annotations
                        .notehead_styles
                        .push(pitched_notehead_style(event, note.string));
                    annotations
                        .parenthesized_noteheads
                        .push(event.annotations.ghost_strings.contains(&note.string));
                }
            }
        }
        GuitarEventKind::Dead(strings) => {
            annotations
                .notehead_styles
                .resize(strings.len().max(1), NoteheadStyle::X);
        }
        GuitarEventKind::Slash => annotations.notehead_styles.push(NoteheadStyle::Slash),
        GuitarEventKind::Percussion(target) => {
            annotations.notehead_styles.push(match target {
                PercussionTarget::Body => NoteheadStyle::CircleX,
                PercussionTarget::Fretboard => NoteheadStyle::Square,
                PercussionTarget::Strings => NoteheadStyle::X,
            });
        }
        GuitarEventKind::Rest => {}
    }
    annotations
}

fn unpitched_written_pitch(string: u8, clef: ClefKind) -> Pitch {
    // Keep non-pitched notation in fixed written staff positions. String 3
    // occupies the middle line; adjacent and extended strings receive distinct
    // diatonic steps regardless of the selected clef or octave transposition.
    let staff_position = 7_i16 - i16::from(string);
    let bottom_line_absolute = match clef {
        ClefKind::Treble | ClefKind::Treble8va | ClefKind::Treble8ba => 30_i16, // E4
        ClefKind::Bass => 18_i16,                                               // G2
        ClefKind::Alto => 24_i16,                                               // F3
        ClefKind::Tenor => 22_i16,                                              // D3
    };
    let absolute = bottom_line_absolute + staff_position;
    let octave = absolute.div_euclid(7) as i8;
    let note = match absolute.rem_euclid(7) {
        0 => Note::C,
        1 => Note::D,
        2 => Note::E,
        3 => Note::F,
        4 => Note::G,
        5 => Note::A,
        6 => Note::B,
        _ => unreachable!("Euclidean remainder is always in 0..7"),
    };
    Pitch::new(note, octave)
}

fn event_standard_written_pitches(
    event: &GuitarEvent,
    clef: ClefKind,
    bends: &[BendGesture],
) -> Vec<Pitch> {
    match &event.kind {
        GuitarEventKind::Pitched(_) if event.annotations.rhythmic_slash => {
            vec![unpitched_written_pitch(3, clef)]
        }
        GuitarEventKind::Pitched(notes) => notes
            .iter()
            .map(|note| written_pitch(projected_sounding_pitch(event, *note, bends), clef))
            .collect(),
        GuitarEventKind::Dead(strings) => strings
            .iter()
            .map(|string| unpitched_written_pitch(*string, clef))
            .collect(),
        GuitarEventKind::Slash | GuitarEventKind::Percussion(_) => {
            vec![unpitched_written_pitch(3, clef)]
        }
        GuitarEventKind::Rest => Vec::new(),
    }
}

fn add_event_to_score(
    score: ScoreBuilder,
    event: &GuitarEvent,
    clef: ClefKind,
    bends: &[BendGesture],
) -> ScoreBuilder {
    if matches!(&event.kind, GuitarEventKind::Rest) {
        return score.rest(event.duration);
    }
    let pitches = event_standard_written_pitches(event, clef, bends);
    let annotations = event_note_annotations(event);
    if pitches.len() == 1 {
        score.note_annotated(pitches[0], event.duration, annotations)
    } else {
        score.chord_annotated(pitches, event.duration, annotations)
    }
}

fn projected_sounding_pitch(
    event: &GuitarEvent,
    physical: FrettedPitch,
    bends: &[BendGesture],
) -> Pitch {
    if let Some((_, harmonic)) = event
        .annotations
        .harmonics
        .iter()
        .find(|(string, _)| *string == physical.string)
    {
        return harmonic.sounding_pitch;
    }
    bends
        .iter()
        .filter(|gesture| gesture.string == physical.string)
        .find_map(|gesture| {
            let sounds_at_target = (gesture.attack == BendAttack::PreBent
                && gesture.source == event.id)
                || matches!(gesture.arrival, GuitarMoment::Onset(candidate) if candidate == event.id)
                || gesture.release.is_some_and(
                    |release| matches!(release.start, GuitarMoment::Onset(candidate) if candidate == event.id),
                )
                || gesture.reattacks.contains(&event.id);
            if sounds_at_target {
                Some(gesture.target.pitch)
            } else if gesture.release.is_some_and(
                |release| matches!(release.end, GuitarMoment::Onset(candidate) if candidate == event.id),
            ) {
                gesture.release.map(|release| release.target.pitch)
            } else {
                None
            }
        })
        .unwrap_or(physical.pitch)
}

fn written_pitch(pitch: Pitch, clef: ClefKind) -> Pitch {
    let octave_shift = match clef {
        ClefKind::Treble8ba => 1,
        ClefKind::Treble8va => -1,
        ClefKind::Treble | ClefKind::Bass | ClefKind::Alto | ClefKind::Tenor => 0,
    };
    pitch
        .raise_octaves(octave_shift)
        .expect("guitar pitch must fit the selected octave-transposing clef")
}

fn find_event_mut(groups: &mut [GuitarGroup], id: GuitarEventId) -> Option<&mut GuitarEvent> {
    groups
        .iter_mut()
        .flat_map(|group| match group {
            GuitarGroup::Event(event) => std::slice::from_mut(event),
            GuitarGroup::Beam(events) | GuitarGroup::Tuplet { events, .. } => events.as_mut_slice(),
        })
        .find(|event| event.id == id)
}

fn all_groups<'a>(
    measures: &'a [GuitarMeasure],
    current: &'a BTreeMap<u8, Vec<GuitarGroup>>,
) -> impl Iterator<Item = &'a GuitarGroup> {
    measures
        .iter()
        .flat_map(|measure| measure.voices.values())
        .chain(current.values())
        .flatten()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fraction {
    num: u128,
    den: u128,
}

impl Fraction {
    fn new(num: u128, den: u128) -> Self {
        let divisor = gcd(num, den);
        Self {
            num: num / divisor,
            den: den / divisor,
        }
    }

    fn as_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Default for Fraction {
    fn default() -> Self {
        Self::new(0, 1)
    }
}
impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.num * other.den).cmp(&(other.num * self.den))
    }
}

impl std::ops::Add for Fraction {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den + rhs.num * self.den, self.den * rhs.den)
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

#[derive(Clone, Debug)]
pub(crate) struct GuitarAnnotationLayout {
    span_lanes: Vec<Option<usize>>,
    system_heights_ss: Vec<f64>,
    legend_y_ss: Option<f64>,
}

impl GuitarAnnotationLayout {
    pub(crate) fn system_height_ss(&self, system_index: usize) -> f64 {
        self.system_heights_ss
            .get(system_index)
            .copied()
            .unwrap_or(0.0)
    }

    fn span_lane(&self, span_index: usize) -> usize {
        self.span_lanes
            .get(span_index)
            .and_then(|lane| *lane)
            .unwrap_or(0)
    }

    fn legend_y_ss(&self) -> Option<f64> {
        self.legend_y_ss
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GuitarRenderAnchor {
    pub(crate) x: f64,
    pub(crate) end_x: f64,
    pub(crate) duration_ticks: usize,
    pub(crate) system_index: usize,
}

#[allow(
    clippy::too_many_arguments,
    reason = "the coordinated renderer borrows independent standard, TAB, score, and anchor state"
)]
pub(crate) fn draw_guitar_tab_system(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    score: &GuitarScore,
    annotation_layout: &GuitarAnnotationLayout,
    system_index: usize,
    measure_start: usize,
    measure_end: usize,
    standard_layout: &SystemLayout,
    standard_staff: &StaffLayout,
    tab_staff: &TabStaffLayout,
    anchors: &mut HashMap<GuitarEventId, GuitarRenderAnchor>,
) -> Result<(), crate::font::FontError> {
    let stem_width = config.stem_thickness_fu();
    for (local_measure, system_measure) in standard_layout.measures.iter().enumerate() {
        let measure_index = measure_start + local_measure;
        if measure_index >= measure_end {
            break;
        }
        let guitar_measure = &score.measures[measure_index];
        for (&voice, groups) in &guitar_measure.voices {
            let layout = if voice == 0 {
                &system_measure.layout
            } else {
                &system_measure.additional_voice_layouts[voice as usize - 1]
            };
            let positioned = layout.elements.iter().filter(|element| {
                matches!(
                    element.element,
                    MeasureElement::Note(_)
                        | MeasureElement::Chord(_)
                        | MeasureElement::Rest(_)
                        | MeasureElement::BeamGroup(_)
                        | MeasureElement::TupletGroup(_)
                )
            });
            debug_assert_eq!(positioned.clone().count(), groups.len());
            for (group, positioned) in groups.iter().zip(positioned) {
                let group_x = tab_staff.x + system_measure.x_offset + positioned.x;
                let xs = match group {
                    GuitarGroup::Event(event) => {
                        vec![group_x + notation_center_offset(font, event)?]
                    }
                    GuitarGroup::Beam(events) | GuitarGroup::Tuplet { events, .. } => {
                        let durations: Vec<i8> = events
                            .iter()
                            .map(|event| super::event::duration_kind_to_log2(event.duration.kind()))
                            .collect();
                        beam_group_note_x_offsets(&durations, positioned.width)
                            .into_iter()
                            .zip(events)
                            .map(|(offset, event)| {
                                notation_center_offset(font, event)
                                    .map(|center_offset| group_x + offset + center_offset)
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    }
                };

                for (index, (event, &x)) in group.events().iter().zip(&xs).enumerate() {
                    let end_x = xs
                        .get(index + 1)
                        .copied()
                        .unwrap_or(group_x + positioned.width.max(tab_staff.staff_space));
                    anchors.insert(
                        event.id,
                        GuitarRenderAnchor {
                            x,
                            end_x,
                            duration_ticks: event.duration.ticks(),
                            system_index,
                        },
                    );
                    draw_guitar_event(
                        svg,
                        font,
                        config,
                        score,
                        standard_staff,
                        tab_staff,
                        event,
                        x,
                    )?;
                }

                match group {
                    GuitarGroup::Event(event) => {
                        let duration = super::event::duration_kind_to_log2(event.duration.kind());
                        if let Some(layout) =
                            layout_tab_rhythm(tab_staff, xs[0], duration, stem_width)
                        {
                            draw_tab_rhythm(svg, &layout, font)?;
                        }
                    }
                    GuitarGroup::Beam(events) | GuitarGroup::Tuplet { events, .. } => {
                        let beam_notes: Vec<_> = events
                            .iter()
                            .zip(&xs)
                            .map(|(event, &x)| TabBeamedNote {
                                x,
                                duration_log2: super::event::duration_kind_to_log2(
                                    event.duration.kind(),
                                ),
                            })
                            .collect();
                        if let Some(layout) = layout_tab_beam_group(
                            tab_staff,
                            &beam_notes,
                            stem_width,
                            config.beam_thickness_fu(),
                            config.beam_spacing_fu(),
                        ) {
                            draw_tab_beam_group(svg, &layout);
                        }
                        if let GuitarGroup::Tuplet { number, .. } = group {
                            draw_tab_tuplet(svg, tab_staff, &xs, *number, stem_width);
                        }
                    }
                }
            }
        }

        // The measure's closing barline is its last `Barline` element; any
        // earlier ones are inline barlines.
        if let Some(barline_x) = system_measure
            .layout
            .elements
            .iter()
            .rev()
            .find_map(|element| {
                matches!(element.element, MeasureElement::Barline(_))
                    .then_some(tab_staff.x + system_measure.x_offset + element.x)
            })
        {
            super::tab::draw_measure_barline(
                svg,
                font,
                config,
                tab_staff,
                barline_x,
                &guitar_measure.barline,
            )?;
        }
    }
    if system_index == 0 {
        if let Some(legend_y_ss) = annotation_layout.legend_y_ss() {
            draw_technique_legend(svg, font, score, tab_staff, legend_y_ss)?;
        }
    }

    Ok(())
}

fn notation_center_offset(
    font: &MusicFont,
    event: &GuitarEvent,
) -> Result<f64, crate::font::FontError> {
    let duration_log2 = super::event::duration_kind_to_log2(event.duration.kind());
    let widest = match &event.kind {
        GuitarEventKind::Pitched(_) if event.annotations.rhythmic_slash => {
            notehead_advance(font, duration_log2, NoteheadStyle::Slash)?
        }
        GuitarEventKind::Pitched(notes) => notes.iter().try_fold(0.0_f64, |widest, note| {
            notehead_advance(
                font,
                duration_log2,
                pitched_notehead_style(event, note.string),
            )
            .map(|advance| widest.max(advance))
        })?,
        GuitarEventKind::Dead(_) => notehead_advance(font, duration_log2, NoteheadStyle::X)?,
        GuitarEventKind::Slash => notehead_advance(font, duration_log2, NoteheadStyle::Slash)?,
        GuitarEventKind::Percussion(PercussionTarget::Body) => {
            notehead_advance(font, duration_log2, NoteheadStyle::CircleX)?
        }
        GuitarEventKind::Percussion(PercussionTarget::Fretboard) => {
            notehead_advance(font, duration_log2, NoteheadStyle::Square)?
        }
        GuitarEventKind::Percussion(PercussionTarget::Strings) => {
            notehead_advance(font, duration_log2, NoteheadStyle::X)?
        }
        GuitarEventKind::Rest => notehead_advance(font, duration_log2, NoteheadStyle::Normal)?,
    };
    Ok(widest / 2.0)
}

fn draw_guitar_glyph(
    svg: &mut SvgWriter,
    font: &MusicFont,
    glyph: Glyph,
    center_x: f64,
    y: f64,
) -> Result<(), crate::font::FontError> {
    let outline = font.glyph_outline(glyph)?;
    let x = center_x - outline.advance_width as f64 / 2.0;
    let transform = format!("translate({x}, {y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

fn draw_scaled_centered_guitar_glyph(
    svg: &mut SvgWriter,
    font: &MusicFont,
    glyph: Glyph,
    center_x: f64,
    center_y: f64,
    scale: f64,
) -> Result<(), crate::font::FontError> {
    let outline = font.glyph_outline(glyph)?;
    let x = center_x - outline.advance_width as f64 * scale / 2.0;
    let y = font
        .glyph_bbox_design_units(glyph)
        .map(|bbox| center_y - (bbox.y_top + bbox.y_bottom) * scale / 2.0)
        .unwrap_or(center_y);
    let transform = format!("translate({x}, {y}) scale({scale})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

fn standard_event_label_y(score: &GuitarScore, staff: &StaffLayout, event: &GuitarEvent) -> f64 {
    let clef = score.clef.to_clef();
    let highest_position = match &event.kind {
        GuitarEventKind::Pitched(_) if event.annotations.rhythmic_slash => {
            pitch_to_staff_position(&unpitched_written_pitch(3, score.clef), &clef)
        }
        GuitarEventKind::Pitched(notes) => notes
            .iter()
            .map(|note| {
                let pitch = written_pitch(
                    projected_sounding_pitch(event, *note, &score.bends),
                    score.clef,
                );
                pitch_to_staff_position(&pitch, &clef)
            })
            .max()
            .unwrap_or(8),
        GuitarEventKind::Dead(strings) => strings
            .iter()
            .map(|string| {
                pitch_to_staff_position(&unpitched_written_pitch(*string, score.clef), &clef)
            })
            .max()
            .unwrap_or(8),
        GuitarEventKind::Slash | GuitarEventKind::Percussion(_) => {
            pitch_to_staff_position(&unpitched_written_pitch(3, score.clef), &clef)
        }
        GuitarEventKind::Rest => 8,
    };
    let event_clearance = staff.y_of(highest_position) - 1.4 * staff.staff_space;
    let reserved_lane = if event.annotations.chord_symbol.is_some() {
        staff.y_origin - 5.2 * staff.staff_space
    } else {
        staff.y_origin - 2.0 * staff.staff_space
    };
    event_clearance.min(reserved_lane)
}

fn draw_guitar_event(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    score: &GuitarScore,
    standard_staff: &StaffLayout,
    tab_staff: &TabStaffLayout,
    event: &GuitarEvent,
    x: f64,
) -> Result<(), crate::font::FontError> {
    match &event.kind {
        GuitarEventKind::Pitched(notes) => {
            for note in notes {
                let mut layout = layout_fret_number(tab_staff, note.string, note.fret, x);
                if event.annotations.ghost_strings.contains(&note.string) {
                    layout.text = format!("({})", note.fret);
                    layout.bg_half_width *= 2.0;
                }
                draw_fret_number(svg, &layout);
            }
        }
        GuitarEventKind::Dead(strings) => {
            for &string in strings {
                draw_fret_number(svg, &layout_muted_string(tab_staff, string, x));
            }
        }
        GuitarEventKind::Slash => {
            let duration_log2 = super::event::duration_kind_to_log2(event.duration.kind());
            draw_guitar_glyph(
                svg,
                font,
                NoteheadStyle::Slash.glyph(duration_log2),
                x,
                tab_staff.y_origin
                    + (tab_staff.line_count as f64 - 1.0) * 0.5 * tab_staff.staff_space,
            )?;
        }
        GuitarEventKind::Percussion(target) => {
            let glyph = match target {
                PercussionTarget::Body => Glyph::GuitarGolpe,
                PercussionTarget::Fretboard => Glyph::NoteheadSquareBlack,
                PercussionTarget::Strings => Glyph::NoteheadXBlack,
            };
            draw_guitar_glyph(
                svg,
                font,
                glyph,
                x,
                tab_staff.y_origin
                    + (tab_staff.line_count as f64 - 1.0) * 0.5 * tab_staff.staff_space,
            )?;
        }
        GuitarEventKind::Rest => {}
    }

    let stroke_width = config.stem_thickness_fu();
    for string in event.strings() {
        if let Some(kind) = event.annotations.vibrato {
            draw_tab_vibrato(
                svg,
                &layout_tab_vibrato(tab_staff, string, x, kind, stroke_width),
            );
        }
    }
    for (string, harmonic) in &event.annotations.harmonics {
        let string_y = tab_staff.string_y(*string);
        match harmonic.kind {
            HarmonicKind::Natural => {
                draw_tab_harmonic(svg, &layout_tab_harmonic(tab_staff, *string, x), font)?;
            }
            HarmonicKind::Artificial { touch_fret } | HarmonicKind::Tapped { touch_fret, .. } => {
                draw_text_label(
                    svg,
                    x,
                    string_y - 0.72 * tab_staff.staff_space,
                    &format!("<{touch_fret}>"),
                    None,
                    0.58 * tab_staff.staff_space,
                    stroke_width,
                );
                draw_text_label(
                    svg,
                    x,
                    string_y - 1.38 * tab_staff.staff_space,
                    harmonic.abbreviation(),
                    None,
                    0.58 * tab_staff.staff_space,
                    stroke_width,
                );
            }
            HarmonicKind::Pinch => {
                draw_text_label(
                    svg,
                    x,
                    string_y - 0.72 * tab_staff.staff_space,
                    harmonic.abbreviation(),
                    None,
                    0.58 * tab_staff.staff_space,
                    stroke_width,
                );
            }
        }
        if let HarmonicKind::Tapped { hand, .. } = harmonic.kind {
            let glyph = match hand {
                TappingHand::Fretting => Glyph::GuitarLeftHandTapping,
                TappingHand::Picking => Glyph::GuitarRightHandTapping,
            };
            draw_guitar_glyph(svg, font, glyph, x + 0.7 * tab_staff.staff_space, string_y)?;
        }
    }

    if let Some(direction) = event.annotations.strum {
        let glyph = match direction {
            StrumDirection::Down => Glyph::GuitarStrumDown,
            StrumDirection::Up => Glyph::GuitarStrumUp,
        };
        draw_guitar_glyph(
            svg,
            font,
            glyph,
            x - 0.65 * tab_staff.staff_space,
            tab_staff.y_origin - 0.8 * tab_staff.staff_space,
        )?;
        draw_guitar_glyph(
            svg,
            font,
            glyph,
            x - 0.65 * standard_staff.staff_space,
            standard_staff.y_origin - 1.2 * standard_staff.staff_space,
        )?;
    }

    let standard_label_y = standard_event_label_y(score, standard_staff, event);
    for (row, (_, harmonic)) in event.annotations.harmonics.iter().enumerate() {
        draw_text_label(
            svg,
            x,
            standard_label_y - row as f64 * 0.8 * standard_staff.staff_space,
            harmonic.abbreviation(),
            None,
            0.58 * standard_staff.staff_space,
            stroke_width,
        );
    }

    for dot in 0..event.duration.num_dots() {
        svg.add_circle(
            x + (0.35 + dot as f64 * 0.22) * tab_staff.staff_space,
            tab_staff.y_origin - 1.5 * tab_staff.staff_space,
            0.07 * tab_staff.staff_space,
            "black",
            0.0,
            "black",
        );
    }
    draw_performance_annotations(
        svg,
        font,
        config,
        score,
        standard_staff,
        tab_staff,
        event,
        x,
    )?;

    Ok(())
}

fn draw_tab_tuplet(
    svg: &mut SvgWriter,
    tab_staff: &TabStaffLayout,
    xs: &[f64],
    number: u32,
    stroke_width: f64,
) {
    let Some((&start_x, &end_x)) = xs.first().zip(xs.last()) else {
        return;
    };
    let y = tab_staff.y_origin - 5.2 * tab_staff.staff_space;
    let hook = 0.4 * tab_staff.staff_space;
    svg.add_line(start_x, y, end_x, y, "black", stroke_width);
    svg.add_line(start_x, y, start_x, y + hook, "black", stroke_width);
    svg.add_line(end_x, y, end_x, y + hook, "black", stroke_width);
    svg.add_text(
        (start_x + end_x) / 2.0,
        y - 0.2 * tab_staff.staff_space,
        &number.to_string(),
        &TextStyle::normal(tab_staff.staff_space),
    );
}

pub(crate) fn draw_guitar_spans(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    score: &GuitarScore,
    annotation_layout: &GuitarAnnotationLayout,
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    system_staves: &[TabStaffLayout],
) -> Result<(), crate::font::FontError> {
    let stroke_width = config.stem_thickness_fu();
    for (span_index, span) in score.spans.iter().enumerate() {
        let Some(start) = anchors.get(&span.start) else {
            continue;
        };
        let Some(end) = anchors.get(&span.end) else {
            continue;
        };
        let lane = match &span.annotation {
            StoredGuitarSpan::Annotation(_) => annotation_layout.span_lane(span_index),
            StoredGuitarSpan::Technique { .. } => 0,
        };
        for (system_index, staff) in system_staves
            .iter()
            .enumerate()
            .take(end.system_index + 1)
            .skip(start.system_index)
        {
            let segment_start = if system_index == start.system_index {
                start.x
            } else if matches!(
                &span.annotation,
                StoredGuitarSpan::Annotation(GuitarSpanAnnotation::Barre(_))
            ) {
                staff.x + 0.8 * staff.staff_space
            } else {
                system_content_left(anchors, system_index, staff.x)
            };
            let segment_end = if system_index == end.system_index {
                end.x
            } else {
                staff.x + staff.width
            };
            match &span.annotation {
                StoredGuitarSpan::Technique {
                    kind: GuitarSpanKind::Slide,
                    string,
                } => {
                    if let Some(layout) =
                        layout_tab_slide(staff, *string, segment_start, segment_end, stroke_width)
                    {
                        draw_tab_slide(svg, &layout);
                    }
                }
                StoredGuitarSpan::Technique { kind, string }
                    if matches!(kind, GuitarSpanKind::HammerOn | GuitarSpanKind::PullOff) =>
                {
                    let kind = if *kind == GuitarSpanKind::HammerOn {
                        LegatoKind::HammerOn
                    } else {
                        LegatoKind::PullOff
                    };
                    if let Some(layout) = layout_tab_legato(
                        staff,
                        *string,
                        segment_start,
                        segment_end,
                        kind,
                        stroke_width,
                    ) {
                        draw_tab_legato(svg, &layout);
                    }
                }
                StoredGuitarSpan::Technique { kind, .. } => {
                    let is_first_segment = system_index == start.system_index;
                    match kind {
                        GuitarSpanKind::PalmMute => {
                            let lane_y = layout_tab_palm_mute(staff, segment_start).y;
                            if is_first_segment {
                                let text_x = segment_start
                                    + 0.5 * PALM_MUTE_DASH_OFFSET_SS * staff.staff_space;
                                draw_tab_palm_mute(svg, &layout_tab_palm_mute(staff, text_x));
                                if let Some(layout) = layout_tab_palm_mute_dash(
                                    staff,
                                    segment_start,
                                    segment_end,
                                    stroke_width,
                                ) {
                                    draw_tab_palm_mute_dash(svg, &layout);
                                }
                            } else if segment_end - segment_start > 0.3 * staff.staff_space {
                                svg.add_dashed_line(
                                    segment_start,
                                    lane_y,
                                    segment_end,
                                    lane_y,
                                    "black",
                                    stroke_width,
                                    "4,3",
                                );
                            }
                        }
                        GuitarSpanKind::LetRing => {
                            let lane_y = layout_tab_let_ring(staff, segment_start).y;
                            if is_first_segment {
                                let text_x = segment_start
                                    + 0.5 * LET_RING_DASH_OFFSET_SS * staff.staff_space;
                                draw_tab_let_ring(svg, &layout_tab_let_ring(staff, text_x));
                                if let Some(layout) = layout_tab_let_ring_dash(
                                    staff,
                                    segment_start,
                                    segment_end,
                                    stroke_width,
                                ) {
                                    draw_tab_let_ring_dash(svg, &layout);
                                }
                            } else if segment_end - segment_start > 0.3 * staff.staff_space {
                                svg.add_dashed_line(
                                    segment_start,
                                    lane_y,
                                    segment_end,
                                    lane_y,
                                    "black",
                                    stroke_width,
                                    "4,3",
                                );
                            }
                        }
                        GuitarSpanKind::Slide
                        | GuitarSpanKind::HammerOn
                        | GuitarSpanKind::PullOff => unreachable!("handled above"),
                    }
                }
                StoredGuitarSpan::Annotation(annotation) => {
                    draw_annotation_span_segment(
                        svg,
                        font,
                        staff,
                        annotation,
                        segment_start,
                        segment_end,
                        system_index == start.system_index,
                        system_index == end.system_index,
                        lane,
                        stroke_width,
                    )?;
                }
            }
        }
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct BendRenderMoment {
    x: f64,
    system_index: usize,
    time: Fraction,
}

pub(crate) fn draw_guitar_bends(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    score: &GuitarScore,
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
    standard_staves: &[StaffLayout],
    tab_staves: &[TabStaffLayout],
) {
    let system_bounds = render_system_timeline_bounds(anchors, timeline, tab_staves.len());
    for (gesture_index, gesture) in score.bends.iter().enumerate() {
        let Some(source_event) = score.event(gesture.source) else {
            continue;
        };
        let Some(source_note) = (match &source_event.kind {
            GuitarEventKind::Pitched(notes) => {
                notes.iter().find(|note| note.string == gesture.string)
            }
            GuitarEventKind::Dead(_)
            | GuitarEventKind::Slash
            | GuitarEventKind::Percussion(_)
            | GuitarEventKind::Rest => None,
        }) else {
            continue;
        };
        let Some(onset) = resolve_render_moment(gesture.onset, anchors, timeline) else {
            continue;
        };
        let Some(arrival) = resolve_render_moment(gesture.arrival, anchors, timeline) else {
            continue;
        };
        let source_pitch = BendPitch::exact(source_note.pitch);
        draw_bend_phase(
            svg,
            config,
            score,
            anchors,
            standard_staves,
            tab_staves,
            &system_bounds,
            gesture_index,
            gesture.string,
            BendSegmentPhase::Rise,
            onset,
            arrival,
            source_pitch,
            gesture.target,
            source_pitch,
            true,
        );

        if let Some(release) = gesture.release {
            let Some(release_start) = resolve_render_moment(release.start, anchors, timeline)
            else {
                continue;
            };
            let Some(release_end) = resolve_render_moment(release.end, anchors, timeline) else {
                continue;
            };
            if !same_render_moment(arrival, release_start) {
                draw_bend_phase(
                    svg,
                    config,
                    score,
                    anchors,
                    standard_staves,
                    tab_staves,
                    &system_bounds,
                    gesture_index,
                    gesture.string,
                    BendSegmentPhase::Hold,
                    arrival,
                    release_start,
                    gesture.target,
                    gesture.target,
                    source_pitch,
                    false,
                );
            }
            draw_bend_phase(
                svg,
                config,
                score,
                anchors,
                standard_staves,
                tab_staves,
                &system_bounds,
                gesture_index,
                gesture.string,
                BendSegmentPhase::Release,
                release_start,
                release_end,
                gesture.target,
                release.target,
                source_pitch,
                false,
            );
        } else if let Some(last_reattack) = gesture.reattacks.last() {
            if let Some(end) =
                resolve_render_moment(GuitarMoment::Onset(*last_reattack), anchors, timeline)
            {
                if !same_render_moment(arrival, end) {
                    draw_bend_phase(
                        svg,
                        config,
                        score,
                        anchors,
                        standard_staves,
                        tab_staves,
                        &system_bounds,
                        gesture_index,
                        gesture.string,
                        BendSegmentPhase::Hold,
                        arrival,
                        end,
                        gesture.target,
                        gesture.target,
                        source_pitch,
                        false,
                    );
                }
            }
        }

        for &reattack in &gesture.reattacks {
            if let Some(anchor) = anchors.get(&reattack) {
                draw_bend_reattack(
                    svg,
                    config,
                    score,
                    gesture_index,
                    gesture.string,
                    gesture.target,
                    source_pitch,
                    anchor,
                    standard_staves,
                    tab_staves,
                );
            }
        }
    }
}

fn resolve_render_moment(
    moment: GuitarMoment,
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
) -> Option<BendRenderMoment> {
    let event = moment.event();
    let anchor = anchors.get(&event)?;
    let location = timeline.get(&event)?;
    let (x, time) = match moment {
        GuitarMoment::Onset(_) => (anchor.x, location.onset),
        GuitarMoment::After { offset, .. } => {
            let event_span = anchor.end_x - anchor.x;
            let x = anchor.x + event_span * offset.ticks() as f64 / anchor.duration_ticks as f64;
            let time = location.onset
                + Fraction::new(
                    offset.ticks() as u128 * location.scale.num,
                    location.scale.den,
                );
            (x, time)
        }
    };
    Some(BendRenderMoment {
        x,
        system_index: anchor.system_index,
        time,
    })
}

fn same_render_moment(left: BendRenderMoment, right: BendRenderMoment) -> bool {
    left.time == right.time
}

fn render_system_timeline_bounds(
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    timeline: &HashMap<GuitarEventId, TimelineEvent<'_>>,
    system_count: usize,
) -> Vec<Option<(Fraction, Fraction)>> {
    let mut bounds: Vec<Option<(Fraction, Fraction)>> = vec![None; system_count];
    for (event, anchor) in anchors {
        let Some(location) = timeline.get(event) else {
            continue;
        };
        let Some(system) = bounds.get_mut(anchor.system_index) else {
            continue;
        };
        match system {
            Some((start, end)) => {
                *start = (*start).min(location.onset);
                *end = (*end).max(location.end);
            }
            None => *system = Some((location.onset, location.end)),
        }
    }
    bounds
}

fn bend_fragment_progress(
    phase_start: Fraction,
    phase_end: Fraction,
    system_start: Fraction,
    system_end: Fraction,
    is_first: bool,
    is_last: bool,
) -> (f64, f64) {
    if phase_start == phase_end {
        return (0.0, 1.0);
    }
    let fragment_start = if is_first {
        phase_start
    } else {
        system_start.max(phase_start)
    };
    let fragment_end = if is_last {
        phase_end
    } else {
        system_end.min(phase_end)
    };
    let start = phase_start.as_f64();
    let duration = phase_end.as_f64() - start;
    (
        ((fragment_start.as_f64() - start) / duration).clamp(0.0, 1.0),
        ((fragment_end.as_f64() - start) / duration).clamp(0.0, 1.0),
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_bend_phase(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    score: &GuitarScore,
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    standard_staves: &[StaffLayout],
    tab_staves: &[TabStaffLayout],
    system_bounds: &[Option<(Fraction, Fraction)>],
    gesture_index: usize,
    string: u8,
    phase: BendSegmentPhase,
    start: BendRenderMoment,
    end: BendRenderMoment,
    start_pitch: BendPitch,
    end_pitch: BendPitch,
    physical_source: BendPitch,
    show_amount: bool,
) {
    if end.system_index < start.system_index {
        return;
    }
    let system_count = end.system_index - start.system_index + 1;
    for system_index in start.system_index..=end.system_index {
        let fragment = if system_count == 1 {
            BendFragment::Complete
        } else if system_index == start.system_index {
            BendFragment::Start
        } else if system_index == end.system_index {
            BendFragment::End
        } else {
            BendFragment::Middle
        };
        let is_first_fragment = system_index == start.system_index;
        let is_last_fragment = system_index == end.system_index;
        let Some((system_start, system_end)) = system_bounds.get(system_index).copied().flatten()
        else {
            continue;
        };
        let (progress_start, progress_end) = bend_fragment_progress(
            start.time,
            end.time,
            system_start,
            system_end,
            is_first_fragment,
            is_last_fragment,
        );
        let x_start = if system_index == start.system_index {
            start.x
        } else {
            system_content_left(anchors, system_index, tab_staves[system_index].x)
        };
        let x_end = if system_index == end.system_index {
            end.x
        } else {
            tab_staves[system_index].x + tab_staves[system_index].width
        };

        for view in [BendView::Standard, BendView::Tab] {
            let (staff_space, y_at_start, y_at_end) = match view {
                BendView::Standard => {
                    let staff = &standard_staves[system_index];
                    (
                        staff.staff_space,
                        standard_bend_y(score, start_pitch, staff),
                        standard_bend_y(score, end_pitch, staff),
                    )
                }
                BendView::Tab => {
                    let staff = &tab_staves[system_index];
                    (
                        staff.staff_space,
                        tab_bend_y(start_pitch, physical_source, staff, string),
                        tab_bend_y(end_pitch, physical_source, staff, string),
                    )
                }
            };
            let y_start = lerp(y_at_start, y_at_end, progress_start);
            let y_end = lerp(y_at_start, y_at_end, progress_end);
            let amount_label = (show_amount && is_first_fragment && view == BendView::Tab)
                .then(|| bend_amount_label(physical_source, end_pitch));
            let target_label =
                (phase != BendSegmentPhase::Hold && is_last_fragment && view == BendView::Standard)
                    .then(|| bend_pitch_label(end_pitch));
            draw_bend_segment(
                svg,
                &BendSegmentLayout {
                    gesture_index,
                    string,
                    view,
                    phase,
                    fragment,
                    x_start,
                    y_start,
                    x_end,
                    y_end,
                    stroke_width: config.stem_thickness_fu(),
                    staff_space,
                    arrow_at_end: is_last_fragment && phase != BendSegmentPhase::Hold,
                    amount_label,
                    target_label,
                },
            );
        }
    }
}

fn system_content_left(
    anchors: &HashMap<GuitarEventId, GuitarRenderAnchor>,
    system_index: usize,
    fallback: f64,
) -> f64 {
    anchors
        .values()
        .filter(|anchor| anchor.system_index == system_index)
        .map(|anchor| anchor.x)
        .min_by(f64::total_cmp)
        .unwrap_or(fallback)
}

fn standard_bend_y(score: &GuitarScore, pitch: BendPitch, staff: &StaffLayout) -> f64 {
    let written = written_pitch(pitch.pitch, score.clef);
    let position = pitch_to_staff_position(&written, &score.clef.to_clef());
    staff.y_of(position) - pitch.cents as f64 / 100.0 * staff.half_space()
}

fn tab_bend_y(
    pitch: BendPitch,
    physical_source: BendPitch,
    staff: &TabStaffLayout,
    string: u8,
) -> f64 {
    let amount = pitch
        .total_cents()
        .saturating_sub(physical_source.total_cents()) as f64;
    let height = if amount == 0.0 {
        0.45
    } else {
        0.7_f64.max(amount / 200.0 * 1.6)
    };
    staff.string_y(string) - height * staff.staff_space
}

fn bend_amount_label(source: BendPitch, target: BendPitch) -> String {
    match target.total_cents() - source.total_cents() {
        50 => String::from("1/4"),
        100 => String::from("1/2"),
        200 => String::from("full"),
        300 => String::from("1 1/2"),
        cents => format!("{cents}c"),
    }
}

fn bend_pitch_label(pitch: BendPitch) -> String {
    if pitch.cents == 0 {
        pitch.pitch.to_string()
    } else {
        format!("{}+{}c", pitch.pitch, pitch.cents)
    }
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + (end - start) * progress
}

#[allow(clippy::too_many_arguments)]
fn draw_bend_reattack(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    score: &GuitarScore,
    gesture_index: usize,
    string: u8,
    target: BendPitch,
    physical_source: BendPitch,
    anchor: &GuitarRenderAnchor,
    standard_staves: &[StaffLayout],
    tab_staves: &[TabStaffLayout],
) {
    for view in [BendView::Standard, BendView::Tab] {
        let (y, staff_space) = match view {
            BendView::Standard => {
                let staff = &standard_staves[anchor.system_index];
                (
                    standard_bend_y(score, target, staff) - 0.65 * staff.staff_space,
                    staff.staff_space,
                )
            }
            BendView::Tab => {
                let staff = &tab_staves[anchor.system_index];
                (
                    tab_bend_y(target, physical_source, staff, string),
                    staff.staff_space,
                )
            }
        };
        svg.add_raw(&format!(
            "<g data-bend-gesture=\"{}\" data-bend-string=\"{}\" data-bend-view=\"{}\" data-bend-phase=\"reattack\">",
            gesture_index,
            string,
            view.label(),
        ));
        svg.add_circle(
            anchor.x,
            y,
            0.18 * staff_space,
            "black",
            config.stem_thickness_fu(),
            "white",
        );
        svg.add_line(
            anchor.x,
            y - 0.35 * staff_space,
            anchor.x,
            y + 0.35 * staff_space,
            "black",
            config.stem_thickness_fu(),
        );
        svg.add_raw("</g>");
    }
}

fn draw_performance_annotations(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    score: &GuitarScore,
    standard_staff: &StaffLayout,
    staff: &TabStaffLayout,
    event: &GuitarEvent,
    x: f64,
) -> Result<(), crate::font::FontError> {
    let ss = staff.staff_space;
    let stroke_width = config.stem_thickness_fu();
    let standard_base_y = standard_event_label_y(score, standard_staff, event);
    let mut standard_attack_row = event.annotations.harmonics.len();
    let mut whole_event_attack_row = 0usize;
    for annotation in &event.annotations.performance {
        match annotation {
            GuitarPerformanceAnnotation::LeftHandFinger { string, finger } => {
                let cx = x - 0.72 * ss;
                let cy = staff.string_y(*string);
                svg.add_circle(cx, cy, 0.34 * ss, "black", stroke_width, "white");
                svg.add_text(cx, cy, finger.label(), &centered_text_style(0.62 * ss));
            }
            GuitarPerformanceAnnotation::Attack { string, source } => {
                let tab_y = string
                    .map(|string| staff.string_y(string))
                    .unwrap_or_else(|| {
                        staff.bottom_y() + (1.1 + whole_event_attack_row as f64 * 0.8) * ss
                    });
                let standard_y =
                    standard_base_y - standard_attack_row as f64 * 0.8 * standard_staff.staff_space;
                match source {
                    AttackSource::Tap(hand) => {
                        let glyph = match hand {
                            TappingHand::Fretting => Glyph::GuitarLeftHandTapping,
                            TappingHand::Picking => Glyph::GuitarRightHandTapping,
                        };
                        draw_guitar_glyph(svg, font, glyph, x + 0.72 * ss, tab_y)?;
                        draw_guitar_glyph(svg, font, glyph, x, standard_y)?;
                    }
                    _ => {
                        let tab_x = if string.is_some() { x + 0.72 * ss } else { x };
                        draw_text_label(
                            svg,
                            tab_x,
                            tab_y,
                            source.label(),
                            None,
                            0.62 * ss,
                            stroke_width,
                        );
                        draw_text_label(
                            svg,
                            x,
                            standard_y,
                            source.label(),
                            None,
                            0.62 * standard_staff.staff_space,
                            stroke_width,
                        );
                    }
                }
                standard_attack_row += 1;
                if string.is_none() {
                    whole_event_attack_row += 1;
                }
            }
            GuitarPerformanceAnnotation::Text(text) => {
                let y = match text.placement {
                    TextPlacement::Above => staff.y_origin - 0.8 * ss,
                    TextPlacement::Below => staff.bottom_y() + 2.2 * ss,
                };
                draw_text_label(
                    svg,
                    x,
                    y,
                    &text.text,
                    text.enclosure,
                    0.72 * ss,
                    stroke_width,
                );
            }
            GuitarPerformanceAnnotation::Position(position) => {
                let label = position_label(*position);
                draw_text_label(
                    svg,
                    x,
                    staff.bottom_y() + 3.1 * ss,
                    &label,
                    None,
                    0.75 * ss,
                    stroke_width,
                );
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_annotation_span_segment(
    svg: &mut SvgWriter,
    font: &MusicFont,
    staff: &TabStaffLayout,
    annotation: &GuitarSpanAnnotation,
    start_x: f64,
    end_x: f64,
    starts_here: bool,
    ends_here: bool,
    lane: usize,
    stroke_width: f64,
) -> Result<(), crate::font::FontError> {
    let ss = staff.staff_space;
    let y = staff.bottom_y()
        + (GUITAR_ANNOTATION_LANE_BASE_SS + lane as f64 * GUITAR_ANNOTATION_LANE_PITCH_SS) * ss;
    let is_barre = matches!(annotation, GuitarSpanAnnotation::Barre(_));
    if is_barre {
        let fragment = match (starts_here, ends_here) {
            (true, true) => "complete",
            (true, false) => "start",
            (false, true) => "end",
            (false, false) => "middle",
        };
        svg.add_raw(&format!(
            "<g data-guitar-barre-fragment=\"{fragment}\" data-guitar-barre-lane=\"{lane}\" data-guitar-barre-length=\"{:.3}\">",
            end_x - start_x
        ));
    }
    svg.add_line(start_x, y, end_x, y, "black", stroke_width);
    if starts_here {
        svg.add_line(start_x, y, start_x, y - 0.35 * ss, "black", stroke_width);
    }
    if ends_here {
        svg.add_line(end_x, y, end_x, y - 0.35 * ss, "black", stroke_width);
    }
    if !starts_here {
        if is_barre {
            svg.add_raw("</g>");
        }
        return Ok(());
    }
    match annotation {
        GuitarSpanAnnotation::Barre(barre) => {
            let glyph = match barre.kind {
                BarreKind::Full => Glyph::GuitarBarreFull,
                BarreKind::Partial => Glyph::GuitarBarreHalf,
            };
            let outline = font.glyph_outline(glyph)?;
            draw_guitar_glyph(svg, font, glyph, start_x, y - 0.5 * ss)?;
            let label = roman_numeral(barre.fret);
            svg.add_text(
                start_x + outline.advance_width as f64 / 2.0 + 0.3 * ss,
                y - 0.45 * ss,
                &label,
                &TextStyle {
                    anchor: "start",
                    dominant_baseline: "central",
                    ..TextStyle::normal(0.72 * ss)
                },
            );
        }
        GuitarSpanAnnotation::Position(position) => {
            let label = position_label(*position);
            draw_text_label(
                svg,
                start_x,
                y - 0.45 * ss,
                &label,
                None,
                0.72 * ss,
                stroke_width,
            );
        }
        GuitarSpanAnnotation::CellBracket { label, enclosure } => {
            draw_text_label(
                svg,
                (start_x + end_x) / 2.0,
                y - 0.45 * ss,
                label,
                *enclosure,
                0.72 * ss,
                stroke_width,
            );
        }
        GuitarSpanAnnotation::Text(text) => {
            draw_text_label(
                svg,
                start_x,
                y - 0.45 * ss,
                &text.text,
                text.enclosure,
                0.72 * ss,
                stroke_width,
            );
        }
    }
    if is_barre {
        svg.add_raw("</g>");
    }
    Ok(())
}

fn push_legend_entry(
    entries: &mut Vec<TechniqueLegendEntry>,
    symbol: &'static str,
    description: &'static str,
) {
    if !entries
        .iter()
        .any(|entry| entry.symbol == symbol && entry.description == description)
    {
        entries.push(TechniqueLegendEntry {
            symbol,
            description,
        });
    }
}

fn draw_technique_legend(
    svg: &mut SvgWriter,
    font: &MusicFont,
    score: &GuitarScore,
    staff: &TabStaffLayout,
    y_offset_ss: f64,
) -> Result<(), crate::font::FontError> {
    let entries = score.technique_legend();
    if entries.is_empty() {
        return Ok(());
    }
    let x = staff.x + 3.0 * staff.staff_space;
    let y = staff.bottom_y() + y_offset_ss * staff.staff_space;
    svg.add_text(
        x,
        y,
        "Legend:",
        &TextStyle {
            anchor: "start",
            font_weight: "bold",
            ..TextStyle::normal(0.68 * staff.staff_space)
        },
    );
    for (row, entry) in entries.iter().enumerate() {
        let row_y = y + (row as f64 + 1.0) * GUITAR_LEGEND_ROW_HEIGHT_SS * staff.staff_space;
        let glyph = match entry.symbol {
            "↧" => Some((Glyph::GuitarStrumDown, "down")),
            "↥" => Some((Glyph::GuitarStrumUp, "up")),
            _ => None,
        };
        if let Some((glyph, direction)) = glyph {
            svg.add_raw(&format!(
                "<g data-guitar-legend-strum=\"{direction}\" data-guitar-legend-scale=\"{GUITAR_LEGEND_STRUM_SCALE}\">"
            ));
            draw_scaled_centered_guitar_glyph(
                svg,
                font,
                glyph,
                x + 0.85 * staff.staff_space,
                row_y,
                GUITAR_LEGEND_STRUM_SCALE,
            )?;
            svg.add_raw("</g>");
            svg.add_text(
                x + 1.4 * staff.staff_space,
                row_y,
                entry.description,
                &TextStyle {
                    anchor: "start",
                    dominant_baseline: "central",
                    ..TextStyle::normal(0.68 * staff.staff_space)
                },
            );
        } else {
            let text = format!("{}  {}", entry.symbol, entry.description);
            svg.add_text(
                x + staff.staff_space,
                row_y,
                &text,
                &TextStyle {
                    anchor: "start",
                    ..TextStyle::normal(0.68 * staff.staff_space)
                },
            );
        }
    }
    Ok(())
}

fn draw_text_label(
    svg: &mut SvgWriter,
    x: f64,
    y: f64,
    text: &str,
    enclosure: Option<TextEnclosure>,
    font_size: f64,
    stroke_width: f64,
) {
    let half_width = (text.chars().count().max(1) as f64 * font_size * 0.3).max(font_size * 0.45);
    let half_height = font_size * 0.62;
    match enclosure {
        Some(TextEnclosure::Circle) => svg.add_circle(
            x,
            y,
            half_width.max(half_height),
            "black",
            stroke_width,
            "white",
        ),
        Some(TextEnclosure::Rectangle) => svg.add_styled_rect(
            x - half_width - 0.15 * font_size,
            y - half_height,
            2.0 * (half_width + 0.15 * font_size),
            2.0 * half_height,
            &RectStyle::boxed(stroke_width),
        ),
        None => svg.add_rect(
            x - half_width,
            y - half_height,
            2.0 * half_width,
            2.0 * half_height,
            "white",
        ),
    }
    svg.add_text(x, y, text, &centered_text_style(font_size));
}

fn centered_text_style(font_size: f64) -> TextStyle<'static> {
    TextStyle {
        dominant_baseline: "central",
        ..TextStyle::normal(font_size)
    }
}

fn position_label(position: PositionLabel) -> String {
    let mut label = match position.shift {
        Some(ShiftDirection::Up) => String::from("↗"),
        Some(ShiftDirection::Down) => String::from("↘"),
        None => String::new(),
    };
    label.push_str(&roman_numeral(position.fret));
    label
}

fn roman_numeral(mut value: u8) -> String {
    if value == 0 {
        return String::from("0");
    }
    let mut result = String::new();
    for (number, numeral) in [(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")] {
        while value >= number {
            result.push_str(numeral);
            value -= number;
        }
    }
    result
}

#[cfg(test)]
pub(crate) fn rhythmic_anchor_xs(layout: &MeasureLayout) -> Vec<f64> {
    layout
        .elements
        .iter()
        .filter_map(|element| match &element.element {
            MeasureElement::Note(_) | MeasureElement::Chord(_) | MeasureElement::Rest(_) => {
                Some(vec![element.x])
            }
            MeasureElement::BeamGroup(group) => {
                let durations: Vec<_> = group.notes.iter().map(|note| note.duration_log2).collect();
                Some(
                    beam_group_note_x_offsets(&durations, element.width)
                        .into_iter()
                        .map(|offset| element.x + offset)
                        .collect(),
                )
            }
            MeasureElement::TupletGroup(group) => {
                let durations: Vec<_> = group
                    .beam_group
                    .notes
                    .iter()
                    .map(|note| note.duration_log2)
                    .collect();
                Some(
                    beam_group_note_x_offsets(&durations, element.width)
                        .into_iter()
                        .map(|offset| element.x + offset)
                        .collect(),
                )
            }
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(pitch: Pitch, duration: Duration, string: u8, fret: u8) -> GuitarEventSpec {
        GuitarEventSpec::pitched(pitch, duration, string, fret)
    }

    #[test]
    fn guitar_clef_in_secondary_voice_changes_other_voices_at_the_same_onset() {
        use crate::layout::system::MeasureEvent;

        let mut guitar = GuitarScore::standard();
        guitar.set_time_signature(2, 4);
        guitar.note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        guitar.note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        guitar.set_voice(1);
        guitar.note(Pitch::new(Note::B, 3), Duration::QTR, 2, 0).unwrap();
        guitar.clef_change(Clef::Bass);
        guitar.note(Pitch::new(Note::B, 3), Duration::QTR, 2, 0).unwrap();
        guitar.end_barline().unwrap();

        let contents = guitar.notation_builder().build_measure_contents().unwrap();
        let positions = contents[0].events.iter().filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.staff_position),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(
            positions,
            [
                pitch_to_staff_position(&Pitch::new(Note::E, 5), &Clef::Treble8ba),
                pitch_to_staff_position(&Pitch::new(Note::E, 4), &Clef::Bass),
            ]
        );
        assert!(matches!(contents[0].events[1],
            MeasureEvent::ClefChange(change) if change.clef == ClefKind::Bass));
    }

    #[test]
    fn guitar_mid_score_clef_and_meter_keep_earlier_measure_and_written_pitch() {
        use crate::layout::measure_meta::MeasureLength;
        use crate::layout::system::MeasureEvent;

        let mut guitar = GuitarScore::standard();
        guitar.set_time_signature(1, 4);
        guitar.set_measure_numbering(MeasureNumbering::EveryBar);
        guitar.set_first_measure_number(17);
        guitar.note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        guitar.barline().unwrap();
        guitar.time_signature_change(2, 4).unwrap();
        guitar.note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        guitar.set_clef(Clef::Bass);
        guitar.note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0).unwrap();
        guitar.end_barline().unwrap();

        let contents = guitar.notation_builder().build_measure_contents().unwrap();
        assert_eq!(contents[0].meta.nominal_length, Some(MeasureLength::new(1, 4)));
        assert_eq!(contents[1].meta.nominal_length, Some(MeasureLength::new(1, 2)));
        assert_eq!((contents[0].meta.number, contents[1].meta.number), (17, 18));
        let first = contents[0].events.iter().find_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.staff_position),
            _ => None,
        }).unwrap();
        let second_notes = contents[1].events.iter().filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.staff_position),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(first, pitch_to_staff_position(&Pitch::new(Note::E, 5), &Clef::Treble8ba));
        assert_eq!(second_notes[0], first);
        assert_eq!(second_notes[1], pitch_to_staff_position(&Pitch::new(Note::E, 4), &Clef::Bass));
        assert!(matches!(contents[1].events[0], MeasureEvent::TimeSignature(_)));
        assert!(contents[1].events.iter().any(|event| matches!(event,
            MeasureEvent::ClefChange(change) if change.clef == ClefKind::Bass)));
        let svg = guitar.try_render_svg().unwrap();
        assert!(svg.contains(">17</text>") && svg.contains(">18</text>"));
        let path = crate::font::bravura_font()
            .glyph_outline(Glyph::FClefChange).unwrap().path_data;
        assert!(svg.contains(&path), "guitar path must render the change-size F clef");
    }

    #[test]
    fn rejects_pitch_that_does_not_match_tuning() {
        let mut score = GuitarScore::standard();
        let error = score
            .note(Pitch::new(Note::F, 4), Duration::QTR, 1, 0)
            .unwrap_err();
        assert!(matches!(error, GuitarScoreError::PitchMismatch { .. }));
    }

    #[test]
    fn validates_tuplet_measure_duration_exactly() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        score
            .tuplet(
                3,
                2,
                vec![
                    spec(Pitch::new(Note::E, 4), Duration::EIGHTH, 1, 0),
                    spec(Pitch::new(Note::Fis, 4), Duration::EIGHTH, 1, 2),
                    spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
                ],
            )
            .unwrap();
        score.end_barline().unwrap();
    }

    #[test]
    fn rejects_incomplete_secondary_voice() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        score
            .note(Pitch::new(Note::E, 4), Duration::WHOLE, 1, 0)
            .unwrap();
        score.set_voice(1);
        score
            .note(Pitch::new(Note::B, 3), Duration::HALF, 2, 0)
            .unwrap();
        assert!(matches!(
            score.end_barline().unwrap_err(),
            GuitarScoreError::IncompleteMeasure { voice: 1, .. }
        ));
    }

    #[test]
    fn span_requires_matching_physical_string() {
        let mut score = GuitarScore::standard();
        let first = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        let second = score
            .note(Pitch::new(Note::Fis, 4), Duration::QTR, 1, 2)
            .unwrap();
        assert!(matches!(
            score
                .span(GuitarSpanKind::Slide, first, second, 2)
                .unwrap_err(),
            GuitarScoreError::SpanStringMismatch { .. }
        ));
    }
    #[test]
    fn tab_anchors_are_centered_on_standard_noteheads() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        let first = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        let triplet = score
            .tuplet(
                3,
                2,
                vec![
                    spec(Pitch::new(Note::E, 4), Duration::EIGHTH, 1, 0),
                    spec(Pitch::new(Note::Fis, 4), Duration::EIGHTH, 1, 2),
                    spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
                ],
            )
            .unwrap();
        score.rest(Duration::HALF);
        score.end_barline().unwrap();

        let notation = score.notation_builder();
        let contents = notation.build_measure_contents().unwrap();
        let prefix = notation.build_prefix();
        let font = crate::font::bravura_font();
        let config = font.engraving_config();
        let layout = crate::layout::system::layout_system(
            &prefix,
            &contents,
            &crate::layout::measure::MeasureLayoutConfig::from_staff_space(config.staff_space),
            Some(10_000.0),
        );
        let tab_staff = TabStaffLayout::new(500.0, 3000.0, 10_000.0, config.staff_space, 6);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 12_000.0, 6000.0);
        let annotation_layout = score.annotation_layout(&[(0, 1)]);
        let mut anchors = HashMap::new();
        draw_guitar_tab_system(
            &mut svg,
            &font,
            &config,
            &score,
            &annotation_layout,
            0,
            0,
            1,
            &layout,
            &StaffLayout::new(500.0, 1000.0, 10_000.0, config.staff_space),
            &tab_staff,
            &mut anchors,
        )
        .unwrap();

        let expected = rhythmic_anchor_xs(&layout.measures[0].layout);
        let ids = [first, triplet[0], triplet[1], triplet[2]];
        let center_offset = notehead_advance(&font, 2, NoteheadStyle::Normal).unwrap() / 2.0;
        for (id, expected_x) in ids.into_iter().zip(expected.into_iter().take(ids.len())) {
            let actual = anchors[&id].x;
            let absolute_expected =
                tab_staff.x + layout.measures[0].x_offset + expected_x + center_offset;
            assert!((actual - absolute_expected).abs() < 1e-9);
        }
    }

    #[test]
    fn standard_guitar_notation_writes_sounding_pitches_one_octave_higher() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        score
            .note(Pitch::new(Note::E, 2), Duration::QTR, 6, 0)
            .unwrap();
        score.end_barline().unwrap();

        let notation = score.notation_builder();
        assert_eq!(notation.build_prefix().clef_kind, ClefKind::Treble8ba);
        let contents = notation.build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(note) = &contents[0].events[0] else {
            panic!("expected one notation note");
        };
        let written_e3 = crate::layout::note_placement::pitch_to_staff_position(
            &Pitch::new(Note::E, 3),
            &Clef::Treble8ba,
        );
        assert_eq!(note.staff_position, written_e3);
    }

    #[test]
    fn first_tab_fret_after_barline_has_visible_clearance() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        score.barline().unwrap();
        let second = score
            .note(Pitch::new(Note::Fis, 4), Duration::QTR, 1, 2)
            .unwrap();
        score.end_barline().unwrap();

        let notation = score.notation_builder();
        let font = crate::font::bravura_font();
        let config = font.engraving_config();
        let mut measure_config =
            crate::layout::measure::MeasureLayoutConfig::from_staff_space(config.staff_space);
        measure_config.barline_width = GUITAR_BARLINE_WIDTH_SS * config.staff_space;
        let layout = crate::layout::system::layout_system(
            &notation.build_prefix(),
            &notation.build_measure_contents().unwrap(),
            &measure_config,
            Some(10_000.0),
        );
        let tab_staff = TabStaffLayout::new(500.0, 3000.0, 10_000.0, config.staff_space, 6);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 12_000.0, 6000.0);
        let annotation_layout = score.annotation_layout(&[(0, 2)]);
        let mut anchors = HashMap::new();
        draw_guitar_tab_system(
            &mut svg,
            &font,
            &config,
            &score,
            &annotation_layout,
            0,
            0,
            2,
            &layout,
            &StaffLayout::new(500.0, 1000.0, 10_000.0, config.staff_space),
            &tab_staff,
            &mut anchors,
        )
        .unwrap();

        let first_measure = &layout.measures[0];
        let barline_x = first_measure
            .layout
            .elements
            .iter()
            .find_map(|element| {
                matches!(element.element, MeasureElement::Barline(_))
                    .then_some(tab_staff.x + first_measure.x_offset + element.x)
            })
            .unwrap();
        let fret = layout_fret_number(&tab_staff, 1, 2, anchors[&second].x);
        let barline_right = barline_x + config.thin_barline_thickness_fu() / 2.0;
        let minimum_clearance = 0.25 * config.staff_space;
        assert!(
            fret.x - fret.bg_half_width >= barline_right + minimum_clearance,
            "fret background must clear the preceding barline"
        );
    }

    #[test]
    fn annotations_survive_inside_beam_groups() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        let ids = score
            .beam_group(vec![
                spec(Pitch::new(Note::E, 4), Duration::EIGHTH, 1, 0),
                spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
            ])
            .unwrap();
        score
            .annotate(ids[1], GuitarAnnotation::Vibrato(VibratoKind::Wide))
            .unwrap();
        score.end_barline().unwrap();
        let svg = score.render_svg();
        assert!(svg.contains("stroke-linecap=\"round\""));
    }

    #[test]
    fn palm_mute_span_is_split_across_systems() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(2, 4);
        let first = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        score
            .note(Pitch::new(Note::Fis, 4), Duration::QTR, 1, 2)
            .unwrap();
        score.barline().unwrap();
        score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let second = score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        score.end_barline().unwrap();
        score
            .span(GuitarSpanKind::PalmMute, first, second, 1)
            .unwrap();
        let svg = MultiStaffScore::guitar(score)
            .measures_per_system(1)
            .render_svg();
        assert_eq!(svg.matches("stroke-dasharray=\"4,3\"").count(), 2);
    }
    #[test]
    fn string_targeted_annotation_requires_a_played_string() {
        let mut score = GuitarScore::standard();
        let event = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        assert!(matches!(
            score
                .annotate(
                    event,
                    GuitarAnnotation::LeftHandFinger {
                        string: 2,
                        finger: LeftHandFinger::One,
                    },
                )
                .unwrap_err(),
            GuitarScoreError::AnnotationStringMismatch {
                event: _,
                string: 2
            }
        ));
    }

    #[test]
    fn physical_assignments_render_on_beam_and_chord_events() {
        let mut score = GuitarScore::standard();
        let beam = score
            .beam_group(vec![
                spec(Pitch::new(Note::E, 4), Duration::EIGHTH, 1, 0),
                spec(Pitch::new(Note::F, 4), Duration::EIGHTH, 1, 1),
            ])
            .unwrap();
        score
            .annotate(
                beam[1],
                GuitarAnnotation::Attack {
                    string: Some(1),
                    source: AttackSource::Tap(TappingHand::Picking),
                },
            )
            .unwrap();
        let chord = score
            .chord(
                vec![
                    FrettedPitch::new(Pitch::new(Note::Fis, 4), 1, 2),
                    FrettedPitch::new(Pitch::new(Note::D, 4), 2, 3),
                ],
                Duration::QTR,
            )
            .unwrap();
        score
            .annotate(
                chord,
                GuitarAnnotation::LeftHandFinger {
                    string: 2,
                    finger: LeftHandFinger::Three,
                },
            )
            .unwrap();
        score.show_technique_legend().end_barline().unwrap();

        let legend = score.technique_legend();
        assert_eq!(
            legend,
            vec![
                TechniqueLegendEntry {
                    symbol: "①–④",
                    description: "left-hand finger",
                },
                TechniqueLegendEntry {
                    symbol: "T(RH)",
                    description: "picking-hand tap",
                },
            ]
        );
        let svg = score.render_svg();
        assert!(svg.contains(">T(RH)  picking-hand tap</text>"));
        assert!(svg.contains(">①–④  left-hand finger</text>"));
    }

    #[test]
    fn annotation_span_continues_across_systems_and_labels_only_once() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        let first = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        score.barline().unwrap();
        let second = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.end_barline().unwrap();
        let baseline = MultiStaffScore::guitar(score.clone())
            .measures_per_system(1)
            .render_svg();
        score
            .annotation_span(
                GuitarSpanAnnotation::CellBracket {
                    label: String::from("cell A"),
                    enclosure: Some(TextEnclosure::Rectangle),
                },
                first,
                second,
            )
            .unwrap();
        let annotated = MultiStaffScore::guitar(score)
            .measures_per_system(1)
            .render_svg();

        assert_eq!(annotated.matches(">cell A</text>").count(), 1);
        assert_eq!(
            annotated.matches("<line ").count(),
            baseline.matches("<line ").count() + 4
        );
        assert!(annotated.matches("<rect ").count() > baseline.matches("<rect ").count());
    }
    #[test]
    fn semantic_bend_validation_rejects_non_source_string_and_invalid_arrival() {
        assert_eq!(
            BendPitch::new(Pitch::new(Note::A, 4), 100),
            Err(GuitarScoreError::InvalidBendCents { cents: 100 })
        );

        let mut score = GuitarScore::standard();
        let source = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let arrival = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let later = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();

        let wrong_string = BendGesture::new(
            source,
            2,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::onset(arrival),
        );
        assert!(matches!(
            score.bend(wrong_string).unwrap_err(),
            GuitarScoreError::BendSourceStringMismatch {
                event: _,
                string: 2
            }
        ));

        let no_motion_time = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::onset(source),
        );
        assert!(matches!(
            score.bend(no_motion_time).unwrap_err(),
            GuitarScoreError::InvalidBendOrder {
                earlier: BendPhase::Onset,
                later: BendPhase::Arrival
            }
        ));

        let onset_after_source = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::onset(later),
        )
        .starting_at(GuitarMoment::onset(arrival));
        assert!(matches!(
            score.bend(onset_after_source).unwrap_err(),
            GuitarScoreError::BendOnsetOutsideSource { .. }
        ));

        let invalid_offset = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::after(source, Duration::QTR),
        );
        assert!(matches!(
            score.bend(invalid_offset).unwrap_err(),
            GuitarScoreError::InvalidBendOffset {
                phase: BendPhase::Arrival,
                ..
            }
        ));
    }

    #[test]
    fn semantic_bend_validation_rejects_target_release_and_reattack_mismatches() {
        let mut score = GuitarScore::standard();
        let source = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let arrival = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let wrong_reattack = score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        let release_end = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();

        let downward_target = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::G, 4)),
            GuitarMoment::onset(arrival),
        );
        assert!(matches!(
            score.bend(downward_target).unwrap_err(),
            GuitarScoreError::InvalidBendTarget { .. }
        ));

        let invalid_release = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::onset(arrival),
        )
        .with_release(BendRelease::new(
            GuitarMoment::onset(wrong_reattack),
            GuitarMoment::onset(release_end),
            BendPitch::exact(Pitch::new(Note::F, 4)),
        ));
        assert!(matches!(
            score.bend(invalid_release).unwrap_err(),
            GuitarScoreError::InvalidBendReleaseTarget { .. }
        ));

        let mismatched_reattack = BendGesture::new(
            source,
            1,
            BendPitch::exact(Pitch::new(Note::A, 4)),
            GuitarMoment::onset(arrival),
        )
        .with_release(BendRelease::new(
            GuitarMoment::onset(wrong_reattack),
            GuitarMoment::onset(release_end),
            BendPitch::exact(Pitch::new(Note::G, 4)),
        ))
        .reattacked_at(wrong_reattack);
        assert!(matches!(
            score.bend(mismatched_reattack).unwrap_err(),
            GuitarScoreError::BendReattackMismatch { .. }
        ));
    }

    #[test]
    fn semantic_bend_validation_rejects_cross_voice_duplicate_and_overlap() {
        let mut cross_voice = GuitarScore::standard();
        let source = cross_voice
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        cross_voice.set_voice(1);
        let other_voice = cross_voice
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        assert!(matches!(
            cross_voice
                .bend(BendGesture::new(
                    source,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(other_voice),
                ))
                .unwrap_err(),
            GuitarScoreError::BendVoiceMismatch {
                phase: BendPhase::Arrival,
                ..
            }
        ));

        let mut overlapping = GuitarScore::standard();
        let first = overlapping
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let second = overlapping
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let third = overlapping
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        overlapping
            .bend(BendGesture::new(
                first,
                1,
                BendPitch::exact(Pitch::new(Note::A, 4)),
                GuitarMoment::onset(second),
            ))
            .unwrap();
        assert!(matches!(
            overlapping
                .bend(BendGesture::new(
                    first,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(second),
                ))
                .unwrap_err(),
            GuitarScoreError::DuplicateBendGesture { .. }
        ));
        assert!(matches!(
            overlapping
                .bend(BendGesture::new(
                    second,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(third),
                ))
                .unwrap_err(),
            GuitarScoreError::OverlappingBendGesture {
                voice: 0,
                string: 1
            }
        ));
    }

    #[test]
    fn semantic_bend_moments_inherit_tuplet_timing() {
        let mut score = GuitarScore::standard();
        let events = score
            .tuplet(
                3,
                2,
                vec![
                    spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
                    spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
                    spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3),
                ],
            )
            .unwrap();
        score
            .bend(
                BendGesture::new(
                    events[0],
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(events[2]),
                )
                .starting_at(GuitarMoment::after(events[0], Duration::SIXTEENTH)),
            )
            .unwrap();
    }

    #[test]
    fn arbitrary_guitar_tuplet_ratio_controls_standard_layout_advance() {
        fn following_note_x(in_time_of: u32) -> (f64, u32) {
            let mut score = GuitarScore::standard();
            score
                .tuplet(
                    5,
                    in_time_of,
                    (0..5)
                        .map(|_| spec(Pitch::new(Note::G, 4), Duration::EIGHTH, 1, 3))
                        .collect(),
                )
                .unwrap();
            score
                .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
                .unwrap();
            score.end_barline().unwrap();

            let notation = score.notation_builder();
            let contents = notation.build_measure_contents().unwrap();
            let crate::layout::system::MeasureEvent::TupletGroup(tuplet) = &contents[0].events[0]
            else {
                panic!("expected tuplet group")
            };
            let config = crate::layout::measure::MeasureLayoutConfig::from_staff_space(
                crate::font::bravura_font().engraving_config().staff_space,
            );
            let layout = crate::layout::system::layout_system(
                &notation.build_prefix(),
                &contents,
                &config,
                None,
            );
            let following_x = layout.measures[0]
                .layout
                .elements
                .iter()
                .find_map(|element| {
                    matches!(element.element, MeasureElement::Note(_)).then_some(element.x)
                })
                .expect("following note must be laid out");
            (following_x, tuplet.in_time_of)
        }

        let (three_x, carried_ratio) = following_note_x(3);
        let (four_x, _) = following_note_x(4);
        assert_eq!(carried_ratio, 3);
        assert!(
            three_x < four_x,
            "a 5:3 tuplet must advance less than the same notes in 5:4"
        );
    }

    #[test]
    fn bend_fragment_progress_uses_absolute_times_at_partial_system_edges() {
        let phase_start = Fraction::new(3, 4);
        let phase_end = Fraction::new(9, 4);

        let first = bend_fragment_progress(
            phase_start,
            phase_end,
            Fraction::new(0, 1),
            Fraction::new(1, 1),
            true,
            false,
        );
        let middle = bend_fragment_progress(
            phase_start,
            phase_end,
            Fraction::new(1, 1),
            Fraction::new(2, 1),
            false,
            false,
        );
        let last = bend_fragment_progress(
            phase_start,
            phase_end,
            Fraction::new(2, 1),
            Fraction::new(3, 1),
            false,
            true,
        );

        assert!((first.0 - 0.0).abs() < f64::EPSILON);
        assert!((first.1 - 1.0 / 6.0).abs() < 1e-12);
        assert!((middle.0 - 1.0 / 6.0).abs() < 1e-12);
        assert!((middle.1 - 5.0 / 6.0).abs() < 1e-12);
        assert!((last.0 - 5.0 / 6.0).abs() < 1e-12);
        assert!((last.1 - 1.0).abs() < f64::EPSILON);
    }

    fn five_measure_bend_score() -> GuitarScore {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        let source = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.barline().unwrap();
        score.rest(Duration::QTR);
        score.barline().unwrap();
        let arrival = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.barline().unwrap();
        let reattack = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.barline().unwrap();
        let release_end = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.end_barline().unwrap();
        score
            .bend(
                BendGesture::new(
                    source,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(arrival),
                )
                .with_release(BendRelease::new(
                    GuitarMoment::after(reattack, Duration::EIGHTH),
                    GuitarMoment::onset(release_end),
                    BendPitch::exact(Pitch::new(Note::G, 4)),
                ))
                .reattacked_at(reattack),
            )
            .unwrap();
        score
    }

    #[test]
    fn semantic_bend_splits_each_phase_on_systems_but_not_barlines() {
        let cross_system = MultiStaffScore::guitar(five_measure_bend_score())
            .measures_per_system(1)
            .render_svg();
        assert_eq!(
            cross_system
                .matches("data-bend-fragment=\"middle\"")
                .count(),
            2,
            "the three-system rise has one middle fragment on each staff"
        );
        assert_eq!(
            cross_system.matches("data-bend-phase=\"reattack\"").count(),
            2,
            "one reattack marker is coordinated across standard and TAB"
        );
        assert_eq!(cross_system.matches(">full</text>").count(), 1);
        assert_eq!(cross_system.matches(">A4</text>").count(), 1);

        let across_barlines = MultiStaffScore::guitar(five_measure_bend_score())
            .measures_per_system(5)
            .render_svg();
        assert_eq!(
            across_barlines
                .matches("data-bend-fragment=\"complete\"")
                .count(),
            6,
            "rise, hold, and release stay whole on both staves across barlines"
        );
        assert!(!across_barlines.contains("data-bend-fragment=\"start\""));
        assert!(!across_barlines.contains("data-bend-fragment=\"middle\""));
        assert!(!across_barlines.contains("data-bend-fragment=\"end\""));
    }

    #[test]
    fn chord_bend_targets_only_the_selected_physical_string() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        let source = score
            .chord(
                vec![
                    FrettedPitch::new(Pitch::new(Note::G, 4), 1, 3),
                    FrettedPitch::new(Pitch::new(Note::D, 4), 2, 3),
                ],
                Duration::QTR,
            )
            .unwrap();
        score.barline().unwrap();
        let arrival = score
            .chord(
                vec![
                    FrettedPitch::new(Pitch::new(Note::G, 4), 1, 3),
                    FrettedPitch::new(Pitch::new(Note::D, 4), 2, 3),
                ],
                Duration::QTR,
            )
            .unwrap();
        score.end_barline().unwrap();
        score
            .bend(BendGesture::new(
                source,
                1,
                BendPitch::exact(Pitch::new(Note::A, 4)),
                GuitarMoment::onset(arrival),
            ))
            .unwrap();
        let svg = score.render_svg();
        assert!(svg.contains("data-bend-string=\"1\""));
        assert!(!svg.contains("data-bend-string=\"2\""));
    }

    #[test]
    fn arrival_and_release_project_their_sounding_pitches_on_standard_staff() {
        let mut score = GuitarScore::standard();
        let source = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let arrival = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let release_start = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let landing = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.end_barline().unwrap();
        score
            .bend(
                BendGesture::new(
                    source,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(arrival),
                )
                .with_release(BendRelease::new(
                    GuitarMoment::onset(release_start),
                    GuitarMoment::onset(landing),
                    BendPitch::exact(Pitch::new(Note::Gis, 4)),
                )),
            )
            .unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(arrival_note) = &contents[0].events[1] else {
            panic!("expected projected arrival note")
        };
        let crate::layout::system::MeasureEvent::Note(release_start_note) = &contents[0].events[2]
        else {
            panic!("expected projected release-start note")
        };
        let crate::layout::system::MeasureEvent::Note(landing_note) = &contents[0].events[3] else {
            panic!("expected projected release note")
        };
        assert_eq!(
            arrival_note.staff_position,
            pitch_to_staff_position(&Pitch::new(Note::A, 5), &Clef::Treble8ba)
        );
        assert_eq!(
            release_start_note.staff_position,
            pitch_to_staff_position(&Pitch::new(Note::A, 5), &Clef::Treble8ba),
            "release motion starts from the held bend target"
        );
        assert_eq!(
            landing_note.staff_position,
            pitch_to_staff_position(&Pitch::new(Note::Gis, 5), &Clef::Treble8ba)
        );
        assert!(
            landing_note.accidental.is_some(),
            "the partial-release target must replace the physical G with G-sharp"
        );
    }

    #[test]
    fn held_bend_reattack_projects_the_sounding_target_on_standard_staff() {
        let mut score = GuitarScore::standard();
        let source = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let reattack = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.end_barline().unwrap();
        score
            .bend(
                BendGesture::new(
                    source,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(reattack),
                )
                .reattacked_at(reattack),
            )
            .unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(note) = &contents[0].events[1] else {
            panic!("expected projected reattack note")
        };
        let written_target = pitch_to_staff_position(&Pitch::new(Note::A, 5), &Clef::Treble8ba);
        assert_eq!(note.staff_position, written_target);
    }

    #[test]
    fn pre_bend_projects_the_sounding_target_on_standard_staff() {
        let mut score = GuitarScore::standard();
        let source = score
            .note(Pitch::new(Note::B, 3), Duration::QTR, 2, 0)
            .unwrap();
        score.end_barline().unwrap();
        score
            .bend(BendGesture::pre_bend(
                source,
                2,
                BendPitch::exact(Pitch::new(Note::Cis, 4)),
            ))
            .unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(note) = &contents[0].events[0] else {
            panic!("expected projected pre-bend note")
        };
        let written_target = pitch_to_staff_position(&Pitch::new(Note::Cis, 5), &Clef::Treble8ba);
        assert_eq!(note.staff_position, written_target);
    }

    #[test]
    fn harmonic_kinds_validate_and_project_distinct_sounding_semantics() {
        let mut score = GuitarScore::standard();
        let natural = score
            .note(Pitch::new(Note::E, 5), Duration::QTR, 1, 12)
            .unwrap();
        let artificial = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        let pinch = score
            .note(Pitch::new(Note::D, 4), Duration::QTR, 3, 7)
            .unwrap();
        let tapped = score
            .note(Pitch::new(Note::F, 3), Duration::QTR, 4, 3)
            .unwrap();
        for (event, string, harmonic) in [
            (natural, 1, Harmonic::natural(Pitch::new(Note::E, 5))),
            (
                artificial,
                2,
                Harmonic::artificial(Pitch::new(Note::E, 5), 17),
            ),
            (pinch, 3, Harmonic::pinch(Pitch::new(Note::D, 5))),
            (
                tapped,
                4,
                Harmonic::tapped(Pitch::new(Note::F, 4), 15, TappingHand::Picking),
            ),
        ] {
            score
                .annotate(event, GuitarAnnotation::Harmonic { string, harmonic })
                .unwrap();
            assert_eq!(
                event_note_annotations(score.event(event).unwrap()).notehead_styles,
                vec![NoteheadStyle::Diamond]
            );
        }
        score.end_barline().unwrap();

        let notation = score.notation_builder().build_measure_contents().unwrap();
        let expected = [
            Pitch::new(Note::E, 6),
            Pitch::new(Note::E, 6),
            Pitch::new(Note::D, 6),
            Pitch::new(Note::F, 5),
        ];
        for (event, pitch) in notation[0].events.iter().zip(expected) {
            let crate::layout::system::MeasureEvent::Note(note) = event else {
                panic!("every harmonic projects as one sounding note");
            };
            assert_eq!(
                note.staff_position,
                pitch_to_staff_position(&pitch, &Clef::Treble8ba)
            );
            assert_eq!(
                note.annotations.notehead_styles,
                vec![NoteheadStyle::Diamond]
            );
        }

        let svg = score.clone().render_svg();
        assert!(svg.contains(">A.H.</text>"));
        assert!(svg.contains(">P.H.</text>"));
        assert!(svg.contains(">T.H.</text>"));
        assert!(svg.contains("&lt;17&gt;"));
        assert!(svg.contains("&lt;15&gt;"));

        let mut invalid = GuitarScore::standard();
        let event = invalid
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        assert!(matches!(
            invalid
                .annotate(
                    event,
                    GuitarAnnotation::Harmonic {
                        string: 2,
                        harmonic: Harmonic::artificial(Pitch::new(Note::E, 5), 5),
                    },
                )
                .unwrap_err(),
            GuitarScoreError::InvalidHarmonicTouch { .. }
        ));
    }

    #[test]
    fn realized_comping_slash_keeps_tab_frets_and_uses_one_standard_slash() {
        let mut score = GuitarScore::standard();
        let event = score
            .chord(
                vec![
                    FrettedPitch::new(Pitch::new(Note::Fis, 4), 1, 2),
                    FrettedPitch::new(Pitch::new(Note::D, 4), 2, 3),
                ],
                Duration::QTR,
            )
            .unwrap();
        score
            .annotate(event, GuitarAnnotation::RhythmicSlash)
            .unwrap()
            .annotate(event, GuitarAnnotation::ChordSymbol(String::from("D")))
            .unwrap()
            .annotate(event, GuitarAnnotation::Strum(StrumDirection::Down))
            .unwrap()
            .end_barline()
            .unwrap();
        assert_eq!(
            score
                .annotate(event, GuitarAnnotation::RhythmicSlash)
                .unwrap_err(),
            GuitarScoreError::DuplicateRhythmicSlash { event: event.get() }
        );
        assert!(matches!(
            score
                .annotate(event, GuitarAnnotation::Ghost { string: Some(1) })
                .unwrap_err(),
            GuitarScoreError::IncompatibleAnnotations {
                first: "ghost",
                second: "rhythmic slash",
                ..
            }
        ));

        let notation = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(note) = &notation[0].events[0] else {
            panic!("a realized comping chord projects to one neutral slash");
        };
        assert_eq!(note.annotations.notehead_styles, vec![NoteheadStyle::Slash]);
        assert_eq!(note.annotations.chord_symbol.as_deref(), Some("D"));

        let font = crate::font::bravura_font();
        let strum_path = font
            .glyph_outline(Glyph::GuitarStrumDown)
            .unwrap()
            .path_data;
        let svg = score.render_svg();
        assert!(svg.contains(">2</text>") && svg.contains(">3</text>"));
        assert_eq!(
            svg.matches(&strum_path).count(),
            2,
            "the same semantic strum is visible on standard notation and TAB"
        );
    }

    #[test]
    fn attack_and_percussion_families_keep_distinct_standard_noteheads() {
        let mut score = GuitarScore::standard();
        let slap = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        let pop = score
            .note(Pitch::new(Note::F, 4), Duration::QTR, 1, 1)
            .unwrap();
        let ghost = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let dead = score.dead(vec![1, 2], Duration::QTR).unwrap();
        score
            .annotate(
                slap,
                GuitarAnnotation::Attack {
                    string: Some(1),
                    source: AttackSource::Slap(SlapTechnique::Thumb),
                },
            )
            .unwrap()
            .annotate(
                pop,
                GuitarAnnotation::Attack {
                    string: Some(1),
                    source: AttackSource::Slap(SlapTechnique::Pop),
                },
            )
            .unwrap()
            .annotate(ghost, GuitarAnnotation::Ghost { string: Some(1) })
            .unwrap();

        assert_eq!(
            event_note_annotations(score.event(slap).unwrap()).notehead_styles,
            vec![NoteheadStyle::CircleX]
        );
        assert_eq!(
            event_note_annotations(score.event(pop).unwrap()).notehead_styles,
            vec![NoteheadStyle::Diamond]
        );
        let ghost_annotations = event_note_annotations(score.event(ghost).unwrap());
        assert_eq!(
            ghost_annotations.notehead_styles,
            vec![NoteheadStyle::Normal]
        );
        assert_eq!(ghost_annotations.parenthesized_noteheads, vec![true]);
        assert_eq!(
            event_note_annotations(score.event(dead).unwrap()).notehead_styles,
            vec![NoteheadStyle::X, NoteheadStyle::X]
        );

        let body = score.percussion(PercussionTarget::Body, Duration::QTR);
        let fretboard = score.percussion(PercussionTarget::Fretboard, Duration::QTR);
        let strings = score.percussion(PercussionTarget::Strings, Duration::QTR);
        for (event, expected) in [
            (body, NoteheadStyle::CircleX),
            (fretboard, NoteheadStyle::Square),
            (strings, NoteheadStyle::X),
        ] {
            assert_eq!(
                event_note_annotations(score.event(event).unwrap()).notehead_styles,
                vec![expected]
            );
        }
        score.show_technique_legend().end_barline().unwrap();
        let contents = score.notation_builder().build_measure_contents().unwrap();
        assert!(
            contents[0]
                .events
                .iter()
                .all(|event| !matches!(event, crate::layout::system::MeasureEvent::Rest(_))),
            "dead and percussion events must never collapse to standard rests"
        );
        let svg = score.render_svg();
        assert!(svg.contains(">slap  thumb slap</text>"));
        assert!(svg.contains(">pop  string pop</text>"));
        assert!(svg.contains(">(note)  pitched ghost attack</text>"));
        assert!(svg.contains(">x  dead/muted attack</text>"));
        assert!(svg.contains(">golpe  body percussion</text>"));
        assert!(svg.contains(">square  fretboard percussion</text>"));
    }

    #[test]
    fn grouped_dead_slash_and_percussion_preserve_styles_and_strums() {
        let mut score = GuitarScore::standard();
        let beam = score
            .beam_group(vec![
                GuitarEventSpec::Dead {
                    strings: vec![1, 2],
                    duration: Duration::EIGHTH,
                },
                GuitarEventSpec::Slash {
                    duration: Duration::EIGHTH,
                },
            ])
            .unwrap();
        score
            .annotate(beam[0], GuitarAnnotation::Strum(StrumDirection::Down))
            .unwrap()
            .annotate(beam[1], GuitarAnnotation::Strum(StrumDirection::Up))
            .unwrap();
        score
            .tuplet(
                3,
                2,
                vec![
                    GuitarEventSpec::Percussion {
                        target: PercussionTarget::Body,
                        duration: Duration::EIGHTH,
                    },
                    GuitarEventSpec::Percussion {
                        target: PercussionTarget::Fretboard,
                        duration: Duration::EIGHTH,
                    },
                    GuitarEventSpec::Slash {
                        duration: Duration::EIGHTH,
                    },
                ],
            )
            .unwrap();
        score.end_barline().unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::BeamGroup(beam) = &contents[0].events[0] else {
            panic!("expected heterogeneous beam group");
        };
        assert_eq!(
            beam.notes[0].annotations.notehead_styles,
            vec![NoteheadStyle::X, NoteheadStyle::X]
        );
        assert_eq!(
            beam.notes[0]
                .annotations
                .grouped_chord
                .as_ref()
                .expect("grouped dead attack retains all standard heads")
                .staff_positions
                .len(),
            2
        );
        assert_eq!(
            beam.notes[1].annotations.notehead_styles,
            vec![NoteheadStyle::Slash]
        );
        let crate::layout::system::MeasureEvent::TupletGroup(tuplet) = &contents[0].events[1]
        else {
            panic!("expected heterogeneous tuplet group");
        };
        assert_eq!(
            tuplet.beam_group.notes[0].annotations.notehead_styles,
            vec![NoteheadStyle::CircleX]
        );
        assert_eq!(
            tuplet.beam_group.notes[1].annotations.notehead_styles,
            vec![NoteheadStyle::Square]
        );
        assert_eq!(
            tuplet.beam_group.notes[2].annotations.notehead_styles,
            vec![NoteheadStyle::Slash]
        );

        let font = crate::font::bravura_font();
        let down = font
            .glyph_outline(Glyph::GuitarStrumDown)
            .unwrap()
            .path_data;
        let up = font.glyph_outline(Glyph::GuitarStrumUp).unwrap().path_data;
        let svg = score.render_svg();
        assert_eq!(svg.matches(&down).count(), 2);
        assert_eq!(svg.matches(&up).count(), 2);
    }

    #[test]
    fn capo_and_named_scordatura_drive_validation_and_visible_setup() {
        let tuning = GuitarTuning::named(
            "DADGAD",
            vec![
                Pitch::new(Note::D, 4),
                Pitch::new(Note::A, 3),
                Pitch::new(Note::G, 3),
                Pitch::new(Note::D, 3),
                Pitch::new(Note::A, 2),
                Pitch::new(Note::D, 2),
            ],
        )
        .unwrap();
        let mut score = GuitarScore::new(tuning.clone());
        score
            .set_capo(Some(Capo::new(2).unwrap()))
            .unwrap()
            .set_tuning_display(TuningDisplay::NameAndPitches);
        score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        assert_eq!(
            score.setup_label().as_deref(),
            Some("Capo II · Tuning: DADGAD · 1=D4 2=A3 3=G3 4=D3 5=A2 6=D2")
        );
        assert_eq!(
            score.set_capo(Some(Capo::new(3).unwrap())).unwrap_err(),
            GuitarScoreError::SetupLocked
        );
        score.end_barline().unwrap();
        let svg = score.render_svg();
        assert_eq!(
            svg.matches("Capo II · Tuning: DADGAD · 1=D4 2=A3 3=G3 4=D3 5=A2 6=D2")
                .count(),
            1
        );
        assert!(svg.contains("data-guitar-setup-header=\"true\""));
        assert!(svg.contains(">0</text>"), "TAB fret remains capo-relative");

        let mut mismatch = GuitarScore::new(tuning);
        mismatch.set_capo(Some(Capo::new(2).unwrap())).unwrap();
        assert!(matches!(
            mismatch
                .note(Pitch::new(Note::D, 4), Duration::QTR, 1, 0)
                .unwrap_err(),
            GuitarScoreError::PitchMismatch {
                expected_midi: 64,
                ..
            }
        ));
    }

    #[test]
    fn barre_validation_and_cross_system_fragment_ownership_are_explicit() {
        fn chord_at(score: &mut GuitarScore, fret: u8) -> GuitarEventId {
            score
                .chord(
                    vec![
                        FrettedPitch::new(
                            Pitch::new(
                                match fret {
                                    2 => Note::Fis,
                                    3 => Note::G,
                                    _ => Note::A,
                                },
                                4,
                            ),
                            1,
                            fret,
                        ),
                        FrettedPitch::new(
                            Pitch::new(
                                match fret {
                                    2 => Note::Cis,
                                    3 => Note::D,
                                    _ => Note::E,
                                },
                                4,
                            ),
                            2,
                            fret,
                        ),
                    ],
                    Duration::QTR,
                )
                .unwrap()
        }

        let mut score = GuitarScore::standard();
        score.set_time_signature(1, 4);
        let start = chord_at(&mut score, 2);
        score.barline().unwrap();
        chord_at(&mut score, 3);
        score.barline().unwrap();
        let end = chord_at(&mut score, 5);
        score.end_barline().unwrap();
        score
            .annotation_span(
                GuitarSpanAnnotation::Barre(GuitarBarre::new(BarreKind::Partial, 2, 1, 2).unwrap()),
                start,
                end,
            )
            .unwrap();
        let svg = MultiStaffScore::guitar(score)
            .measures_per_system(1)
            .render_svg();
        assert_eq!(
            svg.matches("data-guitar-barre-fragment=\"start\"").count(),
            1
        );
        assert_eq!(
            svg.matches("data-guitar-barre-fragment=\"middle\"").count(),
            1
        );
        assert_eq!(svg.matches("data-guitar-barre-fragment=\"end\"").count(), 1);
        assert_eq!(svg.matches(">II</text>").count(), 1);
        assert_eq!(svg.matches(">CII</text>").count(), 0);
        let end_fragment = svg
            .split("data-guitar-barre-fragment=\"end\"")
            .nth(1)
            .expect("cross-system barre has an ending fragment");
        assert!(
            !end_fragment
                .starts_with(" data-guitar-barre-lane=\"0\" data-guitar-barre-length=\"0.000\""),
            "a barre ending at the first event must retain a visible continuation"
        );

        let mut invalid = GuitarScore::standard();
        invalid.set_time_signature(1, 4);
        let start = chord_at(&mut invalid, 2);
        invalid.barline().unwrap();
        let rest = invalid.rest(Duration::QTR);
        invalid.end_barline().unwrap();
        assert!(matches!(
            invalid
                .annotation_span(
                    GuitarSpanAnnotation::Barre(
                        GuitarBarre::new(BarreKind::Partial, 2, 1, 2).unwrap(),
                    ),
                    start,
                    rest,
                )
                .unwrap_err(),
            GuitarScoreError::BarreEndpointMismatch { event } if event == rest.get()
        ));
    }

    #[test]
    fn artificial_harmonics_use_partial_pitch_and_reject_non_nodes() {
        let mut score = GuitarScore::standard();
        let seventh_partial = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        score
            .annotate(
                seventh_partial,
                GuitarAnnotation::Harmonic {
                    string: 2,
                    harmonic: Harmonic::artificial(Pitch::new(Note::B, 5), 12),
                },
            )
            .unwrap();

        let fifth_partial = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        score
            .annotate(
                fifth_partial,
                GuitarAnnotation::Harmonic {
                    string: 2,
                    harmonic: Harmonic::tapped(Pitch::new(Note::E, 6), 10, TappingHand::Picking),
                },
            )
            .unwrap();

        let wrong_pitch = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        assert!(matches!(
            score
                .annotate(
                    wrong_pitch,
                    GuitarAnnotation::Harmonic {
                        string: 2,
                        harmonic: Harmonic::artificial(Pitch::new(Note::B, 4), 12),
                    },
                )
                .unwrap_err(),
            GuitarScoreError::InvalidHarmonicPitch { .. }
        ));

        let non_node = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 2, 5)
            .unwrap();
        assert!(matches!(
            score
                .annotate(
                    non_node,
                    GuitarAnnotation::Harmonic {
                        string: 2,
                        harmonic: Harmonic::artificial(Pitch::new(Note::G, 5), 8),
                    },
                )
                .unwrap_err(),
            GuitarScoreError::InvalidHarmonicTouch {
                stopped_fret: 5,
                touch_fret: 8
            }
        ));
    }

    #[test]
    fn grouped_chords_and_dead_attacks_keep_every_standard_notehead() {
        let chord = || {
            GuitarEventSpec::chord(
                vec![
                    FrettedPitch::new(Pitch::new(Note::Fis, 4), 1, 2),
                    FrettedPitch::new(Pitch::new(Note::D, 4), 2, 3),
                ],
                Duration::EIGHTH,
            )
        };
        let mut score = GuitarScore::standard();
        let beam = score.beam_group(vec![chord(), chord()]).unwrap();
        score
            .annotate(beam[1], GuitarAnnotation::RhythmicSlash)
            .unwrap()
            .annotate(beam[1], GuitarAnnotation::ChordSymbol(String::from("D")))
            .unwrap()
            .annotate(beam[1], GuitarAnnotation::Strum(StrumDirection::Down))
            .unwrap();
        let tuplet = score
            .tuplet(
                3,
                2,
                vec![
                    GuitarEventSpec::Dead {
                        strings: vec![1, 2, 3],
                        duration: Duration::EIGHTH,
                    },
                    chord(),
                    GuitarEventSpec::Slash {
                        duration: Duration::EIGHTH,
                    },
                ],
            )
            .unwrap();
        score
            .annotate(tuplet[1], GuitarAnnotation::RhythmicSlash)
            .unwrap()
            .annotate(tuplet[1], GuitarAnnotation::ChordSymbol(String::from("D")))
            .unwrap()
            .annotate(tuplet[1], GuitarAnnotation::Strum(StrumDirection::Up))
            .unwrap();
        score.end_barline().unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::BeamGroup(beam_group) = &contents[0].events[0]
        else {
            panic!("expected grouped realized chords in a beam");
        };
        assert_eq!(
            beam_group.notes[0]
                .annotations
                .grouped_chord
                .as_ref()
                .unwrap()
                .staff_positions
                .len(),
            2
        );
        assert!(beam_group.notes[1].annotations.grouped_chord.is_none());
        assert_eq!(
            beam_group.notes[1].annotations.chord_symbol.as_deref(),
            Some("D")
        );

        let crate::layout::system::MeasureEvent::TupletGroup(tuplet_group) = &contents[0].events[1]
        else {
            panic!("expected grouped dead attack in a tuplet");
        };
        let dead = &tuplet_group.beam_group.notes[0];
        assert_eq!(
            dead.annotations
                .grouped_chord
                .as_ref()
                .unwrap()
                .staff_positions
                .len(),
            3
        );
        assert_eq!(dead.annotations.notehead_styles, vec![NoteheadStyle::X; 3]);

        let font = crate::font::bravura_font();
        let x_path = font
            .glyph_outline(NoteheadStyle::X.glyph(3))
            .unwrap()
            .path_data;
        let down_path = font
            .glyph_outline(Glyph::GuitarStrumDown)
            .unwrap()
            .path_data;
        let up_path = font.glyph_outline(Glyph::GuitarStrumUp).unwrap().path_data;
        let svg = score.render_svg();
        assert!(svg.matches(&x_path).count() >= 3);
        assert_eq!(svg.matches(&down_path).count(), 2);
        assert_eq!(svg.matches(&up_path).count(), 2);
    }

    #[test]
    fn barre_rejects_lower_frets_in_overlapping_voices() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(2, 4);
        let barre_chord = || {
            vec![
                FrettedPitch::new(Pitch::new(Note::Fis, 4), 1, 2),
                FrettedPitch::new(Pitch::new(Note::Cis, 4), 2, 2),
            ]
        };
        let start = score.chord(barre_chord(), Duration::QTR).unwrap();
        let end = score.chord(barre_chord(), Duration::QTR).unwrap();
        score.set_voice(1);
        let conflict = score
            .note(Pitch::new(Note::B, 3), Duration::HALF, 2, 0)
            .unwrap();
        score.end_barline().unwrap();

        assert!(matches!(
            score
                .annotation_span(
                    GuitarSpanAnnotation::Barre(
                        GuitarBarre::new(BarreKind::Partial, 2, 1, 2).unwrap(),
                    ),
                    start,
                    end,
                )
                .unwrap_err(),
            GuitarScoreError::BarreFretMismatch { event, .. } if event == conflict.get()
        ));
    }

    #[test]
    fn bend_phase_events_cannot_refret_the_bent_string() {
        let mut arrival_score = GuitarScore::standard();
        let source = arrival_score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let arrival = arrival_score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        assert!(matches!(
            arrival_score
                .bend(BendGesture::new(
                    source,
                    1,
                    BendPitch::exact(Pitch::new(Note::A, 4)),
                    GuitarMoment::onset(arrival),
                ))
                .unwrap_err(),
            GuitarScoreError::BendStringRefret { event, .. } if event == arrival.get()
        ));

        let mut release_start_score = GuitarScore::standard();
        let source = release_start_score
            .note(Pitch::new(Note::G, 4), Duration::HALF, 1, 3)
            .unwrap();
        let release_start = release_start_score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        let release_end = release_start_score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let release = BendRelease::new(
            GuitarMoment::onset(release_start),
            GuitarMoment::onset(release_end),
            BendPitch::exact(Pitch::new(Note::G, 4)),
        );
        assert!(matches!(
            release_start_score
                .bend(
                    BendGesture::new(
                        source,
                        1,
                        BendPitch::exact(Pitch::new(Note::A, 4)),
                        GuitarMoment::after(source, Duration::QTR),
                    )
                    .with_release(release),
                )
                .unwrap_err(),
            GuitarScoreError::BendStringRefret { event, .. } if event == release_start.get()
        ));

        let mut release_end_score = GuitarScore::standard();
        let source = release_end_score
            .note(Pitch::new(Note::G, 4), Duration::HALF, 1, 3)
            .unwrap();
        let release_start = release_end_score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let release_end = release_end_score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        let release = BendRelease::new(
            GuitarMoment::onset(release_start),
            GuitarMoment::onset(release_end),
            BendPitch::exact(Pitch::new(Note::G, 4)),
        );
        assert!(matches!(
            release_end_score
                .bend(
                    BendGesture::new(
                        source,
                        1,
                        BendPitch::exact(Pitch::new(Note::A, 4)),
                        GuitarMoment::after(source, Duration::QTR),
                    )
                    .with_release(release),
                )
                .unwrap_err(),
            GuitarScoreError::BendStringRefret { event, .. } if event == release_end.get()
        ));
    }

    #[test]
    fn unpitched_staff_positions_ignore_clef_and_distinguish_extended_strings() {
        fn positions(clef: Clef) -> (Vec<i8>, i8, i8) {
            let tuning = GuitarTuning::new(vec![
                Pitch::new(Note::E, 4),
                Pitch::new(Note::B, 3),
                Pitch::new(Note::G, 3),
                Pitch::new(Note::D, 3),
                Pitch::new(Note::A, 2),
                Pitch::new(Note::E, 2),
                Pitch::new(Note::B, 1),
            ])
            .unwrap();
            let mut score = GuitarScore::new(tuning);
            score.set_clef(clef);
            score.dead(vec![1, 6, 7], Duration::QTR).unwrap();
            score.slash(Duration::QTR);
            score.percussion(PercussionTarget::Body, Duration::QTR);
            score.end_barline().unwrap();
            let contents = score.notation_builder().build_measure_contents().unwrap();
            let crate::layout::system::MeasureEvent::Chord(dead) = &contents[0].events[0] else {
                panic!("multi-string dead attack must remain a standard chord");
            };
            let crate::layout::system::MeasureEvent::Note(slash) = &contents[0].events[1] else {
                panic!("slash must remain one standard note");
            };
            let crate::layout::system::MeasureEvent::Note(percussion) = &contents[0].events[2]
            else {
                panic!("percussion must remain one standard note");
            };
            (
                dead.staff_positions.clone(),
                slash.staff_position,
                percussion.staff_position,
            )
        }

        let treble = positions(Clef::Treble);
        assert_eq!(treble, positions(Clef::Treble8ba));
        assert_eq!(treble, positions(Clef::Bass));
        assert_eq!(treble, positions(Clef::Alto));
        assert_eq!(treble, positions(Clef::Tenor));
        assert_eq!(treble.0, vec![6, 1, 0]);
        assert_eq!(treble.1, 4);
        assert_eq!(treble.2, 4);
    }

    #[test]
    fn unpitched_heads_neither_show_nor_track_key_signature_accidentals() {
        let tuning = GuitarTuning::new(vec![
            Pitch::new(Note::E, 4),
            Pitch::new(Note::B, 3),
            Pitch::new(Note::G, 3),
            Pitch::new(Note::D, 3),
            Pitch::new(Note::A, 2),
            Pitch::new(Note::E, 2),
            Pitch::new(Note::B, 1),
        ])
        .unwrap();
        let mut score = GuitarScore::new(tuning);
        score.set_key_signature(KeySignature::Sharps(1));
        score.dead(vec![6], Duration::QTR).unwrap();
        score.dead(vec![6, 7], Duration::QTR).unwrap();
        score
            .beam_group(vec![
                GuitarEventSpec::Dead {
                    strings: vec![6, 7],
                    duration: Duration::EIGHTH,
                },
                GuitarEventSpec::Percussion {
                    target: PercussionTarget::Body,
                    duration: Duration::EIGHTH,
                },
            ])
            .unwrap();
        score
            .note(Pitch::new(Note::Fis, 4), Duration::QTR, 1, 2)
            .unwrap();
        score.end_barline().unwrap();

        let contents = score.notation_builder().build_measure_contents().unwrap();
        let crate::layout::system::MeasureEvent::Note(dead_note) = &contents[0].events[0] else {
            panic!("single-string dead attack projects as one note");
        };
        assert!(dead_note.accidental.is_none());
        let crate::layout::system::MeasureEvent::Chord(dead_chord) = &contents[0].events[1] else {
            panic!("multi-string dead attack projects as a chord");
        };
        assert!(dead_chord.accidentals.iter().all(Option::is_none));
        let crate::layout::system::MeasureEvent::BeamGroup(group) = &contents[0].events[2] else {
            panic!("grouped unpitched attacks remain a beam");
        };
        assert!(group.notes.iter().all(|note| {
            note.accidental.is_none()
                && note
                    .annotations
                    .grouped_chord
                    .as_ref()
                    .is_none_or(|chord| chord.accidentals.iter().all(Option::is_none))
        }));
        let crate::layout::system::MeasureEvent::Note(f_sharp) = &contents[0].events[3] else {
            panic!("pitched event follows unpitched projections");
        };
        assert!(
            f_sharp.accidental.is_none(),
            "unpitched F-position heads must not force a redundant sharp"
        );

        let mut flat_key = GuitarScore::standard();
        flat_key.set_key_signature(KeySignature::Flats(1));
        flat_key.slash(Duration::QTR);
        flat_key.percussion(PercussionTarget::Body, Duration::QTR);
        flat_key
            .beam_group(vec![
                GuitarEventSpec::Slash {
                    duration: Duration::EIGHTH,
                },
                GuitarEventSpec::Percussion {
                    target: PercussionTarget::Strings,
                    duration: Duration::EIGHTH,
                },
            ])
            .unwrap();
        flat_key
            .note(Pitch::new(Note::B, 4), Duration::QTR, 1, 7)
            .unwrap();
        flat_key.end_barline().unwrap();
        let flat_contents = flat_key
            .notation_builder()
            .build_measure_contents()
            .unwrap();
        for event in &flat_contents[0].events[..3] {
            match event {
                crate::layout::system::MeasureEvent::Note(note) => {
                    assert!(note.accidental.is_none())
                }
                crate::layout::system::MeasureEvent::BeamGroup(group) => {
                    assert!(group.notes.iter().all(|note| note.accidental.is_none()))
                }
                _ => panic!("expected only unpitched note and beam projections"),
            }
        }
        let crate::layout::system::MeasureEvent::Note(b_natural) = &flat_contents[0].events[3]
        else {
            panic!("pitched B-natural follows unpitched B-position heads");
        };
        assert_eq!(
            b_natural.accidental,
            Some(crate::layout::accidental::ResolvedAccidental::plain(
                Glyph::AccidentalNatural
            ))
        );
    }

    #[test]
    fn vibrato_requires_an_event_with_physical_strings() {
        let mut score = GuitarScore::standard();
        let rest = score.rest(Duration::QTR);
        let slash = score.slash(Duration::QTR);
        let percussion = score.percussion(PercussionTarget::Body, Duration::QTR);
        for event in [rest, slash, percussion] {
            assert!(matches!(
                score
                    .annotate(event, GuitarAnnotation::Vibrato(VibratoKind::Normal))
                    .unwrap_err(),
                GuitarScoreError::InvalidAnnotationTarget {
                    annotation: "vibrato",
                    ..
                }
            ));
        }
        let dead = score.dead(vec![1, 2], Duration::QTR).unwrap();
        score
            .annotate(dead, GuitarAnnotation::Vibrato(VibratoKind::Normal))
            .unwrap();
    }

    #[test]
    fn annotation_lanes_reuse_non_overlapping_intervals_and_reserve_per_system() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        let first = score
            .note(Pitch::new(Note::E, 4), Duration::QTR, 1, 0)
            .unwrap();
        let second = score
            .note(Pitch::new(Note::Fis, 4), Duration::QTR, 1, 2)
            .unwrap();
        let third = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        let fourth = score
            .note(Pitch::new(Note::A, 4), Duration::QTR, 1, 5)
            .unwrap();
        score.barline().unwrap();
        score
            .note(Pitch::new(Note::E, 4), Duration::WHOLE, 1, 0)
            .unwrap();
        score.end_barline().unwrap();
        score
            .annotation_span(
                GuitarSpanAnnotation::Position(PositionLabel {
                    fret: 1,
                    shift: None,
                }),
                first,
                second,
            )
            .unwrap();
        score
            .annotation_span(
                GuitarSpanAnnotation::Position(PositionLabel {
                    fret: 2,
                    shift: None,
                }),
                third,
                fourth,
            )
            .unwrap();
        score
            .annotation_span(
                GuitarSpanAnnotation::CellBracket {
                    label: String::from("overlap"),
                    enclosure: None,
                },
                second,
                fourth,
            )
            .unwrap();
        score
            .annotate(
                first,
                GuitarAnnotation::Attack {
                    string: Some(1),
                    source: AttackSource::Pick(PickStroke::Down),
                },
            )
            .unwrap()
            .show_technique_legend();

        let layout = score.annotation_layout(&[(0, 1), (1, 2)]);
        assert_eq!(layout.span_lanes, vec![Some(0), Some(0), Some(1)]);
        assert_eq!(layout.system_height_ss(1), 0.0);
        let legend_y = layout.legend_y_ss().unwrap();
        assert!(legend_y > 5.0);
        assert!(layout.system_height_ss(0) > legend_y);
        let font = crate::font::bravura_font();
        let staff_space = font.units_per_em() as f64 / 4.0;
        for glyph in [Glyph::GuitarBarreFull, Glyph::GuitarBarreHalf] {
            let height_ss = font
                .glyph_bbox_design_units(glyph)
                .expect("barre glyph has SMuFL bounds")
                .height()
                / staff_space;
            assert!(
                0.5 + height_ss + 0.2 <= GUITAR_ANNOTATION_LANE_PITCH_SS,
                "barre lane pitch must clear the glyph's half-space lift and full height"
            );
        }
        assert!(
            legend_y > GUITAR_ANNOTATION_FIRST_LANE_HEIGHT_SS + GUITAR_ANNOTATION_LANE_PITCH_SS,
            "legend must begin below both occupied annotation lanes"
        );
    }

    #[test]
    fn legend_uses_strum_glyphs_without_conflating_pick_strokes() {
        let mut score = GuitarScore::standard();
        score.set_time_signature(4, 4);
        let down = score.slash(Duration::QTR);
        let up = score.slash(Duration::QTR);
        let pick = score
            .note(Pitch::new(Note::G, 4), Duration::QTR, 1, 3)
            .unwrap();
        score.rest(Duration::QTR);
        score
            .annotate(down, GuitarAnnotation::Strum(StrumDirection::Down))
            .unwrap()
            .annotate(up, GuitarAnnotation::Strum(StrumDirection::Up))
            .unwrap()
            .annotate(
                pick,
                GuitarAnnotation::Attack {
                    string: Some(1),
                    source: AttackSource::Pick(PickStroke::Down),
                },
            )
            .unwrap()
            .show_technique_legend()
            .end_barline()
            .unwrap();

        let font = crate::font::bravura_font();
        let down_path = font
            .glyph_outline(Glyph::GuitarStrumDown)
            .unwrap()
            .path_data;
        let up_path = font.glyph_outline(Glyph::GuitarStrumUp).unwrap().path_data;
        let staff_space = font.units_per_em() as f64 / 4.0;
        for glyph in [Glyph::GuitarStrumDown, Glyph::GuitarStrumUp] {
            let scaled_height_ss = font
                .glyph_bbox_design_units(glyph)
                .expect("strum glyph has SMuFL bounds")
                .height()
                / staff_space
                * GUITAR_LEGEND_STRUM_SCALE;
            assert!(
                scaled_height_ss <= GUITAR_LEGEND_ROW_HEIGHT_SS,
                "scaled strum glyph must fit inside one legend row"
            );
        }
        let svg = score.render_svg();
        assert_eq!(svg.matches(&down_path).count(), 3);
        assert_eq!(svg.matches(&up_path).count(), 3);
        assert!(svg.contains(">↓  down-pick</text>"));
        assert!(svg.contains(">down strum</text>"));
        assert!(svg.contains(">up strum</text>"));
        assert!(!svg.contains(">D  down strum</text>"));
        assert!(!svg.contains(">U  up strum</text>"));
        assert_eq!(svg.matches("data-guitar-legend-scale=\"0.3\"").count(), 2);
        assert!(svg.contains("data-guitar-legend-strum=\"down\""));
        assert!(svg.contains("data-guitar-legend-strum=\"up\""));
    }

    #[test]
    fn standard_technique_labels_clear_high_noteheads_and_chord_symbols() {
        let mut score = GuitarScore::standard();
        let event = score
            .note(Pitch::new(Note::E, 6), Duration::QTR, 1, 24)
            .unwrap();
        score
            .annotate(
                event,
                GuitarAnnotation::Harmonic {
                    string: 1,
                    harmonic: Harmonic::natural(Pitch::new(Note::E, 6)),
                },
            )
            .unwrap()
            .annotate(event, GuitarAnnotation::ChordSymbol(String::from("E")))
            .unwrap();
        let staff = StaffLayout::new(0.0, 0.0, 10_000.0, 1_000.0);
        let label_y = standard_event_label_y(&score, &staff, score.event(event).unwrap());
        let written = written_pitch(Pitch::new(Note::E, 6), score.clef);
        let head_y = staff.y_of(pitch_to_staff_position(&written, &score.clef.to_clef()));
        assert!(label_y <= head_y - 1.4 * staff.staff_space);
        assert!(label_y <= staff.y_origin - 5.2 * staff.staff_space);
    }
}
