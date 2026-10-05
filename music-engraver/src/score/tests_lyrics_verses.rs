use super::*;
use crate::layout::group::TupletSpec;
use crate::layout::lyric::{verse_baseline, LyricContinuation, LyricStyle, LyricSyllable};
use crate::layout::measure::{MeasureElement, MeasureLayoutConfig};
use crate::layout::staff::StaffLayout;
use crate::layout::system::{layout_system, MeasureEvent};
use crate::render::system_renderer::collect_lyric_note_info;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;

fn p(note: Note) -> Pitch {
    Pitch::new(note, 4)
}

fn attribute(element: &str, name: &str) -> f64 {
    element
        .split_once(&format!("{name}=\""))
        .unwrap_or_else(|| panic!("missing {name} in {element}"))
        .1
        .split('"')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn text_element<'a>(svg: &'a str, text: &str) -> &'a str {
    svg.lines()
        .find(|line| line.contains(&format!(">{text}</text>")))
        .unwrap_or_else(|| panic!("missing lyric {text}"))
}

fn lyric_line(svg: &str, y: f64) -> (f64, f64) {
    let lines: Vec<_> = svg
        .lines()
        .filter(|line| line.contains("<line ") && (attribute(line, "y1") - y).abs() < 0.001)
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one extender on verse baseline {y}"
    );
    (attribute(lines[0], "x1"), attribute(lines[0], "x2"))
}

#[test]
fn two_verses_have_independent_hyphens_extender_skip_and_font_shape() {
    // mn-c08-r002: Swedish/German underlay over triplets; mn-c12-r002/r003:
    // italic German with an independent `_` skip. mn-c04-r005 hides one hyphen.
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::C), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("Jag"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::with_extender("Ich"), LyricStyle::Italic)
        .note(p(Note::F), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("är"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .note(p(Note::B), Duration::EIGHTH)
        .lyric_verse(
            1,
            LyricSyllable::with_hidden_hyphen("Un"),
            LyricStyle::Upright,
        )
        .end_tuplet()
        .note(p(Note::D), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("zu"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("bin"), LyricStyle::Italic)
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let rhythmic: Vec<_> = contents[0]
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(&note.annotations.lyrics),
            _ => None,
        })
        .collect();
    assert_eq!(rhythmic.len(), 4); // Tuplet marks do not consume lyric anchors.
    assert!(rhythmic[1][1].syllable.skip);
    assert_eq!(
        rhythmic[2][0].syllable.continuation,
        LyricContinuation::HyphenHidden
    );
    let svg = score.render_svg();
    let swedish = text_element(&svg, "Jag");
    let german = text_element(&svg, "Ich");
    assert!(swedish.contains("font-style=\"normal\""));
    assert!(german.contains("font-style=\"italic\""));
    assert_eq!(attribute(german, "x"), attribute(swedish, "x"));
    assert!((attribute(german, "y") - attribute(swedish, "y") - 2.0 * 250.0).abs() < 0.01);
    assert!(!svg.contains(">Un-<"));
    assert_eq!(
        svg.matches(">-</text>").count(),
        1,
        "only Jag/är receives a visible hyphen; Un/zu is hidden"
    );
    let (from, to) = lyric_line(&svg, attribute(german, "y"));
    assert!(
        from > attribute(german, "x") - 200.0 && to < attribute(text_element(&svg, "bin"), "x")
    );
}

#[test]
fn c12_r003_mixed_tuplets_keep_verse_skips_without_false_melisma() {
    // mn-c12-r003: r4 { d'4 d'8 } { cis8 cis8 cis8 } with Swedish
    // \"Yohy -- _ o.\" and italic German \"Yohyo, _ in der ...\".
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .rest(Duration::QTR)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::D), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("Du,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Du,"), LyricStyle::Italic)
        .note(p(Note::D), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("Yohy"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Yohyo,"), LyricStyle::Italic)
        .end_tuplet()
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(Pitch::new(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::skip(), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .note(Pitch::new(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("o."), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("in"), LyricStyle::Italic)
        .note(Pitch::new(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("I"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("der"), LyricStyle::Italic)
        .end_tuplet()
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let cfg = MeasureLayoutConfig::from_staff_space(250.0);
    let layout = layout_system(&score.build_prefix(), &contents, &cfg, Some(40.0 * 250.0));
    let notes = collect_lyric_note_info(&layout, 250.0);
    assert_eq!(notes.len(), 5);
    let svg = score.render_svg();
    let staff = StaffLayout::new(0.0, 0.0, layout.staff_width, 250.0);
    let underscore_count = svg
        .lines()
        .filter(|line| {
            line.contains("<line ")
                && (attribute(line, "y1") - verse_baseline(&staff, 250.0, 2)).abs() < 0.001
        })
        .count();
    assert_eq!(
        underscore_count, 0,
        "German '_' skips an event; it does not print a melisma"
    );
    let hyphen = text_element(&svg, "-");
    assert!(attribute(hyphen, "x") > notes[1].x && attribute(hyphen, "x") < notes[3].x);
    assert_eq!(
        svg.matches(">-</text>").count(),
        1,
        "only Swedish uses a hyphen"
    );
    assert!(text_element(&svg, "der").contains("font-style=\"italic\""));
}

#[test]
fn c12_r002_unbeamed_rest_triplet_keeps_german_skip_on_its_own_verse() {
    // mn-c12-r002 opening: r4 { r8 b8 b8 } d'4 cis8 cis8,
    // \"Jag är rädd, So -- do,\" / \"Ich hab Angst, Sodo. _\".
    let score = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .begin_tuplet(TupletSpec::new(3, 2))
        .rest(Duration::EIGHTH)
        .note(Pitch::new(Note::B, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("Jag"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Ich"), LyricStyle::Italic)
        .note(Pitch::new(Note::B, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("är"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("hab"), LyricStyle::Italic)
        .end_tuplet()
        .note(p(Note::D), Duration::QTR)
        .lyric_verse(1, LyricSyllable::word("rädd,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Angst,"), LyricStyle::Italic)
        .note(Pitch::new(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_hyphen("So"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("Sodo."), LyricStyle::Italic)
        .note(Pitch::new(Note::Cis, 3), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("do,"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::skip(), LyricStyle::Italic)
        .end_barline();
    let svg = score.clone().render_svg();
    let sodo = text_element(&svg, "Sodo.");
    let do_swedish = text_element(&svg, "do,");
    assert!(sodo.contains("font-style=\"italic\""));
    assert!(attribute(do_swedish, "x") > attribute(sodo, "x"));
    assert_eq!(svg.matches(">-</text>").count(), 1);
    let contents = score.build_measure_contents().unwrap();
    let events: Vec<_> = contents[0]
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(&note.annotations.lyrics),
            _ => None,
        })
        .collect();
    assert_eq!(
        events.len(),
        5,
        "the triplet rest cannot acquire an underlay"
    );
    assert!(events.last().unwrap()[1].syllable.skip);
}

#[test]
fn grouped_chord_and_beamed_tuplet_lyrics_span_exact_member_positions() {
    // mn-c11-r028 has beamed notes nested in triplets; substituting one
    // chord here probes the same member annotation path (the source has no chord).
    let score = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::with_extender("Så"), LyricStyle::Upright)
        .chord(vec![p(Note::D), p(Note::F)], Duration::EIGHTH)
        .lyric_verse(2, LyricSyllable::with_hyphen("So"), LyricStyle::Italic)
        .note(p(Note::E), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("ta"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("sprach"), LyricStyle::Italic)
        .end_beam()
        .end_tuplet()
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let cfg = MeasureLayoutConfig::from_staff_space(250.0);
    let system = layout_system(&score.build_prefix(), &contents, &cfg, Some(40.0 * 250.0));
    let notes = collect_lyric_note_info(&system, 250.0);
    assert_eq!(notes.len(), 3);
    assert!(matches!(
        system.measures[0]
            .layout
            .elements
            .iter()
            .find(|element| matches!(&element.element, MeasureElement::Chord(_)))
            .unwrap()
            .element,
        MeasureElement::Chord(_)
    ));
    let svg = score.render_svg();
    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, 250.0);
    let (from, to) = lyric_line(&svg, verse_baseline(&staff, 250.0, 1));
    let source_edge = notes[0].x
        + crate::render::lyric_renderer::lyric_text_half_width("Så", LyricStyle::Upright, 250.0)
        + 0.15 * 250.0;
    let target_edge = notes[2].x
        - crate::render::lyric_renderer::lyric_text_half_width("ta", LyricStyle::Upright, 250.0)
        - 0.15 * 250.0;
    assert!(
        (from - source_edge).abs() < 0.01,
        "melisma must start just past the first member's syllable"
    );
    assert!(
        (to - target_edge).abs() < 0.01,
        "melisma must end just before the third member's syllable"
    );
    let hyphen = text_element(&svg, "-");
    assert!(attribute(hyphen, "x") > notes[1].x && attribute(hyphen, "x") < notes[2].x);
    assert!((attribute(hyphen, "y") - verse_baseline(&staff, 250.0, 2)).abs() < 0.01);
}

#[test]
fn associated_voice_survives_builder_reordering_and_barline_with_cue_notes() {
    // mn-c11-r029 switches associatedVoice to the small-note alt voice and
    // then back; verse 1 remains anchored to the melody throughout.
    let make = |secondary_first: bool| {
        let base = ScoreBuilder::new()
            .measures_per_system(1)
            .lyric_associated_voice(1, 0)
            .lyric_associated_voice(2, 1);
        let base = if secondary_first {
            base.voice(1)
                .note(p(Note::G), Duration::QTR)
                .note_size(NoteSize::Cue)
                .voice(0)
                .note(p(Note::E), Duration::QTR)
        } else {
            base.voice(0)
                .note(p(Note::E), Duration::QTR)
                .voice(1)
                .note(p(Note::G), Duration::QTR)
                .note_size(NoteSize::Cue)
        };
        base.lyric_verse(1, LyricSyllable::word("ment"), LyricStyle::Upright)
            .lyric_verse(2, LyricSyllable::with_hyphen("zu"), LyricStyle::Italic)
            .barline()
            .voice(0)
            .note(p(Note::F), Duration::QTR)
            .voice(1)
            .note(p(Note::A), Duration::QTR)
            .note_size(NoteSize::Cue)
            .lyric_verse(2, LyricSyllable::word("sam"), LyricStyle::Italic)
            .lyric_verse_on_voice(0, 1, LyricSyllable::word("in"), LyricStyle::Upright)
            .end_barline()
    };
    for score in [make(false), make(true)] {
        let contents = score.build_measure_contents().unwrap();
        for (measure, expected_primary, expected_secondary) in [(0, "ment", "zu"), (1, "in", "sam")]
        {
            let primary = match &contents[measure].events[0] {
                MeasureEvent::Note(note) => note,
                _ => panic!("missing primary note"),
            };
            let secondary = match &contents[measure].additional_voices[0][0] {
                MeasureEvent::Note(note) => note,
                _ => panic!("missing cue note"),
            };
            assert_eq!(
                primary.annotations.lyrics[0].syllable.text,
                expected_primary
            );
            assert_eq!(
                secondary.annotations.lyrics[0].syllable.text,
                expected_secondary
            );
            assert_eq!(secondary.annotations.size, NoteSize::Cue);
        }
        let mut layout_builder = score.clone();
        assert_eq!(
            layout_builder
                .page_layout(250.0)
                .unwrap()
                .unwrap()
                .systems
                .len(),
            2
        );
        let svg = score.render_svg();
        assert!(text_element(&svg, "zu").contains("font-style=\"italic\""));
        assert!(
            attribute(text_element(&svg, "ment"), "y") < attribute(text_element(&svg, "zu"), "y")
        );
    }
}

#[test]
fn cross_system_melisma_waits_through_rest_and_skip_for_tuplet_member() {
    let score = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::C), Duration::QTR)
        .lyric_verse(
            1,
            LyricSyllable::with_extender("länge"),
            LyricStyle::Upright,
        )
        .lyric_verse(2, LyricSyllable::with_hyphen("In"), LyricStyle::Italic)
        .barline()
        .begin_tuplet(TupletSpec::new(3, 2))
        .rest(Duration::EIGHTH)
        .note(p(Note::D), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::skip(), LyricStyle::Upright)
        .note(p(Note::E), Duration::EIGHTH)
        .lyric_verse(1, LyricSyllable::word("genug"), LyricStyle::Upright)
        .lyric_verse(2, LyricSyllable::word("stru"), LyricStyle::Italic)
        .end_tuplet()
        .end_barline();
    let page = score.clone().page_layout(250.0).unwrap().unwrap();
    assert_eq!(page.systems.len(), 2);
    let target = collect_lyric_note_info(&page.systems[1].system, 250.0);
    assert_eq!(target.len(), 2, "triplet rest is not a lyric anchor");
    assert!(target[0].lyrics[0].syllable.skip);
    let svg = score.render_svg();
    let dst = &page.systems[1];
    let staff = StaffLayout::new(dst.x, dst.y, dst.system.staff_width, 250.0);
    let (_start, end) = lyric_line(&svg, verse_baseline(&staff, 250.0, 1));
    let expected_end = dst.x + target[1].x
        - crate::render::lyric_renderer::lyric_text_half_width("genug", LyricStyle::Upright, 250.0)
        - 0.15 * 250.0;
    assert!(
        (end - expected_end).abs() < 0.01,
        "incoming melisma must stop before the second pitched triplet member, not its rest/skip"
    );
    let source_hyphen = text_element(&svg, "-");
    assert!(source_hyphen.contains("font-style=\"italic\""));
}

#[test]
fn multistaff_cross_system_hyphen_remains_on_its_verse_and_stave() {
    let upper = ScoreBuilder::new()
        .measures_per_system(1)
        .note(p(Note::C), Duration::QTR)
        .lyric_verse(2, LyricSyllable::with_hyphen("Dorf"), LyricStyle::Italic)
        .barline()
        .note(p(Note::D), Duration::QTR)
        .lyric_verse(2, LyricSyllable::word("neben"), LyricStyle::Italic)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .measures_per_system(1)
        .note(p(Note::F), Duration::QTR)
        .barline()
        .note(p(Note::G), Duration::QTR)
        .end_barline();
    let svg = crate::score::multi_staff::MultiStaffScore::grand_staff(upper, lower)
        .measures_per_system(1)
        .render_svg();
    assert!(svg.contains(">Dorf</text>") && svg.contains(">neben</text>"));
    assert_eq!(
        svg.matches(">-</text>").count(),
        2,
        "cross-system hyphen has a trailing and leading half on the upper stave"
    );
    let german = text_element(&svg, "Dorf");
    assert!(german.contains("font-style=\"italic\""));
    let hyphens: Vec<_> = svg
        .lines()
        .filter(|line| line.contains(">-</text>"))
        .collect();
    assert_eq!(hyphens.len(), 2);
    for (hyphen, word) in hyphens.iter().zip(["Dorf", "neben"]) {
        assert!(hyphen.contains("font-style=\"italic\""));
        assert!(
            (attribute(hyphen, "y") - attribute(text_element(&svg, word), "y")).abs() < 0.01,
            "each half-hyphen must follow verse 2 of its own system's upper stave"
        );
    }
}

#[test]
fn multistaff_reserves_third_verse_clearance_before_lower_stave() {
    let upper = ScoreBuilder::new()
        .note(p(Note::C), Duration::QTR)
        .lyric_verse(3, LyricSyllable::word("langen"), LyricStyle::Italic)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .note(Pitch::new(Note::G, 2), Duration::QTR)
        .end_barline();
    let svg = crate::score::multi_staff::MultiStaffScore::grand_staff(upper, lower).render_svg();
    let without_lyrics = crate::score::multi_staff::MultiStaffScore::grand_staff(
        ScoreBuilder::new()
            .note(p(Note::C), Duration::QTR)
            .end_barline(),
        ScoreBuilder::new()
            .clef(Clef::Bass)
            .note(Pitch::new(Note::G, 2), Duration::QTR)
            .end_barline(),
    )
    .render_svg();
    let lower_staff_top = |svg: &str| {
        let mut staff_lines: Vec<f64> = svg
            .lines()
            .filter(|line| {
                line.contains("<line ")
                    && (attribute(line, "x2") - attribute(line, "x1") - 40.0 * 250.0).abs() < 0.001
            })
            .map(|line| attribute(line, "y1"))
            .collect();
        staff_lines.sort_by(f64::total_cmp);
        assert!(staff_lines.len() >= 10, "expected two five-line staves");
        staff_lines[5]
    };
    let lyric_y = attribute(text_element(&svg, "langen"), "y");
    let lower_top = lower_staff_top(&svg);
    assert!(
        lower_top > lower_staff_top(&without_lyrics),
        "verse 3 must dynamically expand the upper-to-lower stave gap"
    );
    assert!(
        lower_top > lyric_y + 0.22 * 1.4 * 250.0 + 0.5 * 250.0,
        "third-verse descent and half-space clearance must fit above the lower staff"
    );
}

#[test]
fn long_italic_syllables_keep_an_incompressible_text_gap() {
    let score = ScoreBuilder::new()
        .system_width_fu(6.0 * 250.0)
        .note(p(Note::C), Duration::EIGHTH)
        .lyric_verse(2, LyricSyllable::word("Schleier"), LyricStyle::Italic)
        .note(p(Note::D), Duration::EIGHTH)
        .lyric_verse(2, LyricSyllable::word("Sternen"), LyricStyle::Italic)
        .end_barline();
    let svg = score.render_svg();
    let first = attribute(text_element(&svg, "Schleier"), "x");
    let second = attribute(text_element(&svg, "Sternen"), "x");
    let width = |word| {
        crate::layout::text_script::estimate_text_width(word, 1.4 * 250.0, LyricStyle::Italic)
    };
    let required = (width("Schleier") + width("Sternen")) * 0.5 + 0.4 * 250.0;
    assert!(
        second - first >= required,
        "serif text collision: actual separation {}, required {required}",
        second - first
    );
}
