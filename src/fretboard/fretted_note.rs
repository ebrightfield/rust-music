use crate::fretboard::Fretboard;
use crate::note::pitch::Pitch;

/// Useful for times when we want to be able to mark a string as muted.
#[derive(Debug)]
pub enum FrettedNote<'a> {
    /// A note that is played on the fretboard.
    Sounded(SoundedNote<'a>),
    /// Denotes a muted string. Usually most useful for note_collections diagrams.
    Muted {
        string: u8,
        fretboard: &'a Fretboard,
    },
}

/// A note played on a fretboard. A reference to the fretboard ensures
/// that each existing [FrettedNote] instance refers in code back to an
/// actual [Fretboard] instance, which is often useful for performing calculations.
#[derive(Debug)]
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a Fretboard,
}

impl<'a> SoundedNote<'a> {
    /// Preferred constructor for an open string. Validates using
    /// the methods on the [Fretboard] passed in.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
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
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        Ok(fretboard.sounded_note(string, fret)?)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn up_n_frets(&self, n: u8) -> anyhow::Result<Self> {
        Ok(self.fretboard.sounded_note(self.string, self.fret + n)?)
    }

    // pub fn next_note_same_string(&self, notes: &NoteSet) -> Self {
    //     let note = self.pitch.note;
    //     let index = notes.iter().position(|n| *n == note);
    //
    // }
}

impl<'a> FrettedNote<'a> {

    /// Constructor for a [FrettedNote::Muted] variant.
    pub fn muted(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        let _ = fretboard.get_string(string)?;
        Ok(Self::Muted {
            string,
            fretboard,
        })
    }

    /// Constructor for a [FrettedNote::Sounded] variant of an open string.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self::Sounded(SoundedNote {
            pitch: open_string.clone(),
            string,
            fretboard,
            fret: Fretboard::OPEN,
        }))
    }

    /// Construct a [FrettedNote::Sounded] variant that is fretted.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        Ok(Self::Sounded(fretboard.sounded_note(string, fret)?))
    }

    /// Returns the [string] value of either variant.
    fn string(&self) -> u8 {
        match &self {
            FrettedNote::Sounded(SoundedNote { string, ..}) => *string,
            FrettedNote::Muted { string, .. } => *string,
        }
    }

    /// Returns the [fret], unless it's a [FrettedNote::Muted] variant.
    fn fret(&self) -> Option<u8> {
        match &self {
            FrettedNote::Sounded(SoundedNote { fret, ..}) => Some(*fret),
            FrettedNote::Muted { .. } => None
        }
    }

    /// Returns the [pitch], unless it's a [FrettedNote::Muted] variant.
    fn pitch(&self) -> Option<Pitch> {
        match &self {
            FrettedNote::Sounded(SoundedNote { pitch, ..}) => Some(pitch.clone()),
            FrettedNote::Muted { .. } => None
        }
    }

    /// A wrapped version of [SoundedNote::up_n_frets].
    /// If [self] is a [FrettedNote::Muted], then this function returns [Ok(None)],
    ///
    fn up_n_frets(&self, n: u8) -> anyhow::Result<Option<Self>> {
        match &self {
            FrettedNote::Sounded(sounded_note) => Ok(Some(
                FrettedNote::Sounded(sounded_note.up_n_frets(n)?)
            )),
            FrettedNote::Muted { .. } => Ok(None),
        }
    }
}
