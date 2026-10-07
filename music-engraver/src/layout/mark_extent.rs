//! Estimated vertical reach of the marks attached to events, used to size
//! the page and to keep systems far enough apart that one system's marks
//! below the staff clear the next system's marks above it.

use crate::layout::dynamics::{DYNAMICS_ABOVE_STAFF_SS, DYNAMICS_BELOW_STAFF_SS};
use crate::layout::hairpin::HAIRPIN_ABOVE_STAFF_SS;
use crate::layout::lyric::{LYRIC_BELOW_STAFF_SS, LYRIC_FONT_SIZE_SS, LYRIC_VERSE_GAP_SS};
use crate::layout::measure::{MeasureElement, NoteAnnotations};
use crate::layout::placement::Placement;
use crate::layout::system::SystemLayout;
use crate::layout::tempo::{METRONOME_NOTE_SCALE, TEMPO_ABOVE_STAFF_SS, TEMPO_FONT_SIZE_SS};
use crate::layout::text_script::{
    TextScript, TEXT_ASCENT_RATIO, TEXT_DESCENT_RATIO, TEXT_SCRIPT_BELOW_STAFF_SS,
    TEXT_SCRIPT_FONT_SIZE_SS, TEXT_SCRIPT_PADDING_SS, TEXT_SCRIPT_STACK_GAP_SS,
};
use crate::layout::text_spanner::{TEXT_SPANNER_ABOVE_STAFF_SS, TEXT_SPANNER_FONT_SIZE_SS};

/// Annotations on a visible event or an invisible onset.
pub fn element_annotations(element: &MeasureElement) -> Option<&NoteAnnotations> {
    match element {
        MeasureElement::Note(n) => Some(&n.annotations),
        MeasureElement::Chord(c) => Some(&c.annotations),
        MeasureElement::Rest(r) => Some(&r.annotations),
        MeasureElement::Spacer(s) => Some(&s.annotations),
        _ => None,
    }
}

/// Conservative extent of `annotations`' marks beyond the staff, in staff
/// spaces: (above the top line, below the bottom line), covering tempo marks,
/// text scripts and text marks, dynamics and hairpins, above-staff text
/// spanners and lyrics.
pub fn annotation_extent_ss(annotations: &NoteAnnotations) -> (f64, f64) {
    const DYNAMIC_HEIGHT_SS: f64 = 1.7;
    const METRONOME_NOTE_ASCENT_SS: f64 = 2.8 * METRONOME_NOTE_SCALE;
    let gap = TEXT_SCRIPT_STACK_GAP_SS;
    let text_height = |script: &TextScript| {
        (TEXT_ASCENT_RATIO + TEXT_DESCENT_RATIO) * TEXT_SCRIPT_FONT_SIZE_SS * script.size.scale()
    };

    let mut above = 0.0_f64;
    let mut below = 0.0_f64;
    let mut cursor_above = TEXT_SCRIPT_PADDING_SS;
    let mut cursor_below = TEXT_SCRIPT_PADDING_SS;
    if annotations.dynamic.is_some() || annotations.hairpin_start.is_some() {
        match annotations.dynamics_placement {
            Placement::Above => {
                let top = (DYNAMICS_ABOVE_STAFF_SS + 1.1)
                    .max(HAIRPIN_ABOVE_STAFF_SS + 0.5)
                    .max(cursor_above + DYNAMIC_HEIGHT_SS);
                cursor_above = top + gap;
                above = above.max(top);
            }
            Placement::Below => {
                let bottom = (DYNAMICS_BELOW_STAFF_SS + 0.6).max(cursor_below + DYNAMIC_HEIGHT_SS);
                cursor_below = bottom + gap;
                below = below.max(bottom);
            }
        }
    }
    for script in annotations
        .text_scripts
        .iter()
        .chain(&annotations.text_marks)
    {
        let height = text_height(script);
        match script.placement {
            Placement::Above => {
                let top = cursor_above + height;
                cursor_above = top + gap;
                above = above.max(top);
            }
            Placement::Below => {
                let bottom = (TEXT_SCRIPT_BELOW_STAFF_SS
                    + TEXT_DESCENT_RATIO * TEXT_SCRIPT_FONT_SIZE_SS)
                    .max(cursor_below + height);
                cursor_below = bottom + gap;
                below = below.max(bottom);
            }
        }
    }
    if let Some(mark) = &annotations.tempo_mark {
        let lines = usize::from(
            mark.text.is_some() || mark.metronome.is_some() || mark.text_after.is_some(),
        ) + usize::from(mark.text_below.is_some());
        let ascent = if mark.metronome.is_some() {
            METRONOME_NOTE_ASCENT_SS
        } else {
            TEXT_ASCENT_RATIO * TEXT_SCRIPT_FONT_SIZE_SS
        };
        let stacked = lines.saturating_sub(1) as f64 * 1.2 * TEMPO_FONT_SIZE_SS;
        let top = (TEMPO_ABOVE_STAFF_SS + stacked + ascent)
            .max(cursor_above + TEXT_DESCENT_RATIO * TEMPO_FONT_SIZE_SS + stacked + ascent);
        above = above.max(top);
    }
    if let Some(spanner) = &annotations.text_spanner_start {
        if spanner.placement == Placement::Above {
            above = above
                .max(TEXT_SPANNER_ABOVE_STAFF_SS + TEXT_ASCENT_RATIO * TEXT_SPANNER_FONT_SIZE_SS);
        }
    }
    if let Some(last_verse) = annotations.lyrics.iter().map(|lyric| lyric.verse).max() {
        below = below.max(
            LYRIC_BELOW_STAFF_SS
                + f64::from(last_verse.saturating_sub(1)) * LYRIC_VERSE_GAP_SS
                + TEXT_DESCENT_RATIO * LYRIC_FONT_SIZE_SS,
        );
    }
    (above, below)
}

/// Lowest note/ledger ink relative to the bottom staff line, in staff spaces.
/// A whole-system lane prevents a wide dynamic or hairpin from hitting a
/// neighboring note even when its own anchor is higher. The 0.75ss envelope
/// covers notehead half-height and ledger-line stroke below the note center.
pub fn system_note_ink_below_ss(system: &SystemLayout) -> f64 {
    system
        .measures
        .iter()
        .flat_map(|measure| {
            std::iter::once(&measure.layout).chain(&measure.additional_voice_layouts)
        })
        .flat_map(|layout| &layout.elements)
        .filter_map(|positioned| match &positioned.element {
            MeasureElement::Note(note) => Some(note.staff_position),
            MeasureElement::Chord(chord) => chord.staff_positions.iter().copied().min(),
            _ => None,
        })
        .map(|position| {
            if position < 0 {
                0.75 - f64::from(position) * 0.5
            } else {
                0.0
            }
        })
        .fold(0.0_f64, f64::max)
}

/// The largest [`annotation_extent_ss`] over every event of `system`.
pub fn system_mark_extent_ss(system: &SystemLayout) -> (f64, f64) {
    let ink = system_note_ink_below_ss(system);
    system
        .measures
        .iter()
        .flat_map(|m| {
            std::iter::once(&m.layout)
                .chain(m.additional_voice_layouts.iter())
                .flat_map(|layout| layout.elements.iter())
        })
        .filter_map(|positioned| element_annotations(&positioned.element))
        .map(|annotations| {
            let (above, mut below) = annotation_extent_ss(annotations);
            if ink > 0.0
                && annotations
                    .text_scripts
                    .iter()
                    .chain(&annotations.text_marks)
                    .any(|script| script.placement == Placement::Below)
            {
                // Below scripts share the lowered event-mark stack, not
                // their original bottom-staff offset.
                below += ink;
            }
            if ink > 0.0 && annotations.dynamics_placement == Placement::Below {
                if annotations.dynamic.is_some() || annotations.hairpin_start.is_some() {
                    below = below.max(ink + 3.0);
                }
            }
            if let Some(last) = annotations.lyrics.iter().map(|lyric| lyric.verse).max() {
                below =
                    below.max(ink + 4.8 + f64::from(last.saturating_sub(1)) * LYRIC_VERSE_GAP_SS);
            }
            (above, below)
        })
        .fold((0.0, 0.0), |(above, below), (a, b)| {
            (above.max(a), below.max(b))
        })
}
