use music::note::spelling::Accidental;
use smufl::Glyph;

use crate::layout::staff::StaffPosition;

/// Caller-requested display policy for one notated pitch's accidental.
///
/// The policy never changes the pitch itself; it only decides whether its
/// accidental is engraved and how. Every policy records the pitch's own
/// alteration as the in-measure state for its letter and octave, so later
/// notes compare against what the reader has just been told.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AccidentalDisplay {
    /// Engrave the accidental only when the pitch's alteration differs from
    /// the alteration in force for its letter and octave: the in-measure
    /// override if one exists, otherwise the key signature. This covers
    /// cancelling naturals, reinstated key-signature accidentals, and
    /// suppression of repeated accidentals within the measure.
    #[default]
    Auto,
    /// Always engrave the plain accidental, including a natural on a letter the
    /// key signature leaves unaltered, even when it restates the state in force.
    Force,
    /// Always engrave the accidental enclosed in SMuFL accidental parentheses
    /// (a cautionary or courtesy sign). It normally restates the state already
    /// in force; like every policy, it records the pitch's alteration as state.
    Cautionary,
    /// Never engrave the accidental (LilyPond `\once \omit Accidental`). The
    /// pitch still records its alteration as state, so later notes behave
    /// exactly as if the sign had been printed.
    Hide,
}

/// Staff-wide rule deciding which alteration a note's accidental is compared
/// against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AccidentalPolicy {
    /// Common-practice rule: an alteration stays in force for its letter and
    /// octave until the barline, so repeats are suppressed and changes within
    /// the measure are cancelled or reinstated (LilyPond's `default` style).
    #[default]
    Default,
    /// Nothing carries within the measure (LilyPond `\accidentalStyle
    /// forget`): every note is judged against the key signature alone, so an
    /// alteration outside the key prints on every note, no automatic
    /// cancellation natural is printed, and a natural on a letter the key
    /// leaves unaltered prints only when forced or cautionary.
    Forget,
}

/// An accidental the resolver decided to engrave: its SMuFL glyph and whether
/// it is enclosed by `AccidentalParensLeft` / `AccidentalParensRight`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedAccidental {
    /// The accidental glyph itself.
    pub glyph: Glyph,
    /// Whether the glyph is enclosed by SMuFL accidental parentheses.
    pub parenthesized: bool,
}

impl ResolvedAccidental {
    /// An accidental drawn without parentheses.
    pub const fn plain(glyph: Glyph) -> Self {
        Self {
            glyph,
            parenthesized: false,
        }
    }

    /// A cautionary accidental drawn inside SMuFL accidental parentheses.
    pub const fn cautionary(glyph: Glyph) -> Self {
        Self {
            glyph,
            parenthesized: true,
        }
    }
}

/// Map a `music::Accidental` to the corresponding SMuFL glyph.
///
/// Whether the glyph is engraved at all is decided by accidental resolution
/// against the key signature and measure state, not by this mapping.
pub fn accidental_glyph(acc: Accidental) -> Glyph {
    match acc {
        Accidental::Natural => Glyph::AccidentalNatural,
        Accidental::Sharp => Glyph::AccidentalSharp,
        Accidental::Flat => Glyph::AccidentalFlat,
        Accidental::DoubleSharp => Glyph::AccidentalDoubleSharp,
        Accidental::DoubleFlat => Glyph::AccidentalDoubleFlat,
    }
}

/// Padding between the right edge of an accidental glyph and the left edge
/// of the notehead, in staff spaces.
///
/// Standard engraving practice places the accidental close to but not touching
/// the notehead. Typical values range from 0.1 to 0.2 staff spaces; we use
/// approximately 1/8 staff space (0.12).
pub const ACCIDENTAL_NOTEHEAD_PADDING_SS: f64 = 0.12;

/// Horizontal gap between adjacent stacked accidental columns of one chord,
/// in staff spaces.
pub const ACCIDENTAL_COLUMN_GAP_SS: f64 = 0.1;

/// Minimum vertical distance, in staff positions, at which two accidentals of
/// one chord may share a column. Six steps is a seventh: closer accidentals
/// (up to a sixth apart) overlap vertically and need separate columns.
pub const ACCIDENTAL_COLUMN_CLEARANCE: i8 = 6;

/// Compute the x-position at which to draw an accidental, given the notehead's
/// x-position and the advance width of the accidental glyph.
///
/// The accidental is placed to the left of the notehead with standard padding.
/// All values are in font design units.
pub fn accidental_x(notehead_x: f64, accidental_advance_width: f64, staff_space: f64) -> f64 {
    let padding = ACCIDENTAL_NOTEHEAD_PADDING_SS * staff_space;
    notehead_x - accidental_advance_width - padding
}

/// Stacked accidental columns for the accidentals of one chord.
#[derive(Clone, Debug, PartialEq)]
pub struct AccidentalColumns {
    /// For each input accidental, in input order: how far left of the column
    /// nearest the noteheads its column's right edge sits (0 for that column).
    pub column_offsets: Vec<f64>,
    /// Total width of all columns plus the gaps between them.
    pub extent: f64,
}

/// Stack the accidentals of one chord into non-overlapping columns.
///
/// `accidentals` holds `(staff_position, width)` for every engraved accidental
/// of the chord, in any order; `width` includes any enclosing parentheses.
/// Following Gould's ordering, accidentals are placed highest, lowest,
/// second-highest, second-lowest, …; each takes the column nearest the
/// noteheads whose occupants are all at least [`ACCIDENTAL_COLUMN_CLEARANCE`]
/// staff positions away, opening a new column further left otherwise. Every
/// accidental is right-aligned in its column, and a column is as wide as its
/// widest member, so a wide (e.g. parenthesized) accidental pushes every
/// column to its left further out.
pub fn layout_accidental_columns(
    accidentals: &[(StaffPosition, f64)],
    column_gap: f64,
) -> AccidentalColumns {
    let count = accidentals.len();
    let mut by_height: Vec<usize> = (0..count).collect();
    by_height.sort_by_key(|&index| std::cmp::Reverse(accidentals[index].0));
    // Gould: highest, lowest, second-highest, second-lowest, …
    let order: Vec<usize> = (0..count)
        .map(|turn| {
            if turn % 2 == 0 {
                by_height[turn / 2]
            } else {
                by_height[count - 1 - turn / 2]
            }
        })
        .collect();

    let mut column_of = vec![0_usize; count];
    let mut column_widths: Vec<f64> = Vec::new();
    for (turn, &index) in order.iter().enumerate() {
        let (position, width) = accidentals[index];
        let clears = |other: usize| {
            (i32::from(accidentals[other].0) - i32::from(position)).abs()
                >= i32::from(ACCIDENTAL_COLUMN_CLEARANCE)
        };
        let column = (0..=column_widths.len())
            .find(|&column| {
                order[..turn]
                    .iter()
                    .all(|&other| column_of[other] != column || clears(other))
            })
            .expect("a new empty column always fits");
        column_of[index] = column;
        if column == column_widths.len() {
            column_widths.push(width);
        } else {
            column_widths[column] = column_widths[column].max(width);
        }
    }

    let mut column_right_offsets = Vec::with_capacity(column_widths.len());
    let mut extent = 0.0;
    for (column, width) in column_widths.iter().enumerate() {
        if column > 0 {
            extent += column_gap;
        }
        column_right_offsets.push(extent);
        extent += width;
    }
    AccidentalColumns {
        column_offsets: column_of
            .iter()
            .map(|&column| column_right_offsets[column])
            .collect(),
        extent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alteration_maps_to_its_smufl_glyph() {
        let cases = [
            (Accidental::Natural, Glyph::AccidentalNatural),
            (Accidental::Sharp, Glyph::AccidentalSharp),
            (Accidental::Flat, Glyph::AccidentalFlat),
            (Accidental::DoubleSharp, Glyph::AccidentalDoubleSharp),
            (Accidental::DoubleFlat, Glyph::AccidentalDoubleFlat),
        ];
        for (accidental, glyph) in cases {
            assert_eq!(accidental_glyph(accidental), glyph, "{accidental:?}");
        }
    }

    #[test]
    fn accidentals_a_seventh_apart_share_one_column() {
        let columns = layout_accidental_columns(&[(1, 249.0), (8, 249.0)], 25.0);
        assert_eq!(columns.column_offsets, vec![0.0, 0.0]);
        assert_eq!(columns.extent, 249.0);
    }

    #[test]
    fn stacked_thirds_follow_gould_outer_first_column_order() {
        // Bottom-to-top C, E, G, B a third apart (input deliberately unsorted).
        // Highest (B=6) and lowest (C=0) are a seventh apart and share the
        // nearest column; then G (4) and E (2) each need a new column.
        let columns =
            layout_accidental_columns(&[(2, 100.0), (6, 100.0), (0, 100.0), (4, 100.0)], 10.0);
        assert_eq!(columns.column_offsets, vec![220.0, 0.0, 0.0, 110.0]);
        assert_eq!(columns.extent, 320.0);
    }

    #[test]
    fn a_wide_member_widens_its_column_and_pushes_outer_columns() {
        // A parenthesized accidental (width 531) in the nearest column pushes
        // the plain accidental a third below out past its full width.
        let columns = layout_accidental_columns(&[(4, 531.0), (2, 249.0)], 25.0);
        assert_eq!(columns.column_offsets, vec![0.0, 556.0]);
        assert_eq!(columns.extent, 805.0);
        // Narrower nearest column → the outer column sits correspondingly closer.
        let plain = layout_accidental_columns(&[(4, 249.0), (2, 249.0)], 25.0);
        assert_eq!(plain.column_offsets, vec![0.0, 274.0]);
        assert_eq!(plain.extent, 523.0);
    }

    #[test]
    fn accidental_x_places_left_of_notehead() {
        let notehead_x = 500.0;
        let acc_width = 150.0;
        let staff_space = 250.0;
        let x = accidental_x(notehead_x, acc_width, staff_space);

        // Expected: 500 - 150 - (0.12 * 250) = 500 - 150 - 30 = 320
        assert!((x - 320.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn accidental_x_with_zero_width() {
        let x = accidental_x(100.0, 0.0, 250.0);
        // 100 - 0 - 30 = 70
        assert!((x - 70.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn accidental_x_increases_with_smaller_accidental() {
        let staff_space = 250.0;
        let x_wide = accidental_x(500.0, 200.0, staff_space);
        let x_narrow = accidental_x(500.0, 100.0, staff_space);
        // Narrower accidental → placed closer to notehead (higher x)
        assert!(x_narrow > x_wide);
    }

    #[test]
    fn accidental_x_padding_scales_with_staff_space() {
        let acc_width = 150.0;
        let x_small = accidental_x(500.0, acc_width, 200.0);
        let x_large = accidental_x(500.0, acc_width, 400.0);
        // Larger staff space → more padding → accidental farther left (lower x)
        assert!(x_small > x_large);
    }
}
