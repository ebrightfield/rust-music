//! Vibrato and a semantic held bend on coordinated standard and TAB staves.

use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::tab_vibrato::VibratoKind;
use music_engraver::score::guitar::{
    BendGesture, BendPitch, GuitarAnnotation, GuitarMoment, GuitarScore,
};
use music_engraver::score::multi_staff::MultiStaffScore;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = GuitarScore::standard();
    score.set_time_signature(1, 4);
    let source = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.annotate(source, GuitarAnnotation::Vibrato(VibratoKind::Normal))?;
    score.barline()?;
    let reattack = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.annotate(reattack, GuitarAnnotation::Vibrato(VibratoKind::Wide))?;
    score.end_barline()?;
    score.bend(
        BendGesture::new(
            source,
            1,
            BendPitch::exact(p(Note::A, 4)),
            GuitarMoment::onset(reattack),
        )
        .reattacked_at(reattack),
    )?;

    let svg = MultiStaffScore::guitar(score)
        .measures_per_system(1)
        .render_svg();
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/tab_vibrato.svg");
    std::fs::create_dir_all(output.parent().expect("output directory"))?;
    std::fs::write(&output, &svg)?;

    assert!(svg.contains("stroke-linecap=\"round\""));
    assert!(svg.contains("data-bend-phase=\"reattack\""));
    println!("{}: {} bytes", output.display(), svg.len());
    Ok(())
}
