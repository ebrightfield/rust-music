#![feature(concat_idents)]
pub mod note_collections;
pub mod note;
pub mod fretboard;
pub mod error;
pub mod notation;

pub use note::{Note, Pitch, Pc, Spelling};
pub use note_collections::*;
pub use fretboard::*;

pub mod common_chords {
    //use super::*;

    // pub fn major7(note: &Note) -> NoteSet {
    //
    // }
}