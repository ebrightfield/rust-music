//! Renders a 5-staff string-quartet-plus-piano-style score using
//! `MultiStaffScore::with_sub_brackets(...)` — the score-level surface
//! exposing the nested grouping landed in the layout/render layer.
//!
//! The 5 staves are arranged as a single main section bracket with two
//! nested sub-brackets:
//!   * staves 0..2 — e.g. Violin I + Violin II share a sub-bracket
//!   * staves 3..5 — e.g. Viola + Cello + Bass share a sub-bracket
//!
//! Produces `examples/output/sub_brackets_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::multi_staff::SubBracket;
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn line(clef: Clef, pitches: &[(Note, u8)]) -> ScoreBuilder {
    let mut b = ScoreBuilder::new().clef(clef).time_signature(4, 4);
    for (n, oct) in pitches {
        b = b.note(p(*n, *oct), Duration::QTR);
    }
    b.end_barline()
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let v1 = line(Clef::Treble, &[(Note::E, 5), (Note::G, 5), (Note::A, 5), (Note::B, 5)]);
    let v2 = line(Clef::Treble, &[(Note::C, 5), (Note::E, 5), (Note::F, 5), (Note::G, 5)]);
    let va = line(Clef::Treble, &[(Note::G, 4), (Note::A, 4), (Note::B, 4), (Note::C, 5)]);
    let vc = line(Clef::Bass, &[(Note::E, 3), (Note::G, 3), (Note::A, 3), (Note::B, 3)]);
    let cb = line(Clef::Bass, &[(Note::E, 2), (Note::E, 2), (Note::E, 2), (Note::E, 2)]);

    let svg = MultiStaffScore::section(vec![v1, v2, va, vc, cb])
        .with_sub_brackets(vec![
            SubBracket { start_index: 0, staff_count: 2 },
            SubBracket { start_index: 2, staff_count: 3 },
        ])
        .render_svg();

    let path = out_dir.join("sub_brackets_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count
    );

    // Structural sanity: should be a valid SVG with the two scroll glyphs
    // for the main bracket, plus a thin line for each sub-bracket.
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    // After the leftward shift the main bracket scrolls anchor at x=-315
    // (= -(BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS
    // + SUB_BRACKET_GAP_SS) * staff_space = -1.26 sp * 250 fu).
    assert!(
        svg.contains("translate(-315,"),
        "main bracket should be shifted left to make room for sub-brackets"
    );
    // The two sub-brackets each add one thin <line>; their thickness
    // matches the SMuFL `subBracketThickness` default of 0.16 sp = 40 fu.
    let thin_line_hits = svg.matches("stroke-width=\"40\"").count();
    assert!(
        thin_line_hits >= 2,
        "expected ≥2 sub-bracket lines at stroke-width=40 fu, got {thin_line_hits}"
    );
}
