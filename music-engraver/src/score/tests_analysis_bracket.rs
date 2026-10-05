use super::ScoreBuilder;
use crate::font::bravura_font;
use crate::layout::analysis_bracket::{AnalysisBracketSpec, AnalysisBracketStyle};
use crate::layout::barline::BarlineStyle;
use crate::layout::glissando::GlissandoStyle;
use crate::layout::measure::MeasureElement;
use crate::layout::placement::Placement;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}
fn bracket(style: AnalysisBracketStyle, side: Placement) -> AnalysisBracketSpec {
    AnalysisBracketSpec::new(style, side)
}
fn marked_line(svg: &str, dashed: bool, horizontal: bool) -> &str {
    svg.lines()
        .find(|line| {
            line.contains("<line ")
                && line.contains("stroke=\"black\"")
                && line.contains("stroke-dasharray=") == dashed
                && (!horizontal || (attr(line, "y1") - attr(line, "y2")).abs() < 0.001)
                && (horizontal || (attr(line, "y1") - attr(line, "y2")).abs() > 0.001)
                && attr(line, "x2") - attr(line, "x1") > 200.0
        })
        .expect("marked line")
}
fn attr(line: &str, key: &str) -> f64 {
    let start = line.find(&format!("{key}=\"")).unwrap() + key.len() + 2;
    line[start..].split('"').next().unwrap().parse().unwrap()
}
fn event_xs(score: &ScoreBuilder) -> Vec<Vec<f64>> {
    let font = bravura_font();
    let page = score
        .clone()
        .page_layout(font.engraving_config().staff_space)
        .unwrap()
        .unwrap();
    page.systems
        .iter()
        .map(|system| {
            system
                .system
                .measures
                .iter()
                .flat_map(|m| {
                    m.layout.elements.iter().filter_map(|e| {
                        matches!(
                            e.element,
                            MeasureElement::Note(_)
                                | MeasureElement::Chord(_)
                                | MeasureElement::Rest(_)
                        )
                        .then_some(system.x + m.x_offset + e.x)
                    })
                })
                .collect()
        })
        .collect()
}

/// mn-c06-m003: the dashed bracket on d4–c4 passes a barline below bass staff.
#[test]
fn mn_c06_m003_bracket_survives_barline_below() {
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .note(p(Note::D, 3), Duration::QTR)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Dashed, Placement::Below))
        .note(p(Note::B, 2), Duration::QTR)
        .barline()
        .note(p(Note::C, 3), Duration::QTR)
        .analysis_bracket_end()
        .rest(Duration::QTR)
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let line = marked_line(&svg, true, true);
    assert!((attr(line, "x1") - xs[0][0]).abs() < 0.001);
    assert!(attr(line, "x2") > xs[0][2]);
    assert!(attr(line, "y1") > 4.0 * bravura_font().engraving_config().staff_space);
    assert!(svg.contains("stroke-dasharray="));
    assert!(svg.lines().any(|hook| hook.contains("<line ")
        && (attr(hook, "x1") - attr(line, "x1")).abs() < 0.001
        && (attr(hook, "x2") - attr(line, "x1")).abs() < 0.001));
}

/// mn-c01-h007: solid brackets span chord pairs and chord-to-single-note pairs.
#[test]
fn mn_c01_h007_solid_chord_pair_and_note_endpoint() {
    let score = ScoreBuilder::new()
        .clef(Clef::Treble)
        .stemless()
        .chord(vec![p(Note::Cis, 4), p(Note::Ees, 4)], Duration::EIGHTH)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Solid, Placement::Above))
        .chord(vec![p(Note::D, 4), p(Note::E, 4)], Duration::EIGHTH)
        .analysis_bracket_end()
        .barline()
        .chord(vec![p(Note::D, 4), p(Note::Ees, 4)], Duration::EIGHTH)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Solid, Placement::Above))
        .note(p(Note::E, 4), Duration::EIGHTH)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let brackets: Vec<_> = svg
        .lines()
        .filter(|l| {
            l.contains("<line ")
                && l.contains("stroke=\"black\"")
                && !l.contains("stroke-dasharray")
                && (attr(l, "y1") - attr(l, "y2")).abs() < 0.001
                && xs[0].iter().any(|x| (attr(l, "x1") - x).abs() < 0.001)
        })
        .collect();
    assert_eq!(brackets.len(), 2);
    assert!(brackets.iter().all(|l| attr(l, "y1") < 0.0));
    assert!((attr(brackets[0], "x1") - xs[0][0]).abs() < 0.001);
    assert!(attr(brackets[1], "x2") > xs[0][3]);
}

/// mn-c04-r010: the start attaches to the second beam member, not its group boundary.
#[test]
fn mn_c04_r010_dashed_above_beam_member_and_meter() {
    let score = ScoreBuilder::new()
        .time_signature(6, 8)
        .begin_beam()
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .note(p(Note::E, 5), Duration::EIGHTH)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Dashed, Placement::Above))
        .note(p(Note::G, 5), Duration::EIGHTH)
        .end_beam()
        .barline_style(BarlineStyle::Invisible)
        .time_signature_change(2, 4)
        .rest(Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    let line_svg = score.render_svg();
    let line = marked_line(&line_svg, true, true);
    assert!((attr(line, "x1") - xs[0][1]).abs() < 0.001);
    assert!(attr(line, "x2") > xs[0][3]);
    assert!(attr(line, "y1") < 0.0);
}

#[test]
fn bracket_breaks_into_two_segments_with_only_terminal_hooks() {
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::C, 5), Duration::QTR)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Dashed, Placement::Below))
        .barline_style(BarlineStyle::Invisible)
        .note(p(Note::G, 5), Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    assert_eq!(xs.len(), 2);
    let svg = score.render_svg();
    let dashed: Vec<_> = svg
        .lines()
        .filter(|l| l.contains("stroke-dasharray") && l.contains("<line "))
        .collect();
    assert_eq!(dashed.len(), 2);
    assert!((attr(dashed[0], "x1") - xs[0][0]).abs() < 0.001);
    assert!(attr(dashed[1], "x1") < xs[1][0]);
    assert!(attr(dashed[1], "x2") > xs[1][0]);
}

/// One-staff version of mn-c12-i001 dashed-line glissandi (staff hops deferred).
#[test]
fn mn_c12_i001_dashed_gliss_keeps_notehead_trim_across_system() {
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::Aes, 5), Duration::WHOLE)
        .glissando(GlissandoStyle::Dashed)
        .note(p(Note::E, 4), Duration::WHOLE)
        .glissando(GlissandoStyle::Dashed)
        .barline()
        .note(p(Note::Fis, 4), Duration::WHOLE)
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let dashed: Vec<_> = svg
        .lines()
        .filter(|l| {
            l.contains("stroke-dasharray")
                && l.contains("<line ")
                && (attr(l, "y1") - attr(l, "y2")).abs() > 0.001
        })
        .collect();
    assert_eq!(dashed.len(), 3);
    assert!(dashed[0].contains("stroke-dasharray="));
    let head = crate::render::note_renderer::notehead_advance(
        &bravura_font(),
        0,
        crate::layout::measure::NoteheadStyle::Normal,
    )
    .unwrap();
    assert!(attr(dashed[0], "x1") >= xs[0][0] + head);
    assert!(attr(dashed[0], "x2") < xs[0][1]);
    assert!(attr(dashed[1], "x1") > xs[0][1]);
    assert!(attr(dashed[2], "x2") < xs[1][0]);
}

/// mn-c05-m006 has a single zigzag from a' to g' without changing solid gliss.
#[test]
fn mn_c05_m006_zigzag_and_solid_gliss_remain_distinct() {
    let svg = ScoreBuilder::new()
        .note(p(Note::A, 4), Duration::QTR)
        .glissando(GlissandoStyle::Wavy)
        .note(p(Note::G, 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let wiggle = svg
        .lines()
        .find(|l| l.contains("<path d=\"M ") && l.contains("stroke=\"black\""))
        .unwrap();
    assert!(wiggle.matches(" L ").count() >= 3);
    assert!(svg.lines().any(|l| l.contains("<line ")
        && l.contains("stroke=\"black\"")
        && !l.contains("stroke-dasharray")
        && (attr(l, "y1") - attr(l, "y2")).abs() > 0.001));
}

#[test]
fn overlapping_rest_and_note_brackets_take_different_lanes_and_print_label() {
    let outer = bracket(AnalysisBracketStyle::Dashed, Placement::Below).label("phrase");
    let inner = bracket(AnalysisBracketStyle::Solid, Placement::Below).hooks(false, true);
    let score = ScoreBuilder::new()
        .note(p(Note::C, 5), Duration::QTR)
        .analysis_bracket_start(outer)
        .note(p(Note::D, 5), Duration::QTR)
        .analysis_bracket_start(inner)
        .rest(Duration::QTR)
        .analysis_bracket_end()
        .note(p(Note::E, 5), Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let dashed = marked_line(&svg, true, true);
    let solid = svg
        .lines()
        .find(|line| {
            line.contains("<line ")
                && !line.contains("stroke-dasharray")
                && (attr(line, "x1") - xs[0][1]).abs() < 0.001
                && (attr(line, "y1") - attr(line, "y2")).abs() < 0.001
        })
        .unwrap();
    assert!((attr(dashed, "x1") - xs[0][0]).abs() < 0.001);
    assert!(attr(solid, "x2") > xs[0][2]); // the end is at the rest glyph's right edge
    assert!(
        (attr(dashed, "y1") - attr(solid, "y1")).abs()
            >= bravura_font().engraving_config().staff_space * 0.7
    );
    assert!(svg.contains(">phrase</text>"));
    assert!(!svg.lines().any(|line| line.contains("<line ")
        && (attr(line, "x1") - xs[0][1]).abs() < 0.001
        && (attr(line, "x2") - xs[0][1]).abs() < 0.001));
}

#[test]
fn bracket_crosses_two_system_breaks_without_resetting() {
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::G, 4), Duration::QTR)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Dashed, Placement::Above))
        .barline()
        .rest(Duration::QTR)
        .barline()
        .chord(vec![p(Note::C, 5), p(Note::E, 5)], Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    assert_eq!(xs.len(), 3);
    let svg = score.clone().render_svg();
    let horizontal: Vec<_> = svg
        .lines()
        .filter(|line| {
            line.contains("stroke-dasharray")
                && line.contains("<line ")
                && (attr(line, "y1") - attr(line, "y2")).abs() < 0.001
        })
        .collect();
    assert_eq!(horizontal.len(), 3);
    assert!((attr(horizontal[0], "x1") - xs[0][0]).abs() < 0.001);
    assert!(attr(horizontal[1], "x1") < xs[1][0]);
    assert!(attr(horizontal[2], "x2") > xs[2][0]);
    let font = bravura_font();
    let page = score
        .clone()
        .page_layout(font.engraving_config().staff_space)
        .unwrap()
        .unwrap();
    assert!(horizontal
        .iter()
        .enumerate()
        .all(|(i, line)| attr(line, "y1") < page.systems[i].y));
}

/// Inline invisible barlines and tuplet marks are not bracket endpoints.
#[test]
fn tuplet_member_bracket_anchors_across_inline_barline() {
    use crate::layout::group::TupletSpec;
    let score = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::C, 5), Duration::EIGHTH)
        .note(p(Note::D, 5), Duration::EIGHTH)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Solid, Placement::Above))
        .inline_barline(BarlineStyle::Invisible)
        .rest(Duration::EIGHTH)
        .analysis_bracket_end()
        .end_tuplet()
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let segment = svg
        .lines()
        .find(|line| {
            line.contains("<line ")
                && !line.contains("stroke-dasharray")
                && (attr(line, "x1") - xs[0][1]).abs() < 0.001
                && (attr(line, "y1") - attr(line, "y2")).abs() < 0.001
        })
        .expect("bracket line on tuplet member");
    assert!(attr(segment, "x2") > xs[0][2]);
}

#[test]
fn continuation_avoids_a_local_bracket_on_the_same_staff() {
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::C, 5), Duration::QTR)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Dashed, Placement::Below))
        .note(p(Note::D, 5), Duration::QTR)
        .analysis_bracket_start(bracket(AnalysisBracketStyle::Solid, Placement::Below))
        .note(p(Note::E, 5), Duration::QTR)
        .analysis_bracket_end()
        .barline()
        .note(p(Note::F, 5), Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let xs = event_xs(&score);
    let svg = score.render_svg();
    let crossing = svg
        .lines()
        .find(|line| {
            line.contains("stroke-dasharray") && (attr(line, "x1") - xs[0][0]).abs() < 0.001
        })
        .unwrap();
    let local = svg
        .lines()
        .find(|line| {
            line.contains("<line ")
                && !line.contains("stroke-dasharray")
                && (attr(line, "x1") - xs[0][1]).abs() < 0.001
                && (attr(line, "y1") - attr(line, "y2")).abs() < 0.001
        })
        .unwrap();
    assert!(
        attr(crossing, "y1") - attr(local, "y1")
            >= bravura_font().engraving_config().staff_space * 0.75
    );
}
