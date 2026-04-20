//! Fixture-based round-trip: parse a fragment, re-render via music's
//! `ToLilypondString`, and check byte-for-byte equality with the fixture.
//!
//! Fixtures live under `tests/fixtures/*.ly`. Each one is a bare sequence of
//! voice-level elements (no outer `\score` or `{}` block), formatted exactly
//! as `music` emits them, so "parse then re-render" is a true syntactic
//! identity.

use lilypond_parser::{parse, Event, Item};
use music::notation::lilypond::ToLilypondString;
use music::notation::rhythm::duration::Duration;
use music::Pitch;

fn render_pitch(p: &Pitch) -> String {
    p.to_lilypond_string()
}

fn render_duration(d: &Duration) -> String {
    d.to_lilypond_string()
}

fn render_event(ev: &Event) -> String {
    match ev {
        Event::Note(p, d) => format!("{}{}", render_pitch(p), render_duration(d)),
        Event::Rest(d) => format!("r{}", render_duration(d)),
        Event::Chord(ps, d) => {
            let inner = ps.iter().map(render_pitch).collect::<Vec<_>>().join(" ");
            format!("<{}>{}", inner, render_duration(d))
        }
    }
}

fn render_item(item: &Item) -> String {
    match item {
        Item::Event(ev) => render_event(ev),
        Item::Clef(name) => format!("\\clef {}", name),
        Item::Time(n, d) => format!("\\time {}/{}", n, d),
        Item::Key(tonic, mode) => format!("\\key {} \\{}", tonic, mode),
        Item::Block(items) => format!("{{ {} }}", render_items(items)),
        Item::Tuplet { numerator, denominator, items } => {
            format!("\\tuplet {}/{} {{ {} }}", numerator, denominator, render_items(items))
        }
    }
}

fn render_items(items: &[Item]) -> String {
    items.iter().map(render_item).collect::<Vec<_>>().join(" ")
}

fn roundtrip(src: &str) -> String {
    let items = parse(src).unwrap_or_else(|e| panic!("parse failed: {:?}\nsource: {:?}", e, src));
    render_items(&items)
}

fn read_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {:?}: {}", path, e))
        .trim_end_matches('\n')
        .to_string()
}

fn assert_fixture_roundtrip(name: &str) {
    let src = read_fixture(name);
    let got = roundtrip(&src);
    assert_eq!(
        got, src,
        "\nfixture {} did not round-trip\n  expected: {:?}\n  got:      {:?}",
        name, src, got
    );
}

#[test]
fn scale_c_major() {
    assert_fixture_roundtrip("scale_c_major.ly");
}

#[test]
fn c_major_triad() {
    assert_fixture_roundtrip("c_major_triad.ly");
}

#[test]
fn mixed_rhythm() {
    assert_fixture_roundtrip("mixed_rhythm.ly");
}

#[test]
fn accidentals() {
    assert_fixture_roundtrip("accidentals.ly");
}

#[test]
fn tuplet_triplet() {
    assert_fixture_roundtrip("tuplet_triplet.ly");
}

#[test]
fn progression() {
    assert_fixture_roundtrip("progression.ly");
}

#[test]
fn chromatic_scale() {
    assert_fixture_roundtrip("chromatic_scale.ly");
}

#[test]
fn wide_range() {
    assert_fixture_roundtrip("wide_range.ly");
}

#[test]
fn all_durations() {
    assert_fixture_roundtrip("all_durations.ly");
}

#[test]
fn dotted_durations() {
    assert_fixture_roundtrip("dotted_durations.ly");
}

#[test]
fn rests_and_notes() {
    assert_fixture_roundtrip("rests_and_notes.ly");
}

#[test]
fn double_accidentals() {
    assert_fixture_roundtrip("double_accidentals.ly");
}

#[test]
fn enharmonic_edges() {
    assert_fixture_roundtrip("enharmonic_edges.ly");
}

#[test]
fn seventh_chords() {
    assert_fixture_roundtrip("seventh_chords.ly");
}

#[test]
fn inverted_chords() {
    assert_fixture_roundtrip("inverted_chords.ly");
}

#[test]
fn nested_tuplets() {
    assert_fixture_roundtrip("nested_tuplets.ly");
}

#[test]
fn mixed_chords_and_notes() {
    assert_fixture_roundtrip("mixed_chords_and_notes.ly");
}

#[test]
fn bass_line() {
    assert_fixture_roundtrip("bass_line.ly");
}

#[test]
fn arpeggio() {
    assert_fixture_roundtrip("arpeggio.ly");
}

#[test]
fn jazz_ii_v_i() {
    assert_fixture_roundtrip("jazz_ii_v_i.ly");
}

/// Cross-check: the fixtures are in the format that `music`'s Pitch/Duration
/// would emit. This test re-renders from scratch using music's output side
/// and confirms it matches the fixture too (so fixtures aren't just
/// parser-self-consistent, they agree with the canonical emitter).
#[test]
fn c_major_triad_matches_music_output() {
    use music::{Note, Voicing};
    use music::notation::rhythm::duration::DurationKind;

    let voicing = Voicing::new(vec![
        Pitch::new(Note::C, 4),
        Pitch::new(Note::E, 4),
        Pitch::new(Note::G, 4),
    ]);
    let dur = Duration::new(DurationKind::Qtr, 0);
    let rendered = format!("{}{}", voicing.to_lilypond_string(), dur.to_lilypond_string());
    assert_eq!(rendered, read_fixture("c_major_triad.ly"));
}
