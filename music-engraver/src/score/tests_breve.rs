//! RM-MN-008: breve identity must survive `ScoreEvent` → layout conversion
//! for every event family, distinct from whole notes.

use super::*;
use music::note::note::Note;

const BREVE_LOG2: i8 = -1;

fn breve() -> Duration {
    Duration::new(DurationKind::Breve, 0)
}

fn pitch(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn member_log2s(event: &MeasureEvent) -> Vec<i8> {
    match event {
        MeasureEvent::BeamGroup(group) => group.notes.iter().map(|n| n.duration_log2).collect(),
        MeasureEvent::TupletGroup(tuplet) => tuplet
            .beam_group
            .notes
            .iter()
            .map(|n| n.duration_log2)
            .collect(),
        other => panic!("expected a grouped event, got {other:?}"),
    }
}

#[test]
fn breve_note_rest_and_chord_convert_to_breve_log2_with_dots() {
    let contents = ScoreBuilder::new()
        .time_signature(4, 2)
        .note(pitch(Note::G, 4), breve())
        .barline()
        .rest(breve())
        .barline()
        .chord(
            vec![pitch(Note::G, 4), pitch(Note::B, 4), pitch(Note::D, 5)],
            breve(),
        )
        .barline()
        .note(pitch(Note::G, 4), Duration::new(DurationKind::Breve, 1))
        .rest(Duration::WHOLE)
        .barline()
        .build_measure_contents();

    let MeasureEvent::Note(note) = &contents[0].events[0] else {
        panic!("measure 1 holds the breve note");
    };
    assert_eq!((note.duration_log2, note.dots), (BREVE_LOG2, 0));

    let MeasureEvent::Rest(rest) = &contents[1].events[0] else {
        panic!("measure 2 holds the breve rest");
    };
    assert_eq!((rest.duration_log2, rest.dots), (BREVE_LOG2, 0));

    let MeasureEvent::Chord(chord) = &contents[2].events[0] else {
        panic!("measure 3 holds the breve chord");
    };
    assert_eq!((chord.duration_log2, chord.dots), (BREVE_LOG2, 0));

    let MeasureEvent::Note(dotted) = &contents[3].events[0] else {
        panic!("measure 4 starts with the dotted breve");
    };
    assert_eq!((dotted.duration_log2, dotted.dots), (BREVE_LOG2, 1));
    let MeasureEvent::Rest(whole_rest) = &contents[3].events[1] else {
        panic!("measure 4 ends with the whole rest");
    };
    assert_eq!(whole_rest.duration_log2, 0, "whole values stay whole");
}

#[test]
fn breve_members_of_beam_tuplet_and_styled_groups_keep_breve_log2() {
    let members = || {
        vec![
            (vec![pitch(Note::G, 4)], breve(), NoteAnnotations::default()),
            (
                vec![pitch(Note::B, 4), pitch(Note::D, 5)],
                breve(),
                NoteAnnotations::default(),
            ),
        ]
    };
    let contents = ScoreBuilder::new()
        .beam_group(vec![
            (pitch(Note::G, 4), breve()),
            (pitch(Note::A, 4), breve()),
        ])
        .tuplet_ratio(
            3,
            2,
            vec![
                (pitch(Note::G, 4), breve()),
                (pitch(Note::A, 4), breve()),
                (pitch(Note::B, 4), breve()),
            ],
        )
        .styled_beam_group(members())
        .styled_tuplet_ratio(3, 2, members())
        .barline()
        .build_measure_contents();

    let events = &contents[0].events;
    assert_eq!(member_log2s(&events[0]), vec![BREVE_LOG2; 2]);
    assert_eq!(member_log2s(&events[1]), vec![BREVE_LOG2; 3]);
    assert_eq!(member_log2s(&events[2]), vec![BREVE_LOG2; 2]);
    assert_eq!(member_log2s(&events[3]), vec![BREVE_LOG2; 2]);
}

#[test]
fn guitar_breve_projects_breve_log2_to_standard_notation() {
    let mut score = super::guitar::GuitarScore::standard();
    score.set_time_signature(4, 2);
    score.note(pitch(Note::G, 3), breve(), 3, 0).unwrap();
    score.barline().unwrap();
    score.rest(breve());
    score.barline().unwrap();

    let contents = score.notation_builder().build_measure_contents();
    let MeasureEvent::Note(note) = &contents[0].events[0] else {
        panic!("the fretted breve projects to a standard-staff note");
    };
    assert_eq!(note.duration_log2, BREVE_LOG2);
    let MeasureEvent::Rest(rest) = &contents[1].events[0] else {
        panic!("the guitar breve rest projects to a standard-staff rest");
    };
    assert_eq!(rest.duration_log2, BREVE_LOG2);
}
