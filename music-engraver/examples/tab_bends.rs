//! Timed bend gestures rendered from one standard-notation/TAB score.

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
    let source = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let arrival = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let held_attack = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let release_end = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.end_barline()?;

    score.bend(
        BendGesture::new(
            source,
            1,
            BendPitch::exact(p(Note::A, 4)),
            GuitarMoment::onset(arrival),
        )
        .with_release(BendRelease::new(
            GuitarMoment::after(held_attack, Duration::EIGHTH),
            GuitarMoment::onset(release_end),
            BendPitch::exact(p(Note::G, 4)),
        ))
        .reattacked_at(held_attack),
    )?;

    let svg = MultiStaffScore::guitar(score)
        .measures_per_system(2)
        .system_width_fu(12_000.0)
        .render_svg();
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/tab_bends.svg");
    std::fs::create_dir_all(output.parent().expect("output directory"))?;
    std::fs::write(&output, &svg)?;

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("data-bend-view=\"standard\""));
    assert!(svg.contains("data-bend-view=\"tab\""));
    assert!(svg.contains("data-bend-phase=\"hold\""));
    assert!(svg.contains("data-bend-phase=\"reattack\""));
    println!("{}: {} bytes", output.display(), svg.len());
    Ok(())
}
