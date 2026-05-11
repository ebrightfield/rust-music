use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use std::collections::BTreeSet;

use super::input::{parse_input_to_pcs, pc_label};

pub struct CommonTonesArgs {
    /// Each element is a comma-separated group of PCs representing one set
    pub sets: Vec<String>,
    pub verbose: bool,
}

/// Parse a single comma-or-space-separated token group into a BTreeSet<Pc>.
fn parse_set(token: &str) -> Result<BTreeSet<Pc>> {
    let pcs = parse_input_to_pcs(&[token.to_string()])
        .with_context(|| format!("in set '{token}'"))?;
    Ok(pcs.into_iter().collect())
}

fn format_pc_set(pcs: &BTreeSet<Pc>) -> String {
    if pcs.is_empty() {
        return "(empty)".to_string();
    }
    pcs.iter()
        .map(|pc| pc_label(*pc))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_pc_integers(pcs: &BTreeSet<Pc>) -> String {
    if pcs.is_empty() {
        return "{}".to_string();
    }
    let inner: String = pcs
        .iter()
        .map(|pc| format!("{}", u8::from(*pc)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{inner}}}")
}

pub fn run(args: CommonTonesArgs) -> Result<()> {
    anyhow::ensure!(
        args.sets.len() >= 2,
        "common-tones requires at least 2 sets (each as a comma-separated group, e.g. C,E,G D,F,A)"
    );

    let parsed: Vec<BTreeSet<Pc>> = args
        .sets
        .iter()
        .map(|s| parse_set(s))
        .collect::<Result<Vec<_>>>()?;

    // Print input sets
    for (i, set) in parsed.iter().enumerate() {
        println!("Set {}: {} = {}", i + 1, format_pc_integers(set), format_pc_set(set));
    }
    println!();

    // N-ary intersection
    let mut common = parsed[0].clone();
    for set in &parsed[1..] {
        common = common.intersection(set).copied().collect();
    }

    println!(
        "Common tones: {} = {}",
        format_pc_integers(&common),
        format_pc_set(&common)
    );
    println!("Count: {}", common.len());

    // Verbose: pairwise intersections
    if args.verbose && parsed.len() > 2 {
        eprintln!();
        eprintln!("Pairwise intersections:");
        for i in 0..parsed.len() {
            for j in (i + 1)..parsed.len() {
                let pair: BTreeSet<Pc> =
                    parsed[i].intersection(&parsed[j]).copied().collect();
                eprintln!(
                    "  Set {} ∩ Set {}: {} = {} ({})",
                    i + 1,
                    j + 1,
                    format_pc_integers(&pair),
                    format_pc_set(&pair),
                    pair.len()
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::pitch_class::Pc::*;

    #[test]
    fn common_c_major_d_minor() {
        // C major {C,E,G} ∩ D minor {D,F,A} = empty
        let a: BTreeSet<Pc> = [Pc0, Pc4, Pc7].into();
        let b: BTreeSet<Pc> = [Pc2, Pc5, Pc9].into();
        let common: BTreeSet<Pc> = a.intersection(&b).copied().collect();
        assert!(common.is_empty());
    }

    #[test]
    fn common_c_major_a_minor() {
        // C major {C,E,G} ∩ A minor {A,C,E} = {C,E}
        let a: BTreeSet<Pc> = [Pc0, Pc4, Pc7].into();
        let b: BTreeSet<Pc> = [Pc9, Pc0, Pc4].into();
        let common: BTreeSet<Pc> = a.intersection(&b).copied().collect();
        assert_eq!(common.len(), 2);
        assert!(common.contains(&Pc0)); // C
        assert!(common.contains(&Pc4)); // E
    }

    #[test]
    fn common_three_sets() {
        // C major {C,E,G}, Am {A,C,E}, Cmaj7 {C,E,G,B}
        // Three-way intersection = {C, E}
        let sets: Vec<BTreeSet<Pc>> = vec![
            [Pc0, Pc4, Pc7].into(),
            [Pc9, Pc0, Pc4].into(),
            [Pc0, Pc4, Pc7, Pc11].into(),
        ];
        let mut common = sets[0].clone();
        for s in &sets[1..] {
            common = common.intersection(s).copied().collect();
        }
        assert_eq!(common.len(), 2);
        assert!(common.contains(&Pc0));
        assert!(common.contains(&Pc4));
    }

    #[test]
    fn parse_set_valid() {
        let s = parse_set("C,E,G").unwrap();
        assert_eq!(s.len(), 3);
        assert!(s.contains(&Pc0));
        assert!(s.contains(&Pc4));
        assert!(s.contains(&Pc7));
    }

    #[test]
    fn parse_set_integers() {
        let s = parse_set("0,4,7").unwrap();
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn format_empty_set() {
        let s: BTreeSet<Pc> = BTreeSet::new();
        assert_eq!(format_pc_set(&s), "(empty)");
        assert_eq!(format_pc_integers(&s), "{}");
    }

    #[test]
    fn run_rejects_single_set() {
        let result = run(CommonTonesArgs {
            sets: vec!["C,E,G".into()],
            verbose: false,
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("at least 2 sets"));
    }
}
