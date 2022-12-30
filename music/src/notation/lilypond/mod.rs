pub mod scoring;
pub mod staff_elements;
pub mod templates;
pub mod fretboard_diagram;
pub mod command;
pub mod document;
pub mod common_types;
pub mod error;


pub trait ToLilypondString {
    fn to_lilypond_string(&self) -> String;
}
