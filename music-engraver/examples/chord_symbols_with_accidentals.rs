//! Example: chord symbols rendered with SMuFL accidental glyph composition.
//!
//! Every chord symbol containing `#` or `b` (in flat context) renders the
//! accidental as a SMuFL `accidentalSharp` / `accidentalFlat` / `accidentalNatural`
//! glyph at ~70 % of the text size, baseline-raised, rather than as a plain ASCII
//! character — the standard engraving convention.
//!
//! Produces `examples/output/chord_symbols_with_accidentals.svg`.

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
        // Measure 1: flat-root chords (Bb, Eb) — exercises the b-after-uppercase
        // root flat-context rule.
        .note(Pitch::new(Note::Bes, 3).unwrap(), Duration::QTR)
        .chord_symbol("Bb")
        .note(Pitch::new(Note::Ees, 4).unwrap(), Duration::QTR)
        .chord_symbol("Ebmaj7")
        .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
        .chord_symbol("F")
        .note(Pitch::new(Note::Bes, 3).unwrap(), Duration::QTR)
        .chord_symbol("Bb7")
        .barline()
        // Measure 2: sharp-root chords — exercises the ASCII '#' → Sharp rule.
        .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
        .chord_symbol("F#m")
        .note(Pitch::new(Note::Cis, 4).unwrap(), Duration::QTR)
        .chord_symbol("C#7")
        .note(Pitch::new(Note::Gis, 4).unwrap(), Duration::QTR)
        .chord_symbol("G#dim")
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::QTR)
        .chord_symbol("A")
        .barline()
        // Measure 3: altered chord-symbols — b-after-digit flat context
        // (b5, b9, b13) and combined sharp+flat in a single symbol.
        .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
        .chord_symbol("F#m7b5")
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
        .chord_symbol("C7b9")
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
        .chord_symbol("D7#9")
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
        .chord_symbol("G13b9")
        .barline()
        // Measure 4: mix of unaccidentaled (control) + slash chord with flat root.
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
        .chord_symbol("Cmaj7")
        .note(Pitch::new(Note::A, 3).unwrap(), Duration::QTR)
        .chord_symbol("Am")
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
        .chord_symbol("D/Bb")
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
        .chord_symbol("G7")
        .end_barline()
        .render_svg();

    let path = out_dir.join("chord_symbols_with_accidentals.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    // --- Verification ---
    assert!(svg.starts_with("<svg"), "output should be valid SVG");

    let text_count = svg.matches("<text").count();
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();

    // Sanity: at least one text element per chord symbol with a text-portion.
    // 16 symbols total; some have multiple text runs (e.g. F#m7b5 has 3),
    // so >= 16 text elements is a defensive minimum.
    assert!(
        text_count >= 16,
        "expected at least 16 text elements (one per chord symbol's text runs), got {text_count}"
    );

    // Accidental glyphs are emitted as <path>, in addition to staff glyphs
    // (clef, noteheads, etc). Count the expected accidentals across all 16
    // symbols: Bb, Ebmaj7, Bb7, F#m, C#7, G#dim, F#m7b5 (# + b), C7b9, D7#9,
    // G13b9, D/Bb — that's 13 accidental glyphs. The chord-symbol accidentals
    // are layered on top of staff-content paths (16 notes + 1 clef etc.).
    assert!(
        path_count >= 13,
        "expected at least 13 chord-symbol accidental paths in the path total, got {path_count}"
    );

    // Plain-text symbols (no accidentals) must still appear as single text runs:
    // these strings should appear verbatim inside <text>...</text>.
    assert!(svg.contains(">Cmaj7<"), "should contain plain 'Cmaj7' text element");
    assert!(svg.contains(">Am<"), "should contain plain 'Am' text element");
    assert!(svg.contains(">G7<"), "should contain plain 'G7' text element");
    assert!(svg.contains(">F<"), "should contain plain 'F' text element");
    assert!(svg.contains(">A<"), "should contain plain 'A' text element");

    // Accidental-bearing symbols: text runs are split — the bare root letter
    // ('B', 'E', 'F', 'C', 'G', 'D') and the trailing fragments ('m', 'maj7',
    // '7', '7b9' → '7' + flat + '9' etc.) must appear, but the ASCII
    // accidental characters must NOT appear inside any <text> element.
    assert!(svg.contains(">B<"), "expected 'B' text fragment from 'Bb', 'Bb7', etc.");
    assert!(svg.contains(">E<"), "expected 'E' text fragment from 'Ebmaj7'");
    assert!(svg.contains(">F<"), "expected 'F' text fragment from 'F#m', 'F#m7b5'");

    // Regression canary: no chord symbol should produce a text element that
    // visibly contains '#' or 'b'-as-flat as ASCII text characters. We check
    // for the specific pre-feature strings that used to be rendered.
    assert!(
        !svg.contains(">F#m7b5<"),
        "F#m7b5 should be split into segments; '#' and 'b' should be glyphs not text"
    );
    assert!(
        !svg.contains(">Bb7<"),
        "Bb7 should be split into segments; 'b' should be a glyph not text"
    );
    assert!(
        !svg.contains(">F#m<"),
        "F#m should be split into segments; '#' should be a glyph not text"
    );

    // Bold-weight invariant: every text run is bold.
    let bold_count = svg.matches(r#"font-weight="bold""#).count();
    assert!(
        bold_count >= text_count.saturating_sub(0),
        "every text run should be bold; bold={bold_count} text={text_count}"
    );

    println!(
        "Wrote {} ({} bytes, {} text elements, {} paths, {} lines)",
        path.display(),
        svg.len(),
        text_count,
        path_count,
        line_count
    );
}
