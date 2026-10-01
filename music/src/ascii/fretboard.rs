//! Compact single-line ASCII renderings of fretboard shapes.
//!
//! Two rendering shapes are available, both optimized for LLM
//! consumption rather than visual chord diagrams:
//!
//! - **Fret-spec**: `x-3-2-0-1-0` — pure fret numbers, one per string.
//!   `x` marks a muted string. Matches the [`FretboardShape`] `Display`
//!   impl when using the default low-to-high ordering.
//! - **Position list**: `A:3 D:2 G:0 B:1 E2:0` — one token per sounded
//!   string, keyed by the open-string letter. Muted strings are
//!   omitted. Duplicate letters (e.g. the two E strings on standard
//!   tuning) get an ordinal suffix: `E1` low, `E2` high.
//!
//! Both shapes can optionally include the spelled note at each
//! sounded fret via [`AsciiFretboardBuilder::with_notes`].
//!
//! # Configuration
//!
//! The builder mirrors [`crate::svg::fretboard::FretboardBuilder`]
//! where it makes sense for a linear token stream:
//!
//! - [`with_notes`](AsciiFretboardBuilder::with_notes) / [`show_notes`](AsciiFretboardBuilder::show_notes) — note annotations
//! - [`string_names`](AsciiFretboardBuilder::string_names) — override default open-string letter labels
//! - [`string_convention`](AsciiFretboardBuilder::string_convention) — low-to-high vs TAB-style high-to-low ordering
//!
//! Visual/spatial options from the SVG side (orientation, start_fret,
//! num_frets, title, theme, fret markers) don't apply to a linear
//! token stream and are deliberately omitted.
//!
//! # Example
//! ```
//! use music::fretboard::{STD_6STR_GTR, StringConvention};
//! use music::fretboard::fretboard_shape::FretboardShape;
//! use music::ascii::ToAsciiFretboard;
//!
//! let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
//!
//! // Default: low-to-high.
//! assert_eq!(shape.to_ascii().fret_spec(), "x-3-2-0-1-0");
//!
//! // TAB / Lilypond convention: high-to-low.
//! assert_eq!(
//!     shape.to_ascii()
//!         .string_convention(StringConvention::OneIndexedFromHigh)
//!         .fret_spec(),
//!     "0-1-0-2-3-x",
//! );
//! ```

use crate::fretboard::fretboard_shape::FretboardShape;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};
use crate::fretboard::StringConvention;

/// Builder for ASCII fretboard renderings.
///
/// Obtained via [`ToAsciiFretboard::to_ascii`]. Call a terminal method
/// ([`fret_spec`](Self::fret_spec) or
/// [`position_list`](Self::position_list)) to produce the string.
#[derive(Clone, Debug)]
pub struct AsciiFretboardBuilder<'a> {
    shape: &'a FretboardShape<'a>,
    show_notes: bool,
    string_names: Option<Vec<String>>,
    convention: StringConvention,
}

impl<'a> AsciiFretboardBuilder<'a> {
    /// Create a new builder rendering `shape`.
    pub fn new(shape: &'a FretboardShape<'a>) -> Self {
        Self {
            shape,
            show_notes: false,
            string_names: None,
            convention: StringConvention::ZeroIndexedFromLow,
        }
    }

    /// Annotate each sounded fret with its spelled note, e.g.
    /// `3(C)` instead of `3`.
    pub fn with_notes(mut self) -> Self {
        self.show_notes = true;
        self
    }

    /// Explicitly toggle note display.
    pub fn show_notes(mut self, show: bool) -> Self {
        self.show_notes = show;
        self
    }

    /// Override the default open-string labels used by
    /// [`position_list`](Self::position_list).
    ///
    /// Provide one label per string, indexed low-to-high
    /// (matching the internal `ZeroIndexedFromLow` convention),
    /// regardless of the configured `string_convention`.
    pub fn string_names(mut self, names: Vec<String>) -> Self {
        self.string_names = Some(names);
        self
    }

    /// Set the string numbering / ordering convention.
    ///
    /// Controls the order tokens appear in `fret_spec` and
    /// `position_list`. Defaults to
    /// [`ZeroIndexedFromLow`](StringConvention::ZeroIndexedFromLow),
    /// which matches this library's internal ordering (low string
    /// first). Pass
    /// [`OneIndexedFromHigh`](StringConvention::OneIndexedFromHigh)
    /// for TAB/Lilypond-style output (high string first).
    pub fn string_convention(mut self, convention: StringConvention) -> Self {
        self.convention = convention;
        self
    }

    /// Render as `x-3-2-0-1-0` (or `x-3(C)-2(E)-...` with notes).
    pub fn fret_spec(&self) -> String {
        let indices = self.ordered_string_indices();

        let tokens: Vec<String> = indices
            .into_iter()
            .map(|i| match &self.shape.fretted_notes[i] {
                FrettedNote::Muted { .. } => "x".to_string(),
                FrettedNote::Sounded(SoundedNote { fret, pitch, .. }) => {
                    if self.show_notes {
                        format!("{}({})", fret, pitch.note)
                    } else {
                        fret.to_string()
                    }
                }
            })
            .collect();
        tokens.join("-")
    }

    /// Render as a space-separated list keyed by open-string letter:
    /// `A:3 D:2 G:0 B:1 E2:0`. Muted strings are omitted.
    pub fn position_list(&self) -> String {
        let labels = self.labels();
        let indices = self.ordered_string_indices();

        let tokens: Vec<String> = indices
            .into_iter()
            .filter_map(|i| match &self.shape.fretted_notes[i] {
                FrettedNote::Muted { .. } => None,
                FrettedNote::Sounded(SoundedNote { fret, pitch, .. }) => {
                    if self.show_notes {
                        Some(format!("{}:{}({})", labels[i], fret, pitch.note))
                    } else {
                        Some(format!("{}:{}", labels[i], fret))
                    }
                }
            })
            .collect();
        tokens.join(" ")
    }

    /// Resolve labels: either the user-supplied overrides or derived
    /// from the tuning's open-string letters (with duplicate
    /// disambiguation). Always indexed low-to-high.
    fn labels(&self) -> Vec<String> {
        if let Some(names) = &self.string_names {
            return names.clone();
        }
        derive_string_labels(self.shape)
    }

    /// Produce the sequence of string indices (internal
    /// low-to-high indexing) in the order they should appear
    /// in the rendered output.
    fn ordered_string_indices(&self) -> Vec<usize> {
        let n = self.shape.fretted_notes.len();
        match self.convention {
            StringConvention::ZeroIndexedFromLow | StringConvention::OneIndexedFromLow => {
                (0..n).collect()
            }
            StringConvention::OneIndexedFromHigh => (0..n).rev().collect(),
        }
    }
}

/// Extension trait for types renderable as compact ASCII.
pub trait ToAsciiFretboard<'a> {
    /// Start building an ASCII rendering of this shape.
    fn to_ascii(&'a self) -> AsciiFretboardBuilder<'a>;
}

impl<'a> ToAsciiFretboard<'a> for FretboardShape<'a> {
    fn to_ascii(&'a self) -> AsciiFretboardBuilder<'a> {
        AsciiFretboardBuilder::new(self)
    }
}

/// Produce per-string labels from open-string letters, disambiguating
/// duplicates with an ordinal suffix (`E1`, `E2`, ...). Always
/// returned in internal (low-to-high) order.
fn derive_string_labels(shape: &FretboardShape) -> Vec<String> {
    let letters: Vec<String> = shape
        .fretboard
        .open_strings
        .iter()
        .map(|p| p.note.to_string())
        .collect();

    let mut counts = std::collections::HashMap::<String, usize>::new();
    for l in &letters {
        *counts.entry(l.clone()).or_insert(0) += 1;
    }

    let mut seen = std::collections::HashMap::<String, usize>::new();
    letters
        .iter()
        .map(|l| {
            if counts[l] == 1 {
                l.clone()
            } else {
                let n = seen.entry(l.clone()).or_insert(0);
                *n += 1;
                format!("{}{}", l, n)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fretboard::STD_6STR_GTR;

    #[test]
    fn fret_spec_matches_display() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(shape.to_ascii().fret_spec(), "x-3-2-0-1-0");
    }

    #[test]
    fn fret_spec_with_notes_open_c() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(
            shape.to_ascii().with_notes().fret_spec(),
            "x-3(C)-2(E)-0(G)-1(C)-0(E)"
        );
    }

    #[test]
    fn position_list_plain() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(shape.to_ascii().position_list(), "A:3 D:2 G:0 B:1 E2:0");
    }

    #[test]
    fn position_list_with_notes_disambiguates_e_strings() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(
            shape.to_ascii().with_notes().position_list(),
            "A:3(C) D:2(E) G:0(G) B:1(C) E2:0(E)"
        );
    }

    #[test]
    fn position_list_skips_muted_strings() {
        let shape = FretboardShape::from_string("x-x-0-2-3-2", &STD_6STR_GTR).unwrap();
        let out = shape.to_ascii().position_list();
        assert!(!out.contains("E1:"));
        assert!(!out.contains("A:"));
        assert!(out.contains("D:0"));
    }

    #[test]
    fn position_list_full_barre_labels_both_e_strings() {
        let shape = FretboardShape::from_string("1-3-3-2-1-1", &STD_6STR_GTR).unwrap();
        let out = shape.to_ascii().position_list();
        assert!(out.starts_with("E1:1"));
        assert!(out.contains("E2:1"));
    }

    #[test]
    fn show_notes_toggle_off_matches_default() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(
            shape.to_ascii().with_notes().show_notes(false).fret_spec(),
            shape.to_ascii().fret_spec()
        );
    }

    #[test]
    fn fret_spec_tab_convention_reverses_order() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(
            shape
                .to_ascii()
                .string_convention(StringConvention::OneIndexedFromHigh)
                .fret_spec(),
            "0-1-0-2-3-x"
        );
    }

    #[test]
    fn position_list_tab_convention_reverses_order() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(
            shape
                .to_ascii()
                .string_convention(StringConvention::OneIndexedFromHigh)
                .position_list(),
            "E2:0 B:1 G:0 D:2 A:3"
        );
    }

    #[test]
    fn custom_string_names_override_derived_labels() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        let names = vec!["6", "5", "4", "3", "2", "1"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            shape.to_ascii().string_names(names).position_list(),
            "5:3 4:2 3:0 2:1 1:0"
        );
    }

    #[test]
    fn custom_names_interact_with_convention() {
        // String names are always indexed low-to-high; convention only
        // controls emission order.
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        let names = vec!["6", "5", "4", "3", "2", "1"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            shape
                .to_ascii()
                .string_names(names)
                .string_convention(StringConvention::OneIndexedFromHigh)
                .position_list(),
            "1:0 2:1 3:0 4:2 5:3"
        );
    }
}
