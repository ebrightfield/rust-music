/// Render the four accent-extension family articulations via ScoreBuilder.
///
/// `Articulation::{SoftAccent, Stress, Unstress, LaissezVibrer}` are the
/// four "accent-extension" post-v1 variants. All four:
///   * live in the *normal* articulation stack bucket (alongside
///     Staccato/Accent/Tenuto/Marcato/etc.) — not the fermata bucket,
///     not the bow-stroke bucket;
///   * follow the standard stem-opposite default placement (stem-up →
///     glyph below the notehead, stem-down → glyph above);
///   * select between distinct `Above` and `Below` SMuFL glyphs via
///     `glyph(placement)`.
///
/// Semantically they are heterogeneous: `SoftAccent` is a parenthesized
/// accent (a gentle emphasis), `Stress` / `Unstress` are prosodic
/// emphasis marks, and `LaissezVibrer` is the "l.v." let-ring symbol
/// (usually a short curved tie). Visual contrast makes that clear.
///
/// This example walks through:
///   1. All four variants on low quarter notes — stems up, glyphs below.
///   2. All four variants on high quarter notes — stems down, glyphs above.
///   3. Each variant stacked with a `Fermata`. The accent-extension glyph
///      sits in the normal bucket (stem-opposite); the fermata sits
///      outside it, always above. Catches a regression that would
///      misroute any of the four into the fermata bucket.
///   4. Each variant stacked with an `UpBow`. Bow strokes always render
///      above; with stem-up notes the accent extension sits below and
///      the bow above. The stack splitter must separate them onto
///      opposite sides of the staff.
///
/// Byte-exact regression coverage for the same four variants lives in
/// `tests/golden_svg.rs::golden_accent_extensions` — this example is a
/// human-readable walkthrough that drops an SVG onto disk for visual
/// inspection.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn p(name: Note, octave: u8) -> Pitch {
    Pitch::new(name, octave).expect("valid pitch")
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // ── Measure 1: low quarter notes. Stem direction is up by
        // default for notes below the middle line, so the accent-
        // extension glyph is placed below the notehead (the `Below`
        // glyph arm).
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::SoftAccent)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::Stress)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::Unstress)
        .note(p(Note::G, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .barline()
        // ── Measure 2: high quarter notes. Stem direction is down by
        // default for notes above the middle line, so the accent-
        // extension glyph is placed above the notehead (the `Above`
        // glyph arm). Together M1 and M2 exercise both placement
        // arms for every variant on the same canvas.
        .note(p(Note::C, 5), Duration::QTR)
        .articulation(Articulation::SoftAccent)
        .note(p(Note::D, 5), Duration::QTR)
        .articulation(Articulation::Stress)
        .note(p(Note::E, 5), Duration::QTR)
        .articulation(Articulation::Unstress)
        .note(p(Note::G, 5), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .barline()
        // ── Measure 3: accent-extension + Fermata stack. With stem-up
        // notes (low pitches), the accent extension sits below and the
        // fermata sits above. Verifies the two glyphs route into
        // distinct buckets — a regression that put SoftAccent /
        // Stress / Unstress / LaissezVibrer into the fermata bucket
        // would collapse them on top of the fermata above the note.
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::SoftAccent)
        .articulation(Articulation::Fermata)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::Stress)
        .articulation(Articulation::Fermata)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::Unstress)
        .articulation(Articulation::Fermata)
        .note(p(Note::G, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .articulation(Articulation::Fermata)
        .barline()
        // ── Measure 4: accent-extension + UpBow stack. Bow strokes
        // are always-above; with stem-up notes the accent extension
        // sits below and the bow above. The stack splitter must put
        // them on opposite sides. A regression that misrouted the
        // accent extension into the bow bucket would stack the two
        // glyphs on the same side above the note.
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::SoftAccent)
        .articulation(Articulation::UpBow)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::Stress)
        .articulation(Articulation::UpBow)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::Unstress)
        .articulation(Articulation::UpBow)
        .note(p(Note::G, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .articulation(Articulation::UpBow)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/accent_extensions.svg",
        &svg,
    )
    .expect("write SVG");

    assert!(svg.starts_with("<svg"), "output should start with <svg");
    assert!(svg.contains("</svg>"), "output should have closing tag");

    let path_count = svg.matches("<path").count();
    println!(
        "accent_extensions.svg: {} bytes, {} paths",
        svg.len(),
        path_count
    );

    // Path-count lower bound. With `measures_per_system(2)` and 4
    // measures we get 2 systems → 2 clef paths. Per measure:
    //   M1: 4 noteheads + 4 accent-extension glyphs            = 8
    //   M2: 4 noteheads + 4 accent-extension glyphs            = 8
    //   M3: 4 noteheads + 4 extensions + 4 fermatas            = 12
    //   M4: 4 noteheads + 4 extensions + 4 up-bow glyphs       = 12
    // = 40 + 2 clefs = 42 minimum. Use 42 as a tight floor:
    // anything less than 42 means at least one of the 16 expected
    // accent-extension / fermata / bow-stroke glyphs was silently
    // dropped by a regression.
    assert!(
        path_count >= 42,
        "expected >= 42 paths (clefs + noteheads + 16 accent-extension/fermata/bow glyphs), got {path_count}"
    );

    // Each measure ends with a barline. With `measures_per_system(2)`
    // and 4 measures, we expect at least 3 mid-system barlines + 2
    // end-of-system markers. Use the literal `<line` tag count as a
    // structural floor that also catches stems and staff segments.
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 3,
        "expected >= 3 <line> elements for barlines and stems, got {line_count}"
    );

    // Sanity: every glyph beyond the clefs should be present — a
    // catastrophic regression that dropped every articulation would
    // leave only clef paths plus noteheads/rests (~18 paths). The
    // floor of 42 above already trips that, but assert > 2 explicitly
    // so the diagnostic message is unambiguous.
    assert!(
        path_count > 2,
        "expected more than 2 paths (would mean every glyph beyond the clefs vanished), got {path_count}"
    );
}
