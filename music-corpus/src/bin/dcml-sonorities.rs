use std::{env, fs::File, path::Path};

use anyhow::{bail, Context, Result};
use music_corpus::{analyze_piece, write_slice_tsv, DcmlAdapter, DcmlPaths};

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 7 {
        bail!(
            "usage: {} NOTES.tsv MEASURES.tsv EXPANDED.tsv CORPUS PIECE OUTPUT.tsv",
            arguments
                .first()
                .map(String::as_str)
                .unwrap_or("dcml-sonorities")
        );
    }

    let piece = DcmlAdapter::load_piece(
        DcmlPaths {
            notes: Path::new(&arguments[1]),
            measures: Path::new(&arguments[2]),
            expanded: Path::new(&arguments[3]),
        },
        &arguments[4],
        &arguments[5],
    )?;
    let report = analyze_piece(&piece)?;
    let output =
        File::create(&arguments[6]).with_context(|| format!("creating {}", arguments[6]))?;
    write_slice_tsv(&piece, &report, output)?;

    eprintln!(
        "{}/{}: {} slices, {} 3NC, {} 4NC, {} source note rows accounted ({} zero-duration)",
        piece.corpus,
        piece.piece,
        report.slices.len(),
        report.classified_three,
        report.classified_four,
        report.note_rows_accounted,
        report.zero_duration_note_rows,
    );
    eprintln!("cardinality counts: {:?}", report.cardinality_counts);
    Ok(())
}
