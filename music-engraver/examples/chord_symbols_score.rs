//! Example: render notes with chord symbols using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/chord_symbols_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: I–vi–IV–V in C major
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
        .chord_symbol("C")
        .note(Pitch::new(Note::A, 3).unwrap(), Duration::QTR)
        .chord_symbol("Am")
        .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
        .chord_symbol("F")
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
        .chord_symbol("G")
        .barline()
        // Measure 2: extended jazz chords
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::HALF)
        .chord_symbol("Dm7")
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::HALF)
        .chord_symbol("G7")
        .barline()
        // Measure 3: complex symbols over a chord voicing + single notes
        .chord(
            vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
                Pitch::new(Note::B, 4).unwrap(),
            ],
            Duration::HALF,
        )
        .chord_symbol("Cmaj7")
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::QTR)
        .chord_symbol("Am")
        .rest(Duration::QTR)
        // No chord symbol on rest (no-op)
        .barline()
        // Measure 4: altered/extended chord symbols
        .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
        .chord_symbol("F#m7b5")
        .note(Pitch::new(Note::B, 3).unwrap(), Duration::QTR)
        .chord_symbol("B7alt")
        .note(Pitch::new(Note::E, 4).unwrap(), Duration::HALF)
        .chord_symbol("Em")
        .end_barline()
        .render_svg();

    let path = out_dir.join("chord_symbols_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    // Verification
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    let text_count = svg.matches("<text").count();
    // Some symbols (F#m7b5) are now split into multiple text runs by the
    // composite chord-symbol renderer (one text run per non-accidental
    // fragment). 9 is the old single-run lower bound; with splitting it
    // is strictly higher.
    assert!(
        text_count >= 9,
        "expected at least 9 chord symbol text elements, got {text_count}"
    );
    // Accidental-free symbols are still emitted as one text run.
    assert!(svg.contains(">Cmaj7<"), "should contain 'Cmaj7' as one text run");
    assert!(svg.contains(">G7<"), "should contain 'G7' as one text run");
    // F#m7b5 is now split into three text runs (F, m7, 5) + two SMuFL
    // accidental paths (# and b). The whole token must NOT appear in any
    // single <text>...</text>.
    assert!(
        !svg.contains(">F#m7b5<"),
        "F#m7b5 should be split into segments; '#' and 'b' should be glyph paths, not part of the text"
    );
    assert!(svg.contains(">F<"), "should contain 'F' text fragment of F#m7b5");
    assert!(svg.contains(">m7<"), "should contain 'm7' text fragment of F#m7b5");
    assert!(svg.contains(">5<"), "should contain '5' text fragment of F#m7b5");
    assert!(svg.contains("bold"), "chord symbols should be bold");

    println!(
        "Wrote {} ({} bytes, {} text elements, {} paths, {} lines)",
        path.display(),
        svg.len(),
        text_count,
        svg.matches("<path").count(),
        svg.matches("<line").count(),
    );
}
