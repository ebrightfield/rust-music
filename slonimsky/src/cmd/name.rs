use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::{ChordNameDisplayConfig, ChordQuality, MajNotation};
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};

pub struct NameArgs {
    pub pcs: Vec<String>,
    pub root: Option<String>,
    pub verbose: bool,
}

/// Transpose a list of Pcs so that `root` becomes Pc0.
fn normalize_to_root(pcs: &[Pc], root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(&root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

pub fn run(args: NameArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.pcs)?;

    let root_pc = match &args.root {
        Some(r) => {
            let parsed = parse_input_to_pcs(std::slice::from_ref(r))?;
            *parsed.first().context("invalid root")?
        }
        None => pcs[0],
    };

    let normalized = normalize_to_root(&pcs, root_pc);

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        let norm_ints: Vec<u8> = {
            let mut v: Vec<u8> = normalized.iter().map(u8::from).collect();
            v.sort();
            v
        };
        eprintln!("name: input pcs={pc_ints:?}, root={}, normalized={norm_ints:?}",
                  pc_label(root_pc));
    }

    let display_cfg = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        utf8_accidentals: true,
        ..Default::default()
    };

    match infer_chord_quality(&normalized) {
        Some((_heuristic, Some(quality))) => {
            let quality_str = quality.to_string(&display_cfg);
            let root_name = pc_label(root_pc);
            println!("{root_name}{quality_str}");

            if args.verbose {
                print_detail(&pcs, root_pc, &quality, &display_cfg);
            }
        }
        Some((_heuristic, None)) => {
            println!("{} (unclassified)", pc_label(root_pc));
            if args.verbose {
                eprintln!("heuristic matched but produced no quality");
            }
        }
        None => {
            anyhow::bail!(
                "could not identify chord quality for pitch classes: {:?}",
                pcs.iter().map(u8::from).collect::<Vec<_>>()
            );
        }
    }

    Ok(())
}

fn print_detail(pcs: &[Pc], root: Pc, quality: &ChordQuality, cfg: &ChordNameDisplayConfig) {
    // Pitch classes as integers
    let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
    eprintln!("  pitch classes: {pc_ints:?}");

    // Intervals from root
    let root_val = u8::from(&root);
    let intervals: Vec<u8> = pcs
        .iter()
        .map(|p| (u8::from(p) + 12 - root_val) % 12)
        .collect();
    eprintln!("  intervals from root: {intervals:?}");

    // Possible note names for each pc
    let names: Vec<String> = pcs
        .iter()
        .map(|p| {
            let notes = p.notes();
            if notes.len() <= 2 {
                notes.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join("/")
            } else {
                // Show first two (most common) spellings
                notes.iter().take(2).map(|n| format!("{n:?}")).collect::<Vec<_>>().join("/")
            }
        })
        .collect();
    eprintln!("  notes: {}", names.join(", "));

    let _ = (quality, cfg); // used only for the main output line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_c_major() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let norm = normalize_to_root(&pcs, Pc::Pc0);
        assert!(norm.contains(&Pc::Pc0));
        assert!(norm.contains(&Pc::Pc4));
        assert!(norm.contains(&Pc::Pc7));
    }

    #[test]
    fn normalize_a_minor() {
        // A(9) C(0) E(4) → normalized to root A: 0, 3, 7
        let pcs = vec![Pc::Pc9, Pc::Pc0, Pc::Pc4];
        let norm = normalize_to_root(&pcs, Pc::Pc9);
        assert!(norm.contains(&Pc::Pc0)); // A→0
        assert!(norm.contains(&Pc::Pc3)); // C→3
        assert!(norm.contains(&Pc::Pc7)); // E→7
    }

    #[test]
    fn name_c_major_triad() {
        let args = NameArgs {
            pcs: vec!["C".into(), "E".into(), "G".into()],
            root: None,
            verbose: false,
        };
        // Should not error — the quality should be identified as major
        run(args).unwrap();
    }

    #[test]
    fn name_d_minor_triad() {
        let args = NameArgs {
            pcs: vec!["D".into(), "F".into(), "A".into()],
            root: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn name_with_explicit_root() {
        // A C E with root A → A minor
        let args = NameArgs {
            pcs: vec!["A".into(), "C".into(), "E".into()],
            root: Some("A".into()),
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn name_dominant_seventh() {
        let args = NameArgs {
            pcs: vec!["G".into(), "B".into(), "D".into(), "F".into()],
            root: None,
            verbose: false,
        };
        run(args).unwrap();
    }
}
