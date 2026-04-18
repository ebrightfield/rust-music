use music::notation::clef::Clef;
use smufl::Glyph;

/// Key signature type: number of sharps or flats (0–7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeySignature {
    /// Major/minor key with sharps (1–7).
    Sharps(u8),
    /// Major/minor key with flats (1–7).
    Flats(u8),
    /// Open/atonal — no key signature.
    Open,
}

/// A positioned accidental within a key signature.
#[derive(Clone, Debug)]
pub struct KeySigAccidental {
    /// The SMuFL glyph (sharp or flat).
    pub glyph: Glyph,
    /// Staff position (bottom line = 0) where this accidental sits.
    pub staff_position: i8,
    /// Horizontal offset from the start of the key signature group, in font design units.
    pub x_offset: f64,
}

/// Resolved key signature layout.
#[derive(Clone, Debug)]
pub struct KeySignatureLayout {
    /// Ordered accidentals to render.
    pub accidentals: Vec<KeySigAccidental>,
    /// Total advance width of the key signature group in font design units.
    pub width: f64,
}

/// Standard order of sharps by letter: F C G D A E B.
/// Staff positions differ per clef.
///
/// Treble clef sharp positions: F5(8), C5(5), G5(9), D5(6), A4(3), E5(7), B4(4)
/// Treble8va/8ba use same visual positions as treble (same staff-line mapping).
/// Bass clef sharp positions:   F3(6), C3(3), G3(7), D3(4), A2(1), E3(5), B2(2)
fn sharp_positions(clef: Clef) -> [i8; 7] {
    match clef {
        Clef::Treble | Clef::Treble8va | Clef::Treble8ba => [8, 5, 9, 6, 3, 7, 4],
        Clef::Bass => [6, 3, 7, 4, 1, 5, 2],
    }
}

/// Standard order of flats by letter: B E A D G C F.
/// Staff positions differ per clef.
///
/// Treble clef flat positions: Bb4(4), Eb5(7), Ab4(3), Db5(6), Gb4(2), Cb5(5), Fb4(1)
/// Treble8va/8ba use same visual positions as treble (same staff-line mapping).
/// Bass clef flat positions:   Bb2(2), Eb3(5), Ab2(1), Db3(4), Gb2(0), Cb3(3), Fb2(-1)
fn flat_positions(clef: Clef) -> [i8; 7] {
    match clef {
        Clef::Treble | Clef::Treble8va | Clef::Treble8ba => [4, 7, 3, 6, 2, 5, 1],
        Clef::Bass => [2, 5, 1, 4, 0, 3, -1],
    }
}

/// Spacing between consecutive accidentals in a key signature, in staff spaces.
/// Standard engraving places them tightly — approximately one notehead width apart.
const KEY_SIG_ACCIDENTAL_SPACING_SS: f64 = 1.0;

/// Compute the layout for a key signature.
///
/// `advance_of` returns the advance width (in font design units) for a given glyph.
/// `staff_space` is the font's staff space in design units.
pub fn key_signature_layout(
    key: &KeySignature,
    clef: Clef,
    advance_of: impl Fn(Glyph) -> f64,
    staff_space: f64,
) -> KeySignatureLayout {
    match key {
        KeySignature::Open => KeySignatureLayout {
            accidentals: vec![],
            width: 0.0,
        },
        KeySignature::Sharps(count) => {
            let n = (*count).min(7) as usize;
            let positions = sharp_positions(clef);
            let glyph = Glyph::AccidentalSharp;
            let glyph_width = advance_of(glyph);
            build_accidentals(glyph, &positions[..n], glyph_width, staff_space)
        }
        KeySignature::Flats(count) => {
            let n = (*count).min(7) as usize;
            let positions = flat_positions(clef);
            let glyph = Glyph::AccidentalFlat;
            let glyph_width = advance_of(glyph);
            build_accidentals(glyph, &positions[..n], glyph_width, staff_space)
        }
    }
}

fn build_accidentals(
    glyph: Glyph,
    positions: &[i8],
    glyph_width: f64,
    staff_space: f64,
) -> KeySignatureLayout {
    let spacing = KEY_SIG_ACCIDENTAL_SPACING_SS * staff_space;
    let mut accidentals = Vec::with_capacity(positions.len());
    let mut x = 0.0;

    for &pos in positions {
        accidentals.push(KeySigAccidental {
            glyph,
            staff_position: pos,
            x_offset: x,
        });
        x += spacing;
    }

    // Width is the span from first glyph origin to the right edge of the last glyph
    let width = if positions.is_empty() {
        0.0
    } else {
        (positions.len() - 1) as f64 * spacing + glyph_width
    };

    KeySignatureLayout { accidentals, width }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_advance(_g: Glyph) -> f64 {
        200.0
    }

    const SS: f64 = 250.0;

    #[test]
    fn open_key_empty() {
        let layout = key_signature_layout(&KeySignature::Open, Clef::Treble, fixed_advance, SS);
        assert!(layout.accidentals.is_empty());
        assert!((layout.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn one_sharp_treble() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(1), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 1);
        assert_eq!(layout.accidentals[0].staff_position, 8); // F#5
        assert_eq!(layout.accidentals[0].glyph, Glyph::AccidentalSharp);
        assert!((layout.accidentals[0].x_offset - 0.0).abs() < f64::EPSILON);
        // Width = glyph_width only (single glyph)
        assert!((layout.width - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_sharps_treble() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(2), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 2);
        assert_eq!(layout.accidentals[0].staff_position, 8); // F#
        assert_eq!(layout.accidentals[1].staff_position, 5); // C#
        // Second accidental offset = 1 * spacing = 250
        assert!((layout.accidentals[1].x_offset - 250.0).abs() < f64::EPSILON);
        // Width = 1 * 250 + 200 = 450
        assert!((layout.width - 450.0).abs() < f64::EPSILON);
    }

    #[test]
    fn seven_sharps_treble() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(7), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
        let positions: Vec<i8> = layout.accidentals.iter().map(|a| a.staff_position).collect();
        assert_eq!(positions, vec![8, 5, 9, 6, 3, 7, 4]);
    }

    #[test]
    fn one_flat_treble() {
        let layout =
            key_signature_layout(&KeySignature::Flats(1), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 1);
        assert_eq!(layout.accidentals[0].staff_position, 4); // Bb4
        assert_eq!(layout.accidentals[0].glyph, Glyph::AccidentalFlat);
    }

    #[test]
    fn seven_flats_treble() {
        let layout =
            key_signature_layout(&KeySignature::Flats(7), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
        let positions: Vec<i8> = layout.accidentals.iter().map(|a| a.staff_position).collect();
        assert_eq!(positions, vec![4, 7, 3, 6, 2, 5, 1]);
    }

    #[test]
    fn bass_clef_sharps() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(7), Clef::Bass, fixed_advance, SS);
        let positions: Vec<i8> = layout.accidentals.iter().map(|a| a.staff_position).collect();
        assert_eq!(positions, vec![6, 3, 7, 4, 1, 5, 2]);
    }

    #[test]
    fn bass_clef_flats() {
        let layout =
            key_signature_layout(&KeySignature::Flats(7), Clef::Bass, fixed_advance, SS);
        let positions: Vec<i8> = layout.accidentals.iter().map(|a| a.staff_position).collect();
        assert_eq!(positions, vec![2, 5, 1, 4, 0, 3, -1]);
    }

    #[test]
    fn treble_8va_same_positions_as_treble() {
        let layout_treble =
            key_signature_layout(&KeySignature::Sharps(7), Clef::Treble, fixed_advance, SS);
        let layout_8va =
            key_signature_layout(&KeySignature::Sharps(7), Clef::Treble8va, fixed_advance, SS);
        let pos_treble: Vec<i8> = layout_treble.accidentals.iter().map(|a| a.staff_position).collect();
        let pos_8va: Vec<i8> = layout_8va.accidentals.iter().map(|a| a.staff_position).collect();
        assert_eq!(pos_treble, pos_8va);
    }

    #[test]
    fn clamped_to_seven() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(10), Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
    }

    #[test]
    fn zero_sharps_is_empty() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(0), Clef::Treble, fixed_advance, SS);
        assert!(layout.accidentals.is_empty());
        assert!((layout.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn width_scales_with_count() {
        let layout_3 =
            key_signature_layout(&KeySignature::Sharps(3), Clef::Treble, fixed_advance, SS);
        let layout_5 =
            key_signature_layout(&KeySignature::Sharps(5), Clef::Treble, fixed_advance, SS);
        assert!(layout_5.width > layout_3.width);
    }

    #[test]
    fn all_sharps_use_sharp_glyph() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(4), Clef::Treble, fixed_advance, SS);
        for acc in &layout.accidentals {
            assert_eq!(acc.glyph, Glyph::AccidentalSharp);
        }
    }

    #[test]
    fn all_flats_use_flat_glyph() {
        let layout =
            key_signature_layout(&KeySignature::Flats(4), Clef::Treble, fixed_advance, SS);
        for acc in &layout.accidentals {
            assert_eq!(acc.glyph, Glyph::AccidentalFlat);
        }
    }

    #[test]
    fn x_offsets_are_monotonically_increasing() {
        let layout =
            key_signature_layout(&KeySignature::Flats(7), Clef::Treble, fixed_advance, SS);
        for i in 1..layout.accidentals.len() {
            assert!(
                layout.accidentals[i].x_offset > layout.accidentals[i - 1].x_offset,
                "x_offsets should increase"
            );
        }
    }

    #[test]
    fn accidental_spacing_equals_one_staff_space() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(3), Clef::Treble, fixed_advance, SS);
        for i in 1..layout.accidentals.len() {
            let gap = layout.accidentals[i].x_offset - layout.accidentals[i - 1].x_offset;
            assert!(
                (gap - SS).abs() < f64::EPSILON,
                "spacing should be 1 staff space"
            );
        }
    }
}
