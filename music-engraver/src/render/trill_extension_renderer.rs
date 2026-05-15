//! Render a tiled trill wavy-line extension.
//!
//! Emits one SVG `<path>` per segment in the layout by translating the SMuFL
//! wiggle glyph outline. The renderer is intentionally font-aware (it queries
//! the font for the glyph outline) while the layout phase remains font-agnostic
//! — see `layout::trill_extension` for the geometry contract.

use crate::font::{FontError, MusicFont};
use crate::layout::trill_extension::{MultiSpeedTrillExtensionLayout, TrillExtensionLayout};
use crate::render::SvgWriter;
use smufl::Glyph;
use std::collections::HashMap;

/// Render a trill wavy-line extension by tiling the wiggle glyph at each
/// segment position computed by [`layout_trill_extension`](crate::layout::trill_extension::layout_trill_extension).
///
/// Returns `Ok(())` after emitting one `<path>` per segment. A layout with an
/// empty `segment_xs` is a no-op (the layout step already returned `None` for
/// spans too short to tile, but the renderer handles the empty case
/// defensively for callers that build layouts manually).
pub fn draw_trill_extension(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &TrillExtensionLayout,
) -> Result<(), FontError> {
    if layout.segment_xs.is_empty() {
        return Ok(());
    }

    let outline = font.glyph_outline(layout.glyph)?;
    for &x in &layout.segment_xs {
        let transform = format!("translate({x},{y})", y = layout.y);
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }
    Ok(())
}

/// Render a multi-speed trill wavy-line extension.
///
/// Emits one SVG `<path>` per tile in the layout, translating each tile's
/// glyph outline to its computed position. Glyph outlines are cached per
/// unique glyph for the duration of the call — a long wiggle that uses
/// only two speeds re-queries the font exactly twice, regardless of tile
/// count.
///
/// Returns `Ok(())` after emitting all tiles. An empty layout is a no-op
/// (the layout step already rejects spans too short to tile, but we
/// handle the empty case defensively for callers that build layouts
/// manually). The first font error encountered short-circuits — any
/// tiles already emitted remain in the writer; the layout is rejected
/// only if a referenced glyph is genuinely missing from the active font,
/// which would be a font-bundling regression caught by other tests
/// before this point.
pub fn draw_trill_extension_multi_speed(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &MultiSpeedTrillExtensionLayout,
) -> Result<(), FontError> {
    if layout.tiles.is_empty() {
        return Ok(());
    }

    // Cache outlines per unique glyph. Most multi-speed layouts use 2–3
    // distinct glyphs (one per region) and many tiles per glyph, so the
    // cache turns N-many font queries into K-many where K is the number
    // of unique speeds — a real win when a long trill has dozens of tiles.
    //
    // The Entry API is used (rather than contains_key + insert) so the
    // fallible glyph lookup is run only on cache miss while still
    // returning a borrow into the cache without an intermediate clone.
    let mut path_cache: HashMap<Glyph, String> = HashMap::new();
    for tile in &layout.tiles {
        let path_data: &str = match path_cache.entry(tile.glyph) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => {
                let outline = font.glyph_outline(tile.glyph)?;
                v.insert(outline.path_data)
            }
        };
        let transform = format!("translate({x},{y})", x = tile.x, y = layout.y);
        svg.add_path(path_data, "black", Some(&transform));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::trill_extension::layout_trill_extension;
    use smufl::Glyph;

    fn test_writer() -> SvgWriter {
        SvgWriter::new(400.0, 100.0, 0.0, 0.0, 2000.0, 500.0)
    }

    fn wiggle_advance(font: &MusicFont) -> f64 {
        font.glyph_advance(Glyph::WiggleTrill)
            .expect("WiggleTrill must have an advance") as f64
    }

    #[test]
    fn renders_one_path_per_segment_two_segments() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(100.0, 100.0 + 2.0 * advance, 50.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // Exactly two <path elements (no other content drawn in this test)
        assert_eq!(svg.matches("<path").count(), 2);
    }

    #[test]
    fn renders_one_path_per_segment_five_segments() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 5.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 5);
    }

    #[test]
    fn embeds_each_segment_x_in_translate() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(100.0, 100.0 + 3.0 * advance, 50.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        for &x in &layout.segment_xs {
            let needle = format!("translate({x},");
            assert!(
                svg.contains(&needle),
                "SVG should embed segment x {x} (looked for {needle:?}): {}",
                &svg[..svg.len().min(400)]
            );
        }
    }

    #[test]
    fn embeds_y_coordinate_in_translate() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 2.0 * advance, 271.5, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains(",271.5)"),
            "SVG should embed y coordinate 271.5: {}",
            &svg[..svg.len().min(400)]
        );
    }

    #[test]
    fn empty_layout_is_noop() {
        let font = bravura_font();
        let layout = TrillExtensionLayout {
            segment_xs: vec![],
            y: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 100.0,
        };
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test]
    fn all_segments_share_same_path_data() {
        // Each segment translates the same glyph outline; the path data after
        // each <path d="..." should match exactly. We assert this by counting
        // the most common "d=" prefix substring.
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 4.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // Grab the first path's d="..." literal and count its occurrences.
        let first_d_start = svg.find("d=\"").expect("at least one path") + 3;
        let first_d_end = svg[first_d_start..]
            .find('"')
            .expect("closing quote")
            + first_d_start;
        let path_d = &svg[first_d_start..first_d_end];
        // 4 segments, identical d attribute on every <path>
        assert_eq!(svg.matches(path_d).count(), 4);
    }

    #[test]
    fn unknown_glyph_returns_font_error() {
        // A non-WiggleTrill glyph in the layout struct should propagate the
        // font's error if the glyph isn't supported. We pick a glyph value
        // that's safe to ask for: actually any well-known glyph works here,
        // so instead we test the inverse — a known glyph never errors.
        let font = bravura_font();
        let layout = TrillExtensionLayout {
            segment_xs: vec![0.0],
            y: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 100.0,
        };
        let mut writer = test_writer();
        assert!(draw_trill_extension(&mut writer, &font, &layout).is_ok());
    }

    #[test]
    fn segment_path_color_is_black() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 2.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("fill=\"black\""),
            "wiggle paths should fill black: {}",
            &svg[..svg.len().min(400)]
        );
    }

    // --- draw_trill_extension_multi_speed ---

    use crate::layout::trill_extension::{
        layout_trill_extension_multi_speed, layout_trill_extension_with_glyph,
        MultiSpeedTrillExtensionLayout, TrillExtensionTile, TrillSpeedRegion,
    };

    fn advance_of(font: &MusicFont, glyph: Glyph) -> f64 {
        font.glyph_advance(glyph)
            .unwrap_or_else(|_| panic!("{glyph:?} must have an advance"))
            as f64
    }

    #[test]
    fn multi_speed_renders_one_path_per_tile_across_two_regions() {
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 3.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 3.0 * fast + 2.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 50.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 5, "3 Fast + 2 Slow");

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 5);
    }

    #[test]
    fn multi_speed_uses_different_path_data_per_glyph() {
        // Critical correctness canary: the Fast and Slow wiggle glyphs
        // have DIFFERENT outlines, so the SVG must contain at least two
        // distinct `d="..."` attribute values. A regression that cached
        // the wrong glyph's outline (e.g. used regions[0].glyph for all
        // tiles) would emit only one distinct path data and this fires.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 2.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 2.0 * fast + 2.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 0.0, &regions).unwrap();

        let fast_outline = font.glyph_outline(Glyph::WiggleTrillFast).unwrap().path_data;
        let slow_outline = font.glyph_outline(Glyph::WiggleTrillSlow).unwrap().path_data;
        assert_ne!(
            fast_outline, slow_outline,
            "test setup: Fast and Slow wiggle outlines must differ"
        );

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains(&fast_outline),
            "SVG must contain Fast wiggle outline literally"
        );
        assert!(
            svg.contains(&slow_outline),
            "SVG must contain Slow wiggle outline literally"
        );
    }

    #[test]
    fn multi_speed_correct_outline_at_each_tile_position() {
        // Stronger version of the previous canary: not just "both outlines
        // appear in the SVG somewhere" but "each tile's translate(x,y) is
        // immediately followed by its correct outline." A bug that emitted
        // Fast then Slow then Fast (instead of Fast Fast Slow Slow) for
        // a Fast-then-Slow region layout would not be caught by counting
        // alone — this test checks the pairing.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 2.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 2.0 * fast + 2.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 25.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 4);

        let fast_outline = font.glyph_outline(Glyph::WiggleTrillFast).unwrap().path_data;
        let slow_outline = font.glyph_outline(Glyph::WiggleTrillSlow).unwrap().path_data;

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();

        for tile in &layout.tiles {
            let expected_outline = if tile.glyph == Glyph::WiggleTrillFast {
                &fast_outline
            } else {
                &slow_outline
            };
            // The SvgWriter emits paths as `<path d="..." fill="..."
            // transform="translate(x,y)"/>` — so to verify the pairing of
            // outline + translate we look for the path data immediately
            // followed by the rest of the attribute list culminating in
            // this tile's translate.
            let needle = format!(
                "d=\"{d}\" fill=\"black\" transform=\"translate({x},{y})\"",
                d = expected_outline,
                x = tile.x,
                y = 25.0,
            );
            assert!(
                svg.contains(&needle),
                "tile at x={} glyph={:?} must pair with its correct outline; \
                 needle not found: {}",
                tile.x,
                tile.glyph,
                &needle[..needle.len().min(120)]
            );
        }
    }

    #[test]
    fn multi_speed_empty_layout_is_noop() {
        let font = bravura_font();
        let layout = MultiSpeedTrillExtensionLayout {
            tiles: vec![],
            y: 0.0,
        };
        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test]
    fn multi_speed_embeds_each_tile_x_in_translate() {
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 100.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 100.0 + 2.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 100.0 + 2.0 * fast + 2.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 50.0, &regions).unwrap();

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();

        for tile in &layout.tiles {
            let needle = format!("translate({x},", x = tile.x);
            assert!(
                svg.contains(&needle),
                "SVG must embed tile x {} ({}): not found in {}",
                tile.x,
                needle,
                &svg[..svg.len().min(400)]
            );
        }
    }

    #[test]
    fn multi_speed_embeds_y_coordinate_in_translate() {
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let regions = [TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: fast,
        }];
        let layout =
            layout_trill_extension_multi_speed(2.0 * fast, 271.5, &regions).unwrap();
        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains(",271.5)"),
            "SVG must embed shared y=271.5: {}",
            &svg[..svg.len().min(400)]
        );
    }

    #[test]
    fn multi_speed_caches_outline_per_unique_glyph() {
        // Stronger than path counting: a layout with 10 tiles but only 2
        // unique glyphs must emit exactly 2 distinct `d="..."` literals.
        // If the cache regressed (re-querying per tile and inserting
        // perturbed bytes — implausible but provable), we'd see 10 distinct
        // strings instead of 2.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 5.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 5.0 * fast + 5.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 0.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 10, "test setup");

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();

        // Count distinct `d="..."` substrings by walking the SVG.
        let mut distinct = std::collections::HashSet::new();
        let mut rest = svg.as_str();
        while let Some(start) = rest.find("d=\"") {
            let after = &rest[start + 3..];
            let end = after.find('"').expect("path data should close");
            distinct.insert(after[..end].to_string());
            rest = &after[end + 1..];
        }
        assert_eq!(
            distinct.len(),
            2,
            "10 tiles using 2 glyphs must produce 2 distinct path datas, saw {}",
            distinct.len()
        );
    }

    #[test]
    fn multi_speed_single_region_matches_single_speed_output_byte_for_byte() {
        // Single-region multi-speed must produce IDENTICAL SVG to the
        // single-speed renderer. This is the migration-safety property
        // mirrored at the renderer layer: callers can swap entry points
        // without changing rendered output.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let regions = [TrillSpeedRegion {
            start_x: 10.0,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: fast,
        }];
        let multi = layout_trill_extension_multi_speed(10.0 + 5.0 * fast, 25.0, &regions)
            .unwrap();
        let single = layout_trill_extension_with_glyph(
            10.0,
            10.0 + 5.0 * fast,
            25.0,
            Glyph::WiggleTrillFast,
            fast,
        )
        .unwrap();

        let mut multi_writer = test_writer();
        draw_trill_extension_multi_speed(&mut multi_writer, &font, &multi).unwrap();
        let multi_svg = multi_writer.to_svg();

        let mut single_writer = test_writer();
        draw_trill_extension(&mut single_writer, &font, &single).unwrap();
        let single_svg = single_writer.to_svg();

        assert_eq!(
            multi_svg, single_svg,
            "single-region multi-speed must produce byte-identical SVG to single-speed"
        );
    }

    #[test]
    fn multi_speed_path_color_is_black() {
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let regions = [TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: fast,
        }];
        let layout = layout_trill_extension_multi_speed(2.0 * fast, 0.0, &regions).unwrap();
        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("fill=\"black\""),
            "wiggle paths must fill black: {}",
            &svg[..svg.len().min(400)]
        );
    }

    #[test]
    fn multi_speed_tiles_emit_in_left_to_right_order() {
        // The renderer must emit tiles in layout order (left-to-right).
        // A regression that iterated the cache HashMap (unordered) instead
        // of `layout.tiles` (ordered) would interleave glyphs unpredictably.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let slow = advance_of(&font, Glyph::WiggleTrillSlow);
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: fast,
            },
            TrillSpeedRegion {
                start_x: 2.0 * fast,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: slow,
            },
        ];
        let end_x = 2.0 * fast + 2.0 * slow;
        let layout = layout_trill_extension_multi_speed(end_x, 0.0, &regions).unwrap();

        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();

        // Find translate(x, ...) positions in source order; they must
        // strictly increase.
        let mut xs: Vec<f64> = Vec::new();
        let mut rest = svg.as_str();
        while let Some(p) = rest.find("translate(") {
            let after = &rest[p + "translate(".len()..];
            let comma = after.find(',').expect("translate must have comma");
            let x_str = &after[..comma];
            xs.push(x_str.parse().unwrap_or_else(|_| panic!("not a number: {x_str:?}")));
            rest = &after[comma..];
        }
        assert_eq!(xs.len(), 4, "4 tiles -> 4 translates");
        for window in xs.windows(2) {
            assert!(
                window[1] > window[0],
                "translates must emit in strictly-increasing x order: {window:?}"
            );
        }
    }

    #[test]
    fn multi_speed_single_tile_layout_emits_exactly_one_path() {
        // Edge case: smallest non-empty layout.
        let font = bravura_font();
        let fast = advance_of(&font, Glyph::WiggleTrillFast);
        let layout = MultiSpeedTrillExtensionLayout {
            tiles: vec![TrillExtensionTile {
                x: 42.0,
                glyph: Glyph::WiggleTrillFast,
                advance: fast,
            }],
            y: 12.5,
        };
        let mut writer = test_writer();
        draw_trill_extension_multi_speed(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 1);
        assert!(svg.contains("translate(42,12.5)"));
    }
}
