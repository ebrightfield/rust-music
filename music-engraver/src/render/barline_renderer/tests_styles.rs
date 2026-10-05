//! Dashed, tick, and invisible barlines: glyph identity, placement, width.

use super::*;
use crate::font::bravura_font;

fn setup() -> (MusicFont<'static>, StaffLayout) {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
    (font, staff)
}

/// `(d, transform)` of every `<path>` in `svg`.
fn paths(svg: &str) -> Vec<(String, String)> {
    let attribute = |tag: &str, name: &str| {
        let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
        let end = tag[start..].find('"')? + start;
        Some(tag[start..end].to_string())
    };
    svg.split("<path")
        .skip(1)
        .map(|tag| {
            let tag = &tag[..tag.find("/>").expect("path element closes")];
            (
                attribute(tag, "d").expect("path has data"),
                attribute(tag, "transform").unwrap_or_default(),
            )
        })
        .collect()
}

fn render(style: BarlineStyle, x: f64) -> (String, f64) {
    let (font, staff) = setup();
    let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
    let width = draw_barline(&mut svg, &staff, &font, x, style).unwrap();
    (svg.to_svg(), width)
}

#[test]
fn dashed_barline_draws_the_barline_dashed_glyph_centred_on_x() {
    let (font, staff) = setup();
    let config = font.engraving_config();
    let thickness = config.to_font_units(config.dashed_barline_thickness);
    let (svg, width) = render(BarlineStyle::Dashed, 500.0);

    assert_eq!(svg.matches("<line ").count(), 0, "no stroked lines");
    let dashed = font.glyph_outline(Glyph::BarlineDashed).unwrap().path_data;
    assert_eq!(
        paths(&svg),
        vec![(
            dashed,
            format!("translate({}, {})", 500.0 - thickness / 2.0, staff.y_of(0)),
        )]
    );
    assert!((width - thickness).abs() < f64::EPSILON);
}

#[test]
fn tick_barline_draws_the_barline_tick_glyph_on_the_bottom_line_origin() {
    let (font, staff) = setup();
    let config = font.engraving_config();
    let thin = config.thin_barline_thickness_fu();
    let (svg, width) = render(BarlineStyle::Tick, 800.0);

    assert_eq!(svg.matches("<line ").count(), 0);
    let tick = font.glyph_outline(Glyph::BarlineTick).unwrap().path_data;
    assert_ne!(
        tick,
        font.glyph_outline(Glyph::BarlineDashed).unwrap().path_data,
        "tick and dashed glyphs must be distinguishable"
    );
    assert_eq!(
        paths(&svg),
        vec![(
            tick,
            format!("translate({}, {})", 800.0 - thin / 2.0, staff.y_of(0)),
        )]
    );
    assert!((width - thin).abs() < f64::EPSILON);
}

#[test]
fn invisible_barline_draws_nothing_and_has_no_width() {
    let (svg, width) = render(BarlineStyle::Invisible, 500.0);
    assert_eq!(svg.matches("<line ").count(), 0);
    assert_eq!(svg.matches("<path").count(), 0);
    assert_eq!(width, 0.0);

    let (font, staff) = setup();
    let layout = barline_layout(
        BarlineStyle::Invisible,
        500.0,
        &staff,
        &font.engraving_config(),
    );
    assert!(layout.strokes.is_empty());
    assert!(layout.dots.is_none());
    assert!(layout.glyph.is_none());
}
