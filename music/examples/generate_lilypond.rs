/// This example requires that you have `lilypond` installed on your system.
use std::path::PathBuf;
use music::{pitch, Note, Pitch, voicing, Voicing};
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::notation::lilypond::command::LilypondCmdBuilder;
use music::notation::lilypond::document::LilypondBuilder;
use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
use music::notation::rhythm::{RhythmicNotatedEvent, Tuplet};

fn main() {

    // 1. Produce some musical content
    // We can assign rhythmic values to musical events like pitches and voicings.
    let musical_events = vec![
        RhythmicNotatedEvent::voicing(voicing![
            pitch!(c, 3),
            pitch!(e, 3),
            pitch!(bes, 3)
        ], Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(a, 3), Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(g, 3), Duration::QTR),
        Tuplet::new(
            vec![
                RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::EIGHTH),
                RhythmicNotatedEvent::pitch(pitch!(dis, 4), Duration::EIGHTH),
                RhythmicNotatedEvent::pitch(pitch!(e, 4), Duration::EIGHTH),
            ],
            3, 2, DurationKind::Eighth
        ).into(),
        RhythmicNotatedEvent::pitch(pitch!(g, 4), Duration::QTR),
    ];
    // 1b. Type-wrap the content for Lilypond engraving.
    // Since Lilypond allows all sorts of markups that are unique to its engraving system,
    // we first convert these into a Lilypond-specific type.
    let musical_events = musical_events
        .into_iter()
        .map(|e| e.into())
        .collect();

    // 2. Arrange the content into a score.
    // We can then create a staff, potentially with multiple voices.
    let another_voice = vec![
        RhythmicNotatedEvent::pitch(pitch!(c, 5), Duration::WHOLE).into()
    ];
    let staff = LilypondStaff::new()
        .add_voice(musical_events)
        .add_voice(another_voice);
    // And put that staff in a score
    let score = LilypondScore::new()
        .staff_group(LilypondStaffGroup::new(vec![staff]));

    // 3. Create and compile a document with that score.
    // Create a virtual Lilypond document with that score
    let doc = LilypondBuilder::new()
        .path(Some(PathBuf::from("target/test.ly")))
        .score(Some(score));
    // And build/compile it (requires `lilypond` installed).
    LilypondCmdBuilder::new()
        .builder(doc)
        .output(Some(PathBuf::from("target/")))
        .build_and_compile().unwrap();
}