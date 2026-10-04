//! Public model → layout → render tests for the alto and tenor C clefs
//! (RM-MN-003). Each test renders through `ScoreBuilder` / `MultiStaffScore`
//! and reads glyph identity and position back out of the SVG.

use super::*;
use crate::font::bravura_font;
use crate::score::multi_staff::MultiStaffScore;
use music::note::note::Note;
use smufl::Glyph;

/// Bravura staff space in font design units.
const SS: f64 = 250.0;

fn path_data(glyph: Glyph) -> String {
    bravura_font().glyph_outline(glyph).unwrap().path_data
}

/// `(x, y)` translate of every `<path>` whose outline is exactly `glyph`,
/// in document order.
fn glyph_origins(svg: &str, glyph: Glyph) -> Vec<(f64, f64)> {
    let needle = format!("<path d=\"{}\"", path_data(glyph));
    svg.match_indices(&needle)
        .map(|(i, _)| {
            let rest = &svg[i + needle.len()..];
            let t = rest.split("translate(").nth(1).expect("glyph transform");
            let inner = t.split(')').next().unwrap();
            let mut parts = inner.split(',').map(|v| v.trim().parse::<f64>().unwrap());
            (parts.next().unwrap(), parts.next().unwrap())
        })
        .collect()
}

/// Glyph y coordinates ordered left to right.
fn glyph_ys_by_x(svg: &str, glyph: Glyph) -> Vec<f64> {
    let mut origins = glyph_origins(svg, glyph);
    origins.sort_by(|a, b| a.0.total_cmp(&b.0));
    origins.into_iter().map(|(_, y)| y).collect()
}

/// `(x1, x2, y)` of every horizontal `<line>`.
fn horizontal_lines(svg: &str) -> Vec<(f64, f64, f64)> {
    svg.split("<line ")
        .skip(1)
        .filter_map(|l| {
            let attr = |name: &str| -> f64 {
                let key = format!("{name}=\"");
                let v = l.split(&key).nth(1).unwrap();
                v[..v.find('"').unwrap()].parse().unwrap()
            };
            let (y1, y2) = (attr("y1"), attr("y2"));
            (y1 == y2).then(|| (attr("x1"), attr("x2"), y1))
        })
        .collect()
}

/// Top-line y of each rendered staff, top to bottom. Staff lines are the long
/// horizontal lines; ledger lines are only a notehead wide.
fn staff_tops(svg: &str) -> Vec<f64> {
    let mut ys: Vec<f64> = horizontal_lines(svg)
        .into_iter()
        .filter(|(x1, x2, _)| x2 - x1 > 10.0 * SS)
        .map(|(_, _, y)| y)
        .collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup();
    assert_eq!(ys.len() % 5, 0, "staff lines come in fives: {ys:?}");
    ys.chunks(5)
        .map(|c| {
            for w in c.windows(2) {
                assert_eq!(w[1] - w[0], SS, "staff lines one staff space apart");
            }
            c[0]
        })
        .collect()
}

/// y of staff position `pos` (0 = bottom line, 8 = top line).
fn y_of(top: f64, pos: i8) -> f64 {
    top + f64::from(8 - pos) * SS / 2.0
}

fn assert_only_c_clefs(svg: &str, count: usize) {
    assert_eq!(
        glyph_origins(svg, Glyph::CClef).len(),
        count,
        "C clef count"
    );
    for other in [Glyph::GClef, Glyph::GClef8Va, Glyph::GClef8Vb, Glyph::FClef] {
        assert!(
            glyph_origins(svg, other).is_empty(),
            "no {other:?} may stand in for a C clef"
        );
    }
}

#[test]
fn alto_score_centres_c_clef_on_middle_line_and_places_notes() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 3), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::D, 3), Duration::QTR)
        .render_svg();
    let tops = staff_tops(&svg);
    assert_eq!(tops.len(), 1);
    let top = tops[0];

    assert_only_c_clefs(&svg, 1);
    assert_eq!(glyph_origins(&svg, Glyph::CClef)[0].1, y_of(top, 4));
    assert_eq!(
        glyph_ys_by_x(&svg, Glyph::NoteheadBlack),
        vec![y_of(top, 4), y_of(top, 0), y_of(top, 8), y_of(top, -2)]
    );
    // D3 needs exactly one ledger line, one staff space below the bottom line.
    let ledgers: Vec<f64> = horizontal_lines(&svg)
        .into_iter()
        .filter(|(x1, x2, _)| x2 - x1 < 4.0 * SS)
        .map(|(_, _, y)| y)
        .collect();
    assert_eq!(ledgers, vec![y_of(top, -2)]);
}

#[test]
fn tenor_score_centres_c_clef_on_fourth_line_and_places_notes() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Tenor)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::D, 3), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .render_svg();
    let top = staff_tops(&svg)[0];

    assert_only_c_clefs(&svg, 1);
    assert_eq!(glyph_origins(&svg, Glyph::CClef)[0].1, y_of(top, 6));
    assert_eq!(
        glyph_ys_by_x(&svg, Glyph::NoteheadBlack),
        vec![y_of(top, 6), y_of(top, 0), y_of(top, 8), y_of(top, 10)]
    );
    // G4 needs exactly one ledger line, one staff space above the top line.
    let ledgers: Vec<f64> = horizontal_lines(&svg)
        .into_iter()
        .filter(|(x1, x2, _)| x2 - x1 < 4.0 * SS)
        .map(|(_, _, y)| y)
        .collect();
    assert_eq!(ledgers, vec![y_of(top, 10)]);
}

/// Key-signature accidentals rendered through the public builder follow the
/// conventional C-clef patterns (see `layout::key_signature`).
#[test]
fn c_clef_key_signatures_render_at_conventional_positions() {
    let cases: [(Clef, KeySignature, Glyph, [i8; 7]); 4] = [
        (
            Clef::Alto,
            KeySignature::Sharps(7),
            Glyph::AccidentalSharp,
            [7, 4, 8, 5, 2, 6, 3],
        ),
        (
            Clef::Alto,
            KeySignature::Flats(7),
            Glyph::AccidentalFlat,
            [3, 6, 2, 5, 1, 4, 0],
        ),
        (
            Clef::Tenor,
            KeySignature::Sharps(7),
            Glyph::AccidentalSharp,
            [2, 6, 3, 7, 4, 8, 5],
        ),
        (
            Clef::Tenor,
            KeySignature::Flats(7),
            Glyph::AccidentalFlat,
            [5, 8, 4, 7, 3, 6, 2],
        ),
    ];
    for (clef, key, glyph, positions) in cases {
        // A rest keeps the measure free of note accidentals.
        let svg = ScoreBuilder::new()
            .clef(clef)
            .key_signature(key.clone())
            .rest(Duration::WHOLE)
            .render_svg();
        let top = staff_tops(&svg)[0];
        let expected: Vec<f64> = positions.iter().map(|&p| y_of(top, p)).collect();
        assert_eq!(glyph_ys_by_x(&svg, glyph), expected, "{clef:?} {key:?}");
    }
}

/// Per-staff clefs survive multi-staff layout: a viola (alto) and a cello in
/// tenor clef each get a C clef on their own reference line, and middle C
/// lands on that staff's reference line.
#[test]
fn multi_staff_section_renders_alto_and_tenor_per_staff() {
    let line = |clef: Clef| {
        ScoreBuilder::new()
            .clef(clef)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4), Duration::WHOLE)
            .end_barline()
    };
    let svg = MultiStaffScore::section(vec![line(Clef::Alto), line(Clef::Tenor)]).render_svg();
    let tops = staff_tops(&svg);
    assert_eq!(tops.len(), 2);

    assert_only_c_clefs(&svg, 2);
    let mut clef_ys: Vec<f64> = glyph_origins(&svg, Glyph::CClef)
        .into_iter()
        .map(|(_, y)| y)
        .collect();
    clef_ys.sort_by(f64::total_cmp);
    assert_eq!(clef_ys, vec![y_of(tops[0], 4), y_of(tops[1], 6)]);

    let mut note_ys: Vec<f64> = glyph_origins(&svg, Glyph::NoteheadWhole)
        .into_iter()
        .map(|(_, y)| y)
        .collect();
    note_ys.sort_by(f64::total_cmp);
    assert_eq!(note_ys, vec![y_of(tops[0], 4), y_of(tops[1], 6)]);
}
