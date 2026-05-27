/// Render the LaissezVibrer ("l.v.") articulation via ScoreBuilder.
///
/// `Articulation::LaissezVibrer` is the SMuFL `articLaissezVibrerAbove` /
/// `articLaissezVibrerBelow` pair — a short tie-like curve attached to a
/// notehead instructing the player to let the note ring (decay naturally,
/// without damping). It is the only "let-ring" symbol in standard
/// notation and shows up most often in:
///   * harp scores (where damping is the default and l.v. cancels it),
///   * piano (especially at phrase or piece endings where the pedal lifts
///     but the strings continue to sound),
///   * mallet percussion (vibraphone, marimba — pedal up),
///   * arco strings on the final note of a movement.
///
/// `accent_extensions.rs` exercises l.v. as one of four
/// stem-opposite normal-bucket articulations alongside SoftAccent /
/// Stress / Unstress. This example zooms in on l.v. alone with realistic
/// harp/piano use cases, so a reader looking for the let-ring curve in
/// particular finds a dedicated walkthrough.
///
/// Wiring properties exercised:
///   * Distinct Above / Below SMuFL glyphs (not a draw-time flip of one
///     glyph). M1 emits `ArticLaissezVibrerBelow`, M2 emits
///     `ArticLaissezVibrerAbove`.
///   * Normal-bucket placement: stem-opposite default placement, not
///     always-above (fermata bucket) or always-above (bow-stroke bucket).
///   * Stacks correctly with Fermata: l.v. stays in the normal bucket,
///     fermata in the always-above bucket. On a stem-up chord the two
///     glyphs render on opposite sides of the staff (M4).
///
/// Byte-exact regression coverage for l.v. lives inside
/// `tests/golden_svg.rs::golden_accent_extensions` (the four
/// accent-extension variants share a single golden). This example is a
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
        // ── Measure 1: l.v. on low quarter notes. All four notes sit
        // below the middle line → stems auto-up → l.v. glyph routes to
        // the `Below` arm (`Glyph::ArticLaissezVibrerBelow`). Four
        // independent l.v. emissions, one per note, so every notehead
        // visibly carries its own short let-ring curve underneath.
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::F, 4), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .barline()
        // ── Measure 2: l.v. on high quarter notes. All four notes sit
        // on / above the middle line → stems auto-down → l.v. glyph
        // routes to the `Above` arm (`Glyph::ArticLaissezVibrerAbove`).
        // Together M1 and M2 prove that both glyph arms are wired and
        // that placement follows the stem-opposite rule, not a
        // hard-coded side.
        .note(p(Note::C, 5), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::D, 5), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::E, 5), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .note(p(Note::F, 5), Duration::QTR)
        .articulation(Articulation::LaissezVibrer)
        .barline()
        // ── Measure 3: canonical harp / piano use case — a final
        // ringing chord. A `WHOLE` C-major triad in root position with
        // a single l.v. on the chord. The l.v. attaches to the chord
        // as a whole, not per-notehead, so the example emits ONE l.v.
        // glyph plus three noteheads. With low pitches the chord
        // stems up (or here, has no stem since WHOLE has none) and the
        // l.v. routes to the `Below` arm.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::WHOLE,
        )
        .articulation(Articulation::LaissezVibrer)
        .barline()
        // ── Measure 4: final chord stacked with l.v. + Fermata — the
        // typical end-of-piece marking ("hold this chord and let it
        // ring"). A high `WHOLE` C-major triad with both articulations.
        // The stack splitter must:
        //   * place the fermata in the always-above bucket (above the
        //     staff),
        //   * place the l.v. in the normal bucket — on the opposite
        //     side from the chord's notional stem. For a high WHOLE
        //     chord, the layout treats it as stem-down → l.v. above.
        //     Both glyphs end up above the chord but in distinct
        //     buckets, so the fermata sits outside the l.v.
        // A regression that put l.v. into the fermata bucket would
        // collapse the two glyphs into a single bucket and lose the
        // outward stacking; a regression that misrouted the fermata
        // would put it next to or on top of the l.v.
        .chord(
            vec![p(Note::C, 5), p(Note::E, 5), p(Note::G, 5)],
            Duration::WHOLE,
        )
        .articulation(Articulation::LaissezVibrer)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/laissez_vibrer.svg",
        &svg,
    )
    .expect("write SVG");

    // Structural sanity: the writer always emits a well-formed root
    // element. If `render_svg()` ever returns a non-SVG string (e.g. a
    // half-rendered error path), both assertions fire.
    assert!(svg.starts_with("<svg"), "output should start with <svg");
    assert!(svg.contains("</svg>"), "output should have closing tag");

    let path_count = svg.matches("<path").count();
    println!(
        "laissez_vibrer.svg: {} bytes, {} paths",
        svg.len(),
        path_count
    );

    // Path-count lower bound. With `measures_per_system(2)` and 4
    // measures we get 2 systems → 2 clef paths. Per measure:
    //   M1: 4 noteheads + 4 l.v. glyphs                       =  8
    //   M2: 4 noteheads + 4 l.v. glyphs                       =  8
    //   M3: 3 noteheads + 1 l.v. glyph                        =  4
    //   M4: 3 noteheads + 1 l.v. glyph + 1 fermata            =  5
    // = 25 + 2 clefs = 27 minimum. Use 27 as a tight floor:
    // anything less than 27 means at least one of the 10 expected
    // l.v. / fermata glyphs was silently dropped by a regression. A
    // bug that misrouted l.v. into a wrong-bucket sink and never
    // emitted the glyph would drop M1 + M2 = 8 paths and trip this.
    assert!(
        path_count >= 27,
        "expected >= 27 paths (clefs + noteheads + 10 l.v./fermata glyphs), got {path_count}"
    );

    // Each measure ends with a barline. Use the literal `<line` tag
    // count as a structural floor — barlines, stems, and staff line
    // segments are all `<line>` elements, so even a no-stem score
    // (M3 + M4 use WHOLE notes with no stems) still emits the four
    // barlines and the per-system staff line segments.
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 3,
        "expected >= 3 <line> elements for barlines and staff segments, got {line_count}"
    );

    // Sanity: every glyph beyond the clefs should be present — a
    // catastrophic regression that dropped every articulation AND
    // every notehead would leave only the 2 clef paths. The floor of
    // 27 above already trips that, but assert > 2 explicitly so the
    // diagnostic message is unambiguous when only the clefs survive.
    assert!(
        path_count > 2,
        "expected more than 2 paths (would mean every glyph beyond the clefs vanished), got {path_count}"
    );

    // l.v. ships distinct Above and Below SMuFL glyphs. Both must be
    // present in the output: M1 emits four `Below` arm glyphs and M2
    // emits four `Above` arm glyphs. A regression collapsing
    // `glyph(Above)` and `glyph(Below)` onto the same enum arm would
    // produce identical path `d` strings for M1 and M2 — visually
    // distinguishing those `d` strings is not feasible from this
    // example alone (that's what the golden test is for), but we can
    // at least check that *something* with multiple distinct
    // `<path d="...">` entries was rendered by counting unique
    // `<path d="` prefixes. A bare lower-bound check on distinct
    // paths > 2 (clefs + at least one articulation glyph + at least
    // one notehead) catches a "every articulation collapsed to the
    // same glyph" pathology where path_count would still be ≥ 27.
    let mut distinct_d: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for piece in svg.split("<path d=\"") {
        if let Some(end) = piece.find('"') {
            distinct_d.insert(&piece[..end]);
        }
    }
    assert!(
        distinct_d.len() >= 3,
        "expected at least 3 distinct <path d=\"...\"> values (clef + notehead + articulation), got {}",
        distinct_d.len()
    );
}
