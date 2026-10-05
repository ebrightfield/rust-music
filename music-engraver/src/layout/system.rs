use music::notation::clef::Clef;

use crate::layout::barline::BarlineStyle;
use crate::layout::clef::ClefLayout;
use crate::layout::glyph_metrics::{glyph_advance, glyph_box, GlyphBox};
use crate::layout::group::GroupMark;
use crate::layout::key_signature::{key_signature_layout, KeySignature};
use crate::layout::measure::{
    layout_measure, ChordEvent, MeasureElement, MeasureLayout, MeasureLayoutConfig, NoteEvent,
    RestEvent, SpacerEvent,
};
use crate::layout::measure_meta::MeasureMeta;
use crate::layout::staff::TOP_LINE;
use crate::layout::time_signature::{time_signature_layout, TimeSignatureKind};
use crate::layout::volta::VoltaAnnotation;

/// Input description of a single measure's musical content (no layout yet).
#[derive(Clone, Debug)]
pub struct MeasureContent {
    /// Note and rest events in temporal order (voice 0 / primary voice).
    pub events: Vec<MeasureEvent>,
    /// Barline at the end of this measure.
    pub barline: BarlineStyle,
    /// Optional volta bracket annotation for this measure.
    pub volta: Option<VoltaAnnotation>,
    /// Additional voices beyond the primary. `additional_voices[0]` is voice 1,
    /// `additional_voices[1]` is voice 2, etc. Each inner `Vec` holds that
    /// voice's events in temporal order. Stem directions are forced: voice 0
    /// gets stems up, voice 1 gets stems down. Currently only laid out for
    /// rendering when the measure renderer's multi-voice path is active.
    pub additional_voices: Vec<Vec<MeasureEvent>>,
    /// Logical numbering, meter/length semantics, and break permission.
    pub meta: MeasureMeta,
}

/// An event within a measure: a rhythmic event (note, rest, chord, group,
/// spacer) or a zero-duration structural change (clef, time signature).
///
/// Structural changes travel in the primary voice's `events`, ordered by
/// onset. Changes at the very start of a measure (its *leading changes*) are
/// placed by system layout: absorbed into the system prefix at a system start,
/// otherwise drawn around the preceding barline per [`ClefChangePlacement`].
#[derive(Clone, Debug)]
pub enum MeasureEvent {
    Note(NoteEvent),
    Rest(RestEvent),
    Chord(ChordEvent),
    /// A zero-duration beam or tuplet span boundary; the notes, chords, and
    /// rests between a start and its matching end are the span's members.
    GroupMark(GroupMark),
    /// Multi-measure rest: the entire measure shows either an H-bar or a
    /// church-rest cluster of SMuFL rest glyphs (small counts), with a count
    /// number indicating how many consecutive measures of rest.
    MultiMeasureRest {
        /// Number of consecutive measures of rest.
        count: u32,
        /// Visual style (H-bar or church-rest).
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle,
    },
    /// Invisible rhythmic placeholder (LilyPond spacer `s`).
    Spacer(SpacerEvent),
    /// Clef change (zero duration); every later pitch on the staff is placed
    /// in the new clef.
    ClefChange(ClefChange),
    /// Printed time-signature change (zero duration), at a measure start.
    /// Hidden meter changes have no event; see [`MeasureContent::meta`].
    TimeSignature(TimeSignatureKind),
}

/// Where a clef change that falls on a measure boundary is drawn when the
/// boundary is inside a system. Mid-measure changes are always drawn just
/// before the next event; at a system break the new clef goes in the next
/// system's prefix and a courtesy clef ends the previous system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClefChangePlacement {
    /// Before the barline that ends the preceding measure (LilyPond's
    /// default break-align order).
    #[default]
    BeforeBarline,
    /// After that barline, at the start of the new measure.
    AfterBarline,
}

/// A mid-score clef change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClefChange {
    /// The new clef.
    pub clef: ClefKind,
    /// Placement at a measure boundary.
    pub placement: ClefChangePlacement,
}

impl ClefChange {
    /// The change-size clef glyph layout for this change.
    pub fn clef_layout(&self) -> ClefLayout {
        ClefLayout::change(&self.clef.to_clef())
    }
}

/// Describes the frontmatter (clef, key, time sig) that appears at the start of a system.
///
/// `clef_layout` is the visual layout. The original `Clef` is stored internally
/// (as a `ClefKind`) for key sig accidental placement and note rendering.
#[derive(Clone, Debug)]
pub struct SystemPrefix {
    pub clef_layout: ClefLayout,
    pub clef_kind: ClefKind,
    pub key_signature: KeySignature,
    pub time_signature: Option<TimeSignatureKind>,
}

/// Local mirror of `music::Clef` variants that derives Clone/Copy/Debug.
/// `music::Clef` lacks these derives and we cannot modify it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClefKind {
    Treble,
    Treble8va,
    Treble8ba,
    Bass,
    Alto,
    Tenor,
}

impl ClefKind {
    /// Convert a `&Clef` reference into its `ClefKind` equivalent.
    pub fn from_clef(clef: &Clef) -> Self {
        match clef {
            Clef::Treble => Self::Treble,
            Clef::Treble8va => Self::Treble8va,
            Clef::Treble8ba => Self::Treble8ba,
            Clef::Bass => Self::Bass,
            Clef::Alto => Self::Alto,
            Clef::Tenor => Self::Tenor,
        }
    }

    /// Convert back to a `music::Clef` value.
    pub fn to_clef(self) -> Clef {
        match self {
            Self::Treble => Clef::Treble,
            Self::Treble8va => Clef::Treble8va,
            Self::Treble8ba => Clef::Treble8ba,
            Self::Bass => Clef::Bass,
            Self::Alto => Clef::Alto,
            Self::Tenor => Clef::Tenor,
        }
    }
}

impl SystemPrefix {
    /// Create a prefix from a `Clef`, resolving the layout automatically.
    pub fn new(
        clef: &Clef,
        key_signature: KeySignature,
        time_signature: Option<TimeSignatureKind>,
    ) -> Self {
        Self {
            clef_layout: ClefLayout::from_clef_ref(clef),
            clef_kind: ClefKind::from_clef(clef),
            key_signature,
            time_signature,
        }
    }
}

/// A laid-out system: multiple measures arranged horizontally on one line.
#[derive(Clone, Debug)]
pub struct SystemLayout {
    /// The clef used for this system (for key signature accidental placement, note rendering).
    pub clef_kind: ClefKind,
    /// Laid-out measures with their x-offsets within the system.
    pub measures: Vec<SystemMeasure>,
    /// Total width of the system in font design units.
    pub total_width: f64,
    /// Staff width (may equal total_width, or may be fixed).
    pub staff_width: f64,
}

/// Vertical ink extent of the clefs, key signatures, and time signatures
/// among `elements` on a staff whose key signature is placed in `clef`, as
/// `(top, bottom)` in staff spaces below the top staff line (negative =
/// above it). `None` when there are none.
fn glyph_vertical_extent_ss<'a>(
    elements: impl IntoIterator<Item = &'a MeasureElement>,
    clef: ClefKind,
) -> Option<(f64, f64)> {
    let below_top = |position: i8| f64::from(TOP_LINE - position) / 2.0;
    let mut extent: Option<(f64, f64)> = None;
    let mut include = |position: i8, ink: GlyphBox| {
        let origin = below_top(position);
        let (top, bottom) = (origin + ink.y_top, origin + ink.y_bottom);
        extent = Some(match extent {
            None => (top, bottom),
            Some((t, b)) => (t.min(top), b.max(bottom)),
        });
    };
    let clef = clef.to_clef();
    for element in elements {
        match element {
            MeasureElement::Clef(clef_layout) => {
                include(clef_layout.staff_position, clef_layout.ink_box());
            }
            MeasureElement::KeySignature(key) => {
                let layout = key_signature_layout(key, &clef, glyph_advance, 1.0);
                for accidental in &layout.accidentals {
                    if let Some(ink) = glyph_box(accidental.glyph) {
                        include(accidental.staff_position, ink);
                    }
                }
            }
            MeasureElement::TimeSignature(kind) => {
                for (glyph, position, _) in time_signature_layout(kind, glyph_advance).glyphs {
                    if let Some(ink) = glyph_box(glyph) {
                        include(position, ink);
                    }
                }
            }
            _ => {}
        }
    }
    extent
}

impl SystemLayout {
    /// Vertical ink extent of the system's clefs, key signatures, and time
    /// signatures (prefix and inline), as `(top, bottom)` in staff spaces
    /// below the top staff line (negative = above it). `None` when the system
    /// has none. Used to size page bounds so tall prefix glyphs (a G clef's
    /// top, a tenor clef, a high key-signature sharp) are not clipped.
    pub fn glyph_vertical_extent_ss(&self) -> Option<(f64, f64)> {
        glyph_vertical_extent_ss(
            self.measures
                .iter()
                .flat_map(|measure| measure.layout.elements.iter())
                .map(|positioned| &positioned.element),
            self.clef_kind,
        )
    }
}

/// Vertical ink extent, as in [`SystemLayout::glyph_vertical_extent_ss`], of
/// every prefix a staff can show: its initial prefix and, for each clef it
/// changes to, that clef at full and change size with the key signature
/// placed in it. Lets callers size bounds before laying out systems.
pub fn staff_prefix_glyph_extent_ss(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
) -> Option<(f64, f64)> {
    let mut clefs = vec![prefix.clef_kind];
    for measure in measures {
        for event in &measure.events {
            if let MeasureEvent::ClefChange(change) = event {
                clefs.push(change.clef);
            }
        }
    }
    clefs
        .into_iter()
        .filter_map(|clef| {
            let mut elements = vec![
                MeasureElement::Clef(ClefLayout::from_clef_ref(&clef.to_clef())),
                MeasureElement::Clef(ClefLayout::change(&clef.to_clef())),
                MeasureElement::KeySignature(prefix.key_signature.clone()),
            ];
            elements.extend(
                prefix
                    .time_signature
                    .clone()
                    .map(MeasureElement::TimeSignature),
            );
            glyph_vertical_extent_ss(&elements, clef)
        })
        .reduce(|(t1, b1), (t2, b2)| (t1.min(t2), b1.max(b2)))
}

/// A single measure within a laid-out system, with its horizontal offset.
#[derive(Clone, Debug)]
pub struct SystemMeasure {
    /// X-offset of this measure's left edge within the system.
    pub x_offset: f64,
    /// The laid-out measure (voice 0 / primary voice).
    pub layout: MeasureLayout,
    /// Optional volta bracket annotation for this measure.
    pub volta: Option<VoltaAnnotation>,
    /// Laid-out additional voices (voice 1, voice 2, …).
    /// Each entry shares the same temporal x-positions as the primary voice
    /// but may have different notes/rests with forced stem directions.
    pub additional_voice_layouts: Vec<MeasureLayout>,
    /// The measure's metadata (logical number, meter, lengths).
    pub meta: MeasureMeta,
}

/// The leading changes of a measure: the structural events at its very
/// start, before any rhythmic event.
pub(crate) fn leading_changes(content: &MeasureContent) -> &[MeasureEvent] {
    let count = content
        .events
        .iter()
        .take_while(|event| {
            matches!(
                event,
                MeasureEvent::ClefChange(_) | MeasureEvent::TimeSignature(_)
            )
        })
        .count();
    &content.events[..count]
}

/// The prefix of a system that starts at `measures[start]`, given the
/// score's `initial` prefix (the state before the first measure).
///
/// The clef is the one in force at that point, including the start
/// measure's leading clef changes; the key signature is the initial one. The
/// time signature is shown on the first system (the initial meter, unless the
/// first measure changes it) and on a later system only when its first
/// measure begins with a printed meter change.
pub fn system_start_prefix(
    initial: &SystemPrefix,
    measures: &[MeasureContent],
    start: usize,
) -> SystemPrefix {
    let mut clef = initial.clef_kind;
    for measure in &measures[..start.min(measures.len())] {
        for event in &measure.events {
            if let MeasureEvent::ClefChange(change) = event {
                clef = change.clef;
            }
        }
    }
    let mut time_signature = if start == 0 {
        initial.time_signature.clone()
    } else {
        None
    };
    if let Some(measure) = measures.get(start) {
        for event in leading_changes(measure) {
            match event {
                MeasureEvent::ClefChange(change) => clef = change.clef,
                MeasureEvent::TimeSignature(kind) => time_signature = Some(kind.clone()),
                _ => {}
            }
        }
    }
    SystemPrefix::new(
        &clef.to_clef(),
        initial.key_signature.clone(),
        time_signature,
    )
}

/// Primary-voice elements of each measure of one system.
///
/// The first measure starts with the prefix, which already reflects its
/// leading changes. A later measure's leading clef changes are drawn before
/// the preceding barline ([`ClefChangePlacement::BeforeBarline`]) or after it
/// ([`ClefChangePlacement::AfterBarline`]), followed by its printed meter
/// change. When `next` (the first measure of the following system) begins
/// with changes, courtesy elements end this system: clefs on their side of
/// the final barline, a meter change after it.
fn system_measure_elements(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    next: Option<&MeasureContent>,
) -> Vec<Vec<MeasureElement>> {
    let change_clefs = |changes: &[MeasureEvent], placement: ClefChangePlacement| {
        changes
            .iter()
            .filter_map(move |event| match event {
                MeasureEvent::ClefChange(change) if change.placement == placement => {
                    Some(MeasureElement::Clef(change.clef_layout()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let meter_changes = |changes: &[MeasureEvent]| {
        changes
            .iter()
            .filter_map(|event| match event {
                MeasureEvent::TimeSignature(kind) => {
                    Some(MeasureElement::TimeSignature(kind.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let last = measures.len().saturating_sub(1);
    measures
        .iter()
        .enumerate()
        .map(|(i, measure)| {
            let leading = leading_changes(measure);
            let mut elems = Vec::new();
            if i == 0 {
                elems.push(MeasureElement::Clef(prefix.clef_layout.clone()));
                if !matches!(prefix.key_signature, KeySignature::Open) {
                    elems.push(MeasureElement::KeySignature(prefix.key_signature.clone()));
                }
                if let Some(ts) = &prefix.time_signature {
                    elems.push(MeasureElement::TimeSignature(ts.clone()));
                }
            } else {
                elems.extend(change_clefs(leading, ClefChangePlacement::AfterBarline));
                elems.extend(meter_changes(leading));
            }
            elems.extend(
                measure.events[leading.len()..]
                    .iter()
                    .map(measure_event_to_element),
            );
            let following = if i < last { measures.get(i + 1) } else { next };
            let following_changes = following.map_or(&[][..], leading_changes);
            elems.extend(change_clefs(
                following_changes,
                ClefChangePlacement::BeforeBarline,
            ));
            elems.push(MeasureElement::Barline(measure.barline));
            if i == last {
                elems.extend(change_clefs(
                    following_changes,
                    ClefChangePlacement::AfterBarline,
                ));
                elems.extend(meter_changes(following_changes));
            }
            elems
        })
        .collect()
}

/// Lay out a system of measures.
///
/// The first measure gets the system prefix (clef, key sig, optional time
/// sig); `prefix` must describe the state at the system start, leading
/// changes of the first measure included (see [`system_start_prefix`]).
/// Subsequent measures get their rhythmic content, structural changes, and
/// barlines. Measures are arranged left-to-right with no gap between them.
///
/// If `target_width` is `Some(w)`, the layout will scale note spacing so the
/// system fills exactly that width. If `None`, measures use natural widths.
pub fn layout_system(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    config: &MeasureLayoutConfig,
    target_width: Option<f64>,
) -> SystemLayout {
    layout_system_followed_by(prefix, measures, None, config, target_width)
}

/// [`layout_system`] for a system followed by another whose first measure is
/// `next`: changes at the start of `next` add courtesy elements at the end of
/// this system.
pub fn layout_system_followed_by(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    next: Option<&MeasureContent>,
    config: &MeasureLayoutConfig,
    target_width: Option<f64>,
) -> SystemLayout {
    if measures.is_empty() {
        return SystemLayout {
            clef_kind: prefix.clef_kind,
            measures: vec![],
            total_width: 0.0,
            staff_width: 0.0,
        };
    }

    // First pass: lay out primary voice for each measure at natural width
    let measure_elements = system_measure_elements(prefix, measures, next);

    // Lay out primary voice for each measure
    let mut layouts: Vec<MeasureLayout> = measure_elements
        .iter()
        .map(|elems| layout_measure(elems, config))
        .collect();

    // Lay out additional voices for each measure. Each additional voice is
    // laid out independently, then scaled to match the primary voice's width
    // so that temporal positions align visually.
    let mut additional_voice_layouts: Vec<Vec<MeasureLayout>> = Vec::with_capacity(measures.len());
    for (i, measure) in measures.iter().enumerate() {
        let mut voice_layouts = Vec::new();
        for voice_events in &measure.additional_voices {
            let mut elems: Vec<MeasureElement> =
                voice_events.iter().map(measure_event_to_element).collect();
            // Additional voices share the barline with the primary voice
            elems.push(MeasureElement::Barline(measure.barline));
            let mut voice_layout = layout_measure(&elems, config);
            // Start each additional voice at the primary voice's first
            // rhythmic event: past the system prefix in the first measure,
            // and past the primary's leading accidental reservation anywhere.
            let first_rhythmic_x = |layout: &MeasureLayout| {
                layout
                    .elements
                    .iter()
                    .find(|element| {
                        matches!(
                            element.element,
                            MeasureElement::Note(_)
                                | MeasureElement::Rest(_)
                                | MeasureElement::Spacer(_)
                                | MeasureElement::Chord(_)
                                | MeasureElement::MultiMeasureRest { .. }
                        )
                    })
                    .map(|element| element.x)
            };
            let primary_first = first_rhythmic_x(&layouts[i]);
            if let (Some(primary_first), Some(voice_first)) =
                (primary_first, first_rhythmic_x(&voice_layout))
            {
                let leading = primary_first - voice_first;
                for element in &mut voice_layout.elements {
                    element.x += leading;
                }
                voice_layout.total_rod += leading;
                voice_layout.total_width += leading;
            }
            // Spring-only scale to match the primary voice's width, so temporal
            // positions align at measure ends without compressing this voice's
            // rods (Gourlay). (True max-spring-per-tick cross-voice merging is
            // deferred — see docs/implementation-plan-gourlay-spacing.md §4.)
            let primary_width = layouts[i].total_width;
            if voice_layout.total_width > 0.0 && primary_width > 0.0 {
                let s = spring_scale(
                    voice_layout.total_rod,
                    voice_layout.total_spring,
                    primary_width,
                );
                scale_measure_springs(&mut voice_layout, s);
            }
            voice_layouts.push(voice_layout);
        }
        additional_voice_layouts.push(voice_layouts);
    }

    let natural_width: f64 = layouts.iter().map(|l| l.total_width).sum();

    // If a target width is specified, fit the system with Gourlay's one-line
    // analytic solve: find the single spring scale `s` such that
    // `Σ(rod_i + s·spring_i) == target` across all measures, then apply it.
    // Rods (noteheads, accidentals, clef/key/time prefix, barlines) stay
    // fixed; only springs compress or extend. This replaces the previous
    // uniform scale, which wrongly compressed the incompressible prefix and
    // noteheads along with the springs.
    if let Some(target) = target_width {
        if natural_width > 0.0 {
            let total_rod: f64 = layouts.iter().map(|l| l.total_rod).sum();
            let total_spring: f64 = layouts.iter().map(|l| l.total_spring).sum();
            let s = spring_scale(total_rod, total_spring, target);
            for layout in &mut layouts {
                scale_measure_springs(layout, s);
            }
            // Re-fit each additional voice to its (now scaled) primary measure
            // width using the same spring-only rule, preserving alignment.
            for (i, voice_layouts) in additional_voice_layouts.iter_mut().enumerate() {
                let primary_width = layouts[i].total_width;
                for vl in voice_layouts.iter_mut() {
                    if vl.total_width > 0.0 && primary_width > 0.0 {
                        let vs = spring_scale(vl.total_rod, vl.total_spring, primary_width);
                        scale_measure_springs(vl, vs);
                    }
                }
            }
        }
    }

    // Assign x-offsets
    let mut system_measures = Vec::with_capacity(layouts.len());
    let mut x = 0.0;
    for (i, layout) in layouts.iter().enumerate() {
        system_measures.push(SystemMeasure {
            x_offset: x,
            layout: layout.clone(),
            volta: measures[i].volta.clone(),
            additional_voice_layouts: additional_voice_layouts[i].clone(),
            meta: measures[i].meta.clone(),
        });
        x += layout.total_width;
    }

    let total = x;
    let staff_w = target_width.unwrap_or(total);

    SystemLayout {
        clef_kind: prefix.clef_kind,
        measures: system_measures,
        total_width: total,
        staff_width: staff_w,
    }
}

/// Spring scale floor. When a system's incompressible rod total already meets
/// or exceeds the target width, springs cannot shrink further without
/// producing negative or zero element widths; we clamp the scale here and let
/// the system overflow the target. (A line-breaker — out of scope — is the
/// real fix for content that genuinely does not fit; this clamp just keeps
/// geometry well-formed.)
const MIN_SPRING_SCALE: f64 = 0.0;

/// Solve the Gourlay one-line compression/extension scale for a system.
///
/// Returns the spring scale `s` such that `Σ(rod_i + s·spring_i) == target`.
/// Rods are incompressible; only springs scale. When there is no spring to
/// scale (all rod), or the solve would drive springs below the floor, the
/// result is clamped — the system then keeps its natural rod-bound width and
/// may overflow `target`.
fn spring_scale(total_rod: f64, total_spring: f64, target: f64) -> f64 {
    if total_spring <= 0.0 {
        return 1.0;
    }
    ((target - total_rod) / total_spring).max(MIN_SPRING_SCALE)
}

/// Apply a spring scale to one measure layout in place: each element's spring
/// scales by `s`, its rod stays fixed, `width = rod + s·spring`, and x-offsets
/// are re-flowed by accumulation while preserving the original inter-element
/// gaps (trailing prefix padding, which is incompressible rod).
fn scale_measure_springs(layout: &mut MeasureLayout, s: f64) {
    // Capture the original gaps before mutating any widths/positions. The gap
    // after element i is everything between its right edge and the next
    // element's left edge — i.e. trailing padding emitted by `layout_measure`.
    let gaps: Vec<f64> = layout
        .elements
        .windows(2)
        .map(|w| w[1].x - (w[0].x + w[0].width))
        .collect();

    let mut x = layout.elements.first().map(|e| e.x).unwrap_or(0.0);
    for (i, el) in layout.elements.iter_mut().enumerate() {
        el.spring *= s;
        el.width = el.rod + el.spring;
        el.x = x;
        x += el.width;
        if let Some(gap) = gaps.get(i) {
            x += gap;
        }
    }
    layout.total_spring *= s;
    layout.total_width = layout.total_rod + layout.total_spring;
}

pub(crate) fn measure_event_to_element(event: &MeasureEvent) -> MeasureElement {
    match event {
        MeasureEvent::Note(n) => MeasureElement::Note(n.clone()),
        MeasureEvent::Rest(r) => MeasureElement::Rest(r.clone()),
        MeasureEvent::Chord(c) => MeasureElement::Chord(c.clone()),
        MeasureEvent::GroupMark(mark) => MeasureElement::GroupMark(*mark),
        MeasureEvent::MultiMeasureRest { count, style } => MeasureElement::MultiMeasureRest {
            count: *count,
            style: *style,
        },
        MeasureEvent::Spacer(spacer) => MeasureElement::Spacer(*spacer),
        MeasureEvent::ClefChange(change) => MeasureElement::Clef(change.clef_layout()),
        MeasureEvent::TimeSignature(kind) => MeasureElement::TimeSignature(kind.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::measure::NoteAnnotations;
    fn test_config() -> MeasureLayoutConfig {
        MeasureLayoutConfig::from_staff_space(250.0)
    }

    fn test_prefix() -> SystemPrefix {
        SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Open,
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        )
    }

    fn quarter_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        })
    }

    fn quarter_rest() -> MeasureEvent {
        MeasureEvent::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
        })
    }

    #[test]
    fn empty_measures_produce_empty_system() {
        let layout = layout_system(&test_prefix(), &[], &test_config(), None);
        assert!(layout.measures.is_empty());
        assert!((layout.total_width).abs() < f64::EPSILON);
    }

    #[test]
    fn single_measure_has_prefix_elements() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Sharps(2),
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        );
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);

        assert_eq!(layout.measures.len(), 1);
        assert!(layout.total_width > 0.0);

        // First measure should contain: clef, key sig, time sig, note, barline = 5 elements
        let elems = &layout.measures[0].layout.elements;
        assert_eq!(elems.len(), 5);
        assert!(matches!(elems[0].element, MeasureElement::Clef(_)));
        assert!(matches!(elems[1].element, MeasureElement::KeySignature(_)));
        assert!(matches!(elems[2].element, MeasureElement::TimeSignature(_)));
        assert!(matches!(elems[3].element, MeasureElement::Note(_)));
        assert!(matches!(elems[4].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn second_measure_has_no_prefix() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        assert_eq!(layout.measures.len(), 2);

        // Second measure: only note + barline = 2 elements
        let elems = &layout.measures[1].layout.elements;
        assert_eq!(elems.len(), 2);
        assert!(matches!(elems[0].element, MeasureElement::Note(_)));
        assert!(matches!(elems[1].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn measures_are_contiguous() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(8)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        // Each measure starts where the previous ended
        for i in 1..layout.measures.len() {
            let prev_end =
                layout.measures[i - 1].x_offset + layout.measures[i - 1].layout.total_width;
            let curr_start = layout.measures[i].x_offset;
            assert!(
                (prev_end - curr_start).abs() < 1e-6,
                "measure {} should start at {}, got {}",
                i,
                prev_end,
                curr_start,
            );
        }
    }

    #[test]
    fn total_width_equals_sum_of_measures() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4), quarter_note(6)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(2)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        let sum: f64 = layout.measures.iter().map(|m| m.layout.total_width).sum();
        assert!(
            (layout.total_width - sum).abs() < 1e-6,
            "total {} != sum {}",
            layout.total_width,
            sum,
        );
    }

    #[test]
    fn target_width_scales_system() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
        ];

        let natural = layout_system(&test_prefix(), &measures, &cfg, None);
        let target = 10000.0;
        let scaled = layout_system(&test_prefix(), &measures, &cfg, Some(target));

        // Scaled total should match target
        assert!(
            (scaled.total_width - target).abs() < 1.0,
            "scaled width {} should be close to target {}",
            scaled.total_width,
            target,
        );

        // Scaled should differ from natural unless they happen to match
        assert!(
            (natural.total_width - target).abs() > 1.0,
            "natural width should differ from target for this test to be meaningful"
        );
    }

    #[test]
    fn stretching_fixes_rods_and_grows_only_springs() {
        // Gourlay: under target-width stretch, incompressible rods (clef, key
        // sig, time sig, noteheads, barline) keep their natural width; only
        // springs absorb the extra space. Contrast with the old uniform scale,
        // which would have grown the clef and noteheads too.
        let cfg = test_config();
        let prefix = SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Sharps(2),
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        );
        let measures = vec![MeasureContent {
            events: vec![
                quarter_note(4),
                quarter_note(6),
                quarter_note(8),
                quarter_note(4),
            ],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];

        let natural = layout_system(&prefix, &measures, &cfg, None);
        let target = natural.total_width * 1.5; // stretch
        let scaled = layout_system(&prefix, &measures, &cfg, Some(target));

        assert!((scaled.total_width - target).abs() < 1.0);

        let nat_elems = &natural.measures[0].layout.elements;
        let scl_elems = &scaled.measures[0].layout.elements;
        assert_eq!(nat_elems.len(), scl_elems.len());

        for (n, s) in nat_elems.iter().zip(scl_elems.iter()) {
            // Rod is invariant under stretch for every element.
            assert!(
                (n.rod - s.rod).abs() < 1e-6,
                "rod must not change under stretch: {} -> {}",
                n.rod,
                s.rod,
            );
            match &n.element {
                // Prefix + barline are pure rod → width unchanged entirely.
                MeasureElement::Clef(_)
                | MeasureElement::KeySignature(_)
                | MeasureElement::TimeSignature(_)
                | MeasureElement::Barline(_) => {
                    assert!(
                        (n.width - s.width).abs() < 1e-6,
                        "pure-rod element width must not change under stretch",
                    );
                    assert!(s.spring.abs() < 1e-9);
                }
                // Notes carry springs → spring grows, width grows by exactly
                // the spring delta (rod fixed).
                MeasureElement::Note(_) => {
                    assert!(s.spring > n.spring, "note spring should grow under stretch");
                    assert!(
                        ((s.width - n.width) - (s.spring - n.spring)).abs() < 1e-6,
                        "note width delta must equal spring delta",
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn compressing_below_rod_total_clamps_without_negative_widths() {
        // A target smaller than the system's rod total cannot be met by
        // shrinking springs alone; springs clamp at the floor and the system
        // overflows rather than producing negative element widths.
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4), quarter_note(6), quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let scaled = layout_system(&test_prefix(), &measures, &cfg, Some(1.0));
        for el in &scaled.measures[0].layout.elements {
            assert!(el.width >= -1e-9, "no negative element widths");
            assert!(el.spring >= -1e-9, "no negative springs");
        }
        // total_width is bounded below by the rod total (springs floored at 0).
        let total_rod: f64 = scaled.measures[0].layout.total_rod;
        assert!(scaled.total_width + 1e-6 >= total_rod);
    }

    #[test]
    fn first_measure_x_offset_is_zero() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);
        assert!((layout.measures[0].x_offset).abs() < f64::EPSILON);
    }

    #[test]
    fn open_key_signature_omitted_from_elements() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(&Clef::Treble, KeySignature::Open, None);
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);
        let elems = &layout.measures[0].layout.elements;

        // Should have: clef, note, barline = 3 (no key sig, no time sig)
        assert_eq!(elems.len(), 3);
        assert!(matches!(elems[0].element, MeasureElement::Clef(_)));
        assert!(matches!(elems[1].element, MeasureElement::Note(_)));
        assert!(matches!(elems[2].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn clef_is_preserved_in_layout() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(&Clef::Bass, KeySignature::Open, None);
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);
        assert_eq!(layout.clef_kind, ClefKind::Bass);
    }

    #[test]
    fn multiple_events_in_measure() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![
                quarter_note(0),
                quarter_note(4),
                quarter_rest(),
                quarter_note(8),
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        // clef + time sig + 4 events + barline = 7
        let elems = &layout.measures[0].layout.elements;
        assert_eq!(elems.len(), 7);
    }

    #[test]
    fn three_measures_second_starts_after_first() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
                meta: MeasureMeta::default(),
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        assert!(layout.measures[1].x_offset > 0.0);
        assert!(layout.measures[2].x_offset > layout.measures[1].x_offset);
    }

    #[test]
    fn staff_width_equals_target_when_specified() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, Some(8000.0));
        assert!((layout.staff_width - 8000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn staff_width_equals_total_when_no_target() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
            meta: MeasureMeta::default(),
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);
        assert!((layout.staff_width - layout.total_width).abs() < f64::EPSILON);
    }

    #[test]
    fn first_measure_additional_voice_starts_after_system_prefix() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![vec![quarter_note(0)]],
            meta: MeasureMeta::default(),
        }];

        let layout = layout_system(&test_prefix(), &measures, &cfg, Some(8000.0));
        let primary_x = layout.measures[0]
            .layout
            .elements
            .iter()
            .find(|element| matches!(element.element, MeasureElement::Note(_)))
            .unwrap()
            .x;
        let secondary_x = layout.measures[0].additional_voice_layouts[0].elements[0].x;

        assert_eq!(secondary_x, primary_x);
    }
}
