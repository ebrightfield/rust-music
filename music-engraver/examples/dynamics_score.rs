//! Example: render notes with dynamics using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/dynamics_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Flats(2)) // Bb major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: ascending line with crescendo-like dynamics
        .note(Pitch::new(Note::Bes, 3), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .dynamic(Dynamic::Mp)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .dynamic(Dynamic::Mf)
        .note(Pitch::new(Note::Bes, 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .barline()
        // Measure 2: descending with diminuendo-like dynamics
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .dynamic(Dynamic::Ff)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .dynamic(Dynamic::Mf)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .barline()
        // Measure 3: chord with dynamic + rest (no dynamic on rest)
        .chord(
            vec![
                Pitch::new(Note::Bes, 3),
                Pitch::new(Note::D, 4),
                Pitch::new(Note::F, 4),
            ],
            Duration::HALF,
        )
        .dynamic(Dynamic::Fff)
        .rest(Duration::HALF)
        .barline()
        // Measure 4: accents and rinforzando family — exercise the
        // newly-added composite glyphs (sf, sfp, rfz, fz, niente).
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .dynamic(Dynamic::Sf)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .dynamic(Dynamic::Rfz)
        .note(Pitch::new(Note::Ees, 4), Duration::QTR)
        .dynamic(Dynamic::Fz)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .dynamic(Dynamic::Niente)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("dynamics_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Validate structure
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "SVG contains {} paths, {} lines",
        path_count, line_count
    );
    // Must have dynamic markings (extra paths beyond noteheads/clef/key sig)
    // 14 notes/chords + clef + key sig glyphs + 11 dynamics = at least 25 paths
    assert!(path_count > 20, "expected many paths including dynamics");
    // Must have staff lines + stems
    assert!(line_count > 10, "expected staff lines and stems");
}
