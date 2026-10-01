use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::guitar::{
    AttackSource, FrettedPitch, GuitarAnnotation, GuitarEventSpec, GuitarScore,
    GuitarSpanAnnotation, GuitarText, LeftHandArticulation, LeftHandFinger, PercussionTarget,
    PickStroke, PluckingFinger, PositionLabel, ShiftDirection, SlapTechnique, TappingHand,
    TextEnclosure, TextPlacement,
};
use music_engraver::score::multi_staff::MultiStaffScore;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}
fn event(note: Note, octave: i8, duration: Duration, string: u8, fret: u8) -> GuitarEventSpec {
    GuitarEventSpec::pitched(p(note, octave), duration, string, fret)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = GuitarScore::standard();
    score.set_time_signature(4, 4).show_technique_legend();

    let beam = score.beam_group(vec![
        event(Note::E, 4, Duration::EIGHTH, 1, 0),
        event(Note::F, 4, Duration::EIGHTH, 1, 1),
        event(Note::Fis, 4, Duration::EIGHTH, 1, 2),
        event(Note::G, 4, Duration::EIGHTH, 1, 3),
    ])?;
    score.annotate(
        beam[0],
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Pick(PickStroke::Down),
        },
    )?;
    score.annotate(
        beam[1],
        GuitarAnnotation::LeftHandFinger {
            string: 1,
            finger: LeftHandFinger::One,
        },
    )?;
    score.annotate(
        beam[1],
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Pick(PickStroke::Up),
        },
    )?;
    score.annotate(
        beam[2],
        GuitarAnnotation::LeftHandFinger {
            string: 1,
            finger: LeftHandFinger::Two,
        },
    )?;
    score.annotate(
        beam[2],
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Pluck(PluckingFinger::Index),
        },
    )?;
    score.annotate(
        beam[3],
        GuitarAnnotation::LeftHandFinger {
            string: 1,
            finger: LeftHandFinger::Three,
        },
    )?;
    score.annotate(
        beam[3],
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Tap(TappingHand::Picking),
        },
    )?;

    let position_note = score.note(p(Note::A, 4), Duration::QTR, 1, 5)?;
    score.annotate(
        position_note,
        GuitarAnnotation::Position(PositionLabel::new(5).shifted(ShiftDirection::Up)),
    )?;
    score.annotate(
        position_note,
        GuitarAnnotation::Text(
            GuitarText::new("sul pont.", TextPlacement::Below).enclosed(TextEnclosure::Rectangle),
        ),
    )?;
    let pop = score.note(p(Note::B, 4), Duration::QTR, 1, 7)?;
    score.annotate(
        pop,
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Slap(SlapTechnique::Pop),
        },
    )?;
    score.barline()?;

    let hybrid = score.chord(
        vec![
            FrettedPitch::new(p(Note::Fis, 4), 1, 2),
            FrettedPitch::new(p(Note::D, 4), 2, 3),
        ],
        Duration::HALF,
    )?;
    score.annotate(
        hybrid,
        GuitarAnnotation::LeftHandFinger {
            string: 1,
            finger: LeftHandFinger::One,
        },
    )?;
    score.annotate(
        hybrid,
        GuitarAnnotation::LeftHandFinger {
            string: 2,
            finger: LeftHandFinger::Three,
        },
    )?;
    score.annotate(
        hybrid,
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Pluck(PluckingFinger::Middle),
        },
    )?;
    score.annotate(
        hybrid,
        GuitarAnnotation::Attack {
            string: Some(2),
            source: AttackSource::Pick(PickStroke::Down),
        },
    )?;
    let hammer = score.note(p(Note::G, 4), Duration::QTR, 1, 3)?;
    score.annotate(
        hammer,
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::LeftHand(LeftHandArticulation::HammerOn),
        },
    )?;
    score.percussion(PercussionTarget::Body, Duration::QTR);
    score.end_barline()?;

    score.annotation_span(
        GuitarSpanAnnotation::CellBracket {
            label: String::from("cell A"),
            enclosure: Some(TextEnclosure::Circle),
        },
        beam[0],
        position_note,
    )?;
    score.annotation_span(
        GuitarSpanAnnotation::Position(PositionLabel::new(5)),
        position_note,
        hybrid,
    )?;
    score.annotation_span(
        GuitarSpanAnnotation::Text(GuitarText::new("timbre shift", TextPlacement::Below)),
        pop,
        hammer,
    )?;

    let svg = MultiStaffScore::guitar(score)
        .system_width_fu(10_000.0)
        .measures_per_system(1)
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output")?;
    std::fs::write("music-engraver/examples/output/guitar_mechanics.svg", &svg)?;

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains(">Legend:</text>"));
    assert!(svg.contains(">golpe  body percussion</text>"));

    println!(
        "guitar_mechanics.svg: {} text labels, {} span segments, {} bytes",
        svg.matches("<text ").count(),
        svg.matches("<line ").count(),
        svg.len()
    );
    Ok(())
}
