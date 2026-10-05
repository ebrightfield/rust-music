//! Literal excerpts from the certified Modus Novus packets: source .ly, PDF and
//! metadata printed-page references are the oracle, not fixture-generated text.
use super::*;
use crate::layout::analysis_bracket::{AnalysisBracketSpec, AnalysisBracketStyle};
use crate::layout::barline::BarlineStyle;
use crate::layout::group::{BeamSpec, GroupMark, TupletSpec};
use crate::layout::lyric::{LyricStyle, LyricSyllable};
use crate::layout::measure::MeasureElement;
use crate::layout::measure_meta::MeasureLength;
use crate::layout::placement::Placement;
use crate::layout::stem::StemDirection;
use crate::layout::system::MeasureEvent;
use crate::layout::tempo::{MetronomeMark, MetronomeNoteKind as M, MetronomeUnit, MetronomeValue, TempoMark};
use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;

fn p(note: Note, octave: i8) -> Pitch { Pitch::new(note, octave) }
fn dotted(kind: DurationKind) -> Duration { Duration::new(kind, 1) }
fn length(n: u64, d: u64) -> MeasureLength { MeasureLength::new(n, d) }
fn measures(score: &ScoreBuilder) -> Vec<crate::layout::system::MeasureContent> {
    score.build_measure_contents().expect("source fragment converts")
}
fn page(score: &ScoreBuilder) -> crate::layout::page::PageLayout {
    score.clone().page_layout(250.0).unwrap().unwrap()
}
fn note_xs(score: &ScoreBuilder, bar: usize) -> Vec<f64> {
    let page = page(score);
    page.systems.iter().flat_map(|s| s.system.measures.iter())
        .nth(bar).unwrap().layout.elements.iter()
        .filter_map(|e| matches!(e.element, MeasureElement::Note(_)).then_some(e.x))
        .collect()
}
fn line_attr(line: &str, key: &str) -> f64 {
    line.split_once(&format!("{key}=\"")).unwrap().1.split('"').next().unwrap().parse().unwrap()
}
fn text_line<'a>(svg: &'a str, word: &str) -> &'a str {
    svg.lines().find(|line| line.contains(&format!(">{word}</text>"))).unwrap()
}
fn tempo(unit: impl Into<MetronomeUnit>, bpm: u16) -> TempoMark {
    TempoMark::metronome(MetronomeMark::bpm(unit, bpm).approx().parenthesized())
}

/// mn-c06-m004, printed p.60, bars 1–3 and 7: printed 3/4 → 4/4
/// with unprinted 9/8 length → regular 4/4; breath on bar 7 beam member.
#[test]
fn mn_c06_m004_overridden_bar_and_group_member_breath() {
    let score = ScoreBuilder::new().time_signature(3, 4).first_measure_number(1)
        .note(p(Note::B, 4), dotted(DurationKind::Qtr)).tempo(tempo(M::Quarter, 80))
        .note(p(Note::A, 4), Duration::EIGHTH)
        .beam_group(vec![(p(Note::Bes, 4), Duration::EIGHTH), (p(Note::C, 5), Duration::EIGHTH)])
        .barline().time_signature_change(4, 4).measure_length(9, 8)
        .note(p(Note::E, 4), Duration::QTR).note(p(Note::Ees, 4), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::QTR).rest(Duration::EIGHTH)
        .beam_group(vec![(p(Note::Fis, 4), Duration::EIGHTH), (p(Note::D, 5), Duration::EIGHTH), (p(Note::C, 5), Duration::EIGHTH)])
        .barline().reset_measure_length()
        .note(p(Note::B, 4), Duration::QTR)
        .begin_beam().note(p(Note::Ees, 4), Duration::EIGHTH).note(p(Note::C, 4), Duration::EIGHTH).end_beam()
        .begin_beam().note(p(Note::F, 4), Duration::EIGHTH).note(p(Note::Des, 4), Duration::EIGHTH)
        .note(p(Note::A, 3), Duration::EIGHTH).end_beam().rest(Duration::EIGHTH).barline();
    let bars = measures(&score);
    assert_eq!(bars.iter().map(|b| (b.meta.number, b.meta.actual_length, b.meta.nominal_length)).collect::<Vec<_>>(),
        [(1, length(3, 4), Some(length(3, 4))), (2, length(9, 8), Some(length(9, 8))), (3, length(1, 1), Some(length(1, 1)))]);
    let first = bars[0].events.iter().find_map(|e| if let MeasureEvent::Note(n) = e {Some(n)} else {None}).unwrap();
    assert_eq!(first.annotations.tempo_mark.as_ref().unwrap(), &tempo(M::Quarter, 80));
    let svg = score.render_svg();
    assert!(svg.contains("c. 80"));
    let beam_breath = ScoreBuilder::new().time_signature(3,4).first_measure_number(7)
        .begin_beam()
        .note(p(Note::D,4),Duration::EIGHTH).breath_mark(crate::layout::breath::BreathMark::Comma)
        .note(p(Note::Des,5),Duration::EIGHTH).note(p(Note::Bes,4),Duration::EIGHTH)
        .note_with_accidental(p(Note::D,4),Duration::EIGHTH,AccidentalDisplay::Force)
        .note(p(Note::B,3),Duration::EIGHTH).note(p(Note::G,4),Duration::EIGHTH)
        .end_beam().barline();
    let bar=measures(&beam_breath);
    assert_eq!(bar[0].meta.actual_length,length(3,4));
    let breath=bar[0].events.iter().find_map(|e| if let MeasureEvent::Note(n)=e{Some(n.annotations.breath_mark)}else{None}).unwrap();
    assert_eq!(breath,Some(crate::layout::breath::BreathMark::Comma));
    let svg=beam_breath.render_svg();
    assert!(svg.contains("<polygon"));
}

/// mn-c11-r015, printed p.98, bars 10, 14–15 and 28–32: no new
/// printed signature in the short bars; bars 29 and 31 are *empty*, not rests.
#[test]
fn mn_c11_r015_short_bars_and_two_empty_spacer_bars() {
    let short = ScoreBuilder::new().time_signature(3, 4).first_measure_number(10)
        .measure_length(3, 8).note(p(Note::Bes, 4), Duration::EIGHTH)
        .begin_beam().note(p(Note::Aes, 4), Duration::EIGHTH).note(p(Note::Ges, 4), Duration::EIGHTH).end_beam()
        .barline().measure_length(2, 4).rest(Duration::QTR).rest(Duration::QTR)
        .barline().measure_length(1, 4).note(p(Note::Bes, 4), Duration::QTR).barline();
    let bars = measures(&short);
    assert_eq!(bars.iter().map(|m| (m.meta.number, m.meta.actual_length, m.meta.nominal_length)).collect::<Vec<_>>(),
        [(10,length(3,8),Some(length(3,8))), (11,length(1,2),Some(length(1,2))), (12,length(1,4),Some(length(1,4)))]);
    let empty = ScoreBuilder::new().time_signature(3,4).first_measure_number(28)
        .note(p(Note::E,4), dotted(DurationKind::Half)).barline()
        .spacer(dotted(DurationKind::Half)).barline()
        .note_with_accidental(p(Note::C,5), dotted(DurationKind::Half), AccidentalDisplay::Force).barline()
        .spacer(dotted(DurationKind::Half)).barline()
        .note(p(Note::Aes,4), dotted(DurationKind::Half)).barline();
    let bars = measures(&empty);
    assert_eq!(bars.iter().map(|m| m.meta.number).collect::<Vec<_>>(), [28,29,30,31,32]);
    for i in [1,3] {
        assert_eq!(bars[i].meta.actual_length, length(3,4));
        assert!(matches!(bars[i].events[..], [MeasureEvent::Spacer(_)]));
    }
    let layout = page(&empty);
    for i in [1,3] {
        let m = &layout.systems[0].system.measures[i].layout;
        assert!(m.elements.iter().all(|e| !matches!(e.element, MeasureElement::Rest(_)|MeasureElement::Note(_))));
        assert!(m.closing_barline_x() >= crate::layout::measure::MeasureLayoutConfig::from_staff_space(250.0).empty_measure_min_width);
    }
    let svg = empty.render_svg();
    assert!(svg.contains("<svg"));
}

/// mn-c11-r027, printed pp.102–103: contiguous tail of the first unmetered
/// 24/4 phrase around its invisible mid-measure break. The source phrase
/// continues after this fragment: only one logical bar number is consumed.
#[test]
fn mn_c11_r027_inline_invisible_split_keeps_long_phrase() {
    let score = ScoreBuilder::new().clef(Clef::Bass).hidden_time_signature(24,4)
        .first_measure_number(1).accidental_policy(AccidentalPolicy::Forget)
        .note(p(Note::Cis,4), Duration::QTR)
        .note_with_accidental(p(Note::A,3), Duration::QTR, AccidentalDisplay::Force)
        .begin_beam().note(p(Note::D,3), Duration::EIGHTH)
        .note_with_accidental(p(Note::B,3), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam().inline_barline(BarlineStyle::Invisible).system_break()
        .begin_beam()
        .note_with_accidental(p(Note::E,3), Duration::EIGHTH, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::C,3), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::E,3), Duration::EIGHTH).note(p(Note::C,3), Duration::EIGHTH)
        .end_beam()
        .begin_tuplet(TupletSpec::new(3,2)).begin_beam()
        .note(p(Note::E,3), Duration::EIGHTH).note(p(Note::C,3), Duration::EIGHTH)
        .note_with_accidental(p(Note::F,2), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam().end_tuplet().barline_style(BarlineStyle::Invisible);
    let bars = measures(&score);
    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].meta.number, 1);
    assert_eq!(bars[0].meta.nominal_length, Some(length(6,1)));
    assert_eq!(bars[0].meta.actual_length, length(3,2)); // 3/4 + 1/2 + (3 eighths at 2:3 = 1/4).
    assert!(!bars[0].meta.meter_visible);
    let notes: Vec<_> = bars[0].events.iter().filter_map(|e| if let MeasureEvent::Note(n)=e {Some(n)}else{None}).collect();
    assert_eq!(notes[4].accidental.unwrap().glyph, smufl::Glyph::AccidentalNatural);
    assert!(notes[6].accidental.is_none(), "unmarked E after forced E remains unmarked under forget policy");
    let page = page(&score);
    assert_eq!(page.systems.len(), 2);
    assert_eq!(page.systems[0].system.measures[0].meta.number, page.systems[1].system.measures[0].meta.number);
    assert_eq!(page.systems[0].system.measures[0].meta.number, 1);
    score.render_svg();
}

/// mn-c04-r007, printed p.43, last 9/8 + first 6/8 bar:
/// stem-down beam, secondary subdivision, dashed bracket BELOW across meter.
#[test]
fn mn_c04_r007_subdivided_stems_and_bracket_over_meter() {
    let bracket = AnalysisBracketSpec::new(AnalysisBracketStyle::Dashed, Placement::Below);
    let score = ScoreBuilder::new().time_signature(9,8).first_measure_number(4)
        .begin_beam_with(BeamSpec::new().subdivide(3))
        .note(p(Note::G,4), Duration::EIGHTH).note(p(Note::D,4), Duration::EIGHTH)
        .note(p(Note::G,4), Duration::EIGHTH).end_beam()
        .begin_beam_with(BeamSpec::new().stem_direction(StemDirection::Down).subdivide(3))
        .note(p(Note::G,4), Duration::EIGHTH).note(p(Note::Gis,4), Duration::EIGHTH)
        .note(p(Note::Cis,5), Duration::EIGHTH).end_beam()
        .begin_beam_with(BeamSpec::new().subdivide(3))
        .note(p(Note::Cis,5), Duration::EIGHTH)
        .note(p(Note::D,5), Duration::EIGHTH).analysis_bracket_start(bracket)
        .note_with_accidental(p(Note::G,5), Duration::EIGHTH, AccidentalDisplay::Force).end_beam()
        .barline().time_signature_change(6,8)
        .note(p(Note::G,5), Duration::EIGHTH).note(p(Note::Gis,5), Duration::QTR)
        .note(p(Note::Cis,6), Duration::QTR).note(p(Note::D,5), Duration::EIGHTH)
        .analysis_bracket_end().barline_style(BarlineStyle::Invisible);
    let bars = measures(&score);
    assert_eq!((bars[0].meta.actual_length, bars[1].meta.actual_length), (length(9,8),length(3,4)));
    assert_eq!(bars[1].meta.meter, Some(crate::layout::time_signature::TimeSignatureKind::Numeric {numerator:6,denominator:8}));
    assert_eq!(bars[1].barline, BarlineStyle::Invisible);
    assert!(bars[0].events.iter().any(|e| matches!(e, MeasureEvent::GroupMark(GroupMark::BeamStart {spec,..}) if spec.stem_direction==Some(StemDirection::Down) && spec.subdivide_log2==Some(3))));
    let xs = note_xs(&score,0);
    let svg = score.render_svg();
    let bracket_line = svg.lines().find(|l| l.contains("stroke-dasharray=") && l.contains("<line ") && (line_attr(l,"y1")-line_attr(l,"y2")).abs()<0.01).unwrap();
    assert!((line_attr(bracket_line,"x1") - xs[7]).abs()<0.01);
    assert!(line_attr(bracket_line,"x2") > xs[8]);
}

/// mn-c04-r010, printed p.44: 11-sixteenth pickup, approximate dotted
/// quarter tempo, dashed bracket ABOVE on a later beamed member and gliss.
#[test]
fn mn_c04_r010_pickup_and_member_gliss() {
    let score = ScoreBuilder::new().time_signature(6,8).partial(length(11,16))
        .begin_beam().note(p(Note::Gis,4), Duration::SIXTEENTH)
        .tempo(tempo(MetronomeUnit::dotted(M::Quarter),140))
        .note(p(Note::Cis,5), Duration::SIXTEENTH)
        .note(p(Note::E,5), Duration::SIXTEENTH).end_beam()
        .note(p(Note::D,6), Duration::HALF).barline()
        .note(p(Note::D,6), Duration::EIGHTH).note(p(Note::A,5), Duration::EIGHTH)
        .note(p(Note::B,5), Duration::EIGHTH).note(p(Note::E,5), Duration::EIGHTH)
        .note(p(Note::F,5), Duration::EIGHTH).note(p(Note::G,5), Duration::EIGHTH)
        .barline().begin_beam()
        .note(p(Note::Gis,5), dotted(DurationKind::Eighth))
        .note(p(Note::Fis,5), dotted(DurationKind::Eighth)).end_beam()
        .note(p(Note::Cis,5), dotted(DurationKind::Qtr)).barline()
        .note(p(Note::Cis,5), dotted(DurationKind::Half)).barline()
        .note(p(Note::Cis,5), dotted(DurationKind::Qtr))
        .begin_beam().note(p(Note::Cis,5), dotted(DurationKind::Eighth))
        .analysis_bracket_start(AnalysisBracketSpec::new(AnalysisBracketStyle::Dashed,Placement::Above))
        .glissando(crate::layout::glissando::GlissandoStyle::Line)
        .note(p(Note::Gis,4), dotted(DurationKind::Eighth)).end_beam().barline()
        .note(p(Note::B,4), dotted(DurationKind::Half)).analysis_bracket_end()
        .barline_style(BarlineStyle::Single);
    let bars = measures(&score);
    assert_eq!((bars[0].meta.number,bars[0].meta.actual_length,bars[0].meta.anacrusis),(0,length(11,16),true));
    assert_eq!(bars[1].meta.number,1);
    let m = match &bars[0].events[1] { MeasureEvent::Note(n) => n, _ => panic!("first pickup note") };
    assert_eq!(m.annotations.tempo_mark.as_ref().unwrap(), &tempo(MetronomeUnit::dotted(M::Quarter),140));
    let svg = score.clone().render_svg();
    assert!(svg.contains("c. 140"));
    assert_eq!(bars.iter().map(|b| b.meta.actual_length).collect::<Vec<_>>(), [length(11,16),length(3,4),length(3,4),length(3,4),length(3,4),length(3,4)]);
    let above = page(&score);
    let staff_y = above.systems.last().unwrap().y;
    assert!(svg.lines().any(|l| l.contains("stroke-dasharray=") && l.contains("<line ") && line_attr(l,"y1") < staff_y));
}

/// mn-c08-r002, printed p.70: cadenza bass, two unbeamed 3:2
/// eighth-note groups with independent Swedish and German lyric lanes.
#[test]
fn mn_c08_r002_unbeamed_tuplets_and_two_verse_underlay() {
    let mut score = ScoreBuilder::new().clef(Clef::Bass).cadenza_on()
        .note(p(Note::C,3), Duration::EIGHTH)
        .lyric_verse(1,LyricSyllable::word("Jag"),LyricStyle::Upright)
        .lyric_verse(2,LyricSyllable::word("Ich"),LyricStyle::Upright)
        .note(p(Note::C,3), Duration::EIGHTH)
        .lyric_verse(1,LyricSyllable::word("är"),LyricStyle::Upright)
        .lyric_verse(2,LyricSyllable::word("bin"),LyricStyle::Upright);
    for (pitch,sw,de) in [(Note::F,"Un","Un"),(Note::F,"zu","zu"),(Note::F,"från","vom")] {
        if sw=="Un" {score=score.begin_tuplet(TupletSpec::new(3,2).placement(crate::layout::tuplet::TupletPlacement::Above));}
        score=score.note(p(pitch,3),Duration::EIGHTH)
            .lyric_verse(1,LyricSyllable::word(sw),LyricStyle::Upright)
            .lyric_verse(2,LyricSyllable::word(de),LyricStyle::Upright);
    }
    score=score.end_tuplet().begin_tuplet(TupletSpec::new(3,2).placement(crate::layout::tuplet::TupletPlacement::Above));
    for (sw,de) in [("byn","Dorf"),("här","ne"),("in","ben")] {
        score=score.note(p(Note::B,2),Duration::EIGHTH)
            .lyric_verse(1,LyricSyllable::word(sw),LyricStyle::Upright)
            .lyric_verse(2,LyricSyllable::word(de),LyricStyle::Upright);
    }
    score=score.end_tuplet().note(p(Note::D,3),Duration::QTR)
        .lyric_verse(1,LyricSyllable::word("till."),LyricStyle::Upright)
        .lyric_verse(2,LyricSyllable::word("an."),LyricStyle::Upright)
        .barline_style(BarlineStyle::Double);
    let bars=measures(&score);
    assert_eq!(bars[0].meta.nominal_length,None);
    assert_eq!(bars[0].meta.actual_length,length(1,1));
    assert_eq!(bars[0].events.iter().filter(|e| matches!(e,MeasureEvent::GroupMark(GroupMark::TupletStart{..}))).count(),2);
    assert!(!bars[0].events.iter().any(|e| matches!(e,MeasureEvent::GroupMark(GroupMark::BeamStart{..}))));
    let xs=note_xs(&score,0);
    assert_eq!(xs.len(),9);
    assert!(xs.windows(2).all(|pair| pair[1]>pair[0]));
    let svg=score.render_svg();
    assert_eq!(line_attr(text_line(&svg,"Jag"),"y")+500.0,line_attr(text_line(&svg,"Ich"),"y"));
    assert!(line_attr(text_line(&svg,"Un"),"x")>=xs[2]-100.0);
}

/// mn-c11-r028, printed p.103, bars 1–3 and 7: R1 is an H-bar;
/// the mixed quarter/eighth triplet occupies half a measure with its
/// final two eighths beamed, not the three quarter members.
#[test]
fn mn_c11_r028_rest_identity_and_partial_inner_beam_ratio() {
    let opening=ScoreBuilder::new().clef(Clef::Bass).time_signature(4,4)
        .multi_measure_rest(1).barline()
        .rest(Duration::HALF).rest(Duration::QTR).rest(Duration::EIGHTH)
        .note(p(Note::E,2),Duration::EIGHTH).barline()
        .note(p(Note::F,2),Duration::QTR)
        .begin_tuplet(TupletSpec::new(3,2)).begin_beam()
        .note(p(Note::E,2),Duration::EIGHTH).note(p(Note::E,2),Duration::EIGHTH).note(p(Note::E,2),Duration::EIGHTH)
        .end_beam().end_tuplet().note(p(Note::F,2),dotted(DurationKind::Qtr))
        .note(p(Note::E,2),Duration::EIGHTH).barline();
    let bars=measures(&opening);
    assert!(matches!(bars[0].events[..],[MeasureEvent::MultiMeasureRest{count:1,..}]));
    assert_eq!(bars.iter().map(|b|b.meta.actual_length).collect::<Vec<_>>(),[length(0,1),length(1,1),length(1,1)]); // R1 is a multimeasure rest, not a rhythmic note.
    let svg=opening.render_svg();
    assert!(svg.contains("<rect"),"printed R1 has a multi-measure H-bar, not a whole-note rest");
    let bar7=ScoreBuilder::new().clef(Clef::Bass).time_signature(4,4).first_measure_number(7)
        .note(p(Note::B,2),dotted(DurationKind::Qtr)).note(p(Note::Ais,2),Duration::EIGHTH)
        .begin_tuplet(TupletSpec::new(3,2))
        .note(p(Note::D,3),Duration::QTR).note(p(Note::Cis,3),Duration::QTR)
        .begin_beam().note(p(Note::Cis,3),Duration::EIGHTH).note(p(Note::Cis,3),Duration::EIGHTH)
        .end_beam().end_tuplet().barline();
    let bars=measures(&bar7);
    assert_eq!(bars[0].meta.actual_length,length(1,1));
    let marks:Vec<_>=bars[0].events.iter().filter_map(|e|if let MeasureEvent::GroupMark(m)=e{Some(m)}else{None}).collect();
    assert!(matches!(marks.as_slice(),[GroupMark::TupletStart{..},GroupMark::BeamStart{..},GroupMark::BeamEnd{..},GroupMark::TupletEnd{..}]));
    assert_eq!(bars[0].events.iter().filter(|e|matches!(e,MeasureEvent::Note(_))).count(),6);
    assert!(note_xs(&bar7,0).windows(2).all(|xs| xs[1]>xs[0]));
    let svg=bar7.render_svg();
    assert!(svg.contains("<polygon"));
}

/// mn-c11-r029, printed p.104 (Aniara score p.54), bar 7:
/// simultaneous G4 eighths above G4 sixteenth/sixteenth/eighth cue voice;
/// German `associatedVoice` follows the lower layer, Swedish the upper.
#[test]
fn mn_c11_r029_interior_voice_grid_cue_lyrics_and_hidden_bar() {
    let mut score=ScoreBuilder::new().time_signature(4,4).first_measure_number(7)
        .note(p(Note::F,5),dotted(DurationKind::Eighth)).note(p(Note::G,4),Duration::SIXTEENTH)
        .voice(0).begin_beam().note(p(Note::G,4),Duration::EIGHTH)
        .lyric_verse(1,LyricSyllable::with_hyphen("laps"),LyricStyle::Upright)
        .note(p(Note::G,4),Duration::EIGHTH)
        .lyric_verse(1,LyricSyllable::word("i"),LyricStyle::Upright).end_beam()
        .begin_tuplet(TupletSpec::new(3,2))
        .note(p(Note::B,4),Duration::QTR).note(p(Note::B,4),Duration::QTR)
        .begin_beam().note(p(Note::B,4),Duration::EIGHTH).note(p(Note::Cis,5),Duration::EIGHTH)
        .end_beam().end_tuplet()
        .voice(1).spacer(Duration::QTR).begin_beam()
        .note(p(Note::G,4),Duration::SIXTEENTH)
        .lyric_verse_on_voice(1,2,LyricSyllable::with_hyphen("sam"),LyricStyle::Italic)
        .note(p(Note::G,4),Duration::SIXTEENTH).note_size(NoteSize::Cue)
        .lyric_verse_on_voice(1,2,LyricSyllable::with_hyphen("men"),LyricStyle::Italic)
        .note(p(Note::G,4),Duration::EIGHTH).note_size(NoteSize::Cue)
        .lyric_verse_on_voice(1,2,LyricSyllable::word("bruch"),LyricStyle::Italic)
        .end_beam().barline().hidden_time_signature_change(7,8)
        .begin_beam().note(p(Note::Fis,5),Duration::EIGHTH)
        .note(p(Note::Fis,5),Duration::EIGHTH).end_beam().rest(Duration::QTR)
        .note(p(Note::G,5),Duration::EIGHTH).note(p(Note::C,5),dotted(DurationKind::Eighth))
        .note(p(Note::C,5),Duration::SIXTEENTH).barline();
    let bars=measures(&score);
    assert_eq!(bars[0].meta.actual_length,length(1,1));
    assert_eq!(bars[1].meta.actual_length,length(7,8));
    assert!(!bars[1].meta.meter_visible);
    let secondary:Vec<_>=bars[0].additional_voices[0].iter().filter_map(|e|if let MeasureEvent::Note(n)=e{Some(n)}else{None}).collect();
    assert_eq!(secondary.len(),3);
    assert_eq!(secondary.iter().map(|n|n.annotations.size).collect::<Vec<_>>(),[NoteSize::Normal,NoteSize::Cue,NoteSize::Cue]);
    assert_eq!(secondary.iter().map(|n|n.annotations.lyrics[0].syllable.text.as_str()).collect::<Vec<_>>(),["sam","men","bruch"]);
    let main:Vec<_>=bars[0].events.iter().filter_map(|e|if let MeasureEvent::Note(n)=e{Some(n)}else{None}).collect();
    assert_eq!(main[2].annotations.lyrics[0].syllable.text,"laps");
    let layout=page(&score);
    let first=&layout.systems[0].system.measures[0];
    let primary_g:Vec<_>=first.layout.elements.iter().filter_map(|e| match &e.element {MeasureElement::Note(n) if n.staff_position==2 => Some(e.x), _=>None}).collect();
    let secondary_g:Vec<_>=first.additional_voice_layouts[0].elements.iter().filter_map(|e| match &e.element {MeasureElement::Note(n) if n.staff_position==2 => Some(e.x), _=>None}).collect();
    assert_eq!((primary_g.len(),secondary_g.len()),(3,3));
    assert!((primary_g[1]-secondary_g[0]).abs()<0.01,"upper and lower G4 share interior beat column");
    assert!((primary_g[2]-secondary_g[2]).abs()<0.01,"final eighths share the next column");
    assert!(secondary_g[0]<secondary_g[1] && secondary_g[1]<secondary_g[2]);
    let svg=score.render_svg();
    assert!(text_line(&svg,"sam").contains("font-style=\"italic\""));
    assert!(line_attr(text_line(&svg,"laps"),"y")<line_attr(text_line(&svg,"sam"),"y"));
    assert!(svg.contains("scale(0.7"),"small cue noteheads print in secondary voice");
    score=ScoreBuilder::new().time_signature(4,4).rest(Duration::HALF)
        .tempo(TempoMark::metronome(MetronomeMark::bpm(M::Quarter,112).parenthesized()))
        .rest(Duration::EIGHTH).begin_beam()
        .note(p(Note::Des,5),Duration::SIXTEENTH)
        .note(p(Note::Des,5),Duration::SIXTEENTH).end_beam()
        .begin_beam().note(p(Note::Des,5),Duration::EIGHTH)
        .note(p(Note::C,5),Duration::EIGHTH).end_beam().barline();
    assert_eq!(measures(&score)[0].meta.actual_length,length(1,1));
    let MeasureEvent::Rest(rest)=&measures(&score)[0].events[0] else {panic!("leading half rest")};
    assert_eq!(rest.annotations.tempo_mark.as_ref().unwrap().metronome.as_ref().unwrap().value,MetronomeValue::Bpm(112));
    assert!(score.render_svg().contains("112"));
}
