use anyhow::Result;

use super::input::parse_input_to_pcs;
use super::prime_form::{prime_form, PrimeFormArgs};

pub struct ForteArgs {
    pub input: Vec<String>,
    pub verbose: bool,
}

/// Static Forte number table mapping prime forms to Forte labels.
/// Covers cardinalities 1–9 completely (215 entries).
/// Cardinalities 10–12 are trivial (near-chromatic aggregates) and omitted.
const FORTE_TABLE: &[(&[u8], &str)] = &[
    // Cardinality 1
    (&[0], "1-1"),
    // Cardinality 2
    (&[0, 1], "2-1"),
    (&[0, 2], "2-2"),
    (&[0, 3], "2-3"),
    (&[0, 4], "2-4"),
    (&[0, 5], "2-5"),
    (&[0, 6], "2-6"),
    // Cardinality 3
    (&[0, 1, 2], "3-1"),
    (&[0, 1, 3], "3-2"),
    (&[0, 1, 4], "3-3"),
    (&[0, 1, 5], "3-4"),
    (&[0, 1, 6], "3-5"),
    (&[0, 2, 4], "3-6"),
    (&[0, 2, 5], "3-7"),
    (&[0, 2, 6], "3-8"),
    (&[0, 2, 7], "3-9"),
    (&[0, 3, 6], "3-10"),
    (&[0, 3, 7], "3-11"),
    (&[0, 4, 8], "3-12"),
    // Cardinality 4
    (&[0, 1, 2, 3], "4-1"),
    (&[0, 1, 2, 4], "4-2"),
    (&[0, 1, 3, 4], "4-3"),
    (&[0, 1, 2, 5], "4-4"),
    (&[0, 1, 2, 6], "4-5"),
    (&[0, 1, 2, 7], "4-6"),
    (&[0, 1, 4, 5], "4-7"),
    (&[0, 1, 5, 6], "4-8"),
    (&[0, 1, 6, 7], "4-9"),
    (&[0, 2, 3, 5], "4-10"),
    (&[0, 1, 3, 5], "4-11"),
    (&[0, 2, 3, 6], "4-12"),
    (&[0, 1, 3, 6], "4-13"),
    (&[0, 2, 3, 7], "4-14"),
    (&[0, 1, 4, 6], "4-Z15"),
    (&[0, 1, 5, 7], "4-16"),
    (&[0, 3, 4, 7], "4-17"),
    (&[0, 1, 4, 7], "4-18"),
    (&[0, 1, 4, 8], "4-19"),
    (&[0, 1, 5, 8], "4-20"),
    (&[0, 2, 4, 6], "4-21"),
    (&[0, 2, 4, 7], "4-22"),
    (&[0, 2, 5, 7], "4-23"),
    (&[0, 2, 4, 8], "4-24"),
    (&[0, 2, 6, 8], "4-25"),
    (&[0, 3, 5, 8], "4-26"),
    (&[0, 2, 5, 8], "4-27"),
    (&[0, 3, 6, 9], "4-28"),
    (&[0, 1, 3, 7], "4-Z29"),
    // Cardinality 5 (38 set classes, Rahn prime forms)
    (&[0, 1, 2, 3, 4], "5-1"),
    (&[0, 1, 2, 3, 5], "5-2"),
    (&[0, 1, 2, 4, 5], "5-3"),
    (&[0, 1, 2, 3, 6], "5-4"),
    (&[0, 1, 2, 3, 7], "5-5"),
    (&[0, 1, 2, 5, 6], "5-6"),
    (&[0, 1, 2, 6, 7], "5-7"),
    (&[0, 2, 3, 4, 6], "5-8"),
    (&[0, 1, 2, 4, 6], "5-9"),
    (&[0, 1, 3, 4, 6], "5-10"),
    (&[0, 2, 3, 4, 7], "5-11"),
    (&[0, 1, 3, 5, 6], "5-Z12"),
    (&[0, 1, 2, 4, 8], "5-13"),
    (&[0, 1, 2, 5, 7], "5-14"),
    (&[0, 1, 2, 6, 8], "5-15"),
    (&[0, 1, 3, 4, 7], "5-16"),
    (&[0, 1, 3, 4, 8], "5-Z17"),
    (&[0, 1, 4, 5, 7], "5-Z18"),
    (&[0, 1, 3, 6, 7], "5-19"),
    (&[0, 1, 5, 6, 8], "5-20"),
    (&[0, 1, 4, 5, 8], "5-21"),
    (&[0, 1, 4, 7, 8], "5-22"),
    (&[0, 2, 3, 5, 7], "5-23"),
    (&[0, 1, 3, 5, 7], "5-24"),
    (&[0, 2, 3, 5, 8], "5-25"),
    (&[0, 2, 4, 5, 8], "5-26"),
    (&[0, 1, 3, 5, 8], "5-27"),
    (&[0, 2, 3, 6, 8], "5-28"),
    (&[0, 1, 3, 6, 8], "5-29"),
    (&[0, 1, 4, 6, 8], "5-30"),
    (&[0, 1, 3, 6, 9], "5-31"),
    (&[0, 1, 4, 6, 9], "5-32"),
    (&[0, 2, 4, 6, 8], "5-33"),
    (&[0, 2, 4, 6, 9], "5-34"),
    (&[0, 2, 4, 7, 9], "5-35"),
    (&[0, 1, 2, 4, 7], "5-Z36"),
    (&[0, 3, 4, 5, 8], "5-Z37"),
    (&[0, 1, 2, 5, 8], "5-Z38"),
    // Cardinality 6 (50 set classes, Rahn prime forms)
    (&[0, 1, 2, 3, 4, 5], "6-1"),
    (&[0, 1, 2, 3, 4, 6], "6-2"),
    (&[0, 1, 2, 3, 5, 6], "6-Z3"),
    (&[0, 1, 2, 4, 5, 6], "6-Z4"),
    (&[0, 1, 2, 3, 6, 7], "6-5"),
    (&[0, 1, 2, 5, 6, 7], "6-Z6"),
    (&[0, 1, 2, 6, 7, 8], "6-7"),
    (&[0, 2, 3, 4, 5, 7], "6-8"),
    (&[0, 1, 2, 3, 5, 7], "6-9"),
    (&[0, 1, 3, 4, 5, 7], "6-Z10"),
    (&[0, 1, 2, 4, 5, 7], "6-Z11"),
    (&[0, 1, 2, 4, 6, 7], "6-Z12"),
    (&[0, 1, 3, 4, 6, 7], "6-Z13"),
    (&[0, 1, 3, 4, 5, 8], "6-14"),
    (&[0, 1, 2, 4, 5, 8], "6-15"),
    (&[0, 1, 4, 5, 6, 8], "6-16"),
    (&[0, 1, 2, 4, 7, 8], "6-Z17"),
    (&[0, 1, 2, 5, 7, 8], "6-18"),
    (&[0, 1, 3, 4, 7, 8], "6-Z19"),
    (&[0, 1, 4, 5, 8, 9], "6-20"),
    (&[0, 2, 3, 4, 6, 8], "6-21"),
    (&[0, 1, 2, 4, 6, 8], "6-22"),
    (&[0, 2, 3, 5, 6, 8], "6-Z23"),
    (&[0, 1, 3, 4, 6, 8], "6-Z24"),
    (&[0, 1, 3, 5, 6, 8], "6-Z25"),
    (&[0, 1, 3, 5, 7, 8], "6-Z26"),
    (&[0, 1, 3, 4, 6, 9], "6-27"),
    (&[0, 1, 3, 5, 6, 9], "6-Z28"),
    (&[0, 2, 3, 6, 7, 9], "6-Z29"),
    (&[0, 1, 3, 6, 7, 9], "6-30"),
    (&[0, 1, 4, 5, 7, 9], "6-31"),
    (&[0, 2, 4, 5, 7, 9], "6-32"),
    (&[0, 2, 3, 5, 7, 9], "6-33"),
    (&[0, 1, 3, 5, 7, 9], "6-34"),
    (&[0, 2, 4, 6, 8, 10], "6-35"),
    (&[0, 1, 2, 3, 4, 7], "6-Z36"),
    (&[0, 1, 2, 3, 4, 8], "6-Z37"),
    (&[0, 1, 2, 3, 7, 8], "6-Z38"),
    (&[0, 2, 3, 4, 5, 8], "6-Z39"),
    (&[0, 1, 2, 3, 5, 8], "6-Z40"),
    (&[0, 1, 2, 3, 6, 8], "6-Z41"),
    (&[0, 1, 2, 3, 6, 9], "6-Z42"),
    (&[0, 1, 2, 5, 6, 8], "6-Z43"),
    (&[0, 1, 2, 5, 6, 9], "6-Z44"),
    (&[0, 2, 3, 4, 6, 9], "6-Z45"),
    (&[0, 1, 2, 4, 6, 9], "6-Z46"),
    (&[0, 1, 2, 4, 7, 9], "6-Z47"),
    (&[0, 1, 2, 5, 7, 9], "6-Z48"),
    (&[0, 1, 3, 4, 7, 9], "6-Z49"),
    (&[0, 1, 4, 6, 7, 9], "6-Z50"),
    // Cardinality 7 (38 set classes, Rahn prime forms)
    (&[0, 1, 2, 3, 4, 5, 6], "7-1"),
    (&[0, 1, 2, 3, 4, 5, 7], "7-2"),
    (&[0, 1, 2, 3, 4, 5, 8], "7-3"),
    (&[0, 1, 2, 3, 4, 6, 7], "7-4"),
    (&[0, 1, 2, 3, 5, 6, 7], "7-5"),
    (&[0, 1, 2, 3, 4, 7, 8], "7-6"),
    (&[0, 1, 2, 3, 6, 7, 8], "7-7"),
    (&[0, 2, 3, 4, 5, 6, 8], "7-8"),
    (&[0, 1, 2, 3, 4, 6, 8], "7-9"),
    (&[0, 1, 2, 3, 4, 6, 9], "7-10"),
    (&[0, 1, 3, 4, 5, 6, 8], "7-11"),
    (&[0, 1, 2, 3, 4, 7, 9], "7-Z12"),
    (&[0, 1, 2, 4, 5, 6, 8], "7-13"),
    (&[0, 1, 2, 3, 5, 7, 8], "7-14"),
    (&[0, 1, 2, 4, 6, 7, 8], "7-15"),
    (&[0, 1, 2, 3, 5, 6, 9], "7-16"),
    (&[0, 1, 2, 4, 5, 6, 9], "7-Z17"),
    (&[0, 1, 4, 5, 6, 7, 9], "7-Z18"),
    (&[0, 1, 2, 3, 6, 7, 9], "7-19"),
    (&[0, 1, 2, 5, 6, 7, 9], "7-20"),
    (&[0, 1, 2, 4, 5, 8, 9], "7-21"),
    (&[0, 1, 2, 5, 6, 8, 9], "7-22"),
    (&[0, 2, 3, 4, 5, 7, 9], "7-23"),
    (&[0, 1, 2, 3, 5, 7, 9], "7-24"),
    (&[0, 2, 3, 4, 6, 7, 9], "7-25"),
    (&[0, 1, 3, 4, 5, 7, 9], "7-26"),
    (&[0, 1, 2, 4, 5, 7, 9], "7-27"),
    (&[0, 1, 3, 5, 6, 7, 9], "7-28"),
    (&[0, 1, 2, 4, 6, 7, 9], "7-29"),
    (&[0, 1, 2, 4, 6, 8, 9], "7-30"),
    (&[0, 1, 3, 4, 6, 7, 9], "7-31"),
    (&[0, 1, 3, 4, 6, 8, 9], "7-32"),
    (&[0, 1, 2, 4, 6, 8, 10], "7-33"),
    (&[0, 1, 3, 4, 6, 8, 10], "7-34"),
    (&[0, 1, 3, 5, 6, 8, 10], "7-35"),
    (&[0, 1, 2, 3, 5, 6, 8], "7-Z36"),
    (&[0, 1, 3, 4, 5, 7, 8], "7-Z37"),
    (&[0, 1, 2, 4, 5, 7, 8], "7-Z38"),
    // Cardinality 8 (29 set classes, Rahn prime forms)
    (&[0, 1, 2, 3, 4, 5, 6, 7], "8-1"),
    (&[0, 1, 2, 3, 4, 5, 6, 8], "8-2"),
    (&[0, 1, 2, 3, 4, 5, 6, 9], "8-3"),
    (&[0, 1, 2, 3, 4, 5, 7, 8], "8-4"),
    (&[0, 1, 2, 3, 4, 6, 7, 8], "8-5"),
    (&[0, 1, 2, 3, 5, 6, 7, 8], "8-6"),
    (&[0, 1, 2, 3, 4, 5, 8, 9], "8-7"),
    (&[0, 1, 2, 3, 4, 7, 8, 9], "8-8"),
    (&[0, 1, 2, 3, 6, 7, 8, 9], "8-9"),
    (&[0, 2, 3, 4, 5, 6, 7, 9], "8-10"),
    (&[0, 1, 2, 3, 4, 5, 7, 9], "8-11"),
    (&[0, 1, 3, 4, 5, 6, 7, 9], "8-12"),
    (&[0, 1, 2, 3, 4, 6, 7, 9], "8-13"),
    (&[0, 1, 2, 4, 5, 6, 7, 9], "8-14"),
    (&[0, 1, 2, 3, 4, 6, 8, 9], "8-Z15"),
    (&[0, 1, 2, 3, 5, 7, 8, 9], "8-16"),
    (&[0, 1, 3, 4, 5, 6, 8, 9], "8-17"),
    (&[0, 1, 2, 3, 5, 6, 8, 9], "8-18"),
    (&[0, 1, 2, 4, 5, 6, 8, 9], "8-19"),
    (&[0, 1, 2, 4, 5, 7, 8, 9], "8-20"),
    (&[0, 1, 2, 3, 4, 6, 8, 10], "8-21"),
    (&[0, 1, 2, 3, 5, 6, 8, 10], "8-22"),
    (&[0, 1, 2, 3, 5, 7, 8, 10], "8-23"),
    (&[0, 1, 2, 4, 5, 6, 8, 10], "8-24"),
    (&[0, 1, 2, 4, 6, 7, 8, 10], "8-25"),
    (&[0, 1, 3, 4, 5, 7, 8, 10], "8-26"),
    (&[0, 1, 2, 4, 5, 7, 8, 10], "8-27"),
    (&[0, 1, 3, 4, 6, 7, 9, 10], "8-28"),
    (&[0, 1, 2, 3, 5, 6, 7, 9], "8-Z29"),
    // Cardinality 9 (12 set classes, Rahn prime forms)
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8], "9-1"),
    (&[0, 1, 2, 3, 4, 5, 6, 7, 9], "9-2"),
    (&[0, 1, 2, 3, 4, 5, 6, 8, 9], "9-3"),
    (&[0, 1, 2, 3, 4, 5, 7, 8, 9], "9-4"),
    (&[0, 1, 2, 3, 4, 6, 7, 8, 9], "9-5"),
    (&[0, 1, 2, 3, 4, 5, 6, 8, 10], "9-6"),
    (&[0, 1, 2, 3, 4, 5, 7, 8, 10], "9-7"),
    (&[0, 1, 2, 3, 4, 6, 7, 8, 10], "9-8"),
    (&[0, 1, 2, 3, 5, 6, 7, 8, 10], "9-9"),
    (&[0, 1, 2, 3, 4, 6, 7, 9, 10], "9-10"),
    (&[0, 1, 2, 3, 5, 6, 7, 9, 10], "9-11"),
    (&[0, 1, 2, 4, 5, 6, 8, 9, 10], "9-12"),
    // Cardinality 10 (6 set classes, complements of cardinality 2)
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "10-1"),
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8, 10], "10-2"),
    (&[0, 1, 2, 3, 4, 5, 6, 7, 9, 10], "10-3"),
    (&[0, 1, 2, 3, 4, 5, 6, 8, 9, 10], "10-4"),
    (&[0, 1, 2, 3, 4, 5, 7, 8, 9, 10], "10-5"),
    (&[0, 1, 2, 3, 4, 6, 7, 8, 9, 10], "10-6"),
    // Cardinality 11 (1 set class, complement of cardinality 1)
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], "11-1"),
    // Cardinality 12 (1 set class, the chromatic aggregate)
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], "12-1"),
];

/// Look up the Forte number for a given prime form.
pub fn lookup_forte(pf: &[u8]) -> Option<&'static str> {
    FORTE_TABLE
        .iter()
        .find(|(form, _)| *form == pf)
        .map(|(_, label)| *label)
}

/// Format a prime form as bracket notation: [0, 3, 7]
fn format_prime_form(pf: &[u8]) -> String {
    let inner: Vec<String> = pf.iter().map(|n| n.to_string()).collect();
    format!("[{}]", inner.join(", "))
}

/// Format a PcSet for display: {0, 4, 7}
fn format_pc_set(pcs: &[music::note::pitch_class::Pc]) -> String {
    let inner: Vec<String> = pcs
        .iter()
        .map(|pc| u8::from(*pc).to_string())
        .collect();
    format!("{{{}}}", inner.join(", "))
}

pub fn run(args: ForteArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = music::note_collections::pc_set::PcContent::new(pcs);
    let pf = prime_form(&pc_set);

    println!("PcSet:      {}", format_pc_set(&pc_set));
    println!("Prime form: {}", format_prime_form(&pf));

    match lookup_forte(&pf) {
        Some(forte) => println!("Forte:      {}", forte),
        None => println!("Forte:      (not in table — cardinalities 1–9 covered)"),
    }

    if args.verbose {
        // Delegate verbose output to prime-form (IV, symmetry info)
        let pf_args = PrimeFormArgs {
            input: args.input,
            verbose: true,
        };
        // Re-run the prime-form verbose section inline rather than calling run()
        // to avoid duplicating the header lines
        let pf_pcs: Vec<music::note::pitch_class::Pc> =
            pf.iter().map(|&n| music::note::pitch_class::Pc::from(n)).collect();
        let pf_shape = music::note_collections::pc_set::PcShape::new(pf_pcs);
        let matrix = music::geometry::IntervalMatrix::new(&pf_shape);
        let iv = matrix.reduced_interval_vector();
        let iv_str: Vec<String> = iv.iter().map(|n: &usize| n.to_string()).collect();
        println!("IV:         <{}>", iv_str.join(", "));

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

        use music::geometry::symmetry::intervallic::IntervallicSymmetry;
        if pc_shape.is_inversionally_symmetric() {
            println!("I-symmetry: yes (palindromic intervals)");
        } else {
            println!("I-symmetry: no");
        }

        // Z-relation: check if there's a Z-partner in the table
        if let Some(forte_label) = lookup_forte(&pf) {
            if forte_label.contains('Z') {
                let z_partner = find_z_partner(forte_label);
                if let Some(partner) = z_partner {
                    println!("Z-relation: {} (same interval vector)", partner);
                }
            }
        }

        // Suppress unused variable warning
        let _ = pf_args;
    }

    Ok(())
}

/// Find the Z-related partner of a Forte number.
/// Z-related pairs share the same interval vector but different prime forms.
const Z_PAIRS: &[(&str, &str)] = &[
    ("4-Z15", "4-Z29"),
    ("4-Z29", "4-Z15"),
    ("5-Z12", "5-Z36"),
    ("5-Z17", "5-Z37"),
    ("5-Z18", "5-Z38"),
    ("5-Z36", "5-Z12"),
    ("5-Z37", "5-Z17"),
    ("5-Z38", "5-Z18"),
    ("6-Z3", "6-Z36"),
    ("6-Z4", "6-Z37"),
    ("6-Z6", "6-Z38"),
    ("6-Z10", "6-Z39"),
    ("6-Z11", "6-Z40"),
    ("6-Z12", "6-Z41"),
    ("6-Z13", "6-Z42"),
    ("6-Z17", "6-Z43"),
    ("6-Z19", "6-Z44"),
    ("6-Z23", "6-Z45"),
    ("6-Z24", "6-Z46"),
    ("6-Z25", "6-Z47"),
    ("6-Z26", "6-Z48"),
    ("6-Z28", "6-Z49"),
    ("6-Z29", "6-Z50"),
    ("6-Z36", "6-Z3"),
    ("6-Z37", "6-Z4"),
    ("6-Z38", "6-Z6"),
    ("6-Z39", "6-Z10"),
    ("6-Z40", "6-Z11"),
    ("6-Z41", "6-Z12"),
    ("6-Z42", "6-Z13"),
    ("6-Z43", "6-Z17"),
    ("6-Z44", "6-Z19"),
    ("6-Z45", "6-Z23"),
    ("6-Z46", "6-Z24"),
    ("6-Z47", "6-Z25"),
    ("6-Z48", "6-Z26"),
    ("6-Z49", "6-Z28"),
    ("6-Z50", "6-Z29"),
    ("7-Z12", "7-Z36"),
    ("7-Z17", "7-Z37"),
    ("7-Z18", "7-Z38"),
    ("7-Z36", "7-Z12"),
    ("7-Z37", "7-Z17"),
    ("7-Z38", "7-Z18"),
    ("8-Z15", "8-Z29"),
    ("8-Z29", "8-Z15"),
];

fn find_z_partner(forte: &str) -> Option<&'static str> {
    Z_PAIRS
        .iter()
        .find(|(f, _)| *f == forte)
        .map(|(_, partner)| *partner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::pitch_class::Pc;

    #[test]
    fn forte_major_triad() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 3, 7]);
        assert_eq!(lookup_forte(&pf), Some("3-11"));
    }

    #[test]
    fn forte_minor_triad() {
        // Minor triad is the same set class as major
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc7];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 3, 7]);
        assert_eq!(lookup_forte(&pf), Some("3-11"));
    }

    #[test]
    fn forte_diminished_seventh() {
        let pcs = vec![Pc::Pc0, Pc::Pc3, Pc::Pc6, Pc::Pc9];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 3, 6, 9]);
        assert_eq!(lookup_forte(&pf), Some("4-28"));
    }

    #[test]
    fn forte_augmented_triad() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc8];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 4, 8]);
        assert_eq!(lookup_forte(&pf), Some("3-12"));
    }

    #[test]
    fn forte_chromatic_trichord() {
        let pcs = vec![Pc::Pc0, Pc::Pc1, Pc::Pc2];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2]);
        assert_eq!(lookup_forte(&pf), Some("3-1"));
    }

    #[test]
    fn forte_tritone() {
        let pcs = vec![Pc::Pc0, Pc::Pc6];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 6]);
        assert_eq!(lookup_forte(&pf), Some("2-6"));
    }

    #[test]
    fn forte_single_pc() {
        let pcs = vec![Pc::Pc5];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0]);
        assert_eq!(lookup_forte(&pf), Some("1-1"));
    }

    #[test]
    fn forte_z_related_pair() {
        // 4-Z15: [0,1,4,6]
        let pcs_15 = vec![Pc::Pc0, Pc::Pc1, Pc::Pc4, Pc::Pc6];
        let pf_15 = prime_form(&pcs_15);
        assert_eq!(lookup_forte(&pf_15), Some("4-Z15"));

        // 4-Z29: [0,1,3,7]
        let pcs_29 = vec![Pc::Pc0, Pc::Pc1, Pc::Pc3, Pc::Pc7];
        let pf_29 = prime_form(&pcs_29);
        assert_eq!(lookup_forte(&pf_29), Some("4-Z29"));

        // Verify Z-partner lookup
        assert_eq!(find_z_partner("4-Z15"), Some("4-Z29"));
        assert_eq!(find_z_partner("4-Z29"), Some("4-Z15"));
    }

    #[test]
    fn forte_decachord_10_1() {
        // Decachord {0..9} → prime form [0,1,2,3,4,5,6,7,8,9] → 10-1
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3, Pc::Pc4,
            Pc::Pc5, Pc::Pc6, Pc::Pc7, Pc::Pc8, Pc::Pc9,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(lookup_forte(&pf), Some("10-1"));
    }

    #[test]
    fn forte_chromatic_aggregate_12_1() {
        // Full chromatic aggregate → 12-1
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3, Pc::Pc4, Pc::Pc5,
            Pc::Pc6, Pc::Pc7, Pc::Pc8, Pc::Pc9, Pc::Pc10, Pc::Pc11,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(lookup_forte(&pf), Some("12-1"));
    }

    #[test]
    fn forte_undecachord_11_1() {
        // 11 PCs (missing Pc11) → prime form [0,1,2,3,4,5,6,7,8,9,10] → 11-1
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3, Pc::Pc4,
            Pc::Pc5, Pc::Pc6, Pc::Pc7, Pc::Pc8, Pc::Pc9, Pc::Pc10,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(lookup_forte(&pf), Some("11-1"));
    }

    #[test]
    fn forte_table_covers_card_10() {
        let card_10: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 10)
            .collect();
        assert_eq!(card_10.len(), 6);
    }

    #[test]
    fn forte_table_covers_all_card_3() {
        // There are exactly 12 cardinality-3 set classes
        let card_3: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 3)
            .collect();
        assert_eq!(card_3.len(), 12);
    }

    #[test]
    fn forte_table_covers_all_card_4() {
        // There are exactly 29 cardinality-4 set classes
        let card_4: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 4)
            .collect();
        assert_eq!(card_4.len(), 29);
    }

    #[test]
    fn forte_table_covers_all_card_5() {
        // There are exactly 38 cardinality-5 set classes
        let card_5: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 5)
            .collect();
        assert_eq!(card_5.len(), 38);
    }

    #[test]
    fn forte_pentatonic_is_5_35() {
        // Major pentatonic {0,2,4,7,9} → prime form [0,2,4,7,9] → 5-35
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc7, Pc::Pc9];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 2, 4, 7, 9]);
        assert_eq!(lookup_forte(&pf), Some("5-35"));
    }

    #[test]
    fn forte_whole_tone_pentachord_is_5_33() {
        // Whole-tone subset {0,2,4,6,8} → 5-33
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc6, Pc::Pc8];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 2, 4, 6, 8]);
        assert_eq!(lookup_forte(&pf), Some("5-33"));
    }

    #[test]
    fn forte_card5_z_pair() {
        // 5-Z12 and 5-Z36 are Z-related
        let pf_12 = &[0u8, 1, 3, 5, 6];
        let pf_36 = &[0u8, 1, 2, 4, 7];
        assert_eq!(lookup_forte(pf_12), Some("5-Z12"));
        assert_eq!(lookup_forte(pf_36), Some("5-Z36"));
        assert_eq!(find_z_partner("5-Z12"), Some("5-Z36"));
        assert_eq!(find_z_partner("5-Z36"), Some("5-Z12"));
    }

    #[test]
    fn forte_card5_no_duplicate_prime_forms() {
        // Every prime form in the card-5 section must be unique
        let card_5: Vec<&[u8]> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 5)
            .map(|(form, _)| *form)
            .collect();
        for (i, a) in card_5.iter().enumerate() {
            for (j, b) in card_5.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate prime form at indices {} and {}", i, j);
                }
            }
        }
    }

    #[test]
    fn forte_table_covers_all_card_6() {
        // There are exactly 50 cardinality-6 set classes
        let card_6: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 6)
            .collect();
        assert_eq!(card_6.len(), 50);
    }

    #[test]
    fn forte_whole_tone_scale_is_6_35() {
        // Whole-tone scale {0,2,4,6,8,10} → 6-35
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc6, Pc::Pc8, Pc::Pc10];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 2, 4, 6, 8, 10]);
        assert_eq!(lookup_forte(&pf), Some("6-35"));
    }

    #[test]
    fn forte_chromatic_hexachord_is_6_1() {
        // Chromatic hexachord {0,1,2,3,4,5} → 6-1
        let pcs = vec![Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3, Pc::Pc4, Pc::Pc5];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(lookup_forte(&pf), Some("6-1"));
    }

    #[test]
    fn forte_hexatonic_6_20() {
        // Hexatonic (augmented) scale subset {0,1,4,5,8,9} → 6-20
        let pcs = vec![Pc::Pc0, Pc::Pc1, Pc::Pc4, Pc::Pc5, Pc::Pc8, Pc::Pc9];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 4, 5, 8, 9]);
        assert_eq!(lookup_forte(&pf), Some("6-20"));
    }

    #[test]
    fn forte_card6_z_pair() {
        // 6-Z3 and 6-Z36 are Z-related
        let pf_3 = &[0u8, 1, 2, 3, 5, 6];
        let pf_36 = &[0u8, 1, 2, 3, 4, 7];
        assert_eq!(lookup_forte(pf_3), Some("6-Z3"));
        assert_eq!(lookup_forte(pf_36), Some("6-Z36"));
        assert_eq!(find_z_partner("6-Z3"), Some("6-Z36"));
        assert_eq!(find_z_partner("6-Z36"), Some("6-Z3"));
    }

    #[test]
    fn forte_card6_no_duplicate_prime_forms() {
        let card_6: Vec<&[u8]> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 6)
            .map(|(form, _)| *form)
            .collect();
        for (i, a) in card_6.iter().enumerate() {
            for (j, b) in card_6.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate prime form at indices {} and {}", i, j);
                }
            }
        }
    }

    #[test]
    fn forte_table_covers_all_card_7() {
        // There are exactly 38 cardinality-7 set classes
        let card_7: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 7)
            .collect();
        assert_eq!(card_7.len(), 38);
    }

    #[test]
    fn forte_major_scale_is_7_35() {
        // Major scale {0,2,4,5,7,9,11} → prime form [0,1,3,5,6,8,10] → 7-35
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 3, 5, 6, 8, 10]);
        assert_eq!(lookup_forte(&pf), Some("7-35"));
    }

    #[test]
    fn forte_chromatic_heptachord_is_7_1() {
        // Chromatic heptachord {0,1,2,3,4,5,6} → 7-1
        let pcs = vec![Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3, Pc::Pc4, Pc::Pc5, Pc::Pc6];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(lookup_forte(&pf), Some("7-1"));
    }

    #[test]
    fn forte_harmonic_minor_is_7_32() {
        // Harmonic minor {0,2,3,5,7,8,11} → prime form [0,1,3,4,6,8,9] → 7-32
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc3, Pc::Pc5, Pc::Pc7, Pc::Pc8, Pc::Pc11];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 3, 4, 6, 8, 9]);
        assert_eq!(lookup_forte(&pf), Some("7-32"));
    }

    #[test]
    fn forte_melodic_minor_is_7_34() {
        // Melodic minor (ascending) {0,2,3,5,7,9,11} → prime form [0,1,3,4,6,8,10] → 7-34
        let pcs = vec![Pc::Pc0, Pc::Pc2, Pc::Pc3, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 3, 4, 6, 8, 10]);
        assert_eq!(lookup_forte(&pf), Some("7-34"));
    }

    #[test]
    fn forte_card7_z_pair() {
        // 7-Z12 and 7-Z36 are Z-related
        let pf_12 = &[0u8, 1, 2, 3, 4, 7, 9];
        let pf_36 = &[0u8, 1, 2, 3, 5, 6, 8];
        assert_eq!(lookup_forte(pf_12), Some("7-Z12"));
        assert_eq!(lookup_forte(pf_36), Some("7-Z36"));
        assert_eq!(find_z_partner("7-Z12"), Some("7-Z36"));
        assert_eq!(find_z_partner("7-Z36"), Some("7-Z12"));
    }

    #[test]
    fn forte_card7_no_duplicate_prime_forms() {
        let card_7: Vec<&[u8]> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 7)
            .map(|(form, _)| *form)
            .collect();
        for (i, a) in card_7.iter().enumerate() {
            for (j, b) in card_7.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate prime form at indices {} and {}", i, j);
                }
            }
        }
    }

    #[test]
    fn forte_dom7_is_4_27() {
        // Dominant 7th: {0,4,7,10} → prime form [0,2,5,8] → 4-27
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7, Pc::Pc10];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 2, 5, 8]);
        assert_eq!(lookup_forte(&pf), Some("4-27"));
    }

    #[test]
    fn forte_maj7_is_4_20() {
        // Major 7th: {0,4,7,11} → prime form [0,1,5,8] → 4-20
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7, Pc::Pc11];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 5, 8]);
        assert_eq!(lookup_forte(&pf), Some("4-20"));
    }

    #[test]
    fn forte_table_covers_all_card_8() {
        let card_8: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 8)
            .collect();
        assert_eq!(card_8.len(), 29);
    }

    #[test]
    fn forte_table_covers_all_card_9() {
        let card_9: Vec<_> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 9)
            .collect();
        assert_eq!(card_9.len(), 12);
    }

    #[test]
    fn forte_octatonic_is_8_28() {
        // Octatonic scale {0,1,3,4,6,7,9,10} → prime form [0,1,3,4,6,7,9,10] → 8-28
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc3, Pc::Pc4,
            Pc::Pc6, Pc::Pc7, Pc::Pc9, Pc::Pc10,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 3, 4, 6, 7, 9, 10]);
        assert_eq!(lookup_forte(&pf), Some("8-28"));
    }

    #[test]
    fn forte_chromatic_octachord_is_8_1() {
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3,
            Pc::Pc4, Pc::Pc5, Pc::Pc6, Pc::Pc7,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(lookup_forte(&pf), Some("8-1"));
    }

    #[test]
    fn forte_chromatic_nonachord_is_9_1() {
        let pcs = vec![
            Pc::Pc0, Pc::Pc1, Pc::Pc2, Pc::Pc3,
            Pc::Pc4, Pc::Pc5, Pc::Pc6, Pc::Pc7, Pc::Pc8,
        ];
        let pf = prime_form(&pcs);
        assert_eq!(pf, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(lookup_forte(&pf), Some("9-1"));
    }

    #[test]
    fn forte_card8_z_pair() {
        let pf_15 = &[0u8, 1, 2, 3, 4, 6, 8, 9];
        let pf_29 = &[0u8, 1, 2, 3, 5, 6, 7, 9];
        assert_eq!(lookup_forte(pf_15), Some("8-Z15"));
        assert_eq!(lookup_forte(pf_29), Some("8-Z29"));
        assert_eq!(find_z_partner("8-Z15"), Some("8-Z29"));
        assert_eq!(find_z_partner("8-Z29"), Some("8-Z15"));
    }

    #[test]
    fn forte_card8_no_duplicate_prime_forms() {
        let card_8: Vec<&[u8]> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 8)
            .map(|(form, _)| *form)
            .collect();
        for (i, a) in card_8.iter().enumerate() {
            for (j, b) in card_8.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate prime form at indices {} and {}", i, j);
                }
            }
        }
    }

    #[test]
    fn forte_card9_no_duplicate_prime_forms() {
        let card_9: Vec<&[u8]> = FORTE_TABLE
            .iter()
            .filter(|(form, _)| form.len() == 9)
            .map(|(form, _)| *form)
            .collect();
        for (i, a) in card_9.iter().enumerate() {
            for (j, b) in card_9.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate prime form at indices {} and {}", i, j);
                }
            }
        }
    }
}
