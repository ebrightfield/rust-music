//! SVG rendering of composed text lines ([`TextLineLayout`]): text runs as
//! `<text>` elements and inline SMuFL glyphs as outline paths.

use crate::font::{FontError, MusicFont};
use crate::layout::text_script::{PlacedLineItem, TextLineLayout};
use crate::render::{SvgWriter, TextStyle};

/// Draw `line` with its origin (left edge, baseline) at (`x`, `baseline_y`).
pub fn draw_text_line(
    svg: &mut SvgWriter,
    font: &MusicFont,
    line: &TextLineLayout,
    x: f64,
    baseline_y: f64,
) -> Result<(), FontError> {
    for item in &line.items {
        match item {
            PlacedLineItem::Text {
                x: item_x,
                text,
                font: text_font,
                font_size,
                anchor_end,
            } => {
                let style = TextStyle {
                    font_family: "serif",
                    font_size: *font_size,
                    fill: "black",
                    anchor: if *anchor_end { "end" } else { "start" },
                    font_weight: text_font.svg_weight(),
                    font_style: text_font.svg_style(),
                    dominant_baseline: "auto",
                };
                svg.add_text(x + item_x, baseline_y, text, &style);
            }
            PlacedLineItem::Glyph {
                x: item_x,
                glyph,
                scale,
                dy,
            } => {
                let outline = font.glyph_outline(*glyph)?;
                let transform = if (*scale - 1.0).abs() < f64::EPSILON {
                    format!("translate({}, {})", x + item_x, baseline_y + dy)
                } else {
                    format!(
                        "translate({},{}) scale({},{})",
                        x + item_x,
                        baseline_y + dy,
                        scale,
                        scale
                    )
                };
                svg.add_path(&outline.path_data, "black", Some(&transform));
            }
        }
    }
    Ok(())
}
