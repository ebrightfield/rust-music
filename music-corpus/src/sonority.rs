use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
};

use anyhow::{bail, Result};
use music::{note::pitch_class::Pc, note_collections::pc_set::PcShape};
use musical_combinatorics::{FourNoteChordQuality, ThreeNoteChordQuality};

use crate::model::{CorpusPiece, HarmonyAnnotation, Measure, ScoreTime, TimeSignature};

#[derive(Debug, Clone, PartialEq)]
pub enum ClassifiedQuality {
    Three {
        rotation: usize,
        quality: ThreeNoteChordQuality,
    },
    Four {
        rotation: usize,
        quality: FourNoteChordQuality,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetricWeight {
    Downbeat,
    Secondary,
    Beat,
    Offbeat,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SonoritySlice {
    pub start: ScoreTime,
    pub end: ScoreTime,
    pub note_indices: Vec<usize>,
    pub pitch_class_mask: u16,
    pub cardinality: u8,
    pub bass_midi: Option<u8>,
    pub quality: Option<ClassifiedQuality>,
    pub measure_index: Option<usize>,
    pub measure_offset: Option<ScoreTime>,
    pub metric_weight: MetricWeight,
    pub harmony_index: Option<usize>,
}

impl SonoritySlice {
    pub fn duration(&self) -> ScoreTime {
        self.end - self.start
    }

    pub fn pitch_classes(&self) -> impl Iterator<Item = u8> + '_ {
        (0..12).filter(|pc| self.pitch_class_mask & (1 << pc) != 0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SonorityReport {
    pub slices: Vec<SonoritySlice>,
    pub cardinality_counts: [u64; 13],
    pub classified_three: u64,
    pub classified_four: u64,
    pub note_rows_accounted: usize,
    /// Source rows explicitly represented as zero-duration grace attacks.
    pub zero_duration_note_rows: usize,
}

#[derive(Default)]
struct Boundary {
    starts: Vec<usize>,
    ends: Vec<usize>,
}

pub fn analyze_piece(piece: &CorpusPiece) -> Result<SonorityReport> {
    let start = piece
        .measures
        .iter()
        .map(|measure| measure.score_span.start)
        .chain(piece.notes.iter().map(|note| note.score_span.start))
        .min()
        .unwrap_or(ScoreTime::ZERO);
    let end = piece.end();
    if end <= start {
        bail!("piece has no positive analyzed span");
    }

    let mut boundaries: BTreeMap<ScoreTime, Boundary> = BTreeMap::new();
    boundaries.entry(start).or_default();
    boundaries.entry(end).or_default();
    for (index, note) in piece.notes.iter().enumerate() {
        if note.score_span.start == note.score_span.end {
            continue;
        }
        boundaries
            .entry(note.score_span.start)
            .or_default()
            .starts
            .push(index);
        boundaries
            .entry(note.score_span.end)
            .or_default()
            .ends
            .push(index);
    }
    for span in piece
        .harmonies
        .iter()
        .map(|harmony| harmony.score_span)
        .chain(piece.measures.iter().map(|measure| measure.score_span))
    {
        if start < span.start && span.start < end {
            boundaries.entry(span.start).or_default();
        }
        if start < span.end && span.end < end {
            boundaries.entry(span.end).or_default();
        }
    }

    let points: Vec<_> = boundaries.keys().copied().collect();
    let mut active = BTreeSet::new();
    let mut slices = Vec::with_capacity(points.len().saturating_sub(1));
    let mut accounted: Vec<_> = piece
        .notes
        .iter()
        .map(|note| note.score_span.start == note.score_span.end)
        .collect();

    for pair in points.windows(2) {
        let at = pair[0];
        let next = pair[1];
        let boundary = boundaries.get(&at).expect("point came from boundaries");
        for index in &boundary.ends {
            active.remove(index);
        }
        for index in &boundary.starts {
            active.insert(*index);
        }
        if next == at {
            continue;
        }

        let note_indices: Vec<_> = active.iter().copied().collect();
        for index in &note_indices {
            accounted[*index] = true;
        }
        let pitch_class_mask = note_indices.iter().fold(0_u16, |mask, index| {
            mask | 1 << (piece.notes[*index].midi_key % 12)
        });
        let cardinality = pitch_class_mask.count_ones() as u8;
        let quality = classify(pitch_class_mask, cardinality)?;
        let bass_midi = note_indices
            .iter()
            .map(|index| piece.notes[*index].midi_key)
            .min();
        let measure_index = containing_measure(&piece.measures, at);
        let measure_offset = measure_index.map(|index| at - piece.measures[index].score_span.start);
        let metric_weight = measure_index
            .map(|index| metric_weight(&piece.measures[index], at))
            .unwrap_or(MetricWeight::Unknown);
        let harmony_index = containing_harmony(&piece.harmonies, at);

        slices.push(SonoritySlice {
            start: at,
            end: next,
            note_indices,
            pitch_class_mask,
            cardinality,
            bass_midi,
            quality,
            measure_index,
            measure_offset,
            metric_weight,
            harmony_index,
        });
    }

    if let Some(index) = accounted.iter().position(|accounted| !accounted) {
        bail!(
            "note rows {:?} did not participate in any positive-duration slice",
            piece.notes[index].source.rows
        );
    }
    verify_tiling(&slices, start, end)?;

    let mut cardinality_counts = [0_u64; 13];
    let mut classified_three = 0;
    let mut classified_four = 0;
    for slice in &slices {
        cardinality_counts[slice.cardinality as usize] += 1;
        match slice.quality {
            Some(ClassifiedQuality::Three { .. }) => classified_three += 1,
            Some(ClassifiedQuality::Four { .. }) => classified_four += 1,
            None => {}
        }
    }

    Ok(SonorityReport {
        slices,
        cardinality_counts,
        classified_three,
        classified_four,
        note_rows_accounted: piece.notes.iter().map(|note| note.source.rows.len()).sum(),
        zero_duration_note_rows: piece
            .notes
            .iter()
            .filter(|note| note.score_span.start == note.score_span.end)
            .map(|note| note.source.rows.len())
            .sum(),
    })
}

fn classify(mask: u16, cardinality: u8) -> Result<Option<ClassifiedQuality>> {
    let pcs: Vec<_> = (0_u8..12)
        .filter(|pc| mask & (1 << pc) != 0)
        .map(|pc| Pc::from(&pc))
        .collect();
    let shape = PcShape::new(pcs);
    match cardinality {
        3 => {
            let (rotation, quality) = ThreeNoteChordQuality::identify(&shape)?;
            Ok(Some(ClassifiedQuality::Three { rotation, quality }))
        }
        4 => {
            let (rotation, quality) = FourNoteChordQuality::identify(&shape)?;
            Ok(Some(ClassifiedQuality::Four { rotation, quality }))
        }
        _ => Ok(None),
    }
}

fn containing_measure(measures: &[Measure], time: ScoreTime) -> Option<usize> {
    measures
        .partition_point(|measure| measure.score_span.start <= time)
        .checked_sub(1)
        .filter(|index| measures[*index].score_span.contains(time))
}

fn containing_harmony(harmonies: &[HarmonyAnnotation], time: ScoreTime) -> Option<usize> {
    harmonies
        .partition_point(|harmony| harmony.score_span.start <= time)
        .checked_sub(1)
        .filter(|index| harmonies[*index].score_span.contains(time))
}

fn metric_weight(measure: &Measure, time: ScoreTime) -> MetricWeight {
    let Some(TimeSignature {
        numerator,
        denominator,
    }) = measure.time_signature
    else {
        return MetricWeight::Unknown;
    };
    let offset = time - measure.score_span.start;
    if offset == ScoreTime::ZERO {
        return MetricWeight::Downbeat;
    }
    let beat =
        ScoreTime::new(4, i64::from(denominator)).expect("time signature denominator is nonzero");
    let ratio = num_rational::Ratio::new(
        offset.numerator() * beat.denominator(),
        offset.denominator() * beat.numerator(),
    );
    if *ratio.denom() != 1 {
        return MetricWeight::Offbeat;
    }
    let beat_index = *ratio.numer();
    let secondary = if numerator > 3 && numerator % 3 == 0 {
        beat_index > 0 && beat_index % 3 == 0
    } else {
        numerator == 4 && beat_index == 2
    };
    if secondary {
        MetricWeight::Secondary
    } else {
        MetricWeight::Beat
    }
}

fn verify_tiling(slices: &[SonoritySlice], start: ScoreTime, end: ScoreTime) -> Result<()> {
    if slices.first().map(|slice| slice.start) != Some(start)
        || slices.last().map(|slice| slice.end) != Some(end)
    {
        bail!("sonority slices do not cover the analyzed piece span");
    }
    for pair in slices.windows(2) {
        if pair[0].end != pair[1].start {
            bail!("gap or overlap between slices at {}", pair[0].end);
        }
    }
    Ok(())
}

pub fn write_slice_tsv(
    piece: &CorpusPiece,
    report: &SonorityReport,
    writer: impl Write,
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    writer.write_record([
        "corpus",
        "piece",
        "start_qb",
        "end_qb",
        "duration_qb",
        "measure_count",
        "measure_number",
        "measure_offset_qb",
        "metric_weight",
        "active_note_rows",
        "midi_notes",
        "bass_midi",
        "pc_mask",
        "pitch_classes",
        "cardinality",
        "quality",
        "rotation",
        "harmony_label",
        "global_key",
        "local_key",
        "numeral",
        "chord_type",
        "figured_bass",
        "root_fifths",
        "bass_fifths",
        "cadence",
        "phrase_end",
    ])?;

    for slice in &report.slices {
        let measure = slice.measure_index.map(|index| &piece.measures[index]);
        let harmony = slice.harmony_index.map(|index| &piece.harmonies[index]);
        let rows = slice
            .note_indices
            .iter()
            .flat_map(|index| piece.notes[*index].source.rows.iter())
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let midi = slice
            .note_indices
            .iter()
            .map(|index| piece.notes[*index].midi_key.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let pcs = slice
            .pitch_classes()
            .map(|pc| pc.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let (quality, rotation) = match &slice.quality {
            Some(ClassifiedQuality::Three { rotation, quality }) => {
                (format!("{quality:?}"), rotation.to_string())
            }
            Some(ClassifiedQuality::Four { rotation, quality }) => {
                (format!("{quality:?}"), rotation.to_string())
            }
            None => (String::new(), String::new()),
        };

        writer.write_record([
            piece.corpus.clone(),
            piece.piece.clone(),
            slice.start.to_string(),
            slice.end.to_string(),
            slice.duration().to_string(),
            measure.map(|m| m.count.to_string()).unwrap_or_default(),
            measure.map(|m| m.number.clone()).unwrap_or_default(),
            slice
                .measure_offset
                .map(|time| time.to_string())
                .unwrap_or_default(),
            format!("{:?}", slice.metric_weight),
            rows,
            midi,
            slice
                .bass_midi
                .map(|value| value.to_string())
                .unwrap_or_default(),
            format!("0x{:03x}", slice.pitch_class_mask),
            pcs,
            slice.cardinality.to_string(),
            quality,
            rotation,
            harmony.map(|h| h.label.clone()).unwrap_or_default(),
            harmony
                .and_then(|h| h.global_key.clone())
                .unwrap_or_default(),
            harmony
                .and_then(|h| h.local_key.clone())
                .unwrap_or_default(),
            harmony.and_then(|h| h.numeral.clone()).unwrap_or_default(),
            harmony
                .and_then(|h| h.chord_type.clone())
                .unwrap_or_default(),
            harmony
                .and_then(|h| h.figured_bass.clone())
                .unwrap_or_default(),
            harmony
                .and_then(|h| h.root_fifths)
                .map(|v| v.to_string())
                .unwrap_or_default(),
            harmony
                .and_then(|h| h.bass_fifths)
                .map(|v| v.to_string())
                .unwrap_or_default(),
            harmony.and_then(|h| h.cadence.clone()).unwrap_or_default(),
            harmony
                .and_then(|h| h.phrase_end.clone())
                .unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}
