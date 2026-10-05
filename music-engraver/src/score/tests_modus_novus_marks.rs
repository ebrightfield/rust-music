//! Source-derived Modus Novus fragments: public builder → layout → SVG.
//! Each comment names the source ID and printed page, not a LilyPond parser.

use super::ScoreBuilder;
use crate::font::bravura_font;
use crate::layout::accidental::{AccidentalDisplay, AccidentalPolicy};
use crate::layout::analysis_bracket::{AnalysisBracketSpec, AnalysisBracketStyle};
use crate::layout::articulation::{Articulation, ArticulationMark};
use crate::layout::barline::BarlineStyle;
use crate::layout::breath::BreathMark;
use crate::layout::grace::{GraceNoteKind, GraceNotes};
use crate::layout::lyric::{LyricStyle, LyricSyllable};
use crate::layout::measure::{MeasureElement, StemVisibility};
use crate::layout::measure_meta::MeasureLength;
use crate::layout::placement::Placement;
use crate::layout::system::MeasureEvent;
use crate::layout::tempo::{
    MetronomeMark, MetronomeNoteKind as Met, MetronomeUnit, TempoMark, TempoText,
};
use crate::layout::text_script::TextScript;
use crate::svg_probe::{glyph, glyphs, lines, text, texts};
use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use smufl::Glyph;

fn p(n: Note, o: i8) -> Pitch {
    Pitch::new(n, o)
}
fn dotted(k: DurationKind) -> Duration {
    Duration::new(k, 1)
}
fn page(score: &ScoreBuilder) -> crate::layout::page::PageLayout {
    score
        .clone()
        .page_layout(bravura_font().engraving_config().staff_space)
        .unwrap()
        .unwrap()
}
fn note_positions(score: &ScoreBuilder) -> Vec<f64> {
    page(score)
        .systems
        .iter()
        .flat_map(|s| {
            s.system
                .measures
                .iter()
                .flat_map(|m| {
                    m.layout
                        .elements
                        .iter()
                        .filter_map(|e| {
                            matches!(
                                e.element,
                                MeasureElement::Note(_) | MeasureElement::Chord(_)
                            )
                            .then_some(s.x + m.x_offset + e.x)
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn mark(k: Met, bpm: u16) -> TempoMark {
    TempoMark::metronome(MetronomeMark::bpm(k, bpm).parenthesized())
}

fn staff_positions(score: &ScoreBuilder) -> Vec<i8> {
    score
        .clone()
        .build_measure_contents()
        .unwrap()
        .iter()
        .flat_map(|measure| {
            measure
                .events
                .iter()
                .filter_map(|event| match event {
                    MeasureEvent::Note(note) => Some(note.staff_position),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// mn-c03-m008, printed p. 38: cut C → 3/2 → 6/4, with a quarter=quarter reset.
#[test]
fn melody_three_meters_equation_and_cautionary() {
    let score = ScoreBuilder::new()
        .cut_time()
        .note(p(Note::C, 4), Duration::HALF)
        .tempo(mark(Met::Half, 63))
        .note(p(Note::F, 4), Duration::HALF)
        .barline()
        .time_signature_change(3, 2)
        .begin_beam()
        .note(p(Note::C, 5), Duration::EIGHTH)
        .note(p(Note::B, 4), Duration::EIGHTH)
        .end_beam()
        .note_with_accidental(p(Note::D, 5), Duration::HALF, AccidentalDisplay::Force)
        .note(p(Note::Bes, 4), Duration::QTR)
        .note(p(Note::Fis, 4), Duration::QTR)
        .note(p(Note::Dis, 4), Duration::QTR)
        .barline()
        .time_signature_change(6, 4)
        .note_with_accidental(
            p(Note::D, 5),
            dotted(DurationKind::Qtr),
            AccidentalDisplay::Force,
        )
        .tempo(TempoMark::metronome(MetronomeMark::equation(
            Met::Quarter,
            Met::Quarter,
        )))
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .note_with_accidental(p(Note::C, 5), Duration::QTR, AccidentalDisplay::Force)
        .note(p(Note::Aes, 4), Duration::HALF)
        .note(p(Note::E, 4), Duration::QTR)
        .barline()
        .cut_time_change()
        .note_with_accidental(p(Note::C, 4), Duration::HALF, AccidentalDisplay::Cautionary)
        .text_script(TextScript::above("a tempo").italic())
        .note(p(Note::F, 4), Duration::HALF)
        .end_barline();
    let layout = page(&score);
    let measures = &layout.systems[0].system.measures;
    assert!(matches!(
        measures[0]
            .layout
            .elements
            .iter()
            .find(|e| matches!(e.element, MeasureElement::TimeSignature(_)))
            .unwrap()
            .element,
        MeasureElement::TimeSignature(crate::layout::time_signature::TimeSignatureKind::CutCommon)
    ));
    let contents = score.clone().build_measure_contents().unwrap();
    assert_eq!(
        contents
            .iter()
            .map(|m| m.meta.actual_length)
            .collect::<Vec<_>>(),
        [
            MeasureLength::new(1, 1),
            MeasureLength::new(3, 2),
            MeasureLength::new(3, 2),
            MeasureLength::new(1, 1)
        ]
    );
    assert_eq!(
        contents[1]
            .events
            .iter()
            .filter_map(|e| match e {
                MeasureEvent::Note(n) => Some(n.staff_position),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [5, 4, 6, 4, 1, -1]
    ); // C5 B4 D5 Bb4 F#4 D#4 in treble clef.
    let svg = score.render_svg();
    let f = bravura_font();
    assert_eq!(glyphs(&svg, &f, Glyph::TimeSigCutCommon).len(), 2);
    assert_eq!(glyphs(&svg, &f, Glyph::MetNoteQuarterUp).len(), 2);
    let equals = text(&svg, "=");
    let quarters = glyphs(&svg, &f, Glyph::MetNoteQuarterUp);
    assert!(quarters[0].x < equals.x && equals.x < quarters[1].x);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalParensLeft).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalParensRight).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalNatural).len(), 4);
}

/// mn-c05-m001, printed p. 52: numeric 4/4, circa 63, first manual beam, and later breath after C4.
#[test]
fn tritone_melody_no_invented_cautionary() {
    let score = ScoreBuilder::new()
        .time_signature(4, 4)
        .note(p(Note::B, 3), Duration::QTR)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(Met::Quarter, 63)
                .approx()
                .parenthesized(),
        ))
        .note(p(Note::F, 4), Duration::QTR)
        .note(p(Note::B, 3), Duration::HALF)
        .barline()
        .rest(Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::B, 3), Duration::EIGHTH)
        .note(p(Note::F, 4), Duration::EIGHTH)
        .note(p(Note::Ges, 4), Duration::EIGHTH)
        .end_beam()
        .note(p(Note::C, 5), Duration::QTR)
        .begin_beam()
        .note(p(Note::Ges, 4), Duration::EIGHTH)
        .note(p(Note::C, 5), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let svg = score.clone().render_svg();
    let f = bravura_font();
    assert_eq!(text(&svg, "= c. 63)").anchor, "start");
    assert_eq!(glyphs(&svg, &f, Glyph::MetNoteQuarterUp).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::Rest8th).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalParensLeft).len(), 0);
    assert_eq!(note_positions(&score).len(), 9);
    // Same source, bar 5: ges'2 c'4 \\breathe aes'4.
    let breath = ScoreBuilder::new()
        .time_signature(4, 4)
        .note(p(Note::Ges, 4), Duration::HALF)
        .note(p(Note::C, 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(p(Note::Aes, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(glyphs(&breath, &f, Glyph::BreathMarkComma).len(), 1);
}

/// mn-c06-m003, printed p. 60: dashed bracket starts on D2, crosses the barline, ends C2.
#[test]
fn minor_sixth_meter_changes_bracket_below() {
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .common_time()
        .note(p(Note::G, 2), Duration::HALF)
        .note(p(Note::E, 2), Duration::QTR)
        .note(p(Note::Cis, 2), Duration::QTR)
        .barline()
        .time_signature_change(2, 4)
        .note(p(Note::D, 2), Duration::QTR)
        .analysis_bracket_start(AnalysisBracketSpec::new(
            AnalysisBracketStyle::Dashed,
            Placement::Below,
        ))
        .note_with_accidental(p(Note::B, 1), Duration::QTR, AccidentalDisplay::Force)
        .barline()
        .time_signature_change(4, 4)
        .note(p(Note::C, 2), Duration::QTR)
        .analysis_bracket_end()
        .rest(Duration::QTR)
        .note(p(Note::B, 1), Duration::QTR)
        .note(p(Note::C, 2), Duration::QTR)
        .end_barline();
    assert_eq!(&staff_positions(&score)[0..3], [0, -2, -4]); // G2 E2 C#2 on bass staff.
    let xs = note_positions(&score);
    let svg = score.render_svg();
    let bracket = lines(&svg)
        .into_iter()
        .find(|l| l.dashed && (l.y1 - l.y2).abs() < 1e-6 && (l.x1 - xs[3]).abs() < 1e-6)
        .expect("source bracket");
    assert!(bracket.x2 > xs[5] && bracket.y1 > 4.0 * bravura_font().engraving_config().staff_space);
    assert_eq!(glyphs(&svg, &bravura_font(), Glyph::TimeSigCommon).len(), 1);
    assert_eq!(glyphs(&svg, &bravura_font(), Glyph::AccidentalNatural).len(), 1);
}

/// mn-c01-h007, printed p. 24: stemless eighth heads under solid chord-to-chord / chord-to-note brackets.
#[test]
fn chord_series_seven_keeps_two_distinct_bracket_endpoints() {
    let score = ScoreBuilder::new()
        .hidden_time_signature(1, 4)
        .stemless()
        .chord(vec![p(Note::Cis, 4), p(Note::Ees, 4)], Duration::EIGHTH)
        .analysis_bracket_start(AnalysisBracketSpec::new(
            AnalysisBracketStyle::Solid,
            Placement::Below,
        ))
        .chord(vec![p(Note::D, 4), p(Note::E, 4)], Duration::EIGHTH)
        .analysis_bracket_end()
        .barline()
        .chord(vec![p(Note::D, 4), p(Note::Ees, 4)], Duration::EIGHTH)
        .analysis_bracket_start(AnalysisBracketSpec::new(
            AnalysisBracketStyle::Solid,
            Placement::Below,
        ))
        .note_with_accidental(p(Note::E, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .analysis_bracket_end()
        .end_barline();
    let xs = note_positions(&score);
    let svg = score.clone().render_svg();
    let below: Vec<_> = lines(&svg)
        .into_iter()
        .filter(|l| {
            !l.dashed
                && l.y1 == l.y2
                && [xs[0], xs[2]].iter().any(|x| (l.x1 - x).abs() < 1e-6)
                && l.y1 > 1000.0
        })
        .collect();
    assert_eq!(below.len(), 2);
    assert!(below[0].x2 > xs[1] && below[1].x2 > xs[3]);
    let heads = glyphs(&svg, &bravura_font(), Glyph::NoteheadBlack);
    assert_eq!(heads.len(), 7);
    assert!(glyphs(&svg, &bravura_font(), Glyph::TimeSig1).is_empty());
    assert!(page(&score).systems[0]
        .system
        .measures
        .iter()
        .flat_map(|m| &m.layout.elements)
        .filter_map(|e| match &e.element {
            MeasureElement::Chord(c) => Some(c.annotations.stem),
            MeasureElement::Note(n) => Some(n.annotations.stem),
            _ => None,
        })
        .all(|stem| stem == StemVisibility::Hidden));
}

/// mn-c12-r009, printed p. 111: bass→alto, hidden accidental, beamed graces, broad mark and slashed grace.
#[test]
fn schoenberg_marks_grace_and_clef() {
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 8)
        .accidental_policy(AccidentalPolicy::Forget)
        .rest(Duration::SIXTEENTH)
        .note(p(Note::D, 2), Duration::EIGHTH)
        .note_with_accidental(
            p(Note::Cis, 3),
            Duration::SIXTEENTH,
            AccidentalDisplay::Hide,
        )
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .note(p(Note::Bis, 2), Duration::EIGHTH)
        .barline()
        .clef_change_after_barline(Clef::Alto)
        .rest(Duration::SIXTEENTH)
        .note(p(Note::Bes, 2), Duration::SIXTEENTH)
        .note(p(Note::Des, 3), Duration::EIGHTH)
        .tie()
        .articulation_mark(ArticulationMark::broad_mark().above())
        .note(p(Note::Des, 3), Duration::SIXTEENTH)
        .note(p(Note::Aes, 2), Duration::SIXTEENTH)
        .note_with_accidental(
            p(Note::Aes, 2),
            Duration::SIXTEENTH,
            AccidentalDisplay::Hide,
        )
        .note_with_accidental(p(Note::D, 2), Duration::SIXTEENTH, AccidentalDisplay::Force)
        .barline()
        .clef_change_after_barline(Clef::Bass)
        .note_with_accidental(p(Note::D, 3), Duration::QTR, AccidentalDisplay::Force)
        .note_with_accidental(
            p(Note::C, 2),
            dotted(DurationKind::Eighth),
            AccidentalDisplay::Force,
        )
        .grace_notes(
            GraceNotes::new(GraceNoteKind::Appoggiatura)
                .note(p(Note::Aes, 1), Duration::EIGHTH)
                .note(p(Note::Ees, 1), Duration::EIGHTH),
        )
        .note(p(Note::Eis, 3), Duration::SIXTEENTH)
        .barline()
        .rest(Duration::SIXTEENTH)
        .note_with_accidental(p(Note::F, 2), Duration::SIXTEENTH, AccidentalDisplay::Force)
        .note(p(Note::F, 2), Duration::SIXTEENTH)
        .note_with_accidental(p(Note::G, 2), Duration::SIXTEENTH, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::G, 2), Duration::QTR, AccidentalDisplay::Force)
        .grace_notes(
            GraceNotes::new(GraceNoteKind::Acciaccatura)
                .note(p(Note::F, 2), Duration::EIGHTH)
                .slur(),
        )
        .end_barline();
    assert_eq!(&staff_positions(&score)[0..3], [-3, 3, 3]); // D2, C#3, C#3 in bass.
    let contents = score.clone().build_measure_contents().unwrap();
    let first: Vec<_> = contents[0]
        .events
        .iter()
        .filter_map(|e| match e {
            MeasureEvent::Note(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(first[1].accidental, None); // source's hidden C#3.
    assert_eq!(first[2].accidental.unwrap().glyph, Glyph::AccidentalSharp); // forget style repeats the sign.
    let sixth = contents[2]
        .events
        .iter()
        .find_map(|e| match e {
            MeasureEvent::Note(n) if n.annotations.grace_group.is_some() => {
                n.annotations.grace_group.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert!(sixth.is_beamed());
    assert_eq!(
        sixth
            .notes
            .iter()
            .map(|n| n.staff_position)
            .collect::<Vec<_>>(),
        [-6, -9]
    ); // A♭1 E♭1.
    let eighth = contents[3]
        .events
        .iter()
        .find_map(|e| match e {
            MeasureEvent::Note(n) if n.annotations.grace_group.is_some() => {
                n.annotations.grace_group.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(eighth.kind, GraceNoteKind::Acciaccatura);
    assert!(eighth.slur);
    let svg = score.render_svg();
    let f = bravura_font();
    assert_eq!(glyphs(&svg, &f, Glyph::CClefChange).len(), 1);
    assert_eq!(
        svg.matches("<polygon ").count(),
        1,
        "the two unslashed eighth graces share a beam"
    );
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalNatural).len(), 6);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalFlat).len(), 6); // Bes, two Des, Aes, grace Aes/Ees; two other Aes hidden.
    let broad = glyphs(&svg, &f, Glyph::ArticTenutoAbove);
    assert_eq!(broad.len(), 1);
    assert!(svg.contains("scale(2.5,1)"));
    let graces = glyphs(&svg, &f, Glyph::NoteheadBlack)
        .into_iter()
        .filter(|g| (g.scale - 0.65).abs() < 0.1)
        .count();
    assert!(graces >= 3, "two beamed graces and one slashed grace");
}

/// mn-c04-r005, printed p. 43: parenthesized fermata; two lyric lanes and hidden hyphen.
#[test]
fn blomdahl_two_verses_parenthesized_fermata() {
    let score = ScoreBuilder::new()
        .time_signature(3, 4)
        .rest(Duration::HALF)
        .tempo(
            TempoMark::metronome(
                MetronomeMark::bpm(Met::Quarter, 56)
                    .approx()
                    .parenthesized(),
            )
            .with_text_after(TempoText::upright("( Agitato)")),
        )
        .rest(Duration::EIGHTH)
        .note(p(Note::Ges, 4), Duration::EIGHTH)
        .tie()
        .lyric_verse(1, LyricSyllable::with_extender("Å"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Oh,"), LyricStyle::Italic)
        .barline()
        .note_with_accidental(p(Note::Ges, 4), Duration::QTR, AccidentalDisplay::Force)
        .lyric_verse(1, LyricSyllable::skip(), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .note(p(Note::Ges, 4), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("det"), LyricStyle::Upright)
        .lyric_verse(
            2,
            LyricSyllable::with_hidden_hyphen("die"),
            LyricStyle::Italic,
        )
        .note(p(Note::F, 4), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("ta"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("ser"), LyricStyle::Italic)
        .note(p(Note::G, 4), dotted(DurationKind::Eighth))
        .note(p(Note::Fis, 4), Duration::SIXTEENTH)
        .barline()
        .note_with_accidental(p(Note::C, 5), Duration::QTR, AccidentalDisplay::Force)
        .articulation_mark(
            ArticulationMark::from(Articulation::Fermata)
                .above()
                .parenthesized(),
        )
        .rest(Duration::HALF)
        .end_barline();
    let svg = score.render_svg();
    let f = bravura_font();
    let det = text(&svg, "det");
    let die = text(&svg, "die");
    assert!(die.y > det.y && die.style == "italic" && det.style == "normal");
    assert!(text(&svg, "ser").x > die.x);
    let hyphens: Vec<_> = texts(&svg)
        .into_iter()
        .filter(|t| t.content == "-")
        .collect();
    assert_eq!(hyphens.len(), 1, "only Swedish det–ta prints a hyphen");
    assert_eq!(glyphs(&svg, &f, Glyph::FermataAbove).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalParensLeft).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalParensRight).len(), 1);
}

/// mn-c01-m006, printed p. 23: pickup, half=c.100 stacked over Alla gavotta; editorial comma.
#[test]
fn gavotta_stacked_tempo_and_parenthesized_breath() {
    let score = ScoreBuilder::new()
        .common_time()
        .partial(Duration::HALF)
        .note(p(Note::F, 4), Duration::QTR)
        .tempo(
            TempoMark::metronome(MetronomeMark::bpm(Met::Half, 100).approx().parenthesized())
                .with_text_below("Alla gavotta"),
        )
        .note(p(Note::Bes, 4), Duration::QTR)
        .barline()
        .note(p(Note::A, 4), dotted(DurationKind::Qtr))
        .note(p(Note::Gis, 4), Duration::EIGHTH)
        .note(p(Note::Fis, 4), Duration::EIGHTH)
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .note(p(Note::Dis, 4), Duration::EIGHTH)
        .note(p(Note::Eis, 4), Duration::EIGHTH)
        .barline()
        .note_with_accidental(p(Note::C, 4), Duration::HALF, AccidentalDisplay::Force)
        .breath_mark(BreathMark::Comma)
        .note(p(Note::Bes, 4), Duration::EIGHTH)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .note(p(Note::F, 4), Duration::QTR)
        .end_barline();
    let svg = score.clone().render_svg();
    let f = bravura_font();
    let half = glyph(&svg, &f, Glyph::MetNoteHalfUp);
    let words = text(&svg, "Alla gavotta");
    assert!(half.y < words.y && words.weight == "bold");
    assert_eq!(glyphs(&svg, &f, Glyph::TimeSigCommon).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::BreathMarkComma).len(), 1);
    let meta = &page(&score).systems[0].system.measures[0].layout;
    assert!(meta
        .elements
        .iter()
        .any(|e| matches!(e.element, MeasureElement::Note(_))));
    let editorial = ScoreBuilder::new()
        .common_time()
        .note(p(Note::Ces, 4), Duration::EIGHTH)
        .note(p(Note::Des, 4), Duration::EIGHTH)
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .note_with_accidental(p(Note::E, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::F, 4), Duration::QTR)
        .parenthesized_breath_mark(BreathMark::Comma)
        .note(p(Note::Bes, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(glyphs(&editorial, &f, Glyph::BreathMarkComma).len(), 1);
    assert_eq!(glyphs(&editorial, &f, Glyph::AccidentalParensLeft).len(), 1);
    assert_eq!(
        glyphs(&editorial, &f, Glyph::AccidentalParensRight).len(),
        1
    );
}

/// mn-c04-r006, printed p. 43: eighth-rest pickup, dotted notes, 32nd beam, E double-flat and B natural chord.
#[test]
fn bartok_pickup_rest_and_thirty_second_double_flat() {
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(9, 8)
        .partial(dotted(DurationKind::Qtr))
        .rest(Duration::EIGHTH)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(MetronomeUnit::dotted(Met::Quarter), 58).parenthesized(),
        ))
        .note(p(Note::E, 2), Duration::SIXTEENTH)
        .note(p(Note::A, 2), Duration::SIXTEENTH)
        .note(p(Note::Bes, 2), Duration::SIXTEENTH)
        .note(p(Note::Ees, 3), Duration::SIXTEENTH)
        .barline()
        .note(p(Note::D, 3), Duration::QTR)
        .tuplet_ratio(
            3,
            2,
            vec![
                (p(Note::C, 3), Duration::SIXTEENTH),
                (p(Note::D, 3), Duration::SIXTEENTH),
                (p(Note::C, 3), Duration::SIXTEENTH),
            ],
        )
        .note(p(Note::Bes, 2), Duration::QTR)
        .tie()
        .begin_beam()
        .note(
            p(Note::Bes, 2),
            Duration::new(DurationKind::ThirtySecond, 0),
        )
        .note(
            p(Note::Aes, 2),
            Duration::new(DurationKind::ThirtySecond, 0),
        )
        .note(
            p(Note::Ges, 2),
            Duration::new(DurationKind::ThirtySecond, 0),
        )
        .note(
            p(Note::Fes, 2),
            Duration::new(DurationKind::ThirtySecond, 0),
        )
        .end_beam()
        .note(p(Note::Eeses, 2), dotted(DurationKind::Eighth))
        .note(p(Note::Des, 2), Duration::SIXTEENTH)
        .note(p(Note::Aes, 1), Duration::EIGHTH)
        .barline()
        .chord_with_accidentals(
            vec![p(Note::B, 1), p(Note::Fis, 2)],
            dotted(DurationKind::Half),
            vec![AccidentalDisplay::Force, AccidentalDisplay::Auto],
        )
        .barline_style(BarlineStyle::Invisible);
    let contents = score.clone().build_measure_contents().unwrap();
    assert_eq!(
        contents
            .iter()
            .map(|m| m.meta.actual_length)
            .collect::<Vec<_>>(),
        [
            MeasureLength::new(3, 8),
            MeasureLength::new(9, 8),
            MeasureLength::new(3, 4)
        ]
    );
    assert!(contents[0].meta.anacrusis);
    assert!(contents[2].meta.is_incomplete()); // source ends without a final bar check.
    assert_eq!(
        contents[1]
            .events
            .iter()
            .filter_map(|e| match e {
                MeasureEvent::Note(n) if n.duration_log2 == 5 => Some(n.staff_position),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [2, 1, 0, -1]
    ); // four 32nds Bb2 Ab2 Gb2 Fb2.
    let svg = score.clone().render_svg();
    let f = bravura_font();
    assert!(glyphs(&svg, &f, Glyph::MetAugmentationDot)
        .iter()
        .any(|g| (g.y - glyph(&svg, &f, Glyph::MetNoteQuarterUp).y).abs() < 1e-6));
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalDoubleFlat).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalNatural).len(), 1); // B1!, not a cancellation from another octave.
    assert_eq!(glyphs(&svg, &f, Glyph::Rest8th).len(), 1);
    let layout = page(&score);
    let second = &layout.systems[0].system.measures[1];
    assert!(second
        .layout
        .elements
        .iter()
        .any(|e| matches!(e.element, MeasureElement::GroupMark(_))));
    assert_eq!(
        svg.matches("<polygon ").count(),
        5,
        "two tuplet sixteenth beam levels plus three 32nd beam levels"
    );
    assert!(glyphs(&svg, &f, Glyph::NoteheadBlack).len() >= 10);
}

/// mn-c09-p001, printed p. 82: Sing D4, imagine parenthesized stemless G4, sing C5.
#[test]
fn minor_seventh_imagined_note_is_head_only() {
    let score = ScoreBuilder::new()
        .cadenza_on()
        .note(p(Note::D, 4), Duration::QTR)
        .text_script(TextScript::above("Sing"))
        .note(p(Note::G, 4), Duration::QTR)
        .hide_stem()
        .parenthesize()
        .text_script(TextScript::above("Imagine"))
        .note(p(Note::C, 5), Duration::QTR)
        .text_script(TextScript::above("Sing"))
        .barline_style(BarlineStyle::Double)
        .note(p(Note::Des, 5), Duration::QTR)
        .text_script(TextScript::above("Sing"))
        .note(p(Note::Aes, 4), Duration::QTR)
        .hide_stem()
        .parenthesize()
        .text_script(TextScript::above("Imagine"))
        .note(p(Note::Ees, 4), Duration::QTR)
        .text_script(TextScript::above("Sing"))
        .barline_style(BarlineStyle::Double);
    assert_eq!(staff_positions(&score), [-1, 2, 5, 6, 3, 0]);
    let svg = score.clone().render_svg();
    let f = bravura_font();
    assert_eq!(
        texts(&svg).iter().filter(|t| t.content == "Sing").count(),
        4
    );
    assert_eq!(
        texts(&svg)
            .iter()
            .filter(|t| t.content == "Imagine")
            .count(),
        2
    );
    assert_eq!(glyphs(&svg, &f, Glyph::NoteheadParenthesisLeft).len(), 2);
    assert_eq!(glyphs(&svg, &f, Glyph::NoteheadParenthesisRight).len(), 2);
    assert!(texts(&svg)
        .iter()
        .filter(|t| t.content == "Imagine")
        .zip([note_positions(&score)[1], note_positions(&score)[4]])
        .all(|(t, x)| t.x >= x));
    assert_eq!(glyphs(&svg, &f, Glyph::TimeSig4).len(), 0);
}

/// mn-c10-p001, printed p. 87: unmetered stemless C4–B4–C5–Db4–C4 formula.
#[test]
fn major_seventh_formula_keeps_pitches_without_meter_or_stems() {
    let score = ScoreBuilder::new()
        .clef(Clef::Treble)
        .cadenza_on()
        .stemless()
        .note(p(Note::C, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::C, 5), Duration::QTR)
        .note(p(Note::Des, 4), Duration::QTR)
        .note(p(Note::C, 4), Duration::QTR)
        .barline_style(BarlineStyle::Double);
    assert_eq!(staff_positions(&score), [-2, 4, 5, -1, -2]);
    let xs = note_positions(&score);
    let svg = score.clone().render_svg();
    let f = bravura_font();
    assert_eq!(xs.len(), 5);
    assert!(xs.windows(2).all(|w| w[1] > w[0]));
    assert_eq!(glyphs(&svg, &f, Glyph::NoteheadBlack).len(), 5);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalFlat).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::TimeSig4).len(), 0);
    assert!(page(&score).systems[0].system.measures[0]
        .layout
        .elements
        .iter()
        .filter_map(|e| match &e.element {
            MeasureElement::Note(n) => Some(n.annotations.stem),
            _ => None,
        })
        .all(|s| s == StemVisibility::Hidden));
}

/// mn-c01-h001, printed p. 24: nine single-bar sonorities; B♭ to C♭ then A-C♮-E.
#[test]
fn chord_series_one_reinstates_natural() {
    let score = ScoreBuilder::new()
        .hidden_time_signature(1, 4)
        .stemless()
        .chord(
            vec![p(Note::Bes, 3), p(Note::Ces, 4), p(Note::E, 4)],
            Duration::QTR,
        )
        .barline()
        .chord_with_accidentals(
            vec![p(Note::A, 3), p(Note::C, 4), p(Note::E, 4)],
            Duration::QTR,
            vec![
                AccidentalDisplay::Auto,
                AccidentalDisplay::Force,
                AccidentalDisplay::Auto,
            ],
        )
        .end_barline();
    let svg = score.clone().render_svg();
    let f = bravura_font();
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalFlat).len(), 2);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalNatural).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::NoteheadBlack).len(), 6);
    assert!(glyphs(&svg, &f, Glyph::TimeSig1).is_empty());
    assert!(note_positions(&score)[1] > note_positions(&score)[0]);
}

/// mn-c05-h001, printed p. 55: cadenza chord series marked 1–7, forget policy, C♮ reinstated.
#[test]
fn tritone_chord_series_numbers_stay_over_sonorities() {
    let score = ScoreBuilder::new()
        .cadenza_on()
        .stemless()
        .accidental_policy(AccidentalPolicy::Forget)
        .note(p(Note::C, 4), Duration::QTR)
        .text_script(TextScript::above("1"))
        .barline()
        .chord(vec![p(Note::C, 4), p(Note::Fis, 4)], Duration::QTR)
        .text_script(TextScript::above("2"))
        .barline()
        .chord(
            vec![p(Note::D, 4), p(Note::G, 4), p(Note::Cis, 5)],
            Duration::QTR,
        )
        .text_script(TextScript::above("5"))
        .barline()
        .chord_with_accidentals(
            vec![p(Note::D, 4), p(Note::G, 4), p(Note::C, 5)],
            Duration::QTR,
            vec![
                AccidentalDisplay::Auto,
                AccidentalDisplay::Auto,
                AccidentalDisplay::Force,
            ],
        )
        .text_script(TextScript::above("6"))
        .barline()
        .chord(
            vec![p(Note::E, 4), p(Note::Gis, 4), p(Note::B, 4)],
            Duration::QTR,
        )
        .text_script(TextScript::above("7"))
        .barline_style(BarlineStyle::Double);
    let xs = note_positions(&score);
    let svg = score.render_svg();
    let f = bravura_font();
    for (label, x) in [
        ("1", xs[0]),
        ("2", xs[1]),
        ("5", xs[2]),
        ("6", xs[3]),
        ("7", xs[4]),
    ] {
        assert!(
            (text(&svg, label).x - x).abs() < 1e-6,
            "event number {label} follows its chord"
        );
    }
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalNatural).len(), 1);
    assert_eq!(glyphs(&svg, &f, Glyph::AccidentalSharp).len(), 3);
    assert!(glyphs(&svg, &f, Glyph::TimeSig4).is_empty());
}
