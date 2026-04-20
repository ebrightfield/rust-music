//! Example: render notes with lyrics using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/lyrics_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::lyric::LyricSyllable;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: "Hap-py birth-day"
        .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("Hap"))
        .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::word("py"))
        .note(Pitch::new(Note::D, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("birth"))
        .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::word("day"))
        .barline()
        // Measure 2: "to you, dear"
        .note(Pitch::new(Note::F, 4).expect("valid pitch"), Duration::HALF)
        .lyric(LyricSyllable::word("to"))
        .note(Pitch::new(Note::E, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::word("you,"))
        .rest(Duration::QTR) // lyric on rest is a no-op
        .lyric(LyricSyllable::word("SKIP"))
        .barline()
        // Measure 3: "Hap-py birth-day" (ascending)
        .note(Pitch::new(Note::C, 5).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("Hap"))
        .note(Pitch::new(Note::C, 5).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::word("py"))
        .note(Pitch::new(Note::B, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("birth"))
        .note(Pitch::new(Note::A, 4).expect("valid pitch"), Duration::QTR)
        .lyric(LyricSyllable::word("day"))
        .barline()
        // Measure 4: "dear friend!" with extender on sustained syllable
        .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::HALF)
        .lyric(LyricSyllable::with_extender("dear"))
        .note(Pitch::new(Note::F, 4).expect("valid pitch"), Duration::HALF)
        .lyric(LyricSyllable::word("friend!"))
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("lyrics_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Validate structure
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let text_count = svg.matches("<text").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );

    // Lyric text should appear as <text> elements
    assert!(text_count >= 10, "expected at least 10 lyric text elements, got {text_count}");
    assert!(svg.contains(">Hap -<"), "should contain 'Hap' with hyphen");
    assert!(svg.contains(">py<"), "should contain 'py'");
    assert!(svg.contains(">birth -<"), "should contain 'birth' with hyphen");
    assert!(svg.contains(">day<"), "should contain 'day'");
    assert!(svg.contains(">to<"), "should contain 'to'");
    assert!(svg.contains(">you,<"), "should contain 'you,'");
    assert!(svg.contains(">dear<"), "should contain 'dear'");
    assert!(svg.contains(">friend!<"), "should contain 'friend!'");
    // "SKIP" should NOT appear — lyric on rest is a no-op
    assert!(!svg.contains(">SKIP<"), "'SKIP' lyric on rest should not render");

    // Lyrics should NOT be italic (unlike expression text)
    // Check that text elements exist without font-style="italic" nearby
    // (expression text is italic; lyrics are roman/upright)

    // Notes + clef paths
    assert!(path_count > 10, "expected notehead and clef paths");
    // Staff lines + stems
    assert!(line_count > 10, "expected staff lines and stems");
}
