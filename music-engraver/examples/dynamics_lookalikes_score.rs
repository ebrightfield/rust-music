//! Example: render the two visually-similar `Dynamic` clusters
//! side-by-side so a viewer can A/B compare the letterforms in a single
//! glance.
//!
//! - **m-cluster** (M1): `Mezzo` (bare letter-`m`), `Mp`, `Mf`. The `Mp`
//!   and `Mf` glyphs are dedicated composites that look superficially like
//!   `Mezzo` + a second letter, but Bravura draws them as a single tighter
//!   shape with a different weight balance — adjacent placement makes the
//!   distinction visible.
//! - **sfp-cluster** (M2): `Sfp` (sforzando-prefixed) and `SforzatoPiano`
//!   (sforzato-prefixed). Both spell "sfp" but use a different `s`
//!   letterform; only side-by-side rendering makes the difference easy to
//!   read at typical staff sizes.
//!
//! Layout: 2 measures of 4 quarter notes each on a treble staff. The
//! lookalike dynamics sit on the lookalike-bearing notes; the remaining
//! quarters are plain padding so the measure closes cleanly in 4/4 and
//! the structural delta vs the dynamics-stripped baseline is exactly the
//! number of attached dynamics (5).
//!
//! Produces `examples/output/dynamics_lookalikes_score.svg`.

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

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: m-cluster lookalikes — Mezzo, Mp, Mf on three adjacent quarters
        // plus a plain padding quarter so the visual focus stays on the
        // m-shaped glyphs.
        .note(p(Note::G, 4), Duration::QTR)
        .dynamic(Dynamic::Mezzo)
        .note(p(Note::A, 4), Duration::QTR)
        .dynamic(Dynamic::Mp)
        .note(p(Note::B, 4), Duration::QTR)
        .dynamic(Dynamic::Mf)
        .note(p(Note::C, 5), Duration::QTR)
        .barline()
        // M2: sfp-cluster lookalikes — Sfp and SforzatoPiano on two adjacent
        // quarters plus two plain padding quarters. The cluster is only two
        // glyphs but the visual difference between the two `s` letterforms
        // is the whole point of this example, so they get the spotlight.
        .note(p(Note::G, 4), Duration::QTR)
        .dynamic(Dynamic::Sfp)
        .note(p(Note::A, 4), Duration::QTR)
        .dynamic(Dynamic::SforzatoPiano)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::C, 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let path = out_dir.join("dynamics_lookalikes_score.svg");
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

    // ---- Assertion 1: exact 5-path delta vs the dynamics-stripped baseline
    // (Mezzo, Mp, Mf, Sfp, SforzatoPiano = 5). Catches a silent
    // missing-glyph regression — if Bravura ever swapped one of these
    // glyphs to `none` or the renderer dropped it, the delta would be <5.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::C, 5), Duration::QTR)
        .barline()
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::C, 5), Duration::QTR)
        .end_barline()
        .render_svg();
    let plain_paths = plain.matches("<path").count();
    let delta = path_count.saturating_sub(plain_paths);
    assert_eq!(
        delta, 5,
        "the 5 lookalike dynamics must add exactly 5 paths over the no-dynamic baseline \
         (full={path_count}, plain={plain_paths}, delta={delta})"
    );

    // ---- Assertion 2: each lookalike must contribute a unique d-string.
    // The whole point of putting these glyphs side-by-side is that they
    // are visually similar but provably distinct in Bravura. If two
    // collapsed (e.g. a future font swap aliased `Mezzo` to `Mp`'s glyph)
    // the comparison example would mislead — this assertion catches it.
    use std::collections::HashSet;
    fn distinct_d(svg: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for chunk in svg.split("d=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                out.insert(chunk[..end].to_string());
            }
        }
        out
    }
    let added: HashSet<_> = distinct_d(&svg)
        .difference(&distinct_d(&plain))
        .cloned()
        .collect();
    assert_eq!(
        added.len(),
        5,
        "each lookalike dynamic must contribute a unique path d-string \
         (no glyph collapse in either cluster), got {}",
        added.len()
    );
}
