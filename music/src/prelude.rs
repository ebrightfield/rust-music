//! Convenience re-exports for the most commonly used items in the crate.
//!
//! Glob-importing this module gives you the primitives, collections, traits,
//! macros, and error type needed for typical music-theory work without
//! having to know the internal module layout:
//!
//! ```
//! use music::prelude::*;
//!
//! let c_major = PcShape::new(vec![pc!(0), pc!(4), pc!(7)]);
//! let spelled = c_major.try_spell(&Note::C).unwrap();
//! ```

// Primitive pitch types.
pub use crate::note::pitch::MIDDLE_C;
pub use crate::note::pitch_class::PcIter;
pub use crate::note::spelling::{Accidental, Letter};
pub use crate::note::{Note, Pc, Pitch, Spelling};

// Collections.
pub use crate::note_collections::voicing::{StackedIntervals, Voicing};
pub use crate::note_collections::{
    AsPcSlice, IntervalClass, NoteSet, OctavePartition, PcContent, PcShape,
};

// Spelling / transposition traits. These are almost always needed to call
// methods on the collection types.
pub use crate::note_collections::geometry::symmetry::transpositional::{
    Modes, Transpose, TryTranspose,
};
pub use crate::note_collections::spelling::HasSpelling;

// Chord naming.
pub use crate::note_collections::chord_name::parsing::parse_chord_name;
pub use crate::note_collections::chord_name::{
    ChordName, ChordNameDisplayConfig, ChordQuality, ExtensionStyle, MajNotation, NamingConfig,
    TonalSpecification,
};

// Fretboard primitives (the crate's other big domain).
pub use crate::fretboard::{
    ChordShapeClassification, Fretboard, FretboardShape, FrettedNote, SoundedNote, StringConvention,
};
pub use crate::fretboard::{BASS_4, BASS_5, DADGAD, DROP_D, OPEN_G, STANDARD_7, STD_6STR_GTR};

// Error type.
pub use crate::error::MusicSemanticsError;

// Macros. #[macro_export] puts these at the crate root, so re-export from there.
pub use crate::{content, pc, pc_shape, pitch, voicing};
