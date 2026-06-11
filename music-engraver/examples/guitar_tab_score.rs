use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::tab::TabScoreBuilder;
use music_engraver::score::ScoreBuilder;

fn main() {
    let p = |note: Note, oct: i8| Pitch::new(note, oct);

    // Standard notation: E minor arpeggio across 2 measures
    let notation = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Measure 1: ascending E minor arpeggio
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .barline()
        // Measure 2: descending
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::HALF)
        .end_barline();

    // Tablature: same arpeggio on guitar strings
    let tab = TabScoreBuilder::guitar()
        // Measure 1: E4=str1/fret0, G4=str3/fret0, B4=str2/fret0, E5=str1/fret5 (not exact, simplified)
        .quarter().fret(1, 0).next()
        .quarter().fret(3, 0).next()
        .quarter().fret(2, 0).next()
        .quarter().fret(1, 5)
        .barline()
        // Measure 2: reverse
        .quarter().fret(1, 5).next()
        .quarter().fret(2, 0).next()
        .half().fret(3, 0)
        .end_barline();

    let svg = MultiStaffScore::guitar_tab(notation, tab)
        .system_width_fu(10000.0)
        .render_svg();

    // Write output
    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/guitar_tab_score.svg", &svg)
        .expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "valid SVG");
    assert!(svg.contains("</svg>"), "closed SVG");

    // Should have fret numbers from tab staff
    assert!(svg.contains(">0</text>"), "should show fret 0");
    assert!(svg.contains(">5</text>"), "should show fret 5");

    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    let text_count = svg.matches("<text ").count();

    println!("guitar_tab_score.svg: {path_count} paths, {line_count} lines, {text_count} texts, {} bytes",
        svg.len());
}
