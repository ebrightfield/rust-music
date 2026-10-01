use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use csv::{ReaderBuilder, StringRecord};

use crate::model::{
    CorpusPiece, ExactScoreSpan, HarmonyAnnotation, Measure, NoteEvent, ScoreTime, SourceRef,
};

pub struct DcmlPaths<'a> {
    pub notes: &'a Path,
    pub measures: &'a Path,
    pub expanded: &'a Path,
}

pub struct DcmlAdapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChordToneChange {
    pub interval: u8,
    pub accidental: i8,
    pub replaces: Option<usize>,
}

const NINTH: ChordToneChange = ChordToneChange {
    interval: 9,
    accidental: 0,
    replaces: None,
};
const SEVENTH: ChordToneChange = ChordToneChange {
    interval: 7,
    accidental: 0,
    replaces: Some(0),
};
const SHARP_SEVENTH: ChordToneChange = ChordToneChange {
    interval: 7,
    accidental: 1,
    replaces: Some(0),
};
const FOURTH: ChordToneChange = ChordToneChange {
    interval: 4,
    accidental: 0,
    replaces: Some(1),
};
const FLAT_SIXTH: ChordToneChange = ChordToneChange {
    interval: 6,
    accidental: -1,
    replaces: Some(2),
};
const SIXTH: ChordToneChange = ChordToneChange {
    interval: 6,
    accidental: 0,
    replaces: Some(2),
};

/// Parses only change spellings whose pitch-class semantics are unambiguous
/// for the currently supported DCML data. The digits are separate descending
/// diatonic intervals, not a decimal number (`64` means `6` plus `4`).
pub(crate) fn supported_chord_changes(changes: Option<&str>) -> Option<&'static [ChordToneChange]> {
    match changes {
        None | Some("") => Some(&[]),
        Some("9") => Some(&[NINTH]),
        Some("4") => Some(&[FOURTH]),
        Some("64") => Some(&[SIXTH, FOURTH]),
        Some("#7b64") => Some(&[SHARP_SEVENTH, FLAT_SIXTH, FOURTH]),
        Some("974") => Some(&[NINTH, SEVENTH, FOURTH]),
        Some(_) => None,
    }
}

impl DcmlAdapter {
    pub fn load_piece(paths: DcmlPaths<'_>, corpus: &str, piece: &str) -> Result<CorpusPiece> {
        let measures = read_measures(paths.measures, corpus, piece)?;
        let notes = read_notes(paths.notes, corpus, piece)?;

        if notes.is_empty() {
            bail!("no note rows found for {corpus}/{piece}");
        }
        if measures.is_empty() {
            bail!("no measure rows found for {corpus}/{piece}");
        }

        let piece_end = measures
            .iter()
            .map(|measure| measure.score_span.end)
            .max()
            .expect("measures were checked non-empty");
        let harmonies = read_harmonies(paths.expanded, corpus, piece, piece_end)?;

        validate_measures(&measures)?;
        validate_notes_within_piece(&notes, piece_end)?;

        Ok(CorpusPiece {
            corpus: corpus.into(),
            piece: piece.into(),
            notes,
            measures,
            harmonies,
        })
    }
}

struct Rows {
    reader: csv::Reader<std::fs::File>,
    columns: HashMap<String, usize>,
}

impl Rows {
    fn open(path: &Path) -> Result<Self> {
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .from_path(path)
            .with_context(|| format!("opening {}", path.display()))?;
        let columns = reader
            .headers()
            .with_context(|| format!("reading headers from {}", path.display()))?
            .iter()
            .enumerate()
            .map(|(index, name)| (name.to_owned(), index))
            .collect();
        Ok(Self { reader, columns })
    }

    fn required<'a>(&self, row: &'a StringRecord, column: &str) -> Result<&'a str> {
        self.optional(row, column)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing required column value {column:?}"))
    }

    fn optional<'a>(&self, row: &'a StringRecord, column: &str) -> Option<&'a str> {
        self.columns.get(column).and_then(|index| row.get(*index))
    }

    fn matches(&self, row: &StringRecord, corpus: &str, piece: &str) -> bool {
        self.optional(row, "corpus")
            .is_none_or(|value| value == corpus)
            && self
                .optional(row, "piece")
                .is_none_or(|value| value == piece)
    }

    fn row_id(&self, row: &StringRecord, physical_row: u64) -> Result<u64> {
        self.optional(row, "i")
            .filter(|value| !value.is_empty())
            .map(|value| parse_u64(value, "i"))
            .unwrap_or(Ok(physical_row))
    }
}

fn read_notes(path: &Path, corpus: &str, piece: &str) -> Result<Vec<NoteEvent>> {
    struct PendingNote {
        event: NoteEvent,
        tie: Option<i8>,
    }

    let mut rows = Rows::open(path)?;
    let mut pending = Vec::new();
    let mut physical_row = 1_u64;
    while let Some(record) = rows.reader.records().next() {
        let record = record.with_context(|| format!("reading note row from {}", path.display()))?;
        physical_row += 1;
        if !rows.matches(&record, corpus, piece) {
            continue;
        }
        let row = rows.row_id(&record, physical_row)?;
        let start = parse_time(rows.required(&record, "quarterbeats")?, "quarterbeats", row)?;
        let whole_note_duration = parse_time(rows.required(&record, "duration")?, "duration", row)?;
        let duration = whole_note_duration * 4;
        let end = start + duration;
        let is_grace = duration == ScoreTime::ZERO
            || rows
                .optional(&record, "gracenote")
                .is_some_and(|value| !value.is_empty());
        let midi = parse_u8(rows.required(&record, "midi")?, "midi")?;
        if midi > 127 {
            bail!("note row {row} has MIDI key outside 0..=127: {midi}");
        }
        pending.push(PendingNote {
            event: NoteEvent {
                source: SourceRef {
                    corpus: corpus.into(),
                    piece: piece.into(),
                    facet: "notes",
                    rows: vec![row],
                },
                score_span: ExactScoreSpan::new(start, end)
                    .with_context(|| format!("invalid note span at row {row}"))?,
                midi_key: midi,
                tonal_pitch_class: parse_optional(&rows, &record, "tpc", parse_i32)?,
                staff: parse_optional(&rows, &record, "staff", parse_u16)?,
                voice: parse_optional(&rows, &record, "voice", parse_u8)?,
                is_grace,
            },
            tie: parse_optional(&rows, &record, "tied", parse_i8)?,
        });
    }

    pending.sort_by_key(|note| {
        (
            note.event.staff,
            note.event.voice,
            note.event.midi_key,
            note.event.score_span.start,
        )
    });
    let mut merged: Vec<PendingNote> = Vec::with_capacity(pending.len());
    for note in pending {
        let can_continue = matches!(note.tie, Some(-1 | 0));
        if can_continue {
            if let Some(previous) = merged.last_mut() {
                let previous_continues = matches!(previous.tie, Some(0 | 1));
                let same_lane = previous.event.staff == note.event.staff
                    && previous.event.voice == note.event.voice
                    && previous.event.midi_key == note.event.midi_key;
                if previous_continues
                    && same_lane
                    && previous.event.score_span.end == note.event.score_span.start
                {
                    previous.event.score_span.end = note.event.score_span.end;
                    previous.event.source.rows.extend(note.event.source.rows);
                    previous.tie = note.tie;
                    continue;
                }
            }
        }
        merged.push(note);
    }

    let mut notes: Vec<_> = merged.into_iter().map(|note| note.event).collect();
    notes.sort_by_key(|note| (note.score_span.start, note.score_span.end, note.midi_key));
    Ok(notes)
}

fn read_measures(path: &Path, corpus: &str, piece: &str) -> Result<Vec<Measure>> {
    let mut rows = Rows::open(path)?;
    let mut measures = Vec::new();
    let mut physical_row = 1_u64;
    while let Some(record) = rows.reader.records().next() {
        let record =
            record.with_context(|| format!("reading measure row from {}", path.display()))?;
        physical_row += 1;
        if !rows.matches(&record, corpus, piece) {
            continue;
        }
        let row = rows.row_id(&record, physical_row)?;
        let start = parse_time(rows.required(&record, "quarterbeats")?, "quarterbeats", row)?;
        let duration =
            if let Some(actual) = rows.optional(&record, "act_dur").filter(|v| !v.is_empty()) {
                parse_time(actual, "act_dur", row)? * 4
            } else {
                parse_time(rows.required(&record, "duration_qb")?, "duration_qb", row)?
            };
        let signature = rows
            .optional(&record, "timesig")
            .filter(|value| !value.is_empty())
            .map(str::parse)
            .transpose()
            .with_context(|| format!("invalid time signature at measure row {row}"))?;
        measures.push(Measure {
            source: SourceRef {
                corpus: corpus.into(),
                piece: piece.into(),
                facet: "measures",
                rows: vec![row],
            },
            count: parse_i64(rows.required(&record, "mc")?, "mc")?,
            number: rows.optional(&record, "mn").unwrap_or_default().into(),
            score_span: ExactScoreSpan::new(start, start + duration)
                .with_context(|| format!("invalid measure span at row {row}"))?,
            time_signature: signature,
        });
    }
    measures.sort_by_key(|measure| (measure.score_span.start, measure.count));
    Ok(measures)
}

fn read_harmonies(
    path: &Path,
    corpus: &str,
    piece: &str,
    piece_end: ScoreTime,
) -> Result<Vec<HarmonyAnnotation>> {
    #[derive(Debug)]
    struct Pending {
        row: u64,
        start: ScoreTime,
        label: String,
        global_key: Option<String>,
        local_key: Option<String>,
        numeral: Option<String>,
        chord_type: Option<String>,
        figured_bass: Option<String>,
        changes: Option<String>,
        root_fifths: Option<i32>,
        bass_fifths: Option<i32>,
        cadence: Option<String>,
        phrase_end: Option<String>,
    }

    let mut rows = Rows::open(path)?;
    let mut pending = Vec::new();
    let mut physical_row = 1_u64;
    while let Some(record) = rows.reader.records().next() {
        let record =
            record.with_context(|| format!("reading harmony row from {}", path.display()))?;
        physical_row += 1;
        if !rows.matches(&record, corpus, piece) {
            continue;
        }
        let row = rows.row_id(&record, physical_row)?;
        pending.push(Pending {
            row,
            start: parse_time(rows.required(&record, "quarterbeats")?, "quarterbeats", row)?,
            label: rows.optional(&record, "label").unwrap_or_default().into(),
            global_key: owned_optional(&rows, &record, "globalkey"),
            local_key: owned_optional(&rows, &record, "localkey"),
            numeral: owned_optional(&rows, &record, "numeral"),
            chord_type: owned_optional(&rows, &record, "chord_type"),
            figured_bass: owned_optional(&rows, &record, "figbass"),
            changes: owned_optional(&rows, &record, "changes"),
            root_fifths: parse_optional(&rows, &record, "root", parse_i32)?,
            bass_fifths: parse_optional(&rows, &record, "bass_note", parse_i32)?,
            cadence: owned_optional(&rows, &record, "cadence"),
            phrase_end: owned_optional(&rows, &record, "phraseend"),
        });
    }
    pending.sort_by_key(|harmony| (harmony.start, harmony.row));
    let starts: Vec<_> = pending.iter().map(|harmony| harmony.start).collect();

    pending
        .into_iter()
        .enumerate()
        .map(|(index, harmony)| {
            let end = starts.get(index + 1).copied().unwrap_or(piece_end);
            if end < harmony.start {
                bail!("harmony row {} ends before it begins", harmony.row);
            }
            Ok(HarmonyAnnotation {
                source: SourceRef {
                    corpus: corpus.into(),
                    piece: piece.into(),
                    facet: "expanded",
                    rows: vec![harmony.row],
                },
                score_span: ExactScoreSpan::new(harmony.start, end)?,
                label: harmony.label,
                global_key: harmony.global_key,
                local_key: harmony.local_key,
                numeral: harmony.numeral,
                chord_type: harmony.chord_type,
                figured_bass: harmony.figured_bass,
                changes: harmony.changes,
                root_fifths: harmony.root_fifths,
                bass_fifths: harmony.bass_fifths,
                cadence: harmony.cadence,
                phrase_end: harmony.phrase_end,
            })
        })
        .collect()
}

fn validate_measures(measures: &[Measure]) -> Result<()> {
    for pair in measures.windows(2) {
        if pair[0].score_span.end != pair[1].score_span.start {
            bail!(
                "measure spans do not close exactly between measure counts {} and {}: {} != {}",
                pair[0].count,
                pair[1].count,
                pair[0].score_span.end,
                pair[1].score_span.start,
            );
        }
    }
    Ok(())
}

fn validate_notes_within_piece(notes: &[NoteEvent], piece_end: ScoreTime) -> Result<()> {
    if let Some(note) = notes.iter().find(|note| note.score_span.end > piece_end) {
        bail!(
            "note row {:?} ends at {} after piece end {}",
            note.source.rows,
            note.score_span.end,
            piece_end
        );
    }
    Ok(())
}

fn owned_optional(rows: &Rows, record: &StringRecord, column: &str) -> Option<String> {
    rows.optional(record, column)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn parse_optional<T>(
    rows: &Rows,
    record: &StringRecord,
    column: &str,
    parse: impl Fn(&str, &str) -> Result<T>,
) -> Result<Option<T>> {
    rows.optional(record, column)
        .filter(|value| !value.is_empty())
        .map(|value| parse(value, column))
        .transpose()
}

fn parse_time(value: &str, column: &str, row: u64) -> Result<ScoreTime> {
    value
        .parse()
        .with_context(|| format!("invalid {column} at row {row}: {value:?}"))
}

fn parse_u8(value: &str, column: &str) -> Result<u8> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}

fn parse_u16(value: &str, column: &str) -> Result<u16> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}

fn parse_u64(value: &str, column: &str) -> Result<u64> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}
fn parse_i8(value: &str, column: &str) -> Result<i8> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}

fn parse_i32(value: &str, column: &str) -> Result<i32> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}

fn parse_i64(value: &str, column: &str) -> Result<i64> {
    value
        .parse()
        .with_context(|| format!("invalid {column}: {value:?}"))
}
