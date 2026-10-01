use anyhow::{bail, Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::{ChordNameDisplayConfig, MajNotation};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;

use super::input::{parse_input_to_pcs, pc_label};

pub struct AnalyzeArgs {
    pub chords: Vec<String>,
    pub key: Option<String>,
    pub scale: Option<String>,
    pub format: OutputFormat,
    pub output: Option<String>,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("text") => Ok(Self::Text),
            Some("json") => Ok(Self::Json),
            Some(other) => bail!("unknown format: '{other}' (options: text, json)"),
        }
    }

    /// Infer format from file extension when `--format` is not given.
    pub fn infer_from_output(path: &str, explicit: Self) -> Self {
        if explicit != Self::Text {
            return explicit;
        }
        if path.ends_with(".json") {
            Self::Json
        } else {
            Self::Text
        }
    }
}

/// A scale quality with its interval pattern (semitones from root).
struct ScaleType {
    name: &'static str,
    intervals: &'static [u8],
}

const SCALE_TYPES: &[ScaleType] = &[
    ScaleType {
        name: "major",
        intervals: &[0, 2, 4, 5, 7, 9, 11],
    },
    ScaleType {
        name: "natural-minor",
        intervals: &[0, 2, 3, 5, 7, 8, 10],
    },
    ScaleType {
        name: "harmonic-minor",
        intervals: &[0, 2, 3, 5, 7, 8, 11],
    },
    ScaleType {
        name: "melodic-minor",
        intervals: &[0, 2, 3, 5, 7, 9, 11],
    },
    ScaleType {
        name: "harmonic-major",
        intervals: &[0, 2, 4, 5, 7, 8, 11],
    },
];

/// Generate a PcSet for a scale given root and interval pattern.
fn scale_pcs(root: Pc, intervals: &[u8]) -> BTreeSet<Pc> {
    let root_val = u8::from(root);
    intervals
        .iter()
        .map(|i| Pc::from((root_val + i) % 12))
        .collect()
}

/// Estimate the key of a chord progression by brute-force scoring.
///
/// Tiebreaking when multiple key candidates share the same diatonic score:
///   1. Prefer the first chord's root (strongest tonal signal).
///   2. Prefer major scale over other scale types.
///   3. Fall back to lower root PC value.
fn estimate_key(all_pcs: &BTreeSet<Pc>, first_chord_root: Option<Pc>) -> (Pc, &'static str, usize) {
    let mut best_root = Pc::Pc0;
    let mut best_scale = "major";
    let mut best_score = 0;
    // Tiebreak priority: higher = preferred. (first_chord_bonus, major_bonus, low_root_bonus)
    let mut best_priority: (u8, u8, u8) = (0, 0, 0);

    for root_val in 0u8..12 {
        let root = Pc::from(root_val);
        for st in SCALE_TYPES {
            let s = scale_pcs(root, st.intervals);
            let score = all_pcs.iter().filter(|pc| s.contains(pc)).count();
            let first_chord_bonus = if first_chord_root == Some(root) {
                1u8
            } else {
                0
            };
            let major_bonus = if st.name == "major" { 1u8 } else { 0 };
            let low_root_bonus = 11 - root_val; // prefer lower root values
            let priority = (first_chord_bonus, major_bonus, low_root_bonus);

            if score > best_score || (score == best_score && priority > best_priority) {
                best_score = score;
                best_root = root;
                best_scale = st.name;
                best_priority = priority;
            }
        }
    }

    (best_root, best_scale, best_score)
}

/// Resolve scale name string to an interval pattern.
fn resolve_scale(name: &str) -> Result<&'static [u8]> {
    match name.to_lowercase().as_str() {
        "major" => Ok(&[0, 2, 4, 5, 7, 9, 11]),
        "natural-minor" | "minor" => Ok(&[0, 2, 3, 5, 7, 8, 10]),
        "harmonic-minor" => Ok(&[0, 2, 3, 5, 7, 8, 11]),
        "melodic-minor" => Ok(&[0, 2, 3, 5, 7, 9, 11]),
        "harmonic-major" => Ok(&[0, 2, 4, 5, 7, 8, 11]),
        other => bail!("unknown scale: '{other}' (options: major, natural-minor, harmonic-minor, melodic-minor, harmonic-major)"),
    }
}

/// Transpose PcSet so root becomes Pc0.
fn normalize_to_root(pcs: &BTreeSet<Pc>, root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(*pc) + offset) % 12))
        .collect()
}

/// Infer a chord quality name for a PcSet at a given root.
fn name_chord(pcs: &BTreeSet<Pc>, root: Pc) -> String {
    let normalized = normalize_to_root(pcs, root);
    let display_cfg = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        utf8_accidentals: true,
        ..Default::default()
    };

    match infer_chord_quality(&normalized) {
        Some((_heuristic, Some(quality))) => {
            let quality_str = quality.to_string(&display_cfg);
            format!("{}{}", pc_label(root), quality_str)
        }
        _ => format!("{}(?)", pc_label(root)),
    }
}

/// Find which scale degree (1-indexed) a chord root sits on in the parent scale.
fn find_degree(chord_root: Pc, scale_root: Pc, scale_intervals: &[u8]) -> Option<usize> {
    let root_val = u8::from(scale_root);
    for (i, interval) in scale_intervals.iter().enumerate() {
        let degree_pc = (root_val + interval) % 12;
        if degree_pc == u8::from(chord_root) {
            return Some(i + 1);
        }
    }
    None
}

/// Convert a degree + quality to a Roman numeral string.
fn roman_numeral(degree: usize, quality_name: &str) -> String {
    let base = match degree {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        _ => "?",
    };

    let lower = quality_name.to_lowercase();
    let is_minor_family = lower.contains("min") || lower.contains("dim");

    let roman = if is_minor_family {
        base.to_lowercase()
    } else {
        base.to_string()
    };

    // Add quality suffixes
    if lower.contains("dim") {
        format!("{roman}°")
    } else if lower.contains("aug") {
        format!("{roman}+")
    } else if lower.contains("7") || lower.contains("maj7") {
        format!("{roman}{}", if lower.contains("7") { "7" } else { "" })
    } else {
        roman
    }
}

/// Compute common tones between two BTreeSets.
fn common_tones(a: &BTreeSet<Pc>, b: &BTreeSet<Pc>) -> BTreeSet<Pc> {
    a.intersection(b).copied().collect()
}

/// Compute L1 voice-leading cost as minimum sum of absolute semitone motion
/// between two PcSets of equal size (simple approximation: sorted pair matching).
fn l1_cost(a: &BTreeSet<Pc>, b: &BTreeSet<Pc>) -> usize {
    let mut a_vals: Vec<u8> = a.iter().map(|pc| u8::from(*pc)).collect();
    let mut b_vals: Vec<u8> = b.iter().map(|pc| u8::from(*pc)).collect();
    a_vals.sort();
    b_vals.sort();

    // For each pair of sorted PCs, compute min(|a-b|, 12-|a-b|)
    a_vals
        .iter()
        .zip(b_vals.iter())
        .map(|(a, b)| {
            let diff = a.abs_diff(*b);
            diff.min(12 - diff) as usize
        })
        .sum()
}

struct ChordAnalysis {
    input_label: String,
    pcs: BTreeSet<Pc>,
    root: Pc,
    quality_name: String,
    degree: Option<usize>,
    roman: String,
    diatonic: bool,
}

pub fn run(args: AnalyzeArgs) -> Result<()> {
    if args.chords.len() < 2 {
        bail!("analyze requires at least 2 chords (each as a comma-separated group, e.g. C,E,G F,A,C G,B,D)");
    }

    // Parse each chord group, keeping the first PC as the intended root
    let mut chord_sets: Vec<(String, Pc, BTreeSet<Pc>)> = Vec::new();
    for group in &args.chords {
        let pcs = parse_input_to_pcs(std::slice::from_ref(group))?;
        let root = pcs[0]; // first PC in user's input order = chord root
        let set: BTreeSet<Pc> = pcs.into_iter().collect();
        chord_sets.push((group.clone(), root, set));
    }

    // Collect union of all PCs
    let all_pcs: BTreeSet<Pc> = chord_sets
        .iter()
        .flat_map(|(_, _, s)| s.iter().copied())
        .collect();

    // Key estimation or user-provided key
    let (key_root, scale_name, confidence) = if let Some(key_str) = &args.key {
        let key_pcs = parse_input_to_pcs(std::slice::from_ref(key_str))?;
        let key_root = key_pcs[0];
        let scale_name = args.scale.as_deref().unwrap_or("major");
        let scale_intervals = resolve_scale(scale_name)?;
        let s = scale_pcs(key_root, scale_intervals);
        let confidence = all_pcs.iter().filter(|pc| s.contains(pc)).count();
        (key_root, scale_name.to_string(), confidence)
    } else {
        let first_root = chord_sets.first().map(|(_, r, _)| *r);
        let (root, name, score) = estimate_key(&all_pcs, first_root);
        (root, name.to_string(), score)
    };

    let scale_intervals = resolve_scale(&scale_name)?;
    let parent_scale = scale_pcs(key_root, scale_intervals);
    let key_provided = args.key.is_some();

    // Analyze each chord
    let mut analyses: Vec<ChordAnalysis> = Vec::new();
    for (label, root, pcs) in &chord_sets {
        let root = *root;
        let quality_name = name_chord(pcs, root);
        let degree = find_degree(root, key_root, scale_intervals);
        let diatonic = pcs.is_subset(&parent_scale);
        let roman = if let Some(deg) = degree {
            roman_numeral(deg, &quality_name)
        } else {
            // Non-diatonic: prefix with accidental
            let root_val = u8::from(root);
            let key_val = u8::from(key_root);
            let interval = (root_val + 12 - key_val) % 12;
            // Find closest diatonic degree
            let closest = scale_intervals
                .iter()
                .enumerate()
                .min_by_key(|(_, &si)| {
                    let d = si.abs_diff(interval);
                    d.min(12 - d)
                })
                .map(|(i, _)| i + 1)
                .unwrap_or(1);
            let base_roman = roman_numeral(closest, &quality_name);
            format!("♭/♯{base_roman}")
        };

        analyses.push(ChordAnalysis {
            input_label: label.clone(),
            pcs: pcs.clone(),
            root,
            quality_name,
            degree,
            roman,
            diatonic,
        });
    }

    // Infer format from output path when --format wasn't explicitly set
    let format = match &args.output {
        Some(path) => OutputFormat::infer_from_output(path, args.format),
        None => args.format,
    };

    let output_str = match format {
        OutputFormat::Text => format_text(
            &analyses,
            key_root,
            &scale_name,
            confidence,
            key_provided,
            args.verbose,
        ),
        OutputFormat::Json => format_json(&analyses, key_root, &scale_name, confidence),
    };

    match &args.output {
        Some(path) => {
            fs::write(path, &output_str)
                .with_context(|| format!("writing output to '{}'", path))?;
        }
        None => {
            print!("{}", output_str);
        }
    }

    Ok(())
}

fn format_text(
    analyses: &[ChordAnalysis],
    key_root: Pc,
    scale_name: &str,
    confidence: usize,
    key_provided: bool,
    verbose: bool,
) -> String {
    let mut out = String::new();
    let total_unique = {
        let all: BTreeSet<Pc> = analyses
            .iter()
            .flat_map(|a| a.pcs.iter().copied())
            .collect();
        all.len()
    };

    if key_provided {
        writeln!(out, "Key: {} {}", pc_label(key_root), scale_name).unwrap();
    } else {
        writeln!(
            out,
            "Key: {} {} (estimated, confidence: {}/{})",
            pc_label(key_root),
            scale_name,
            confidence,
            total_unique
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    // Chord table
    writeln!(
        out,
        "  {:<12} {:<14} {:<12} {:<6} {:<8}",
        "Chord", "PcSet", "Quality", "Deg", "Roman"
    )
    .unwrap();
    writeln!(
        out,
        "  {:<12} {:<14} {:<12} {:<6} {:<8}",
        "─".repeat(12),
        "─".repeat(14),
        "─".repeat(12),
        "─".repeat(6),
        "─".repeat(8)
    )
    .unwrap();

    for a in analyses {
        let pcs_str = format!(
            "{{{}}}",
            a.pcs
                .iter()
                .map(|pc| format!("{}", u8::from(*pc)))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let deg_str = a
            .degree
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let diatonic_mark = if a.diatonic { "" } else { "*" };
        writeln!(
            out,
            "  {:<12} {:<14} {:<12} {:<6} {:<8}{}",
            a.input_label, pcs_str, a.quality_name, deg_str, a.roman, diatonic_mark
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    // Common tones between adjacent chords
    if analyses.len() >= 2 {
        let mut ct_parts = Vec::new();
        for i in 0..analyses.len() - 1 {
            let ct = common_tones(&analyses[i].pcs, &analyses[i + 1].pcs);
            let ct_str = if ct.is_empty() {
                "∅".to_string()
            } else {
                format!(
                    "{{{}}}",
                    ct.iter()
                        .map(|pc| format!("{}", u8::from(*pc)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            ct_parts.push(format!(
                "{}→{}: {}",
                analyses[i].input_label,
                analyses[i + 1].input_label,
                ct_str
            ));
        }
        writeln!(out, "Common tones: {}", ct_parts.join("  ")).unwrap();
    }

    // Voice-leading cost (L1) between adjacent chords
    if analyses.len() >= 2 {
        let mut costs = Vec::new();
        let mut total_cost = 0;
        for i in 0..analyses.len() - 1 {
            if analyses[i].pcs.len() == analyses[i + 1].pcs.len() {
                let cost = l1_cost(&analyses[i].pcs, &analyses[i + 1].pcs);
                costs.push(format!("{}", cost));
                total_cost += cost;
            } else {
                costs.push("n/a".to_string());
            }
        }
        writeln!(
            out,
            "Voice-leading cost (L1): {}  total: {}",
            costs.join(" → "),
            total_cost
        )
        .unwrap();
    }

    if verbose {
        eprintln!(
            "analyze: {} chords, key={} {}, confidence={}/{}",
            analyses.len(),
            pc_label(key_root),
            scale_name,
            confidence,
            {
                let all: BTreeSet<Pc> = analyses
                    .iter()
                    .flat_map(|a| a.pcs.iter().copied())
                    .collect();
                all.len()
            }
        );
    }

    out
}

fn format_json(
    analyses: &[ChordAnalysis],
    key_root: Pc,
    scale_name: &str,
    confidence: usize,
) -> String {
    let mut out = String::new();
    // Manual JSON construction (avoids serde_json dep for default build)
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"key\": {{").unwrap();
    writeln!(out, "    \"root\": \"{}\",", pc_label(key_root)).unwrap();
    writeln!(out, "    \"scale\": \"{}\",", scale_name).unwrap();
    writeln!(out, "    \"confidence\": {}", confidence).unwrap();
    writeln!(out, "  }},").unwrap();
    writeln!(out, "  \"chords\": [").unwrap();

    for (i, a) in analyses.iter().enumerate() {
        let pcs: Vec<u8> = a.pcs.iter().map(|pc| u8::from(*pc)).collect();
        let trailing = if i < analyses.len() - 1 { "," } else { "" };
        writeln!(out, "    {{").unwrap();
        writeln!(out, "      \"input\": \"{}\",", a.input_label).unwrap();
        writeln!(out, "      \"pcs\": {:?},", pcs).unwrap();
        writeln!(out, "      \"root\": \"{}\",", pc_label(a.root)).unwrap();
        writeln!(out, "      \"quality\": \"{}\",", a.quality_name).unwrap();
        writeln!(
            out,
            "      \"degree\": {},",
            a.degree
                .map(|d| d.to_string())
                .unwrap_or_else(|| "null".to_string())
        )
        .unwrap();
        writeln!(out, "      \"roman\": \"{}\",", a.roman).unwrap();
        writeln!(out, "      \"diatonic\": {}", a.diatonic).unwrap();
        writeln!(out, "    }}{trailing}").unwrap();
    }
    writeln!(out, "  ],").unwrap();

    // Transitions
    writeln!(out, "  \"transitions\": [").unwrap();
    for i in 0..analyses.len().saturating_sub(1) {
        let ct = common_tones(&analyses[i].pcs, &analyses[i + 1].pcs);
        let ct_pcs: Vec<u8> = ct.iter().map(|pc| u8::from(*pc)).collect();
        let l1 = if analyses[i].pcs.len() == analyses[i + 1].pcs.len() {
            l1_cost(&analyses[i].pcs, &analyses[i + 1].pcs) as i64
        } else {
            -1
        };
        let trailing = if i < analyses.len() - 2 { "," } else { "" };
        writeln!(out, "    {{").unwrap();
        writeln!(out, "      \"from\": {},", i).unwrap();
        writeln!(out, "      \"to\": {},", i + 1).unwrap();
        writeln!(out, "      \"common_tones\": {:?},", ct_pcs).unwrap();
        writeln!(out, "      \"l1_cost\": {}", l1).unwrap();
        writeln!(out, "    }}{trailing}").unwrap();
    }
    writeln!(out, "  ]").unwrap();
    writeln!(out, "}}").unwrap();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_key_c_major() {
        // I-vi-ii-V in C major: C,E,G + A,C,E + D,F,A + G,B,D
        let all: BTreeSet<Pc> = [0, 2, 4, 5, 7, 9, 11]
            .iter()
            .map(|&i| Pc::from(i))
            .collect();
        let (root, name, score) = estimate_key(&all, Some(Pc::Pc0));
        assert_eq!(root, Pc::Pc0);
        assert_eq!(name, "major");
        assert_eq!(score, 7);
    }

    #[test]
    fn estimate_key_prefers_first_chord_root() {
        // Rock vamp C/Bb/F/C: PCs = {0,2,4,5,7,9,10}
        // F major and D natural-minor both score 7/7, but first chord root is C.
        // C major scores 6/7 (missing Bb). F major is best at 7/7 but not C-rooted.
        // With first-chord-root tiebreaking, when no candidate on root C beats the top score,
        // the algorithm should prefer F major (7/7) over D natural-minor (7/7).
        let all: BTreeSet<Pc> = [0, 2, 4, 5, 7, 9, 10]
            .iter()
            .map(|&i| Pc::from(i))
            .collect();
        let (root, name, score) = estimate_key(&all, Some(Pc::Pc0));
        // Should NOT pick D natural-minor (the old incorrect result)
        assert_ne!((root, name), (Pc::Pc2, "natural-minor"));
        assert_eq!(score, 7);
    }

    #[test]
    fn find_degree_in_c_major() {
        let intervals = &[0, 2, 4, 5, 7, 9, 11];
        assert_eq!(find_degree(Pc::Pc0, Pc::Pc0, intervals), Some(1)); // C = I
        assert_eq!(find_degree(Pc::Pc2, Pc::Pc0, intervals), Some(2)); // D = ii
        assert_eq!(find_degree(Pc::Pc4, Pc::Pc0, intervals), Some(3)); // E = iii
        assert_eq!(find_degree(Pc::Pc5, Pc::Pc0, intervals), Some(4)); // F = IV
        assert_eq!(find_degree(Pc::Pc7, Pc::Pc0, intervals), Some(5)); // G = V
        assert_eq!(find_degree(Pc::Pc9, Pc::Pc0, intervals), Some(6)); // A = vi
        assert_eq!(find_degree(Pc::Pc11, Pc::Pc0, intervals), Some(7)); // B = vii
    }

    #[test]
    fn find_degree_nondiatonic() {
        let intervals = &[0, 2, 4, 5, 7, 9, 11];
        // Bb is not in C major
        assert_eq!(find_degree(Pc::Pc10, Pc::Pc0, intervals), None);
    }

    #[test]
    fn roman_numeral_major_triad() {
        assert_eq!(roman_numeral(1, "CMaj"), "I");
        assert_eq!(roman_numeral(4, "FMaj"), "IV");
        assert_eq!(roman_numeral(5, "GMaj"), "V");
    }

    #[test]
    fn roman_numeral_minor_triad() {
        assert_eq!(roman_numeral(2, "Dmin"), "ii");
        assert_eq!(roman_numeral(6, "Amin"), "vi");
    }

    #[test]
    fn roman_numeral_diminished() {
        assert_eq!(roman_numeral(7, "Bdim"), "vii°");
    }

    #[test]
    fn l1_cost_unison() {
        let a: BTreeSet<Pc> = [0, 4, 7].iter().map(|&i| Pc::from(i)).collect();
        assert_eq!(l1_cost(&a, &a), 0);
    }

    #[test]
    fn l1_cost_step() {
        let a: BTreeSet<Pc> = [0, 4, 7].iter().map(|&i| Pc::from(i)).collect();
        let b: BTreeSet<Pc> = [0, 5, 9].iter().map(|&i| Pc::from(i)).collect();
        let cost = l1_cost(&a, &b);
        assert!(cost > 0);
        assert!(cost <= 6); // max 2 semitones per voice × 3 voices
    }

    #[test]
    fn common_tones_cmaj_amin() {
        let a: BTreeSet<Pc> = [0, 4, 7].iter().map(|&i| Pc::from(i)).collect();
        let b: BTreeSet<Pc> = [0, 4, 9].iter().map(|&i| Pc::from(i)).collect();
        let ct = common_tones(&a, &b);
        assert_eq!(ct.len(), 2); // C and E
        assert!(ct.contains(&Pc::Pc0));
        assert!(ct.contains(&Pc::Pc4));
    }

    #[test]
    fn run_basic_progression() {
        let args = AnalyzeArgs {
            chords: vec![
                "0,4,7".into(),
                "9,0,4".into(),
                "2,5,9".into(),
                "7,11,2".into(),
            ],
            key: None,
            scale: None,
            format: OutputFormat::Text,
            output: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn run_with_explicit_key() {
        let args = AnalyzeArgs {
            chords: vec!["C,E,G".into(), "F,A,C".into()],
            key: Some("C".into()),
            scale: Some("major".into()),
            format: OutputFormat::Text,
            output: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn run_json_output() {
        let args = AnalyzeArgs {
            chords: vec!["C,E,G".into(), "F,A,C".into()],
            key: Some("C".into()),
            scale: Some("major".into()),
            format: OutputFormat::Json,
            output: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn rejects_single_chord() {
        let args = AnalyzeArgs {
            chords: vec!["C,E,G".into()],
            key: None,
            scale: None,
            format: OutputFormat::Text,
            output: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
