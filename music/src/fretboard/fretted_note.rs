use std::fmt::{Display, Formatter};
use crate::error::MusicSemanticsError;
use crate::fretboard::{Fretboard, StringConvention};
use crate::note::note::Note;
use crate::note::pitch::Pitch;
use crate::note_collections::NoteSet;

/// This enum is useful for times when you need to be able to mark a string as muted,
/// for example when creating guitar chord diagrams.
/// But if you're never going to have to notated a muted string,
/// it is better to use a [SoundedNote] directly instead.
#[derive(Debug, Clone, PartialEq)]
pub enum FrettedNote<'a> {
    /// A note that is played on the fretboard.
    Sounded(SoundedNote<'a>),
    /// Denotes a muted string. Usually most useful for chord diagrams.
    Muted {
        string: u8,
        fretboard: &'a Fretboard,
    },
}

/// A note played on a fretboard. A reference to the fretboard ensures
/// that each existing [FrettedNote] instance refers in code back to an
/// actual [Fretboard] instance, which is often useful for performing calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a Fretboard,
}

impl<'a> Display for SoundedNote<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}({})", self.string+1, self.fret, self.pitch.note)
    }
}

impl<'a> SoundedNote<'a> {
    /// Preferred constructor for an open string. Validates using
    /// the methods on the [Fretboard] passed in.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self {
            fret: Fretboard::OPEN,
            pitch: open_string.clone(),
            string,
            fretboard,
        })
    }

    /// Preferred constructor for a fretted string. Validates using
    /// the methods on the [Fretboard] passed in.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        Ok(fretboard.sounded_note(string, fret)?)
    }

    /// Returns a clone of self, but with the pitch spelled according
    /// to a vec of [Note].
    /// This is a way to assert a musical context (i.e. "correct" spelling)
    /// over `self.pitch`.
    pub fn spelled_as_in(&self, notes: &Vec<Note>) -> Result<Self, MusicSemanticsError> {
        let pitch = Pitch::new_spelled_as_in(self.pitch.midi_note, notes)?;
        Ok(Self {
            string: self.string,
            fret: self.fret,
            pitch,
            fretboard: self.fretboard,
        })
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn up_n_frets(&self, n: u8) -> Result<Self, MusicSemanticsError> {
        self.fretboard.sounded_note(self.string, self.fret + n)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn down_n_frets(&self, n: u8) -> Result<Self, MusicSemanticsError> {
        if n < self.fret {
            return Err(MusicSemanticsError::CantMoveDownFrets(n));
        }
        Ok(self.fretboard.sounded_note(self.string, self.fret - n)?)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn up_an_octave(&self) -> Result<Self, MusicSemanticsError> {
        self.fretboard.sounded_note(self.string, self.fret + 12)
    }

    /// Moves down the same string 12 semitones, if possible.
    pub fn down_an_octave(&self) -> Result<Self, MusicSemanticsError> {
        if self.fret < 12 {
            return Err(MusicSemanticsError::CantMoveDownFrets(12));
        }
        Ok(self.fretboard.sounded_note(self.string, self.fret - 12)?)
    }

    /// Produces a [SoundedNote] on the next chord/scale degree, on the same string.
    pub fn next_note_same_string(&self, notes: &NoteSet) -> Result<Self, MusicSemanticsError> {
        let next_note = notes.up_n_steps(&self.pitch.note, 1)?;
        let pitch = self.pitch.up_to_note(&next_note)?;
        let this_string = self.fretboard.get_string(self.string).unwrap();
        let fret = pitch.midi_note - this_string.midi_note;
        self.fretboard.sounded_note(self.string, fret)
    }

    /// Produces a [SoundedNote] on the next chord/scale degree, on the next string up.
    pub fn next_note_next_string(&self, notes: &NoteSet) -> Result<Self, MusicSemanticsError> {
        let next_note = notes.up_n_steps(&self.pitch.note, 1)?;
        let pitch = self.pitch.up_to_note(&next_note)?;
        let next_string = self.fretboard.get_string(self.string + 1)?;
        if next_string.midi_note > pitch.midi_note {
            return Err(MusicSemanticsError::FretBelowZero(pitch, next_string.clone()));
        }
        self.fretboard.sounded_note(
            self.string + 1, pitch.midi_note - next_string.midi_note
        )
    }

    /// Move to the same pitch class on the next higher (numerically larger) string.
    /// On a standard guitar, this moves toward the thinner strings.
    /// Returns an error if already on the highest string or if the fret would be negative.
    pub fn same_note_higher_string(&self) -> Result<Self, MusicSemanticsError> {
        let target_string = self.string + 1;
        if target_string >= self.fretboard.num_strings() {
            return Err(MusicSemanticsError::StringTooHighForFretboard(
                target_string,
                self.fretboard.clone()
            ));
        }

        let higher_string_pitch = self.fretboard.get_string(target_string)?;

        // Calculate fret needed on higher string to get same pitch
        if self.pitch.midi_note < higher_string_pitch.midi_note {
            return Err(MusicSemanticsError::FretBelowZero(
                self.pitch.clone(),
                higher_string_pitch.clone()
            ));
        }

        let fret = self.pitch.midi_note - higher_string_pitch.midi_note;
        if fret > Fretboard::MAX {
            return Err(MusicSemanticsError::FretTooHigh(fret));
        }

        self.fretboard.sounded_note(target_string, fret)
    }

    /// Move to the same pitch class on the next lower (numerically smaller) string.
    /// On a standard guitar, this moves toward the thicker strings.
    /// Returns an error if already on the lowest string or if the fret would exceed MAX.
    pub fn same_note_lower_string(&self) -> Result<Self, MusicSemanticsError> {
        if self.string == 0 {
            // Already on lowest string, can't go lower
            // Return a fret-below-zero error with the current pitch and lowest string
            let lowest = self.fretboard.get_string(0)?;
            return Err(MusicSemanticsError::FretBelowZero(
                self.pitch.clone(),
                lowest.clone()
            ));
        }

        let target_string = self.string - 1;
        let lower_string_pitch = self.fretboard.get_string(target_string)?;

        // Calculate fret needed on lower string to get same pitch
        if self.pitch.midi_note < lower_string_pitch.midi_note {
            return Err(MusicSemanticsError::FretBelowZero(
                self.pitch.clone(),
                lower_string_pitch.clone()
            ));
        }

        let fret = self.pitch.midi_note - lower_string_pitch.midi_note;
        if fret > Fretboard::MAX {
            return Err(MusicSemanticsError::FretTooHigh(fret));
        }

        self.fretboard.sounded_note(target_string, fret)
    }

    /// Find all positions on the fretboard where this exact pitch can be played.
    /// Returns a vector of SoundedNote instances at the same pitch across all strings.
    pub fn all_positions(&self) -> Vec<Self> {
        let mut positions = Vec::new();

        for string in 0..self.fretboard.num_strings() {
            if let Ok(open_pitch) = self.fretboard.get_string(string) {
                // Check if the pitch is playable on this string
                if self.pitch.midi_note >= open_pitch.midi_note {
                    let fret = self.pitch.midi_note - open_pitch.midi_note;
                    if fret <= Fretboard::MAX {
                        if let Ok(pos) = self.fretboard.sounded_note(string, fret) {
                            positions.push(pos);
                        }
                    }
                }
            }
        }

        positions
    }

    /// Find all positions where any octave of this pitch class can be played.
    /// Includes positions at octave up and octave down from the current pitch.
    pub fn all_octave_positions(&self) -> Vec<Self> {
        use crate::note::pitch_class::Pc;
        let target_pc = Pc::from(&self.pitch.note);
        let mut positions = Vec::new();

        for string in 0..self.fretboard.num_strings() {
            if let Ok(open_pitch) = self.fretboard.get_string(string) {
                let open_pc = Pc::from(&open_pitch.note);
                let base_fret = open_pc.distance_up_to(&target_pc);

                // Check this position and octave above
                for offset in [0, 12, 24] {
                    let fret = base_fret + offset;
                    if fret <= Fretboard::MAX {
                        if let Ok(pos) = self.fretboard.sounded_note(string, fret) {
                            positions.push(pos);
                        }
                    }
                }
            }
        }

        positions
    }

    /// Get the string number in the specified convention.
    ///
    /// # Arguments
    /// * `convention` - The string numbering convention to use
    ///
    /// # Returns
    /// The string number in the requested convention.
    ///
    /// # Example
    /// ```
    /// use music::fretboard::{STD_6STR_GTR, StringConvention};
    ///
    /// // String 0 (low E) in internal convention
    /// let note = STD_6STR_GTR.sounded_note(0, 5).unwrap();
    ///
    /// // Internal: 0
    /// assert_eq!(note.string_number(StringConvention::ZeroIndexedFromLow), 0);
    ///
    /// // Lilypond/TAB: 6 (6th string from top = low E)
    /// assert_eq!(note.string_number(StringConvention::OneIndexedFromHigh), 6);
    ///
    /// // One-indexed from low: 1
    /// assert_eq!(note.string_number(StringConvention::OneIndexedFromLow), 1);
    /// ```
    pub fn string_number(&self, convention: StringConvention) -> u8 {
        let num_strings = self.fretboard.num_strings();
        match convention {
            StringConvention::ZeroIndexedFromLow => self.string,
            StringConvention::OneIndexedFromHigh => num_strings - self.string,
            StringConvention::OneIndexedFromLow => self.string + 1,
        }
    }

    /// Get the Lilypond string number (1 = highest string, same as TAB notation).
    /// This is a convenience method equivalent to `string_number(StringConvention::OneIndexedFromHigh)`.
    pub fn lilypond_string_number(&self) -> u8 {
        self.string_number(StringConvention::OneIndexedFromHigh)
    }
}

impl<'a> FrettedNote<'a> {

    /// Constructor for a [FrettedNote::Muted] variant.
    pub fn muted(string: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        let _ = fretboard.get_string(string)?;
        Ok(Self::Muted {
            string,
            fretboard,
        })
    }

    /// Constructor for a [FrettedNote::Sounded] variant of an open string.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self::Sounded(SoundedNote {
            pitch: open_string.clone(),
            string,
            fretboard,
            fret: Fretboard::OPEN,
        }))
    }

    /// Construct a [FrettedNote::Sounded] variant that is fretted.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        Ok(Self::Sounded(fretboard.sounded_note(string, fret)?))
    }

    /// Returns a clone of self, but with the pitch spelled according
    /// to a vec of [Note].
    /// This is a way to assert a musical context (i.e. "correct" spelling)
    /// over `self.pitch`.
    pub fn spelled_as_in(&self, notes: &Vec<Note>) -> Result<Self, MusicSemanticsError> {
        Ok(match &self {
            FrettedNote::Sounded(sounded_note) => FrettedNote::Sounded(
                sounded_note.spelled_as_in(notes)?
            ),
            FrettedNote::Muted { .. } => self.clone(),
        })
    }

    /// Returns the string value of either variant.
    pub fn string(&self) -> u8 {
        match &self {
            FrettedNote::Sounded(SoundedNote { string, ..}) => *string,
            FrettedNote::Muted { string, .. } => *string,
        }
    }

    /// Returns the fret, unless it's a [FrettedNote::Muted] variant.
    pub fn fret(&self) -> Option<u8> {
        match &self {
            FrettedNote::Sounded(SoundedNote { fret, ..}) => Some(*fret),
            FrettedNote::Muted { .. } => None
        }
    }

    /// Returns the pitch, unless it's a [FrettedNote::Muted] variant.
    pub fn pitch(&self) -> Option<Pitch> {
        match &self {
            FrettedNote::Sounded(SoundedNote { pitch, ..}) => Some(pitch.clone()),
            FrettedNote::Muted { .. } => None
        }
    }

    pub fn is_sounded(&self) -> bool {
        match &self {
            FrettedNote::Sounded(_) => true,
            FrettedNote::Muted { .. } => false,
        }
    }

    /// Get the string number in the specified convention.
    ///
    /// See [`SoundedNote::string_number`] for more details on conventions.
    pub fn string_number(&self, convention: StringConvention) -> u8 {
        let (string, fretboard) = match self {
            FrettedNote::Sounded(SoundedNote { string, fretboard, .. }) => (*string, *fretboard),
            FrettedNote::Muted { string, fretboard } => (*string, *fretboard),
        };
        let num_strings = fretboard.num_strings();
        match convention {
            StringConvention::ZeroIndexedFromLow => string,
            StringConvention::OneIndexedFromHigh => num_strings - string,
            StringConvention::OneIndexedFromLow => string + 1,
        }
    }

    /// Get the Lilypond string number (1 = highest string, same as TAB notation).
    /// This is a convenience method equivalent to `string_number(StringConvention::OneIndexedFromHigh)`.
    pub fn lilypond_string_number(&self) -> u8 {
        self.string_number(StringConvention::OneIndexedFromHigh)
    }
}

/// Wrap a [SoundedNote] in a [FrettedNote::Sounded].
impl<'a> From<SoundedNote<'a>> for FrettedNote<'a> {
    fn from(value: SoundedNote<'a>) -> Self {
        FrettedNote::Sounded(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fretboard::STD_6STR_GTR;

    #[test]
    fn test_same_note_higher_string() {
        // G on 6th string (E) at fret 3
        let g_on_low_e = STD_6STR_GTR.sounded_note(0, 3).unwrap();
        assert_eq!(g_on_low_e.pitch.note, Note::G); // G natural at fret 3 on E string

        // Move to next string - should be same pitch at a lower fret
        // A string (string 1) is 5 semitones higher than E, so G would be at fret 3 - 5 = -2
        // This should fail because fret would be negative
        let result = g_on_low_e.same_note_higher_string();
        assert!(result.is_err());

        // A at 6th string fret 5 should work - A on A string is open (fret 0)
        let a_on_low_e = STD_6STR_GTR.sounded_note(0, 5).unwrap();
        let a_on_a_string = a_on_low_e.same_note_higher_string().unwrap();
        assert_eq!(a_on_a_string.string, 1);
        assert_eq!(a_on_a_string.fret, 0);
        assert_eq!(a_on_a_string.pitch.midi_note, a_on_low_e.pitch.midi_note);
    }

    #[test]
    fn test_same_note_lower_string() {
        // G on 3rd string (G) open
        let g_open = STD_6STR_GTR.sounded_note(3, 0).unwrap();

        // Move to string 2 (D string) - G on D string is fret 5
        let g_on_d = g_open.same_note_lower_string().unwrap();
        assert_eq!(g_on_d.string, 2);
        assert_eq!(g_on_d.fret, 5);
        assert_eq!(g_on_d.pitch.midi_note, g_open.pitch.midi_note);

        // Try to go lower from the lowest string - should fail
        let e_low = STD_6STR_GTR.sounded_note(0, 0).unwrap();
        assert!(e_low.same_note_lower_string().is_err());
    }

    #[test]
    fn test_all_positions() {
        // G4 (open G string) - should be playable on multiple strings
        let g_open = STD_6STR_GTR.sounded_note(3, 0).unwrap();
        let positions = g_open.all_positions();

        // G4 should be playable on:
        // - String 3 (G): fret 0
        // - String 2 (D): fret 5
        // - String 1 (A): fret 10
        // - String 0 (E): fret 15
        assert!(positions.len() >= 3);  // At least on G, D, and A strings

        // All positions should have the same MIDI note
        for pos in &positions {
            assert_eq!(pos.pitch.midi_note, g_open.pitch.midi_note);
        }
    }

    #[test]
    fn test_all_octave_positions() {
        // E on string 0, fret 0 (E3)
        let e_low = STD_6STR_GTR.sounded_note(0, 0).unwrap();
        let positions = e_low.all_octave_positions();

        // E should be playable at many positions across all octaves
        // Each string should have E at 0, 12, 24 (if within range)
        assert!(positions.len() >= 6);  // At least one per string

        // All positions should be E (pitch class 4 from C=0, but note is E)
        for pos in &positions {
            assert_eq!(pos.pitch.note, Note::E);
        }
    }

    #[test]
    fn test_same_note_higher_string_boundary() {
        // Test from the highest string - should fail
        let e_high = STD_6STR_GTR.sounded_note(5, 5).unwrap();  // A on high E string
        assert!(e_high.same_note_higher_string().is_err());
    }

    #[test]
    fn test_string_convention_zero_indexed_from_low() {
        // String 0 in our system is the low E (thickest)
        let low_e = STD_6STR_GTR.sounded_note(0, 0).unwrap();
        assert_eq!(low_e.string_number(StringConvention::ZeroIndexedFromLow), 0);

        // String 5 in our system is the high E (thinnest)
        let high_e = STD_6STR_GTR.sounded_note(5, 0).unwrap();
        assert_eq!(high_e.string_number(StringConvention::ZeroIndexedFromLow), 5);
    }

    #[test]
    fn test_string_convention_one_indexed_from_high() {
        // String 0 in our system (low E) is the 6th string in Lilypond/TAB
        let low_e = STD_6STR_GTR.sounded_note(0, 0).unwrap();
        assert_eq!(low_e.string_number(StringConvention::OneIndexedFromHigh), 6);

        // String 5 in our system (high E) is the 1st string in Lilypond/TAB
        let high_e = STD_6STR_GTR.sounded_note(5, 0).unwrap();
        assert_eq!(high_e.string_number(StringConvention::OneIndexedFromHigh), 1);

        // Also test the lilypond_string_number convenience method
        assert_eq!(low_e.lilypond_string_number(), 6);
        assert_eq!(high_e.lilypond_string_number(), 1);
    }

    #[test]
    fn test_string_convention_one_indexed_from_low() {
        // String 0 in our system is the 1st string in this convention
        let low_e = STD_6STR_GTR.sounded_note(0, 0).unwrap();
        assert_eq!(low_e.string_number(StringConvention::OneIndexedFromLow), 1);

        // String 5 in our system is the 6th string in this convention
        let high_e = STD_6STR_GTR.sounded_note(5, 0).unwrap();
        assert_eq!(high_e.string_number(StringConvention::OneIndexedFromLow), 6);
    }

    #[test]
    fn test_string_convention_fretted_note() {
        // Test StringConvention with FrettedNote enum
        let sounded = FrettedNote::fretted(2, 3, &STD_6STR_GTR).unwrap();
        assert_eq!(sounded.string_number(StringConvention::ZeroIndexedFromLow), 2);
        assert_eq!(sounded.string_number(StringConvention::OneIndexedFromHigh), 4);
        assert_eq!(sounded.string_number(StringConvention::OneIndexedFromLow), 3);

        let muted = FrettedNote::muted(0, &STD_6STR_GTR).unwrap();
        assert_eq!(muted.string_number(StringConvention::ZeroIndexedFromLow), 0);
        assert_eq!(muted.string_number(StringConvention::OneIndexedFromHigh), 6);
        assert_eq!(muted.lilypond_string_number(), 6);
    }
}