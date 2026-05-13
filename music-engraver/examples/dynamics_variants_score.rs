//! Example: render the three less-common `Dynamic` variants exposed by the
//! 2026-05-13 Dynamic-expansion chunk — `Mezzo` (the bare letter-`m` glyph,
//! distinct from `Mp`/`Mf`), `SforzatoPiano` (the sforzato-prefixed "sfp",
//! distinct from `Sfp`'s sforzando-prefixed glyph), and `Z` (the rare
//! single-letter "z" sudden-accent mark).
//!
//! The score places each variant on a quarter note in a single 4/4 measure
//! plus one plain padding quarter so the visual layout is balanced. A
//! viewer can compare the rendered output against `dynamics_score.svg` for
//! `Mp`/`Mf`/`Sfp`/`Sf`/`Sfz`/`Fz` to see the subtle letterform
//! differences.
//!
//! Produces `examples/output/dynamics_variants_score.svg`.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    // The three new variants placed in source-order on an ascending C–F
    // quarter line. The fourth quarter is a plain padding note so the
    // measure closes cleanly in 4/4 and the structural delta vs the
    // dynamics-stripped baseline below is exactly 3.
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p(Note::C, 4), Duration::QTR)
        .dynamic(Dynamic::Mezzo)
        .note(p(Note::D, 4), Duration::QTR)
        .dynamic(Dynamic::SforzatoPiano)
        .note(p(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Z)
        .note(p(Note::F, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let path = out_dir.join("dynamics_variants_score.svg");
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

    // Same notes with no dynamics — each new variant must add exactly one
    // path on top of this baseline. Catches a silent missing-glyph
    // regression where a variant maps to a font slot that renders empty.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p(Note::C, 4), Duration::QTR)
        .note(p(Note::D, 4), Duration::QTR)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::F, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let plain_paths = plain.matches("<path").count();
    let delta = path_count.saturating_sub(plain_paths);
    assert_eq!(
        delta, 3,
        "the 3 new dynamic variants must add exactly 3 paths over the no-dynamic baseline \
         (full={path_count}, plain={plain_paths}, delta={delta})"
    );

    // Distinct-d guard: each new variant must contribute a unique SMuFL
    // path payload. `Mezzo`, `SforzatoPiano`, and `Z` are visually similar
    // to neighbours (`Mp`/`Mf`, `Sfp`, `Sfz`/`Fz`) but must produce
    // different path data in Bravura — see the
    // `new_variants_render_distinct_path_data_in_bravura` renderer-level
    // test for the per-glyph proof.
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
        3,
        "each new dynamic variant must contribute a unique path d-string \
         (expected 3 added, got {})",
        added.len(),
    );
}
