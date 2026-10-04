use crate::error::MusicSemanticsError;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::note::spelling::Spelling;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

pub const MIDDLE_C: u8 = 60;

/// Semitones from the written octave's C up to `note`: the natural letter's
/// offset plus its alteration. Spellings that cross the C boundary leave the
/// `0..=11` block — C♭ is −1 and B♯ is 12.
fn semitones_above_written_c(note: &Note) -> i16 {
    let spelling = Spelling::from(note);
    i16::from(spelling.letter.natural_semitone()) + i16::from(spelling.acc.alteration())
}

/// MIDI number of `note` written in scientific `octave`, unvalidated:
/// `(octave + 1) * 12 + natural letter semitone + alteration`.
///
/// For octave -1..=9 the result lies in -1..=132; [`Pitch::try_new`] rejects
/// anything outside MIDI 0..=127.
fn written_midi(note: &Note, octave: i8) -> i16 {
    (i16::from(octave) + 1) * 12 + semitones_above_written_c(note)
}

/// Scientific octave in which `note` must be written to sound as `midi_note`.
/// The caller guarantees that `note` has `midi_note`'s pitch class. For MIDI
/// 0..=127 the result lies in -2..=9 (only B♯ at MIDI 0 needs octave -2).
fn written_octave(note: &Note, midi_note: u8) -> i8 {
    ((i16::from(midi_note) - semitones_above_written_c(note)).div_euclid(12) - 1) as i8
}

/// A spelled [Note] in a written, scientific octave.
///
/// `octave` always belongs to the written letter, never to the sounding pitch
/// class, so the MIDI number follows the spelling across the C boundary:
/// `Pitch::new(Note::Ces, 4)` is C♭4 = MIDI 59 (sounds as B3) and
/// `Pitch::new(Note::Bis, 3)` is B♯3 = MIDI 60 (sounds as C4). Staff
/// placement, accidental bookkeeping, LilyPond/VexTab octave marks, and
/// `Display` all read `octave` directly as the written octave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pitch {
    /// Associated [Note] instance.
    pub note: Note,
    /// The written scientific octave of `note`'s letter. Middle C = C4.
    /// Range: -1 through 9, further limited so that `midi_note` stays within
    /// 0..=127 (C♭-1 and everything above G9 are unrepresentable).
    pub octave: i8,
    /// The associated MIDI note, which also serves as a good means of measurement in
    /// semitone space. Middle C = 60. Always
    /// `(octave + 1) * 12 + natural letter semitone + alteration`.
    pub midi_note: u8,
}

impl Pitch {
    /// Create a pitch from a spelling and its written scientific octave,
    /// returning an error if the octave lies outside -1..=9 or the resulting
    /// MIDI note lies outside 0..=127 (e.g. C♭-1 would be MIDI -1).
    ///
    /// Prefer [`Pitch::new`] when `octave` is known at compile time.
    pub fn try_new(note: Note, octave: i8) -> Result<Self, MusicSemanticsError> {
        if !(-1..=9).contains(&octave) {
            return Err(MusicSemanticsError::OctaveTooHigh(octave.unsigned_abs()));
        }
        let midi_note = written_midi(&note, octave);
        if midi_note < 0 {
            return Err(MusicSemanticsError::OutOfBoundsLower(0));
        }
        if midi_note > 127 {
            return Err(MusicSemanticsError::MidiTooHigh(midi_note as u8));
        }
        Ok(Self {
            note,
            octave,
            midi_note: midi_note as u8,
        })
    }

    /// Create a pitch, panicking if the octave is out of MIDI range.
    ///
    /// Use [`Pitch::try_new`] for fallible construction. This infallible form
    /// is convenient for literals like `Pitch::new(Note::C, 4)`.
    pub fn new(note: Note, octave: i8) -> Self {
        Self::try_new(note, octave).expect("pitch octave out of range")
    }

    /// Produce a pitch from a MIDI note value. Middle C = 60.
    /// Uses default spelling (naturals where possible, sharps for accidentals).
    ///
    /// # Known Limitations (Tier C debt — D7)
    /// Octave numbering follows the convention where MIDI 60 = C4 (not C5).
    /// Values in `128..=255` return `Err(_)`; values in `0..=127` always succeed.
    /// See `docs/spec-music-midi-debt.md` §4 R23-R26 and §6 D7 for context.
    pub fn from_midi(midi_note_value: u8) -> Result<Self, MusicSemanticsError> {
        Self::from_midi_spelled(midi_note_value, true)
    }

    /// Produce a pitch from a MIDI note value with a spelling preference.
    /// When `prefer_sharp` is true, accidentals are spelled as sharps (C#, D#, etc.).
    /// When `prefer_sharp` is false, accidentals are spelled as flats (Db, Eb, etc.).
    /// Natural notes (C, D, E, F, G, A, B) are always spelled as naturals.
    pub fn from_midi_spelled(
        midi_note_value: u8,
        prefer_sharp: bool,
    ) -> Result<Self, MusicSemanticsError> {
        let pc = Pc::from(midi_note_value % 12);
        let note = if prefer_sharp {
            pc.default_sharp_spelling()
        } else {
            pc.default_flat_spelling()
        };
        Self::from_midi_as(midi_note_value, note)
    }

    /// Spell a MIDI note value as `note`, deriving the written octave from that
    /// spelling: `from_midi_as(59, Note::Ces)` is C♭4 and
    /// `from_midi_as(60, Note::Bis)` is B♯3.
    ///
    /// Errors when `note` does not have the MIDI note's pitch class, when the
    /// value exceeds 127, or when the spelling would need an octave below -1
    /// (B♯ at MIDI 0).
    pub fn from_midi_as(midi_note_value: u8, note: Note) -> Result<Self, MusicSemanticsError> {
        if midi_note_value > 127 {
            return Err(MusicSemanticsError::MidiTooHigh(midi_note_value));
        }
        let pc = Pc::from(midi_note_value % 12);
        if Pc::from(&note) != pc {
            return Err(MusicSemanticsError::NotAMember(
                *pc.notes().first().unwrap(),
                vec![note],
            ));
        }
        Self::try_new(note, written_octave(&note, midi_note_value))
    }

    /// Produce a pitch from a MIDI note value, choosing its spelling from a
    /// "palette" of candidate note values (enharmonic alternatives).
    ///
    /// Uses the first note in `notes` whose pitch class matches the MIDI note;
    /// the octave is the written octave of that spelling (see
    /// [`Pitch::from_midi_as`]).
    pub fn from_midi_spelled_as(
        midi_note_value: u8,
        notes: &Vec<Note>,
    ) -> Result<Self, MusicSemanticsError> {
        let pc = Pc::from(midi_note_value % 12);
        match notes.iter().find(|note| Pc::from(*note) == pc) {
            Some(note) => Self::from_midi_as(midi_note_value, *note),
            None => Err(MusicSemanticsError::NotAMember(
                *pc.notes().first().unwrap(),
                notes.clone(),
            )),
        }
    }

    /// Subtract up or down from a pitch to arrive at another one.
    /// This does not control spelling.
    pub fn at_distance_from(&self, distance: isize) -> Result<Self, MusicSemanticsError> {
        let new_pitch = self.midi_note as isize + distance;
        let new_pitch = u8::try_from(new_pitch)
            .map_err(|_| MusicSemanticsError::OutOfBoundsLower(self.midi_note))?;
        Self::from_midi(new_pitch)
    }

    /// Compare pitches by their MIDI note, to equivocate over
    /// spellings but not octaves.
    pub fn is_same_frequency(&self, other: &Pitch) -> bool {
        self.midi_note == other.midi_note
    }

    /// Returns the next [Pitch] above [self] whose note is equivalent to
    /// to the input [Note], spelled as that note. For when you want to "go up
    /// to G from B3".
    pub fn up_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_up_to_note(note);
        Self::from_midi_as(self.midi_note + d, *note)
    }

    /// Returns the next [Pitch] below [self] whose note is equivalent to
    /// to the input [Note], spelled as that note. For when you want to "go
    /// down to G from B3".
    pub fn down_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_down_to_note(note);
        let midi_note = self
            .midi_note
            .checked_sub(d)
            .ok_or(MusicSemanticsError::OutOfBoundsLower(self.midi_note))?;
        Self::from_midi_as(midi_note, *note)
    }

    /// Returns the number of letters up/down between self and other,
    /// accounting for (written) octaves.
    pub fn diatonic_distance(&self, other: &Pitch) -> i32 {
        let self_diat = (self.octave as i32) * 7 + i32::from(&Spelling::from(&self.note).letter);
        let other_diat = (other.octave as i32) * 7 + i32::from(&Spelling::from(&other.note).letter);
        other_diat - self_diat
    }

    /// Shift a pitch by some number of written octaves, keeping its spelling
    /// (so the MIDI note moves by `12 * n`).
    pub fn raise_octaves(&self, n: isize) -> Result<Self, MusicSemanticsError> {
        let new_oct = i8::try_from(self.octave as isize + n)
            .map_err(|_| MusicSemanticsError::MidiTooHigh(u8::MAX))?;
        Self::try_new(self.note, new_oct)
    }
}

/// Prints the spelling followed by its written octave: `Cb4` is MIDI 59 and
/// `B#3` is MIDI 60.
impl Display for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.note, self.octave)
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.midi_note.partial_cmp(&other.midi_note)
    }
}

/// `(note, octave)` determines `midi_note`, so hashing those two fields agrees
/// with `Eq`.
impl Hash for Pitch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.note.hash(state);
        self.octave.hash(state);
    }
}

/// As this is meant to afford a shorthand syntax, this _will_ unwrap the pitch.
/// If that's not the behavior you want, use `Pitch::new` directly.
#[macro_export]
macro_rules! pitch {
    (bis, $octave:expr) => {
        Pitch::new(Note::Bis, $octave)
    };
    (c, $octave:expr) => {
        Pitch::new(Note::C, $octave)
    };
    (deses, $octave:expr) => {
        Pitch::new(Note::Deses, $octave)
    };
    (cis, $octave:expr) => {
        Pitch::new(Note::Cis, $octave)
    };
    (des, $octave:expr) => {
        Pitch::new(Note::Des, $octave)
    };
    (d, $octave:expr) => {
        Pitch::new(Note::D, $octave)
    };
    (cisis, $octave:expr) => {
        Pitch::new(Note::Cisis, $octave)
    };
    (eeses, $octave:expr) => {
        Pitch::new(Note::Eeses, $octave)
    };
    (dis, $octave:expr) => {
        Pitch::new(Note::Dis, $octave)
    };
    (ees, $octave:expr) => {
        Pitch::new(Note::Ees, $octave)
    };
    (e, $octave:expr) => {
        Pitch::new(Note::E, $octave)
    };
    (disis, $octave:expr) => {
        Pitch::new(Note::Disis, $octave)
    };
    (fes, $octave:expr) => {
        Pitch::new(Note::Fes, $octave)
    };
    (f, $octave:expr) => {
        Pitch::new(Note::F, $octave)
    };
    (eis, $octave:expr) => {
        Pitch::new(Note::Eis, $octave)
    };
    (geses, $octave:expr) => {
        Pitch::new(Note::Geses, $octave)
    };
    (fis, $octave:expr) => {
        Pitch::new(Note::Fis, $octave)
    };
    (ges, $octave:expr) => {
        Pitch::new(Note::Ges, $octave)
    };
    (g, $octave:expr) => {
        Pitch::new(Note::G, $octave)
    };
    (fisis, $octave:expr) => {
        Pitch::new(Note::Fisis, $octave)
    };
    (aeses, $octave:expr) => {
        Pitch::new(Note::Aeses, $octave)
    };
    (gis, $octave:expr) => {
        Pitch::new(Note::Gis, $octave)
    };
    (aes, $octave:expr) => {
        Pitch::new(Note::Aes, $octave)
    };
    (a, $octave:expr) => {
        Pitch::new(Note::A, $octave)
    };
    (gisis, $octave:expr) => {
        Pitch::new(Note::Gisis, $octave)
    };
    (beses, $octave:expr) => {
        Pitch::new(Note::Beses, $octave)
    };
    (ais, $octave:expr) => {
        Pitch::new(Note::Ais, $octave)
    };
    (bes, $octave:expr) => {
        Pitch::new(Note::Bes, $octave)
    };
    (b, $octave:expr) => {
        Pitch::new(Note::B, $octave)
    };
    (ces, $octave:expr) => {
        Pitch::new(Note::Ces, $octave)
    };
    (aisis, $octave:expr) => {
        Pitch::new(Note::Aisis, $octave)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_collections::spelling::HasSpelling;

    #[test]
    fn pitch_macro() {
        assert_eq!(Pitch::new(Note::C, 4), pitch!(c, 4));
    }

    #[test]
    fn from_midi_spelled_works() {
        // Middle C (MIDI 60) - natural note
        let c4_sharp = Pitch::from_midi_spelled(60, true).unwrap();
        let c4_flat = Pitch::from_midi_spelled(60, false).unwrap();
        assert_eq!(c4_sharp.note, Note::C);
        assert_eq!(c4_flat.note, Note::C);
        assert_eq!(c4_sharp.octave, 4);

        // C# / Db (MIDI 61)
        let cis4 = Pitch::from_midi_spelled(61, true).unwrap();
        let des4 = Pitch::from_midi_spelled(61, false).unwrap();
        assert_eq!(cis4.note, Note::Cis);
        assert_eq!(des4.note, Note::Des);

        // Bb (MIDI 70)
        let ais4 = Pitch::from_midi_spelled(70, true).unwrap();
        let bes4 = Pitch::from_midi_spelled(70, false).unwrap();
        assert_eq!(ais4.note, Note::Ais);
        assert_eq!(bes4.note, Note::Bes);

        // from_midi defaults to sharp spelling
        let default = Pitch::from_midi(61).unwrap();
        assert_eq!(default.note, Note::Cis);
    }

    #[test]
    fn diatonic_distance_works() {
        let p1 = Pitch::new(Note::C, 5);
        let p2 = Pitch::new(Note::B, 4);
        assert_eq!(p1.diatonic_distance(&p2), -1);
        let p2 = Pitch::new(Note::B, 5);
        assert_eq!(p1.diatonic_distance(&p2), 6);
        let p2 = Pitch::new(Note::B, 6);
        assert_eq!(p1.diatonic_distance(&p2), 13);
        let p1 = Pitch::new(Note::G, 3);
        let p2 = Pitch::new(Note::F, 3);
        assert_eq!(p1.diatonic_distance(&p2), -1);
        let p2 = Pitch::new(Note::F, 4);
        assert_eq!(p1.diatonic_distance(&p2), 6);
    }

    #[test]
    fn try_new_rejects_high_octave() {
        assert!(Pitch::try_new(Note::C, 10).is_err());
        assert!(Pitch::try_new(Note::C, 9).is_ok()); // C9 = MIDI 120
        assert!(Pitch::try_new(Note::C, 8).is_ok());
    }

    #[test]
    fn try_new_accepts_negative_one() {
        let p = Pitch::try_new(Note::C, -1).unwrap();
        assert_eq!(p.midi_note, 0); // C-1 = MIDI 0
        assert_eq!(p.octave, -1);
    }

    #[test]
    fn try_new_rejects_below_negative_one() {
        assert!(Pitch::try_new(Note::C, -2).is_err());
    }

    #[test]
    #[should_panic(expected = "pitch octave out of range")]
    fn new_panics_on_high_octave() {
        let _ = Pitch::new(Note::C, 10);
    }

    /// All spellings, including the two (C♭, B♯) whose written octave differs
    /// from the sounding pitch's MIDI C..B block.
    const ALL_NOTES: [Note; 31] = [
        Note::C,
        Note::Deses,
        Note::Cis,
        Note::Des,
        Note::Cisis,
        Note::D,
        Note::Eeses,
        Note::Dis,
        Note::Ees,
        Note::Disis,
        Note::E,
        Note::Fes,
        Note::Eis,
        Note::F,
        Note::Geses,
        Note::Fis,
        Note::Ges,
        Note::Fisis,
        Note::G,
        Note::Aeses,
        Note::Gis,
        Note::Aes,
        Note::Gisis,
        Note::A,
        Note::Beses,
        Note::Ais,
        Note::Bes,
        Note::Aisis,
        Note::B,
        Note::Ces,
        Note::Bis,
    ];

    #[test]
    fn c_flat_and_b_sharp_use_the_written_octave() {
        let c_flat_4 = Pitch::new(Note::Ces, 4);
        assert_eq!(c_flat_4.midi_note, 59); // sounds B3
        assert_eq!(Pitch::new(Note::Ces, 5).midi_note, 71); // sounds B4
        let b_sharp_3 = Pitch::new(Note::Bis, 3);
        assert_eq!(b_sharp_3.midi_note, 60); // sounds C4
        assert_eq!(Pitch::new(Note::Bis, 4).midi_note, 72); // sounds C5

        // Sounding order follows the written spelling across the C boundary.
        assert!(c_flat_4 < Pitch::new(Note::C, 4));
        assert!(c_flat_4.is_same_frequency(&Pitch::new(Note::B, 3)));
        assert!(b_sharp_3 > Pitch::new(Note::B, 3));
        assert!(b_sharp_3.is_same_frequency(&Pitch::new(Note::C, 4)));
    }

    #[test]
    fn display_prints_the_written_octave() {
        assert_eq!(Pitch::new(Note::Ces, 4).to_string(), "Cb4");
        assert_eq!(Pitch::new(Note::Bis, 3).to_string(), "B#3");
        let from_midi_ces = Pitch::from_midi_spelled_as(59, &vec![Note::Ces]).unwrap();
        assert_eq!(from_midi_ces.to_string(), "Cb4");
        let from_midi_bis = Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap();
        assert_eq!(from_midi_bis.to_string(), "B#3");
    }

    #[test]
    fn midi_formula_and_midi_spelling_agree_for_every_spelling() {
        for note in ALL_NOTES {
            let spelling = Spelling::from(&note);
            for octave in -1i8..=9 {
                let expected = (i16::from(octave) + 1) * 12
                    + i16::from(spelling.letter.natural_semitone())
                    + i16::from(spelling.acc.alteration());
                match Pitch::try_new(note, octave) {
                    Ok(pitch) => {
                        assert_eq!(i16::from(pitch.midi_note), expected, "{note:?}{octave}");
                        assert_eq!(
                            Pitch::from_midi_as(pitch.midi_note, note).unwrap(),
                            pitch,
                            "MIDI {} spelled {note:?}",
                            pitch.midi_note
                        );
                        let pc = Pc::from(pitch.midi_note % 12);
                        assert_eq!(pc, Pc::from(&note), "{note:?}{octave} pitch class");
                    }
                    Err(_) => assert!(
                        !(0..=127).contains(&expected),
                        "{note:?}{octave} (MIDI {expected}) must be constructible"
                    ),
                }
            }
        }
    }

    #[test]
    fn from_midi_spelled_as_derives_octave_from_the_chosen_spelling() {
        let ces = Pitch::from_midi_spelled_as(59, &vec![Note::Ces]).unwrap();
        assert_eq!((ces.note, ces.octave, ces.midi_note), (Note::Ces, 4, 59));
        let ces5 = Pitch::from_midi_spelled_as(71, &vec![Note::Ces]).unwrap();
        assert_eq!((ces5.note, ces5.octave, ces5.midi_note), (Note::Ces, 5, 71));
        let bis = Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap();
        assert_eq!((bis.note, bis.octave, bis.midi_note), (Note::Bis, 3, 60));
        // The first palette entry with the right pitch class wins.
        let b = Pitch::from_midi_spelled_as(59, &vec![Note::B, Note::Ces]).unwrap();
        assert_eq!((b.note, b.octave), (Note::B, 3));
        let c = Pitch::from_midi_spelled_as(60, &vec![Note::C, Note::Bis]).unwrap();
        assert_eq!((c.note, c.octave), (Note::C, 4));
    }

    #[test]
    fn from_midi_as_rejects_mismatched_or_unrepresentable_spellings() {
        assert!(Pitch::from_midi_as(60, Note::Ces).is_err());
        assert!(Pitch::from_midi_as(128, Note::Gis).is_err());
        // MIDI 0 as B♯ would need octave -2.
        assert!(Pitch::from_midi_as(0, Note::Bis).is_err());
        assert_eq!(Pitch::from_midi_as(0, Note::C).unwrap().octave, -1);
        // MIDI 11 as C♭ is C♭0.
        assert_eq!(Pitch::from_midi_as(11, Note::Ces).unwrap().octave, 0);
        assert!(Pitch::from_midi(128).is_err());
        assert_eq!(Pitch::from_midi(127).unwrap(), Pitch::new(Note::G, 9));
    }

    #[test]
    fn try_new_rejects_spellings_outside_midi_range() {
        // C♭-1 would be MIDI -1.
        assert!(Pitch::try_new(Note::Ces, -1).is_err());
        assert_eq!(Pitch::try_new(Note::Ces, 0).unwrap().midi_note, 11);
        assert_eq!(Pitch::try_new(Note::Bis, -1).unwrap().midi_note, 12);
        // B♯9 would be MIDI 132; B♯8 is MIDI 120.
        assert!(Pitch::try_new(Note::Bis, 9).is_err());
        assert_eq!(Pitch::try_new(Note::Bis, 8).unwrap().midi_note, 120);
        assert_eq!(Pitch::try_new(Note::Ces, 9).unwrap().midi_note, 119);
    }

    #[test]
    fn up_and_down_to_note_spell_the_arrival_in_its_written_octave() {
        // B4 (71) "up to C♭" is the same sound respelled: C♭5.
        let b4 = Pitch::new(Note::B, 4);
        assert_eq!(b4.up_to_note(&Note::Ces).unwrap(), Pitch::new(Note::Ces, 5));
        assert_eq!(
            Pitch::new(Note::Bes, 4).up_to_note(&Note::Ces).unwrap(),
            Pitch::new(Note::Ces, 5)
        );
        // A3 (57) up to B♯ lands on MIDI 60, written B♯3.
        assert_eq!(
            Pitch::new(Note::A, 3).up_to_note(&Note::Bis).unwrap(),
            Pitch::new(Note::Bis, 3)
        );
        // D4 (62) down to B♯ lands on MIDI 60, written B♯3.
        assert_eq!(
            Pitch::new(Note::D, 4).down_to_note(&Note::Bis).unwrap(),
            Pitch::new(Note::Bis, 3)
        );
        // D5 (74) down to C♭ lands on MIDI 71, written C♭5.
        assert_eq!(
            Pitch::new(Note::D, 5).down_to_note(&Note::Ces).unwrap(),
            Pitch::new(Note::Ces, 5)
        );
    }

    #[test]
    fn respelling_keeps_midi_and_rederives_the_written_octave() {
        let b4 = Pitch::new(Note::B, 4);
        let ces5 = b4.spelled_as_in(&vec![Note::Ces]).unwrap();
        assert_eq!((ces5.note, ces5.octave, ces5.midi_note), (Note::Ces, 5, 71));
        let c4 = Pitch::new(Note::C, 4);
        let bis3 = c4.spelled_as_in(&vec![Note::Bis]).unwrap();
        assert_eq!((bis3.note, bis3.octave, bis3.midi_note), (Note::Bis, 3, 60));
        // And back again.
        assert_eq!(bis3.spelled_as_in(&vec![Note::C]).unwrap(), c4);
    }

    #[test]
    fn diatonic_distance_counts_written_letters() {
        let bis3 = Pitch::new(Note::Bis, 3);
        assert_eq!(bis3.diatonic_distance(&Pitch::new(Note::C, 4)), 1);
        assert_eq!(bis3.diatonic_distance(&Pitch::new(Note::B, 3)), 0);
        let ces5 = Pitch::new(Note::Ces, 5);
        assert_eq!(Pitch::new(Note::B, 4).diatonic_distance(&ces5), 1);
        assert_eq!(ces5.diatonic_distance(&Pitch::new(Note::C, 5)), 0);
    }

    #[test]
    fn raise_octaves_keeps_spelling_and_moves_midi_by_twelve() {
        let ces4 = Pitch::new(Note::Ces, 4);
        let up = ces4.raise_octaves(1).unwrap();
        assert_eq!((up.note, up.octave, up.midi_note), (Note::Ces, 5, 71));
        let bis3 = Pitch::new(Note::Bis, 3);
        let down = bis3.raise_octaves(-1).unwrap();
        assert_eq!((down.note, down.octave, down.midi_note), (Note::Bis, 2, 48));
    }
}
