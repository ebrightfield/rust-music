//! Lossless corpus events and analysis pipelines for the rust-music workspace.
//!
//! The first adapter targets the DCML tabular release. It preserves exact
//! rational score time and source provenance, then derives literal sounding
//! slices without conflating them with analyst-supplied harmony labels.

pub mod dcml;
pub mod dcml_batch;
pub mod divergence;
pub mod model;
pub mod sonority;

pub use dcml::{DcmlAdapter, DcmlPaths};
pub use dcml_batch::{
    analyze_dcml_divergence_manifest, write_corpus_divergence_summary_tsv,
    write_piece_divergence_summaries_tsv, DcmlDivergenceBatch, PieceDivergenceSummary,
};
pub use divergence::{
    analyze_divergence, write_divergence_summary_tsv, write_divergence_tsv, DivergenceCategory,
    DivergenceRecord, DivergenceReport, DivergenceSummaryKey, DivergenceSummaryRow,
};
pub use model::{
    CorpusPiece, ExactScoreSpan, HarmonyAnnotation, Measure, NoteEvent, PerformanceSpan, ScoreTime,
    SourceRef, TimeSignature,
};
pub use sonority::{
    analyze_piece, write_slice_tsv, ClassifiedQuality, MetricWeight, SonorityReport, SonoritySlice,
};
