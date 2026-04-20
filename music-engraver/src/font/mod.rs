//! Font loading, glyph outline extraction, and engraving configuration.
//!
//! The bundled Bravura OTF is the default font. All glyph lookups use
//! `smufl::Glyph` enum variants so that layout and render code remain
//! font-agnostic — only this module touches the raw OTF bytes.

mod engraving_config;
mod glyph_outline;
mod music_font;

pub use engraving_config::EngravingConfig;
pub use glyph_outline::GlyphOutline;
pub use music_font::{FontError, MusicFont};

/// Bravura OTF font bytes, bundled at compile time.
pub static BRAVURA_OTF: &[u8] = include_bytes!("../../fonts/Bravura.otf");

/// Bravura font metadata JSON, bundled at compile time.
pub static BRAVURA_METADATA: &[u8] = include_bytes!("../../fonts/bravura_metadata.json");

/// Convenience: create a `MusicFont` backed by the bundled Bravura font.
///
/// # Panics
///
/// Panics if the bundled Bravura OTF or metadata JSON cannot be parsed.
/// This is unreachable in normal operation: both assets are compiled in
/// via `include_bytes!` from known-good files and validated by unit tests
/// (`bravura_otf_is_valid_opentype`, `bravura_metadata_is_valid_json`).
pub fn bravura_font() -> MusicFont<'static> {
    MusicFont::new(BRAVURA_OTF, BRAVURA_METADATA)
        .expect("bundled Bravura OTF + metadata are compile-time constants validated by tests")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bravura_otf_is_valid_opentype() {
        assert_eq!(&BRAVURA_OTF[..4], b"OTTO", "Expected CFF-based OpenType magic");
        assert!(BRAVURA_OTF.len() > 100_000, "Font file suspiciously small");
    }

    #[test]
    fn bravura_otf_parseable_by_ttf_parser() {
        let face = ttf_parser::Face::parse(BRAVURA_OTF, 0)
            .expect("ttf-parser should parse Bravura.otf");
        assert!(face.units_per_em() > 0);
        assert!(
            face.number_of_glyphs() > 2000,
            "Bravura should have thousands of glyphs"
        );
    }

    #[test]
    fn bravura_metadata_is_valid_json() {
        let value: serde_json::Value =
            serde_json::from_slice(BRAVURA_METADATA).expect("metadata should be valid JSON");
        assert!(value.is_object());
        assert!(
            value.get("engravingDefaults").is_some(),
            "metadata should contain engravingDefaults"
        );
    }

    #[test]
    fn bravura_font_convenience_works() {
        let font = bravura_font();
        assert_eq!(font.units_per_em(), 1000);
        assert_eq!(font.metadata().font_name, "Bravura");
    }
}
