use std::{collections::BTreeMap, fmt, io::Write};

use anyhow::Result;

use crate::{
    dcml::supported_chord_changes,
    model::{CorpusPiece, HarmonyAnnotation, ScoreTime},
    sonority::{MetricWeight, SonorityReport, SonoritySlice},
};

/// A conservative, mechanically established relationship between a sounding
/// pitch-class set and the active DCML harmony annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DivergenceCategory {
    Agreement,
    NoSoundingNotes,
    IncompleteHarmony,
    IncompleteHarmonyBassMismatch,
    PassingOrNeighbor,
    Suspension,
    Anticipation,
    BassOrInversionMismatch,
    Unresolved,
}

impl fmt::Display for DivergenceCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceRecord {
    pub slice_index: usize,
    pub category: DivergenceCategory,
    pub expected_pitch_class_mask: Option<u16>,
    pub literal_root_pitch_class: Option<u8>,
    pub annotated_root_pitch_class: Option<u8>,
    pub literal_bass_pitch_class: Option<u8>,
    pub annotated_bass_pitch_class: Option<u8>,
    pub non_chord_pitch_class_mask: Option<u16>,
    pub missing_pitch_class_mask: Option<u16>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DivergenceSummaryKey {
    pub category: DivergenceCategory,
    pub cardinality: u8,
    pub metric_weight: MetricWeight,
    pub harmony_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceSummaryRow {
    pub key: DivergenceSummaryKey,
    pub slice_count: u64,
    pub total_duration: ScoreTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceReport {
    pub records: Vec<DivergenceRecord>,
    pub summary: Vec<DivergenceSummaryRow>,
}

pub fn analyze_divergence(piece: &CorpusPiece, sonorities: &SonorityReport) -> DivergenceReport {
    let records: Vec<_> = sonorities
        .slices
        .iter()
        .enumerate()
        .map(|(index, slice)| classify_slice(piece, &sonorities.slices, index, slice))
        .collect();

    let mut summaries: BTreeMap<DivergenceSummaryKey, (u64, ScoreTime)> = BTreeMap::new();
    for record in &records {
        let slice = &sonorities.slices[record.slice_index];
        let harmony_label = slice
            .harmony_index
            .map(|index| piece.harmonies[index].label.clone())
            .unwrap_or_default();
        let key = DivergenceSummaryKey {
            category: record.category,
            cardinality: slice.cardinality,
            metric_weight: slice.metric_weight,
            harmony_label,
        };
        let entry = summaries.entry(key).or_insert((0, ScoreTime::ZERO));
        entry.0 += 1;
        entry.1 = entry.1 + slice.duration();
    }
    let summary = summaries
        .into_iter()
        .map(
            |(key, (slice_count, total_duration))| DivergenceSummaryRow {
                key,
                slice_count,
                total_duration,
            },
        )
        .collect();

    DivergenceReport { records, summary }
}

fn classify_slice(
    piece: &CorpusPiece,
    slices: &[SonoritySlice],
    index: usize,
    slice: &SonoritySlice,
) -> DivergenceRecord {
    let literal_bass = slice.bass_midi.map(|midi| midi % 12);
    // Set-class rotations are not harmonic roots; keep the literal root unknown.
    let literal_root = None;
    let Some(harmony_index) = slice.harmony_index else {
        return unresolved(
            index,
            literal_root,
            literal_bass,
            "no active harmony annotation",
        );
    };
    let harmony = &piece.harmonies[harmony_index];
    let tonic = harmony.global_key.as_deref().and_then(key_pitch_class);
    let annotated_root = harmony
        .root_fifths
        .zip(tonic)
        .map(|(root, tonic)| (tonic + fifths_to_pitch_class(root)) % 12);
    // DCML `bass_note` and `root` are fifth offsets from the global tonic.
    let annotated_bass = harmony
        .bass_fifths
        .zip(tonic)
        .map(|(bass, tonic)| (tonic + fifths_to_pitch_class(bass)) % 12);
    let Some(expected) = annotated_chord_mask(harmony) else {
        return unresolved_with_annotation(
            index,
            literal_root,
            annotated_root,
            literal_bass,
            annotated_bass,
            "annotation lacks a supported chord spelling or change interpretation",
        );
    };

    let extras = slice.pitch_class_mask & !expected & 0x0fff;
    let missing = expected & !slice.pitch_class_mask & 0x0fff;
    let bass_agrees = annotated_bass.is_none() || annotated_bass == literal_bass;

    let (category, reason) = if slice.pitch_class_mask == 0 {
        (
            DivergenceCategory::NoSoundingNotes,
            "no positive-duration notes sound during the annotated harmony",
        )
    } else if extras == 0 && missing == 0 && bass_agrees {
        (
            DivergenceCategory::Agreement,
            "pitch classes and bass agree; literal root is not inferred",
        )
    } else if extras == 0 && missing != 0 && bass_agrees {
        (
            DivergenceCategory::IncompleteHarmony,
            "literal sonority is a strict subset of the annotated chord and bass agrees",
        )
    } else if extras == 0 && missing != 0 {
        (
            DivergenceCategory::IncompleteHarmonyBassMismatch,
            "literal sonority is a strict subset but annotated and sounding bass differ",
        )
    } else if extras == 0 && missing == 0 {
        (
            DivergenceCategory::BassOrInversionMismatch,
            "pitch-class set agrees but annotated and sounding bass differ",
        )
    } else if is_suspension(piece, slices, index, harmony_index, extras) {
        (
            DivergenceCategory::Suspension,
            "non-chord pitch is held across the harmony boundary from the preceding harmony",
        )
    } else if is_anticipation(piece, slices, index, harmony_index, extras) {
        (
            DivergenceCategory::Anticipation,
            "non-chord pitch is held into the following harmony as a chord tone",
        )
    } else if is_passing_or_neighbor(piece, slice, extras, expected) {
        (
            DivergenceCategory::PassingOrNeighbor,
            "single non-chord attack is stepwise between adjacent notes in the same voice",
        )
    } else {
        (
            DivergenceCategory::Unresolved,
            "literal and annotated sonorities diverge without a mechanically established cause",
        )
    };

    DivergenceRecord {
        slice_index: index,
        category,
        expected_pitch_class_mask: Some(expected),
        literal_root_pitch_class: literal_root,
        annotated_root_pitch_class: annotated_root,
        literal_bass_pitch_class: literal_bass,
        annotated_bass_pitch_class: annotated_bass,
        non_chord_pitch_class_mask: Some(extras),
        missing_pitch_class_mask: Some(missing),
        reason,
    }
}

fn unresolved(
    index: usize,
    literal_root: Option<u8>,
    literal_bass: Option<u8>,
    reason: &'static str,
) -> DivergenceRecord {
    unresolved_with_annotation(index, literal_root, None, literal_bass, None, reason)
}

fn unresolved_with_annotation(
    index: usize,
    literal_root: Option<u8>,
    annotated_root: Option<u8>,
    literal_bass: Option<u8>,
    annotated_bass: Option<u8>,
    reason: &'static str,
) -> DivergenceRecord {
    DivergenceRecord {
        slice_index: index,
        category: DivergenceCategory::Unresolved,
        expected_pitch_class_mask: None,
        literal_root_pitch_class: literal_root,
        annotated_root_pitch_class: annotated_root,
        literal_bass_pitch_class: literal_bass,
        annotated_bass_pitch_class: annotated_bass,
        non_chord_pitch_class_mask: None,
        missing_pitch_class_mask: None,
        reason,
    }
}

fn annotated_chord_mask(harmony: &HarmonyAnnotation) -> Option<u16> {
    let changes = supported_chord_changes(harmony.changes.as_deref())?;
    let tonic = key_pitch_class(harmony.global_key.as_deref()?)?;
    let root_fifths = harmony.root_fifths?;
    let root = (tonic + fifths_to_pitch_class(root_fifths)) % 12;
    let intervals: &[u8] = match harmony.chord_type.as_deref()? {
        "M" => &[0, 4, 7],
        "m" => &[0, 3, 7],
        "o" => &[0, 3, 6],
        "+" => &[0, 4, 8],
        "Mm7" => &[0, 4, 7, 10],
        "mm7" => &[0, 3, 7, 10],
        "MM7" => &[0, 4, 7, 11],
        "mM7" => &[0, 3, 7, 11],
        "o7" => &[0, 3, 6, 9],
        "%7" => &[0, 3, 6, 10],
        _ => return None,
    };
    let mut mask = intervals
        .iter()
        .fold(0, |mask, interval| mask | 1 << ((root + interval) % 12));
    for change in changes {
        if let Some(member) = change.replaces {
            let interval = *intervals.get(member)?;
            mask &= !(1 << ((root + interval) % 12));
        }
        let pitch_class = changed_pitch_class(harmony, tonic, root_fifths, *change)?;
        mask |= 1 << pitch_class;
    }
    Some(mask)
}

#[derive(Clone, Copy)]
enum ScaleMode {
    Major,
    Minor,
}

fn changed_pitch_class(
    harmony: &HarmonyAnnotation,
    global_tonic: u8,
    root_fifths: i32,
    change: crate::dcml::ChordToneChange,
) -> Option<u8> {
    let global_key = harmony.global_key.as_deref()?;
    let global_mode = key_mode(global_key)?;
    let (local_tonic_fifths, local_mode) = match harmony.local_key.as_deref() {
        Some(local_key) => local_key_context(local_key, global_mode)?,
        None => (0, global_mode),
    };
    let local_tonic = (global_tonic + fifths_to_pitch_class(local_tonic_fifths)) % 12;
    let root_degree = ((root_fifths - local_tonic_fifths) * 4).rem_euclid(7) as usize;
    let changed_degree = (root_degree + usize::from(change.interval - 1)) % 7;
    let scale = match local_mode {
        ScaleMode::Major => [0_i8, 2, 4, 5, 7, 9, 11],
        ScaleMode::Minor => [0_i8, 2, 3, 5, 7, 8, 10],
    };
    Some(
        (i16::from(local_tonic) + i16::from(scale[changed_degree]) + i16::from(change.accidental))
            .rem_euclid(12) as u8,
    )
}

fn key_mode(key: &str) -> Option<ScaleMode> {
    match key.chars().next()? {
        'A'..='G' => Some(ScaleMode::Major),
        'a'..='g' => Some(ScaleMode::Minor),
        _ => None,
    }
}

fn local_key_context(local_key: &str, global_mode: ScaleMode) -> Option<(i32, ScaleMode)> {
    let accidental_end = local_key
        .char_indices()
        .find_map(|(index, character)| (!matches!(character, 'b' | '#')).then_some(index))
        .unwrap_or(local_key.len());
    let (accidentals, numeral) = local_key.split_at(accidental_end);
    let degree = match numeral.to_ascii_uppercase().as_str() {
        "I" => 0,
        "II" => 1,
        "III" => 2,
        "IV" => 3,
        "V" => 4,
        "VI" => 5,
        "VII" => 6,
        _ => return None,
    };
    let diatonic_fifths = match global_mode {
        ScaleMode::Major => [0, 2, 4, -1, 1, 3, 5],
        ScaleMode::Minor => [0, 2, -3, -1, 1, -4, -2],
    }[degree];
    let accidental_fifths = accidentals.chars().try_fold(0, |fifths, accidental| {
        Some(
            fifths
                + match accidental {
                    '#' => 7,
                    'b' => -7,
                    _ => return None,
                },
        )
    })?;
    let mode = if numeral.chars().next()?.is_ascii_uppercase() {
        ScaleMode::Major
    } else {
        ScaleMode::Minor
    };
    Some((diatonic_fifths + accidental_fifths, mode))
}

fn fifths_to_pitch_class(fifths: i32) -> u8 {
    (fifths * 7).rem_euclid(12) as u8
}

fn key_pitch_class(key: &str) -> Option<u8> {
    let mut characters = key.chars();
    let natural = match characters.next()?.to_ascii_uppercase() {
        'C' => 0_i16,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let pitch = characters.try_fold(natural, |pitch, accidental| match accidental {
        '#' | '♯' => Some(pitch + 1),
        'b' | '♭' => Some(pitch - 1),
        _ => None,
    })?;
    Some(pitch.rem_euclid(12) as u8)
}

fn is_suspension(
    piece: &CorpusPiece,
    slices: &[SonoritySlice],
    index: usize,
    harmony_index: usize,
    extras: u16,
) -> bool {
    if extras.count_ones() != 1
        || index == 0
        || slices[index - 1].harmony_index == Some(harmony_index)
    {
        return false;
    }
    let boundary = slices[index].start;
    slices[index].note_indices.iter().any(|note_index| {
        let note = &piece.notes[*note_index];
        note.score_span.start < boundary && extras & (1 << (note.midi_key % 12)) != 0
    })
}

fn is_anticipation(
    piece: &CorpusPiece,
    slices: &[SonoritySlice],
    index: usize,
    harmony_index: usize,
    extras: u16,
) -> bool {
    if extras.count_ones() != 1
        || index + 1 >= slices.len()
        || slices[index + 1].harmony_index == Some(harmony_index)
    {
        return false;
    }
    let Some(next_harmony) = slices[index + 1]
        .harmony_index
        .and_then(|i| annotated_chord_mask(&piece.harmonies[i]))
    else {
        return false;
    };
    let boundary = slices[index].end;
    extras & next_harmony != 0
        && slices[index].note_indices.iter().any(|note_index| {
            let note = &piece.notes[*note_index];
            note.score_span.end > boundary && extras & (1 << (note.midi_key % 12)) != 0
        })
}

fn is_passing_or_neighbor(
    piece: &CorpusPiece,
    slice: &SonoritySlice,
    extras: u16,
    expected: u16,
) -> bool {
    if extras.count_ones() != 1 {
        return false;
    }
    slice.note_indices.iter().any(|note_index| {
        let note = &piece.notes[*note_index];
        if note.score_span.start != slice.start || extras & (1 << (note.midi_key % 12)) == 0 {
            return false;
        }
        let same_voice = |candidate: &&crate::model::NoteEvent| {
            candidate.staff == note.staff
                && candidate.voice == note.voice
                && candidate.source.rows != note.source.rows
        };
        let previous = piece
            .notes
            .iter()
            .filter(same_voice)
            .filter(|candidate| candidate.score_span.end == note.score_span.start)
            .max_by_key(|candidate| candidate.score_span.start);
        let next = piece
            .notes
            .iter()
            .filter(same_voice)
            .filter(|candidate| candidate.score_span.start == note.score_span.end)
            .min_by_key(|candidate| candidate.score_span.end);
        let (Some(previous), Some(next)) = (previous, next) else {
            return false;
        };
        let neighbors_are_chord_tones = expected & (1 << (previous.midi_key % 12)) != 0
            && expected & (1 << (next.midi_key % 12)) != 0;
        let stepwise = previous.midi_key.abs_diff(note.midi_key) <= 2
            && note.midi_key.abs_diff(next.midi_key) <= 2;
        let passing = (previous.midi_key < note.midi_key && note.midi_key < next.midi_key)
            || (previous.midi_key > note.midi_key && note.midi_key > next.midi_key);
        let neighbor = previous.midi_key == next.midi_key;
        neighbors_are_chord_tones && stepwise && (passing || neighbor)
    })
}

pub fn write_divergence_tsv(
    piece: &CorpusPiece,
    sonorities: &SonorityReport,
    report: &DivergenceReport,
    writer: impl Write,
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    writer.write_record([
        "corpus",
        "piece",
        "slice_index",
        "start_qb",
        "end_qb",
        "duration_qb",
        "measure_count",
        "measure_number",
        "metric_weight",
        "cardinality",
        "pc_mask",
        "expected_pc_mask",
        "literal_root_pc",
        "annotated_root_pc",
        "literal_bass_pc",
        "annotated_bass_pc",
        "non_chord_pc_mask",
        "missing_pc_mask",
        "category",
        "reason",
        "harmony_label",
        "chord_type",
        "active_note_rows",
        "harmony_rows",
    ])?;
    for record in &report.records {
        let slice = &sonorities.slices[record.slice_index];
        let measure = slice.measure_index.map(|i| &piece.measures[i]);
        let harmony = slice.harmony_index.map(|i| &piece.harmonies[i]);
        writer.write_record([
            piece.corpus.clone(),
            piece.piece.clone(),
            record.slice_index.to_string(),
            slice.start.to_string(),
            slice.end.to_string(),
            slice.duration().to_string(),
            measure.map(|m| m.count.to_string()).unwrap_or_default(),
            measure.map(|m| m.number.clone()).unwrap_or_default(),
            format!("{:?}", slice.metric_weight),
            slice.cardinality.to_string(),
            mask(slice.pitch_class_mask),
            optional_mask(record.expected_pitch_class_mask),
            optional_number(record.literal_root_pitch_class),
            optional_number(record.annotated_root_pitch_class),
            optional_number(record.literal_bass_pitch_class),
            optional_number(record.annotated_bass_pitch_class),
            optional_mask(record.non_chord_pitch_class_mask),
            optional_mask(record.missing_pitch_class_mask),
            record.category.to_string(),
            record.reason.to_owned(),
            harmony.map(|h| h.label.clone()).unwrap_or_default(),
            harmony
                .and_then(|h| h.chord_type.clone())
                .unwrap_or_default(),
            source_rows(
                slice
                    .note_indices
                    .iter()
                    .flat_map(|i| piece.notes[*i].source.rows.iter().copied()),
            ),
            harmony
                .map(|h| source_rows(h.source.rows.iter().copied()))
                .unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_divergence_summary_tsv(report: &DivergenceReport, writer: impl Write) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    writer.write_record([
        "category",
        "cardinality",
        "metric_weight",
        "harmony_label",
        "slice_count",
        "total_duration_qb",
    ])?;
    for row in &report.summary {
        writer.write_record([
            row.key.category.to_string(),
            row.key.cardinality.to_string(),
            format!("{:?}", row.key.metric_weight),
            row.key.harmony_label.clone(),
            row.slice_count.to_string(),
            row.total_duration.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn mask(value: u16) -> String {
    format!("0x{value:03x}")
}
fn optional_mask(value: Option<u16>) -> String {
    value.map(mask).unwrap_or_default()
}
fn optional_number(value: Option<u8>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}
fn source_rows(rows: impl Iterator<Item = u64>) -> String {
    rows.map(|row| row.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ExactScoreSpan, SourceRef};

    fn harmony(changes: Option<&str>) -> HarmonyAnnotation {
        HarmonyAnnotation {
            source: SourceRef {
                corpus: "fixture".into(),
                piece: "changes".into(),
                facet: "expanded",
                rows: vec![17],
            },
            score_span: ExactScoreSpan::new(ScoreTime::ZERO, ScoreTime::from_integer(1)).unwrap(),
            label: changes
                .map(|changes| format!("I({changes})"))
                .unwrap_or_else(|| "I".into()),
            global_key: Some("C".into()),
            local_key: Some("I".into()),
            numeral: Some("I".into()),
            chord_type: Some("M".into()),
            figured_bass: None,
            changes: changes.map(str::to_owned),
            root_fifths: Some(0),
            bass_fifths: Some(0),
            cadence: None,
            phrase_end: None,
        }
    }

    #[test]
    fn supported_dcml_changes_produce_exact_pitch_class_sets() {
        let cases = [
            ("9", 0x095),
            ("4", 0x0a1),
            ("64", 0x221),
            ("#7b64", 0x121),
            ("974", 0x8a4),
        ];

        for (changes, expected) in cases {
            let annotation = harmony(Some(changes));
            assert_eq!(
                annotated_chord_mask(&annotation),
                Some(expected),
                "{changes}"
            );
            assert_eq!(annotation.changes.as_deref(), Some(changes));
            assert_eq!(annotation.label, format!("I({changes})"));
            assert_eq!(annotation.source.rows, [17]);
        }
    }

    #[test]
    fn unsupported_or_ambiguous_changes_remain_unresolved() {
        for changes in ["+9", "94", "11", " 9", "9 ", "6#4"] {
            assert_eq!(annotated_chord_mask(&harmony(Some(changes))), None);
        }
    }

    #[test]
    fn unmodified_chord_derivation_is_unchanged() {
        assert_eq!(annotated_chord_mask(&harmony(None)), Some(0x091));
    }

    #[test]
    fn changed_intervals_follow_the_local_diatonic_scale() {
        let mut major_subdominant = harmony(Some("4"));
        major_subdominant.label = "iv(4)".into();
        major_subdominant.numeral = Some("iv".into());
        major_subdominant.chord_type = Some("m".into());
        major_subdominant.root_fifths = Some(-1);
        assert_eq!(annotated_chord_mask(&major_subdominant), Some(0x821));

        let mut minor_subdominant = major_subdominant;
        minor_subdominant.global_key = Some("c".into());
        minor_subdominant.local_key = Some("i".into());
        assert_eq!(annotated_chord_mask(&minor_subdominant), Some(0x421));
    }
}
