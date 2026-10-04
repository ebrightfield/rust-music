//! Accidental state (key-signature baseline overridden by in-measure state,
//! resolved staff-wide in musical order) and caller display policies
//! (`Auto` / `Force` / `Cautionary`), through the score model, measure
//! layout, and public SVG rendering.

use super::*;
use crate::layout::accidental::{ACCIDENTAL_COLUMN_GAP_SS, ACCIDENTAL_NOTEHEAD_PADDING_SS};
use crate::layout::measure::{layout_measure, NoteEvent};
use crate::layout::system::measure_event_to_element;
use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;
use smufl::Glyph;

const Q: Duration = Duration::QTR;
const E: Duration = Duration::EIGHTH;
const SHARP: Glyph = Glyph::AccidentalSharp;
const FLAT: Glyph = Glyph::AccidentalFlat;
const NATURAL: Glyph = Glyph::AccidentalNatural;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn plain(glyph: Glyph) -> Option<ResolvedAccidental> {
    Some(ResolvedAccidental::plain(glyph))
}

fn cautionary(glyph: Glyph) -> Option<ResolvedAccidental> {
    Some(ResolvedAccidental::cautionary(glyph))
}

fn member_accidentals(note: &NoteEvent) -> Vec<Option<ResolvedAccidental>> {
    note.annotations
        .grouped_chord
        .as_ref()
        .map_or_else(|| vec![note.accidental], |chord| chord.accidentals.clone())
}

/// Every pitch's resolved accidental in a voice, in notation order.
fn voice_accidentals(events: &[MeasureEvent]) -> Vec<Option<ResolvedAccidental>> {
    events
        .iter()
        .flat_map(|event| match event {
            MeasureEvent::Note(note) => vec![note.accidental],
            MeasureEvent::Chord(chord) => chord.accidentals.clone(),
            MeasureEvent::BeamGroup(group) => {
                group.notes.iter().flat_map(member_accidentals).collect()
            }
            MeasureEvent::TupletGroup(tuplet) => tuplet
                .beam_group
                .notes
                .iter()
                .flat_map(member_accidentals)
                .collect(),
            MeasureEvent::Rest(_) | MeasureEvent::MultiMeasureRest { .. } => Vec::new(),
        })
        .collect()
}

/// Primary-voice accidentals of every measure.
fn measure_accidentals(builder: &ScoreBuilder) -> Vec<Vec<Option<ResolvedAccidental>>> {
    builder
        .build_measure_contents()
        .iter()
        .map(|measure| voice_accidentals(&measure.events))
        .collect()
}

/// Accidentals of every voice of the first measure, voice 0 first.
fn first_measure_voice_accidentals(builder: &ScoreBuilder) -> Vec<Vec<Option<ResolvedAccidental>>> {
    let measure = &builder.build_measure_contents()[0];
    std::iter::once(&measure.events)
        .chain(&measure.additional_voices)
        .map(|events| voice_accidentals(events))
        .collect()
}

/// `(x, y)` of every placement of `glyph` in a rendered SVG, in document order.
fn placements(svg: &str, glyph: Glyph) -> Vec<(f64, f64)> {
    let path = bravura_font().glyph_outline(glyph).unwrap().path_data;
    let needle = format!("<path d=\"{path}\" fill=\"black\" transform=\"translate(");
    svg.match_indices(&needle)
        .map(|(start, _)| {
            let (xy, _) = svg[start + needle.len()..].split_once(')').unwrap();
            let (x, y) = xy.split_once(", ").unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

fn advance(glyph: Glyph) -> f64 {
    bravura_font().glyph_advance(glyph).unwrap() as f64
}

// --- key-signature baseline overridden by in-measure state ---

#[test]
fn g_major_natural_reinstated_sharp_then_suppressed_repeat() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note(p(Note::F, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL), plain(SHARP), None]]
    );
}

#[test]
fn b_flat_major_natural_reinstated_flat_then_suppressed_repeat() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Flats(2))
        .note(p(Note::B, 4), Q)
        .note(p(Note::Bes, 4), Q)
        .note(p(Note::Bes, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL), plain(FLAT), None]]
    );
}

#[test]
fn opposing_accidentals_are_followed_by_the_reinstated_key_signature_accidental() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note(p(Note::Fisis, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .barline()
        .note(p(Note::Fes, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![
            vec![plain(Glyph::AccidentalDoubleSharp), plain(SHARP)],
            vec![plain(FLAT), plain(SHARP)],
        ]
    );

    let flat_key = ScoreBuilder::new()
        .key_signature(KeySignature::Flats(2))
        .note(p(Note::Beses, 4), Q)
        .note(p(Note::Bes, 4), Q)
        .note(p(Note::Eis, 4), Q)
        .note(p(Note::Ees, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&flat_key),
        vec![vec![
            plain(Glyph::AccidentalDoubleFlat),
            plain(FLAT),
            plain(SHARP),
            plain(FLAT),
        ]]
    );
}

#[test]
fn the_key_signature_is_the_next_measures_baseline() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note(p(Note::F, 4), Q)
        .barline()
        .note(p(Note::Fis, 4), Q)
        .note(p(Note::F, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL)], vec![None, plain(NATURAL)]]
    );
}

#[test]
fn every_alteration_is_compared_with_the_active_one_symmetrically() {
    // D is sharp in four sharps and flat in four flats.
    let d = |note| p(note, 4);
    let cases: [(KeySignature, &[Note], Note, Option<Glyph>); 12] = [
        (KeySignature::Sharps(4), &[], Note::Dis, None),
        (KeySignature::Sharps(4), &[], Note::D, Some(NATURAL)),
        (
            KeySignature::Sharps(4),
            &[],
            Note::Disis,
            Some(Glyph::AccidentalDoubleSharp),
        ),
        (
            KeySignature::Sharps(4),
            &[],
            Note::Deses,
            Some(Glyph::AccidentalDoubleFlat),
        ),
        (KeySignature::Sharps(4), &[Note::D], Note::Dis, Some(SHARP)),
        (
            KeySignature::Sharps(4),
            &[Note::Disis],
            Note::Dis,
            Some(SHARP),
        ),
        (
            KeySignature::Sharps(4),
            &[Note::Des],
            Note::Dis,
            Some(SHARP),
        ),
        (KeySignature::Sharps(4), &[Note::Disis], Note::Disis, None),
        (KeySignature::Flats(4), &[], Note::Des, None),
        (
            KeySignature::Flats(4),
            &[Note::Deses],
            Note::Des,
            Some(FLAT),
        ),
        (KeySignature::Flats(4), &[Note::D], Note::Des, Some(FLAT)),
        (KeySignature::Open, &[Note::Dis], Note::D, Some(NATURAL)),
    ];
    for (key, earlier, note, expected) in cases {
        let mut seen = AccidentalTracker::new();
        for &previous in earlier {
            convert_event(
                &ScoreEvent::Note {
                    pitch: d(previous),
                    duration: Q,
                    annotations: NoteAnnotations::default(),
                },
                &Clef::Treble,
                &key,
                Some(&mut seen),
            );
        }
        let resolved = resolve_accidental(&d(note), &key, AccidentalDisplay::Auto, Some(&seen));
        assert_eq!(
            resolved.map(|accidental| accidental.glyph),
            expected,
            "{note:?} after {earlier:?} in {key:?}"
        );
    }
}

#[test]
fn convert_event_tracks_reinstatement_across_events() {
    let key = KeySignature::Sharps(1);
    let mut seen = AccidentalTracker::new();
    let glyphs: Vec<_> = [Note::F, Note::Fis, Note::Fis]
        .into_iter()
        .map(|note| {
            let event = ScoreEvent::Note {
                pitch: p(note, 4),
                duration: Q,
                annotations: NoteAnnotations::default(),
            };
            match convert_event(&event, &Clef::Treble, &key, Some(&mut seen)) {
                MeasureEvent::Note(note) => note.accidental,
                other => panic!("expected a note, got {other:?}"),
            }
        })
        .collect();
    assert_eq!(glyphs, vec![plain(NATURAL), plain(SHARP), None]);
}

#[test]
fn reinstatement_inside_beam_groups() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .beam_group(vec![
            (p(Note::F, 4), E),
            (p(Note::Fis, 4), E),
            (p(Note::Fis, 4), E),
        ])
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL), plain(SHARP), None]]
    );

    let flat_key = ScoreBuilder::new()
        .key_signature(KeySignature::Flats(2))
        .beam_group(vec![
            (p(Note::B, 4), E),
            (p(Note::Bes, 4), E),
            (p(Note::Bes, 4), E),
        ])
        .end_barline();
    assert_eq!(
        measure_accidentals(&flat_key),
        vec![vec![plain(NATURAL), plain(FLAT), None]]
    );
}

#[test]
fn reinstatement_inside_tuplets() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .tuplet_ratio(
            3,
            2,
            vec![
                (p(Note::F, 4), E),
                (p(Note::Fis, 4), E),
                (p(Note::Fis, 4), E),
            ],
        )
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL), plain(SHARP), None]]
    );

    let flat_key = ScoreBuilder::new()
        .key_signature(KeySignature::Flats(2))
        .tuplet_ratio(
            3,
            2,
            vec![
                (p(Note::B, 4), E),
                (p(Note::Bes, 4), E),
                (p(Note::Bes, 4), E),
            ],
        )
        .end_barline();
    assert_eq!(
        measure_accidentals(&flat_key),
        vec![vec![plain(NATURAL), plain(FLAT), None]]
    );
}

#[test]
fn reinstatement_across_chords() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .chord(vec![p(Note::F, 4), p(Note::A, 4)], Q)
        .chord(vec![p(Note::Fis, 4), p(Note::A, 4)], Q)
        .chord(vec![p(Note::A, 4), p(Note::Fis, 4)], Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(NATURAL), None, plain(SHARP), None, None, None]]
    );

    let flat_key = ScoreBuilder::new()
        .key_signature(KeySignature::Flats(2))
        .chord(vec![p(Note::B, 4), p(Note::D, 5)], Q)
        .chord(vec![p(Note::Bes, 4), p(Note::D, 5)], Q)
        .chord(vec![p(Note::Bes, 4), p(Note::D, 5)], Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&flat_key),
        vec![vec![plain(NATURAL), None, plain(FLAT), None, None, None]]
    );
}

// --- staff-wide state in musical order across voices ---

/// Voice 0: F♯4 half, F4 half. Voice 1: C4, F4 quarters then F♯4 half.
fn two_voices(voice_one_first: bool) -> ScoreBuilder {
    let voice_zero = |score: ScoreBuilder| {
        score
            .voice(0)
            .note(p(Note::Fis, 4), Duration::HALF)
            .note(p(Note::F, 4), Duration::HALF)
    };
    let voice_one = |score: ScoreBuilder| {
        score
            .voice(1)
            .note(p(Note::C, 4), Q)
            .note(p(Note::F, 4), Q)
            .note(p(Note::Fis, 4), Duration::HALF)
    };
    let score = ScoreBuilder::new().time_signature(4, 4);
    let score = if voice_one_first {
        voice_zero(voice_one(score))
    } else {
        voice_one(voice_zero(score))
    };
    score.end_barline()
}

#[test]
fn voices_resolve_in_onset_order_regardless_of_builder_order() {
    // Beat 1: F♯ (voice 0) shows its sharp. Beat 2: voice 1's F cancels it.
    // Beat 3: voice 0's F is already natural; voice 1's F♯ then needs a sharp.
    let expected = vec![
        vec![plain(SHARP), None],
        vec![None, plain(NATURAL), plain(SHARP)],
    ];
    assert_eq!(
        first_measure_voice_accidentals(&two_voices(false)),
        expected
    );
    assert_eq!(first_measure_voice_accidentals(&two_voices(true)), expected);
    assert_eq!(
        two_voices(false).render_svg(),
        two_voices(true).render_svg(),
        "builder voice order must not change the engraving"
    );
}

#[test]
fn simultaneous_onsets_resolve_lower_voice_index_first() {
    let build = |voice_one_first: bool| {
        let zero = |score: ScoreBuilder| score.voice(0).note(p(Note::Fis, 4), Q);
        let one = |score: ScoreBuilder| score.voice(1).note(p(Note::F, 4), Q);
        let score = ScoreBuilder::new();
        let score = if voice_one_first {
            zero(one(score))
        } else {
            one(zero(score))
        };
        score.end_barline()
    };
    let expected = vec![vec![plain(SHARP)], vec![plain(NATURAL)]];
    assert_eq!(first_measure_voice_accidentals(&build(false)), expected);
    assert_eq!(first_measure_voice_accidentals(&build(true)), expected);
}

#[test]
fn tuplet_member_onsets_use_the_performed_ratio() {
    // Voice 0's triplet F♯ starts 10⅔ ticks in; voice 1's F starts at 12
    // (after a dotted sixteenth). The F follows the F♯, so it needs a natural.
    // Ignoring the 3:2 ratio would place the F♯ at 16 ticks, after the F.
    let score = ScoreBuilder::new()
        .voice(1)
        .note(p(Note::C, 4), Duration::new(DurationKind::Sixteenth, 1))
        .note(p(Note::F, 4), Duration::SIXTEENTH)
        .voice(0)
        .tuplet_ratio(
            3,
            2,
            vec![(p(Note::C, 5), E), (p(Note::Fis, 4), E), (p(Note::C, 5), E)],
        )
        .end_barline();
    assert_eq!(
        first_measure_voice_accidentals(&score),
        vec![vec![None, plain(SHARP), None], vec![None, plain(NATURAL)]]
    );
}

// --- display policies ---

#[test]
fn force_restates_a_repeated_accidental_and_shows_naturals_on_unaltered_notes() {
    let score = ScoreBuilder::new()
        .note(p(Note::Fis, 4), Q)
        .note_with_accidental(p(Note::Fis, 4), Q, AccidentalDisplay::Force)
        .note_with_accidental(p(Note::C, 4), Q, AccidentalDisplay::Force)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![plain(SHARP), plain(SHARP), plain(NATURAL)]]
    );
}

#[test]
fn forced_and_cautionary_signs_set_the_measure_state() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note_with_accidental(p(Note::F, 4), Q, AccidentalDisplay::Force)
        .note(p(Note::F, 4), Q)
        .barline()
        .note(p(Note::Ais, 4), Q)
        .note_with_accidental(p(Note::A, 4), Q, AccidentalDisplay::Cautionary)
        .note(p(Note::A, 4), Q)
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![
            vec![plain(NATURAL), None],
            vec![plain(SHARP), cautionary(NATURAL), None],
        ]
    );
}

#[test]
fn cautionary_restates_the_key_signature_in_parentheses() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note_with_accidental(p(Note::Fis, 4), Q, AccidentalDisplay::Cautionary)
        .end_barline();
    assert_eq!(measure_accidentals(&score), vec![vec![cautionary(SHARP)]]);
}

#[test]
fn chord_and_group_members_carry_their_own_display_policies() {
    let score = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .chord_with_accidentals(
            vec![p(Note::Fis, 4), p(Note::A, 4), p(Note::C, 5)],
            Q,
            vec![AccidentalDisplay::Cautionary, AccidentalDisplay::Force],
        )
        .beam_group_with_accidentals(vec![
            (p(Note::Fis, 4), E, AccidentalDisplay::Auto),
            (p(Note::Fis, 4), E, AccidentalDisplay::Force),
        ])
        .tuplet_ratio_with_accidentals(
            3,
            2,
            vec![
                (p(Note::F, 4), E, AccidentalDisplay::Auto),
                (p(Note::F, 4), E, AccidentalDisplay::Cautionary),
                (p(Note::Fis, 4), E, AccidentalDisplay::Auto),
            ],
        )
        .end_barline();
    assert_eq!(
        measure_accidentals(&score),
        vec![vec![
            // Chord: (♯) cautionary, forced ♮ on A, C left automatic.
            cautionary(SHARP),
            plain(NATURAL),
            None,
            // Beam: F♯ already active after the chord; forced restatement.
            None,
            plain(SHARP),
            // Tuplet: cancel, cautionary repeat, reinstated key sharp.
            plain(NATURAL),
            cautionary(NATURAL),
            plain(SHARP),
        ]]
    );
}

// --- rendering and spacing ---

#[test]
fn reinstated_key_signature_sharp_is_engraved_after_the_natural() {
    let svg = ScoreBuilder::new()
        .key_signature(KeySignature::Sharps(1))
        .note(p(Note::F, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .note(p(Note::Fis, 4), Q)
        .end_barline()
        .render_svg();
    let naturals = placements(&svg, NATURAL);
    let sharps = placements(&svg, SHARP);
    assert_eq!(naturals.len(), 1, "one cancelling natural");
    // The key signature's sharp plus exactly one reinstating sharp.
    assert_eq!(sharps.len(), 2);
    let f4_y = naturals[0].1;
    assert_eq!(sharps[1].1, f4_y, "the reinstating sharp sits on F4");
    assert!(sharps[1].0 > naturals[0].0);
}

#[test]
fn cautionary_render_encloses_the_glyph_in_accidental_parentheses() {
    let svg = ScoreBuilder::new()
        .note_with_accidental(p(Note::Fis, 4), Q, AccidentalDisplay::Cautionary)
        .end_barline()
        .render_svg();
    let open = placements(&svg, Glyph::AccidentalParensLeft);
    let sharp = placements(&svg, SHARP);
    let close = placements(&svg, Glyph::AccidentalParensRight);
    let notehead = placements(&svg, Glyph::NoteheadBlack);
    assert_eq!((open.len(), sharp.len(), close.len()), (1, 1, 1));
    let y = notehead[0].1;
    assert_eq!((open[0].1, sharp[0].1, close[0].1), (y, y, y));
    assert_close(open[0].0 + advance(Glyph::AccidentalParensLeft), sharp[0].0);
    assert_close(sharp[0].0 + advance(SHARP), close[0].0);
    let staff_space = bravura_font().engraving_config().staff_space;
    assert!(
        (close[0].0
            + advance(Glyph::AccidentalParensRight)
            + ACCIDENTAL_NOTEHEAD_PADDING_SS * staff_space
            - notehead[0].0)
            .abs()
            < 1e-9
    );
}

/// Layout of the first measure's primary voice, without a system prefix.
fn first_measure_layout(score: &ScoreBuilder) -> crate::layout::measure::MeasureLayout {
    let elements: Vec<_> = score.build_measure_contents()[0]
        .events
        .iter()
        .map(measure_event_to_element)
        .collect();
    layout_measure(&elements, &MeasureLayoutConfig::from_staff_space(250.0))
}

#[test]
fn a_cautionary_accidental_reserves_its_parentheses_left_of_the_note() {
    let config = MeasureLayoutConfig::from_staff_space(250.0);
    let with = |display| {
        first_measure_layout(
            &ScoreBuilder::new()
                .note(p(Note::C, 5), Q)
                .note_with_accidental(p(Note::Fis, 4), Q, display)
                .note(p(Note::G, 4), Q)
                .end_barline(),
        )
    };
    let plain = with(AccidentalDisplay::Force);
    let parenthesized = with(AccidentalDisplay::Cautionary);
    let (c5, f_sharp, g4) = (0, 1, 2);
    // The note itself moves right by exactly the parentheses' width: the
    // extra space sits between it and the preceding C5…
    assert_eq!(plain.elements[c5].x, parenthesized.elements[c5].x);
    assert!(
        (parenthesized.elements[f_sharp].x
            - plain.elements[f_sharp].x
            - config.accidental_parens_rod)
            .abs()
            < 1e-9
    );
    // …and is incompressible rod, not spring.
    assert!(
        (parenthesized.total_rod - plain.total_rod - config.accidental_parens_rod).abs() < 1e-9
    );
    assert_eq!(
        parenthesized.elements[f_sharp].rod,
        plain.elements[f_sharp].rod
    );
    // The F♯'s own slot is unchanged, so G4 shifts only by the same amount.
    assert!(
        (parenthesized.elements[g4].x - plain.elements[g4].x - config.accidental_parens_rod).abs()
            < 1e-9
    );
}

#[test]
fn cautionary_accidentals_clear_the_preceding_notehead_at_full_compression() {
    // A narrow system clamps every spring to its floor, leaving rods alone.
    let mut score = ScoreBuilder::new().system_width_fu(250.0);
    for _ in 0..4 {
        score = score
            .note(p(Note::A, 4), Duration::SIXTEENTH)
            .note_with_accidental(
                p(Note::Fis, 4),
                Duration::SIXTEENTH,
                AccidentalDisplay::Cautionary,
            );
    }
    let svg = score.end_barline().render_svg();
    let noteheads = placements(&svg, Glyph::NoteheadBlack);
    let opens = placements(&svg, Glyph::AccidentalParensLeft);
    assert_eq!((noteheads.len(), opens.len()), (8, 4));
    let notehead_width = advance(Glyph::NoteheadBlack);
    for (pair, open) in opens.iter().enumerate() {
        let preceding_a4 = noteheads[pair * 2];
        assert!(
            open.0 > preceding_a4.0 + notehead_width,
            "cautionary group {pair} starts at {} inside the A4 notehead ending at {}",
            open.0,
            preceding_a4.0 + notehead_width
        );
    }
}

#[test]
fn chord_accidental_columns_stack_clear_of_a_cautionary_member() {
    let svg = ScoreBuilder::new()
        .chord_with_accidentals(
            vec![p(Note::Fis, 4), p(Note::Ais, 4)],
            Q,
            vec![AccidentalDisplay::Auto, AccidentalDisplay::Cautionary],
        )
        .end_barline()
        .render_svg();
    let staff_space = bravura_font().engraving_config().staff_space;
    let open = placements(&svg, Glyph::AccidentalParensLeft);
    let close = placements(&svg, Glyph::AccidentalParensRight);
    let sharps = placements(&svg, SHARP);
    let noteheads = placements(&svg, Glyph::NoteheadBlack);
    assert_eq!((open.len(), close.len(), sharps.len()), (1, 1, 2));
    // The upper (A♯) cautionary takes the column nearest the noteheads.
    let a_sharp_y = open[0].1;
    let chord_x = noteheads[0].0;
    assert!(
        (close[0].0
            + advance(Glyph::AccidentalParensRight)
            + ACCIDENTAL_NOTEHEAD_PADDING_SS * staff_space
            - chord_x)
            .abs()
            < 1e-9
    );
    // The F♯ a third below would collide vertically, so it sits one column
    // further left: its right edge ends exactly one column gap before the
    // left parenthesis.
    let f_sharp = sharps
        .iter()
        .find(|(_, y)| *y != a_sharp_y)
        .expect("F♯ sharp");
    assert!(
        (open[0].0 - (f_sharp.0 + advance(SHARP)) - ACCIDENTAL_COLUMN_GAP_SS * staff_space).abs()
            < 1e-9
    );
}
