//! Example: a multi-speed trill wavy-line extension that changes density
//! partway through. Demonstrates `layout_trill_extension_multi_speed` +
//! `draw_trill_extension_multi_speed`.
//!
//! Renders three wiggle lines stacked vertically, all spanning the same x
//! range:
//!
//! 1. **Single-speed `WiggleTrill`** (reference).
//! 2. **Accelerating**: `WiggleTrillSlow` → `WiggleTrill` → `WiggleTrillFast`
//!    in three equal-width regions.
//! 3. **Decelerating**: `WiggleTrillFast` → `WiggleTrill` → `WiggleTrillSlow`.
//!
//! Output: `examples/output/trill_multi_speed.svg`.
//!
//! This is intentionally a low-level layout-and-render example, not a
//! ScoreBuilder example — wiring multi-speed into the score builder is a
//! separate chunk. The point here is to exercise the new geometry &
//! rendering pipeline end-to-end and produce a visually-verifiable SVG.

use music_engraver::font::bravura_font;
use music_engraver::layout::trill_extension::{
    layout_trill_extension, layout_trill_extension_multi_speed, TrillSpeedRegion,
};
use music_engraver::render::trill_extension_renderer::{
    draw_trill_extension, draw_trill_extension_multi_speed,
};
use music_engraver::render::SvgWriter;
use smufl::Glyph;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let font = bravura_font();
    let upe = font.units_per_em() as f64;

    let advance_of =
        |g: Glyph| -> f64 { font.glyph_advance(g).expect("advance must be present") as f64 };
    let std_adv = advance_of(Glyph::WiggleTrill);
    let slow_adv = advance_of(Glyph::WiggleTrillSlow);
    let fast_adv = advance_of(Glyph::WiggleTrillFast);

    let start_x = 0.5 * upe;
    let end_x = 8.0 * upe;
    // 3 equal regions for the multi-speed lines
    let third = (end_x - start_x) / 3.0;
    let mid1 = start_x + third;
    let mid2 = start_x + 2.0 * third;

    // Stack lines a staff-space apart vertically. units_per_em is the
    // font's design-unit reference; "0.25 upe ≈ one staff space" is
    // SMuFL-conventional. We don't rely on engraving_config here since
    // this is a pure-render demo.
    let line_dy = 0.5 * upe;
    let y_single = 1.0 * upe;
    let y_accel = y_single + line_dy;
    let y_decel = y_accel + line_dy;

    let mut svg = SvgWriter::new(
        ((end_x - 0.0) / upe) * 50.0, // px display width
        4.0 * 50.0,                   // px display height
        0.0,
        0.0,
        end_x + start_x,
        4.0 * upe,
    );

    // Line 1: single-speed (reference).
    if let Some(layout) = layout_trill_extension(start_x, end_x, y_single, std_adv) {
        draw_trill_extension(&mut svg, &font, &layout).expect("draw single-speed");
    }

    // Line 2: accelerating — Slow then Standard then Fast.
    let accel = [
        TrillSpeedRegion {
            start_x,
            glyph: Glyph::WiggleTrillSlow,
            segment_advance: slow_adv,
        },
        TrillSpeedRegion {
            start_x: mid1,
            glyph: Glyph::WiggleTrill,
            segment_advance: std_adv,
        },
        TrillSpeedRegion {
            start_x: mid2,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: fast_adv,
        },
    ];
    if let Some(layout) = layout_trill_extension_multi_speed(end_x, y_accel, &accel) {
        draw_trill_extension_multi_speed(&mut svg, &font, &layout)
            .expect("draw accelerating multi-speed");
    }

    // Line 3: decelerating — Fast then Standard then Slow.
    let decel = [
        TrillSpeedRegion {
            start_x,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: fast_adv,
        },
        TrillSpeedRegion {
            start_x: mid1,
            glyph: Glyph::WiggleTrill,
            segment_advance: std_adv,
        },
        TrillSpeedRegion {
            start_x: mid2,
            glyph: Glyph::WiggleTrillSlow,
            segment_advance: slow_adv,
        },
    ];
    if let Some(layout) = layout_trill_extension_multi_speed(end_x, y_decel, &decel) {
        draw_trill_extension_multi_speed(&mut svg, &font, &layout)
            .expect("draw decelerating multi-speed");
    }

    let out_path = out_dir.join("trill_multi_speed.svg");
    let rendered = svg.to_svg();
    std::fs::write(&out_path, &rendered).expect("write svg");

    // Sanity: the SVG must contain three trill lines worth of <path> elements.
    assert!(rendered.starts_with("<svg"), "output must start with <svg>");
    let path_count = rendered.matches("<path").count();
    assert!(
        path_count >= 9,
        "expected at least 9 wiggle tiles total across 3 lines, saw {path_count}"
    );

    println!(
        "wrote {} ({} bytes, {} <path> elements)",
        out_path.display(),
        rendered.len(),
        path_count
    );
}
