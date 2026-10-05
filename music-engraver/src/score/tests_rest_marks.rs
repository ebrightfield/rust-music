//! Builder → layout → SVG tests for marks on rests, text scripts, text
//! spanners, dynamics placement, barline marks and tempo-mark composition.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use smufl::Glyph;

use super::ScoreBuilder;
use crate::font::{bravura_font, MusicFont};
use crate::layout::dynamics::{CustomDynamic, Dynamic, DynamicMark};
use crate::layout::hairpin::HairpinType;
use crate::layout::lyric::LyricSyllable;
use crate::layout::measure::NoteAnnotations;
use crate::layout::placement::Placement;
use crate::layout::system::MeasureEvent;
use crate::layout::tempo::{MetronomeMark, MetronomeNoteKind, MetronomeUnit, TempoMark, TempoText};
use crate::layout::text_script::{estimate_text_width, TextFont, TextScript};
use crate::layout::text_spanner::{SpannerLine, TextSpanner};
use crate::svg_probe::{assert_close, glyph, glyphs, lines, text, texts, SvgLine};

const SS: f64 = 250.0;

fn font() -> MusicFont<'static> {
    bravura_font()
}

fn pitch(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn svg(builder: ScoreBuilder) -> String {
    builder.try_render_svg().unwrap()
}

/// Top staff line of the system holding a quarter rest drawn at `rest_y`
/// (quarter rests sit on the middle line, two staff spaces below the top).
fn top_line_from_quarter_rest(rest_y: f64) -> f64 {
    rest_y - 2.0 * SS
}

/// Annotations of the first rest in the built score.
fn first_rest_annotations(builder: &ScoreBuilder) -> NoteAnnotations {
    let mut b = builder.clone();
    b.flush_pending();
    b.build_measure_contents()
        .unwrap()
        .into_iter()
        .flat_map(|m| m.events)
        .find_map(|e| match e {
            MeasureEvent::Rest(r) => Some(r.annotations),
            _ => None,
        })
        .expect("a rest")
}

/// Wedge strokes: solid diagonal lines.
fn hairpin_lines(svg: &str) -> Vec<SvgLine> {
    lines(svg)
        .into_iter()
        .filter(|l| !l.dashed && l.x1 != l.x2 && l.y1 != l.y2)
        .collect()
}

#[test]
fn tempo_on_a_leading_rest_is_kept_and_drawn_from_the_rest() {
    let mark = TempoMark::metronome(
        MetronomeMark::bpm(MetronomeNoteKind::Quarter, 60)
            .approx()
            .parenthesized(),
    );
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(3, 4)
        .rest(Duration::QTR)
        .tempo(mark.clone())
        .rest(Duration::QTR)
        .note(pitch(Note::Ees, 4), Duration::QTR)
        .end_barline();
    assert_eq!(first_rest_annotations(&builder).tempo_mark, Some(mark));

    let out = svg(builder);
    let f = font();
    let rests = glyphs(&out, &f, Glyph::RestQuarter);
    assert_eq!(rests.len(), 2);
    let first_rest = rests[0];
    let note = glyph(&out, &f, Glyph::MetNoteQuarterUp);
    // "(" starts at the rest and ends exactly where the note glyph starts.
    let paren = text(&out, "(");
    assert_eq!(paren.anchor, "end");
    assert_close(paren.x, note.x);
    assert_close(
        note.x,
        first_rest.x + estimate_text_width("(", 1.6 * SS, TextFont::Bold),
    );
    assert_close(note.scale, 0.7);
    let value = text(&out, "= c. 60)");
    assert_eq!(value.anchor, "start");
    assert!(value.x > note.x);
    let baseline = top_line_from_quarter_rest(first_rest.y) - 2.8 * SS;
    assert_close(paren.y, baseline);
    assert_close(value.y, baseline);
    assert_close(note.y, baseline);
}

#[test]
fn dotted_range_tempo_composes_paren_note_dot_value_in_order() {
    let out = svg(ScoreBuilder::new()
        .rest(Duration::QTR)
        .tempo(TempoMark::metronome(
            MetronomeMark::range(MetronomeUnit::dotted(MetronomeNoteKind::Quarter), 58, 56)
                .approx()
                .parenthesized(),
        ))
        .end_barline());
    let f = font();
    let note = glyph(&out, &f, Glyph::MetNoteQuarterUp);
    let dot = glyph(&out, &f, Glyph::MetAugmentationDot);
    let paren = text(&out, "(");
    let value = text(&out, "= c. 58-56)");
    assert_close(paren.x, note.x);
    let note_advance = f64::from(f.glyph_advance(Glyph::MetNoteQuarterUp).unwrap()) * 0.7;
    assert_close(dot.x, note.x + note_advance + 0.15 * SS);
    assert!(value.x > dot.x);
    assert_close(dot.y, note.y);
}

#[test]
fn note_equals_note_equation_and_stacked_text() {
    let out = svg(ScoreBuilder::new()
        .note(pitch(Note::D, 5), Duration::QTR)
        .tempo(TempoMark::metronome(MetronomeMark::equation(
            MetronomeNoteKind::Quarter,
            MetronomeNoteKind::Eighth,
        )))
        .note(pitch(Note::C, 5), Duration::QTR)
        .tempo(
            TempoMark::metronome(MetronomeMark::bpm(MetronomeNoteKind::Half, 100).approx().parenthesized())
                .with_text_after(TempoText::upright("Monodia"))
                .with_text_below("Alla gavotta"),
        )
        .end_barline());
    let f = font();
    let quarter = glyph(&out, &f, Glyph::MetNoteQuarterUp);
    let eighth = glyph(&out, &f, Glyph::MetNote8thUp);
    let equals = text(&out, "=");
    assert!(quarter.x < equals.x && equals.x < eighth.x);
    assert_close(quarter.y, eighth.y);
    assert_close(equals.y, quarter.y);

    let half = glyph(&out, &f, Glyph::MetNoteHalfUp);
    let monodia = text(&out, "Monodia");
    assert_eq!(monodia.weight, "normal");
    assert_close(monodia.y, half.y);
    let gavotta = text(&out, "Alla gavotta");
    assert_eq!(gavotta.weight, "bold");
    // The stacked line sits on the default tempo baseline; the metronome
    // line one line-step (1.2 × 1.6 ss) above it, left-aligned together.
    assert_close(gavotta.y, quarter.y);
    assert_close(half.y, gavotta.y - 1.2 * 1.6 * SS);
    assert_close(gavotta.x, text(&out, "(").x - estimate_text_width("(", 1.6 * SS, TextFont::Bold));
}

#[test]
fn dynamics_on_rests_follow_the_dynamics_placement_and_hairpins_end_on_rests() {
    let out = svg(ScoreBuilder::new()
        .dynamics_placement(Placement::Above)
        .rest(Duration::QTR)
        .dynamic(Dynamic::Piano)
        .note(pitch(Note::B, 4), Duration::QTR)
        .hairpin_start(HairpinType::Decrescendo)
        .note(pitch(Note::A, 4), Duration::EIGHTH)
        .rest(Duration::EIGHTH)
        .hairpin_end()
        .dynamics_placement(Placement::Below)
        .rest(Duration::QTR)
        .dynamic(Dynamic::Pp)
        .end_barline());
    let f = font();
    let quarter_rests = glyphs(&out, &f, Glyph::RestQuarter);
    let (first, last) = (quarter_rests[0], quarter_rests[1]);
    let top = top_line_from_quarter_rest(first.y);
    let p = glyph(&out, &f, Glyph::DynamicPiano);
    let quarter_rest_advance = f64::from(f.glyph_advance(Glyph::RestQuarter).unwrap());
    assert_close(
        p.x,
        first.x + quarter_rest_advance / 2.0
            - f64::from(f.glyph_advance(Glyph::DynamicPiano).unwrap()) / 2.0,
    );
    assert_close(p.y, top - 1.3 * SS);
    let pp = glyph(&out, &f, Glyph::DynamicPp);
    assert_close(pp.y, top + 8.0 * SS / 2.0 + 2.5 * SS);
    assert!(pp.x > last.x - SS);

    // The decrescendo runs above the staff and stops 0.3 ss before the
    // eighth rest that ends it.
    let eighth_rest = glyph(&out, &f, Glyph::Rest8th);
    let wedge = hairpin_lines(&out);
    assert_eq!(wedge.len(), 2, "{wedge:?}");
    for stroke in &wedge {
        assert_close(stroke.x2, eighth_rest.x - 0.3 * SS);
    }
    let center = (wedge[0].y2 + wedge[1].y2) / 2.0;
    assert_close(center, top - 1.6 * SS);
}

#[test]
fn hairpin_starting_on_a_rest_begins_after_the_rest_glyph() {
    let out = svg(ScoreBuilder::new()
        .rest(Duration::EIGHTH)
        .cresc()
        .note(pitch(Note::C, 5), Duration::EIGHTH)
        .note(pitch(Note::D, 5), Duration::QTR)
        .hairpin_end()
        .end_barline());
    let f = font();
    let rest = glyph(&out, &f, Glyph::Rest8th);
    let advance = f64::from(f.glyph_advance(Glyph::Rest8th).unwrap());
    let wedge = hairpin_lines(&out);
    assert_eq!(wedge.len(), 2);
    for stroke in &wedge {
        assert_close(stroke.x1, rest.x + advance + 0.3 * SS);
    }
}

#[test]
fn custom_dynamic_on_a_rest_reaches_the_svg() {
    let piu_p = CustomDynamic::new().text("più").mark(Dynamic::Piano);
    let builder = ScoreBuilder::new()
        .rest(Duration::EIGHTH)
        .dynamic(piu_p.clone())
        .note(pitch(Note::F, 4), Duration::EIGHTH)
        .end_barline();
    assert_eq!(
        first_rest_annotations(&builder).dynamic,
        Some(DynamicMark::Custom(piu_p))
    );
    let out = svg(builder);
    let words = text(&out, "più");
    let p = glyph(&out, &font(), Glyph::DynamicPiano);
    assert_eq!(words.style, "italic");
    assert_close(words.y, p.y);
    assert!(words.x <= p.x);
}

#[test]
fn rit_text_spanner_crosses_a_system_break() {
    let out = svg(ScoreBuilder::new()
        .measures_per_system(1)
        .time_signature(2, 4)
        .note(pitch(Note::B, 4), Duration::QTR)
        .text_spanner_start(TextSpanner::rit())
        .note(pitch(Note::D, 5), Duration::QTR)
        .barline()
        .note(pitch(Note::Cis, 5), Duration::QTR)
        .note(pitch(Note::E, 5), Duration::QTR)
        .text_spanner_end()
        .end_barline());
    let rit = text(&out, "rit.");
    assert_eq!(rit.style, "italic");
    assert_eq!(texts(&out).iter().filter(|t| t.content == "rit.").count(), 1);
    let dashed: Vec<SvgLine> = lines(&out).into_iter().filter(|l| l.dashed).collect();
    assert_eq!(dashed.len(), 2, "{dashed:?}");
    let (head, tail) = (&dashed[0], &dashed[1]);
    // Head: on the label baseline, after the label, to the first system's
    // right edge. Tail: same height relative to the second system, ending
    // 0.3 ss before the E that stops the spanner.
    assert_close(head.y1, rit.y);
    assert!(head.x1 > rit.x);
    let staff_lines: Vec<SvgLine> = lines(&out)
        .into_iter()
        .filter(|l| !l.dashed && l.y1 == l.y2 && l.x2 - l.x1 > 20.0 * SS)
        .collect();
    let first_top = staff_lines.iter().map(|l| l.y1).fold(f64::INFINITY, f64::min);
    let second_top = staff_lines
        .iter()
        .map(|l| l.y1)
        .filter(|y| *y > first_top + 5.0 * SS)
        .fold(f64::INFINITY, f64::min);
    let first_right = staff_lines
        .iter()
        .filter(|l| l.y1 == first_top)
        .map(|l| l.x2)
        .fold(f64::NEG_INFINITY, f64::max);
    assert_close(head.x2, first_right);
    assert_close(head.y1, first_top - 1.6 * SS);
    assert_close(tail.y1, second_top - 1.6 * SS);
    let f = font();
    let last_head = glyphs(&out, &f, Glyph::NoteheadBlack)
        .into_iter()
        .filter(|g| g.y > (first_top + second_top) / 2.0)
        .map(|g| g.x)
        .fold(f64::NEG_INFINITY, f64::max);
    assert_close(tail.x2, last_head - 0.3 * SS);
    assert!(tail.x1 < tail.x2);
}

#[test]
fn dim_text_spanner_follows_the_dynamics_placement() {
    let out = svg(ScoreBuilder::new()
        .dynamics_placement(Placement::Above)
        .note(pitch(Note::B, 4), Duration::QTR)
        .dim_text()
        .note(pitch(Note::A, 4), Duration::QTR)
        .note(pitch(Note::G, 4), Duration::QTR)
        .note(pitch(Note::F, 4), Duration::QTR)
        .text_spanner_end()
        .end_barline());
    let dim = text(&out, "dim.");
    let f = font();
    let heads = glyphs(&out, &f, Glyph::NoteheadBlack);
    // B4 sits on the middle line, two staff spaces below the top line.
    let top = heads[0].y - 2.0 * SS;
    assert_close(dim.y, top - 1.6 * SS);
}

#[test]
fn solid_and_lineless_spanners_from_the_builder() {
    let out = svg(ScoreBuilder::new()
        .note(pitch(Note::B, 4), Duration::QTR)
        .text_spanner_start(TextSpanner::new("secco", SpannerLine::None, Placement::Below))
        .note(pitch(Note::A, 4), Duration::QTR)
        .text_spanner_end()
        .end_barline());
    text(&out, "secco");
    assert!(lines(&out).iter().all(|l| !l.dashed));
}

#[test]
fn text_scripts_and_marks_attach_to_rests_and_barlines() {
    let builder = ScoreBuilder::new()
        .text_mark(TextScript::above("1").small())
        .note(pitch(Note::C, 5), Duration::QTR)
        .barline()
        .text_mark(TextScript::glyph(Glyph::FermataAbove, Placement::Above))
        .rest(Duration::QTR)
        .text_script(TextScript::above("a)"))
        .end_barline();
    let mut b = builder.clone();
    b.flush_pending();
    let contents = b.build_measure_contents().unwrap();
    let MeasureEvent::Note(first) = &contents[0].events[0] else {
        panic!("first event must be the note");
    };
    assert_eq!(first.annotations.text_scripts, vec![TextScript::above("1").small()]);
    assert_eq!(
        first.annotations.text_marks,
        vec![TextScript::glyph(Glyph::FermataAbove, Placement::Above)]
    );
    let MeasureEvent::Rest(rest) = &contents[1].events[0] else {
        panic!("second measure must start with the rest");
    };
    assert_eq!(rest.annotations.text_scripts, vec![TextScript::above("a)")]);

    let out = svg(builder);
    let f = font();
    let rest_glyph = glyph(&out, &f, Glyph::RestQuarter);
    assert_close(text(&out, "a)").x, rest_glyph.x);
    // The fermata is centered on the barline between the measures: left of
    // the rest, right of the note.
    let fermata = glyph(&out, &f, Glyph::FermataAbove);
    let note = glyph(&out, &f, Glyph::NoteheadBlack);
    assert!(note.x < fermata.x && fermata.x < rest_glyph.x);
}

#[test]
fn pitch_bound_builders_after_a_rest_attach_nowhere() {
    let builder = ScoreBuilder::new()
        .note(pitch(Note::C, 5), Duration::QTR)
        .rest(Duration::QTR)
        .tie()
        .slur_start()
        .lyric(LyricSyllable::word("la"))
        .end_barline();
    let mut b = builder.clone();
    b.flush_pending();
    let contents = b.build_measure_contents().unwrap();
    for event in &contents[0].events {
        let annotations = match event {
            MeasureEvent::Note(n) => &n.annotations,
            MeasureEvent::Rest(r) => &r.annotations,
            other => panic!("unexpected {other:?}"),
        };
        assert!(!annotations.tie_forward);
        assert!(!annotations.slur_start);
        assert!(annotations.lyric.is_none());
    }
}
