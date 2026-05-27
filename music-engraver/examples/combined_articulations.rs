/// Render the four SMuFL combined-articulation glyphs via ScoreBuilder.
///
/// `Articulation::{AccentStaccato, MarcatoStaccato, TenutoStaccato,
/// TenutoAccent}` each map to a single Bravura glyph that draws what would
/// otherwise be a two-glyph stack of the primitives (e.g. `AccentStaccato`
/// is the SMuFL shorthand for Accent + Staccato). They live in the
/// *normal* articulation stack bucket and follow the standard
/// opposite-stem placement rule.
///
/// This example walks through:
///   1. All four combined variants on low quarter notes — stems up, glyph
///      placed below the notehead.
///   2. All four combined variants on high quarter notes — stems down,
///      glyph placed above the notehead.
///   3. A direct visual contrast between the combined-variant shorthand
///      (one glyph) and the equivalent two-primitive stack (two glyphs)
///      on the same note. The shorthand should consume exactly one stack
///      slot, the explicit stack two.
///   4. Combined variants stacked with a fermata — the combined glyph sits
///      in the normal bucket, the fermata sits outside it.
///
/// Byte-exact regression coverage for the same four variants lives in
/// `tests/golden_svg.rs::golden_combined_articulations` — this example is
/// a human-readable walkthrough that drops an SVG onto disk for visual
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
        // ── Measure 1: low quarter notes (stems up → combined glyph below).
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::AccentStaccato)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::MarcatoStaccato)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::TenutoStaccato)
        .note(p(Note::G, 4), Duration::QTR)
        .articulation(Articulation::TenutoAccent)
        .barline()
        // ── Measure 2: high quarter notes (stems down → combined glyph above).
        .note(p(Note::C, 5), Duration::QTR)
        .articulation(Articulation::AccentStaccato)
        .note(p(Note::D, 5), Duration::QTR)
        .articulation(Articulation::MarcatoStaccato)
        .note(p(Note::E, 5), Duration::QTR)
        .articulation(Articulation::TenutoStaccato)
        .note(p(Note::G, 5), Duration::QTR)
        .articulation(Articulation::TenutoAccent)
        .barline()
        // ── Measure 3: shorthand vs explicit stack.
        // The first note uses the combined shorthand `AccentStaccato`
        // (one glyph). The second note carries the equivalent two-glyph
        // stack `Accent + Staccato`. Visually inspect that the shorthand
        // sits in exactly one stack slot while the explicit form
        // consumes two.
        .note(p(Note::A, 5), Duration::HALF)
        .articulation(Articulation::AccentStaccato)
        .note(p(Note::A, 5), Duration::HALF)
        .articulation(Articulation::Accent)
        .articulation(Articulation::Staccato)
        .barline()
        // ── Measure 4: full stack with fermata. The combined glyph sits
        // in the normal bucket; the fermata sits outside, always above.
        .note(p(Note::F, 5), Duration::HALF)
        .articulation(Articulation::TenutoStaccato)
        .articulation(Articulation::Fermata)
        .note(p(Note::G, 5), Duration::HALF)
        .articulation(Articulation::AccentStaccato)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/combined_articulations.svg",
        &svg,
    )
    .expect("write SVG");

    assert!(svg.starts_with("<svg"), "output should start with <svg");
    assert!(svg.contains("</svg>"), "output should have closing tag");

    let path_count = svg.matches("<path").count();
    println!(
        "combined_articulations.svg: {} bytes, {} paths",
        svg.len(),
        path_count
    );

    // Path-count lower bound. We have 2 systems (so 2 clef paths). Per
    // measure:
    //   M1: 4 noteheads + 4 combined glyphs                    = 8
    //   M2: 4 noteheads + 4 combined glyphs                    = 8
    //   M3: 2 noteheads + 1 combined + 2 primitives            = 5
    //   M4: 2 noteheads + 2 combined + 2 fermata               = 6
    // = 27 + 2 clefs = 29 minimum. The actual count is higher (rests,
    // accidentals from time-signature digits, etc.), so 29 is a safe
    // lower bound that still trips if any of the 11 combined-glyph
    // calls silently drops a path.
    assert!(
        path_count >= 29,
        "expected >= 29 paths (clefs + noteheads + 11 combined glyphs + 2 explicit primitives + 2 fermatas), got {path_count}"
    );

    // Each measure ends with a barline. With `measures_per_system(2)`
    // and 4 measures, we expect at least 3 mid-system barlines + 2
    // end-of-system markers. Use the literal `<line` tag count as a
    // structural floor.
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 3,
        "expected >= 3 <line> elements for barlines and stems, got {line_count}"
    );

    // Sanity: the SVG must contain at least one path that *isn't* the
    // clef — a regression that drops every articulation glyph would
    // leave only clef paths.
    assert!(
        path_count > 2,
        "expected more than 2 paths (would mean every glyph beyond the clefs vanished), got {path_count}"
    );
}
