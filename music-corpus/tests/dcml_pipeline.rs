use std::path::Path;

use music::{note::pitch_class::Pc, note_collections::pc_set::PcShape};
use music_corpus::{
    analyze_divergence, analyze_piece, write_divergence_summary_tsv, write_divergence_tsv,
    write_slice_tsv, ClassifiedQuality, DcmlAdapter, DcmlPaths, DivergenceCategory, MetricWeight,
    ScoreTime,
};
use musical_combinatorics::{FourNoteChordQuality, ThreeNoteChordQuality};

fn fixture_piece() -> music_corpus::CorpusPiece {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    DcmlAdapter::load_piece(
        DcmlPaths {
            notes: &root.join("notes.tsv"),
            measures: &root.join("measures.tsv"),
            expanded: &root.join("expanded.tsv"),
        },
        "fixture",
        "movement",
    )
    .unwrap()
}

#[test]
fn loads_ties_as_one_lossless_note_event() {
    let piece = fixture_piece();
    assert_eq!(piece.notes.len(), 8);
    let tied = piece
        .notes
        .iter()
        .find(|note| note.source.rows == [1, 2])
        .expect("tie chain should preserve both source rows");
    assert_eq!(tied.score_span.start, ScoreTime::ZERO);
    assert_eq!(tied.score_span.end, ScoreTime::from_integer(4));
}

#[test]
fn slices_one_movement_and_keeps_harmony_separate() {
    let piece = fixture_piece();
    let report = analyze_piece(&piece).unwrap();

    assert_eq!(report.slices.len(), 2);
    assert_eq!(report.note_rows_accounted, 9);
    assert_eq!(report.zero_duration_note_rows, 1);
    assert_eq!(report.cardinality_counts[3], 1);
    assert_eq!(report.cardinality_counts[4], 1);
    assert_eq!(report.classified_three, 1);
    assert_eq!(report.classified_four, 1);

    let first = &report.slices[0];
    assert_eq!(first.start, ScoreTime::ZERO);
    assert_eq!(first.end, ScoreTime::from_integer(4));
    assert_eq!(first.metric_weight, MetricWeight::Downbeat);
    assert_eq!(piece.harmonies[first.harmony_index.unwrap()].label, "I");
    assert!(matches!(
        first.quality,
        Some(ClassifiedQuality::Three { .. })
    ));

    let second = &report.slices[1];
    assert_eq!(second.start, first.end);
    assert_eq!(second.end, ScoreTime::from_integer(8));
    assert_eq!(
        piece.harmonies[second.harmony_index.unwrap()].label,
        "V7|PAC}"
    );
    assert!(matches!(
        second.quality,
        Some(ClassifiedQuality::Four { .. })
    ));
}

#[test]
fn report_contains_provenance_and_cardinality() {
    let piece = fixture_piece();
    let report = analyze_piece(&piece).unwrap();
    let mut output = Vec::new();
    write_slice_tsv(&piece, &report, &mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("active_note_rows"));
    assert!(output.contains("1,2,3,4"));
    assert!(output.contains("0x091"));
    assert!(output.contains("\t3\tMajor\t"));
    assert!(output.contains("V7|PAC}"));
}

#[test]
fn divergence_report_preserves_roots_basses_and_source_links() {
    let piece = fixture_piece();
    let sonorities = analyze_piece(&piece).unwrap();
    let report = analyze_divergence(&piece, &sonorities);

    assert_eq!(report.records.len(), 2);
    assert!(report
        .records
        .iter()
        .all(|record| record.category == DivergenceCategory::Agreement));
    assert_eq!(report.records[0].annotated_root_pitch_class, Some(0));
    assert_eq!(report.records[1].annotated_root_pitch_class, Some(7));
    assert_eq!(
        report
            .summary
            .iter()
            .map(|row| row.slice_count)
            .sum::<u64>(),
        2
    );

    let mut details = Vec::new();
    write_divergence_tsv(&piece, &sonorities, &report, &mut details).unwrap();
    let details = String::from_utf8(details).unwrap();
    assert!(details.contains("non_chord_pc_mask"));
    assert!(details.contains("Agreement"));
    assert!(details.contains("1,2,3,4"));

    let mut summary = Vec::new();
    write_divergence_summary_tsv(&report, &mut summary).unwrap();
    let summary = String::from_utf8(summary).unwrap();
    assert!(summary.contains("total_duration_qb"));
    assert!(summary.contains("V7|PAC}"));
}

#[test]
fn supported_dcml_changes_classify_against_their_exact_pitch_classes() {
    let cases = [
        ("9", 0x095),
        ("4", 0x0a1),
        ("64", 0x221),
        ("#7b64", 0x121),
        ("974", 0x8a4),
    ];

    for (changes, expected) in cases {
        let mut piece = fixture_piece();
        let mut sonorities = analyze_piece(&piece).unwrap();
        piece.harmonies[0].label = format!("I({changes})");
        piece.harmonies[0].changes = Some(changes.into());
        piece.harmonies[0].bass_fifths = None;
        sonorities.slices[0].pitch_class_mask = expected;
        sonorities.slices[0].cardinality = expected.count_ones() as u8;
        sonorities.slices[0].bass_midi = None;
        sonorities.slices[0].quality = None;

        let report = analyze_divergence(&piece, &sonorities);
        assert_eq!(
            report.records[0].expected_pitch_class_mask,
            Some(expected),
            "{changes}"
        );
        assert_eq!(
            report.records[0].category,
            DivergenceCategory::Agreement,
            "{changes}"
        );
        assert_eq!(piece.harmonies[0].changes.as_deref(), Some(changes));
        assert_eq!(piece.harmonies[0].source.rows, [1]);
    }
}

#[test]
fn divergence_taxonomy_does_not_infer_agreement_from_root_free_quality() {
    let mut piece = fixture_piece();
    let mut sonorities = analyze_piece(&piece).unwrap();

    sonorities.slices[0].pitch_class_mask &= !(1 << 4);
    sonorities.slices[0].cardinality = 2;
    sonorities.slices[0].quality = None;
    let incomplete = analyze_divergence(&piece, &sonorities);
    assert_eq!(
        incomplete.records[0].category,
        DivergenceCategory::IncompleteHarmony
    );
    assert_eq!(incomplete.records[0].missing_pitch_class_mask, Some(1 << 4));

    sonorities.slices[0].bass_midi = Some(64);
    let incomplete_bass = analyze_divergence(&piece, &sonorities);
    assert_eq!(
        incomplete_bass.records[0].category,
        DivergenceCategory::IncompleteHarmonyBassMismatch
    );

    sonorities.slices[0].pitch_class_mask = 0;
    sonorities.slices[0].cardinality = 0;
    sonorities.slices[0].bass_midi = None;
    let rest = analyze_divergence(&piece, &sonorities);
    assert_eq!(
        rest.records[0].category,
        DivergenceCategory::NoSoundingNotes
    );

    sonorities.slices[0].pitch_class_mask = 0x091;
    sonorities.slices[0].cardinality = 3;
    sonorities.slices[0].bass_midi = Some(64);
    let ambiguous = analyze_divergence(&piece, &sonorities);
    assert_eq!(
        ambiguous.records[0].category,
        DivergenceCategory::BassOrInversionMismatch
    );

    piece.harmonies[0].changes = Some("+6".into());
    let unresolved = analyze_divergence(&piece, &sonorities);
    assert_eq!(
        unresolved.records[0].category,
        DivergenceCategory::Unresolved
    );
    assert_eq!(unresolved.records[0].expected_pitch_class_mask, None);
}

#[test]
fn sonority_slices_split_at_harmony_boundaries_without_note_attacks() {
    let mut piece = fixture_piece();
    piece.harmonies[0].score_span.end = ScoreTime::from_integer(3);
    piece.harmonies[1].score_span.start = ScoreTime::from_integer(3);

    let report = analyze_piece(&piece).unwrap();
    let boundary = report
        .slices
        .iter()
        .position(|slice| slice.start == ScoreTime::from_integer(3))
        .expect("harmony change must create a slice");

    assert_eq!(report.slices[boundary - 1].end, ScoreTime::from_integer(3));
    assert_eq!(report.slices[boundary].harmony_index, Some(1));
}

#[test]
fn classifiers_cover_every_three_and_four_pc_mask() {
    let mut threes = 0;
    let mut fours = 0;
    for mask in 0_u16..(1 << 12) {
        let cardinality = mask.count_ones();
        if cardinality != 3 && cardinality != 4 {
            continue;
        }
        let pcs = (0_u8..12)
            .filter(|pc| mask & (1 << pc) != 0)
            .map(|pc| Pc::from(&pc))
            .collect();
        let shape = PcShape::new(pcs);
        if cardinality == 3 {
            ThreeNoteChordQuality::identify(&shape).unwrap();
            threes += 1;
        } else {
            FourNoteChordQuality::identify(&shape).unwrap();
            fours += 1;
        }
    }
    assert_eq!(threes, 220);
    assert_eq!(fours, 495);
}
