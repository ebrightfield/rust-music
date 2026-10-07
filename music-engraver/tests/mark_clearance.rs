use music::notation::rhythm::duration::Duration;
use music::note::{note::Note, pitch::Pitch};
use music_engraver::font::bravura_font;
use music_engraver::layout::text_script::TextScript;
use music_engraver::layout::{dynamics::Dynamic, lyric::LyricSyllable};
use music_engraver::score::ScoreBuilder;

fn attr(line: &str, name: &str) -> f64 {
    line.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn low_score() -> ScoreBuilder {
    ScoreBuilder::new()
        .measures_per_system(1)
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .cresc()
        .lyric(LyricSyllable::word("low"))
        .note(Pitch::new(Note::D, 3), Duration::QTR)
        .lyric(LyricSyllable::word("note"))
        .note(Pitch::new(Note::C, 3), Duration::HALF)
        .hairpin_end()
        .barline()
        .note(Pitch::new(Note::D, 3), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .cresc()
        .lyric(LyricSyllable::word("deep"))
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .lyric(LyricSyllable::word("again"))
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .hairpin_end()
        .end_barline()
}

#[test]
fn low_notes_clear_dynamics_hairpins_lyrics_and_system_breaks() {
    let svg = low_score().render_svg();
    let lines: Vec<_> = svg
        .lines()
        .filter(|line| line.starts_with("  <line "))
        .collect();
    let hairpins: Vec<_> = lines
        .iter()
        .filter(|line| {
            attr(line, "x2") - attr(line, "x1") > 1000.0 && attr(line, "y1") != attr(line, "y2")
        })
        .collect();
    assert_eq!(hairpins.len(), 4);
    let ledger: Vec<_> = lines
        .iter()
        .filter(|line| {
            (attr(line, "x2") - attr(line, "x1") - 495.0).abs() < 1.0
                && attr(line, "y1") == attr(line, "y2")
        })
        .collect();
    let syllables: Vec<_> = svg
        .lines()
        .filter(|line| line.starts_with("  <text "))
        .collect();
    assert_eq!(syllables.len(), 4);
    let font = bravura_font();
    let paths: Vec<_> = svg
        .lines()
        .filter(|line| line.starts_with("  <path "))
        .collect();
    for (system, dynamic) in [Dynamic::Piano, Dynamic::Forte].into_iter().enumerate() {
        let outline = font.glyph_outline(dynamic.glyph()).unwrap();
        let path = paths
            .iter()
            .find(|path| path.contains(&format!("d=\"{}\"", outline.path_data)))
            .unwrap();
        let baseline: f64 = path
            .split_once("translate(")
            .unwrap()
            .1
            .split_once(')')
            .unwrap()
            .0
            .split_once(',')
            .unwrap()
            .1
            .trim()
            .parse()
            .unwrap();
        let ink_bottom = baseline
            + font
                .glyph_bbox_design_units(dynamic.glyph())
                .unwrap()
                .y_bottom;
        let lyric_top =
            attr(syllables[system * 2], "y") - 0.9 * attr(syllables[system * 2], "font-size");
        assert!(
            lyric_top > ink_bottom + 50.0,
            "lyric must clear dynamic ink on system {system}"
        );
    }
    for system in 0..2 {
        let first_staff_y = if system == 0 {
            0.0
        } else {
            lines
                .iter()
                .find(|line| attr(line, "x1") == 0.0 && attr(line, "y1") > 2500.0)
                .map(|line| attr(line, "y1"))
                .unwrap()
        };
        let last_ledger = ledger
            .iter()
            .map(|line| attr(line, "y1"))
            .filter(|&y| y > first_staff_y && y < first_staff_y + 2500.0)
            .fold(0.0_f64, f64::max);
        let wedge_top = hairpins[system * 2..system * 2 + 2]
            .iter()
            .flat_map(|line| [attr(line, "y1"), attr(line, "y2")])
            .fold(f64::INFINITY, f64::min);
        let wedge_bottom = hairpins[system * 2..system * 2 + 2]
            .iter()
            .flat_map(|line| [attr(line, "y1"), attr(line, "y2")])
            .fold(0.0_f64, f64::max);
        assert!(
            wedge_top > last_ledger + 100.0,
            "wedge must clear adjacent ledger ink: system {system}"
        );
        for lyric in &syllables[system * 2..system * 2 + 2] {
            assert!(
                attr(lyric, "y") - 0.9 * attr(lyric, "font-size") > wedge_bottom + 50.0,
                "lyrics must clear the entire wedge: system {system}"
            );
        }
    }
    let vb: Vec<f64> = svg
        .lines()
        .next()
        .unwrap()
        .split_once("viewBox=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    assert!(vb[1] + vb[3] > attr(syllables[3], "y") + 0.3 * attr(syllables[3], "font-size"));
}

#[test]
fn below_text_script_uses_low_note_lane_and_stays_inside_page() {
    let svg = ScoreBuilder::new()
        .note(Pitch::new(Note::C, 3), Duration::WHOLE)
        .text_script(TextScript::below("cantabile"))
        .end_barline()
        .render_svg();
    let text = svg
        .lines()
        .find(|line| line.contains(">cantabile</text>"))
        .unwrap();
    let bottom = svg
        .lines()
        .next()
        .unwrap()
        .split_once("viewBox=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .split_whitespace()
        .map(|v| v.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert!(attr(text, "y") - attr(text, "font-size") > 2100.0);
    assert!(bottom[1] + bottom[3] > attr(text, "y") + 0.3 * attr(text, "font-size"));
}

#[cfg(feature = "png")]
#[test]
fn low_note_and_dynamic_have_clear_pixels_between_them() {
    let png = low_score().try_render_png(4.0).unwrap();
    let image = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
    let svg = low_score().render_svg();
    let header = svg.lines().next().unwrap();
    let vb: Vec<f64> = header
        .split_once("viewBox=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    let px = |x: f64| ((x - vb[0]) / vb[2] * f64::from(image.width())) as u32;
    let py = |y: f64| ((y - vb[1]) / vb[3] * f64::from(image.height())) as u32;
    let ink =
        |y: f64| (px(1180.0)..px(1280.0)).any(|x| image.pixel(x, py(y)).unwrap().alpha() > 32);
    assert!(ink(2150.0), "low notehead must rasterize");
    assert!(
        !ink(2340.0),
        "notehead and dynamic must have an empty pixel corridor"
    );
    assert!(ink(2670.0), "dynamic must rasterize below the corridor");
}

#[test]
fn low_register_hairpin_continuation_clears_both_systems() {
    let svg = ScoreBuilder::new()
        .measures_per_system(1)
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .lyric(LyricSyllable::word("away"))
        .cresc()
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .barline()
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .lyric(LyricSyllable::word("home"))
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .hairpin_end()
        .end_barline()
        .render_svg();
    let wedges: Vec<_> = svg
        .lines()
        .filter(|line| {
            line.starts_with("  <line ")
                && attr(line, "x2") - attr(line, "x1") > 1000.0
                && attr(line, "y1") != attr(line, "y2")
        })
        .collect();
    assert_eq!(
        wedges.len(),
        4,
        "broken wedge has an outgoing and incoming half"
    );
    assert!(wedges[2..]
        .iter()
        .all(|line| line.contains("stroke-dasharray=")));
    let ledgers: Vec<_> = svg
        .lines()
        .filter(|line| {
            line.starts_with("  <line ")
                && (attr(line, "x2") - attr(line, "x1") - 495.0).abs() < 1.0
                && attr(line, "y1") == attr(line, "y2")
        })
        .collect();
    for half in 0..2 {
        let top = wedges[half * 2..half * 2 + 2]
            .iter()
            .flat_map(|line| [attr(line, "y1"), attr(line, "y2")])
            .fold(f64::INFINITY, f64::min);
        let ledger_bottom = ledgers
            .iter()
            .map(|line| attr(line, "y1"))
            .filter(|&y| y < top)
            .fold(0.0_f64, f64::max);
        assert!(
            top > ledger_bottom + 100.0,
            "half {half} must clear its ledger ink"
        );
    }
}
