//! Bar-number placement shared by every score renderer (single staff,
//! multi-staff, guitar, and tablature).
//!
//! Numbers come from each measure's logical number
//! ([`MeasureMeta::number`](crate::layout::measure_meta::MeasureMeta::number)):
//! a pickup is bar 0 and is not printed, and the later visual pieces of a
//! measure split at an inline barline share its number and print none.

use crate::layout::measure::MeasureElement;
use crate::layout::system::SystemLayout;

/// Distance of a bar number's baseline above the top staff line, in staff
/// spaces. Slightly below rehearsal/tempo mark territory (~2.5–2.8 ss).
pub const BAR_NUMBER_ABOVE_STAFF_SS: f64 = 1.8;

/// Font size of bar numbers, in staff spaces.
pub const BAR_NUMBER_FONT_SIZE_SS: f64 = 1.2;

/// Which measures print their number above the staff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeasureNumbering {
    /// No bar numbers.
    #[default]
    Hidden,
    /// The number of each system's first measure, at the start of its
    /// content (after the clef, key and time signature).
    SystemStart,
    /// Every measure's number where it begins: at its opening barline, or at
    /// the start of the content for a measure that opens a system.
    EveryBar,
}

/// Where one measure begins, for bar numbering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarNumberSlot {
    /// The measure's logical number.
    pub number: i32,
    /// Absolute x where the measure begins: its opening barline, or the
    /// first content x when it opens a system.
    pub x: f64,
    /// Whether the measure is the first on its system.
    pub system_start: bool,
    /// Whether the measure prints a number at all (false for pickups and for
    /// continuation pieces of a split measure).
    pub numbered: bool,
}

/// A placed bar number: its text is drawn left-aligned at `(x, y)`, `y`
/// being the baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarNumberLayout {
    /// Left edge of the text.
    pub x: f64,
    /// Text baseline.
    pub y: f64,
    /// The number printed.
    pub number: i32,
}

/// Place the bar numbers `numbering` asks for among `slots`, above a staff
/// whose top line is at `staff_top_y`.
pub fn layout_bar_numbers(
    numbering: MeasureNumbering,
    slots: impl IntoIterator<Item = BarNumberSlot>,
    staff_top_y: f64,
    staff_space: f64,
) -> Vec<BarNumberLayout> {
    let y = staff_top_y - BAR_NUMBER_ABOVE_STAFF_SS * staff_space;
    slots
        .into_iter()
        .filter(|slot| {
            slot.numbered
                && match numbering {
                    MeasureNumbering::Hidden => false,
                    MeasureNumbering::SystemStart => slot.system_start,
                    MeasureNumbering::EveryBar => true,
                }
        })
        .map(|slot| BarNumberLayout {
            x: slot.x,
            y,
            number: slot.number,
        })
        .collect()
}

/// Bar-number slots of one laid-out system drawn with its origin at `x`.
///
/// A measure that opens the system begins at its first element after the
/// prefix (clef, key and time signature); any other measure begins at the
/// closing barline of the measure before it.
pub fn system_bar_number_slots(system: &SystemLayout, x: f64) -> Vec<BarNumberSlot> {
    let mut slots = Vec::with_capacity(system.measures.len());
    for (index, measure) in system.measures.iter().enumerate() {
        let start = if index == 0 {
            let content_x = measure
                .layout
                .elements
                .iter()
                .find(|element| {
                    !matches!(
                        element.element,
                        MeasureElement::Clef(_)
                            | MeasureElement::KeySignature(_)
                            | MeasureElement::TimeSignature(_)
                    )
                })
                .map_or(measure.layout.total_width, |element| element.x);
            measure.x_offset + content_x
        } else {
            let previous = &system.measures[index - 1];
            previous.x_offset + previous.layout.closing_barline_x()
        };
        slots.push(BarNumberSlot {
            number: measure.meta.number,
            x: x + start,
            system_start: index == 0,
            numbered: !measure.meta.anacrusis && !measure.meta.continuation,
        });
    }
    slots
}
