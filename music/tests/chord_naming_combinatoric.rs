//! §7 — Combinatoric exhaustive snapshot dump.
//!
//! Iterates every subset of `{Pc0..Pc11}` containing `Pc0` (2^11 = 2048
//! sets), infers a chord quality for each under a specific `NamingConfig`,
//! and writes one line per set. The output is committed as a text snapshot;
//! a diff indicates a change to inference or rendering behavior.
//!
//! ## Format
//! ```text
//! [Pc0, Pc4, Pc7]                      => Maj                      [MajOrMinN]
//! [Pc0, Pc4, Pc7, Pc10]                => 7                        [MajOrMinN]
//! ...
//! ```
//!
//! Columns:
//! 1. Pc set (sorted, comma-separated, braces-free)
//! 2. Inferred rendered name (or `<no match>`)
//! 3. Heuristic that fired (or `<degenerate>` for size≤2 fast-path hits)
//!
//! ## Updating the snapshot
//! Run `cargo test --test chord_naming_combinatoric -- --ignored update`
//! to regenerate `music/tests/snapshots/chord_name_corpus_*.txt`.
//!
//! ## Variant configs
//! Four presets produce four snapshot files, so diffs between them expose
//! where the config flags genuinely alter output.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::{
    infer_chord_quality_kind, ChordHeuristicKind,
};
use music::note_collections::chord_name::{ChordNameDisplayConfig, NamingConfig};

fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
}

fn all_pcs() -> Vec<Pc> {
    use Pc::*;
    vec![Pc0, Pc1, Pc2, Pc3, Pc4, Pc5, Pc6, Pc7, Pc8, Pc9, Pc10, Pc11]
}

fn pcs_from_mask(mask: u16) -> Vec<Pc> {
    let pcs = all_pcs();
    (0..12)
        .filter(|i| (mask >> i) & 1 == 1)
        .map(|i| pcs[i as usize])
        .collect()
}

fn format_pcs(pcs: &[Pc]) -> String {
    let s: Vec<String> = pcs.iter().map(|p| format!("{:?}", p)).collect();
    format!("[{}]", s.join(", "))
}

/// Produce the snapshot content for a given config pair. Canonical Pc set
/// formatting is stable so the output is reproducible.
fn generate_corpus(naming: &NamingConfig, display: &ChordNameDisplayConfig) -> String {
    let mut lines = Vec::with_capacity(2048);
    // Iterate every subset of the 11-bit space above Pc0 (Pc0 always set).
    for upper in 0u16..(1 << 11) {
        // Shift by 1 because bit 0 is Pc0 (always in the set).
        let mask = (upper << 1) | 1;
        let pcs = pcs_from_mask(mask);
        let set_label = format_pcs(&pcs);
        let pc_hashset: HashSet<Pc> = pcs.iter().copied().collect();

        let (rendered, heuristic_label) = match infer_chord_quality_kind(&pc_hashset, naming) {
            Some((kind, Some(q))) => (q.to_string(display), kind.label().to_string()),
            Some((kind, None)) => ("<no name>".to_string(), kind.label().to_string()),
            None => ("<no match>".to_string(), "<none>".to_string()),
        };
        // Fixed-width columns so the snapshot is diffable by eye.
        lines.push(format!(
            "{:<40} => {:<32} [{}]",
            set_label, rendered, heuristic_label,
        ));
    }
    lines.join("\n") + "\n"
}

fn corpus_filename(preset: &str) -> PathBuf {
    snapshot_dir().join(format!("chord_name_corpus_{}.txt", preset))
}

fn audit_filename() -> PathBuf {
    snapshot_dir().join("chord_name_audit.tsv")
}

/// Build a single flat TSV combining every preset's output side-by-side,
/// one row per Pc0-containing subset. Intended for LLM-judged quality
/// review: a reviewer can scan a single file and judge each row in one pass
/// rather than diffing four separate corpora.
///
/// Columns (tab-separated):
///   1. pcs               — sorted set, e.g. `Pc0,Pc4,Pc7`
///   2. intervals         — semitone distances from Pc0, e.g. `0,4,7`
///   3. size              — cardinality
///   4. default           — rendered name under NamingConfig::default()
///   5. strict            — rendered name under NamingConfig::strict()
///   6. jazz              — rendered name under NamingConfig::jazz()
///   7. pop               — rendered name under NamingConfig::pop()
///   8. heuristic         — which heuristic fired (same for all presets;
///                          presets affect rendering, not dispatch order)
///   9. disagreement      — `same` if all four preset outputs are equal,
///                          otherwise `diff` — lets a reviewer focus on
///                          config-sensitive rows
fn generate_audit() -> String {
    let cfgs = configs();
    let mut out = String::new();
    out.push_str("pcs\tintervals\tsize\tdefault\tstrict\tjazz\tpop\theuristic\tdisagreement\n");
    for upper in 0u16..(1 << 11) {
        let mask = (upper << 1) | 1;
        let pcs = pcs_from_mask(mask);
        let pcs_label = pcs
            .iter()
            .map(|p| format!("{:?}", p))
            .collect::<Vec<_>>()
            .join(",");
        let intervals = pcs
            .iter()
            .map(|p| u8::from(p).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let size = pcs.len();
        let pc_hashset: HashSet<Pc> = pcs.iter().copied().collect();

        let mut renders = Vec::with_capacity(4);
        let mut heuristic = String::from("<none>");
        for (_, naming, display) in &cfgs {
            let (label, name) = match infer_chord_quality_kind(&pc_hashset, naming) {
                Some((kind, Some(q))) => (kind.label().to_string(), q.to_string(display)),
                Some((kind, None)) => (kind.label().to_string(), "<no name>".to_string()),
                None => ("<none>".to_string(), "<no match>".to_string()),
            };
            heuristic = label;
            renders.push(name);
        }
        let disagreement = if renders.windows(2).all(|w| w[0] == w[1]) {
            "same"
        } else {
            "diff"
        };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            pcs_label,
            intervals,
            size,
            renders[0],
            renders[1],
            renders[2],
            renders[3],
            heuristic,
            disagreement,
        ));
    }
    out
}

fn configs() -> Vec<(&'static str, NamingConfig, ChordNameDisplayConfig)> {
    vec![
        (
            "default",
            NamingConfig::default(),
            ChordNameDisplayConfig::default(),
        ),
        (
            "strict",
            NamingConfig::strict(),
            ChordNameDisplayConfig::default(),
        ),
        (
            "jazz",
            NamingConfig::jazz(),
            ChordNameDisplayConfig::default(),
        ),
        (
            "pop",
            NamingConfig::pop(),
            ChordNameDisplayConfig::default(),
        ),
    ]
}

/// Regenerate the snapshots. Invoked manually:
/// `cargo test --test chord_naming_combinatoric -- --ignored update`
#[test]
#[ignore]
fn update() {
    fs::create_dir_all(snapshot_dir()).expect("create snapshot dir");
    for (preset, naming, display) in configs() {
        let corpus = generate_corpus(&naming, &display);
        let path = corpus_filename(preset);
        fs::write(&path, corpus).expect("write snapshot");
        eprintln!("wrote {}", path.display());
    }
    // Flat audit file — all presets side-by-side, for LLM-judged review.
    let audit = generate_audit();
    let audit_path = audit_filename();
    fs::write(&audit_path, audit).expect("write audit");
    eprintln!("wrote {}", audit_path.display());
}

/// Compare generated snapshots against committed ones. Covers both the
/// per-preset corpus files and the flat side-by-side audit TSV. Mismatch =
/// fail with a short diff preview. Regenerate via `update` above to accept.
#[test]
fn snapshot_compare() {
    let diff_preview = |expected: &str, actual: &str| -> String {
        let first_diff = expected
            .lines()
            .zip(actual.lines())
            .enumerate()
            .find(|(_, (e, a))| e != a);
        match first_diff {
            Some((i, (e, a))) => format!(
                "first divergence at line {}:\n  expected: {}\n  actual:   {}",
                i + 1,
                e,
                a,
            ),
            None => "line counts differ".to_string(),
        }
    };

    for (preset, naming, display) in configs() {
        let expected_path = corpus_filename(preset);
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
            panic!(
                "missing snapshot {}: {}\nRun `cargo test --test chord_naming_combinatoric -- --ignored update` to generate.",
                expected_path.display(),
                e,
            )
        });
        let actual = generate_corpus(&naming, &display);
        if expected != actual {
            panic!(
                "snapshot `{}` drift. {}\n(Re-run with `-- --ignored update` to accept changes.)",
                preset,
                diff_preview(&expected, &actual),
            );
        }
    }

    // Flat audit TSV — same rigor as per-preset snapshots.
    let audit_path = audit_filename();
    let expected_audit = fs::read_to_string(&audit_path).unwrap_or_else(|e| {
        panic!(
            "missing audit {}: {}\nRun `cargo test --test chord_naming_combinatoric -- --ignored update` to generate.",
            audit_path.display(),
            e,
        )
    });
    let actual_audit = generate_audit();
    if expected_audit != actual_audit {
        panic!(
            "audit file drift. {}\n(Re-run with `-- --ignored update` to accept changes.)",
            diff_preview(&expected_audit, &actual_audit),
        );
    }
}

/// Sanity: heuristic coverage. A diff between runs might reveal dead
/// heuristics; this test just logs the counts so that PR reviewers can see
/// them.
#[test]
fn heuristic_coverage_default() {
    use std::collections::HashMap;
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let naming = NamingConfig::default();
    for upper in 0u16..(1 << 11) {
        let mask = (upper << 1) | 1;
        let pcs = pcs_from_mask(mask);
        let pc_hashset: HashSet<Pc> = pcs.iter().copied().collect();
        let label = match infer_chord_quality_kind(&pc_hashset, &naming) {
            Some((kind, _)) => kind.label(),
            None => "<none>",
        };
        *counts.entry(label).or_insert(0) += 1;
    }
    // Must have coverage for at least the degenerate fast-paths and the big
    // major/minor families. Fail if any of these end up with zero hits —
    // that'd indicate a broken fast-path or broken dispatch.
    for must_appear in ["DegenerateSingleton", "DegenerateInterval", "MajOrMinN"] {
        assert!(
            counts.get(must_appear).copied().unwrap_or(0) > 0,
            "heuristic {} never matched across 2048 sets; check dispatch",
            must_appear,
        );
    }
    // Print counts for visibility when running with `-- --nocapture`.
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (label, count) in sorted {
        eprintln!("{:4}  {}", count, label);
    }
}

// Sanity-suppress the unused-import lint for ChordHeuristicKind when the
// snapshot tests run disabled.
#[allow(dead_code)]
fn _keep_chord_heuristic_kind_public(k: ChordHeuristicKind) -> &'static str {
    k.label()
}
