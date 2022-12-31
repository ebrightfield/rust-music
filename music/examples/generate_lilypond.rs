use std::path::PathBuf;
use music::{pitch, Note, Pitch, voicing, Voicing};
use music::notation::duration::Duration;
use music::notation::lilypond::command::LilypondCmdBuilder;
use music::notation::lilypond::document::LilypondBuilder;
use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
use music::notation::rhythm::RhythmicNotatedEvent;

fn main() {
    // We can assign rhythmic values to musical events like pitches and voicings.
    let musical_events = vec![
        RhythmicNotatedEvent::voicing(voicing![
            pitch!(c, 3),
            pitch!(e, 3),
            pitch!(bes, 3)
        ], Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(a, 3), Duration::QTR),
        RhythmicNotatedEvent::pitch(pitch!(g, 3), Duration::QTR),
        RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::QTR),
        RhythmicNotatedEvent::pitch(pitch!(dis, 4), Duration::QTR),
        RhythmicNotatedEvent::pitch(pitch!(e, 4), Duration::QTR),
        RhythmicNotatedEvent::pitch(pitch!(g, 4), Duration::QTR),
    ];
    // But since Lilypond allows all sorts of markups that are unique to its engraving system,
    // we first convert these into a Lilypond-specific type.
    let musical_events = musical_events
        .into_iter()
        .map(|e| e.into())
        .collect();
    // We can then create a simple staff and score.
    let mut staff = LilypondStaff::new()
        .add_voice(musical_events);
    let score = LilypondScore::new()
        .staff_group(LilypondStaffGroup::new(vec![staff]));

    // And create a document.
    let doc = LilypondBuilder::new()
        .path(Some(PathBuf::from("target/test.ly")))
        .score(Some(score));
    // And build/compile it.
    LilypondCmdBuilder::new()
        .builder(doc)
        .output(Some(PathBuf::from("target/")))
        .build_and_compile().unwrap();
}