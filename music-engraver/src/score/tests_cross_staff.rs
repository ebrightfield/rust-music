use super::*;
use crate::layout::accidental::{AccidentalDisplay, AccidentalPolicy};
use crate::layout::glissando::GlissandoStyle;
use crate::layout::measure::MeasureElement;
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::system::MeasureEvent;
use crate::error::EngraverError;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

// Spelled written pitches and destinations transcribed from the six
// mn-c12-i*.ly `\absolute` melody blocks (LilyPond c' = middle C).
const I001: &[(Note, i8, usize)] = &[
    (Note::Aes, 5, 0), (Note::E, 4, 0), (Note::Fis, 3, 1),
    (Note::A, 4, 0), (Note::G, 3, 1), (Note::Ees, 5, 0),
    (Note::Bes, 3, 1), (Note::F, 2, 1), (Note::D, 4, 0),
    (Note::C, 3, 1), (Note::Cis, 5, 0), (Note::B, 3, 1),
];
const I002: &[(Note, i8, usize)] = &[
    (Note::Fis,5,0),(Note::F,4,0),(Note::D,4,0),(Note::Cis,3,1),
    (Note::B,4,0),(Note::G,2,1),(Note::C,2,1),(Note::E,5,0),
    (Note::Bes,3,1),(Note::A,4,0),(Note::Ees,3,1),(Note::Aes,5,0),
];
const I003: &[(Note, i8, usize)] = &[
    (Note::C,3,1),(Note::G,4,0),(Note::Aes,2,1),(Note::B,3,0),
    (Note::Cis,5,0),(Note::Fis,3,1),(Note::E,5,0),(Note::D,3,1),
    (Note::Bes,2,1),(Note::Ees,4,0),(Note::F,5,0),(Note::A,3,1),
    (Note::C,6,0),(Note::Aes,2,1),
];
const I004: &[(Note, i8, usize)] = &[
    (Note::A,3,1),(Note::B,4,0),(Note::Cis,3,1),(Note::D,4,0),
    (Note::Ees,3,1),(Note::F,5,0),(Note::Fis,4,0),(Note::Gis,3,1),
    (Note::G,2,1),(Note::E,2,1),(Note::Bes,4,0),(Note::C,3,1),
];
const I005: &[(Note, i8, usize)] = &[
    (Note::Cis,5,0),(Note::C,4,0),(Note::Ees,4,0),(Note::Des,3,1),
    (Note::D,5,0),(Note::E,6,0),(Note::F,4,0),(Note::G,3,1),
    (Note::Aes,4,0),(Note::B,2,1),(Note::Bes,4,0),(Note::Ges,2,1),
];
const I006: &[(Note, i8, usize)] = &[
    (Note::D,2,1),(Note::A,5,0),(Note::Ees,4,0),(Note::Bes,2,1),
    (Note::Fis,4,0),(Note::G,2,1),(Note::E,2,1),(Note::F,5,0),
    (Note::B,3,0),(Note::C,3,1),(Note::Des,5,0),(Note::Aes,3,1),
];

fn voice(pitches: &[(Note, i8, usize)]) -> ScoreBuilder {
    let mut voice = ScoreBuilder::new().cadenza_on().accidental_policy(AccidentalPolicy::Forget);
    for (i, &(note, octave, stave)) in pitches.iter().enumerate() {
        voice = voice.note(Pitch::new(note, octave), Duration::WHOLE).on_staff(stave);
        if i + 1 < pitches.len() { voice = voice.glissando(GlissandoStyle::Dashed); }
    }
    voice.end_barline()
}

fn grand(voice: ScoreBuilder) -> MultiStaffScore {
    MultiStaffScore::grand_staff(
        ScoreBuilder::new().clef(Clef::Treble).accidental_policy(AccidentalPolicy::Forget),
        ScoreBuilder::new().clef(Clef::Bass).accidental_policy(AccidentalPolicy::Forget),
    ).cross_staff_voice(voice).system_width_fu(25_000.0)
}

#[test]
fn six_source_timelines_keep_pitch_staff_and_onset_identity() {
    for (name, source) in [("i001",I001),("i002",I002),("i003",I003),("i004",I004),("i005",I005),("i006",I006)] {
        let original = voice(source);
        let mut score = grand(original.clone());
        let mut source_voice = score.cross_staff_voice.take().unwrap();
        source_voice.flush_pending();
        let glisses = cross_staff::distribute_voice(source_voice, &mut score.staves).unwrap();
        assert_eq!(glisses.len(), source.len()-1, "{name}");
        let contents: Vec<_> = score.staves.iter().map(|s| s.build_measure_contents().unwrap()).collect();
        let mut actual = vec![None; source.len()];
        for (staff, measures) in contents.iter().enumerate() {
            assert_eq!(measures.len(),1,"{name}");
            assert_eq!(measures[0].barline,BarlineStyle::Final,"{name}");
            assert!(measures[0].meta.meter.is_none(),"{name}");
            for event in &measures[0].events {
                assert!(!matches!(event,MeasureEvent::Rest(_)),"{name}: phantom rest");
                if let MeasureEvent::Note(note) = event {
                    let id=note.annotations.cross_staff_id.unwrap();
                    assert!(actual[id].replace((staff,note.staff_position)).is_none(),"{name}: duplicated note");
                    let expected=Pitch::new(source[id].0,source[id].1);
                    let clef=if staff==0 {Clef::Treble} else {Clef::Bass};
                    assert_eq!(note.staff_position,pitch_to_staff_position(&expected,&clef),"{name} event {id}");
                    assert_eq!(staff,source[id].2,"{name} event {id}");
                }
            }
        }
        assert!(actual.iter().all(Option::is_some),"{name}: missing notes");
        let svg=grand(original).try_render_svg().unwrap();
        assert_eq!(svg.matches("stroke-dasharray=").count(),source.len()-1,"{name}: gliss count");
    }
}

#[test]
fn assigned_staff_accidentals_and_errors() {
    let voice=ScoreBuilder::new().cadenza_on().accidental_policy(AccidentalPolicy::Forget)
        .note_with_accidental(Pitch::new(Note::F,4),Duration::WHOLE,AccidentalDisplay::Force).on_staff(0)
        .note_with_accidental(Pitch::new(Note::Fis,3),Duration::WHOLE,AccidentalDisplay::Hide).on_staff(1)
        .note(Pitch::new(Note::Fis,3),Duration::WHOLE).on_staff(1)
        .end_barline();
    let mut score=grand(voice);
    let source=score.cross_staff_voice.take().unwrap();
    cross_staff::distribute_voice(source,&mut score.staves).unwrap();
    let top=score.staves[0].build_measure_contents().unwrap();
    let bottom=score.staves[1].build_measure_contents().unwrap();
    let top_note=top[0].events.iter().find_map(|e|if let MeasureEvent::Note(n)=e {Some(n)} else {None}).unwrap();
    assert!(top_note.accidental.is_some());
    let notes:Vec<_>=bottom[0].events.iter().filter_map(|e|if let MeasureEvent::Note(n)=e {Some(n)} else {None}).collect();
    assert!(notes[0].accidental.is_none());
    assert!(notes[1].accidental.is_some(),"forget ignores prior hidden accidentals");
    let error=grand(ScoreBuilder::new().note(Pitch::new(Note::C,4),Duration::WHOLE).on_staff(2)).try_render_svg().unwrap_err();
    assert!(matches!(error,EngraverError::CrossStaff(CrossStaffError::InvalidStaff{staff:2,..})));
    let error=grand(ScoreBuilder::new().voice(1).note(Pitch::new(Note::C,4),Duration::WHOLE)).try_render_svg().unwrap_err();
    assert!(matches!(error,EngraverError::CrossStaff(CrossStaffError::InvalidVoice{voice:1,..})));
}

#[test]
fn routed_chord_and_rest_keep_durations_and_destination_key() {
    let voice=ScoreBuilder::new().cadenza_on()
        .rest(Duration::QTR).on_staff(1)
        .chord(vec![Pitch::new(Note::Fis,3),Pitch::new(Note::A,3)],Duration::HALF).on_staff(1)
        .note(Pitch::new(Note::Fis,4),Duration::QTR).on_staff(0)
        .end_barline();
    let mut score=MultiStaffScore::grand_staff(
        ScoreBuilder::new().clef(Clef::Treble).key_signature(crate::layout::key_signature::KeySignature::Sharps(1)),
        ScoreBuilder::new().clef(Clef::Bass),
    );
    cross_staff::distribute_voice(voice,&mut score.staves).unwrap();
    let top=score.staves[0].build_measure_contents().unwrap();
    let bottom=score.staves[1].build_measure_contents().unwrap();
    assert!(matches!(&top[0].events[0],MeasureEvent::Spacer(s) if s.duration_log2==2));
    assert!(matches!(&top[0].events[1],MeasureEvent::Spacer(s) if s.duration_log2==1));
    assert!(matches!(&bottom[0].events[0],MeasureEvent::Rest(r) if r.duration_log2==2));
    let MeasureEvent::Chord(chord)=&bottom[0].events[1] else { panic!("chord routed to bass") };
    assert_eq!(chord.staff_positions,vec![
        pitch_to_staff_position(&Pitch::new(Note::Fis,3),&Clef::Bass),
        pitch_to_staff_position(&Pitch::new(Note::A,3),&Clef::Bass),
    ]);
    assert!(chord.accidentals[0].is_some(),"F sharp outside bass's open key");
    let MeasureEvent::Note(note)=&top[0].events[2] else { panic!("note routed to treble") };
    assert!(note.accidental.is_none(),"F sharp in treble's one-sharp key");
    assert!(matches!(&bottom[0].events[2],MeasureEvent::Spacer(s) if s.duration_log2==2));
}

#[test]
fn shared_columns_and_cross_system_fragments_with_independent_voice() {
    let voice=ScoreBuilder::new().cadenza_on()
        .note(Pitch::new(Note::C,5),Duration::WHOLE).on_staff(0).glissando(GlissandoStyle::Dashed)
        .note(Pitch::new(Note::G,2),Duration::WHOLE).on_staff(1).system_break().glissando(GlissandoStyle::Dashed)
        .note(Pitch::new(Note::E,5),Duration::WHOLE).on_staff(0).end_barline();
    let bass=ScoreBuilder::new().clef(Clef::Bass).cadenza_on()
        .note(Pitch::new(Note::C,3),Duration::HALF)
        .note(Pitch::new(Note::D,3),Duration::HALF)
        .note(Pitch::new(Note::E,3),Duration::WHOLE)
        .note(Pitch::new(Note::F,3),Duration::WHOLE).end_barline();
    let mut score=MultiStaffScore::grand_staff(ScoreBuilder::new().clef(Clef::Treble),bass)
        .cross_staff_voice(voice).explicit_line_breaks().system_width_fu(10_000.0);
    let mut source=score.cross_staff_voice.take().unwrap();
    source.flush_pending();
    score.cross_staff_breaks=Some(source.line_break_plan(&source.build_measure_contents().unwrap()));
    score.cross_staff_glissandos=cross_staff::distribute_voice(source,&mut score.staves).unwrap();
    let config=MeasureLayoutConfig::from_staff_space(bravura_font().engraving_config().staff_space);
    let (contents,chunks)=score.staves_into_systems(&config,10_000.0,4).unwrap();
    assert_eq!(chunks.len(),2,"break within the one logical measure is shared");
    for &(start,end) in &chunks {
        let prefixes:Vec<_>=contents.iter().map(|(m,p)|system_start_prefix(p,m,start)).collect();
        let slices:Vec<_>=contents.iter().zip(&prefixes).map(|((m,_),p)|(p,&m[start..end],m.get(end))).collect();
        let systems=layout_staves_followed_by(&slices,&config,Some(10_000.0));
        for measure_idx in 0..systems[0].measures.len() {
            let onsets:Vec<Vec<_>>=systems.iter().enumerate().map(|(staff,sys)| {
                let measure=&sys.measures[measure_idx];
                let layout=if staff==0 {&measure.layout} else {measure.additional_voice_layouts.last().unwrap()};
                layout.elements.iter().filter(|e|matches!(e.element,MeasureElement::Note(_) | MeasureElement::Spacer(_))).map(|e|e.x).collect()
            }).collect();
            assert_eq!(onsets[0],onsets[1],"shared exact x columns in each system");
            assert_eq!(systems[0].measures[measure_idx].shared_closing_barline_x,systems[1].measures[measure_idx].shared_closing_barline_x);
        }
    }
    let svg=MultiStaffScore::grand_staff(ScoreBuilder::new().clef(Clef::Treble),ScoreBuilder::new().clef(Clef::Bass)
        .cadenza_on().note(Pitch::new(Note::C,3),Duration::HALF).note(Pitch::new(Note::D,3),Duration::HALF)
        .note(Pitch::new(Note::E,3),Duration::WHOLE).note(Pitch::new(Note::F,3),Duration::WHOLE).end_barline())
        .cross_staff_voice(ScoreBuilder::new().cadenza_on()
            .note(Pitch::new(Note::C,5),Duration::WHOLE).on_staff(0).glissando(GlissandoStyle::Dashed)
            .note(Pitch::new(Note::G,2),Duration::WHOLE).on_staff(1).system_break().glissando(GlissandoStyle::Dashed)
            .note(Pitch::new(Note::E,5),Duration::WHOLE).on_staff(0).end_barline())
        .explicit_line_breaks().system_width_fu(10_000.0).try_render_svg().unwrap();
    assert_eq!(svg.matches("stroke-dasharray=").count(),3,"one full span plus two cross-system fragments");
}

#[cfg(feature="png")]
#[test]
fn full_source_excerpt_rasterizes_to_png() {
    let svg=grand(voice(I001)).try_render_svg().unwrap();
    let png=crate::render::png::svg_to_png(&svg,2.0).unwrap();
    assert_eq!(&png[..8],b"\x89PNG\r\n\x1a\n");
    if let Ok(path)=std::env::var("CROSS_STAFF_VISUAL_OUT") {
        std::fs::write(path,png).unwrap();
    }
}
