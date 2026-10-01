use anyhow::{bail, Context, Result};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use music::notation::lilypond::command::{LilypondCmdBuilder, LilypondOutput};
use music::notation::lilypond::document::score::{
    LilypondLayout, LilypondMidi, LilypondScore, LilypondStaffGroup,
};
use music::notation::lilypond::document::staff::LilypondStaff;
use music::notation::lilypond::document::{LilypondBuilder, LilypondHeader};
use music::notation::rhythm::duration::Duration;
use music::notation::rhythm::RhythmicNotatedEvent;
use music::{Pitch, Voicing};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

struct Selection {
    slug: &'static str,
    slice: &'static str,
    context_slices: &'static [&'static str],
    start: &'static str,
    end: &'static str,
    category: &'static str,
    label: &'static str,
    note_rows: &'static str,
    harmony_rows: &'static str,
}

// These are source-row identities from the completed manual K282-1 audit. The
// generator rejects changed rows instead of silently choosing a new example.
const SELECTIONS: &[Selection] = &[
    Selection {
        slug: "agreement-s0",
        slice: "0",
        context_slices: &["0", "1", "2"],
        start: "0",
        end: "3/4",
        category: "Agreement",
        label: "Eb.I{",
        note_rows: "4,2,3",
        harmony_rows: "2",
    },
    Selection {
        slug: "passing-neighbor-s1",
        slice: "1",
        context_slices: &["0", "1", "2"],
        start: "3/4",
        end: "1",
        category: "PassingOrNeighbor",
        label: "Eb.I{",
        note_rows: "2,3,5",
        harmony_rows: "2",
    },
    Selection {
        slug: "suspension-s586",
        slice: "586",
        context_slices: &["585", "586", "587"],
        start: "138",
        end: "829/6",
        category: "Suspension",
        label: "I",
        note_rows: "965,968,966,967",
        harmony_rows: "100",
    },
    Selection {
        slug: "bass-inversion-mismatch-s428",
        slice: "428",
        context_slices: &["427", "428", "429"],
        start: "421/4",
        end: "211/2",
        category: "BassOrInversionMismatch",
        label: "I{",
        note_rows: "690,691,692",
        harmony_rows: "74",
    },
    Selection {
        slug: "incomplete-bass-mismatch-s27",
        slice: "27",
        context_slices: &["26", "27", "28"],
        start: "49/4",
        end: "25/2",
        category: "IncompleteHarmonyBassMismatch",
        label: "I|IAC}{",
        note_rows: "50,51",
        harmony_rows: "14",
    },
];

struct Table {
    headers: StringRecord,
    rows: Vec<StringRecord>,
}

impl Table {
    fn read(path: &Path) -> Result<Self> {
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(path)
            .with_context(|| format!("cannot open source TSV {}", path.display()))?;
        let headers = reader
            .headers()
            .with_context(|| format!("cannot read header from {}", path.display()))?
            .clone();
        let rows = reader
            .records()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("cannot parse source TSV {}", path.display()))?;
        Ok(Self { headers, rows })
    }

    fn column(&self, name: &str) -> Result<usize> {
        self.headers
            .iter()
            .position(|header| header == name)
            .with_context(|| format!("source TSV lacks required {name:?} column"))
    }
}

fn main() -> Result<()> {
    let (details_path, notes_path, output_dir) = arguments()?;
    let details_path = details_path.canonicalize().with_context(|| {
        format!(
            "divergence detail TSV is absent: {}",
            details_path.display()
        )
    })?;
    let notes_path = notes_path
        .canonicalize()
        .with_context(|| format!("DCML notes TSV is absent: {}", notes_path.display()))?;
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("cannot create output directory {}", output_dir.display()))?;

    let details = Table::read(&details_path)?;
    let notes = Table::read(&notes_path)?;
    let midi_column = notes.column("midi")?;
    let mut manifest = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output_dir.join("manifest.tsv"))
        .context("cannot create engraving manifest")?;
    manifest.write_record([
        "slug",
        "corpus",
        "piece",
        "slice_index",
        "start_qb",
        "end_qb",
        "category",
        "harmony_label",
        "active_note_rows",
        "harmony_rows",
        "context_slices",
        "detail_tsv",
        "notes_tsv",
    ])?;

    for selection in SELECTIONS {
        let target = detail_row(&details, selection.slice)?;
        verify_selection(&details, target, selection)?;
        let events = selection
            .context_slices
            .iter()
            .map(|slice| detail_row(&details, slice))
            .map(|row| row.and_then(|row| event_for_row(&details, &notes, midi_column, row)))
            .collect::<Result<Vec<_>>>()?;

        let title = format!(
            "{} — {} — qb [{}, {})",
            display_category(selection.category),
            selection.label,
            selection.start,
            selection.end
        );
        let provenance = format!(
            "slice {}; notes {}; harmony {}",
            selection.slice, selection.note_rows, selection.harmony_rows
        );
        let staff = LilypondStaff::new().add_voice(events);
        let score = LilypondScore::new()
            .staff_group(LilypondStaffGroup::new(vec![staff]))
            .layout(Some(LilypondLayout::new().ragged_right(true)))
            .midi(Some(
                LilypondMidi::new().tempo(72).instrument("acoustic grand"),
            ));
        let source_path = output_dir.join(format!("{}.ly", selection.slug));
        let document = LilypondBuilder::new()
            .path(Some(source_path))
            .header(Some(
                LilypondHeader::new()
                    .title(Some(lilypond_string(&title)))
                    .composer(Some(lilypond_string(&provenance))),
            ))
            .score(Some(score));
        LilypondCmdBuilder::new()
            .formats(vec![LilypondOutput::Pdf])
            .output(Some(output_dir.join(selection.slug)))
            .builder(document)
            .build_and_compile()
            .with_context(|| format!("failed to engrave {} with LilyPond", selection.slug))?;

        let context_slices = selection.context_slices.join(",");
        let detail_source = details_path.to_string_lossy();
        let notes_source = notes_path.to_string_lossy();
        manifest.write_record([
            selection.slug,
            "dcml",
            "K282-1",
            selection.slice,
            selection.start,
            selection.end,
            selection.category,
            selection.label,
            selection.note_rows,
            selection.harmony_rows,
            &context_slices,
            detail_source.as_ref(),
            notes_source.as_ref(),
        ])?;
    }
    manifest.flush()?;
    Ok(())
}

fn arguments() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let args = std::env::args_os().skip(1).collect::<Vec<OsString>>();
    if args.len() != 3 {
        bail!(
            "usage: cargo run -p music-corpus --example k282_divergence_engraving -- \
             DETAILS.tsv NOTES.tsv OUTPUT_DIR"
        );
    }
    Ok((
        PathBuf::from(&args[0]),
        PathBuf::from(&args[1]),
        PathBuf::from(&args[2]),
    ))
}

fn detail_row<'a>(table: &'a Table, slice: &str) -> Result<&'a StringRecord> {
    let corpus = table.column("corpus")?;
    let piece = table.column("piece")?;
    let slice_index = table.column("slice_index")?;
    let matches = table
        .rows
        .iter()
        .filter(|row| {
            row.get(corpus) == Some("dcml")
                && row.get(piece) == Some("K282-1")
                && row.get(slice_index) == Some(slice)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [row] => Ok(*row),
        [] => bail!("validated K282-1 slice {slice} is absent from the detail TSV"),
        _ => bail!("validated K282-1 slice {slice} is duplicated in the detail TSV"),
    }
}

fn verify_selection(table: &Table, row: &StringRecord, selection: &Selection) -> Result<()> {
    for (column, expected) in [
        ("start_qb", selection.start),
        ("end_qb", selection.end),
        ("category", selection.category),
        ("harmony_label", selection.label),
        ("active_note_rows", selection.note_rows),
        ("harmony_rows", selection.harmony_rows),
    ] {
        let actual = row.get(table.column(column)?).unwrap_or_default();
        if actual != expected {
            bail!(
                "validated selection {} drifted: {column} is {actual:?}, expected {expected:?}",
                selection.slug
            );
        }
    }
    Ok(())
}

fn event_for_row(
    details: &Table,
    notes: &Table,
    midi_column: usize,
    row: &StringRecord,
) -> Result<music::notation::lilypond::staff_elements::LilypondVoiceElement<'static>> {
    let source_rows = row
        .get(details.column("active_note_rows")?)
        .unwrap_or_default();
    let pitches = source_rows
        .split(',')
        .filter(|value| !value.is_empty())
        .map(|value| {
            let source_row = value
                .parse::<usize>()
                .with_context(|| format!("invalid note source row {value:?}"))?;
            let note = notes
                .rows
                .get(
                    source_row
                        .checked_sub(2)
                        .context("note source row precedes data")?,
                )
                .with_context(|| format!("note source row {source_row} is absent"))?;
            let midi = note
                .get(midi_column)
                .context("note row lacks MIDI value")?
                .parse::<u8>()
                .with_context(|| format!("invalid MIDI value at note source row {source_row}"))?;
            Pitch::from_midi_spelled(midi, false)
                .with_context(|| format!("cannot spell MIDI note at source row {source_row}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let event = match pitches.as_slice() {
        [] => RhythmicNotatedEvent::rest(Duration::QTR),
        [pitch] => RhythmicNotatedEvent::pitch(*pitch, Duration::QTR),
        _ => RhythmicNotatedEvent::voicing(Voicing::new(pitches), Duration::QTR),
    };
    Ok(event.into())
}

fn display_category(category: &str) -> &str {
    match category {
        "PassingOrNeighbor" => "Passing/neighbor",
        "BassOrInversionMismatch" => "Bass/inversion mismatch",
        "IncompleteHarmonyBassMismatch" => "Incomplete harmony, bass mismatch",
        other => other,
    }
}

fn lilypond_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
