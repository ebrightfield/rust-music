use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::tab_vibrato::VibratoKind;
use music_engraver::score::guitar::{
    GuitarAnnotation, GuitarEventSpec, GuitarScore, GuitarScoreError, GuitarSpanKind, Harmonic,
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
    score.set_time_signature(4, 4);

    let first = score.note(p(Note::E, 4), Duration::QTR, 1, 0)?;
    let beamed = score.beam_group(vec![
        event(Note::Fis, 4, Duration::EIGHTH, 1, 2),
        event(Note::G, 4, Duration::EIGHTH, 1, 3),
        event(Note::A, 4, Duration::EIGHTH, 1, 5),
        event(Note::B, 4, Duration::EIGHTH, 1, 7),
    ])?;
    score.annotate(beamed[1], GuitarAnnotation::Vibrato(VibratoKind::Normal))?;
    let harmonic = score.note(p(Note::E, 5), Duration::QTR, 1, 12)?;
    score.annotate(
        harmonic,
        GuitarAnnotation::Harmonic {
            string: 1,
            harmonic: Harmonic::natural(p(Note::E, 5)),
        },
    )?;
    score.set_voice(1);
    score.note(p(Note::E, 2), Duration::WHOLE, 6, 0)?;
    score.barline()?;

    let triplet = score.tuplet(
        3,
        2,
        vec![
            event(Note::E, 4, Duration::EIGHTH, 1, 0),
            event(Note::Fis, 4, Duration::EIGHTH, 1, 2),
            event(Note::G, 4, Duration::EIGHTH, 1, 3),
        ],
    )?;
    let slide_start = score.note(p(Note::A, 4), Duration::QTR, 1, 5)?;
    let second_end = score.note(p(Note::B, 4), Duration::HALF, 1, 7)?;
    score.set_voice(1);
    score.note(p(Note::B, 2), Duration::WHOLE, 5, 2)?;
    score.barline()?;

    let last = score.note(p(Note::G, 4), Duration::WHOLE, 1, 3)?;
    score.set_voice(1);
    score.note(p(Note::G, 2), Duration::WHOLE, 6, 3)?;
    score.end_barline()?;

    score.span(GuitarSpanKind::Slide, slide_start, second_end, 1)?;
    score.span(GuitarSpanKind::PalmMute, first, second_end, 1)?;
    score.span(GuitarSpanKind::LetRing, triplet[0], last, 1)?;

    let svg = MultiStaffScore::guitar(score)
        .system_width_fu(10000.0)
        .measures_per_system(2)
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output")?;
    std::fs::write("music-engraver/examples/output/guitar_tab_score.svg", &svg)?;

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains(">3</text>"), "TAB tuplet number must render");
    assert!(
        svg.contains("stroke-dasharray"),
        "technique spans must render"
    );
    assert!(
        svg.contains(">12</text>"),
        "upper-position realization must render"
    );

    let mut invalid = GuitarScore::standard();
    let mismatch = invalid
        .note(p(Note::F, 4), Duration::QTR, 1, 0)
        .expect_err("open first string sounds E4, not F4");
    assert!(matches!(mismatch, GuitarScoreError::PitchMismatch { .. }));

    println!(
        "guitar_tab_score.svg: {} paths, {} lines, {} bytes",
        svg.matches("<path ").count(),
        svg.matches("<line ").count(),
        svg.len()
    );
    Ok(())
}
