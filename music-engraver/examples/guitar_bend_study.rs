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

    // Pick G4, arrive at A4 across the first barline, hold through a
    // system break, reattack the held pitch, then release to G4.
    let source = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let arrival = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let reattack = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;
    let release_end = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.barline()?;

    // Pre-bend B3 to C#4, hold for an eighth note, then release across the
    // final barline. This also proves event-relative phase timing.
    let pre_source = score.note(p(Note::B, 3), Duration::QTR, 2, 0)?;
    score.barline()?;
    let pre_release_end = score.note(p(Note::B, 3), Duration::QTR, 2, 0)?;
    score.end_barline()?;

    score.bend(
        BendGesture::new(
            source,
            1,
            BendPitch::exact(p(Note::A, 4)),
            GuitarMoment::onset(arrival),
        )
        .with_release(BendRelease::new(
            GuitarMoment::after(reattack, Duration::EIGHTH),
            GuitarMoment::onset(release_end),
            BendPitch::exact(p(Note::G, 4)),
        ))
        .reattacked_at(reattack),
    )?;
    score.bend(
        BendGesture::pre_bend(pre_source, 2, BendPitch::exact(p(Note::Cis, 4))).with_release(
            BendRelease::new(
                GuitarMoment::after(pre_source, Duration::EIGHTH),
                GuitarMoment::onset(pre_release_end),
                BendPitch::exact(p(Note::B, 3)),
            ),
        ),
    )?;

    let svg = MultiStaffScore::guitar(score)
        .system_width_fu(10_000.0)
        .measures_per_system(2)
        .render_svg();

    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/output/guitar_bend_study.svg");
    std::fs::create_dir_all(output.parent().expect("output directory"))?;
    std::fs::write(&output, &svg)?;

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("data-bend-phase=\"rise\""));
    assert!(svg.contains("data-bend-phase=\"hold\""));
    assert!(svg.contains("data-bend-phase=\"release\""));
    assert!(svg.contains("data-bend-phase=\"reattack\""));
    assert!(svg.contains("data-bend-fragment=\"start\""));
    assert!(svg.contains(">A4</text>"));
    assert!(svg.contains(">C#4</text>"));

    println!(
        "{}: {} bend fragments, {} text labels, {} bytes",
        output.display(),
        svg.matches("data-bend-fragment=").count(),
        svg.matches("<text ").count(),
        svg.len()
    );
    Ok(())
}
