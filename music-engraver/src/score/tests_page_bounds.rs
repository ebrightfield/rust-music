use super::multi_staff::MultiStaffScore;
use super::ScoreBuilder;
use crate::font::bravura_font;
use crate::layout::accidental::{AccidentalDisplay, AccidentalPolicy};
use crate::layout::barline::BarlineStyle;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use smufl::Glyph;

fn attribute(tag: &str, name: &str) -> f64 {
    let start = tag.find(&format!("{name}=\"")).expect("SVG coordinate") + name.len() + 2;
    tag[start..].split('"').next().unwrap().parse().unwrap()
}

fn viewbox_right(svg: &str) -> f64 {
    let values: Vec<f64> = svg
        .split("viewBox=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    values[0] + values[2]
}

// Compare rendered coordinates with actual font glyph bboxes, independently
// of the layout's spacing rods.
fn assert_horizontal_ink_inside(svg: &str, min_padding: f64, expect_accidentals: bool) -> f64 {
    let right = viewbox_right(svg);
    let font = bravura_font();
    let glyphs = [
        Glyph::NoteheadBlack,
        Glyph::AccidentalSharp,
        Glyph::AccidentalFlat,
        Glyph::AccidentalNatural,
    ]
    .map(|glyph| {
        let outline = font.glyph_outline(glyph).unwrap().path_data;
        let ink_right = font.glyph_bbox_design_units(glyph).unwrap().x_right;
        (glyph, format!("d=\"{outline}\""), ink_right)
    });
    let mut noteheads = 0;
    let mut accidentals = 0;
    let mut staff_lines = 0;
    let mut barlines = 0;
    let mut max_ink = f64::NEG_INFINITY;
    for line in svg.lines() {
        if line.contains("<line ") {
            let x1 = attribute(line, "x1");
            let x2 = attribute(line, "x2");
            let ink = x1.max(x2) + attribute(line, "stroke-width") / 2.0;
            max_ink = max_ink.max(ink);
            if x2 - x1 > 1000.0 {
                staff_lines += 1;
            } else if (x2 - x1).abs() < 0.001 {
                barlines += 1;
            }
            assert!(
                right >= ink + min_padding,
                "line ink {ink} outside viewBox {right}"
            );
        } else if line.contains("<path ") {
            for (glyph, outline, ink_right) in &glyphs {
                if !line.contains(outline) {
                    continue;
                }
                let translation = line.split("translate(").nth(1).unwrap();
                let x: f64 = translation
                    .split([',', ' '])
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                let ink = x + ink_right;
                if *glyph == Glyph::NoteheadBlack {
                    noteheads += 1;
                } else {
                    accidentals += 1;
                }
                max_ink = max_ink.max(ink);
                assert!(
                    right >= ink + min_padding,
                    "{glyph:?} ink {ink} outside viewBox {right}"
                );
                break;
            }
        }
    }
    assert!(staff_lines >= 5, "expected rendered staff lines");
    assert!(barlines > 0, "expected rendered barlines");
    assert!(noteheads > 0, "expected rendered noteheads");
    if expect_accidentals {
        assert!(accidentals > 0, "expected rendered accidentals");
    }
    max_ink
}

// Representative of mn-c04-r038 (printed p. 48 / PDF 39): forced accidentals,
// dashed intra-measure divisions and a mid-score 12/8 change make rods wider
// than a fixed 40-staff-space system. This uses only the public score builder.
fn dense_score() -> ScoreBuilder {
    let mut score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(8, 8)
        .accidental_policy(AccidentalPolicy::Forget)
        .system_width_fu(10_000.0)
        .measures_per_system(3)
        .note(Pitch::new(Note::A, 3), Duration::EIGHTH)
        .barline()
        .time_signature_change(12, 8);
    for (index, note) in [
        Note::Bes,
        Note::Cis,
        Note::D,
        Note::Ees,
        Note::C,
        Note::B,
        Note::Bes,
        Note::Cis,
        Note::E,
        Note::Dis,
        Note::D,
        Note::C,
    ]
    .into_iter()
    .enumerate()
    {
        if index == 3 || index == 6 || index == 9 {
            score = score.inline_barline(BarlineStyle::Dashed);
        }
        let pitch = Pitch::new(note, 4);
        score = if matches!(index, 4 | 5 | 8 | 10 | 11) {
            score.note_with_accidental(pitch, Duration::EIGHTH, AccidentalDisplay::Force)
        } else {
            score.note(pitch, Duration::EIGHTH)
        };
    }
    score
        .barline()
        .system_break()
        .note(Pitch::new(Note::A, 3), Duration::QTR)
        .end_barline()
}

#[test]
fn dense_fixed_width_system_expands_page_and_staff_to_fit_actual_ink() {
    let svg = dense_score().render_svg();
    let max_ink = assert_horizontal_ink_inside(&svg, 100.0, true);
    assert!(
        max_ink > 10_000.0,
        "fixture must exceed the requested width"
    );
}

#[test]
fn dense_grand_staff_page_expands_to_shared_grid_width() {
    let upper = dense_score();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .rest(Duration::EIGHTH)
        .barline()
        .rest(Duration::WHOLE)
        .end_barline();
    let svg = MultiStaffScore::grand_staff(upper, lower)
        .system_width_fu(10_000.0)
        .measures_per_system(3)
        .render_svg();

    let max_ink = assert_horizontal_ink_inside(&svg, 100.0, true);
    assert!(
        max_ink > 10_000.0,
        "shared grid must exceed the requested width"
    );
}

#[test]
fn fitting_system_retains_requested_staff_width() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .system_width_fu(10_000.0)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .end_barline()
        .render_svg();
    let staff_line = svg
        .lines()
        .find(|line| {
            line.contains("<line ")
                && (attribute(line, "x2") - attribute(line, "x1") - 10_000.0).abs() < 0.001
        })
        .unwrap();
    assert_eq!(attribute(staff_line, "x2"), 10_000.0);
    assert_horizontal_ink_inside(&svg, 100.0, false);
}
#[test]
fn dense_guitar_standard_and_tab_lines_share_expanded_page() {
    use super::guitar::GuitarScore;

    let mut guitar = GuitarScore::standard();
    for _ in 0..40 {
        guitar
            .note(Pitch::new(Note::E, 4), Duration::SIXTEENTH, 1, 0)
            .unwrap();
    }
    guitar.end_barline().unwrap();
    let svg = MultiStaffScore::guitar(guitar)
        .system_width_fu(10_000.0)
        .render_svg();
    let max_ink = assert_horizontal_ink_inside(&svg, 100.0, false);
    assert!(
        max_ink > 10_000.0,
        "guitar rods must exceed the requested width"
    );
}
