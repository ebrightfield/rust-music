use super::ScoreBuilder;
use crate::font::bravura_font;
use crate::layout::lyric::{LyricStyle, LyricSyllable};
use crate::layout::measure::MeasureElement;
use crate::layout::NoteSize;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use smufl::Glyph;

fn g4() -> Pitch {
    Pitch::new(Note::G, 4)
}

// Read the actual SVG glyph transforms rather than testing the collision detector's
// offsets: this catches noteheads whose stems/beams were drawn at a different x.
fn glyph_positions(svg: &str, glyph_name: Glyph) -> Vec<(f64, f64, f64)> {
    let font = bravura_font();
    let glyph = font.glyph_outline(glyph_name).unwrap();
    let needle = format!("d=\"{}\"", glyph.path_data);
    svg.lines()
        .filter(|line| line.contains(&needle))
        .map(|line| {
            let transform = line.split("translate(").nth(1).unwrap();
            let (x, rest) = transform.split_once(',').unwrap();
            let (y, tail) = rest.split_once(')').unwrap();
            let scale = tail
                .split("scale(")
                .nth(1)
                .and_then(|s| s.split_once(')').map(|(s, _)| s.parse::<f64>().unwrap()))
                .unwrap_or(1.0);
            (x.parse().unwrap(), y.trim().parse().unwrap(), scale)
        })
        .collect()
}

fn filled_heads(svg: &str) -> Vec<(f64, f64, f64)> {
    glyph_positions(svg, Glyph::NoteheadBlack)
}

fn svg_number(line: &str, attribute: &str) -> f64 {
    let start = format!("{attribute}=\"");
    line.split(&start)
        .nth(1)
        .unwrap()
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn has_down_stem_at(svg: &str, x: f64, y: f64) -> bool {
    svg.lines()
        .filter(|line| line.contains("<line ") && line.contains("stroke=\"black\""))
        .any(|line| {
            (svg_number(line, "x1") - x).abs() < 1.0
                && (svg_number(line, "x2") - x).abs() < 1.0
                && (svg_number(line, "y1") - y).abs() < 1.0
                && svg_number(line, "y2") > y + 50.0
        })
}

fn has_up_stem_at(svg: &str, x: f64, y: f64) -> bool {
    svg.lines()
        .filter(|line| line.contains("<line ") && line.contains("stroke=\"black\""))
        .any(|line| {
            (svg_number(line, "x1") - x).abs() < 1.0
                && (svg_number(line, "x2") - x).abs() < 1.0
                && (svg_number(line, "y2") - y).abs() < 1.0
                && svg_number(line, "y1") < y - 50.0
        })
}

#[test]
fn aniara_g4_beamed_cue_unison_is_two_readable_heads() {
    // mn-c11-r029: G4 upper eighths and a lower sixteenth group whose
    // first G4 is small. Both voices begin at the same underlying beat.
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .voice(0)
        .rest(Duration::QTR)
        .beam_group(vec![(g4(), Duration::EIGHTH), (g4(), Duration::EIGHTH)])
        .voice(1)
        .rest(Duration::QTR)
        .begin_beam()
        .note(g4(), Duration::SIXTEENTH)
        .note_size(NoteSize::Cue)
        .note(g4(), Duration::SIXTEENTH)
        .note(g4(), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let mut layout_score = score.clone();
    let page = layout_score.page_layout(250.0).unwrap().unwrap();
    let measure = &page.systems[0].system.measures[0];
    let first_note_x = |elements: &[_]| -> f64 {
        elements
            .iter()
            .find_map(|e: &crate::layout::measure::PositionedElement| {
                matches!(e.element, MeasureElement::Note(_)).then_some(e.x)
            })
            .unwrap()
    };
    let beat_x = first_note_x(&measure.layout.elements);
    assert_eq!(
        beat_x,
        first_note_x(&measure.additional_voice_layouts[0].elements),
        "both voices must keep their shared rhythmic column"
    );
    let svg = score.render_svg();
    let heads = filled_heads(&svg);
    let normal = heads.iter().find(|(_, _, scale)| *scale == 1.0).unwrap();
    let cue = heads.iter().find(|(_, _, scale)| *scale < 1.0).unwrap();
    assert_eq!(normal.1, cue.1, "G4 must remain on the same staff line");
    let head_width = bravura_font()
        .glyph_outline(Glyph::NoteheadBlack)
        .unwrap()
        .advance_width as f64;
    let center_gap = ((normal.0 + head_width / 2.0) - (cue.0 + cue.2 * head_width / 2.0)).abs();
    assert!(center_gap >= (1.0 + cue.2) * head_width / 2.0 - 1.0,
        "same-onset normal/cue heads must not merge: normal={normal:?}, cue={cue:?}, gap={center_gap}");
    let stem_x = cue.0 + bravura_font().engraving_config().stem_thickness_fu() * cue.2 / 2.0;
    assert!(
        has_down_stem_at(&svg, stem_x, cue.1),
        "lower beamed stem must attach to displaced cue head at x={stem_x}, y={}",
        cue.1
    );
}

#[test]
fn aniara_exact_g4_beat_two_normal_and_tiny_alt_stems_attach() {
    // Source measure 7: f''8. g'16, then g'8[g'8] above
    // g'16[ \tiny g'16 g'8] on the alternate voice.
    let score = ScoreBuilder::new()
        .voice(0)
        .note(
            Pitch::new(Note::F, 5),
            Duration::new(DurationKind::Eighth, 1),
        )
        .note(g4(), Duration::SIXTEENTH)
        .begin_beam()
        .note(g4(), Duration::EIGHTH)
        .note(g4(), Duration::EIGHTH)
        .end_beam()
        .voice(1)
        .rest(Duration::QTR)
        .begin_beam()
        .note(g4(), Duration::SIXTEENTH)
        .note(g4(), Duration::SIXTEENTH)
        .note_size(NoteSize::Cue)
        .note(g4(), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let mut layout_score = score.clone();
    let page = layout_score.page_layout(250.0).unwrap().unwrap();
    let measure = &page.systems[0].system.measures[0];
    let upper = measure
        .layout
        .elements
        .iter()
        .filter_map(|item| matches!(item.element, MeasureElement::Note(_)).then_some(item.x))
        .collect::<Vec<_>>();
    let lower = measure.additional_voice_layouts[0]
        .elements
        .iter()
        .filter_map(|item| matches!(item.element, MeasureElement::Note(_)).then_some(item.x))
        .collect::<Vec<_>>();
    assert_eq!(
        &upper[2..],
        &[lower[0], lower[2]],
        "both shared beats must retain exact rhythmic x"
    );
    let onset = page.systems[0].x + measure.x_offset + upper[2];
    let svg = score.render_svg();
    let heads = filled_heads(&svg);
    let upper_head = heads
        .iter()
        .find(|head| (head.0 - onset).abs() < 1.0)
        .unwrap();
    let width = bravura_font()
        .glyph_outline(Glyph::NoteheadBlack)
        .unwrap()
        .advance_width as f64;
    let lower_head = heads
        .iter()
        .find(|head| (head.0 - onset - width).abs() < 1.0 && (head.1 - upper_head.1).abs() < 1.0)
        .expect("separate lower G4 head on the same underlying beat");
    let thickness = bravura_font().engraving_config().stem_thickness_fu();
    assert!(has_up_stem_at(
        &svg,
        upper_head.0 + width - thickness / 2.0,
        upper_head.1
    ));
    assert!(has_down_stem_at(
        &svg,
        lower_head.0 + thickness / 2.0,
        lower_head.1
    ));
    assert_eq!(
        heads.iter().filter(|head| head.2 < 1.0).count(),
        1,
        "the second alternate sixteenth is cue-sized like the source"
    );
}

#[test]
fn same_kind_unison_keeps_both_stems_distinct() {
    let svg = ScoreBuilder::new()
        .voice(0)
        .note(g4(), Duration::QTR)
        .voice(1)
        .note(g4(), Duration::QTR)
        .end_barline()
        .render_svg();
    let heads = filled_heads(&svg);
    assert_eq!(heads.len(), 2);
    let width = bravura_font()
        .glyph_outline(Glyph::NoteheadBlack)
        .unwrap()
        .advance_width as f64;
    let thickness = bravura_font().engraving_config().stem_thickness_fu();
    assert!(
        (heads[1].0 - heads[0].0).abs() >= width - 1.0,
        "shared-kind unison cannot merge its opposing stems into one"
    );
    let up_x = heads[0].0 + width - thickness / 2.0;
    let down_x = heads[1].0 + thickness / 2.0;
    assert!((up_x - down_x).abs() >= thickness - 1.0);
    assert!(has_up_stem_at(&svg, up_x, heads[0].1));
    assert!(has_down_stem_at(&svg, down_x, heads[1].1));
}

#[test]
fn second_collision_displaces_ink_but_a_third_does_not() {
    let width = bravura_font()
        .glyph_outline(Glyph::NoteheadBlack)
        .unwrap()
        .advance_width as f64;
    let second = ScoreBuilder::new()
        .voice(0)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .voice(1)
        .note(g4(), Duration::QTR)
        .end_barline()
        .render_svg();
    let heads = filled_heads(&second);
    assert_eq!(heads.len(), 2);
    assert!((heads[0].1 - heads[1].1).abs() > 0.0);
    assert!(
        (heads[0].0 - heads[1].0).abs() >= width - 1.0,
        "heads a second apart must be staggered"
    );
    let third = ScoreBuilder::new()
        .voice(0)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .voice(1)
        .note(g4(), Duration::QTR)
        .end_barline()
        .render_svg();
    let heads = filled_heads(&third);
    assert_eq!(heads.len(), 2);
    assert!(
        (heads[0].0 - heads[1].0).abs() < 1.0,
        "non-colliding thirds stay in their shared time column"
    );
}

#[test]
fn displaced_cue_flag_accidental_dots_and_ledger_follow_ink_not_lyrics() {
    let svg = ScoreBuilder::new()
        .voice(0)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .voice(1)
        .note(
            Pitch::new(Note::Cis, 4),
            Duration::new(DurationKind::Sixteenth, 1),
        )
        .note_size(NoteSize::Cue)
        .lyric_verse_on_voice(1, 1, LyricSyllable::word("zu"), LyricStyle::Italic)
        .end_barline()
        .render_svg();
    let heads = filled_heads(&svg);
    assert_eq!(heads.len(), 2);
    let cue = heads.iter().find(|(_, _, scale)| *scale < 1.0).unwrap();
    let normal = heads.iter().find(|(_, _, scale)| *scale == 1.0).unwrap();
    let font = bravura_font();
    let width = font
        .glyph_outline(Glyph::NoteheadBlack)
        .unwrap()
        .advance_width as f64;
    assert!((cue.0 - normal.0 - width).abs() < 1.0);
    let accidental = glyph_positions(&svg, Glyph::AccidentalSharp);
    assert_eq!(accidental.len(), 1);
    assert_eq!(accidental[0].1, cue.1);
    assert!(accidental[0].0 < cue.0);
    assert!(
        accidental[0].0 > normal.0,
        "cue accidental must follow shifted head"
    );
    let dot = glyph_positions(&svg, Glyph::AugmentationDot);
    assert_eq!(dot.len(), 1);
    assert!(dot[0].0 > cue.0 + width * cue.2);
    let stem_x = cue.0 + font.engraving_config().stem_thickness_fu() * cue.2 / 2.0;
    assert!(has_down_stem_at(&svg, stem_x, cue.1));
    let flag = glyph_positions(&svg, Glyph::Flag16thDown);
    assert_eq!(flag.len(), 1);
    assert!(
        (flag[0].0 - stem_x).abs() < 1.0,
        "flag stays on the displaced stem"
    );
    assert!(
        svg.lines()
            .filter(|line| line.contains("<line ") && line.contains("stroke=\"black\""))
            .any(|line| {
                (svg_number(line, "y1") - cue.1).abs() < 1.0
                    && (svg_number(line, "y2") - cue.1).abs() < 1.0
                    && svg_number(line, "x1") < cue.0
                    && svg_number(line, "x2") > cue.0 + width * cue.2
            }),
        "cue ledger must straddle its displaced head"
    );
    let lyric = svg
        .lines()
        .find(|line| line.contains(">zu</text>"))
        .unwrap();
    let lyric_x = svg_number(lyric, "x");
    assert!(
        (lyric_x - (normal.0 + width * cue.2 / 2.0)).abs() < 1.0,
        "lyric stays anchored to the shared beat, not the displaced cue head"
    );
}
