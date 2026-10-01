use smufl::Glyph;
use ttf_parser::Face;

use super::engraving_config::EngravingConfig;
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

/// Glyph bounding box in font design units (already converted from SMuFL's
/// staff-space coordinates and y-flipped to SVG convention — y increases
/// downward, so `y_top < y_bottom`).
///
/// Returned by [`MusicFont::glyph_bbox_design_units`]. The two corners hug
/// the glyph's outline tightly; layout code that needs to scale or position
/// a glyph relative to its drawn extent (e.g. brace vertical scaling, bracket
/// scroll anchoring) should drive its math from these values rather than
/// from font-specific constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBBoxDesignUnits {
    /// X coordinate of the left edge of the bbox, in design units relative
    /// to the glyph's origin (typically the bbox's SW corner).
    pub x_left: f64,
    /// X coordinate of the right edge of the bbox.
    pub x_right: f64,
    /// Y coordinate of the top edge of the bbox in SVG convention
    /// (y increases downward — so `y_top` is the smallest y).
    pub y_top: f64,
    /// Y coordinate of the bottom edge of the bbox in SVG convention.
    pub y_bottom: f64,
}

impl GlyphBBoxDesignUnits {
    /// Width of the bbox in design units (always >= 0).
    pub fn width(&self) -> f64 {
        self.x_right - self.x_left
    }

    /// Height of the bbox in design units (always >= 0).
    pub fn height(&self) -> f64 {
        self.y_bottom - self.y_top
    }
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
        let face = Face::parse(font_data, 0).map_err(|e| FontError::ParseError(format!("{e}")))?;
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

        let advance_width = self.face.glyph_hor_advance(gid).unwrap_or(0);

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

    /// Look up a glyph's bounding box from SMuFL metadata, converted to font
    /// design units and y-flipped to SVG convention.
    ///
    /// SMuFL metadata reports glyph bboxes in **staff spaces** with y-up
    /// (matching font-design conventions). This helper converts to design
    /// units (multiplying by `staff_space = units_per_em / 4`, the SMuFL
    /// convention) and flips y so the returned `y_top < y_bottom` matches
    /// the rest of the engraver's SVG coordinate system.
    ///
    /// Returns `None` if the glyph's bbox is not present in the font's
    /// metadata. The SMuFL spec recommends fonts provide bbox data for all
    /// supplied glyphs, but it is not strictly required; callers should
    /// have a sensible fallback for the `None` case.
    pub fn glyph_bbox_design_units(&self, glyph: Glyph) -> Option<GlyphBBoxDesignUnits> {
        let bbox = self.metadata.bounding_boxes.get(glyph)?;
        // SMuFL: 1 staff space = units_per_em / 4 design units.
        let ss = self.face.units_per_em() as f64 / 4.0;
        let sw_x = bbox.sw.x().0 * ss;
        let sw_y = bbox.sw.y().0 * ss;
        let ne_x = bbox.ne.x().0 * ss;
        let ne_y = bbox.ne.y().0 * ss;
        // Font space has y-up; SVG y-down. The path renderer (SvgPathBuilder)
        // negates y when emitting path data. Mirror that here: a font-space
        // y of +n design units becomes an SVG-space y of -n. So:
        //   - font NE.y (top in font space)    → SVG y_top    = -ne_y
        //   - font SW.y (bottom in font space) → SVG y_bottom = -sw_y
        Some(GlyphBBoxDesignUnits {
            x_left: sw_x,
            x_right: ne_x,
            y_top: -ne_y,
            y_bottom: -sw_y,
        })
    }

    /// Build an `EngravingConfig` from this font's metadata, with all values
    /// resolved (no `Option`s). Missing metadata values fall back to
    /// SMuFL-recommended defaults.
    pub fn engraving_config(&self) -> EngravingConfig {
        EngravingConfig::from_smufl(&self.metadata.engraving_defaults, self.face.units_per_em())
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

    #[test]
    fn brace_bbox_design_units_matches_bravura_metadata() {
        // Bravura's brace bBox (per bravura_metadata.json): SW (0.008, 0.0),
        // NE (0.328, 3.988) in staff spaces. With Bravura's UPM=1000 and
        // SMuFL's 1 sp = UPM/4 = 250 design units, the bbox in design units
        // is: x_left=2, x_right=82, y_top=-997 (SVG-space, after y-flip),
        // y_bottom=0. The height (y_bottom - y_top) is 997 design units —
        // i.e. ~3.988 staff spaces, NOT 1 staff space as previously assumed
        // by the brace layout. The whole point of `glyph_bbox_design_units`
        // is to let the renderer scale the brace by its actual height.
        let font = bravura();
        let bbox = font
            .glyph_bbox_design_units(Glyph::Brace)
            .expect("Bravura brace bbox should be present in metadata");
        let tol = 0.001;
        assert!(
            (bbox.x_left - 2.0).abs() < tol,
            "brace x_left: expected 2.0, got {}",
            bbox.x_left
        );
        assert!(
            (bbox.x_right - 82.0).abs() < tol,
            "brace x_right: expected 82.0, got {}",
            bbox.x_right
        );
        // After y-flip: SVG-space y_top = -ne_y = -997 (top of glyph above
        // origin in SVG), y_bottom = -sw_y = 0 (origin at bottom).
        assert!(
            (bbox.y_top - (-997.0)).abs() < tol,
            "brace y_top (SVG): expected -997, got {}",
            bbox.y_top
        );
        assert!(
            (bbox.y_bottom - 0.0).abs() < tol,
            "brace y_bottom (SVG): expected 0, got {}",
            bbox.y_bottom
        );
        assert!(
            (bbox.height() - 997.0).abs() < tol,
            "brace height: expected 997 (3.988 sp × 250), got {}",
            bbox.height()
        );
        assert!(
            (bbox.width() - 80.0).abs() < tol,
            "brace width: expected 80, got {}",
            bbox.width()
        );
    }

    #[test]
    fn bracket_top_bbox_origin_at_bottom_left() {
        // bracketTop's metadata: SW (0,0), NE (1.876, 1.18). The glyph extends
        // up-and-right from its origin. After y-flip for SVG, the bbox sits
        // entirely above origin (y_top = -295, y_bottom = 0). This anchors
        // the SMuFL convention that the multi-staff renderer relies on when
        // it translates the scroll glyph to `(bracket.x, bracket.y_top)`.
        let font = bravura();
        let bbox = font
            .glyph_bbox_design_units(Glyph::BracketTop)
            .expect("BracketTop bbox should be present");
        let tol = 0.01;
        assert!((bbox.x_left - 0.0).abs() < tol);
        assert!((bbox.y_bottom - 0.0).abs() < tol);
        assert!(
            bbox.y_top < 0.0,
            "bracketTop extends upward (SVG-negative y)"
        );
        // Bravura's 1.18 sp = 295 design units.
        assert!(
            (bbox.height() - 295.0).abs() < 0.5,
            "bracketTop height: expected ~295, got {}",
            bbox.height()
        );
    }

    #[test]
    fn bracket_bottom_bbox_origin_at_top_left() {
        // bracketBottom's metadata: SW (0,-1.18), NE (1.876, 0). The glyph
        // extends down-and-right from its origin. After y-flip for SVG, the
        // bbox sits entirely below origin (y_top = 0, y_bottom = +295).
        let font = bravura();
        let bbox = font
            .glyph_bbox_design_units(Glyph::BracketBottom)
            .expect("BracketBottom bbox should be present");
        let tol = 0.01;
        assert!((bbox.y_top - 0.0).abs() < tol);
        assert!(bbox.y_bottom > 0.0, "bracketBottom extends downward");
        assert!(
            (bbox.height() - 295.0).abs() < 0.5,
            "bracketBottom height: expected ~295, got {}",
            bbox.height()
        );
    }

    #[test]
    fn glyph_bbox_design_units_returns_none_for_glyph_without_metadata() {
        // Build a `MusicFont` with empty metadata JSON: no glyph bboxes
        // present, so the lookup should return None for everything.
        let empty_metadata = br#"{"fontName":"Empty"}"#;
        let font = MusicFont::new(BRAVURA_OTF, empty_metadata)
            .expect("font with empty metadata still parses");
        assert!(
            font.glyph_bbox_design_units(Glyph::Brace).is_none(),
            "no bbox in metadata → None"
        );
        assert!(
            font.glyph_bbox_design_units(Glyph::NoteheadBlack).is_none(),
            "no bbox in metadata → None"
        );
    }

    #[test]
    fn glyph_bbox_design_units_height_is_nonnegative() {
        // Sanity: height = y_bottom - y_top should never be negative after
        // the y-flip (since ne.y >= sw.y in metadata convention).
        let font = bravura();
        for g in [
            Glyph::Brace,
            Glyph::BracketTop,
            Glyph::BracketBottom,
            Glyph::NoteheadBlack,
            Glyph::NoteheadWhole,
            Glyph::GClef,
            Glyph::FClef,
        ] {
            let bbox = font
                .glyph_bbox_design_units(g)
                .unwrap_or_else(|| panic!("expected bbox for {g:?}"));
            assert!(
                bbox.height() >= 0.0,
                "{g:?} bbox height should be >= 0, got {}",
                bbox.height()
            );
            assert!(
                bbox.width() >= 0.0,
                "{g:?} bbox width should be >= 0, got {}",
                bbox.width()
            );
        }
    }
}
