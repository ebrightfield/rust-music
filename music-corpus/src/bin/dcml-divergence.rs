use std::{env, fs::File, path::Path};

use anyhow::{bail, Context, Result};
use music_corpus::{
    analyze_divergence, analyze_piece, write_divergence_summary_tsv, write_divergence_tsv,
    DcmlAdapter, DcmlPaths,
};

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 8 {
        bail!(
            "usage: {} NOTES.tsv MEASURES.tsv EXPANDED.tsv CORPUS PIECE DETAILS.tsv SUMMARY.tsv",
            arguments
                .first()
                .map(String::as_str)
                .unwrap_or("dcml-divergence")
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
    let sonorities = analyze_piece(&piece)?;
    let divergence = analyze_divergence(&piece, &sonorities);

    let details =
        File::create(&arguments[6]).with_context(|| format!("creating {}", arguments[6]))?;
    write_divergence_tsv(&piece, &sonorities, &divergence, details)?;
    let summary =
        File::create(&arguments[7]).with_context(|| format!("creating {}", arguments[7]))?;
    write_divergence_summary_tsv(&divergence, summary)?;

    let mut counts = std::collections::BTreeMap::new();
    for record in &divergence.records {
        *counts.entry(record.category).or_insert(0_u64) += 1;
    }
    eprintln!(
        "{}/{}: {} divergence records; {:?}",
        piece.corpus,
        piece.piece,
        divergence.records.len(),
        counts
    );
    Ok(())
}
