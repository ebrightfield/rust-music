//! Source-derived score → measure → layout → SVG regressions, not LilyPond parsing.
//! Each fixture cites its transcribed packet and printed Modus Novus page.

use super::*;
use crate::layout::barline::BarlineStyle;
use crate::layout::dynamics::Dynamic;
use crate::layout::group::{scan_groups, TupletSpec};
use crate::layout::lyric::{LyricStyle, LyricSyllable};
use crate::layout::measure::{MeasureElement, NoteEvent};
use crate::layout::placement::Placement;
use crate::layout::tempo::{MetronomeMark, MetronomeNoteKind, MetronomeUnit};
use crate::layout::text_script::TextScript;
use crate::layout::tuplet::TupletPlacement;
use music::note::note::Note;
use smufl::Glyph;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn notes(events: &[MeasureEvent]) -> Vec<(i8, i8, u8, Option<Glyph>)> {
    events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(n) => Some((
                n.staff_position,
                n.duration_log2,
                n.dots,
                n.accidental.map(|a| a.glyph),
            )),
            _ => None,
        })
        .collect()
}

fn rhythmic(events: &[MeasureEvent]) -> Vec<&'static str> {
    events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(_) => Some("note"),
            MeasureEvent::Rest(_) => Some("rest"),
            _ => None,
        })
        .collect()
}

fn note_at(events: &[MeasureEvent], n: usize) -> &NoteEvent {
    events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note),
            _ => None,
        })
        .nth(n)
        .unwrap()
}

fn layout_of(score: ScoreBuilder) -> crate::layout::page::PageLayout {
    score.clone().page_layout(250.0).unwrap().unwrap()
}

fn glyph_count(svg: &str, glyph: Glyph) -> usize {
    let outline = bravura_font().glyph_outline(glyph).unwrap().path_data;
    svg.matches(&format!("<path d=\"{outline}\"")).count()
}

fn lyric_y(svg: &str, syllable: &str) -> f64 {
    let line = svg
        .lines()
        .find(|line| line.contains(&format!(">{syllable}</text>")))
        .unwrap();
    line.split(" y=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn c04_r035_alto_pickup_c_flat_octave_forced_natural_and_slurred_beam() {
    // mn-c04-r035, printed p. 48 / PDF 39: pickup bes8 | ces'8[( es') d'] | c'!8[( a!) bes].
    let score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(3, 8)
        .accidental_policy(AccidentalPolicy::Forget)
        .partial(Duration::EIGHTH)
        .note(p(Note::Bes, 3), Duration::EIGHTH)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(MetronomeUnit::dotted(MetronomeNoteKind::Quarter), 84)
                .approx()
                .parenthesized(),
        ))
        .barline()
        .begin_beam()
        .note(p(Note::Ces, 4), Duration::EIGHTH)
        .slur_start()
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .slur_end()
        .note(p(Note::D, 4), Duration::EIGHTH)
        .end_beam()
        .barline()
        .begin_beam()
        .note_with_accidental(p(Note::C, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_start()
        .note_with_accidental(p(Note::A, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_end()
        .note(p(Note::Bes, 3), Duration::EIGHTH)
        .end_beam()
        .barline_style(BarlineStyle::Single);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(contents.len(), 3);
    assert_eq!(
        (
            contents[0].meta.number,
            contents[0].meta.anacrusis,
            contents[0].meta.actual_length
        ),
        (0, true, MeasureLength::new(1, 8))
    );
    assert_eq!(contents[1].meta.actual_length, MeasureLength::new(3, 8));
    assert_eq!(
        notes(&contents[1].events),
        [
            (4, 3, 0, Some(Glyph::AccidentalFlat)),
            (6, 3, 0, Some(Glyph::AccidentalFlat)),
            (5, 3, 0, None)
        ]
    );
    assert_eq!(
        p(Note::Ces, 4).midi_note,
        59,
        "C-flat is written C4 but sounds B3"
    );
    assert_eq!(
        notes(&contents[2].events),
        [
            (4, 3, 0, Some(Glyph::AccidentalNatural)),
            (2, 3, 0, Some(Glyph::AccidentalNatural)),
            (3, 3, 0, Some(Glyph::AccidentalFlat))
        ]
    );
    assert!(note_at(&contents[1].events, 0).annotations.slur_start);
    assert!(note_at(&contents[1].events, 1).annotations.slur_end);
    let page = layout_of(score.clone());
    let elements = &page.systems[0].system.measures[1].layout.elements;
    let groups = scan_groups(elements.iter().map(|e| &e.element));
    assert_eq!((groups.beams.len(), groups.beams[0].members.len()), (1, 3));
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::CClef), 1);
    assert_eq!(glyph_count(&svg, Glyph::AccidentalNatural), 2);
    assert!(svg.contains("c. 84"), "dotted-quarter approximate tempo");
    assert_eq!(glyph_count(&svg, Glyph::MetAugmentationDot), 1);
}

#[test]
fn c04_r038_dashed_divisions_change_meter_without_resetting_forget_accidentals() {
    // mn-c04-r038, printed p. 48 / PDF 39: 8/8 pickup a8(pp) | bes8 cis'4 ! c'!4 b!8) ! r8 a8( |
    // 12/8 bes8[ cis' d'] ! es'8 c'!4 ! b!4 bes8) ! r8 cis'8[( e'!]) | ... final a4, no barline.
    let score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(8, 8)
        .accidental_policy(AccidentalPolicy::Forget)
        .partial(Duration::EIGHTH)
        .note(p(Note::A, 3), Duration::EIGHTH)
        .tempo(TempoMark::metronome(
            MetronomeMark::range(MetronomeNoteKind::Eighth, 116, 112)
                .approx()
                .parenthesized(),
        ))
        .dynamic(Dynamic::Pp)
        .slur_start()
        .barline()
        .note(p(Note::Bes, 3), Duration::EIGHTH)
        .note(p(Note::Cis, 4), Duration::QTR)
        .inline_barline(BarlineStyle::Dashed)
        .note_with_accidental(p(Note::C, 4), Duration::QTR, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::B, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_end()
        .inline_barline(BarlineStyle::Dashed)
        .rest(Duration::EIGHTH)
        .note(p(Note::A, 3), Duration::EIGHTH)
        .slur_start()
        .barline()
        .time_signature_change(12, 8)
        .begin_beam()
        .note(p(Note::Bes, 3), Duration::EIGHTH)
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .note(p(Note::D, 4), Duration::EIGHTH)
        .end_beam()
        .inline_barline(BarlineStyle::Dashed)
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .note_with_accidental(p(Note::C, 4), Duration::QTR, AccidentalDisplay::Force)
        .inline_barline(BarlineStyle::Dashed)
        .note_with_accidental(p(Note::B, 3), Duration::QTR, AccidentalDisplay::Force)
        .note(p(Note::Bes, 3), Duration::EIGHTH)
        .slur_end()
        .inline_barline(BarlineStyle::Dashed)
        .rest(Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .slur_start()
        .note_with_accidental(p(Note::E, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_end()
        .end_beam()
        .barline()
        .system_break()
        .time_signature_change(8, 8)
        .begin_beam()
        .note(p(Note::Dis, 4), Duration::EIGHTH)
        .slur_start()
        .note_with_accidental(p(Note::D, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::C, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam()
        .inline_barline(BarlineStyle::Dashed)
        .begin_beam()
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .note_with_accidental(p(Note::B, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_end()
        .end_beam()
        .rest(Duration::EIGHTH)
        .inline_barline(BarlineStyle::Dashed)
        .begin_beam()
        .note_with_accidental(p(Note::C, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_start()
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .end_beam()
        .barline()
        .time_signature_change(7, 8)
        .note(p(Note::D, 4), Duration::QTR)
        .begin_beam()
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .end_beam()
        .inline_barline(BarlineStyle::Dashed)
        .note_with_accidental(p(Note::C, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::Bes, 3), Duration::QTR)
        .slur_end()
        .barline()
        .note(p(Note::A, 3), Duration::QTR)
        .barline_style(BarlineStyle::Invisible);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(
        contents.iter().map(|m| m.meta.number).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );
    assert_eq!(
        contents
            .iter()
            .map(|m| m.meta.meter.as_ref())
            .collect::<Vec<_>>(),
        [
            Some(&TimeSignatureKind::Numeric {
                numerator: 8,
                denominator: 8
            }),
            Some(&TimeSignatureKind::Numeric {
                numerator: 8,
                denominator: 8
            }),
            Some(&TimeSignatureKind::Numeric {
                numerator: 12,
                denominator: 8
            }),
            Some(&TimeSignatureKind::Numeric {
                numerator: 8,
                denominator: 8
            }),
            Some(&TimeSignatureKind::Numeric {
                numerator: 7,
                denominator: 8
            }),
            Some(&TimeSignatureKind::Numeric {
                numerator: 7,
                denominator: 8
            })
        ]
    );
    assert_eq!(
        (
            contents[1].meta.actual_length,
            contents[2].meta.actual_length
        ),
        (MeasureLength::new(1, 1), MeasureLength::new(3, 2))
    );
    assert_eq!(
        notes(&contents[1].events),
        [
            (3, 3, 0, Some(Glyph::AccidentalFlat)),
            (4, 2, 0, Some(Glyph::AccidentalSharp)),
            (4, 2, 0, Some(Glyph::AccidentalNatural)),
            (3, 3, 0, Some(Glyph::AccidentalNatural)),
            (2, 3, 0, None)
        ]
    );
    assert_eq!(
        contents[1]
            .events
            .iter()
            .filter(|event| matches!(event, MeasureEvent::Barline(BarlineStyle::Dashed)))
            .count(),
        2
    );
    assert_eq!(
        contents[2]
            .events
            .iter()
            .filter(|event| matches!(event, MeasureEvent::Barline(BarlineStyle::Dashed)))
            .count(),
        3
    );
    assert_eq!(contents[3].meta.actual_length, MeasureLength::new(1, 1));
    assert_eq!(contents[4].meta.actual_length, MeasureLength::new(7, 8));
    assert_eq!(contents[5].meta.actual_length, MeasureLength::new(1, 4));
    assert_eq!(
        contents[3]
            .events
            .iter()
            .filter(|event| matches!(event, MeasureEvent::Barline(BarlineStyle::Dashed)))
            .count(),
        2
    );
    assert_eq!(
        contents[4]
            .events
            .iter()
            .filter(|event| matches!(event, MeasureEvent::Barline(BarlineStyle::Dashed)))
            .count(),
        1
    );
    assert!(matches!(contents[5].barline, BarlineStyle::Invisible));
    let page = layout_of(score.clone());
    let elements = &page.systems[0].system.measures[1].layout.elements;
    let xs: Vec<_> = elements
        .iter()
        .filter_map(|e| match &e.element {
            MeasureElement::Note(_) | MeasureElement::Rest(_) => Some((e.x, false)),
            MeasureElement::Barline(BarlineStyle::Dashed) => Some((e.x, true)),
            _ => None,
        })
        .collect();
    assert!(
        xs.windows(2).all(|w| w[0].0 < w[1].0),
        "dashed bars occupy ordered columns between notes"
    );
    assert!(xs[2].1 && xs[5].1, "two mid-measure dashed divisions");
    let svg = score.render_svg();
    assert_eq!(
        glyph_count(&svg, Glyph::CClef),
        page.systems.len(),
        "alto clef opens each system"
    );
    assert_eq!(glyph_count(&svg, Glyph::AccidentalNatural), 10);
    assert!(svg.contains("c. 116-112"), "source's eighth-note BPM range");
}

#[test]
fn c08_r009_alto_tied_tenuto_slurs_and_forced_a_natural() {
    // mn-c08-r009, printed p. 72 / PDF 63: d'2~ d'4-- cis'4( gis4) | ... a'!2( ees'4) r4 r4.
    let score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(5, 4)
        .note(p(Note::D, 4), Duration::HALF)
        .tempo(TempoMark::text("Andante con moto"))
        .tie()
        .note(p(Note::D, 4), Duration::QTR)
        .articulation(Articulation::Tenuto)
        .note(p(Note::Cis, 4), Duration::QTR)
        .slur_start()
        .note(p(Note::Gis, 3), Duration::QTR)
        .slur_end()
        .barline()
        .note_with_accidental(p(Note::A, 4), Duration::HALF, AccidentalDisplay::Force)
        .slur_start()
        .note(p(Note::Ees, 4), Duration::QTR)
        .slur_end()
        .rest(Duration::QTR)
        .rest(Duration::QTR)
        .barline_style(BarlineStyle::Single);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(
        notes(&contents[0].events),
        [
            (5, 1, 0, None),
            (5, 2, 0, None),
            (4, 2, 0, Some(Glyph::AccidentalSharp)),
            (1, 2, 0, Some(Glyph::AccidentalSharp))
        ]
    );
    assert!(note_at(&contents[0].events, 0).annotations.tie_forward);
    assert!(!note_at(&contents[0].events, 1)
        .annotations
        .articulations
        .is_empty());
    assert!(note_at(&contents[0].events, 2).annotations.slur_start);
    assert!(note_at(&contents[0].events, 3).annotations.slur_end);
    assert_eq!(
        notes(&contents[1].events),
        [
            (9, 1, 0, Some(Glyph::AccidentalNatural)),
            (6, 2, 0, Some(Glyph::AccidentalFlat))
        ]
    );
    assert_eq!(
        rhythmic(&contents[1].events),
        ["note", "note", "rest", "rest"]
    );
    assert_eq!(contents[1].meta.actual_length, MeasureLength::new(5, 4));
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::AccidentalNatural), 1);
    assert!(svg.contains(">Andante con moto</text>"));
}

#[test]
fn c08_r011_meter_change_beamed_sixteenth_triplet_and_tied_slur() {
    // mn-c08-r011, printed p. 72 / PDF 63: r4 eis'8( a'8[ c'' e''!]) | 9/8 d''4 { c''16[( d'' c'']) } g'4.~( g'4 b'8).
    let score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(6, 8)
        .rest(Duration::QTR)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(MetronomeNoteKind::Quarter, 54).parenthesized(),
        ))
        .note(p(Note::Eis, 4), Duration::EIGHTH)
        .slur_start()
        .begin_beam()
        .note(p(Note::A, 4), Duration::EIGHTH)
        .note(p(Note::C, 5), Duration::EIGHTH)
        .note_with_accidental(p(Note::E, 5), Duration::EIGHTH, AccidentalDisplay::Force)
        .slur_end()
        .end_beam()
        .barline()
        .time_signature_change(9, 8)
        .note(p(Note::D, 5), Duration::QTR)
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::C, 5), Duration::SIXTEENTH)
        .slur_start()
        .note(p(Note::D, 5), Duration::SIXTEENTH)
        .note(p(Note::C, 5), Duration::SIXTEENTH)
        .slur_end()
        .end_beam()
        .end_tuplet()
        .note(p(Note::G, 4), Duration::new(DurationKind::Qtr, 1))
        .tie()
        .slur_start()
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::EIGHTH)
        .slur_end()
        .barline_style(BarlineStyle::Single);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(
        notes(&contents[0].events),
        [
            (6, 3, 0, Some(Glyph::AccidentalSharp)),
            (9, 3, 0, None),
            (11, 3, 0, None),
            (13, 3, 0, Some(Glyph::AccidentalNatural))
        ]
    );
    assert_eq!(contents[0].meta.actual_length, MeasureLength::new(3, 4));
    assert_eq!(contents[1].meta.actual_length, MeasureLength::new(9, 8));
    assert!(matches!(
        contents[1].events[0],
        MeasureEvent::TimeSignature(TimeSignatureKind::Numeric {
            numerator: 9,
            denominator: 8
        })
    ));
    assert!(note_at(&contents[1].events, 1).annotations.slur_start);
    assert!(note_at(&contents[1].events, 3).annotations.slur_end);
    assert!(
        note_at(&contents[1].events, 4).annotations.tie_forward
            && note_at(&contents[1].events, 4).annotations.slur_start
    );
    let page = layout_of(score.clone());
    let elements = &page.systems[0].system.measures[1].layout.elements;
    let groups = scan_groups(elements.iter().map(|e| &e.element));
    assert_eq!((groups.tuplets.len(), groups.beams.len()), (1, 1));
    assert_eq!(groups.tuplets[0].members, groups.beams[0].members);
    let onsets: Vec<_> = groups.tuplets[0]
        .members
        .iter()
        .map(|&i| groups.onsets[i])
        .collect();
    for (actual, expected) in onsets
        .iter()
        .zip([0.25, 0.25 + 1.0 / 24.0, 0.25 + 2.0 / 24.0])
    {
        assert!(
            (actual - expected).abs() < 1e-12,
            "sixteenth triplet performed onset"
        );
    }
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::TimeSig9), 1);
    assert!(
        svg.contains("= 54)"),
        "quarter-note tempo survives the leading rest"
    );
}

#[test]
fn c11_r017_midbar_alto_clef_and_b_sharp_written_octave() {
    // mn-c11-r017, printed p. 99 / PDF 90, fragment e): bass r8^"e)" e,8[ d8 fis8] alto bis8[ eis'8] | b'!8 ||.
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(3, 4)
        .rest(Duration::EIGHTH)
        .text_script(TextScript::above("e)"))
        .begin_beam()
        .note(p(Note::E, 2), Duration::EIGHTH)
        .note(p(Note::D, 3), Duration::EIGHTH)
        .note(p(Note::Fis, 3), Duration::EIGHTH)
        .end_beam()
        .clef_change(Clef::Alto)
        .begin_beam()
        .note(p(Note::Bis, 3), Duration::EIGHTH)
        .note(p(Note::Eis, 4), Duration::EIGHTH)
        .end_beam()
        .barline()
        .note_with_accidental(p(Note::B, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .barline_style(BarlineStyle::Double);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(contents[0].meta.actual_length, MeasureLength::new(3, 4));
    assert_eq!(
        p(Note::Bis, 3).midi_note,
        60,
        "B-sharp sounds C4 but stays on written B3"
    );
    assert_eq!(
        notes(&contents[0].events),
        [
            (-2, 3, 0, None),
            (4, 3, 0, None),
            (6, 3, 0, Some(Glyph::AccidentalSharp)),
            (3, 3, 0, Some(Glyph::AccidentalSharp)),
            (6, 3, 0, Some(Glyph::AccidentalSharp))
        ]
    );
    assert_eq!(
        notes(&contents[1].events),
        [(10, 3, 0, Some(Glyph::AccidentalNatural))]
    );
    assert!(matches!(
        contents[0]
            .events
            .iter()
            .find(|e| matches!(e, MeasureEvent::ClefChange(_))),
        Some(MeasureEvent::ClefChange(ClefChange {
            clef: ClefKind::Alto,
            ..
        }))
    ));
    let MeasureEvent::Rest(rest) = &contents[0].events[0] else {
        panic!("opening rest")
    };
    assert_eq!(rest.annotations.text_scripts, [TextScript::above("e)")]);
    let page = layout_of(score.clone());
    let layout = &page.systems[0].system.measures[0].layout;
    let clef_i = layout
        .elements
        .iter()
        .position(|e| matches!(e.element, MeasureElement::Clef(_)))
        .unwrap();
    let change_i = layout
        .elements
        .iter()
        .rposition(|e| matches!(e.element, MeasureElement::Clef(_)))
        .unwrap();
    assert!(change_i > clef_i);
    assert!(
        layout.elements[change_i].x
            > layout
                .elements
                .iter()
                .find(|e| matches!(e.element, MeasureElement::Note(ref n) if n.staff_position == 6))
                .unwrap()
                .x
    );
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::CClefChange), 1);
    assert!(svg.contains(">e)</text>"));
}

#[test]
fn c11_r018_dynamic_on_rest_and_slur_begins_inside_triplet() {
    // mn-c11-r018, printed p. 99 / PDF 90: r2 r4\p { e8[ f( c'] } | ais4 b2) dis4~( | dis8[ d]) cis'2 { d!8[ ees( f] }.
    let score = ScoreBuilder::new()
        .clef(Clef::Alto)
        .time_signature(4, 4)
        .rest(Duration::HALF)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(MetronomeNoteKind::Half, 152).parenthesized(),
        ))
        .rest(Duration::QTR)
        .dynamic(Dynamic::Piano)
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::F, 3), Duration::EIGHTH)
        .slur_start()
        .note(p(Note::C, 4), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .barline()
        .note(p(Note::Ais, 3), Duration::QTR)
        .note(p(Note::B, 3), Duration::HALF)
        .slur_end()
        .note(p(Note::Dis, 4), Duration::QTR)
        .tie()
        .slur_start()
        .barline()
        .begin_beam()
        .note(p(Note::Dis, 4), Duration::EIGHTH)
        .note(p(Note::D, 4), Duration::EIGHTH)
        .slur_end()
        .end_beam()
        .note(p(Note::Cis, 4), Duration::HALF)
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note_with_accidental(p(Note::D, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .slur_start()
        .note(p(Note::F, 4), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .barline_style(BarlineStyle::Final);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(contents[0].meta.actual_length, MeasureLength::new(1, 1));
    assert_eq!(
        rhythmic(&contents[0].events),
        ["rest", "rest", "note", "note", "note"]
    );
    let MeasureEvent::Rest(rest) = &contents[0].events[1] else {
        panic!("piano on quarter rest")
    };
    assert_eq!(
        rest.annotations.dynamic,
        Some(DynamicMark::Standard(Dynamic::Piano))
    );
    assert!(note_at(&contents[0].events, 1).annotations.slur_start);
    assert!(!note_at(&contents[0].events, 2).annotations.slur_end);
    assert!(note_at(&contents[1].events, 1).annotations.slur_end);
    assert_eq!(contents[2].meta.actual_length, MeasureLength::new(1, 1));
    assert_eq!(
        note_at(&contents[1].events, 2).staff_position,
        note_at(&contents[2].events, 0).staff_position
    );
    assert_eq!(
        note_at(&contents[2].events, 0).accidental,
        Some(ResolvedAccidental::plain(Glyph::AccidentalSharp))
    );
    assert!(note_at(&contents[2].events, 1).annotations.slur_end);
    let page = layout_of(score.clone());
    let groups = scan_groups(
        page.systems[0].system.measures[0]
            .layout
            .elements
            .iter()
            .map(|e| &e.element),
    );
    assert_eq!(
        (
            groups.tuplets.len(),
            groups.beams.len(),
            groups.tuplets[0].members.len()
        ),
        (1, 1, 3)
    );
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::DynamicPiano), 1);
    assert!(
        svg.contains("= 152)"),
        "half-note tempo remains on the opening rest"
    );
}

#[test]
fn c12_r002_unbeamed_rest_triplet_two_lyric_lanes_and_six_beat_invisible_end() {
    // mn-c12-r002, printed p. 109 / PDF 100: r4\p { r8 b8 b8 } d'4 cis8 cis8 | (6 beats, no meter change) ... \bar "".
    let up = TupletSpec::new(3, 2).placement(TupletPlacement::Above);
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .dynamics_placement(Placement::Above)
        .rest(Duration::QTR)
        .dynamic(Dynamic::Piano)
        .begin_tuplet(up)
        .rest(Duration::EIGHTH)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("Jag"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Ich"), LyricStyle::Italic)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("är"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("hab"), LyricStyle::Italic)
        .end_tuplet()
        .note(p(Note::D, 4), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("rädd,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Angst,"), LyricStyle::Italic)
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("So"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Sodo."), LyricStyle::Italic)
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("do,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .barline()
        .measure_length(3, 2)
        .rest(Duration::new(DurationKind::Eighth, 1))
        .note(p(Note::Cis, 3), Duration::SIXTEENTH)
        .begin_tuplet(up)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .end_tuplet()
        .begin_tuplet(up)
        .note(p(Note::Fis, 3), Duration::EIGHTH)
        .note(p(Note::Fis, 3), Duration::EIGHTH)
        .note(p(Note::Fis, 3), Duration::EIGHTH)
        .end_tuplet()
        .note(p(Note::Ees, 4), Duration::SIXTEENTH)
        .note(p(Note::Ees, 4), Duration::SIXTEENTH)
        .note(p(Note::Ees, 4), Duration::SIXTEENTH)
        .note(p(Note::Ees, 4), Duration::SIXTEENTH)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .note(p(Note::A, 2), Duration::EIGHTH)
        .note(p(Note::Gis, 2), Duration::EIGHTH)
        .note(p(Note::Gis, 2), Duration::EIGHTH)
        .barline_style(BarlineStyle::Invisible);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(contents[0].meta.actual_length, MeasureLength::new(1, 1));
    assert_eq!(
        (
            contents[1].meta.nominal_length,
            contents[1].meta.actual_length
        ),
        (Some(MeasureLength::new(3, 2)), MeasureLength::new(3, 2))
    );
    assert_eq!(
        contents[1].meta.meter,
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4
        })
    );
    assert!(matches!(contents[1].barline, BarlineStyle::Invisible));
    assert_eq!(
        rhythmic(&contents[0].events),
        ["rest", "rest", "note", "note", "note", "note", "note"]
    );
    assert_eq!(
        notes(&contents[0].events),
        [
            (9, 3, 0, None),
            (9, 3, 0, None),
            (11, 2, 0, None),
            (3, 3, 0, Some(Glyph::AccidentalSharp)),
            (3, 3, 0, None)
        ]
    );
    let MeasureEvent::Rest(first) = &contents[0].events[0] else {
        panic!("opening rest")
    };
    assert_eq!(
        (
            first.annotations.dynamic.as_ref(),
            first.annotations.dynamics_placement
        ),
        (
            Some(&DynamicMark::Standard(Dynamic::Piano)),
            Placement::Above
        )
    );
    let page = layout_of(score.clone());
    let groups = scan_groups(
        page.systems[0].system.measures[0]
            .layout
            .elements
            .iter()
            .map(|e| &e.element),
    );
    assert_eq!((groups.tuplets.len(), groups.beams.len()), (1, 0));
    let members: Vec<_> = groups.tuplets[0]
        .members
        .iter()
        .map(|&i| &page.systems[0].system.measures[0].layout.elements[i].element)
        .collect();
    assert!(matches!(
        members[..],
        [
            MeasureElement::Rest(_),
            MeasureElement::Note(_),
            MeasureElement::Note(_)
        ]
    ));
    assert_eq!(
        groups.tuplets[0].spec.placement,
        Some(TupletPlacement::Above)
    );
    let svg = score.render_svg();
    assert!(lyric_y(&svg, "Ich") > lyric_y(&svg, "Jag"));
    assert!(svg
        .lines()
        .find(|line| line.contains(">Ich</text>"))
        .unwrap()
        .contains("font-style=\"italic\""));
    assert_eq!(glyph_count(&svg, Glyph::TimeSig4), 2);
    assert_eq!(glyph_count(&svg, Glyph::DynamicPiano), 1);
}

#[test]
fn c12_r003_mixed_quarter_triplet_rest_member_meter_and_reinstated_naturals() {
    // mn-c12-r003, printed p. 109 / PDF 100: 5/4 r4 {d'4 d'8} {cis8 cis8 cis8} {e'4 e'8} fis16*4 |
    // 4/4 dis'16*4 {c'!8 c'8 c'8} {f!8 f8 f8} {b8 b8 r8} | \bar "".
    let up = TupletSpec::new(3, 2).placement(TupletPlacement::Above);
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(5, 4)
        .rest(Duration::QTR)
        .begin_tuplet(up)
        .note(p(Note::D, 4), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("Du,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Du,"), LyricStyle::Italic)
        .note(p(Note::D, 4), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("Yohy"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Yohyo,"), LyricStyle::Italic)
        .end_tuplet()
        .begin_tuplet(up)
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::skip(), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("o."), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("in"), LyricStyle::Italic)
        .note(p(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("I"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("der"), LyricStyle::Italic)
        .end_tuplet()
        .begin_tuplet(up)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::E, 4), Duration::EIGHTH)
        .end_tuplet()
        .note(p(Note::Fis, 3), Duration::SIXTEENTH)
        .note(p(Note::Fis, 3), Duration::SIXTEENTH)
        .note(p(Note::Fis, 3), Duration::SIXTEENTH)
        .note(p(Note::Fis, 3), Duration::SIXTEENTH)
        .barline()
        .no_break()
        .time_signature_change(4, 4)
        .note(p(Note::Dis, 4), Duration::SIXTEENTH)
        .note(p(Note::Dis, 4), Duration::SIXTEENTH)
        .note(p(Note::Dis, 4), Duration::SIXTEENTH)
        .note(p(Note::Dis, 4), Duration::SIXTEENTH)
        .begin_tuplet(up)
        .note_with_accidental(p(Note::C, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .end_tuplet()
        .begin_tuplet(up)
        .note_with_accidental(p(Note::F, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::F, 3), Duration::EIGHTH)
        .note(p(Note::F, 3), Duration::EIGHTH)
        .end_tuplet()
        .begin_tuplet(up)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .rest(Duration::EIGHTH)
        .end_tuplet()
        .barline_style(BarlineStyle::Invisible);
    let contents = score.build_measure_contents().unwrap();
    assert_eq!(
        (
            contents[0].meta.actual_length,
            contents[1].meta.actual_length
        ),
        (MeasureLength::new(5, 4), MeasureLength::new(1, 1))
    );
    assert_eq!(
        contents[0].meta.meter,
        Some(TimeSignatureKind::Numeric {
            numerator: 5,
            denominator: 4
        })
    );
    assert_eq!(
        contents[1].meta.meter,
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4
        })
    );
    assert_eq!(
        notes(&contents[0].events)[..2],
        [(11, 2, 0, None), (11, 3, 0, None)]
    );
    assert_eq!(
        notes(&contents[1].events)[4..7],
        [
            (10, 3, 0, Some(Glyph::AccidentalNatural)),
            (10, 3, 0, None),
            (10, 3, 0, None)
        ]
    );
    assert_eq!(
        notes(&contents[1].events)[7..10],
        [
            (6, 3, 0, Some(Glyph::AccidentalNatural)),
            (6, 3, 0, None),
            (6, 3, 0, None)
        ]
    );
    assert!(matches!(contents[1].barline, BarlineStyle::Invisible));
    let page = layout_of(score.clone());
    let first = scan_groups(
        page.systems[0].system.measures[0]
            .layout
            .elements
            .iter()
            .map(|e| &e.element),
    );
    assert_eq!((first.tuplets.len(), first.beams.len()), (3, 0));
    assert_eq!(first.tuplets[0].members.len(), 2);
    assert!(
        (first.onsets[first.tuplets[0].members[1]]
            - first.onsets[first.tuplets[0].members[0]]
            - 1.0 / 6.0)
            .abs()
            < 1e-12,
        "quarter triplet takes one sixth of a whole note"
    );
    let bar_two = page
        .systems
        .iter()
        .flat_map(|system| &system.system.measures)
        .find(|measure| measure.meta.number == 2)
        .expect("4/4 source bar on some system");
    let second = scan_groups(bar_two.layout.elements.iter().map(|e| &e.element));
    assert_eq!((second.tuplets.len(), second.beams.len()), (3, 0));
    let members: Vec<_> = second.tuplets[2]
        .members
        .iter()
        .map(|&i| &bar_two.layout.elements[i].element)
        .collect();
    assert!(matches!(
        members[..],
        [
            MeasureElement::Note(_),
            MeasureElement::Note(_),
            MeasureElement::Rest(_)
        ]
    ));
    let svg = score.render_svg();
    assert!(lyric_y(&svg, "der") > lyric_y(&svg, "I"));
    assert_eq!(glyph_count(&svg, Glyph::AccidentalNatural), 2);
    assert_eq!(glyph_count(&svg, Glyph::TimeSig5), 1);
}
