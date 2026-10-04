//! Break-request resolution, mid-measure splits, and break policies honoring
//! explicit directives.

use super::*;
use crate::layout::accidental::ResolvedAccidental;
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{
    BeamGroupEvent, MeasureLayoutConfig, NoteAnnotations, RestEvent, TupletGroupEvent,
};
use crate::layout::measure_meta::MeasureMeta;
use crate::layout::page::{break_into_systems, SystemBreaking};
use crate::layout::system::SystemPrefix;
use music::notation::clef::Clef;
use smufl::Glyph;

fn note(duration_log2: i8, dots: u8) -> NoteEvent {
    NoteEvent {
        staff_position: 4,
        duration_log2,
        dots,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    }
}

fn quarter(staff_position: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position,
        ..note(2, 0)
    })
}

fn half() -> MeasureEvent {
    MeasureEvent::Note(note(1, 0))
}

fn whole() -> MeasureEvent {
    MeasureEvent::Note(note(0, 0))
}

fn measure(events: Vec<MeasureEvent>) -> MeasureContent {
    MeasureContent {
        events,
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
        meta: MeasureMeta::default(),
    }
}

fn four_quarters() -> MeasureContent {
    measure((0..4).map(quarter).collect())
}

fn request(measure: usize, position: usize, kind: LineBreak) -> LineBreakRequest {
    LineBreakRequest {
        measure,
        position,
        kind,
    }
}

fn len(numerator: u64, denominator: u64) -> MeasureLength {
    MeasureLength::new(numerator, denominator)
}

fn staff_positions(events: &[MeasureEvent]) -> Vec<i8> {
    events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.staff_position),
            _ => None,
        })
        .collect()
}

// --- event lengths ---

#[test]
fn event_length_counts_breves_dots_groups_and_tuplets() {
    assert_eq!(event_length(&MeasureEvent::Note(note(-1, 0))), len(2, 1));
    assert_eq!(event_length(&MeasureEvent::Note(note(2, 1))), len(3, 8));
    assert_eq!(event_length(&MeasureEvent::Note(note(1, 2))), len(7, 8));
    assert_eq!(
        event_length(&MeasureEvent::Rest(RestEvent {
            duration_log2: 3,
            dots: 0
        })),
        len(1, 8)
    );
    let eighths = BeamGroupEvent {
        notes: vec![note(3, 0), note(3, 0), note(3, 0)],
        stem_direction: None,
    };
    assert_eq!(
        event_length(&MeasureEvent::BeamGroup(eighths.clone())),
        len(3, 8)
    );
    // A 3:2 eighth triplet lasts one quarter.
    assert_eq!(
        event_length(&MeasureEvent::TupletGroup(TupletGroupEvent {
            beam_group: eighths,
            tuplet_number: 3,
            in_time_of: 2,
        })),
        len(1, 4)
    );
    assert_eq!(
        event_length(&MeasureEvent::Barline(BarlineStyle::Dashed)),
        MeasureLength::ZERO
    );
}

// --- request resolution ---

#[test]
fn request_before_a_measures_first_event_breaks_after_the_previous_measure() {
    let contents = vec![four_quarters(), four_quarters()];
    let plan = LineBreakPlan::from_requests(&[request(1, 0, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].meta.line_break, LineBreak::Force);
    assert_eq!(pieces[1].meta.line_break, LineBreak::Auto);
}

#[test]
fn request_after_a_measures_last_event_breaks_after_that_measure() {
    let contents = vec![four_quarters(), four_quarters()];
    let plan = LineBreakPlan::from_requests(&[request(0, 4, LineBreak::Forbid)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].meta.line_break, LineBreak::Forbid);
}

#[test]
fn request_after_trailing_inline_barline_still_breaks_at_the_measure_end() {
    let mut events: Vec<MeasureEvent> = (0..4).map(quarter).collect();
    events.push(MeasureEvent::Barline(BarlineStyle::Dashed));
    let contents = vec![measure(events), four_quarters()];
    let plan = LineBreakPlan::from_requests(&[request(0, 5, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces.len(), 2, "a measure-end request does not split");
    assert_eq!(pieces[0].meta.line_break, LineBreak::Force);
}

#[test]
fn requests_before_the_first_measure_or_after_the_last_have_no_effect() {
    let contents = vec![four_quarters()];
    let plan = LineBreakPlan::from_requests(
        &[
            request(0, 0, LineBreak::Force),
            request(1, 0, LineBreak::Force),
        ],
        &contents,
    );
    assert_eq!(plan, LineBreakPlan::default());
}

#[test]
fn later_request_at_the_same_boundary_wins() {
    let contents = vec![four_quarters(), four_quarters()];
    let plan = LineBreakPlan::from_requests(
        &[
            request(0, 4, LineBreak::Force),
            request(1, 0, LineBreak::Forbid),
        ],
        &contents,
    );
    assert_eq!(plan.apply(contents)[0].meta.line_break, LineBreak::Forbid);
}

#[test]
fn forbid_inside_a_measure_changes_nothing() {
    let contents = vec![four_quarters()];
    let plan = LineBreakPlan::from_requests(&[request(0, 2, LineBreak::Forbid)], &contents);
    assert_eq!(plan, LineBreakPlan::default());
}

#[test]
fn later_no_break_cancels_a_forced_mid_measure_break_at_the_same_onset() {
    let contents = vec![four_quarters()];
    let plan = LineBreakPlan::from_requests(
        &[
            request(0, 2, LineBreak::Force),
            request(0, 2, LineBreak::Forbid),
        ],
        &contents,
    );
    assert_eq!(plan.apply(contents).len(), 1);
}

// --- splits ---

#[test]
fn forced_break_inside_a_measure_splits_it_into_numbered_continuation_pieces() {
    let sharp = Some(ResolvedAccidental::plain(Glyph::AccidentalSharp));
    let mut first = note(2, 0);
    first.accidental = sharp;
    let mut content = measure(vec![
        MeasureEvent::Note(first),
        quarter(1),
        MeasureEvent::Barline(BarlineStyle::Invisible),
        quarter(2),
        quarter(3),
    ]);
    content.barline = BarlineStyle::Double;
    content.meta.number = 7;
    content.meta.line_break = LineBreak::Forbid;
    let contents = vec![content];

    let plan = LineBreakPlan::from_requests(&[request(0, 3, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);

    assert_eq!(pieces.len(), 2);
    // The first piece ends at the break, closed by the inline barline.
    assert_eq!(staff_positions(&pieces[0].events), vec![4, 1]);
    assert_eq!(
        pieces[0].events.len(),
        2,
        "inline barline became the closing one"
    );
    assert_eq!(pieces[0].barline, BarlineStyle::Invisible);
    assert_eq!(pieces[0].meta.line_break, LineBreak::Force);
    assert!(!pieces[0].meta.continuation);
    // The second keeps the measure's own barline and boundary directive.
    assert_eq!(staff_positions(&pieces[1].events), vec![2, 3]);
    assert_eq!(pieces[1].barline, BarlineStyle::Double);
    assert_eq!(pieces[1].meta.line_break, LineBreak::Forbid);
    assert!(pieces[1].meta.continuation);
    // Both pieces belong to logical measure 7; resolved accidentals survive.
    assert_eq!(pieces[0].meta.number, 7);
    assert_eq!(pieces[1].meta.number, 7);
    let MeasureEvent::Note(kept) = &pieces[0].events[0] else {
        panic!("first event is a note");
    };
    assert_eq!(kept.accidental, sharp);
}

#[test]
fn inline_barline_entered_after_the_break_request_still_closes_the_first_piece() {
    let contents = vec![measure(vec![
        quarter(0),
        quarter(1),
        MeasureEvent::Barline(BarlineStyle::Dashed),
        quarter(2),
    ])];
    // `system_break()` called before `inline_barline(Dashed)`.
    let plan = LineBreakPlan::from_requests(&[request(0, 2, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces[0].barline, BarlineStyle::Dashed);
    assert_eq!(pieces[0].events.len(), 2);
    assert_eq!(staff_positions(&pieces[1].events), vec![2]);
}

#[test]
fn split_without_an_inline_barline_closes_the_first_piece_invisibly() {
    let contents = vec![four_quarters()];
    let plan = LineBreakPlan::from_requests(&[request(0, 1, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].barline, BarlineStyle::Invisible);
    assert_eq!(staff_positions(&pieces[0].events), vec![0]);
    assert_eq!(staff_positions(&pieces[1].events), vec![1, 2, 3]);
}

#[test]
fn several_splits_produce_one_piece_each() {
    let contents = vec![four_quarters()];
    let plan = LineBreakPlan::from_requests(
        &[
            request(0, 3, LineBreak::Force),
            request(0, 1, LineBreak::Force),
        ],
        &contents,
    );
    let pieces = plan.apply(contents);
    let positions: Vec<Vec<i8>> = pieces.iter().map(|p| staff_positions(&p.events)).collect();
    assert_eq!(positions, vec![vec![0], vec![1, 2], vec![3]]);
    let continuation: Vec<bool> = pieces.iter().map(|p| p.meta.continuation).collect();
    assert_eq!(continuation, vec![false, true, true]);
}

#[test]
fn additional_voices_split_at_the_same_onset() {
    let mut content = four_quarters();
    // Voice 1: two halves split exactly at the half; voice 2: a whole note
    // sounding across the split stays in the first piece.
    content.additional_voices = vec![vec![half(), half()], vec![whole()]];
    let contents = vec![content];
    let plan = LineBreakPlan::from_requests(&[request(0, 2, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(pieces[0].additional_voices[0].len(), 1);
    assert_eq!(pieces[1].additional_voices[0].len(), 1);
    assert_eq!(pieces[0].additional_voices[1].len(), 1);
    assert!(pieces[1].additional_voices[1].is_empty());
}

#[test]
fn volta_text_and_left_hook_stay_on_the_first_piece() {
    let mut content = four_quarters();
    content.volta = Some(VoltaAnnotation {
        text: Some("1.".into()),
        hooks: VoltaHooks::Both,
    });
    let contents = vec![content];
    let plan = LineBreakPlan::from_requests(&[request(0, 2, LineBreak::Force)], &contents);
    let pieces = plan.apply(contents);
    assert_eq!(
        pieces[0].volta,
        Some(VoltaAnnotation {
            text: Some("1.".into()),
            hooks: VoltaHooks::LeftOnly,
        })
    );
    assert_eq!(
        pieces[1].volta,
        Some(VoltaAnnotation {
            text: None,
            hooks: VoltaHooks::RightOnly,
        })
    );
}

#[test]
fn merged_plans_split_every_stave_and_force_wins_over_forbid() {
    let upper = vec![four_quarters(), four_quarters(), four_quarters()];
    let lower = vec![
        measure(vec![half(), half()]),
        measure(vec![half(), half()]),
        measure(vec![half(), half()]),
    ];
    let mut plan = LineBreakPlan::from_requests(
        &[
            request(0, 2, LineBreak::Force),
            request(1, 4, LineBreak::Forbid),
        ],
        &upper,
    );
    plan.merge(&LineBreakPlan::from_requests(
        &[request(1, 2, LineBreak::Force)],
        &lower,
    ));

    let lower_pieces = plan.apply(lower);
    // Measure 0 split at the upper stave's half-bar request.
    assert_eq!(lower_pieces.len(), 4);
    assert_eq!(lower_pieces[0].events.len(), 1);
    assert!(lower_pieces[1].meta.continuation);
    // Boundary after measure 1: the lower stave's Force beats the upper's Forbid.
    assert_eq!(lower_pieces[2].meta.line_break, LineBreak::Force);
}

// --- policies honoring directives ---

fn prefix() -> SystemPrefix {
    SystemPrefix::new(&Clef::Treble, KeySignature::Open, None)
}

fn directed(n: usize, directives: &[(usize, LineBreak)]) -> Vec<MeasureContent> {
    (0..n)
        .map(|i| {
            let mut content = four_quarters();
            if let Some(&(_, kind)) = directives.iter().find(|(at, _)| *at == i) {
                content.meta.line_break = kind;
            }
            content
        })
        .collect()
}

fn systems(
    measures: &[MeasureContent],
    width: f64,
    breaking: &SystemBreaking,
) -> Vec<(usize, usize)> {
    break_into_systems(
        &prefix(),
        measures,
        &MeasureLayoutConfig::from_staff_space(250.0),
        width,
        breaking,
    )
}

#[test]
fn fixed_count_restarts_after_a_forced_break() {
    let measures = directed(6, &[(0, LineBreak::Force)]);
    assert_eq!(
        systems(&measures, 100_000.0, &SystemBreaking::Fixed(4)),
        vec![(0, 1), (1, 5), (5, 6)]
    );
}

#[test]
fn forbidden_break_keeps_measures_on_one_system_in_every_policy() {
    let measures = directed(6, &[(1, LineBreak::Forbid)]);
    assert_eq!(
        systems(&measures, 100_000.0, &SystemBreaking::Fixed(2)),
        vec![(0, 3), (3, 5), (5, 6)]
    );
    // A target narrower than any measure puts each unit on its own system.
    let narrow = [(0, 1), (1, 3), (3, 4), (4, 5), (5, 6)];
    assert_eq!(systems(&measures, 1.0, &SystemBreaking::Auto), narrow);
    assert_eq!(systems(&measures, 1.0, &SystemBreaking::Optimal), narrow);
}

#[test]
fn width_policies_always_break_at_forced_breaks() {
    let measures = directed(5, &[(1, LineBreak::Force)]);
    for breaking in [SystemBreaking::Auto, SystemBreaking::Optimal] {
        assert_eq!(
            systems(&measures, 100_000.0, &breaking),
            vec![(0, 2), (2, 5)],
            "{breaking:?}"
        );
    }
}

#[test]
fn explicit_policy_never_breaks_overflowing_content_elsewhere() {
    let measures = directed(12, &[(4, LineBreak::Force)]);
    // Far too narrow for even one measure: still only the forced break.
    assert_eq!(
        systems(&measures, 1.0, &SystemBreaking::Explicit),
        vec![(0, 5), (5, 12)]
    );
    assert_eq!(
        systems(&directed(12, &[]), 1.0, &SystemBreaking::Explicit),
        vec![(0, 12)]
    );
}
