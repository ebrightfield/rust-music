use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::{ChordNameDisplayConfig, MajNotation};
use music::note_collections::geometry::sets::get_subchords;
use music::note_collections::pc_set::PcShape;
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};

pub struct SubchordsArgs {
    pub input: Vec<String>,
    pub size: u8,
    pub name: bool,
    /// Report sets and names relative to prime form (rooted on pitch-class 0)
    /// instead of in the key of the queried input.
    pub relative: bool,
    pub verbose: bool,
}

/// Transpose a set of Pcs so that `root` becomes Pc0.
fn normalize_to_root(pcs: &[Pc], root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(&root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

/// Try to name a subset of raw Pcs.
///
/// `preferred_root` is the root of the enclosing query; when the subset
/// contains it, the subset is named from it so labels read in the queried key.
/// Otherwise the lowest pitch class is used as the root.
fn try_name_subset(pcs: &[Pc], preferred_root: Option<Pc>) -> Option<String> {
    if pcs.is_empty() {
        return None;
    }
    let root = match preferred_root {
        Some(r) if pcs.contains(&r) => r,
        _ => *pcs.iter().min_by_key(|pc| u8::from(*pc))?,
    };
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
    let pc_set = PcShape::new(pcs.clone());

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

    // `PcShape::new` sorts and zero-anchors, so the shape is rooted on pc 0
    // regardless of what was queried. Unless the caller explicitly asked for
    // prime-form output, shift everything back into the key of the queried
    // root (the first pitch class given) so the sets and their names describe
    // the chords the user actually asked about.
    let query_root = pcs[0];
    let shift = if args.relative {
        0
    } else {
        // The shape's zero corresponds to the lowest pc of the sorted input;
        // realign it to the pc the user named first.
        let mut sorted = pcs.clone();
        sorted.sort();
        sorted.dedup();
        (12 + u8::from(&query_root) - u8::from(&sorted[0])) % 12
    };

    let subchords: Vec<Vec<Pc>> = subchords
        .iter()
        .map(|sub| {
            sub.iter()
                .map(|pc| Pc::from((u8::from(pc) + shift) % 12))
                .collect()
        })
        .collect();

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        eprintln!(
            "subchords: input={pc_ints:?}, size={}, found {} subsets, shift=+{shift}",
            args.size,
            subchords.len()
        );
    }

    let mut header_pcs: Vec<u8> = pc_set
        .iter()
        .map(|pc| (u8::from(pc) + shift) % 12)
        .collect();
    header_pcs.sort();
    println!(
        "Subchords of {{{}}} (size {}){}:",
        header_pcs
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(","),
        args.size,
        if args.relative {
            " [prime-form relative]"
        } else {
            ""
        }
    );
    println!();

    for (i, sub) in subchords.iter().enumerate() {
        // Print each set rooted on its own lowest note, in ascending order.
        let mut sorted = sub.clone();
        sorted.sort();
        let pc_ints: Vec<u8> = sorted.iter().map(u8::from).collect();
        let set_str = pc_ints
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        if args.name {
            let name = try_name_subset(sub, Some(query_root)).unwrap_or_else(|| "?".to_string());
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
            relative: false,
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
            relative: false,
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
            relative: false,
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
            relative: false,
            verbose: false,
        };
        assert!(run(args).is_err());
    }

    /// A subset containing the queried root is named from that root, so labels
    /// read in the key the user asked about rather than in prime form.
    /// Regression: `subchords G,A,B,...` used to label the G-rooted triad "C".
    #[test]
    fn names_use_the_queried_root() {
        // G,A,B in a G-major query → G major (add 9), not C-anything.
        let g_a_b = vec![Pc::Pc7, Pc::Pc9, Pc::Pc11];
        let name = try_name_subset(&g_a_b, Some(Pc::Pc7)).expect("should name");
        assert!(name.starts_with('G'), "expected a G-rooted name, got: {name}");

        // Bb,C,D,F in a Bb-major query → Bb-rooted.
        let bb_set = vec![Pc::Pc10, Pc::Pc0, Pc::Pc2, Pc::Pc5];
        let name = try_name_subset(&bb_set, Some(Pc::Pc10)).expect("should name");
        assert!(
            name.starts_with("A#/Bb"),
            "expected a Bb-rooted name, got: {name}"
        );
    }

    /// When the subset does not contain the queried root, fall back to naming
    /// from its own lowest pitch class (by pc value, not input order).
    #[test]
    fn names_fall_back_to_lowest_pc() {
        // D,F,A with a G query: no G present, so name from the lowest pc (D).
        let d_min = vec![Pc::Pc5, Pc::Pc9, Pc::Pc2];
        let name = try_name_subset(&d_min, Some(Pc::Pc7)).expect("should name");
        assert!(name.starts_with('D'), "expected a D-rooted name, got: {name}");
    }

    /// `--relative` keeps the historical prime-form output; the default does not.
    #[test]
    fn relative_flag_reports_prime_form() {
        let input = vec!["G".into(), "A".into(), "B".into(), "C".into(),
                         "D".into(), "E".into(), "F#".into()];
        for relative in [true, false] {
            let args = SubchordsArgs {
                input: input.clone(),
                size: 3,
                name: true,
                relative,
                verbose: false,
            };
            run(args).unwrap();
        }
    }

    #[test]
    fn try_name_c_major() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let name = try_name_subset(&pcs, None);
        assert!(name.is_some());
        let n = name.unwrap();
        assert!(n.contains("Maj") || n.contains("maj"),
                "C major triad should be named as major, got: {n}");
    }

    #[test]
    fn try_name_d_minor() {
        let pcs = vec![Pc::Pc2, Pc::Pc5, Pc::Pc9];
        let name = try_name_subset(&pcs, None);
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
            relative: false,
            verbose: false,
        };
        // C7 has C(4,3) = 4 three-note subsets
        run(args).unwrap();
    }
}
