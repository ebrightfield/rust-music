use music::notation::clef::Clef;
use music::note::pitch::Pitch;
use music::note::spelling::{Letter, Spelling};

use super::staff::StaffPosition;

/// Diatonic index of a letter within an octave (C=0 through B=6).
fn letter_diatonic_index(letter: Letter) -> i16 {
    match letter {
        Letter::C => 0,
        Letter::D => 1,
        Letter::E => 2,
        Letter::F => 3,
        Letter::G => 4,
        Letter::A => 5,
        Letter::B => 6,
    }
}

/// Absolute diatonic position: octave * 7 + letter index.
/// This gives a unique monotonic value for every diatonic pitch,
/// independent of accidentals.
fn absolute_diatonic(letter: Letter, octave: i8) -> i16 {
    octave as i16 * 7 + letter_diatonic_index(letter)
}

/// Reference pitch and staff position for each clef.
///
/// Treble: G4 sits on staff position 2 (second line from bottom).
/// Bass: F3 sits on staff position 6 (fourth line from bottom).
///
/// Octave-transposing clefs (`treble8va`, `treble8ba`) are *notational*
/// transpositions: the `8` marks that written pitch sounds an octave away, so
/// the written note keeps the base clef's staff placement and only the
/// sounding pitch differs. A written G4 under `treble8ba` therefore sits on the
/// second line exactly as under plain treble — and sounds G3. Giving these
/// clefs a shifted reference pitch would apply the transposition twice, pushing
/// an ordinary guitar part onto ledger lines above the staff.
fn clef_reference(clef: &Clef) -> (Letter, i8, StaffPosition) {
    match clef {
        Clef::Treble | Clef::Treble8va | Clef::Treble8ba => (Letter::G, 4, 2),
        Clef::Bass => (Letter::F, 3, 6),
    }
}

/// Map a `Pitch` to its staff position for a given clef.
///
/// Staff position 0 = bottom line, 8 = top line. Each step is one
/// diatonic half-space (line to adjacent space or vice versa).
/// Accidentals do not affect vertical position — C♯4 and C♭4 both
/// sit at the same staff position as C4.
pub fn pitch_to_staff_position(pitch: &Pitch, clef: &Clef) -> StaffPosition {
    let spelling = Spelling::from(&pitch.note);
    let pitch_diatonic = absolute_diatonic(spelling.letter, pitch.octave);
    let (ref_letter, ref_octave, ref_pos) = clef_reference(clef);
    let ref_diatonic = absolute_diatonic(ref_letter, ref_octave);
    let offset = pitch_diatonic - ref_diatonic;
    // Each diatonic step = one staff position (half-space)
    ref_pos + offset as i8
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::note::Note;

    fn p(note: Note, octave: i8) -> Pitch {
        Pitch::new(note, octave)
    }

    // --- Treble clef ---

    #[test]
    fn treble_middle_c_is_one_ledger_line_below() {
        // C4 = staff position -2 in treble clef
        assert_eq!(pitch_to_staff_position(&p(Note::C, 4), &Clef::Treble), -2);
    }

    #[test]
    fn treble_bottom_line_is_e4() {
        assert_eq!(pitch_to_staff_position(&p(Note::E, 4), &Clef::Treble), 0);
    }

    #[test]
    fn treble_first_space_is_f4() {
        assert_eq!(pitch_to_staff_position(&p(Note::F, 4), &Clef::Treble), 1);
    }

    #[test]
    fn treble_second_line_is_g4() {
        assert_eq!(pitch_to_staff_position(&p(Note::G, 4), &Clef::Treble), 2);
    }

    #[test]
    fn treble_middle_line_is_b4() {
        assert_eq!(pitch_to_staff_position(&p(Note::B, 4), &Clef::Treble), 4);
    }

    #[test]
    fn treble_top_line_is_f5() {
        assert_eq!(pitch_to_staff_position(&p(Note::F, 5), &Clef::Treble), 8);
    }

    #[test]
    fn treble_g5_is_one_space_above() {
        assert_eq!(pitch_to_staff_position(&p(Note::G, 5), &Clef::Treble), 9);
    }

    #[test]
    fn treble_a5_is_one_ledger_line_above() {
        assert_eq!(pitch_to_staff_position(&p(Note::A, 5), &Clef::Treble), 10);
    }

    #[test]
    fn treble_c6_is_two_ledger_lines_above() {
        // C6: diatonic = 6*7+0=42, G4=32, diff=10, pos=2+10=12
        assert_eq!(pitch_to_staff_position(&p(Note::C, 6), &Clef::Treble), 12);
    }

    // --- Accidentals share position with their natural ---

    #[test]
    fn csharp4_same_position_as_c4_in_treble() {
        let natural = pitch_to_staff_position(&p(Note::C, 4), &Clef::Treble);
        let sharp = pitch_to_staff_position(&p(Note::Cis, 4), &Clef::Treble);
        assert_eq!(natural, sharp);
    }

    #[test]
    fn eflat4_same_position_as_e4_in_treble() {
        let natural = pitch_to_staff_position(&p(Note::E, 4), &Clef::Treble);
        let flat = pitch_to_staff_position(&p(Note::Ees, 4), &Clef::Treble);
        assert_eq!(natural, flat);
    }

    // --- Bass clef ---

    #[test]
    fn bass_bottom_line_is_g2() {
        assert_eq!(pitch_to_staff_position(&p(Note::G, 2), &Clef::Bass), 0);
    }

    #[test]
    fn bass_fourth_line_is_f3() {
        // F3 is the clef reference at position 6
        assert_eq!(pitch_to_staff_position(&p(Note::F, 3), &Clef::Bass), 6);
    }

    #[test]
    fn bass_top_line_is_a3() {
        assert_eq!(pitch_to_staff_position(&p(Note::A, 3), &Clef::Bass), 8);
    }

    #[test]
    fn bass_middle_c_is_one_ledger_line_above() {
        // C4 in bass: diatonic = 28, F3 = 24, diff = 4, pos = 6+4 = 10
        assert_eq!(pitch_to_staff_position(&p(Note::C, 4), &Clef::Bass), 10);
    }

    #[test]
    fn bass_middle_line_is_d3() {
        // D3: diatonic = 3*7+1=22, F3=24, diff=-2, pos=6-2=4
        assert_eq!(pitch_to_staff_position(&p(Note::D, 3), &Clef::Bass), 4);
    }

    // --- Octave-transposing clefs ---
    //
    // These are notational transpositions: the written note keeps the base
    // clef's staff placement and only its *sounding* pitch moves. So a written
    // E4/G4 sits on the bottom/second line under `treble8va` and `treble8ba`
    // just as it does under plain treble.

    #[test]
    fn treble8va_places_written_pitches_like_treble() {
        assert_eq!(pitch_to_staff_position(&p(Note::E, 4), &Clef::Treble8va), 0);
        assert_eq!(pitch_to_staff_position(&p(Note::G, 4), &Clef::Treble8va), 2);
    }

    #[test]
    fn treble8ba_places_written_pitches_like_treble() {
        assert_eq!(pitch_to_staff_position(&p(Note::E, 4), &Clef::Treble8ba), 0);
        assert_eq!(pitch_to_staff_position(&p(Note::G, 4), &Clef::Treble8ba), 2);
    }

    /// Regression for docs/slonimsky-cli-bugs.md §7: `treble-8` placed notes an
    /// octave high, so a Bb4–G5 guitar phrase floated on ledger lines instead of
    /// sitting on the staff. Placement must match plain treble exactly.
    #[test]
    fn transposing_clefs_match_treble_placement_across_the_range() {
        let pitches = [
            p(Note::C, 4), p(Note::Bes, 4), p(Note::D, 5),
            p(Note::F, 5), p(Note::G, 5), p(Note::C, 6),
        ];
        for pitch in pitches {
            let base = pitch_to_staff_position(&pitch, &Clef::Treble);
            for clef in [Clef::Treble8va, Clef::Treble8ba] {
                assert_eq!(
                    pitch_to_staff_position(&pitch, &clef),
                    base,
                    "{pitch:?} under {clef:?} must sit where plain treble puts it"
                );
            }
        }
    }

    // --- Edge cases ---

    #[test]
    fn very_low_pitch_gives_large_negative_position() {
        // C2 in treble: diatonic = 14, G4 = 32, diff = -18, pos = 2-18 = -16
        assert_eq!(pitch_to_staff_position(&p(Note::C, 2), &Clef::Treble), -16);
    }

    #[test]
    fn very_high_pitch_gives_large_positive_position() {
        // C7 in treble: diatonic = 49, G4 = 32, diff = 17, pos = 2+17 = 19
        assert_eq!(pitch_to_staff_position(&p(Note::C, 7), &Clef::Treble), 19);
    }

    #[test]
    fn enharmonic_spellings_have_different_positions() {
        // B#3 and C4 are enharmonic but on different lines
        let b_sharp = pitch_to_staff_position(&p(Note::Bis, 3), &Clef::Treble);
        let c_nat = pitch_to_staff_position(&p(Note::C, 4), &Clef::Treble);
        // B#3 diatonic: octave 3, letter B(6) → 3*7+6 = 27; G4=32; diff=-5; pos=2-5=-3
        // C4 diatonic: octave 4, letter C(0) → 4*7+0 = 28; G4=32; diff=-4; pos=2-4=-2
        assert_eq!(b_sharp, -3);
        assert_eq!(c_nat, -2);
        assert_ne!(b_sharp, c_nat);
    }

    #[test]
    fn double_sharp_same_position_as_natural() {
        let natural = pitch_to_staff_position(&p(Note::C, 4), &Clef::Treble);
        let double_sharp = pitch_to_staff_position(&p(Note::Cisis, 4), &Clef::Treble);
        assert_eq!(natural, double_sharp);
    }

    #[test]
    fn double_flat_same_position_as_natural() {
        let natural = pitch_to_staff_position(&p(Note::D, 4), &Clef::Treble);
        let double_flat = pitch_to_staff_position(&p(Note::Deses, 4), &Clef::Treble);
        assert_eq!(natural, double_flat);
    }
}
