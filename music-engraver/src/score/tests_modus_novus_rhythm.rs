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
use crate::layout::tempo::{
    MetronomeMark, MetronomeNoteKind as M, MetronomeUnit, MetronomeValue, TempoMark,
};
use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}
fn dotted(kind: DurationKind) -> Duration {
    Duration::new(kind, 1)
}
fn length(n: u64, d: u64) -> MeasureLength {
    MeasureLength::new(n, d)
}
fn measures(score: &ScoreBuilder) -> Vec<crate::layout::system::MeasureContent> {
    score
        .build_measure_contents()
        .expect("source fragment converts")
}
fn page(score: &ScoreBuilder) -> crate::layout::page::PageLayout {
    score.clone().page_layout(250.0).unwrap().unwrap()
}
fn note_xs(score: &ScoreBuilder, bar: usize) -> Vec<f64> {
    let page = page(score);
    page.systems
        .iter()
        .flat_map(|s| s.system.measures.iter())
        .nth(bar)
        .unwrap()
        .layout
        .elements
        .iter()
        .filter_map(|e| matches!(e.element, MeasureElement::Note(_)).then_some(e.x))
        .collect()
}
fn line_attr(line: &str, key: &str) -> f64 {
    line.split_once(&format!("{key}=\""))
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
fn text_line<'a>(svg: &'a str, word: &str) -> &'a str {
    svg.lines()
        .find(|line| line.contains(&format!(">{word}</text>")))
        .unwrap()
}
fn tempo(unit: impl Into<MetronomeUnit>, bpm: u16) -> TempoMark {
    TempoMark::metronome(MetronomeMark::bpm(unit, bpm).approx().parenthesized())
}

/// mn-c06-m004, printed p.60, bars 1–5 and 7: printed 3/4 → 4/4
/// with unprinted 9/8 length → regular 4/4; tied beam and breath member.
#[test]
fn mn_c06_m004_overridden_bar_and_group_member_breath() {
    let score = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(1)
        .note(p(Note::B, 4), dotted(DurationKind::Qtr))
        .tempo(tempo(M::Quarter, 80))
        .note(p(Note::A, 4), Duration::EIGHTH)
        .beam_group(vec![
            (p(Note::Bes, 4), Duration::EIGHTH),
            (p(Note::C, 5), Duration::EIGHTH),
        ])
        .barline()
        .time_signature_change(4, 4)
        .measure_length(9, 8)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::QTR)
        .rest(Duration::EIGHTH)
        .beam_group(vec![
            (p(Note::Fis, 4), Duration::EIGHTH),
            (p(Note::D, 5), Duration::EIGHTH),
            (p(Note::C, 5), Duration::EIGHTH),
        ])
        .barline()
        .reset_measure_length()
        .note(p(Note::B, 4), Duration::QTR)
        .begin_beam()
        .note(p(Note::Ees, 4), Duration::EIGHTH)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .end_beam()
        .begin_beam()
        .note(p(Note::F, 4), Duration::EIGHTH)
        .note(p(Note::Des, 4), Duration::EIGHTH)
        .note(p(Note::A, 3), Duration::EIGHTH)
        .end_beam()
        .rest(Duration::EIGHTH)
        .barline();
    let bars = measures(&score);
    assert_eq!(
        bars.iter()
            .map(|b| (b.meta.number, b.meta.actual_length, b.meta.nominal_length))
            .collect::<Vec<_>>(),
        [
            (1, length(3, 4), Some(length(3, 4))),
            (2, length(9, 8), Some(length(9, 8))),
            (3, length(1, 1), Some(length(1, 1)))
        ]
    );
    let first = bars[0]
        .events
        .iter()
        .find_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        first.annotations.tempo_mark.as_ref().unwrap(),
        &tempo(M::Quarter, 80)
    );
    let svg = score.render_svg();
    assert!(svg.contains("c. 80"));
    let beam_breath = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(7)
        .begin_beam()
        .note(p(Note::D, 4), Duration::EIGHTH)
        .breath_mark(crate::layout::breath::BreathMark::Comma)
        .note(p(Note::Des, 5), Duration::EIGHTH)
        .note(p(Note::Bes, 4), Duration::EIGHTH)
        .note_with_accidental(p(Note::D, 4), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::B, 3), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .end_beam()
        .barline();
    let bar = measures(&beam_breath);
    assert_eq!(bar[0].meta.actual_length, length(3, 4));
    let breath = bar[0]
        .events
        .iter()
        .find_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n.annotations.breath_mark)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(breath, Some(crate::layout::breath::BreathMark::Comma));
    let svg = beam_breath.render_svg();
    assert!(svg.contains("<polygon"));
    let tied = ScoreBuilder::new()
        .time_signature(4, 4)
        .first_measure_number(4)
        .rest(Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::E, 4), Duration::EIGHTH)
        .note(p(Note::D, 4), Duration::EIGHTH)
        .note(p(Note::C, 4), Duration::EIGHTH)
        .end_beam()
        .note(p(Note::Aes, 4), Duration::QTR)
        .begin_beam()
        .note(p(Note::E, 4), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .tie()
        .end_beam()
        .barline()
        .begin_beam()
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note(p(Note::Fis, 4), Duration::EIGHTH)
        .note(p(Note::Bes, 4), Duration::EIGHTH)
        .note(p(Note::D, 5), Duration::EIGHTH)
        .end_beam()
        .begin_beam()
        .note(p(Note::G, 4), dotted(DurationKind::Eighth))
        .note(p(Note::B, 4), Duration::SIXTEENTH)
        .end_beam()
        .note(p(Note::Ais, 4), Duration::QTR)
        .barline();
    let tied_bars = measures(&tied);
    assert_eq!(
        tied_bars
            .iter()
            .map(|m| m.meta.actual_length)
            .collect::<Vec<_>>(),
        [length(1, 1); 2]
    );
    let last = tied_bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n)
            } else {
                None
            }
        })
        .last()
        .unwrap();
    assert!(last.annotations.tie_forward);
    assert_eq!(note_xs(&tied, 1).len(), 7);
    let svg = tied.render_svg();
    assert!(
        svg.lines().any(|line| line.contains("<path d=\"M")
            && line.matches(" C").count() == 2
            && line.contains("Z\"")),
        "source's G4 beam-to-beam tie prints a closed crescent across the barline"
    );
}

/// mn-c11-r015, printed p.98, bars 10, 14–15, 20 and 28–32: no
/// printed signature in the short bars; bars 29 and 31 are empty spacers.
#[test]
fn mn_c11_r015_short_bars_and_two_empty_spacer_bars() {
    let short_10 = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(10)
        .measure_length(3, 8)
        .note(p(Note::Bes, 4), Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::Aes, 4), Duration::EIGHTH)
        .note(p(Note::Ges, 4), Duration::EIGHTH)
        .end_beam()
        .barline();
    assert_eq!(
        (
            measures(&short_10)[0].meta.number,
            measures(&short_10)[0].meta.actual_length
        ),
        (10, length(3, 8))
    );
    let short_14 = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(14)
        .measure_length(2, 4)
        .rest(Duration::QTR)
        .rest(Duration::QTR)
        .barline()
        .measure_length(1, 4)
        .note(p(Note::Bes, 4), Duration::QTR)
        .barline();
    let bars = measures(&short_14);
    assert_eq!(
        bars.iter()
            .map(|m| (m.meta.number, m.meta.actual_length, m.meta.nominal_length))
            .collect::<Vec<_>>(),
        [
            (14, length(1, 2), Some(length(1, 2))),
            (15, length(1, 4), Some(length(1, 4)))
        ]
    );
    let empty = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(28)
        .note(p(Note::E, 4), dotted(DurationKind::Half))
        .barline()
        .spacer(dotted(DurationKind::Half))
        .barline()
        .note_with_accidental(
            p(Note::C, 5),
            dotted(DurationKind::Half),
            AccidentalDisplay::Force,
        )
        .barline()
        .spacer(dotted(DurationKind::Half))
        .barline()
        .note(p(Note::Aes, 4), dotted(DurationKind::Half))
        .barline();
    let bars = measures(&empty);
    assert_eq!(
        bars.iter().map(|m| m.meta.number).collect::<Vec<_>>(),
        [28, 29, 30, 31, 32]
    );
    for i in [1, 3] {
        assert_eq!(bars[i].meta.actual_length, length(3, 4));
        assert!(matches!(bars[i].events[..], [MeasureEvent::Spacer(_)]));
    }
    let layout = page(&empty);
    for i in [1, 3] {
        let m = &layout.systems[0].system.measures[i].layout;
        assert!(m
            .elements
            .iter()
            .all(|e| !matches!(e.element, MeasureElement::Rest(_) | MeasureElement::Note(_))));
        assert!(
            m.closing_barline_x()
                >= crate::layout::measure::MeasureLayoutConfig::from_staff_space(250.0)
                    .empty_measure_min_width
        );
    }
    let svg = empty.render_svg();
    assert!(svg.contains("<svg"));
    // Bar 20: the custom \"più p\" belongs to the eighth rest, above the
    // staff (\dynamicUp), not to the following tenuto note.
    let piu = crate::layout::dynamics::CustomDynamic::new()
        .text("più")
        .mark(crate::layout::dynamics::Dynamic::Piano);
    let expressive = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(20)
        .begin_beam()
        .note(p(Note::Aes, 4), Duration::EIGHTH)
        .note(p(Note::Fes, 4), Duration::EIGHTH)
        .end_beam()
        .rest(Duration::QTR)
        .rest(Duration::EIGHTH)
        .dynamic_placed(piu.clone(), Placement::Above)
        .note(p(Note::F, 4), Duration::EIGHTH)
        .articulation(crate::layout::articulation::Articulation::Tenuto)
        .barline();
    let bar = measures(&expressive);
    assert_eq!(bar[0].meta.actual_length, length(3, 4));
    let rest = bar[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Rest(r) = e {
                Some(r)
            } else {
                None
            }
        })
        .last()
        .unwrap();
    assert_eq!(
        rest.annotations.dynamic,
        Some(crate::layout::dynamics::DynamicMark::Custom(piu))
    );
    let staff_y = page(&expressive).systems[0].y;
    let rendered = expressive.render_svg();
    assert!(
        line_attr(text_line(&rendered, "più"), "y") < staff_y,
        "source's dynamicUp puts the custom rest dynamic above the staff"
    );
}

/// mn-c11-r027, printed pp.102–103: contiguous tail of the first unmetered
/// 24/4 phrase around its invisible mid-measure break. The source phrase
/// continues after this fragment: only one logical bar number is consumed.
#[test]
fn mn_c11_r027_inline_invisible_split_keeps_long_phrase() {
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .hidden_time_signature(24, 4)
        .first_measure_number(1)
        .accidental_policy(AccidentalPolicy::Forget)
        .note(p(Note::Cis, 4), Duration::QTR)
        .note_with_accidental(p(Note::A, 3), Duration::QTR, AccidentalDisplay::Force)
        .begin_beam()
        .note(p(Note::D, 3), Duration::EIGHTH)
        .note_with_accidental(p(Note::B, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam()
        .inline_barline(BarlineStyle::Invisible)
        .system_break()
        .begin_beam()
        .note_with_accidental(p(Note::E, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::C, 3), Duration::EIGHTH, AccidentalDisplay::Force)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::C, 3), Duration::EIGHTH)
        .end_beam()
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::C, 3), Duration::EIGHTH)
        .note_with_accidental(p(Note::F, 2), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam()
        .end_tuplet()
        .barline_style(BarlineStyle::Invisible);
    let bars = measures(&score);
    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].meta.number, 1);
    assert_eq!(bars[0].meta.nominal_length, Some(length(6, 1)));
    assert_eq!(bars[0].meta.actual_length, length(3, 2)); // 3/4 + 1/2 + (3 eighths at 2:3 = 1/4).
    assert!(!bars[0].meta.meter_visible);
    let notes: Vec<_> = bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        notes[4].accidental.unwrap().glyph,
        smufl::Glyph::AccidentalNatural
    );
    assert!(
        notes[6].accidental.is_none(),
        "unmarked E after forced E remains unmarked under forget policy"
    );
    let page = page(&score);
    assert_eq!(page.systems.len(), 2);
    assert_eq!(
        page.systems[0].system.measures[0].meta.number,
        page.systems[1].system.measures[0].meta.number
    );
    assert_eq!(page.systems[0].system.measures[0].meta.number, 1);
    let svg = score.render_svg();
    let printed_two = crate::font::bravura_font()
        .glyph_outline(smufl::Glyph::TimeSig2)
        .unwrap()
        .path_data;
    assert!(
        !svg.contains(&printed_two),
        "24/4 is retained as a hidden meter, never printed"
    );
}

/// mn-c04-r007, printed p.43, opening range tempo and last 9/8
/// plus first 6/8 bar: stem-down beam, secondary subdivision,
/// dashed bracket BELOW across the meter change.
#[test]
fn mn_c04_r007_subdivided_stems_and_bracket_over_meter() {
    let bracket = AnalysisBracketSpec::new(AnalysisBracketStyle::Dashed, Placement::Below);
    let score = ScoreBuilder::new()
        .time_signature(9, 8)
        .first_measure_number(4)
        .begin_beam_with(BeamSpec::new().subdivide(3))
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note(p(Note::D, 4), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .end_beam()
        .begin_beam_with(
            BeamSpec::new()
                .stem_direction(StemDirection::Down)
                .subdivide(3),
        )
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note(p(Note::Gis, 4), Duration::EIGHTH)
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .end_beam()
        .begin_beam_with(BeamSpec::new().subdivide(3))
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .note(p(Note::D, 5), Duration::EIGHTH)
        .analysis_bracket_start(bracket)
        .note_with_accidental(p(Note::G, 5), Duration::EIGHTH, AccidentalDisplay::Force)
        .end_beam()
        .barline()
        .time_signature_change(6, 8)
        .note(p(Note::G, 5), Duration::EIGHTH)
        .note(p(Note::Gis, 5), Duration::QTR)
        .note(p(Note::Cis, 6), Duration::QTR)
        .note(p(Note::D, 5), Duration::EIGHTH)
        .analysis_bracket_end()
        .barline_style(BarlineStyle::Invisible);
    let bars = measures(&score);
    assert_eq!(
        (bars[0].meta.actual_length, bars[1].meta.actual_length),
        (length(9, 8), length(3, 4))
    );
    assert_eq!(
        bars[1].meta.meter,
        Some(crate::layout::time_signature::TimeSignatureKind::Numeric {
            numerator: 6,
            denominator: 8
        })
    );
    assert_eq!(bars[1].barline, BarlineStyle::Invisible);
    assert!(bars[0].events.iter().any(|e| matches!(e, MeasureEvent::GroupMark(GroupMark::BeamStart {spec,..}) if spec.stem_direction==Some(StemDirection::Down) && spec.subdivide_log2==Some(3))));
    let xs = note_xs(&score, 0);
    let svg = score.render_svg();
    let bracket_line = svg
        .lines()
        .find(|l| {
            l.contains("stroke-dasharray=")
                && l.contains("<line ")
                && (line_attr(l, "y1") - line_attr(l, "y2")).abs() < 0.01
        })
        .unwrap();
    assert!((line_attr(bracket_line, "x1") - xs[7]).abs() < 0.01);
    assert!(line_attr(bracket_line, "x2") > xs[8]);
    // The same source's opening bar starts on a dotted-eighth rest: its
    // parenthesized dotted-quarter = 58-56 belongs above that rest.
    let mark = TempoMark::metronome(
        MetronomeMark::range(MetronomeUnit::dotted(M::Quarter), 58, 56).parenthesized(),
    );
    let opening = |subdivided| {
        ScoreBuilder::new()
            .time_signature(9, 8)
            .rest(dotted(DurationKind::Eighth))
            .tempo(mark.clone())
            .begin_beam_with(if subdivided {
                BeamSpec::new().subdivide(3)
            } else {
                BeamSpec::new()
            })
            .note(p(Note::E, 4), Duration::SIXTEENTH)
            .note(p(Note::A, 4), Duration::SIXTEENTH)
            .note(p(Note::D, 5), Duration::SIXTEENTH)
            .end_beam()
            .note(p(Note::Cis, 5), dotted(DurationKind::Qtr))
            .note(p(Note::Cis, 5), Duration::QTR)
            .note(p(Note::Gis, 4), Duration::EIGHTH)
            .barline()
    };
    let plain_svg = opening(false).render_svg();
    let opening = opening(true);
    let bar = measures(&opening);
    assert_eq!(bar[0].meta.actual_length, length(9, 8));
    let MeasureEvent::Rest(rest) = &bar[0].events[0] else {
        panic!("tempo begins over rest")
    };
    assert_eq!(rest.annotations.tempo_mark.as_ref(), Some(&mark));
    assert_eq!(
        mark.metronome.as_ref().unwrap().value,
        MetronomeValue::Range(58, 56)
    );
    assert_eq!(mark.metronome.as_ref().unwrap().unit.dots, 1);
    let svg = opening.render_svg();
    assert!(svg.contains("58-56"));
    assert!(
        svg.matches("<polygon").count() > plain_svg.matches("<polygon").count(),
        "e'16 a'16 d''16 has a secondary-beam break at the eighth boundary"
    );
}

/// mn-c04-r010, printed p.44: 11-sixteenth pickup, approximate dotted
/// quarter tempo, dashed bracket ABOVE on a later beamed member and gliss.
#[test]
fn mn_c04_r010_pickup_and_member_gliss() {
    let score = ScoreBuilder::new()
        .time_signature(6, 8)
        .partial(length(11, 16))
        .begin_beam()
        .note(p(Note::Gis, 4), Duration::SIXTEENTH)
        .tempo(tempo(MetronomeUnit::dotted(M::Quarter), 140))
        .note(p(Note::Cis, 5), Duration::SIXTEENTH)
        .note(p(Note::E, 5), Duration::SIXTEENTH)
        .end_beam()
        .note(p(Note::D, 6), Duration::HALF)
        .barline()
        .note(p(Note::D, 6), Duration::EIGHTH)
        .note(p(Note::A, 5), Duration::EIGHTH)
        .note(p(Note::B, 5), Duration::EIGHTH)
        .note(p(Note::E, 5), Duration::EIGHTH)
        .note(p(Note::F, 5), Duration::EIGHTH)
        .note(p(Note::G, 5), Duration::EIGHTH)
        .barline()
        .begin_beam()
        .note(p(Note::Gis, 5), dotted(DurationKind::Eighth))
        .note(p(Note::Fis, 5), dotted(DurationKind::Eighth))
        .end_beam()
        .note(p(Note::Cis, 5), dotted(DurationKind::Qtr))
        .barline()
        .note(p(Note::Cis, 5), dotted(DurationKind::Half))
        .barline()
        .note(p(Note::Cis, 5), dotted(DurationKind::Qtr))
        .begin_beam()
        .note(p(Note::Cis, 5), dotted(DurationKind::Eighth))
        .analysis_bracket_start(AnalysisBracketSpec::new(
            AnalysisBracketStyle::Dashed,
            Placement::Above,
        ))
        .glissando(crate::layout::glissando::GlissandoStyle::Line)
        .text_script(crate::layout::text_script::TextScript::below("gliss.").italic())
        .note(p(Note::Gis, 4), dotted(DurationKind::Eighth))
        .end_beam()
        .barline()
        .note(p(Note::B, 4), dotted(DurationKind::Half))
        .analysis_bracket_end()
        .barline_style(BarlineStyle::Single);
    let bars = measures(&score);
    assert_eq!(
        (
            bars[0].meta.number,
            bars[0].meta.actual_length,
            bars[0].meta.anacrusis
        ),
        (0, length(11, 16), true)
    );
    assert_eq!(bars[1].meta.number, 1);
    let m = match &bars[0].events[1] {
        MeasureEvent::Note(n) => n,
        _ => panic!("first pickup note"),
    };
    assert_eq!(
        m.annotations.tempo_mark.as_ref().unwrap(),
        &tempo(MetronomeUnit::dotted(M::Quarter), 140)
    );
    let svg = score.clone().render_svg();
    assert!(svg.contains("c. 140"));
    assert_eq!(
        bars.iter()
            .map(|b| b.meta.actual_length)
            .collect::<Vec<_>>(),
        [
            length(11, 16),
            length(3, 4),
            length(3, 4),
            length(3, 4),
            length(3, 4),
            length(3, 4)
        ]
    );
    let above = page(&score);
    let staff_y = above.systems.last().unwrap().y;
    let gliss_text = text_line(&svg, "gliss.");
    assert!(gliss_text.contains("font-style=\"italic\""));
    assert!(line_attr(gliss_text, "y") > staff_y + 4.0 * 250.0);
    assert!(svg.lines().any(|l| l.contains("stroke-dasharray=")
        && l.contains("<line ")
        && line_attr(l, "y1") < staff_y));
}

/// mn-c08-r002, printed p.70: cadenza bass, two unbeamed 3:2
/// eighth-note groups with independent Swedish and German lyric lanes.
#[test]
fn mn_c08_r002_unbeamed_tuplets_and_two_verse_underlay() {
    let mut score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .cadenza_on()
        .note(p(Note::C, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("Jag"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Ich"), LyricStyle::Upright)
        .note(p(Note::C, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("är"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("bin"), LyricStyle::Upright);
    for (pitch, sw, de) in [
        (Note::F, "Un", "Un"),
        (Note::F, "zu", "zu"),
        (Note::F, "från", "vom"),
    ] {
        if sw == "Un" {
            score = score.begin_tuplet(
                TupletSpec::new(3, 2).placement(crate::layout::tuplet::TupletPlacement::Above),
            );
        }
        score = score
            .note(p(pitch, 3), Duration::EIGHTH)
            .lyric_verse(1, LyricSyllable::word(sw), LyricStyle::Upright)
            .lyric_verse(2, LyricSyllable::word(de), LyricStyle::Upright);
    }
    score = score.end_tuplet().begin_tuplet(
        TupletSpec::new(3, 2).placement(crate::layout::tuplet::TupletPlacement::Above),
    );
    for (sw, de) in [("byn", "Dorf"), ("här", "ne"), ("in", "ben")] {
        score = score
            .note(p(Note::B, 3), Duration::EIGHTH)
            .lyric_verse(1, LyricSyllable::word(sw), LyricStyle::Upright)
            .lyric_verse(2, LyricSyllable::word(de), LyricStyle::Upright);
    }
    score = score
        .end_tuplet()
        .note(p(Note::D, 3), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("till."), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("an."), LyricStyle::Upright)
        .barline_style(BarlineStyle::Double);
    let bars = measures(&score);
    assert_eq!(bars[0].meta.nominal_length, None);
    assert_eq!(bars[0].meta.actual_length, length(1, 1));
    assert_eq!(
        bars[0]
            .events
            .iter()
            .filter(|e| matches!(e, MeasureEvent::GroupMark(GroupMark::TupletStart { .. })))
            .count(),
        2
    );
    assert!(!bars[0]
        .events
        .iter()
        .any(|e| matches!(e, MeasureEvent::GroupMark(GroupMark::BeamStart { .. }))));
    let bass_notes: Vec<_> = bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n.staff_position)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(bass_notes, [3, 3, 6, 6, 6, 9, 9, 9, 4]); // C3 F3 B3 D3 in bass clef (G2 = 0).
    let xs = note_xs(&score, 0);
    assert_eq!(xs.len(), 9);
    assert!(xs.windows(2).all(|pair| pair[1] > pair[0]));
    let svg = score.render_svg();
    assert_eq!(
        line_attr(text_line(&svg, "Jag"), "y") + 500.0,
        line_attr(text_line(&svg, "Ich"), "y")
    );
    assert!(line_attr(text_line(&svg, "Un"), "x") >= xs[2] - 100.0);
}

/// mn-c11-r028, printed p.103, bars 1–3 and 7: R1 is an H-bar;
/// the mixed quarter/eighth triplet occupies half a measure with its
/// final two eighths beamed, not the three quarter members.
#[test]
fn mn_c11_r028_rest_identity_and_partial_inner_beam_ratio() {
    let opening = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .multi_measure_rest(1)
        .barline()
        .rest(Duration::HALF)
        .rest(Duration::QTR)
        .rest(Duration::EIGHTH)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .barline()
        .note(p(Note::F, 3), Duration::QTR)
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .note(p(Note::E, 3), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .note(p(Note::F, 3), dotted(DurationKind::Qtr))
        .note(p(Note::E, 3), Duration::EIGHTH)
        .barline();
    let bars = measures(&opening);
    assert!(matches!(
        bars[0].events[..],
        [MeasureEvent::MultiMeasureRest { count: 1, .. }]
    ));
    assert_eq!(
        bars.iter()
            .map(|b| b.meta.actual_length)
            .collect::<Vec<_>>(),
        [length(1, 1), length(1, 1), length(1, 1)]
    );
    let svg = opening.render_svg();
    assert!(
        svg.contains("<rect"),
        "printed R1 has a multi-measure H-bar, not a whole-note rest"
    );
    let bar7 = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .first_measure_number(7)
        .note(p(Note::B, 3), dotted(DurationKind::Qtr))
        .note(p(Note::Ais, 3), Duration::EIGHTH)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::D, 4), Duration::QTR)
        .note(p(Note::Cis, 4), Duration::QTR)
        .begin_beam()
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .note(p(Note::Cis, 4), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .barline();
    let bars = measures(&bar7);
    assert_eq!(bars[0].meta.actual_length, length(1, 1));
    let events = &bar7.measures[0].events;
    let midi: Vec<_> = events
        .iter()
        .filter_map(|(_, e)| {
            if let event::ScoreEvent::Note { pitch, .. } = e {
                Some(pitch.midi_note)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(midi, [59, 58, 62, 61, 61, 61]); // B3 A#3 D4 C#4 C#4 C#4 as printed.
    let staff_positions: Vec<_> = bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n.staff_position)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(staff_positions, [9, 8, 11, 10, 10, 10]);
    let timeline = event::measure_timeline(events, &[], Some(length(1, 1)));
    let onsets: Vec<_> = events
        .iter()
        .zip(&timeline.onsets)
        .filter_map(|((_, e), &t)| {
            matches!(e, event::ScoreEvent::Note { .. }).then_some(timeline.to_length(t))
        })
        .collect();
    assert_eq!(
        onsets,
        [
            length(0, 1),
            length(3, 8),
            length(1, 2),
            length(2, 3),
            length(5, 6),
            length(11, 12)
        ]
    );
    let marks: Vec<_> = bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::GroupMark(m) = e {
                Some(m)
            } else {
                None
            }
        })
        .collect();
    assert!(matches!(
        marks.as_slice(),
        [
            GroupMark::TupletStart { .. },
            GroupMark::BeamStart { .. },
            GroupMark::BeamEnd { .. },
            GroupMark::TupletEnd { .. }
        ]
    ));
    assert_eq!(
        bars[0]
            .events
            .iter()
            .filter(|e| matches!(e, MeasureEvent::Note(_)))
            .count(),
        6
    );
    assert!(note_xs(&bar7, 0).windows(2).all(|xs| xs[1] > xs[0]));
    let svg = bar7.render_svg();
    assert!(svg.contains("<polygon"));
}

/// Compressed multi-bar rests must consume *performed* time and bar numbers,
/// even when a non-power-of-two measure-length override is in force.
#[test]
fn multimeasure_rest_two_bars_advances_exact_grid_and_number() {
    let score = ScoreBuilder::new()
        .time_signature(3, 4)
        .first_measure_number(7)
        .measure_length(5, 6)
        .multi_measure_rest(2)
        .barline()
        .system_break()
        .reset_measure_length()
        .note(p(Note::E, 3), dotted(DurationKind::Half))
        .barline();
    let bars = measures(&score);
    assert_eq!(
        (
            bars[0].meta.number,
            bars[0].meta.nominal_length,
            bars[0].meta.actual_length
        ),
        (7, Some(length(5, 3)), length(5, 3))
    );
    assert_eq!(
        (bars[1].meta.number, bars[1].meta.actual_length),
        (9, length(3, 4))
    );
    assert!(matches!(
        bars[0].events[..],
        [MeasureEvent::MultiMeasureRest { count: 2, .. }]
    ));
    assert!(
        !bars[0].meta.is_incomplete(),
        "compressed two-bar frame has matching expected and performed spans"
    );
    let layout = page(&score);
    assert_eq!(layout.systems.len(), 2);
    assert_eq!(layout.systems[0].system.measures[0].meta.number, 7);
    assert_eq!(layout.systems[1].system.measures[0].meta.number, 9);
    let svg = score.render_svg();
    assert!(
        text_line(&svg, "2").contains(">2</text>"),
        "compressed rest prints its two-bar count"
    );
}

/// mn-c11-r029, printed p.104 (Aniara score p.54), bar 7:
/// simultaneous G4 eighths above G4 sixteenth/sixteenth/eighth cue voice;
/// German `associatedVoice` follows the lower layer, Swedish the upper.
#[test]
fn mn_c11_r029_interior_voice_grid_cue_lyrics_and_hidden_bar() {
    let mut score = ScoreBuilder::new()
        .time_signature(4, 4)
        .first_measure_number(7)
        .begin_beam_with(BeamSpec::new().stem_direction(StemDirection::Down))
        .note(p(Note::F, 5), dotted(DurationKind::Eighth))
        .lyric_verse(1, LyricSyllable::with_hyphen("ment"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("ment"), LyricStyle::Italic)
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("kol"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::with_hyphen("zu"), LyricStyle::Italic)
        .end_beam()
        .begin_beam()
        .note(p(Note::G, 4), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("laps"), LyricStyle::Upright)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("i"), LyricStyle::Upright)
        .end_beam()
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .begin_beam()
        .note(p(Note::B, 4), Duration::EIGHTH)
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .lyric_associated_voice(2, 1)
        .voice(1)
        .spacer(Duration::QTR)
        .begin_beam()
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .lyric_verse(2, LyricSyllable::with_hyphen("sam"), LyricStyle::Italic)
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .note_size(NoteSize::Cue)
        .lyric_verse(2, LyricSyllable::with_hyphen("men"), LyricStyle::Italic)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note_size(NoteSize::Cue)
        .lyric_verse(2, LyricSyllable::word("bruch"), LyricStyle::Italic)
        .end_beam()
        .barline()
        .lyric_associated_voice(2, 0)
        .hidden_time_signature_change(7, 8)
        .begin_beam()
        .note(p(Note::Fis, 5), Duration::EIGHTH)
        .note(p(Note::Fis, 5), Duration::EIGHTH)
        .end_beam()
        .rest(Duration::QTR)
        .note(p(Note::G, 5), Duration::EIGHTH)
        .note(p(Note::C, 5), dotted(DurationKind::Eighth))
        .note(p(Note::C, 5), Duration::SIXTEENTH)
        .barline();
    let bars = measures(&score);
    assert_eq!(bars[0].meta.actual_length, length(1, 1));
    assert_eq!(bars[1].meta.actual_length, length(7, 8));
    assert!(!bars[1].meta.meter_visible);
    let timeline_events = &score.measures[0].events;
    let ticks = event::measure_timeline(timeline_events, &[], Some(length(1, 1)));
    let lower_onsets: Vec<_> = timeline_events
        .iter()
        .zip(&ticks.onsets)
        .filter_map(|((voice, e), &t)| {
            (*voice == 1 && matches!(e, event::ScoreEvent::Note { .. }))
                .then_some(ticks.to_length(t))
        })
        .collect();
    assert_eq!(lower_onsets, [length(1, 4), length(5, 16), length(3, 8)]);
    let secondary: Vec<_> = bars[0].additional_voices[0]
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(secondary.len(), 3);
    assert_eq!(
        secondary
            .iter()
            .map(|n| n.annotations.size)
            .collect::<Vec<_>>(),
        [NoteSize::Normal, NoteSize::Cue, NoteSize::Cue]
    );
    assert_eq!(
        secondary
            .iter()
            .map(|n| n.annotations.lyrics[0].syllable.text.as_str())
            .collect::<Vec<_>>(),
        ["sam", "men", "bruch"]
    );
    let main: Vec<_> = bars[0]
        .events
        .iter()
        .filter_map(|e| {
            if let MeasureEvent::Note(n) = e {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(main[2].annotations.lyrics[0].syllable.text, "laps");
    assert_eq!(main[0].stem_direction, Some(StemDirection::Down));
    assert_eq!(main[1].stem_direction, Some(StemDirection::Down));
    assert_eq!(main[2].stem_direction, Some(StemDirection::Up));
    let layout = page(&score);
    let first = &layout.systems[0].system.measures[0];
    let primary_g: Vec<_> = first
        .layout
        .elements
        .iter()
        .filter_map(|e| match &e.element {
            MeasureElement::Note(n) if n.staff_position == 2 => Some(e.x),
            _ => None,
        })
        .collect();
    let secondary_g: Vec<_> = first.additional_voice_layouts[0]
        .elements
        .iter()
        .filter_map(|e| match &e.element {
            MeasureElement::Note(n) if n.staff_position == 2 => Some(e.x),
            _ => None,
        })
        .collect();
    assert_eq!((primary_g.len(), secondary_g.len()), (3, 3));
    assert!(
        (primary_g[1] - secondary_g[0]).abs() < 0.01,
        "upper and lower G4 share interior beat column"
    );
    assert!(
        (primary_g[2] - secondary_g[2]).abs() < 0.01,
        "final eighths share the next column"
    );
    assert!(secondary_g[0] < secondary_g[1] && secondary_g[1] < secondary_g[2]);
    let svg = score.render_svg();
    assert!(text_line(&svg, "sam").contains("font-style=\"italic\""));
    assert!(line_attr(text_line(&svg, "laps"), "y") < line_attr(text_line(&svg, "sam"), "y"));
    assert!(
        svg.contains("scale(0.7"),
        "small cue noteheads print in secondary voice"
    );
    score = ScoreBuilder::new()
        .time_signature(4, 4)
        .rest(Duration::HALF)
        .tempo(TempoMark::metronome(
            MetronomeMark::bpm(M::Quarter, 112).parenthesized(),
        ))
        .rest(Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::Des, 5), Duration::SIXTEENTH)
        .dynamic(crate::layout::dynamics::Dynamic::Forte)
        .text_script(crate::layout::text_script::TextScript::above("secco ma con forza").italic())
        .note(p(Note::Des, 5), Duration::SIXTEENTH)
        .end_beam()
        .begin_beam()
        .note(p(Note::Des, 5), Duration::EIGHTH)
        .note(p(Note::C, 5), Duration::EIGHTH)
        .end_beam()
        .barline();
    assert_eq!(measures(&score)[0].meta.actual_length, length(1, 1));
    let MeasureEvent::Rest(rest) = &measures(&score)[0].events[0] else {
        panic!("leading half rest")
    };
    assert_eq!(
        rest.annotations
            .tempo_mark
            .as_ref()
            .unwrap()
            .metronome
            .as_ref()
            .unwrap()
            .value,
        MetronomeValue::Bpm(112)
    );
    let svg = score.render_svg();
    assert!(svg.contains("112"));
    assert!(text_line(&svg, "secco ma con forza").contains("font-style=\"italic\""));
    let forte = crate::font::bravura_font()
        .glyph_outline(smufl::Glyph::DynamicForte)
        .unwrap()
        .path_data;
    assert!(
        svg.contains(&forte),
        "source attack is forte on the first sixteenth"
    );
}

/// mn-c11-r029, final bar: `<< e''4 { s8 s8\! } >>`. The second
/// invisible eighth ends the long "dim" span at 7/8, *inside* the
/// primary voice's sustained final quarter, not at its next note or barline.
#[test]
fn mn_c11_r029_spacer_end_uses_exact_intra_note_onset() {
    use crate::layout::text_spanner::TextSpanner;
    use crate::render::system_renderer::{
        collect_hairpin_note_info, collect_text_spanner_note_info,
    };

    let score = ScoreBuilder::new()
        .time_signature(4, 4)
        .dynamics_placement(Placement::Above) // source `\dynamicUp`
        .note(p(Note::Fis, 4), dotted(DurationKind::Qtr))
        .hairpin_start(HairpinType::Decrescendo)
        .text_spanner_start(TextSpanner::dim().placed(Placement::Above))
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .voice(1)
        .spacer(dotted(DurationKind::Half)) // 3/4: simultaneous with final quarter
        .spacer(Duration::EIGHTH)
        .spacer(Duration::EIGHTH)
        .hairpin_end()
        .text_spanner_end()
        .end_barline();
    let bars = measures(&score);
    assert_eq!(bars[0].meta.actual_length, length(1, 1));
    let secondary = &bars[0].additional_voices[0];
    let MeasureEvent::Spacer(prelude) = &secondary[1] else { panic!("first eighth") };
    assert!(!prelude.annotations.hairpin_end);
    let MeasureEvent::Spacer(end) = &secondary[2] else { panic!("second eighth ends span") };
    assert_eq!(end.duration_log2, Some(3));
    assert!(end.annotations.hairpin_end && end.annotations.text_spanner_end);

    let timeline = event::measure_timeline(&score.measures[0].events, &[], Some(length(1, 1)));
    let end_tick = score.measures[0].events.iter().zip(&timeline.onsets)
        .find_map(|((voice, event), &tick)| {
            (*voice == 1 && matches!(event, event::ScoreEvent::Spacer { annotations, .. } if annotations.hairpin_end))
                .then_some(timeline.to_length(tick))
        }).unwrap();
    assert_eq!(end_tick, length(7, 8));

    let page = page(&score);
    let system = &page.systems[0].system;
    let primary = &system.measures[0].layout.elements;
    let last_note_x = primary.iter().filter_map(|e| matches!(e.element, MeasureElement::Note(_)).then_some(e.x)).last().unwrap();
    let secondary = &system.measures[0].additional_voice_layouts[0].elements;
    let spacer_x = secondary.iter().find_map(|e| match &e.element {
        MeasureElement::Spacer(s) if s.annotations.hairpin_end => Some(e.x),
        _ => None,
    }).unwrap();
    let barline_x = system.measures[0].layout.closing_barline_x();
    assert!(last_note_x < spacer_x && spacer_x < barline_x, "the 7/8 stop precedes bar end");
    let hp = collect_hairpin_note_info(system);
    let text = collect_text_spanner_note_info(system);
    assert_eq!(hp.iter().find(|e| e.hairpin_end).unwrap().x, system.measures[0].x_offset + spacer_x);
    assert_eq!(text.iter().find(|e| e.end).unwrap().x, system.measures[0].x_offset + spacer_x);

    let svg = score.render_svg();
    assert!(svg.contains(">dim</text>") || svg.contains(">dim.</text>"), "text span label");
    let x2 = svg.lines().filter(|line| line.contains("<line ") && line.contains(" x2=\""))
        .filter_map(|line| {
            let x1 = line_attr(line, "x1");
            let x2 = line_attr(line, "x2");
            (x1 < x2 && line_attr(line, "y1") != line_attr(line, "y2")).then_some(x2)
        }).next().expect("rendered wedge");
    let expected = page.systems[0].x + hp.iter().find(|e| e.hairpin_end).unwrap().x - 0.3 * 250.0;
    assert!((x2 - expected).abs() < 0.02, "wedge SVG endpoint {x2} vs 7/8 onset {expected}");
    let rest = crate::font::bravura_font().glyph_outline(smufl::Glyph::Rest8th).unwrap().path_data;
    assert!(!svg.contains(&rest), "invisible eighths must not render as eighth rests");
    #[cfg(feature = "png")]
    if let Ok(path) = std::env::var("SPANNER_ANCHOR_VISUAL_OUT") {
        let mut renderer = crate::render::png::PngRenderer::new(1.5);
        renderer.load_system_fonts();
        let png = renderer.render_png(&svg).unwrap();
        std::fs::write(&path, png).unwrap();
        std::fs::write(path.replace(".png", ".svg"), &svg).unwrap();
    }
}

/// mn-c11-r015 final `d'2.)\> <>\!`: the stop belongs after the
/// dotted half at the bar end and must not lengthen its 3/4 measure.
#[test]
fn mn_c11_r015_zero_duration_final_anchor_is_not_a_note_or_rest() {
    use crate::render::system_renderer::collect_hairpin_note_info;
    let score = ScoreBuilder::new()
        .time_signature(3, 4)
        .note(p(Note::D, 4), dotted(DurationKind::Half))
        .decresc()
        .spanner_anchor().hairpin_end()
        .end_barline();
    let bars = measures(&score);
    assert_eq!(bars[0].meta.actual_length, length(3, 4));
    let MeasureEvent::Spacer(anchor) = &bars[0].events[1] else { panic!("zero-duration endpoint") };
    assert_eq!(anchor.duration_log2, None);
    assert!(anchor.annotations.hairpin_end);
    let ticks = event::measure_timeline(&score.measures[0].events, &[], Some(length(3, 4)));
    assert_eq!(ticks.to_length(ticks.onsets[1]), length(3, 4));
    let page = page(&score);
    let layout = &page.systems[0].system.measures[0].layout;
    let spacer = layout.elements.iter().find(|el| matches!(el.element, MeasureElement::Spacer(_))).unwrap();
    assert_eq!((spacer.width, spacer.rod, spacer.spring), (0.0, 0.0, 0.0));
    let info = collect_hairpin_note_info(&page.systems[0].system);
    assert!(info[1].is_spacer && info[1].x > info[0].x);
    let svg = score.render_svg();
    let font = crate::font::bravura_font();
    let head = font.glyph_outline(smufl::Glyph::NoteheadHalf).unwrap().path_data;
    let rest = font.glyph_outline(smufl::Glyph::RestHalf).unwrap().path_data;
    assert_eq!(svg.matches(&head).count(), 1, "only the actual sounding half notehead");
    assert!(!svg.contains(&rest), "anchor must not draw a rest");
}

#[test]
fn intra_quarter_hairpin_end_differs_from_next_note_shared_column() {
    use crate::render::system_renderer::collect_hairpin_note_info;

    let build = |on_spacer: bool| {
        let mut score = ScoreBuilder::new()
            .time_signature(2, 4)
            .note(p(Note::E, 5), Duration::QTR).cresc()
            .note(p(Note::F, 5), Duration::QTR);
        if !on_spacer { score = score.hairpin_end(); }
        score = score.voice(1).spacer(Duration::EIGHTH)
            .spacer(Duration::EIGHTH);
        if on_spacer { score = score.hairpin_end(); }
        score.spacer(Duration::QTR).end_barline()
    };
    let inside = build(true);
    let next_note = build(false);
    let a = page(&inside);
    let b = page(&next_note);
    let sa = &a.systems[0].system;
    let sb = &b.systems[0].system;
    let a_end = collect_hairpin_note_info(sa).into_iter().find(|e| e.hairpin_end).unwrap();
    let b_end = collect_hairpin_note_info(sb).into_iter().find(|e| e.hairpin_end).unwrap();
    assert!(a_end.is_spacer && !b_end.is_spacer);
    let a_next_x = sa.measures[0].layout.elements.iter()
        .filter_map(|e| matches!(e.element, MeasureElement::Note(_)).then_some(e.x))
        .nth(1).unwrap() + sa.measures[0].x_offset;
    assert!(a_end.x < a_next_x, "1/8 is inside sustained quarter");
    assert_eq!(b_end.x, sb.measures[0].layout.elements.iter()
        .filter_map(|e| matches!(e.element, MeasureElement::Note(_)).then_some(e.x))
        .nth(1).unwrap() + sb.measures[0].x_offset);
    let font = crate::font::bravura_font();
    let padding = 0.3 * 250.0;
    let a_target = a_end.hairpin_end_x(&font, 250.0, Placement::Below).unwrap();
    let b_target = b_end.hairpin_end_x(&font, 250.0, Placement::Below).unwrap();
    assert!((a_target - (a_end.x - padding)).abs() < 1e-9);
    assert!(a_target < b_target, "shared-column endpoints must be numerically distinct");
    let svg = inside.render_svg();
    let rest = font.glyph_outline(smufl::Glyph::Rest8th).unwrap().path_data;
    assert!(!svg.contains(&rest));
}

#[test]
fn invisible_onset_endings_survive_system_break_with_visible_marks_only() {
    use crate::layout::dynamics::Dynamic;
    use crate::layout::text_spanner::TextSpanner;
    use crate::render::system_renderer::{
        collect_hairpin_note_info, collect_text_spanner_note_info,
    };
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .time_signature(2, 4)
        .note(p(Note::B, 4), Duration::HALF)
        .cresc()
        .text_spanner_start(TextSpanner::rit())
        .barline()
        .spacer(Duration::EIGHTH)
        .hairpin_end().text_spanner_end()
        .dynamic(Dynamic::Piano)
        .text_script(crate::layout::text_script::TextScript::above("fine"))
        .note(p(Note::C, 5), Duration::QTR)
        .spacer(Duration::EIGHTH)
        .end_barline();
    let page = page(&score);
    assert_eq!(page.systems.len(), 2);
    let target = &page.systems[1].system;
    let hp = collect_hairpin_note_info(target);
    let text = collect_text_spanner_note_info(target);
    assert!(hp[0].is_spacer && hp[0].hairpin_end);
    assert!(text[0].is_spacer && text[0].end);
    let x = target.measures[0].layout.elements.iter()
        .find_map(|e| matches!(e.element, MeasureElement::Spacer(_)).then_some(e.x)).unwrap()
        + target.measures[0].x_offset;
    assert_eq!(hp[0].x, x);
    assert_eq!(text[0].x, x);
    let svg = score.render_svg();
    assert!(svg.contains(">fine</text>") && svg.contains(">rit.</text>"));
    assert_eq!(svg.matches("stroke-dasharray").count(), 4, "text line on both systems and dashed incoming hairpin");
    let dashed: Vec<_> = svg.lines()
        .filter(|line| line.contains("<line ") && line.contains("stroke-dasharray"))
        .collect();
    let text_end = dashed.iter()
        .filter(|line| line_attr(line, "y1") == line_attr(line, "y2"))
        .map(|line| line_attr(line, "x2")).last().unwrap();
    let hairpin_end = dashed.iter()
        .filter(|line| line_attr(line, "y1") != line_attr(line, "y2"))
        .map(|line| line_attr(line, "x2")).last().unwrap();
    let expected_text = page.systems[1].x + x - 0.3 * 250.0;
    let expected_hairpin = page.systems[1].x
        + hp[0].hairpin_end_x(&crate::font::bravura_font(), 250.0, Placement::Below).unwrap();
    assert!((text_end - expected_text).abs() < 0.02, "text continuation ends at spacer onset");
    assert!((hairpin_end - expected_hairpin).abs() < 0.02, "wedge clears spacer dynamic at same onset");
    let rest = crate::font::bravura_font().glyph_outline(smufl::Glyph::Rest8th).unwrap().path_data;
    assert!(!svg.contains(&rest));
}

#[test]
#[should_panic(expected = "cannot attach to a multi-measure rest")]
fn unsupported_spanner_end_cannot_silently_disappear() {
    let _ = ScoreBuilder::new().multi_measure_rest(2).hairpin_end();
}

#[test]
#[should_panic(expected = "pitch-bound mark cannot attach to an invisible spacer")]
fn spacer_rejects_pitch_bound_attachment_instead_of_dropping_it() {
    let _ = ScoreBuilder::new().spacer(Duration::EIGHTH).tie();
}

#[test]
fn tupled_invisible_stop_preserves_performed_rational_onsets() {
    use crate::layout::group::scan_groups;
    use crate::layout::text_spanner::TextSpanner;
    let score = ScoreBuilder::new()
        .time_signature(2, 4)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::B, 4), Duration::QTR)
        .text_spanner_start(TextSpanner::dim())
        .spacer(Duration::EIGHTH).text_spanner_end()
        .spacer(Duration::EIGHTH)
        .end_tuplet()
        .note(p(Note::C, 5), Duration::QTR)
        .end_barline();
    assert_eq!(measures(&score)[0].meta.actual_length, length(7, 12));
    let ticks = event::measure_timeline(&score.measures[0].events, &[], Some(length(7, 12)));
    let actual = score.measures[0].events.iter().zip(ticks.onsets.iter())
        .find_map(|((_, event), &tick)| matches!(event,
            event::ScoreEvent::Spacer { annotations, .. } if annotations.text_spanner_end)
            .then_some(ticks.to_length(tick))).unwrap();
    assert_eq!(actual, length(1, 6));
    let page = page(&score);
    let layout = &page.systems[0].system.measures[0].layout;
    let stop_x = layout.elements.iter().find_map(|e| match &e.element {
        MeasureElement::Spacer(s) if s.annotations.text_spanner_end => Some(e.x),
        _ => None,
    }).unwrap();
    let next_note_x = layout.elements.iter().filter_map(|e|
        matches!(e.element, MeasureElement::Note(_)).then_some(e.x)).nth(1).unwrap();
    assert!(stop_x < next_note_x);
    let scan = scan_groups(layout.elements.iter().map(|e| &e.element));
    let note_index = layout.elements.iter().enumerate()
        .filter(|(_, e)| matches!(e.element, MeasureElement::Note(_)))
        .map(|(i, _)| i).nth(1).unwrap();
    assert!((scan.onsets[note_index] - 1.0 / 3.0).abs() < 1e-9,
        "the following note starts at performed third despite invisible tuplet members");
}

#[test]
#[should_panic(expected = "text_spanner_end requires a note")]
fn text_span_end_without_any_attachment_is_rejected() {
    let _ = ScoreBuilder::new().text_spanner_end();
}
