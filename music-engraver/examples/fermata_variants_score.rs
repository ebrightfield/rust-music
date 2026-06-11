//! Example: render every fermata variant from the SMuFL duration-coded
//! family via the ScoreBuilder `.articulation()` method. Each measure
//! exercises one variant on a whole note so the visual difference between
//! the standard semicircle, the square long, the triangle short, the
//! very-long/very-short pair, and the Henze bracket pair is easy to compare
//! in a single rendering.
//!
//! Produces `examples/output/fermata_variants_score.svg`.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    // The 7 fermata variants in canonical order. Each lands on a whole note
    // in its own measure so the glyph is easy to read; 2 measures per
    // system spreads the 7 variants across 4 systems (with one padding
    // whole-note in measure 8 to balance the final system).
    let variants = [
        Articulation::Fermata,
        Articulation::FermataLong,
        Articulation::FermataShort,
        Articulation::FermataVeryLong,
        Articulation::FermataVeryShort,
        Articulation::FermataHenzeLong,
        Articulation::FermataHenzeShort,
    ];

    // Stagger pitches so successive variants aren't all at the same staff
    // position — keeps the visual gap between glyphs honest.
    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
    ];

    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);

    let count = variants.len();
    for (i, variant) in variants.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b.note(p(n, oct), Duration::WHOLE).articulation(*variant);
        if i + 1 < count {
            b = b.barline();
        }
    }
    // Padding measure 8: a plain whole note (no articulation) so the
    // 4-system layout closes neatly.
    let svg = b
        .barline()
        .note(p(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let path = out_dir.join("fermata_variants_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count,
    );

    // Basic SVG sanity.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // Same score with no articulations — each variant must add exactly one
    // path on top of this baseline.
    let plain = {
        let mut pb = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Open)
            .time_signature(4, 4)
            .measures_per_system(2);
        for (i, _variant) in variants.iter().enumerate() {
            let (n, oct) = pitches[i];
            pb = pb.note(p(n, oct), Duration::WHOLE);
            if i + 1 < count {
                pb = pb.barline();
            }
        }
        pb.barline()
            .note(p(Note::G, 4), Duration::WHOLE)
            .end_barline()
            .render_svg()
    };
    let plain_paths = plain.matches("<path").count();
    let delta = path_count.saturating_sub(plain_paths);
    assert_eq!(
        delta,
        variants.len(),
        "each fermata variant must add exactly one path on top of the no-fermata baseline \
         (variants={}, full={path_count}, plain={plain_paths}, delta={delta})",
        variants.len(),
    );

    // Distinct-d guard: every variant must produce a unique SMuFL path
    // payload, otherwise users can't tell long/short/Henze etc. apart.
    use std::collections::HashSet;
    let mut full_d: HashSet<String> = HashSet::new();
    for chunk in svg.split("d=\"").skip(1) {
        if let Some(end) = chunk.find('"') {
            full_d.insert(chunk[..end].to_string());
        }
    }
    let mut plain_d: HashSet<String> = HashSet::new();
    for chunk in plain.split("d=\"").skip(1) {
        if let Some(end) = chunk.find('"') {
            plain_d.insert(chunk[..end].to_string());
        }
    }
    let added: HashSet<_> = full_d.difference(&plain_d).cloned().collect();
    assert_eq!(
        added.len(),
        variants.len(),
        "each fermata variant must contribute a unique path d-string: \
         expected {} added, got {}",
        variants.len(),
        added.len(),
    );
}
