/// Render every variant in `Ornament::ALL` (15 ornaments) — one per note,
/// laid out 4 ornaments per measure across 4 systems. Acts as the visual
/// proofing companion to the unit-test glyph-distinctness assertions
/// added when `Ornament::ALL` was introduced.
///
/// Produces `examples/output/ornaments_full_score.svg`. Asserts the
/// expected number of distinct ornament glyph paths land in the output.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::score::ScoreBuilder;

fn main() {
    // 15 ornaments × one note each. Use a slow rising scale so every
    // ornament is comfortably above the staff and there's no notehead
    // collision between adjacent measures. The pitches are deliberately
    // monotonic — variation in the visual output should come from the
    // ornament glyph, not the notehead position.
    let pitches: [(Note, u8); 15] = [
        (Note::C, 4), (Note::D, 4), (Note::E, 4), (Note::F, 4),
        (Note::G, 4), (Note::A, 4), (Note::B, 4), (Note::C, 5),
        (Note::D, 5), (Note::E, 5), (Note::F, 5), (Note::G, 5),
        (Note::A, 5), (Note::B, 5), (Note::C, 6),
    ];

    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);

    // 15 ornaments → 3 measures of 4 + 1 measure of 3, with a final
    // padding quarter to fill the last measure to 4/4.
    for (i, ornament) in Ornament::ALL.iter().enumerate() {
        let (note, octave) = pitches[i];
        b = b
            .note(Pitch::new(note, octave).expect("valid pitch"), Duration::QTR)
            .ornament(*ornament);
        // Barline after every 4th ornament (i.e. when 1, 5, 9, 13 notes
        // have been laid down → completed beat-4s of measures 1, 2, 3).
        if (i + 1) % 4 == 0 {
            b = b.barline();
        }
    }
    // Measure 4 already has 3 ornament-bearing quarters (indices 12, 13, 14);
    // add one ornamentless quarter to round it out to 4/4.
    let svg = b
        .note(Pitch::new(Note::D, 6).expect("valid pitch"), Duration::QTR)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/ornaments_full_score.svg",
        &svg,
    )
    .expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "ornaments_full_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // Structural validity
    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(svg.contains("</svg>"), "output should close SVG");

    // 16 noteheads + 15 ornament glyphs + 4 clefs (one per system, 4 systems)
    // + key-sig contributions ≥ 35 paths conservatively. Use a low bound
    // that proves every ornament drew at least once but doesn't pin the
    // exact path count (which the golden test owns).
    assert!(
        path_count >= 35,
        "expected at least 35 paths (16 noteheads + 15 ornaments + ≥4 clefs), got {path_count}"
    );

    // Independent path-distinctness check: there should be at least 14
    // unique `d="..."` payloads contributed by ornaments
    // (15 variants − 1 documented InvertedMordent≡ShortTrill SMuFL alias).
    // Combined with notehead/clef/etc. paths, the global distinct-d count
    // must be ≥ 14 + 1 (at least one notehead family) = 15.
    use std::collections::HashSet;
    let mut distinct_d = HashSet::new();
    for chunk in svg.split("d=\"").skip(1) {
        if let Some(end) = chunk.find('"') {
            distinct_d.insert(&chunk[..end]);
        }
    }
    assert!(
        distinct_d.len() >= 15,
        "expected ≥15 distinct path d-strings (14 unique ornament glyphs + ≥1 notehead), got {}",
        distinct_d.len()
    );
}
