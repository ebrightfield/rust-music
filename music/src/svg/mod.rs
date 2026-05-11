//! Pure Rust SVG generation for music theory diagrams.
//!
//! This module provides SVG generation for:
//! - Pitch circle (chromatic clock) diagrams
//! - Fretboard diagrams (chord diagrams)
//! - Interval diagrams (vectors, matrices, linear)
//!
//! # Examples
//!
//! ## Pitch Circle
//! ```
//! use music::svg::pitch_circle::PitchCircleBuilder;
//! use music::Pc;
//!
//! // Create a C major triad diagram
//! let svg = PitchCircleBuilder::new()
//!     .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
//!     .root(Pc::Pc0)
//!     .title("C Major Triad")
//!     .show_intervals(true)
//!     .build();
//!
//! // Save to file
//! // std::fs::write("diagram.svg", svg).unwrap();
//! ```
//!
//! ## Fretboard Diagram
//! ```
//! use music::svg::fretboard::{FretboardBuilder, FretPosition};
//!
//! // Create an open C chord diagram
//! let svg = FretboardBuilder::new()
//!     .position(FretPosition::Muted { string: 0 })
//!     .position(FretPosition::Fretted { string: 1, fret: 3 })
//!     .position(FretPosition::Fretted { string: 2, fret: 2 })
//!     .position(FretPosition::Open { string: 3 })
//!     .position(FretPosition::Fretted { string: 4, fret: 1 })
//!     .position(FretPosition::Open { string: 5 })
//!     .title("C Major")
//!     .build();
//! ```
//!
//! ## Interval Diagram
//! ```
//! use music::svg::interval::IntervalBuilder;
//! use music::Pc;
//!
//! // Create an interval vector diagram
//! let svg = IntervalBuilder::new()
//!     .pitches([Pc::Pc0, Pc::Pc4, Pc::Pc7])
//!     .title("Major Triad Intervals")
//!     .build_vector();
//! ```

pub mod fretboard;
pub mod interval;
pub mod pitch_circle;
pub mod theme;
pub mod util;

pub use fretboard::{Barre, Finger, FretboardBuilder, FretboardConfig, FretPosition, Orientation};
pub use interval::{IntervalBuilder, IntervalConfig};
pub use pitch_circle::{NoteLabels, PitchCircleBuilder, PitchCircleConfig};
pub use theme::SvgTheme;

use crate::fretboard::fretboard_shape::FretboardShape;
use crate::note_collections::{NoteSet, pc_set::{PcShape, PcContent}};

use std::io;
use std::path::Path;

/// Extension trait for types that can be rendered as pitch circle diagrams.
pub trait ToPitchCircleSvg {
    /// Create a pitch circle builder from this type.
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder;
}

impl ToPitchCircleSvg for PcShape {
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder {
        PitchCircleBuilder::new().from_pc_shape(self)
    }
}

impl ToPitchCircleSvg for NoteSet {
    fn to_pitch_circle_svg(&self) -> PitchCircleBuilder {
        let content = PcContent::from(self);
        PitchCircleBuilder::new().from_pc_shape(&content.to_shape())
    }
}

/// Extension trait for types that can be rendered as fretboard diagrams.
pub trait ToFretboardSvg<'a> {
    /// Create a fretboard builder from this type.
    fn to_fretboard_svg(&'a self) -> FretboardBuilder<'a>;
}

impl<'a> ToFretboardSvg<'a> for FretboardShape<'a> {
    fn to_fretboard_svg(&'a self) -> FretboardBuilder<'a> {
        FretboardBuilder::new().from_shape(self)
    }
}

/// Extension trait for types that can be rendered as interval diagrams.
pub trait ToIntervalSvg {
    /// Create an interval builder from this type.
    fn to_interval_svg(&self) -> IntervalBuilder;
}

impl ToIntervalSvg for PcShape {
    fn to_interval_svg(&self) -> IntervalBuilder {
        IntervalBuilder::new().from_pc_shape(self)
    }
}

impl ToIntervalSvg for NoteSet {
    fn to_interval_svg(&self) -> IntervalBuilder {
        let content = PcContent::from(self);
        IntervalBuilder::new().from_pc_shape(&content.to_shape())
    }
}

/// Save SVG string to a file.
pub fn save_svg(svg: &str, path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::write(path, svg)
}

/// Convert SVG to a data URI for HTML embedding.
///
/// The returned string can be used directly as the `src` attribute of an `<img>` tag.
pub fn svg_to_data_uri(svg: &str) -> String {
    // Use percent encoding instead of base64 to avoid external dependency
    let encoded: String = svg
        .bytes()
        .map(|b| match b {
            b'<' => "%3C".to_string(),
            b'>' => "%3E".to_string(),
            b'"' => "%22".to_string(),
            b'#' => "%23".to_string(),
            b' ' => "%20".to_string(),
            b'\n' => "%0A".to_string(),
            _ if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) => {
                (b as char).to_string()
            }
            _ => format!("%{:02X}", b),
        })
        .collect();
    format!("data:image/svg+xml,{}", encoded)
}

/// Generate an HTML page with embedded SVG.
pub fn svg_to_html(svg: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body style="display:flex;justify-content:center;padding:20px;">
{}
</body>
</html>"#,
        title, svg
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fretboard::STD_6STR_GTR;
    use crate::note::note::Note;
    use crate::Pc;

    #[test]
    fn test_svg_to_data_uri() {
        let svg = "<svg></svg>";
        let uri = svg_to_data_uri(svg);
        assert!(uri.starts_with("data:image/svg+xml,"));
    }

    #[test]
    fn test_svg_to_html() {
        let svg = "<svg></svg>";
        let html = svg_to_html(svg, "Test");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("<svg></svg>"));
    }

    #[test]
    fn test_integration_with_pc_set() {
        let c_major = PcShape::new(vec![
            Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11,
        ]);
        let svg = PitchCircleBuilder::new()
            .from_pc_shape(&c_major)
            .root(Pc::Pc0)
            .title("C Major Scale")
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("C Major Scale"));
    }

    #[test]
    fn test_pc_set_to_pitch_circle_svg_trait() {
        let c_major = PcShape::new(vec![
            Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11,
        ]);

        // Use the extension trait
        let svg = c_major
            .to_pitch_circle_svg()
            .root(Pc::Pc0)
            .title("C Major Scale (trait)")
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("C Major Scale (trait)"));
        // 7 notes total should be highlighted (6 with highlight_color, 1 with root_color)
        let highlight_count = svg.matches(SvgTheme::default().highlight_color).count();
        let root_count = svg.matches(SvgTheme::default().root_color).count();
        assert_eq!(highlight_count + root_count, 7);
    }

    #[test]
    fn test_note_set_to_pitch_circle_svg_trait() {
        let c_major = NoteSet::with_root(
            vec![
                Note::C,
                Note::D,
                Note::E,
                Note::F,
                Note::G,
                Note::A,
                Note::B,
            ],
            &Note::C,
        );

        let svg = c_major
            .to_pitch_circle_svg()
            .title("C Major (NoteSet)")
            .show_intervals(true)
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("C Major (NoteSet)"));
        assert!(svg.contains("<line")); // interval lines
    }

    #[test]
    fn test_fretboard_shape_to_svg_trait() {
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();

        let svg = shape
            .to_fretboard_svg()
            .title("C Major (trait)")
            .root_at(1, 3) // C on the A string
            .build();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("C Major (trait)"));
        assert!(svg.contains("×")); // muted string
    }

    #[test]
    fn test_combined_traits_workflow() {
        // A complete workflow: chord on fretboard + pitch circle
        let shape = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();

        let fretboard_svg = shape
            .to_fretboard_svg()
            .title("C Major Chord")
            .build();

        let c_major_triad = PcShape::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
        let circle_svg = c_major_triad
            .to_pitch_circle_svg()
            .root(Pc::Pc0)
            .title("C Major Triad")
            .build();

        // Both should be valid SVGs
        assert!(fretboard_svg.starts_with("<svg"));
        assert!(circle_svg.starts_with("<svg"));
    }

    #[test]
    fn test_pc_set_to_interval_svg_trait() {
        let c_major_triad = PcShape::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);

        // Use the extension trait
        let svg = c_major_triad
            .to_interval_svg()
            .title("Major Triad Intervals")
            .build_vector();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Major Triad Intervals"));
    }

    #[test]
    fn test_note_set_to_interval_svg_trait() {
        let c_major = NoteSet::with_root(
            vec![Note::C, Note::E, Note::G],
            &Note::C,
        );

        let svg = c_major
            .to_interval_svg()
            .title("C Major Triad")
            .build_matrix();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("C Major Triad"));
        assert!(svg.contains("<rect")); // Matrix cells
    }
}
