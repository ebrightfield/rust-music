use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::error::MusicSemanticsError;
use crate::note::spelling::Spelling;
use crate::note_collections::spelling::HasSpelling;

pub const MIDDLE_C: u8 = 60;

/// MIDI-compliant formula: Note + octave → MIDI note number.
/// Accepts i8 octave (-1..=9); caller must validate range first.
fn calc_midi_note(note: &Note, octave: &i8) -> u8 {
    // Safe: for octave in -1..=9, (octave+1) is 0..=10, *12 is 0..=120,
    // and max pc is 11, so max result is 131 — fits i16 but may exceed u8.
    // try_new validates the final value ≤ 127 before constructing Pitch.
    ((*octave as i16 + 1) * 12 + u8::from(&Pc::from(note)) as i16) as u8
}

/// [Note] with octave information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pitch {
    /// Associated [Note] instance.
    pub note: Note,
    /// The octave register. Middle C = C4. Range: -1 (MIDI 0–11) through 9.
    pub octave: i8,
    /// The associated MIDI note, which also serves as a good means of measurement in
    /// semitone space. Middle C = 60.
    pub midi_note: u8,
}

impl Pitch {
    /// Create a pitch, returning an error if the octave is out of MIDI range.
    ///
    /// Accepts octave -1 (MIDI 0–11) through 9 (up to G9 = MIDI 127).
    /// Prefer [`Pitch::new`] when `octave` is known at compile time.
    pub fn try_new(note: Note, octave: i8) -> Result<Self, MusicSemanticsError> {
        if !(-1..=9).contains(&octave) {
            return Err(MusicSemanticsError::OctaveTooHigh(octave.unsigned_abs()));
        }
        let midi_note = calc_midi_note(&note, &octave);
        if midi_note > 127 {
            return Err(MusicSemanticsError::MidiTooHigh(127));
        }
        Ok(Self {
            note,
            octave,
            midi_note,
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
    pub fn from_midi_spelled(midi_note_value: u8, prefer_sharp: bool) -> Result<Self, MusicSemanticsError> {
        // i16 intermediate avoids u8 underflow for MIDI 0–11 (octave -1).
        let octave: i8 = ((midi_note_value as i16 / 12) - 1) as i8;
        if !(-1..=9).contains(&octave) {
            return Err(MusicSemanticsError::MidiTooHigh(midi_note_value));
        }
        let pc = Pc::from(midi_note_value % 12);
        let note = if prefer_sharp {
            pc.default_sharp_spelling()
        } else {
            pc.default_flat_spelling()
        };
        Ok(Self {
            note,
            octave,
            midi_note: midi_note_value,
        })
    }

    /// Produce a pitch from a MIDI note value, choosing its spelling from a
    /// "palette" of candidate note values (enharmonic alternatives).
    ///
    /// Returns the first note in `notes` whose pitch class matches the MIDI note.
    pub fn from_midi_spelled_as(midi_note_value: u8, notes: &Vec<Note>) -> Result<Self, MusicSemanticsError> {
        // i16 intermediate avoids u8 underflow for MIDI 0–11 (octave -1).
        let octave: i8 = ((midi_note_value as i16 / 12) - 1) as i8;
        if !(-1..=9).contains(&octave) {
            return Err(MusicSemanticsError::OctaveTooHigh(octave.unsigned_abs()));
        }
        let pc = midi_note_value - ((octave + 1) as u8 * 12);
        let pc = Pc::from(&pc);
        for note in notes {
            if Pc::from(note) == pc {
                return Ok(Self {
                    note: *note,
                    octave,
                    midi_note: midi_note_value,
                });
            }
        }
        Err(MusicSemanticsError::NotAMember(
            *pc.notes().first().unwrap(),
            notes.clone())
        )
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
    /// to the input [Note]. For when you want to "go up to G from B3".
    pub fn up_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_up_to_note(note);
        self.at_distance_from(d as isize)?.spelled_as_in(&vec![*note])
    }

    /// Returns the next [Pitch] below [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go down to G from B3".
    pub fn down_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_down_to_note(note);
        self.at_distance_from(-(d as isize))?.spelled_as_in(&vec![*note])
    }

    /// Returns the number of letters up/down between self and other,
    /// accounting for octaves.
    pub fn diatonic_distance(&self, other: &Pitch) -> i32 {
        let self_diat = (self.octave as i32) * 7
            + i32::from(&Spelling::from(&self.note).letter);
        let other_diat = (other.octave as i32) * 7
            + i32::from(&Spelling::from(&other.note).letter);
        other_diat - self_diat
    }

    /// Shift a pitch by some number of octaves.
    pub fn raise_octaves(&self, n: isize) -> Result<Self, MusicSemanticsError> {
        let new_oct = i8::try_from(self.octave as isize + n)
            .map_err(|_| MusicSemanticsError::MidiTooHigh(u8::MAX))?;
        Self::try_new(self.note, new_oct)
    }
}

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
    (bis, $octave:expr) => { Pitch::new(Note::Bis, $octave)};
    (c, $octave:expr) => { Pitch::new(Note::C, $octave)};
    (deses, $octave:expr) => { Pitch::new(Note::Deses, $octave)};
    (cis, $octave:expr) => { Pitch::new(Note::Cis, $octave)};
    (des, $octave:expr) => { Pitch::new(Note::Des, $octave)};
    (d, $octave:expr) => { Pitch::new(Note::D, $octave)};
    (cisis, $octave:expr) => { Pitch::new(Note::Cisis, $octave)};
    (eeses, $octave:expr) => { Pitch::new(Note::Eeses, $octave)};
    (dis, $octave:expr) => { Pitch::new(Note::Dis, $octave)};
    (ees, $octave:expr) => { Pitch::new(Note::Ees, $octave)};
    (e, $octave:expr) => { Pitch::new(Note::E, $octave)};
    (disis, $octave:expr) => { Pitch::new(Note::Disis, $octave)};
    (fes, $octave:expr) => { Pitch::new(Note::Fes, $octave)};
    (f, $octave:expr) => { Pitch::new(Note::F, $octave)};
    (eis, $octave:expr) => { Pitch::new(Note::Eis, $octave)};
    (geses, $octave:expr) => { Pitch::new(Note::Geses, $octave)};
    (fis, $octave:expr) => { Pitch::new(Note::Fis, $octave)};
    (ges, $octave:expr) => { Pitch::new(Note::Ges, $octave)};
    (g, $octave:expr) => { Pitch::new(Note::G, $octave)};
    (fisis, $octave:expr) => { Pitch::new(Note::Fisis, $octave)};
    (aeses, $octave:expr) => { Pitch::new(Note::Aeses, $octave)};
    (gis, $octave:expr) => { Pitch::new(Note::Gis, $octave)};
    (aes, $octave:expr) => { Pitch::new(Note::Aes, $octave)};
    (a, $octave:expr) => { Pitch::new(Note::A, $octave)};
    (gisis, $octave:expr) => { Pitch::new(Note::Gisis, $octave)};
    (beses, $octave:expr) => { Pitch::new(Note::Beses, $octave)};
    (ais, $octave:expr) => { Pitch::new(Note::Ais, $octave)};
    (bes, $octave:expr) => { Pitch::new(Note::Bes, $octave)};
    (b, $octave:expr) => { Pitch::new(Note::B, $octave)};
    (ces, $octave:expr) => { Pitch::new(Note::Ces, $octave)};
    (aisis, $octave:expr) => { Pitch::new(Note::Aisis, $octave)};
}

#[cfg(test)]
mod tests {
    use super::*;

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
}