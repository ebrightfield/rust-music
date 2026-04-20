//! Compact ASCII renderers for LLM-friendly output.
//!
//! These formats prioritize token efficiency over visual grid layout.
//! For pixel-accurate chord diagrams, see [`crate::svg::fretboard`].
//!
//! # Example
//! ```
//! use music::fretboard::STD_6STR_GTR;
//! use music::fretboard::fretboard_shape::FretboardShape;
//! use music::ascii::ToAsciiFretboard;
//!
//! let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
//! assert_eq!(shape.to_ascii().fret_spec(), "x-3-2-0-1-0");
//! ```

pub mod fretboard;

pub use fretboard::{AsciiFretboardBuilder, ToAsciiFretboard};
