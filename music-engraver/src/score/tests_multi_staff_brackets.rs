use super::MultiStaffScore;
use crate::layout::analysis_bracket::{AnalysisBracketSpec, AnalysisBracketStyle};
use crate::layout::placement::Placement;
use crate::score::ScoreBuilder;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

fn attr(line: &str, key: &str) -> f64 {
    let start = line.find(&format!("{key}=\"")).expect("SVG coordinate") + key.len() + 2;
    line[start..].split('"').next().unwrap().parse().unwrap()
}

#[test]
fn upper_stave_bracket_crosses_system_boundary_without_entering_lower_stave() {
    let upper = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .analysis_bracket_start(AnalysisBracketSpec::new(
            AnalysisBracketStyle::Dashed,
            Placement::Above,
        ))
        .barline()
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .analysis_bracket_end()
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .rest(Duration::WHOLE)
        .barline()
        .rest(Duration::WHOLE)
        .end_barline();
    let svg = MultiStaffScore::grand_staff(upper, lower)
        .system_width_fu(18_000.0)
        .measures_per_system(1)
        .render_svg();

    let dashed: Vec<_> = svg
        .lines()
        .filter(|line| line.contains("<line ") && line.contains("stroke-dasharray="))
        .filter(|line| (attr(line, "y1") - attr(line, "y2")).abs() < 0.001)
        .collect();
    assert_eq!(dashed.len(), 2, "one continuation segment per system");
    assert!(dashed
        .iter()
        .all(|line| attr(line, "x2") > attr(line, "x1")));
    assert!(attr(dashed[1], "y1") > attr(dashed[0], "y1"));
    let mut staff_rows: Vec<f64> = svg
        .lines()
        .filter(|line| line.contains("<line ") && !line.contains("stroke-dasharray="))
        .filter(|line| {
            (attr(line, "y1") - attr(line, "y2")).abs() < 0.001
                && attr(line, "x2") - attr(line, "x1") > 10_000.0
        })
        .map(|line| attr(line, "y1"))
        .collect();
    staff_rows.sort_by(f64::total_cmp);
    assert_eq!(
        staff_rows.len(),
        20,
        "two systems with two five-line staves"
    );
    assert!(attr(dashed[0], "y1") < staff_rows[0]);
    assert!(attr(dashed[1], "y1") < staff_rows[10]);
    assert!(attr(dashed[1], "y1") > staff_rows[5]);
}
