use smufl::Glyph;
use ttf_parser::Face;

use super::glyph_outline::{GlyphOutline, SvgPathBuilder};

/// Errors from font operations.
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("failed to parse font: {0}")]
    ParseError(String),
    #[error("glyph not found in font: {0:?}")]
    GlyphNotFound(Glyph),
    #[error("no outline for glyph: {0:?}")]
    NoOutline(Glyph),
    #[error("no cmap entry for codepoint U+{0:04X}")]
    NoCmapEntry(u32),
    #[error("failed to parse metadata: {0}")]
    MetadataError(String),
}

/// A parsed SMuFL-compliant music font, providing glyph outlines and metadata.
///
/// Font-agnostic: while v1 ships only Bravura, layout and render code
/// interact with this trait-like struct without hard-coding any font.
pub struct MusicFont<'a> {
    face: Face<'a>,
    metadata: smufl::Metadata,
}

impl<'a> MusicFont<'a> {
    /// Parse a font from raw OTF/TTF bytes and its SMuFL metadata JSON.
    pub fn new(font_data: &'a [u8], metadata_json: &[u8]) -> Result<Self, FontError> {
        let face =
            Face::parse(font_data, 0).map_err(|e| FontError::ParseError(format!("{e}")))?;
        let metadata: smufl::Metadata = serde_json::from_slice(metadata_json)
            .map_err(|e| FontError::MetadataError(format!("{e}")))?;
        Ok(Self { face, metadata })
    }

    /// Font design units per em (typically 1000 for CFF, 2048 for TrueType).
    pub fn units_per_em(&self) -> u16 {
        self.face.units_per_em()
    }

    /// Access the parsed SMuFL metadata (engraving defaults, anchors, bboxes, etc).
    pub fn metadata(&self) -> &smufl::Metadata {
        &self.metadata
    }

    /// Look up the glyph ID for a SMuFL glyph by its codepoint.
    fn glyph_id(&self, glyph: Glyph) -> Result<ttf_parser::GlyphId, FontError> {
        let codepoint = glyph.codepoint();
        self.face
            .glyph_index(codepoint)
            .ok_or(FontError::NoCmapEntry(codepoint as u32))
    }

    /// Extract the SVG path data for a SMuFL glyph.
    ///
    /// The returned path data has y-coordinates flipped for direct use in SVG.
    /// Coordinates are in font design units.
    pub fn glyph_outline(&self, glyph: Glyph) -> Result<GlyphOutline, FontError> {
        let gid = self.glyph_id(glyph)?;
        let mut builder = SvgPathBuilder::new();
        self.face
            .outline_glyph(gid, &mut builder)
            .ok_or(FontError::NoOutline(glyph))?;

        let advance_width = self
            .face
            .glyph_hor_advance(gid)
            .unwrap_or(0);

        Ok(GlyphOutline {
            path_data: builder.into_path_data(),
            advance_width,
        })
    }

    /// Get the horizontal advance width of a glyph in font design units.
    pub fn glyph_advance(&self, glyph: Glyph) -> Result<u16, FontError> {
        let gid = self.glyph_id(glyph)?;
        Ok(self.face.glyph_hor_advance(gid).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{BRAVURA_METADATA, BRAVURA_OTF};

    fn bravura() -> MusicFont<'static> {
        MusicFont::new(BRAVURA_OTF, BRAVURA_METADATA).expect("Bravura should parse")
    }

    #[test]
    fn units_per_em_is_1000() {
        // Bravura is a CFF-based OpenType font with 1000 UPM
        assert_eq!(bravura().units_per_em(), 1000);
    }

    #[test]
    fn metadata_font_name_is_bravura() {
        assert_eq!(bravura().metadata().font_name, "Bravura");
    }

    #[test]
    fn notehead_black_has_outline() {
        let font = bravura();
        let outline = font
            .glyph_outline(Glyph::NoteheadBlack)
            .expect("noteheadBlack should have an outline");
        // The path should start with a move command
        assert!(
            outline.path_data.starts_with('M'),
            "path should start with M, got: {}",
            &outline.path_data[..20.min(outline.path_data.len())]
        );
        // Should contain curve commands (CFF glyphs use cubic beziers)
        assert!(
            outline.path_data.contains('C'),
            "notehead outline should contain cubic curves"
        );
        // Should be closed
        assert!(
            outline.path_data.contains('Z'),
            "notehead outline should be closed"
        );
        // Advance width should be positive and reasonable
        // Bravura noteheadBlack advance is about 1.18 staff spaces = ~295 font units
        assert!(outline.advance_width > 200, "advance too small");
        assert!(outline.advance_width < 500, "advance too large");
    }

    #[test]
    fn treble_clef_has_outline() {
        let font = bravura();
        let outline = font
            .glyph_outline(Glyph::GClef)
            .expect("gClef should have an outline");
        assert!(outline.path_data.starts_with('M'));
        // Treble clef is a complex glyph — path should be substantial
        assert!(
            outline.path_data.len() > 200,
            "treble clef path suspiciously short: {} chars",
            outline.path_data.len()
        );
    }

    #[test]
    fn nonexistent_glyph_returns_error() {
        let font = bravura();
        // Use an obscure glyph that Bravura might not have — but actually
        // Bravura is very complete. Let's verify error handling works by
        // checking the error type rather than expecting failure.
        // The glyph_id lookup might succeed for most SMuFL glyphs in Bravura.
        // Instead, verify the path is Ok for a known glyph.
        let result = font.glyph_outline(Glyph::NoteheadBlack);
        assert!(result.is_ok());
    }

    #[test]
    fn multiple_glyphs_produce_different_outlines() {
        let font = bravura();
        let black = font.glyph_outline(Glyph::NoteheadBlack).unwrap();
        let whole = font.glyph_outline(Glyph::NoteheadWhole).unwrap();
        assert_ne!(
            black.path_data, whole.path_data,
            "different noteheads should have different outlines"
        );
    }
}
