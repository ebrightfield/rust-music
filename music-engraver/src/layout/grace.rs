//! Grace notes: the builder's written grace groups, their resolved layout
//! form, and the geometry of a grace group engraved before its principal.
//!
//! Grace notes are engraved at [`GRACE_NOTE_SCALE`] from the ordinary
//! notehead, accidental, stem, flag, and dot glyphs, so a group may hold any
//! number of notes of any written duration. Two or more eighths (or shorter)
//! are beamed together; a lone flagged grace keeps its flag; an acciaccatura
//! slashes its first stem. Grace stems point up unless the principal's stem
//! is forced down (a lower voice). The group reserves horizontal space before
//! its principal ([`grace_group_extent`]), so the principal moves right.

use music::notation::rhythm::duration::Duration;
use music::note::pitch::Pitch;

use crate::font::EngravingConfig;
use crate::layout::accidental::{AccidentalDisplay, ResolvedAccidental};
use crate::layout::dot::{DOT_INTER_DOT_SPACING_SS, DOT_NOTEHEAD_PADDING_SS};
use crate::layout::slur::{layout_slur, slur_direction_from_stem, SlurLayout};
use crate::layout::staff::{StaffLayout, StaffPosition};
use crate::layout::stem::{StemDirection, DEFAULT_STEM_LENGTH_SS, MIN_STEM_LENGTH_SS};

/// Grace note type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraceNoteKind {
    /// Slashed grace (acciaccatura; LilyPond `\slashedGrace`, or
    /// `\acciaccatura` together with a slur): the first stem carries a slash.
    Acciaccatura,
    /// Unslashed grace (LilyPond `\grace`, or `\appoggiatura` together with a
    /// slur).
    Appoggiatura,
}

/// Size of grace notes relative to normal notes: LilyPond's grace font size
/// −3, i.e. `2^(−3/6)`.
pub const GRACE_NOTE_SCALE: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Gap between a grace group's rightmost ink and the principal's leftmost ink
/// (its accidental, or its notehead), in staff spaces.
pub const GRACE_PRINCIPAL_GAP_SS: f64 = 0.5;

/// Gap after each grace note before the next one, in full-size staff spaces
/// (scaled with the grace).
pub const GRACE_NOTE_GAP_SS: f64 = 0.5;

/// Notehead advance estimate at full size, in staff spaces (Bravura
/// `noteheadBlack`; also the measure layout's notehead rod).
const NOTEHEAD_WIDTH_SS: f64 = 1.18;

/// Width reserved for one plain grace accidental at full size, padding
/// included, in staff spaces (the measure layout's accidental rod).
const ACCIDENTAL_WIDTH_SS: f64 = 1.0;

/// Extra width reserved for an accidental's parentheses at full size, in
/// staff spaces (Bravura `accidentalParensLeft` + `accidentalParensRight`).
const ACCIDENTAL_PARENS_WIDTH_SS: f64 = 1.128;

/// How far a stem-up flag reaches right of its stem at full size, in staff
/// spaces (Bravura `flag8thUp` is 1.056 wide).
const FLAG_REACH_SS: f64 = 1.1;

/// How far an acciaccatura slash reaches right of its stem at full size, in
/// staff spaces (Bravura `flag8thUp` anchor `graceNoteSlashNE.x` is 1.284).
const SLASH_REACH_SS: f64 = 1.3;

/// Largest rise or fall of a grace beam from its first to its last stem, at
/// full size, in staff spaces.
const BEAM_MAX_RISE_SS: f64 = 1.0;

/// Length of a fractional (stub) grace beam at full size, in staff spaces.
const BEAM_STUB_LENGTH_SS: f64 = 0.75;

/// One written grace note: pitch, written duration, and accidental display.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WrittenGraceNote {
    /// Written pitch.
    pub pitch: Pitch,
    /// Written duration (eighth, sixteenth, …; dots allowed).
    pub duration: Duration,
    /// Accidental display policy, resolved through the staff's accidental
    /// state like any other note.
    pub accidental: AccidentalDisplay,
}

/// A written grace group engraved before its principal note or chord, as
/// passed to [`crate::score::ScoreBuilder::grace_notes`].
///
/// Score conversion resolves it (staff positions with the clef in force,
/// accidentals through the measure's accidental state in onset order just
/// before the principal) into a [`GraceGroup`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraceNotes {
    /// Slashed or unslashed.
    pub kind: GraceNoteKind,
    /// Whether a slur runs from the first grace note to the principal.
    pub slur: bool,
    /// Grace notes in written order.
    pub notes: Vec<WrittenGraceNote>,
}

impl GraceNotes {
    /// An empty grace group of `kind` without a slur.
    pub fn new(kind: GraceNoteKind) -> Self {
        Self {
            kind,
            slur: false,
            notes: Vec::new(),
        }
    }

    /// Append a grace note with automatic accidental display.
    pub fn note(self, pitch: Pitch, duration: Duration) -> Self {
        self.note_with_accidental(pitch, duration, AccidentalDisplay::Auto)
    }

    /// Append a grace note with an explicit [`AccidentalDisplay`] policy.
    pub fn note_with_accidental(
        mut self,
        pitch: Pitch,
        duration: Duration,
        accidental: AccidentalDisplay,
    ) -> Self {
        self.notes.push(WrittenGraceNote {
            pitch,
            duration,
            accidental,
        });
        self
    }

    /// Slur the first grace note to the principal (as LilyPond's
    /// `\acciaccatura` and `\appoggiatura` do, or a written `( )`).
    pub fn slur(mut self) -> Self {
        self.slur = true;
        self
    }
}

/// One resolved grace note, ready for layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraceNoteEvent {
    /// Staff position (bottom line = 0).
    pub staff_position: StaffPosition,
    /// Log2 of the written duration (3 = eighth, 4 = sixteenth, …).
    pub duration_log2: i8,
    /// Number of augmentation dots.
    pub dots: u8,
    /// Accidental to engrave, already resolved.
    pub accidental: Option<ResolvedAccidental>,
}

/// A resolved grace group carried by its principal's
/// [`crate::layout::measure::NoteAnnotations::grace_group`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraceGroup {
    /// Slashed or unslashed.
    pub kind: GraceNoteKind,
    /// Whether a slur runs from the first grace note to the principal.
    pub slur: bool,
    /// Grace notes in written order.
    pub notes: Vec<GraceNoteEvent>,
}

impl GraceGroup {
    /// Whether the group is beamed: two or more notes, all eighths or shorter.
    pub fn is_beamed(&self) -> bool {
        self.notes.len() >= 2 && self.notes.iter().all(|note| note.duration_log2 >= 3)
    }
}

/// Stem direction of a grace group: up, unless the principal's stem is
/// forced down (an even-numbered lower voice).
pub fn grace_stem_direction(principal_stem: Option<StemDirection>) -> StemDirection {
    match principal_stem {
        Some(StemDirection::Down) => StemDirection::Down,
        _ => StemDirection::Up,
    }
}

/// A grace note's stem: x of its centre line and its two ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraceStemLayout {
    /// X of the stem's centre line.
    pub x: f64,
    /// Y where the stem meets the notehead.
    pub y_notehead: f64,
    /// Y of the stem tip (at the beam's outer edge when beamed).
    pub y_tip: f64,
}

/// Placement of one grace note.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraceNoteLayout {
    /// Left edge of the (scaled) notehead.
    pub x: f64,
    /// Y of the notehead.
    pub y: f64,
    /// Staff position.
    pub staff_position: StaffPosition,
    /// Log2 of the written duration.
    pub duration_log2: i8,
    /// Number of augmentation dots.
    pub dots: u8,
    /// Accidental to engrave left of the notehead.
    pub accidental: Option<ResolvedAccidental>,
    /// Stem, absent for whole notes and breves.
    pub stem: Option<GraceStemLayout>,
    /// Flags at the stem tip (0 when beamed).
    pub flags: u8,
}

/// One beam line of a beamed grace group. `(x1, y1)`–`(x2, y2)` is the
/// beam's outer edge (the stem-tip side); the beam extends
/// [`GraceGroupLayout::beam_thickness`] towards the noteheads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraceBeamSegment {
    /// Left end x.
    pub x1: f64,
    /// Outer-edge y at the left end.
    pub y1: f64,
    /// Right end x.
    pub x2: f64,
    /// Outer-edge y at the right end.
    pub y2: f64,
}

/// Geometry of a grace group placed before its principal.
#[derive(Clone, Debug, PartialEq)]
pub struct GraceGroupLayout {
    /// Glyph scale ([`GRACE_NOTE_SCALE`]).
    pub scale: f64,
    /// Stem direction shared by the group.
    pub stem_direction: StemDirection,
    /// Grace notes in written order.
    pub notes: Vec<GraceNoteLayout>,
    /// Beam lines (empty unless [`GraceGroup::is_beamed`]).
    pub beams: Vec<GraceBeamSegment>,
    /// Beam thickness (scaled), in font units.
    pub beam_thickness: f64,
    /// Whether the first note's stem carries an acciaccatura slash.
    pub slashed: bool,
    /// Leftmost x the group occupies (its first accidental or notehead).
    pub left_x: f64,
}

/// Width of a grace note's accidental column, in font units.
fn accidental_width(note: &GraceNoteEvent, unit: f64) -> f64 {
    note.accidental.map_or(0.0, |accidental| {
        let parens = if accidental.parenthesized {
            ACCIDENTAL_PARENS_WIDTH_SS
        } else {
            0.0
        };
        (ACCIDENTAL_WIDTH_SS + parens) * unit
    })
}

/// Width of a grace note's augmentation dots past its notehead.
fn dots_width(note: &GraceNoteEvent, unit: f64) -> f64 {
    if note.dots == 0 {
        return 0.0;
    }
    (DOT_NOTEHEAD_PADDING_SS + f64::from(note.dots) * DOT_INTER_DOT_SPACING_SS) * unit
}

/// Notehead left edges of `group`, measured from the group's leftmost ink,
/// and the group's total ink width. `unit` is one scaled staff space.
fn horizontal_layout(
    group: &GraceGroup,
    stem_direction: StemDirection,
    unit: f64,
) -> (Vec<f64>, f64) {
    let head = NOTEHEAD_WIDTH_SS * unit;
    let beamed = group.is_beamed();
    let mut xs = Vec::with_capacity(group.notes.len());
    let mut cursor = 0.0_f64;
    for (index, note) in group.notes.iter().enumerate() {
        if index > 0 {
            cursor += GRACE_NOTE_GAP_SS * unit;
        }
        let x = cursor + accidental_width(note, unit);
        xs.push(x);
        cursor = x + head + dots_width(note, unit);
        if stem_direction == StemDirection::Up && note.duration_log2 >= 1 {
            let stem = x + head;
            let flagged = !beamed && note.duration_log2 >= 3;
            if flagged {
                cursor = cursor.max(stem + FLAG_REACH_SS * unit);
            }
            if index == 0 && group.kind == GraceNoteKind::Acciaccatura {
                cursor = cursor.max(stem + SLASH_REACH_SS * unit);
            }
        }
    }
    (xs, cursor)
}

/// Horizontal space a grace group occupies before its principal's leftmost
/// ink, including the gap to the principal, in font units. Zero for an empty
/// group.
pub fn grace_group_extent(
    group: &GraceGroup,
    stem_direction: StemDirection,
    staff_space: f64,
) -> f64 {
    if group.notes.is_empty() {
        return 0.0;
    }
    let (_, width) = horizontal_layout(group, stem_direction, staff_space * GRACE_NOTE_SCALE);
    width + GRACE_PRINCIPAL_GAP_SS * staff_space
}

/// Lay out `group` so that its rightmost ink ends [`GRACE_PRINCIPAL_GAP_SS`]
/// before `principal_left_x`, the principal's leftmost ink.
pub fn layout_grace_group(
    group: &GraceGroup,
    principal_left_x: f64,
    stem_direction: StemDirection,
    staff: &StaffLayout,
    config: &EngravingConfig,
) -> GraceGroupLayout {
    let scale = GRACE_NOTE_SCALE;
    let unit = staff.staff_space * scale;
    let head = NOTEHEAD_WIDTH_SS * unit;
    let left_x =
        principal_left_x - grace_group_extent(group, stem_direction, staff.staff_space);
    let (offsets, _) = horizontal_layout(group, stem_direction, unit);
    let stem_thickness = config.stem_thickness_fu() * scale;
    let stem_length = DEFAULT_STEM_LENGTH_SS * unit;
    let sign = match stem_direction {
        StemDirection::Up => -1.0,
        StemDirection::Down => 1.0,
    };
    let stem_x_of = |x: f64| match stem_direction {
        StemDirection::Up => x + head - stem_thickness / 2.0,
        StemDirection::Down => x + stem_thickness / 2.0,
    };

    let mut notes: Vec<GraceNoteLayout> = group
        .notes
        .iter()
        .zip(&offsets)
        .map(|(note, &offset)| {
            let x = left_x + offset;
            let y = staff.y_of(note.staff_position);
            let stem = (note.duration_log2 >= 1).then(|| GraceStemLayout {
                x: stem_x_of(x),
                y_notehead: y,
                y_tip: y + sign * stem_length,
            });
            GraceNoteLayout {
                x,
                y,
                staff_position: note.staff_position,
                duration_log2: note.duration_log2,
                dots: note.dots,
                accidental: note.accidental,
                stem,
                flags: u8::try_from(note.duration_log2.saturating_sub(2)).unwrap_or(0),
            }
        })
        .collect();

    let beam_thickness = config.beam_thickness_fu() * scale;
    let mut beams = Vec::new();
    if group.is_beamed() {
        let beam_step = (config.beam_thickness_fu() + config.beam_spacing_fu()) * scale;
        let levels: Vec<u8> = notes.iter().map(|note| note.flags).collect();
        let first = notes[0].stem.expect("beamed graces are stemmed");
        let last = notes[notes.len() - 1].stem.expect("beamed graces are stemmed");
        let max_rise = BEAM_MAX_RISE_SS * unit;
        let y_first = first.y_tip;
        let y_last = y_first + (last.y_tip - y_first).clamp(-max_rise, max_rise);
        let slope = (y_last - y_first) / (last.x - first.x);
        // Shift the beam away from the noteheads until every stem reaches the
        // minimum length plus room for its own secondary beams.
        let min_length = MIN_STEM_LENGTH_SS * unit;
        let shift = notes
            .iter()
            .zip(&levels)
            .map(|(note, &level)| {
                let stem = note.stem.expect("beamed graces are stemmed");
                let beam_y = y_first + slope * (stem.x - first.x);
                let required = min_length + f64::from(level.saturating_sub(1)) * beam_step;
                // Positive when the beam sits too close to the notehead.
                (beam_y - (note.y + sign * required)) * -sign
            })
            .fold(0.0_f64, f64::max);
        let beam_y_at = |x: f64| y_first + slope * (x - first.x) + sign * shift;
        for note in &mut notes {
            let stem = note.stem.as_mut().expect("beamed graces are stemmed");
            stem.y_tip = beam_y_at(stem.x);
            note.flags = 0;
        }
        let half = stem_thickness / 2.0;
        let segment = |level: u8, x1: f64, x2: f64| {
            let inset = -sign * f64::from(level - 1) * beam_step;
            GraceBeamSegment {
                x1,
                y1: beam_y_at(x1) + inset,
                x2,
                y2: beam_y_at(x2) + inset,
            }
        };
        let stem_xs: Vec<f64> = notes
            .iter()
            .map(|note| note.stem.expect("beamed graces are stemmed").x)
            .collect();
        let count = stem_xs.len();
        beams.push(segment(1, stem_xs[0] - half, stem_xs[count - 1] + half));
        let deepest = levels.iter().copied().max().unwrap_or(1);
        let stub = BEAM_STUB_LENGTH_SS * unit;
        for level in 2..=deepest {
            let mut index = 0;
            while index < count {
                if levels[index] < level {
                    index += 1;
                    continue;
                }
                let start = index;
                while index + 1 < count && levels[index + 1] >= level {
                    index += 1;
                }
                if index > start {
                    beams.push(segment(
                        level,
                        stem_xs[start] - half,
                        stem_xs[index] + half,
                    ));
                } else if start + 1 < count {
                    beams.push(segment(level, stem_xs[start] - half, stem_xs[start] + stub));
                } else {
                    beams.push(segment(level, stem_xs[start] - stub, stem_xs[start] + half));
                }
                index += 1;
            }
        }
    }

    GraceGroupLayout {
        scale,
        stem_direction,
        notes,
        beams,
        beam_thickness,
        slashed: group.kind == GraceNoteKind::Acciaccatura,
        left_x,
    }
}

/// Slur from the first grace note to the principal notehead.
///
/// Like any slur it curves away from the principal's stem. It starts under
/// (or over) the first grace notehead's centre and ends at
/// `principal_center_x`, the centre of the principal notehead it attaches to.
/// Returns `None` when the span is degenerate.
pub fn layout_grace_slur(
    layout: &GraceGroupLayout,
    principal_center_x: f64,
    principal_staff_position: StaffPosition,
    principal_stem_direction: StemDirection,
    staff: &StaffLayout,
    config: &EngravingConfig,
) -> Option<SlurLayout> {
    let first = layout.notes.first()?;
    let x_start = first.x + NOTEHEAD_WIDTH_SS * staff.staff_space * layout.scale / 2.0;
    let x_end = principal_center_x;
    if x_end - x_start < 0.1 * staff.staff_space {
        return None;
    }
    Some(layout_slur(
        x_start,
        x_end,
        first.y,
        staff.y_of(principal_staff_position),
        slur_direction_from_stem(principal_stem_direction),
        config,
    ))
}

#[cfg(test)]
mod tests;
