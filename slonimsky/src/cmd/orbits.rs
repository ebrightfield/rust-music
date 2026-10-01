use anyhow::Result;
use music::geometry::symmetry::intervallic::IntervallicSymmetry;
use music::geometry::symmetry::transpositional::{
    find_transpositional_symmetries, TranspositionalSymmetry,
};
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcContent;
use std::collections::HashSet;

use super::input::parse_input_to_pcs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymmetryType {
    Transpositional,
    Inversional,
    Both,
}

impl SymmetryType {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("both") => Ok(Self::Both),
            Some("transpositional") | Some("t") => Ok(Self::Transpositional),
            Some("inversional") | Some("i") => Ok(Self::Inversional),
            Some(other) => anyhow::bail!(
                "unknown symmetry type '{}'; expected transpositional, inversional, or both",
                other
            ),
        }
    }
}

pub struct OrbitsArgs {
    pub input: Vec<String>,
    pub sym_type: SymmetryType,
    pub verbose: bool,
}

fn format_pc_set(pcs: &[Pc]) -> String {
    let inner: Vec<String> = pcs.iter().map(|pc| u8::from(*pc).to_string()).collect();
    format!("{{{}}}", inner.join(", "))
}

fn sym_label(s: &TranspositionalSymmetry) -> &'static str {
    match s {
        TranspositionalSymmetry::T2 => "T2",
        TranspositionalSymmetry::T3 => "T3",
        TranspositionalSymmetry::T4 => "T4",
        TranspositionalSymmetry::T6 => "T6",
    }
}

pub fn run(args: OrbitsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = PcContent::new(pcs.clone());

    println!("PcSet: {}", format_pc_set(&pc_set));
    println!();

    let show_t =
        args.sym_type == SymmetryType::Transpositional || args.sym_type == SymmetryType::Both;
    let show_i = args.sym_type == SymmetryType::Inversional || args.sym_type == SymmetryType::Both;

    if show_t {
        let sym_map = find_transpositional_symmetries(&pc_set.to_vec());

        // Collect unique symmetry types across all PCs
        let mut unique_syms: Vec<&'static str> = Vec::new();
        for syms in sym_map.values() {
            for s in syms {
                let label = sym_label(s);
                if !unique_syms.contains(&label) {
                    unique_syms.push(label);
                }
            }
        }
        unique_syms.sort();

        if unique_syms.is_empty() {
            println!("Transpositional symmetries: none (only T0)");
        } else {
            println!("Transpositional symmetries: {}", unique_syms.join(", "));

            if args.verbose {
                // Per-PC detail: sort by PC integer value
                let mut pc_entries: Vec<(u8, &HashSet<TranspositionalSymmetry>)> = sym_map
                    .iter()
                    .map(|(pc, syms)| (u8::from(*pc), syms))
                    .collect();
                pc_entries.sort_by_key(|(n, _)| *n);

                for (pc_val, syms) in &pc_entries {
                    let mut labels: Vec<&str> = syms.iter().map(|s| sym_label(s)).collect();
                    labels.sort();
                    println!("  Pc{}: {{{}}}", pc_val, labels.join(", "));
                }
            }
        }
    }

    if show_t && show_i {
        println!();
    }

    if show_i {
        let pc_shape = pc_set.to_shape();
        let inverted = pc_shape.invert_intervals();
        if let Some(inv) = inverted {
            println!("Inversionally symmetric: no");
            println!("Inverted form: {}", format_pc_set(&inv));
        } else {
            println!("Inversionally symmetric: yes (palindromic intervals)");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcs_from(nums: &[u8]) -> Vec<Pc> {
        nums.iter().map(|&n| Pc::from(n)).collect()
    }

    #[test]
    fn dim7_has_t3_and_t6() {
        let pcs = pcs_from(&[0, 3, 6, 9]);
        let pc_set = PcContent::new(pcs);
        let sym = find_transpositional_symmetries(&pc_set.to_vec());
        assert!(!sym.is_empty());
        // Should have T3 and T6 symmetry
        let mut all_labels: Vec<&str> = Vec::new();
        for syms in sym.values() {
            for s in syms {
                let l = sym_label(s);
                if !all_labels.contains(&l) {
                    all_labels.push(l);
                }
            }
        }
        all_labels.sort();
        assert_eq!(all_labels, vec!["T3", "T6"]);
    }

    #[test]
    fn augmented_triad_has_t4() {
        let pcs = pcs_from(&[0, 4, 8]);
        let pc_set = PcContent::new(pcs);
        let sym = find_transpositional_symmetries(&pc_set.to_vec());
        let mut all_labels: Vec<&str> = Vec::new();
        for syms in sym.values() {
            for s in syms {
                let l = sym_label(s);
                if !all_labels.contains(&l) {
                    all_labels.push(l);
                }
            }
        }
        assert!(all_labels.contains(&"T4"));
    }

    #[test]
    fn major_triad_no_t_symmetry() {
        let pcs = pcs_from(&[0, 4, 7]);
        let pc_set = PcContent::new(pcs);
        let sym = find_transpositional_symmetries(&pc_set.to_vec());
        assert!(sym.is_empty());
    }

    #[test]
    fn dim7_is_inversionally_symmetric() {
        let pcs = pcs_from(&[0, 3, 6, 9]);
        let pc_set = PcContent::new(pcs);
        assert!(pc_set.to_shape().is_inversionally_symmetric());
    }

    #[test]
    fn major_triad_not_inversionally_symmetric() {
        let pcs = pcs_from(&[0, 4, 7]);
        let pc_set = PcContent::new(pcs);
        let pc_shape = pc_set.to_shape();
        // Major triad inverts to minor triad — not symmetric
        assert!(!pc_shape.is_inversionally_symmetric());
        let inverted = pc_shape.invert_intervals().unwrap();
        // Inverted form should be a different pitch-class shape
        assert_ne!(pc_shape, inverted);
    }

    #[test]
    fn symmetry_type_parsing() {
        assert_eq!(
            SymmetryType::from_str_opt(None).unwrap(),
            SymmetryType::Both
        );
        assert_eq!(
            SymmetryType::from_str_opt(Some("both")).unwrap(),
            SymmetryType::Both
        );
        assert_eq!(
            SymmetryType::from_str_opt(Some("transpositional")).unwrap(),
            SymmetryType::Transpositional
        );
        assert_eq!(
            SymmetryType::from_str_opt(Some("t")).unwrap(),
            SymmetryType::Transpositional
        );
        assert_eq!(
            SymmetryType::from_str_opt(Some("inversional")).unwrap(),
            SymmetryType::Inversional
        );
        assert_eq!(
            SymmetryType::from_str_opt(Some("i")).unwrap(),
            SymmetryType::Inversional
        );
        assert!(SymmetryType::from_str_opt(Some("garbage")).is_err());
    }

    #[test]
    fn whole_tone_has_t2() {
        let pcs = pcs_from(&[0, 2, 4, 6, 8, 10]);
        let pc_set = PcContent::new(pcs);
        let sym = find_transpositional_symmetries(&pc_set.to_vec());
        let mut all_labels: Vec<&str> = Vec::new();
        for syms in sym.values() {
            for s in syms {
                let l = sym_label(s);
                if !all_labels.contains(&l) {
                    all_labels.push(l);
                }
            }
        }
        assert!(all_labels.contains(&"T2"));
    }
}
