//! Semantic pre-bend and release rendered on coordinated standard and TAB staves.

use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::guitar::{
    BendGesture, BendPitch, BendRelease, GuitarMoment, GuitarScore,
};
use music_engraver::score::multi_staff::MultiStaffScore;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = GuitarScore::standard();
    score.set_time_signature(1, 4);
    let source = score.note(p(Note::B, 3), Duration::QTR, 2, 0)?;
    score.barline()?;
    let release_end = score.note(p(Note::B, 3), Duration::QTR, 2, 0)?;
    score.end_barline()?;

    score.bend(
        BendGesture::pre_bend(source, 2, BendPitch::exact(p(Note::Cis, 4))).with_release(
            BendRelease::new(
                GuitarMoment::after(source, Duration::EIGHTH),
                GuitarMoment::onset(release_end),
                BendPitch::exact(p(Note::B, 3)),
            ),
        ),
    )?;

    let svg = MultiStaffScore::guitar(score)
        .measures_per_system(1)
        .system_width_fu(12_000.0)
        .render_svg();
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/tab_pre_bends.svg");
    std::fs::create_dir_all(output.parent().expect("output directory"))?;
    std::fs::write(&output, &svg)?;

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("data-bend-phase=\"rise\""));
    assert!(svg.contains("data-bend-phase=\"hold\""));
    assert!(svg.contains("data-bend-phase=\"release\""));
    assert!(svg.contains("data-bend-fragment=\"start\""));
    println!("{}: {} bytes", output.display(), svg.len());
    Ok(())
}
