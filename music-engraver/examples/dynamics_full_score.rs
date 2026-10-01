/// Render every variant in `Dynamic::ALL` (28 dynamics) — one per quarter
/// note, 4 per measure across 7 measures (2 measures per system). Acts as
/// the visual proofing companion to the renderer unit-test
/// `all_dynamics_produce_distinct_svg_output` and the per-variant
/// distinctness tests in `render::dynamics_renderer`.
///
/// Produces `examples/output/dynamics_full_score.svg`. Asserts the
/// expected number of distinct dynamic glyph paths land in the output.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    // 28 pitches: two passes through C4-B4-C5-B5 so noteheads stay within
    // the staff's comfortable range and don't introduce ledger lines below
    // the staff (which would crowd the dynamics' below-staff baseline).
    let pitches: [(Note, i8); 28] = [
        (Note::C, 4),
        (Note::D, 4),
        (Note::E, 4),
        (Note::F, 4),
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
        (Note::B, 5),
        (Note::C, 4),
        (Note::D, 4),
        (Note::E, 4),
        (Note::F, 4),
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
        (Note::B, 5),
    ];

    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);

    for (i, dynamic) in Dynamic::ALL.iter().enumerate() {
        let (note, octave) = pitches[i];
        b = b
            .note(Pitch::new(note, octave), Duration::QTR)
            .dynamic(*dynamic);
        // Barline after every 4th note, except the last (closed by end_barline).
        if (i + 1) % 4 == 0 && (i + 1) < pitches.len() {
            b = b.barline();
        }
    }
    let svg = b.end_barline().render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/dynamics_full_score.svg",
        &svg,
    )
    .expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "dynamics_full_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // Structural validity
    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(svg.contains("</svg>"), "output should close SVG");

    // 28 noteheads + 28 dynamic glyphs + ≥4 clefs (one per system, 4 systems)
    // ≥ 60 paths conservatively.
    assert!(
        path_count >= 60,
        "expected ≥60 paths (28 noteheads + 28 dynamics + ≥4 clefs), got {path_count}"
    );

    // Independent path-distinctness check: 28 dynamic glyphs are all
    // pairwise distinct in Bravura per the renderer unit tests, so the
    // global distinct-d count must include at least 28 unique dynamic
    // payloads. Combined with notehead/clef paths, the total must be ≥ 29.
    use std::collections::HashSet;
    let mut distinct_d = HashSet::new();
    for chunk in svg.split("d=\"").skip(1) {
        if let Some(end) = chunk.find('"') {
            distinct_d.insert(&chunk[..end]);
        }
    }
    assert!(
        distinct_d.len() >= 29,
        "expected ≥29 distinct path d-strings (28 unique dynamic glyphs + ≥1 notehead), got {}",
        distinct_d.len()
    );
}
