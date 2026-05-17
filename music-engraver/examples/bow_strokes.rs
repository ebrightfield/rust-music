/// Render string bow-stroke articulations via ScoreBuilder.
///
/// Demonstrates up-bow / down-bow placement and stacking with other
/// articulations. Bow strokes always go above the note (Gould convention
/// for string-articulation markings) and slot between normal articulations
/// and fermatas in a stack: notehead → normal → bow → fermata.
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
        // Measure 1: alternating down-bow / up-bow on a 4-note phrase.
        // All notes are above the middle line (stems down) so the bow
        // marks sit above without competing with a stem-opposite
        // articulation underneath.
        .note(p(Note::G, 5), Duration::QTR)
        .articulation(Articulation::DownBow)
        .note(p(Note::F, 5), Duration::QTR)
        .articulation(Articulation::UpBow)
        .note(p(Note::E, 5), Duration::QTR)
        .articulation(Articulation::DownBow)
        .note(p(Note::D, 5), Duration::QTR)
        .articulation(Articulation::UpBow)
        .barline()
        // Measure 2: bow strokes on low notes (auto-stems-up). Bow marks
        // still go above, demonstrating that they ignore stem direction.
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::DownBow)
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::UpBow)
        .note(p(Note::E, 4), Duration::HALF)
        .articulation(Articulation::DownBow)
        .barline()
        // Measure 3: bow stroke stacked with a normal articulation. With
        // stem-up notes the staccato goes below, the bow goes above —
        // the stack splitter must separate them onto opposite sides.
        .note(p(Note::C, 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::DownBow)
        .note(p(Note::E, 4), Duration::QTR)
        .articulation(Articulation::Accent)
        .articulation(Articulation::UpBow)
        .note(p(Note::G, 4), Duration::HALF)
        .articulation(Articulation::Tenuto)
        .articulation(Articulation::DownBow)
        .barline()
        // Measure 4: full triple stack — staccato + bow + fermata. From
        // notehead outward: staccato → bow → fermata.
        .note(p(Note::A, 5), Duration::HALF)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::DownBow)
        .articulation(Articulation::Fermata)
        .note(p(Note::G, 5), Duration::HALF)
        .articulation(Articulation::UpBow)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/bow_strokes.svg", &svg)
        .expect("write SVG");

    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let path_count = svg.matches("<path").count();
    println!(
        "bow_strokes.svg: {} bytes, {} paths",
        svg.len(),
        path_count
    );

    // Sanity bound: 2 clefs + 13 noteheads + 14 articulation/bow/fermata
    // glyphs (1+1+1+1, 1+1+1, 2+2+2, 3+2) = ≥ 29 paths. Use a permissive
    // lower bound so a minor renderer change doesn't trip this example.
    assert!(
        path_count >= 25,
        "expected at least 25 paths (clefs + noteheads + bow/articulation glyphs), got {path_count}"
    );
}
