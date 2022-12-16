use music::notation::clef::Clef;
use music::note::{Note, Pitch};
use music::note_collections::spelling::HasSpelling;
use music::note_collections::Voicing;

/// Since the canonical voicing calculation is basically the same for three and four
/// note chords, it is DRYest to create this shared implementation.
pub trait CanonicalVoicings {
    /// The size of the chord.
    const N: usize;
    /// Set of voicing families. Each of these can be rotated to obtain
    /// N distinct inversions for each family.
    const FAMILIES: &'static [&'static[usize]];

    /// Returns the canonical voicings for an N-note chord, well-ordered and grouped.
    ///
    /// By "canonical voicings", here we simply mean that no adjacently stacked
    /// interval is wider than an octave, and every chord-tone occurs exactly once.
    ///
    /// WARNING: Does not check at runtime whether you've passed it a three-note chord!
    fn voicings(notes: &Vec<Note>) -> Vec<Vec<Voicing>> {
        let mut voicings = vec![];
        for family in Self::FAMILIES {
            let mut voicing_family = vec![];
            for inversion_num in 0usize..Self::N {
                let mut pitches: Vec<Pitch> = vec![];
                for idx_from_family in *family {
                    let idx = (idx_from_family + inversion_num).rem_euclid(Self::N);
                    pitches.push(pitches
                        .last()
                        .map(|p| p.up_to_note(&notes[idx]).unwrap())
                        .unwrap_or(Pitch::new(notes[inversion_num], 4).unwrap())
                    );
                }
                voicing_family.push(
                    Voicing::new(pitches)
                        .normalize_register_to_clef(Clef::Treble).unwrap()
                );
            }
            voicings.push(voicing_family);
        }
        voicings
    }
}