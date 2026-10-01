pub mod command;
pub mod common_types;
pub mod document;
pub mod error;
pub mod fretboard_diagram;
pub mod scoring;
pub mod staff_elements;
pub mod templates;

pub trait ToLilypondString {
    fn to_lilypond_string(&self) -> String;
}
