use std::{env, fs::File, path::Path};

use anyhow::{bail, Context, Result};
use music_corpus::{
    analyze_dcml_divergence_manifest, write_corpus_divergence_summary_tsv,
    write_piece_divergence_summaries_tsv,
};

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 4 {
        bail!(
            "usage: {} MANIFEST.tsv PER_PIECE.tsv CORPUS.tsv",
            arguments
                .first()
                .map(String::as_str)
                .unwrap_or("dcml-divergence-batch")
        );
    }

    let batch = analyze_dcml_divergence_manifest(Path::new(&arguments[1]))?;
    let per_piece =
        File::create(&arguments[2]).with_context(|| format!("creating {}", arguments[2]))?;
    write_piece_divergence_summaries_tsv(&batch, per_piece)
        .with_context(|| format!("writing {}", arguments[2]))?;
    let corpus =
        File::create(&arguments[3]).with_context(|| format!("creating {}", arguments[3]))?;
    write_corpus_divergence_summary_tsv(&batch, corpus)
        .with_context(|| format!("writing {}", arguments[3]))?;

    eprintln!(
        "analyzed {} DCML pieces from {}",
        batch.pieces.len(),
        arguments[1]
    );
    Ok(())
}
