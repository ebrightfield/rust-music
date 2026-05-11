use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::{ChordNameDisplayConfig, MajNotation};
use music::note_collections::geometry::sets::get_subchords;
use music::note_collections::PcSet;
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};

pub struct SubchordsArgs {
    pub input: Vec<String>,
    pub size: u8,
    pub name: bool,
    pub verbose: bool,
}

/// Transpose a set of Pcs so that `root` becomes Pc0.
fn normalize_to_root(pcs: &[Pc], root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(&root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

/// Try to name a subset given as raw Pcs, treating the first Pc as root.
fn try_name_subset(pcs: &[Pc]) -> Option<String> {
    if pcs.is_empty() {
        return None;
    }
    let root = pcs[0];
    let normalized = normalize_to_root(pcs, root);

    let display_cfg = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        utf8_accidentals: true,
        ..Default::default()
    };

    match infer_chord_quality(&normalized) {
        Some((_heuristic, Some(quality))) => {
            let quality_str = quality.to_string(&display_cfg);
            Some(format!("{}{}", pc_label(root), quality_str))
        }
        _ => None,
    }
}

pub fn run(args: SubchordsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = PcSet::new(pcs.clone());

    anyhow::ensure!(
        args.size >= 3,
        "subchord size must be at least 3 (got {})",
        args.size
    );
    anyhow::ensure!(
        (args.size as usize) < pcs.len(),
        "subchord size {} must be less than the input size {}",
        args.size,
        pcs.len()
    );

    let subchords = get_subchords(&pc_set, args.size)
        .context("failed to compute subchords")?;

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        eprintln!(
            "subchords: input={pc_ints:?}, size={}, found {} subsets",
            args.size,
            subchords.len()
        );
    }

    let header_pcs: Vec<u8> = pc_set.iter().map(u8::from).collect();
    println!(
        "Subchords of {{{}}} (size {}):",
        header_pcs
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(","),
        args.size
    );
    println!();

    for (i, sub) in subchords.iter().enumerate() {
        let pc_ints: Vec<u8> = sub.iter().map(u8::from).collect();
        let set_str = pc_ints
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        if args.name {
            let name = try_name_subset(sub).unwrap_or_else(|| "?".to_string());
            println!("  {:>3}. {{{set_str}}}  {name}", i + 1);
        } else {
            println!("  {:>3}. {{{set_str}}}", i + 1);
        }
    }

    println!();
    println!("Total: {} subchords", subchords.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subchords_of_major_scale_size_3() {
        let args = SubchordsArgs {
            input: vec!["C".into(), "D".into(), "E".into(), "F".into(),
                        "G".into(), "A".into(), "B".into()],
            size: 3,
            name: false,
            verbose: false,
        };
        // C major scale has C(7,3) = 35 three-note subsets
        run(args).unwrap();
    }

    #[test]
    fn subchords_with_naming() {
        let args = SubchordsArgs {
            input: vec!["C".into(), "D".into(), "E".into(), "F".into(),
                        "G".into(), "A".into(), "B".into()],
            size: 3,
            name: true,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn subchords_size_too_large() {
        let args = SubchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            size: 3,
            name: false,
            verbose: false,
        };
        // Size 3 from a 3-element set → must be < input size
        assert!(run(args).is_err());
    }

    #[test]
    fn subchords_size_too_small() {
        let args = SubchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into(), "B".into()],
            size: 2,
            name: false,
            verbose: false,
        };
        assert!(run(args).is_err());
    }

    #[test]
    fn try_name_c_major() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let name = try_name_subset(&pcs);
        assert!(name.is_some());
        let n = name.unwrap();
        assert!(n.contains("Maj") || n.contains("maj"),
                "C major triad should be named as major, got: {n}");
    }

    #[test]
    fn try_name_d_minor() {
        let pcs = vec![Pc::Pc2, Pc::Pc5, Pc::Pc9];
        let name = try_name_subset(&pcs);
        assert!(name.is_some());
        let n = name.unwrap();
        assert!(n.starts_with("D"), "root should be D, got: {n}");
    }

    #[test]
    fn subchords_of_dom7_size_3() {
        let args = SubchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into(), "Bb".into()],
            size: 3,
            name: true,
            verbose: false,
        };
        // C7 has C(4,3) = 4 three-note subsets
        run(args).unwrap();
    }
}
