use crate::font::{FontError, MusicFont};
use crate::layout::tab_harmonic::TabHarmonicLayout;
use crate::render::SvgWriter;

/// Draw a natural harmonic indicator (small ○) above a tab fret number.
///
/// The glyph is scaled down and translated to the layout position.
pub fn draw_tab_harmonic(
    svg: &mut SvgWriter,
    layout: &TabHarmonicLayout,
    font: &MusicFont,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;
    let transform = format!(
        "translate({}, {}) scale({})",
        layout.x, layout.y, layout.scale
    );
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_harmonic::layout_tab_harmonic;
    use crate::render::SvgWriter;

    fn setup() -> (MusicFont<'static>, TabStaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, -100.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn draw_harmonic_adds_path() {
        let (font, staff) = setup();
        let layout = layout_tab_harmonic(&staff, 1, 1000.0);
        let mut svg = make_svg();
        draw_tab_harmonic(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("<path "),
            "should render a path element for the harmonic glyph"
        );
    }

    #[test]
    fn draw_harmonic_has_scale_transform() {
        let (font, staff) = setup();
        let layout = layout_tab_harmonic(&staff, 1, 1000.0);
        let mut svg = make_svg();
        draw_tab_harmonic(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("scale(0.6)"),
            "should contain scale transform for reduced glyph size"
        );
    }

    #[test]
    fn draw_harmonic_has_translate_transform() {
        let (font, staff) = setup();
        let layout = layout_tab_harmonic(&staff, 3, 750.0);
        let mut svg = make_svg();
        draw_tab_harmonic(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("translate(750,"),
            "should contain translate with correct x position"
        );
    }

    #[test]
    fn different_strings_produce_different_svg() {
        let (font, staff) = setup();
        let l1 = layout_tab_harmonic(&staff, 1, 500.0);
        let l6 = layout_tab_harmonic(&staff, 6, 500.0);
        let mut svg1 = make_svg();
        let mut svg6 = make_svg();
        draw_tab_harmonic(&mut svg1, &l1, &font).unwrap();
        draw_tab_harmonic(&mut svg6, &l6, &font).unwrap();
        assert_ne!(
            svg1.to_svg(),
            svg6.to_svg(),
            "harmonics on different strings should produce different SVG"
        );
    }

    #[test]
    fn draw_harmonic_is_filled_black() {
        let (font, staff) = setup();
        let layout = layout_tab_harmonic(&staff, 1, 500.0);
        let mut svg = make_svg();
        draw_tab_harmonic(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("fill=\"black\""),
            "harmonic glyph should be filled black"
        );
    }
}
