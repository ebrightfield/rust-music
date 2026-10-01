use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::guitar::{
    AttackSource, BarreKind, Capo, FrettedPitch, GuitarAnnotation, GuitarBarre, GuitarEventSpec,
    GuitarScore, GuitarSpanAnnotation, GuitarTuning, Harmonic, PercussionTarget, PickStroke,
    SlapTechnique, StrumDirection, TappingHand, TuningDisplay,
};
use music_engraver::score::multi_staff::MultiStaffScore;

fn pitch(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn chord(
    score: &mut GuitarScore,
    pitches: &[(Note, i8, u8, u8)],
) -> Result<
    music_engraver::score::guitar::GuitarEventId,
    music_engraver::score::guitar::GuitarScoreError,
> {
    score.chord(
        pitches
            .iter()
            .map(|&(note, octave, string, fret)| {
                FrettedPitch::new(pitch(note, octave), string, fret)
            })
            .collect(),
        Duration::QTR,
    )
}

fn chord_spec(pitches: &[(Note, i8, u8, u8)], duration: Duration) -> GuitarEventSpec {
    GuitarEventSpec::chord(
        pitches
            .iter()
            .map(|&(note, octave, string, fret)| {
                FrettedPitch::new(pitch(note, octave), string, fret)
            })
            .collect(),
        duration,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tuning = GuitarTuning::named(
        "DADGAD",
        vec![
            pitch(Note::D, 4),
            pitch(Note::A, 3),
            pitch(Note::G, 3),
            pitch(Note::D, 3),
            pitch(Note::A, 2),
            pitch(Note::D, 2),
        ],
    )?;
    let mut score = GuitarScore::new(tuning);
    score.set_capo(Some(Capo::new(2)?))?;
    score
        .set_tuning_display(TuningDisplay::NameAndPitches)
        .set_time_signature(4, 4)
        .show_technique_legend();

    // Measure 1: all four harmonic-production families.
    let natural = score.note(pitch(Note::E, 5), Duration::QTR, 1, 12)?;
    score.annotate(
        natural,
        GuitarAnnotation::Harmonic {
            string: 1,
            harmonic: Harmonic::natural(pitch(Note::E, 5)),
        },
    )?;
    score.annotate(natural, GuitarAnnotation::ChordSymbol("E".to_owned()))?;
    let artificial = score.note(pitch(Note::E, 4), Duration::QTR, 2, 5)?;
    score.annotate(
        artificial,
        GuitarAnnotation::Harmonic {
            string: 2,
            harmonic: Harmonic::artificial(pitch(Note::B, 5), 12),
        },
    )?;
    let pinch = score.note(pitch(Note::E, 4), Duration::QTR, 3, 7)?;
    score.annotate(
        pinch,
        GuitarAnnotation::Harmonic {
            string: 3,
            harmonic: Harmonic::pinch(pitch(Note::E, 5)),
        },
    )?;
    score.annotate(
        pinch,
        GuitarAnnotation::Attack {
            string: Some(3),
            source: AttackSource::Pick(PickStroke::Down),
        },
    )?;
    let tapped = score.note(pitch(Note::G, 3), Duration::QTR, 4, 3)?;
    score.annotate(
        tapped,
        GuitarAnnotation::Harmonic {
            string: 4,
            harmonic: Harmonic::tapped(pitch(Note::D, 5), 10, TappingHand::Picking),
        },
    )?;
    score.barline()?;

    // Measure 2: grouped realized chords driving rhythmic slash comping.
    let comping = score.beam_group(vec![
        chord_spec(
            &[(Note::Fis, 4, 1, 2), (Note::D, 4, 2, 3), (Note::B, 3, 3, 2)],
            Duration::EIGHTH,
        ),
        chord_spec(
            &[(Note::G, 4, 1, 3), (Note::D, 4, 2, 3), (Note::A, 3, 3, 0)],
            Duration::EIGHTH,
        ),
        chord_spec(
            &[(Note::E, 4, 1, 0), (Note::Cis, 4, 2, 2), (Note::B, 3, 3, 2)],
            Duration::EIGHTH,
        ),
        chord_spec(
            &[
                (Note::Fis, 4, 1, 2),
                (Note::Cis, 4, 2, 2),
                (Note::B, 3, 3, 2),
            ],
            Duration::EIGHTH,
        ),
    ])?;
    for ((event, symbol), direction) in comping.iter().copied().zip(["D", "G", "A", "Bm"]).zip([
        StrumDirection::Down,
        StrumDirection::Up,
        StrumDirection::Down,
        StrumDirection::Up,
    ]) {
        score
            .annotate(event, GuitarAnnotation::RhythmicSlash)?
            .annotate(event, GuitarAnnotation::ChordSymbol(symbol.to_owned()))?
            .annotate(event, GuitarAnnotation::Strum(direction))?;
    }
    let partial_barre_start = comping[3];
    score.rest(Duration::HALF);
    score.barline()?;

    // Measure 3: cross-system partial barre, full barre, slap/pop, ghost/dead.
    let full_barre_start = chord(
        &mut score,
        &[
            (Note::Fis, 4, 1, 2),
            (Note::Cis, 4, 2, 2),
            (Note::B, 3, 3, 2),
            (Note::Fis, 3, 4, 2),
            (Note::Cis, 3, 5, 2),
            (Note::Fis, 2, 6, 2),
        ],
    )?;
    let slap = score.note(pitch(Note::G, 2), Duration::EIGHTH, 6, 3)?;
    score.annotate(
        slap,
        GuitarAnnotation::Attack {
            string: Some(6),
            source: AttackSource::Slap(SlapTechnique::Thumb),
        },
    )?;
    let pop = score.note(pitch(Note::A, 4), Duration::EIGHTH, 1, 5)?;
    score.annotate(
        pop,
        GuitarAnnotation::Attack {
            string: Some(1),
            source: AttackSource::Slap(SlapTechnique::Pop),
        },
    )?;
    let ghost = score.note(pitch(Note::E, 4), Duration::QTR, 2, 5)?;
    score.annotate(ghost, GuitarAnnotation::Ghost { string: Some(2) })?;
    let dead = score.dead(vec![1, 2, 3, 4], Duration::QTR)?;
    score.annotate(dead, GuitarAnnotation::Strum(StrumDirection::Down))?;
    score.barline()?;

    // Measure 4: body/fretboard/string percussion, slash, and grouped dead
    // attacks in beams and tuplets.
    score.beam_group(vec![
        GuitarEventSpec::Percussion {
            target: PercussionTarget::Body,
            duration: Duration::EIGHTH,
        },
        GuitarEventSpec::Percussion {
            target: PercussionTarget::Fretboard,
            duration: Duration::EIGHTH,
        },
        GuitarEventSpec::Percussion {
            target: PercussionTarget::Strings,
            duration: Duration::EIGHTH,
        },
        GuitarEventSpec::Slash {
            duration: Duration::EIGHTH,
        },
    ])?;
    let grouped_percussion = score.tuplet(
        3,
        2,
        vec![
            GuitarEventSpec::Dead {
                strings: vec![1, 2, 3, 4],
                duration: Duration::EIGHTH,
            },
            GuitarEventSpec::Percussion {
                target: PercussionTarget::Fretboard,
                duration: Duration::EIGHTH,
            },
            GuitarEventSpec::Slash {
                duration: Duration::EIGHTH,
            },
        ],
    )?;
    score.annotate(
        grouped_percussion[0],
        GuitarAnnotation::Strum(StrumDirection::Up),
    )?;
    score.rest(Duration::QTR);
    score.end_barline()?;

    score.annotation_span(
        GuitarSpanAnnotation::Barre(GuitarBarre::new(BarreKind::Partial, 2, 1, 3)?),
        partial_barre_start,
        full_barre_start,
    )?;
    score.annotation_span(
        GuitarSpanAnnotation::Barre(GuitarBarre::new(BarreKind::Full, 2, 1, 6)?),
        full_barre_start,
        pop,
    )?;

    let svg = MultiStaffScore::guitar(score)
        .system_width_fu(12_000.0)
        .measures_per_system(2)
        .render_svg();
    std::fs::create_dir_all("music-engraver/examples/output")?;
    std::fs::write(
        "music-engraver/examples/output/advanced_guitar_vocabulary.svg",
        &svg,
    )?;

    assert_eq!(
        svg.matches("Capo II · Tuning: DADGAD · 1=D4 2=A3 3=G3 4=D3 5=A2 6=D2")
            .count(),
        1
    );
    assert_eq!(svg.matches(">Legend:</text>").count(), 1);
    assert!(svg.contains("data-guitar-barre-fragment=\"start\""));
    assert!(svg.contains("data-guitar-barre-fragment=\"end\""));
    assert!(svg.contains("data-guitar-barre-fragment=\"complete\""));
    assert!(svg.contains(">harm.</text>"));
    assert!(svg.contains(">A.H.</text>"));
    assert!(svg.contains(">P.H.</text>"));
    assert!(svg.contains(">T.H.</text>"));
    assert!(svg.contains(">(5)</text>"));
    assert!(svg.contains(">x</text>"));
    assert!(svg.contains(">golpe  body percussion</text>"));
    assert!(svg.contains(">square  fretboard percussion</text>"));
    assert!(svg.contains(">down strum</text>"));
    assert!(svg.contains(">up strum</text>"));
    assert!(svg.contains(">↓  down-pick</text>"));
    assert!(!svg.contains(">D  down strum</text>"));
    assert!(!svg.contains(">U  up strum</text>"));
    assert!(!svg.contains(">CII</text>"));
    assert!(svg.contains(">II</text>"));

    println!(
        "advanced_guitar_vocabulary.svg: {} paths, {} labels, {} bytes",
        svg.matches("<path ").count(),
        svg.matches("<text ").count(),
        svg.len()
    );
    Ok(())
}
