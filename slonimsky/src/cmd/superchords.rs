use anyhow::Result;
use music::note::pitch_class::Pc;
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};

use musical_combinatorics::four_note_chords;
use musical_combinatorics::seven_note_scales;
use musical_combinatorics::three_note_chords;

pub struct SuperchordsArgs {
    pub input: Vec<String>,
    pub min_size: Option<u8>,
    pub max_size: Option<u8>,
    pub verbose: bool,
}

/// A known chord/scale type with its canonical PcSet and display name.
pub(crate) struct KnownType {
    pub(crate) name: &'static str,
    pub(crate) pcs: &'static [Pc],
}

/// Build the catalog of known chord/scale types from musical-combinatorics.
pub(crate) fn known_types() -> Vec<KnownType> {
    vec![
        // 3-note chords
        KnownType {
            name: "Major",
            pcs: three_note_chords::MAJOR_PCS,
        },
        KnownType {
            name: "Minor",
            pcs: three_note_chords::MINOR_PCS,
        },
        KnownType {
            name: "Aug",
            pcs: three_note_chords::AUG_PCS,
        },
        KnownType {
            name: "Dim",
            pcs: three_note_chords::DIM_PCS,
        },
        KnownType {
            name: "PP (sus4+5)",
            pcs: three_note_chords::PP_PCS,
        },
        KnownType {
            name: "AP",
            pcs: three_note_chords::AP_PCS,
        },
        KnownType {
            name: "PA",
            pcs: three_note_chords::PA_PCS,
        },
        KnownType {
            name: "MW",
            pcs: three_note_chords::MW_PCS,
        },
        KnownType {
            name: "WM",
            pcs: three_note_chords::WM_PCS,
        },
        KnownType {
            name: "MH",
            pcs: three_note_chords::MH_PCS,
        },
        KnownType {
            name: "HM",
            pcs: three_note_chords::HM_PCS,
        },
        KnownType {
            name: "AW",
            pcs: three_note_chords::AW_PCS,
        },
        KnownType {
            name: "WA",
            pcs: three_note_chords::WA_PCS,
        },
        KnownType {
            name: "HA",
            pcs: three_note_chords::HA_PCS,
        },
        KnownType {
            name: "AH",
            pcs: three_note_chords::AH_PCS,
        },
        KnownType {
            name: "WW",
            pcs: three_note_chords::WW_PCS,
        },
        KnownType {
            name: "WH",
            pcs: three_note_chords::WH_PCS,
        },
        KnownType {
            name: "HW",
            pcs: three_note_chords::HW_PCS,
        },
        KnownType {
            name: "HH",
            pcs: three_note_chords::HH_PCS,
        },
        // 4-note chords
        KnownType {
            name: "Maj7",
            pcs: four_note_chords::MAJ7_PCS,
        },
        KnownType {
            name: "Dom7",
            pcs: four_note_chords::DOM7_PCS,
        },
        KnownType {
            name: "Min7",
            pcs: four_note_chords::MIN7_PCS,
        },
        KnownType {
            name: "MinMaj7",
            pcs: four_note_chords::MINMAJ7_PCS,
        },
        KnownType {
            name: "Dim7",
            pcs: four_note_chords::DIM7_PCS,
        },
        KnownType {
            name: "Min7b5",
            pcs: four_note_chords::MIN7_FLAT5_PCS,
        },
        KnownType {
            name: "Aug7",
            pcs: four_note_chords::AUG7_PCS,
        },
        KnownType {
            name: "AugMaj7",
            pcs: four_note_chords::AUG_MAJ7_PCS,
        },
        KnownType {
            name: "Dom7b5",
            pcs: four_note_chords::DOM7_FLAT5_PCS,
        },
        KnownType {
            name: "Maj(add9)",
            pcs: four_note_chords::MAJ9_PCS,
        },
        KnownType {
            name: "Maj(addb9)",
            pcs: four_note_chords::MAJ_FLAT9_PCS,
        },
        KnownType {
            name: "Maj(add#9)",
            pcs: four_note_chords::MAJ_SHARP9_PCS,
        },
        KnownType {
            name: "Min(add9)",
            pcs: four_note_chords::MIN9_PCS,
        },
        KnownType {
            name: "Min(addb9)",
            pcs: four_note_chords::MIN_FLAT9_PCS,
        },
        KnownType {
            name: "Dim(add9)",
            pcs: four_note_chords::DIM9_PCS,
        },
        KnownType {
            name: "Dim(addb9)",
            pcs: four_note_chords::DIM_FLAT9_PCS,
        },
        KnownType {
            name: "Maj(add11)",
            pcs: four_note_chords::MAJ11_PCS,
        },
        KnownType {
            name: "Maj(add#11)",
            pcs: four_note_chords::MAJ_SHARP11_PCS,
        },
        KnownType {
            name: "Min(add11)",
            pcs: four_note_chords::MIN11_PCS,
        },
        KnownType {
            name: "Min(add#11)",
            pcs: four_note_chords::MIN_SHARP11_PCS,
        },
        KnownType {
            name: "Dim(add11)",
            pcs: four_note_chords::DIM11_PCS,
        },
        KnownType {
            name: "Dim(addb11)",
            pcs: four_note_chords::DIM_FLAT11_PCS,
        },
        // 7-note scales
        KnownType {
            name: "Major",
            pcs: seven_note_scales::MAJOR_SCALE_PCS,
        },
        KnownType {
            name: "Melodic Minor",
            pcs: seven_note_scales::MELODIC_MINOR_PCS,
        },
        KnownType {
            name: "Harmonic Minor",
            pcs: seven_note_scales::HARMONIC_MINOR_PCS,
        },
        KnownType {
            name: "Harmonic Major",
            pcs: seven_note_scales::HARMONIC_MAJOR_PCS,
        },
        KnownType {
            name: "Major b9 b13",
            pcs: seven_note_scales::MAJOR_FLAT9_FLAT13_PCS,
        },
        KnownType {
            name: "Mixolydian #11 b13",
            pcs: seven_note_scales::MIXOLYDIAN_SHARP11_FLAT13_PCS,
        },
        KnownType {
            name: "Major b9",
            pcs: seven_note_scales::MAJOR_FLAT9_PCS,
        },
        KnownType {
            name: "Major #9",
            pcs: seven_note_scales::MAJOR_SHARP9_PCS,
        },
        KnownType {
            name: "Mixolydian #9",
            pcs: seven_note_scales::MIXOLYDIAN_SHARP9_PCS,
        },
        KnownType {
            name: "Lydian b9",
            pcs: seven_note_scales::LYDIAN_FLAT9_PCS,
        },
        KnownType {
            name: "Mixolydian #9 #11",
            pcs: seven_note_scales::MIXOLYDIAN_SHARP9_SHARP11_PCS,
        },
        KnownType {
            name: "Mixolydian b9 #11",
            pcs: seven_note_scales::MIXOLYDIAN_FLAT9_SHARP11_PCS,
        },
        KnownType {
            name: "Melodic Minor b9 #11",
            pcs: seven_note_scales::MELODIC_MINOR_FLAT9_SHARP11_PCS,
        },
        KnownType {
            name: "Mixolydian b9 #11 b13",
            pcs: seven_note_scales::MIXOLYDIAN_FLAT9_SHARP11_FLAT13_PCS,
        },
        KnownType {
            name: "Mixolydian #9 b13",
            pcs: seven_note_scales::MIXOLYDIAN_SHARP9_FLAT13_PCS,
        },
        KnownType {
            name: "Melodic Minor b11",
            pcs: seven_note_scales::MELODIC_MINOR_FLAT11_PCS,
        },
        KnownType {
            name: "Mixolydian #9 #11 b13",
            pcs: seven_note_scales::MIXOLYDIAN_SHARP9_SHARP11_FLAT13_PCS,
        },
        KnownType {
            name: "Melodic Minor b9 b11",
            pcs: seven_note_scales::MELODIC_MINOR_FLAT9_FLAT11_PCS,
        },
        KnownType {
            name: "Lydian b9 b13",
            pcs: seven_note_scales::LYDIAN_FLAT9_FLAT13_PCS,
        },
        KnownType {
            name: "Melodic Minor b9 #11 b13",
            pcs: seven_note_scales::MELODIC_MINOR_FLAT9_SHARP11_FLAT13_PCS,
        },
        KnownType {
            name: "Lydian #9 b13",
            pcs: seven_note_scales::LYDIAN_SHARP9_FLAT13_PCS,
        },
        KnownType {
            name: "Major #9 b13",
            pcs: seven_note_scales::MAJOR_SHARP9_FLAT13_PCS,
        },
    ]
}

/// Transpose a slice of Pcs by `offset` semitones, returning a HashSet.
pub(crate) fn transpose(pcs: &[Pc], offset: u8) -> HashSet<Pc> {
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

/// A match: a known type at a specific transposition contains the input.
struct Match {
    name: &'static str,
    size: usize,
    root: Pc,
}

pub fn run(args: SuperchordsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let input_set: HashSet<Pc> = pcs.iter().copied().collect();
    let input_size = input_set.len();

    let min_size = args.min_size.unwrap_or((input_size + 1) as u8) as usize;
    let max_size = args.max_size.unwrap_or(12) as usize;

    anyhow::ensure!(
        min_size > input_size,
        "min-size ({min_size}) must be greater than input size ({input_size})"
    );

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        eprintln!("superchords: input={pc_ints:?}, size range={min_size}..={max_size}");
    }

    let catalog = known_types();
    let mut matches: Vec<Match> = Vec::new();

    for entry in &catalog {
        let entry_size = entry.pcs.len();
        if entry_size < min_size || entry_size > max_size {
            continue;
        }
        // Skip if candidate is same size or smaller than input
        if entry_size <= input_size {
            continue;
        }
        for t in 0u8..12 {
            let transposed = transpose(entry.pcs, t);
            if input_set.is_subset(&transposed) {
                matches.push(Match {
                    name: entry.name,
                    size: entry_size,
                    root: Pc::from(t),
                });
            }
        }
    }

    // Sort by size then by root
    matches.sort_by(|a, b| {
        a.size
            .cmp(&b.size)
            .then_with(|| u8::from(&a.root).cmp(&u8::from(&b.root)))
    });

    // Print the queried pitch classes themselves, not the zero-anchored shape
    // `PcShape::new` would produce — the matches below are reported at absolute
    // roots, so a prime-form header would disagree with them.
    let mut header_pcs: Vec<u8> = input_set.iter().map(u8::from).collect();
    header_pcs.sort();
    println!(
        "Superchords of {{{}}} (size {min_size}..={max_size}):",
        header_pcs
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!();

    if matches.is_empty() {
        println!("  (none found)");
    } else {
        let mut current_size = 0;
        for (i, m) in matches.iter().enumerate() {
            if m.size != current_size {
                if current_size != 0 {
                    println!();
                }
                let kind = if m.size <= 4 { "chords" } else { "scales" };
                println!("  --- {}-note {kind} ---", m.size);
                current_size = m.size;
            }
            println!("  {:>3}. {} {}", i + 1, pc_label(m.root), m.name);
        }
    }

    println!();
    println!("Total: {} superchords", matches.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_triad_finds_four_note_superchords() {
        // C major triad {0,4,7} should be contained in Cmaj7, C dom7, etc.
        let args = SuperchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            min_size: None,
            max_size: Some(4),
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn c_major_triad_finds_scales() {
        // C major triad should be in C Major scale, plus others
        let args = SuperchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            min_size: Some(7),
            max_size: Some(7),
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn superchords_rejects_min_size_too_small() {
        // min_size must be > input size
        let args = SuperchordsArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            min_size: Some(3),
            max_size: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }

    #[test]
    fn dim7_superchords_include_known_types() {
        // Dim7 {0,3,6,9} — should find 7-note scales containing it
        let args = SuperchordsArgs {
            input: vec!["0".into(), "3".into(), "6".into(), "9".into()],
            min_size: Some(7),
            max_size: Some(7),
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn transposition_finds_matches_at_all_roots() {
        // A minor triad {9,0,4} = A,C,E should find superchords rooted at A
        let input: Vec<Pc> = vec![Pc::Pc9, Pc::Pc0, Pc::Pc4];
        let input_set: HashSet<Pc> = input.into_iter().collect();
        // A minor triad should be in A Melodic Minor (A=9, transposition=9)
        let mel_minor_at_a = transpose(seven_note_scales::MELODIC_MINOR_PCS, 9);
        assert!(
            input_set.is_subset(&mel_minor_at_a),
            "A minor triad should be subset of A Melodic Minor"
        );
    }

    #[test]
    fn transpose_correctness() {
        // Transpose C major triad {0,4,7} by 2 → {2,6,9} = D major triad
        let transposed = transpose(three_note_chords::MAJOR_PCS, 2);
        assert!(transposed.contains(&Pc::Pc2));
        assert!(transposed.contains(&Pc::Pc6));
        assert!(transposed.contains(&Pc::Pc9));
        assert_eq!(transposed.len(), 3);
    }
}
