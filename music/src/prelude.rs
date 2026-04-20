//! Convenience re-exports for the most commonly used items in the crate.
//!
//! Glob-importing this module gives you the primitives, collections, traits,
//! macros, and error type needed for typical music-theory work without
//! having to know the internal module layout:
//!
//! ```
//! use music::prelude::*;
//!
//! let c_major = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);
//! let spelled = c_major.try_spell(&Note::C).unwrap();
//! ```

// Primitive pitch types.
pub use crate::note::{Note, Pitch, Pc, Spelling};
pub use crate::note::pitch::MIDDLE_C;
pub use crate::note::pitch_class::PcIter;
pub use crate::note::spelling::{Accidental, Letter};

// Collections.
pub use crate::note_collections::{NoteSet, PcSet, IntervalClass, OctavePartition};
pub use crate::note_collections::voicing::{Voicing, StackedIntervals};

// Spelling / transposition traits. These are almost always needed to call
// methods on the collection types.
pub use crate::note_collections::spelling::HasSpelling;
pub use crate::note_collections::geometry::symmetry::transpositional::{
    Modes, Transpose, TryTranspose,
};

// Chord naming.
pub use crate::note_collections::chord_name::{
    ChordName, ChordNameDisplayConfig, ChordQuality, ExtensionStyle, MajNotation, NamingConfig,
    TonalSpecification,
};
pub use crate::note_collections::chord_name::parsing::parse_chord_name;

// Fretboard primitives (the crate's other big domain).
pub use crate::fretboard::{
    Fretboard, FretboardShape, FrettedNote, SoundedNote, StringConvention,
    ChordShapeClassification,
};
pub use crate::fretboard::{STD_6STR_GTR, DROP_D, DADGAD, OPEN_G, STANDARD_7, BASS_4, BASS_5};

// Error type.
pub use crate::error::MusicSemanticsError;

// Macros. #[macro_export] puts these at the crate root, so re-export from there.
pub use crate::{pc, pcs, pitch, voicing, validated_pcs};
