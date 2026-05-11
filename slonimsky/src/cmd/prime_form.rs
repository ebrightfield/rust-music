use anyhow::Result;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::{PcContent, PcShape};

use super::input::parse_input_to_pcs;

pub struct PrimeFormArgs {
    pub input: Vec<String>,
    pub verbose: bool,
}

/// Compute the normal form of a pitch-class set.
///
/// Finds the rotation with the smallest outer interval (span), then
/// tiebreaks by lexicographic comparison of the normalized intervals.
/// Returns the result transposed to start at 0.
fn normal_form(pcs: &[u8]) -> Vec<u8> {
    let mut sorted: Vec<u8> = pcs.to_vec();
    sorted.sort();
    sorted.dedup();
    let n = sorted.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![0];
    }

    let mut best: Option<Vec<u8>> = None;
    let mut best_span = 13u8;

    for i in 0..n {
        let last_idx = (i + n - 1) % n;
        let span = (sorted[last_idx] + 12 - sorted[i]) % 12;

        let normalized: Vec<u8> = (0..n)
            .map(|j| (sorted[(i + j) % n] + 12 - sorted[i]) % 12)
            .collect();

        if span < best_span || (span == best_span && normalized < *best.as_ref().unwrap()) {
            best_span = span;
            best = Some(normalized);
        }
    }

    best.unwrap()
}

/// Compute the prime form of a PcSet using the Forte/Rahn algorithm.
///
/// 1. Compute the normal form of the input set.
/// 2. Compute the normal form of the inverted set.
/// 3. Return the lexicographically smaller of the two.
///
/// This produces the standard prime forms used in Forte's set-class
/// catalog (e.g. major triad → [0,3,7] = 3-11).
pub fn prime_form(pcs: &[Pc]) -> Vec<u8> {
    let set: Vec<u8> = pcs.iter().map(|pc| u8::from(*pc)).collect();
    if set.is_empty() {
        return vec![];
    }

    let nf = normal_form(&set);

    let inverted: Vec<u8> = set.iter().map(|&pc| (12 - pc) % 12).collect();
    let nf_inv = normal_form(&inverted);

    if nf <= nf_inv {
        nf
    } else {
        nf_inv
    }
}

/// Format a prime form as bracket notation: [0, 3, 7]
fn format_prime_form(pf: &[u8]) -> String {
    let inner: Vec<String> = pf.iter().map(|n| n.to_string()).collect();
    format!("[{}]", inner.join(", "))
}

/// Format a PcSet for display: {0, 4, 7}
fn format_pc_set(pcs: &[Pc]) -> String {
    let inner: Vec<String> = pcs.iter().map(|pc| u8::from(*pc).to_string()).collect();
    format!("{{{}}}", inner.join(", "))
}

pub fn run(args: PrimeFormArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = PcContent::new(pcs);
    let pf = prime_form(&pc_set);

    println!("PcSet:      {}", format_pc_set(&pc_set));
    println!("Prime form: {}", format_prime_form(&pf));

    if args.verbose {
        // Show the interval vector from the prime form
        let pf_pcs: Vec<Pc> = pf.iter().map(|&n| Pc::from(n)).collect();
        let pf_shape = PcShape::new(pf_pcs);
        let matrix = music::geometry::IntervalMatrix::new(&pf_shape);
        let iv = matrix.reduced_interval_vector();
        let iv_str: Vec<String> = iv.iter().map(|n: &usize| n.to_string()).collect();
        println!("IV:         <{}>", iv_str.join(", "));

        // Transpositional symmetry
        let pc_shape = pc_set.to_shape();
        let sym = pc_shape.transpositional_symmetry();
        if sym.is_empty() {
            println!("T-symmetry: none (only T0)");
        } else {
            let mut sym_labels: Vec<String> = Vec::new();
            for syms in sym.values() {
                for s in syms {
                    let label = format!("{:?}", s);
                    if !sym_labels.contains(&label) {
                        sym_labels.push(label);
                    }
                }
            }
            sym_labels.sort();
            println!("T-symmetry: {}", sym_labels.join(", "));
        }

        // Inversional symmetry
        use music::geometry::symmetry::intervallic::IntervallicSymmetry;
        if pc_shape.is_inversionally_symmetric() {
            println!("I-symmetry: yes (palindromic intervals)");
        } else {
            println!("I-symmetry: no");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_form_major_triad() {
        // C major {0,4,7} → prime form [0,3,7]
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        assert_eq!(prime_form(&pcs), vec![0, 3, 7]);
    }

    #[test]
    fn prime_form_minor_triad() {
        // C minor {0,3,7} → prime form [0,3,7] (same set class as major)
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc7];
        assert_eq!(prime_form(&pcs), vec![0, 3, 7]);
    }

    #[test]
    fn prime_form_d_minor_same_class() {
        // D minor {2,5,9} → prime form [0,3,7]
        let pcs = vec![Pc::Pc2, Pc::Pc5, Pc::Pc9];
        assert_eq!(prime_form(&pcs), vec![0, 3, 7]);
    }

    #[test]
    fn prime_form_diminished_seventh() {
        // Dim7 {0,3,6,9} → prime form [0,3,6,9]
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc6, Pc::Pc9];
        assert_eq!(prime_form(&pcs), vec![0, 3, 6, 9]);
    }

    #[test]
    fn prime_form_whole_tone() {
        // Whole-tone {0,2,4,6,8,10} → prime form [0,2,4,6,8,10]
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc6, Pc::Pc8, Pc::Pc10];
        assert_eq!(prime_form(&pcs), vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn prime_form_chromatic_trichord() {
        // {0,1,2} → [0,1,2]
        let pcs = vec![Pc::Pc0, Pc::Pc1, Pc::Pc2];
        assert_eq!(prime_form(&pcs), vec![0, 1, 2]);
    }

    #[test]
    fn prime_form_transposed_input() {
        // F# major {6,10,1} → prime form [0,3,7]
        let pcs = vec![Pc::Pc6, Pc::Pc10, Pc::Pc1];
        assert_eq!(prime_form(&pcs), vec![0, 3, 7]);
    }

    #[test]
    fn prime_form_empty() {
        assert_eq!(prime_form(&[]), Vec::<u8>::new());
    }

    #[test]
    fn prime_form_single_pc() {
        let pcs = vec![Pc::Pc5];
        assert_eq!(prime_form(&pcs), vec![0]);
    }

    #[test]
    fn format_prime_form_display() {
        assert_eq!(format_prime_form(&[0, 3, 7]), "[0, 3, 7]");
        assert_eq!(format_prime_form(&[0, 1, 2, 3]), "[0, 1, 2, 3]");
    }

    #[test]
    fn prime_form_minor_seventh() {
        // Min7 {0,3,7,10} → normal form via most compact rotation [0,3,5,8] = 4-26
        // The old pure-lex algorithm incorrectly returned [0,2,5,9].
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc7, Pc::Pc10];
        assert_eq!(prime_form(&pcs), vec![0, 3, 5, 8]);
    }

    #[test]
    fn prime_form_half_diminished() {
        // Half-dim7 {0,3,6,10} → [0,2,5,8] = 4-27 (same class as dom7)
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc6, Pc::Pc10];
        assert_eq!(prime_form(&pcs), vec![0, 2, 5, 8]);
    }

    #[test]
    fn prime_form_normal_form_most_compact() {
        // Verify normal_form picks the most compact rotation, not just lex smallest.
        // {0, 1, 6, 7}: rotations:
        //   start 0: [0,1,6,7] span=7
        //   start 1: [0,5,6,11] span=11
        //   start 6: [0,1,6,7] span=7 (same by symmetry)
        //   start 7: [0,5,6,11] span=11
        // Both span-7 rotations give [0,1,6,7].
        let result = normal_form(&[0, 1, 6, 7]);
        assert_eq!(result, vec![0, 1, 6, 7]);
    }
}
