use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    analyze_divergence, analyze_piece, DcmlAdapter, DcmlPaths, DivergenceSummaryKey,
    DivergenceSummaryRow, ScoreTime,
};

/// The divergence summary for one explicitly selected DCML piece.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceDivergenceSummary {
    pub corpus: String,
    pub piece: String,
    pub summary: Vec<DivergenceSummaryRow>,
}

/// Deterministically ordered divergence summaries for an explicit set of pieces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcmlDivergenceBatch {
    pub pieces: Vec<PieceDivergenceSummary>,
}

#[derive(Debug)]
struct ManifestEntry {
    corpus: String,
    piece: String,
    notes: PathBuf,
    measures: PathBuf,
    expanded: PathBuf,
    row: usize,
}

/// Loads and analyzes every piece named by a tab-separated manifest.
///
/// The manifest must contain `corpus`, `piece`, `notes`, `measures`, and
/// `expanded` columns. Relative source paths are resolved from the manifest's
/// directory. Pieces are analyzed and returned in `(corpus, piece)` order.
pub fn analyze_dcml_divergence_manifest(path: &Path) -> Result<DcmlDivergenceBatch> {
    let entries = read_manifest(path)?;
    let mut pieces = Vec::with_capacity(entries.len());

    for entry in entries {
        let identity = format!("{}/{}", entry.corpus, entry.piece);
        let piece = DcmlAdapter::load_piece(
            DcmlPaths {
                notes: &entry.notes,
                measures: &entry.measures,
                expanded: &entry.expanded,
            },
            &entry.corpus,
            &entry.piece,
        )
        .with_context(|| {
            format!(
                "loading DCML piece {identity} from manifest row {}",
                entry.row
            )
        })?;
        let sonorities = analyze_piece(&piece).with_context(|| {
            format!(
                "analyzing sonorities for DCML piece {identity} from manifest row {}",
                entry.row
            )
        })?;
        let divergence = analyze_divergence(&piece, &sonorities);
        pieces.push(PieceDivergenceSummary {
            corpus: entry.corpus,
            piece: entry.piece,
            summary: divergence.summary,
        });
    }

    Ok(DcmlDivergenceBatch { pieces })
}

fn read_manifest(path: &Path) -> Result<Vec<ManifestEntry>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(path)
        .with_context(|| format!("opening DCML batch manifest {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| {
            format!(
                "reading headers from DCML batch manifest {}",
                path.display()
            )
        })?
        .clone();
    let required_column = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .ok_or_else(|| anyhow!("DCML batch manifest is missing required column {name:?}"))
    };
    let corpus = required_column("corpus")?;
    let piece = required_column("piece")?;
    let notes = required_column("notes")?;
    let measures = required_column("measures")?;
    let expanded = required_column("expanded")?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut entries = Vec::new();

    for (index, record) in reader.records().enumerate() {
        let row = index + 2;
        let record = record.with_context(|| {
            format!(
                "reading row {row} from DCML batch manifest {}",
                path.display()
            )
        })?;
        let required = |column: usize, name: &str| -> Result<&str> {
            record
                .get(column)
                .filter(|value| !value.trim().is_empty())
                .map(str::trim)
                .ok_or_else(|| anyhow!("manifest row {row} has no {name:?} value"))
        };
        entries.push(ManifestEntry {
            corpus: required(corpus, "corpus")?.to_owned(),
            piece: required(piece, "piece")?.to_owned(),
            notes: resolve_path(base, required(notes, "notes")?),
            measures: resolve_path(base, required(measures, "measures")?),
            expanded: resolve_path(base, required(expanded, "expanded")?),
            row,
        });
    }

    if entries.is_empty() {
        bail!("DCML batch manifest {} contains no pieces", path.display());
    }
    entries.sort_by(|left, right| (&left.corpus, &left.piece).cmp(&(&right.corpus, &right.piece)));
    for duplicate in entries.windows(2) {
        if duplicate[0].corpus == duplicate[1].corpus && duplicate[0].piece == duplicate[1].piece {
            bail!(
                "DCML batch manifest contains duplicate piece {}/{} at rows {} and {}",
                duplicate[0].corpus,
                duplicate[0].piece,
                duplicate[0].row,
                duplicate[1].row
            );
        }
    }
    Ok(entries)
}

fn resolve_path(base: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

/// Writes one row per piece-specific divergence summary effect.
pub fn write_piece_divergence_summaries_tsv(
    batch: &DcmlDivergenceBatch,
    writer: impl Write,
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    writer.write_record([
        "corpus",
        "piece",
        "category",
        "cardinality",
        "metric_weight",
        "harmony_label",
        "slice_count",
        "total_duration_qb",
    ])?;
    for piece in &batch.pieces {
        for row in &piece.summary {
            writer.write_record([
                piece.corpus.clone(),
                piece.piece.clone(),
                row.key.category.to_string(),
                row.key.cardinality.to_string(),
                format!("{:?}", row.key.metric_weight),
                row.key.harmony_label.clone(),
                row.slice_count.to_string(),
                row.total_duration.to_string(),
            ])?;
        }
    }
    writer.flush()?;
    Ok(())
}

/// Writes summary effects aggregated across all selected pieces in each corpus.
pub fn write_corpus_divergence_summary_tsv(
    batch: &DcmlDivergenceBatch,
    writer: impl Write,
) -> Result<()> {
    let mut aggregate: BTreeMap<(String, DivergenceSummaryKey), (u64, u64, ScoreTime)> =
        BTreeMap::new();
    for piece in &batch.pieces {
        for row in &piece.summary {
            let totals = aggregate
                .entry((piece.corpus.clone(), row.key.clone()))
                .or_insert((0, 0, ScoreTime::ZERO));
            totals.0 += 1;
            totals.1 += row.slice_count;
            totals.2 = totals.2 + row.total_duration;
        }
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    writer.write_record([
        "corpus",
        "category",
        "cardinality",
        "metric_weight",
        "harmony_label",
        "piece_count",
        "slice_count",
        "total_duration_qb",
    ])?;
    for ((corpus, key), (piece_count, slice_count, total_duration)) in aggregate {
        writer.write_record([
            corpus,
            key.category.to_string(),
            key.cardinality.to_string(),
            format!("{:?}", key.metric_weight),
            key.harmony_label,
            piece_count.to_string(),
            slice_count.to_string(),
            total_duration.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}
