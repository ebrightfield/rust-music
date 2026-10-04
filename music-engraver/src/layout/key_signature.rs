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
/// Alto clef sharp positions:   F4(7), C4(4), G4(8), D4(5), A3(2), E4(6), B3(3)
/// Tenor clef sharp positions:  F3(2), C4(6), G3(3), D4(7), A3(4), E4(8), B3(5)
///
/// The C-clef patterns follow VexFlow `KeySignature::convertAccLines`
/// (src/keysignature.ts): alto is the treble shape shifted down half a line
/// (`offset = 0.5`), while tenor sharps use VexFlow's custom line list
/// `[3, 1, 2.5, 0.5, 2, 0, 1.5]` — the conventional tenor pattern whose first
/// sharp (F♯3) sits low in the staff rather than on a ledger line above it.
/// VexFlow lines count down from the top line in whole-line units, so
/// `staff_position = 8 - 2 * line`.
fn sharp_positions(clef: &Clef) -> [i8; 7] {
    match clef {
        Clef::Treble | Clef::Treble8va | Clef::Treble8ba => [8, 5, 9, 6, 3, 7, 4],
        Clef::Bass => [6, 3, 7, 4, 1, 5, 2],
        Clef::Alto => [7, 4, 8, 5, 2, 6, 3],
        Clef::Tenor => [2, 6, 3, 7, 4, 8, 5],
    }
}

/// Standard order of flats by letter: B E A D G C F.
/// Staff positions differ per clef.
///
/// Treble clef flat positions: Bb4(4), Eb5(7), Ab4(3), Db5(6), Gb4(2), Cb5(5), Fb4(1)
/// Treble8va/8ba use same visual positions as treble (same staff-line mapping).
/// Bass clef flat positions:   Bb2(2), Eb3(5), Ab2(1), Db3(4), Gb2(0), Cb3(3), Fb2(-1)
/// Alto clef flat positions:   Bb3(3), Eb4(6), Ab3(2), Db4(5), Gb3(1), Cb4(4), Fb3(0)
/// Tenor clef flat positions:  Bb3(5), Eb4(8), Ab3(4), Db4(7), Gb3(3), Cb4(6), Fb3(2)
///
/// The C-clef patterns follow VexFlow `KeySignature::convertAccLines`
/// (src/keysignature.ts): both are the treble flat shape shifted by a
/// constant (alto `offset = 0.5`, tenor `offset = -0.5` lines), converted with
/// `staff_position = 8 - 2 * line`.
fn flat_positions(clef: &Clef) -> [i8; 7] {
    match clef {
        Clef::Treble | Clef::Treble8va | Clef::Treble8ba => [4, 7, 3, 6, 2, 5, 1],
        Clef::Bass => [2, 5, 1, 4, 0, 3, -1],
        Clef::Alto => [3, 6, 2, 5, 1, 4, 0],
        Clef::Tenor => [5, 8, 4, 7, 3, 6, 2],
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
    clef: &Clef,
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
        let layout = key_signature_layout(&KeySignature::Open, &Clef::Treble, fixed_advance, SS);
        assert!(layout.accidentals.is_empty());
        assert!((layout.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn one_sharp_treble() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(1), &Clef::Treble, fixed_advance, SS);
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
            key_signature_layout(&KeySignature::Sharps(2), &Clef::Treble, fixed_advance, SS);
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
            key_signature_layout(&KeySignature::Sharps(7), &Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
        let positions: Vec<i8> = layout
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        assert_eq!(positions, vec![8, 5, 9, 6, 3, 7, 4]);
    }

    #[test]
    fn one_flat_treble() {
        let layout =
            key_signature_layout(&KeySignature::Flats(1), &Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 1);
        assert_eq!(layout.accidentals[0].staff_position, 4); // Bb4
        assert_eq!(layout.accidentals[0].glyph, Glyph::AccidentalFlat);
    }

    #[test]
    fn seven_flats_treble() {
        let layout =
            key_signature_layout(&KeySignature::Flats(7), &Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
        let positions: Vec<i8> = layout
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        assert_eq!(positions, vec![4, 7, 3, 6, 2, 5, 1]);
    }

    #[test]
    fn bass_clef_sharps() {
        let layout = key_signature_layout(&KeySignature::Sharps(7), &Clef::Bass, fixed_advance, SS);
        let positions: Vec<i8> = layout
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        assert_eq!(positions, vec![6, 3, 7, 4, 1, 5, 2]);
    }

    #[test]
    fn bass_clef_flats() {
        let layout = key_signature_layout(&KeySignature::Flats(7), &Clef::Bass, fixed_advance, SS);
        let positions: Vec<i8> = layout
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        assert_eq!(positions, vec![2, 5, 1, 4, 0, 3, -1]);
    }

    #[test]
    fn treble_8va_same_positions_as_treble() {
        let layout_treble =
            key_signature_layout(&KeySignature::Sharps(7), &Clef::Treble, fixed_advance, SS);
        let layout_8va = key_signature_layout(
            &KeySignature::Sharps(7),
            &Clef::Treble8va,
            fixed_advance,
            SS,
        );
        let pos_treble: Vec<i8> = layout_treble
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        let pos_8va: Vec<i8> = layout_8va
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect();
        assert_eq!(pos_treble, pos_8va);
    }

    #[test]
    fn clamped_to_seven() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(10), &Clef::Treble, fixed_advance, SS);
        assert_eq!(layout.accidentals.len(), 7);
    }

    #[test]
    fn zero_sharps_is_empty() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(0), &Clef::Treble, fixed_advance, SS);
        assert!(layout.accidentals.is_empty());
        assert!((layout.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn width_scales_with_count() {
        let layout_3 =
            key_signature_layout(&KeySignature::Sharps(3), &Clef::Treble, fixed_advance, SS);
        let layout_5 =
            key_signature_layout(&KeySignature::Sharps(5), &Clef::Treble, fixed_advance, SS);
        assert!(layout_5.width > layout_3.width);
    }

    #[test]
    fn all_sharps_use_sharp_glyph() {
        let layout =
            key_signature_layout(&KeySignature::Sharps(4), &Clef::Treble, fixed_advance, SS);
        for acc in &layout.accidentals {
            assert_eq!(acc.glyph, Glyph::AccidentalSharp);
        }
    }

    #[test]
    fn all_flats_use_flat_glyph() {
        let layout =
            key_signature_layout(&KeySignature::Flats(4), &Clef::Treble, fixed_advance, SS);
        for acc in &layout.accidentals {
            assert_eq!(acc.glyph, Glyph::AccidentalFlat);
        }
    }

    #[test]
    fn x_offsets_are_monotonically_increasing() {
        let layout =
            key_signature_layout(&KeySignature::Flats(7), &Clef::Treble, fixed_advance, SS);
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
            key_signature_layout(&KeySignature::Sharps(3), &Clef::Treble, fixed_advance, SS);
        for i in 1..layout.accidentals.len() {
            let gap = layout.accidentals[i].x_offset - layout.accidentals[i - 1].x_offset;
            assert!(
                (gap - SS).abs() < f64::EPSILON,
                "spacing should be 1 staff space"
            );
        }
    }

    fn positions_of(key: KeySignature, clef: Clef) -> Vec<i8> {
        key_signature_layout(&key, &clef, fixed_advance, SS)
            .accidentals
            .iter()
            .map(|a| a.staff_position)
            .collect()
    }

    /// Positions pinned to VexFlow `convertAccLines` (see `sharp_positions`).
    #[test]
    fn alto_clef_seven_sharps_and_flats() {
        // F#4 C#4 G#4 D#4 A#3 E#4 B#3
        assert_eq!(
            positions_of(KeySignature::Sharps(7), Clef::Alto),
            vec![7, 4, 8, 5, 2, 6, 3]
        );
        // Bb3 Eb4 Ab3 Db4 Gb3 Cb4 Fb3
        assert_eq!(
            positions_of(KeySignature::Flats(7), Clef::Alto),
            vec![3, 6, 2, 5, 1, 4, 0]
        );
    }

    #[test]
    fn tenor_clef_seven_sharps_and_flats() {
        // F#3 C#4 G#3 D#4 A#3 E#4 B#3 — first sharp low, not on a ledger line.
        assert_eq!(
            positions_of(KeySignature::Sharps(7), Clef::Tenor),
            vec![2, 6, 3, 7, 4, 8, 5]
        );
        // Bb3 Eb4 Ab3 Db4 Gb3 Cb4 Fb3
        assert_eq!(
            positions_of(KeySignature::Flats(7), Clef::Tenor),
            vec![5, 8, 4, 7, 3, 6, 2]
        );
    }

    /// Every C-clef key-signature accidental sits on the staff degree of the
    /// written pitch it alters, as placed by `pitch_to_staff_position`, so the
    /// table and the clef reference cannot drift apart. Natural letters are
    /// used because placement ignores the accidental (and C♭/B♯ carry an
    /// octave convention that is irrelevant to this check).
    #[test]
    fn c_clef_key_signatures_agree_with_note_placement() {
        use crate::layout::note_placement::pitch_to_staff_position;
        use music::note::note::Note;
        use music::note::pitch::Pitch;

        let sharps = |f, c, g, d, a, e, b| {
            [
                (Note::F, f),
                (Note::C, c),
                (Note::G, g),
                (Note::D, d),
                (Note::A, a),
                (Note::E, e),
                (Note::B, b),
            ]
        };
        let flats = |b, e, a, d, g, c, f| {
            [
                (Note::B, b),
                (Note::E, e),
                (Note::A, a),
                (Note::D, d),
                (Note::G, g),
                (Note::C, c),
                (Note::F, f),
            ]
        };
        let cases = [
            (
                Clef::Alto,
                KeySignature::Sharps(7),
                sharps(4, 4, 4, 4, 3, 4, 3),
            ),
            (
                Clef::Alto,
                KeySignature::Flats(7),
                flats(3, 4, 3, 4, 3, 4, 3),
            ),
            (
                Clef::Tenor,
                KeySignature::Sharps(7),
                sharps(3, 4, 3, 4, 3, 4, 3),
            ),
            (
                Clef::Tenor,
                KeySignature::Flats(7),
                flats(3, 4, 3, 4, 3, 4, 3),
            ),
        ];
        for (clef, key, pitches) in cases {
            let expected: Vec<i8> = pitches
                .iter()
                .map(|&(note, octave)| pitch_to_staff_position(&Pitch::new(note, octave), &clef))
                .collect();
            assert_eq!(
                positions_of(key.clone(), clef),
                expected,
                "{clef:?} {key:?}"
            );
        }
    }
}
