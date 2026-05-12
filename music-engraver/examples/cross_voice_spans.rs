//! Example: ties, slurs, and hairpins inside additional voices.
//!
//! Renders a 4-measure score across 2 systems where each voice independently
//! carries its own spans:
//!
//! - Voice 0 (stems up): tie, slur, crescendo hairpin
//! - Voice 1 (stems down): slur, tie, decrescendo hairpin
//!
//! The interesting case is the third measure: voice 0 crescendos while voice 1
//! decrescendos at the same time, exercising independent hairpin tracking per
//! voice.
//!
//! Produces `examples/output/cross_voice_spans.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::hairpin::HairpinType;
use music_engraver::score::ScoreBuilder;

fn n(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // ── Measure 1 ──────────────────────────────────────────────────────
        // Voice 0: E5 tied across the bar.
        // Voice 1: slur over C4–E4–G4–C5.
        .note(n(Note::E, 5), Duration::HALF)
        .tie()
        .note(n(Note::E, 5), Duration::HALF)
        .voice(1)
        .note(n(Note::C, 4), Duration::QTR)
        .slur_start()
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::G, 4), Duration::QTR)
        .note(n(Note::C, 5), Duration::QTR)
        .slur_end()
        .voice(0)
        .barline()
        // ── Measure 2 ──────────────────────────────────────────────────────
        // Voice 0: slur over G5–F5–E5–D5.
        // Voice 1: C4 tied, then E4 plain.
        .note(n(Note::G, 5), Duration::QTR)
        .slur_start()
        .note(n(Note::F, 5), Duration::QTR)
        .note(n(Note::E, 5), Duration::QTR)
        .note(n(Note::D, 5), Duration::QTR)
        .slur_end()
        .voice(1)
        .note(n(Note::C, 4), Duration::HALF)
        .tie()
        .note(n(Note::C, 4), Duration::HALF)
        .voice(0)
        .barline()
        // ── Measure 3 ──────────────────────────────────────────────────────
        // Simultaneous opposing hairpins:
        //   voice 0 crescendos from E5 to A5,
        //   voice 1 decrescendos from C5 down to C4.
        .note(n(Note::E, 5), Duration::QTR)
        .hairpin_start(HairpinType::Crescendo)
        .note(n(Note::F, 5), Duration::QTR)
        .note(n(Note::G, 5), Duration::QTR)
        .note(n(Note::A, 5), Duration::QTR)
        .hairpin_end()
        .voice(1)
        .note(n(Note::C, 5), Duration::QTR)
        .hairpin_start(HairpinType::Decrescendo)
        .note(n(Note::A, 4), Duration::QTR)
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::C, 4), Duration::QTR)
        .hairpin_end()
        .voice(0)
        .barline()
        // ── Measure 4 ──────────────────────────────────────────────────────
        // Voice 0 ties its half note; voice 1 carries a slur.
        .note(n(Note::G, 5), Duration::HALF)
        .tie()
        .note(n(Note::G, 5), Duration::HALF)
        .voice(1)
        .note(n(Note::C, 4), Duration::QTR)
        .slur_start()
        .note(n(Note::D, 4), Duration::QTR)
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::F, 4), Duration::QTR)
        .slur_end()
        .voice(0)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/cross_voice_spans.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/cross_voice_spans.svg ({} bytes)",
        svg.len()
    );

    assert!(svg.starts_with("<svg"));

    // Span elements we expect:
    //   - 2 ties (voice 0 m1, voice 1 m2, voice 0 m4)  → 3 filled stroke="none" paths
    //   - 3 slurs (voice 1 m1, voice 0 m2, voice 1 m4) → 3 filled stroke="none" paths
    //   - 2 hairpins (voice 0 m3 cresc, voice 1 m3 decresc) → 4 lines
    let fill_paths = svg.matches(r#"stroke="none""#).count();
    let line_count = svg.matches("<line ").count();
    println!("  stroke=\"none\" (curves): {fill_paths}");
    println!("  <line>: {line_count}");

    // A bare 4-measure two-voice score with no spans has 0 filled curves.
    // We add 3 ties + 3 slurs = 6 filled curves.
    assert!(
        fill_paths >= 6,
        "expected ≥6 filled curve paths for ties+slurs, got {fill_paths}",
    );

    // Hairpins add at least 4 lines (2 wedges × 2 sides) on top of the
    // staff+stem+barline lines. Compare against a structurally identical
    // 4-measure two-voice score with all span markers removed.
    let plain_two_voices = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // m1
        .note(n(Note::E, 5), Duration::HALF)
        .note(n(Note::E, 5), Duration::HALF)
        .voice(1)
        .note(n(Note::C, 4), Duration::QTR)
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::G, 4), Duration::QTR)
        .note(n(Note::C, 5), Duration::QTR)
        .voice(0)
        .barline()
        // m2
        .note(n(Note::G, 5), Duration::QTR)
        .note(n(Note::F, 5), Duration::QTR)
        .note(n(Note::E, 5), Duration::QTR)
        .note(n(Note::D, 5), Duration::QTR)
        .voice(1)
        .note(n(Note::C, 4), Duration::HALF)
        .note(n(Note::C, 4), Duration::HALF)
        .voice(0)
        .barline()
        // m3
        .note(n(Note::E, 5), Duration::QTR)
        .note(n(Note::F, 5), Duration::QTR)
        .note(n(Note::G, 5), Duration::QTR)
        .note(n(Note::A, 5), Duration::QTR)
        .voice(1)
        .note(n(Note::C, 5), Duration::QTR)
        .note(n(Note::A, 4), Duration::QTR)
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::C, 4), Duration::QTR)
        .voice(0)
        .barline()
        // m4
        .note(n(Note::G, 5), Duration::HALF)
        .note(n(Note::G, 5), Duration::HALF)
        .voice(1)
        .note(n(Note::C, 4), Duration::QTR)
        .note(n(Note::D, 4), Duration::QTR)
        .note(n(Note::E, 4), Duration::QTR)
        .note(n(Note::F, 4), Duration::QTR)
        .voice(0)
        .end_barline()
        .render_svg();
    let plain_lines = plain_two_voices.matches("<line ").count();
    let plain_fills = plain_two_voices.matches(r#"stroke="none""#).count();
    println!("  plain (no spans) lines: {plain_lines}, fills: {plain_fills}");
    // Two hairpin wedges add exactly 4 line segments.
    assert_eq!(
        line_count,
        plain_lines + 4,
        "expected exactly 4 extra lines from 2 hairpins (have={line_count}, plain={plain_lines})",
    );
    // 3 ties + 3 slurs add exactly 6 stroke="none" filled curves.
    assert_eq!(
        fill_paths,
        plain_fills + 6,
        "expected exactly 6 extra filled curves from 3 ties + 3 slurs \
         (have={fill_paths}, plain={plain_fills})",
    );
}
