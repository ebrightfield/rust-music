//! Glyph metrics in staff spaces, read from the bundled SMuFL font.
//!
//! Layout reserves space for prefix glyphs (clefs, time signatures) from the
//! same metrics the renderers draw with, so reserved and inked extents agree.

use smufl::Glyph;

use crate::font::BUNDLED_BRAVURA;

/// Ink bounding box of a glyph relative to its origin, in staff spaces, in
/// SVG orientation (`y` grows downward: `y_top < 0` reaches above the origin).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBox {
    /// Left ink edge.
    pub x_left: f64,
    /// Right ink edge.
    pub x_right: f64,
    /// Top ink edge (negative above the origin).
    pub y_top: f64,
    /// Bottom ink edge (positive below the origin).
    pub y_bottom: f64,
}

impl GlyphBox {
    /// The box scaled uniformly about the glyph origin.
    pub fn scaled(self, scale: f64) -> Self {
        Self {
            x_left: self.x_left * scale,
            x_right: self.x_right * scale,
            y_top: self.y_top * scale,
            y_bottom: self.y_bottom * scale,
        }
    }

    /// Ink height.
    pub fn height(self) -> f64 {
        self.y_bottom - self.y_top
    }
}

fn font_staff_space() -> f64 {
    f64::from(BUNDLED_BRAVURA.units_per_em()) / 4.0
}

/// The glyph's SMuFL bounding box in staff spaces, or `None` when the font
/// metadata has no box for it.
pub fn glyph_box(glyph: Glyph) -> Option<GlyphBox> {
    let bbox = BUNDLED_BRAVURA.glyph_bbox_design_units(glyph)?;
    let ss = font_staff_space();
    Some(GlyphBox {
        x_left: bbox.x_left / ss,
        x_right: bbox.x_right / ss,
        y_top: bbox.y_top / ss,
        y_bottom: bbox.y_bottom / ss,
    })
}

/// The glyph's horizontal advance in staff spaces (0 for unmapped glyphs).
pub fn glyph_advance(glyph: Glyph) -> f64 {
    BUNDLED_BRAVURA
        .glyph_advance(glyph)
        .map_or(0.0, |advance| f64::from(advance) / font_staff_space())
}
