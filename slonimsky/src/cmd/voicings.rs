use anyhow::{Context, Result};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcShape;
use music::note_collections::{StackedIntervals, Voicing};
use musical_combinatorics::canonical_voicings::CanonicalVoicings;
use musical_combinatorics::FourNoteChordQuality;
use musical_combinatorics::ThreeNoteChordQuality;
use std::fmt::Write as FmtWrite;

use super::input::{parse_input_to_pcs, pc_label};

pub struct VoicingsArgs {
    pub input: Vec<String>,
    pub limit: Option<usize>,
    pub verbose: bool,
}

/// Pick the most common spelling for a pitch class.
fn pc_to_note(pc: Pc) -> Note {
    pc.notes()[0]
}

/// Format a voicing as a compact pitch string (e.g. "C4 E4 G4").
fn format_voicing(v: &Voicing) -> String {
    v.iter()
        .map(|p| format!("{}", p))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format stacked intervals as a bracketed list (e.g. "[4, 3, 5]").
fn format_intervals(si: &StackedIntervals) -> String {
    let nums: Vec<String> = si.iter().map(|i| i.to_string()).collect();
    format!("[{}]", nums.join(", "))
}

pub fn run(args: VoicingsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input).context("parsing input")?;
    let n = pcs.len();

    if !(3..=4).contains(&n) {
        anyhow::bail!(
            "voicings requires a 3- or 4-note chord (got {} pitch classes). \
             The canonical voicing algorithm is defined for triads and seventh chords.",
            n
        );
    }

    // Convert PCs to Notes in input order
    let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();

    let pc_set = PcShape::new(pcs.clone());

    // Display header
    let pc_strs: Vec<String> = pcs.iter().map(|pc| pc_label(*pc).to_string()).collect();
    println!("Voicings for: {} (PcSet {{{}}})", pc_strs.join(" "),
        pcs.iter().map(|pc| format!("{}", *pc as u8)).collect::<Vec<_>>().join(", "));

    // Identify quality for labeling
    let quality_label = if n == 3 {
        ThreeNoteChordQuality::identify(&pc_set)
            .ok()
            .map(|(_, q)| format!("{:?}", q))
    } else {
        FourNoteChordQuality::identify(&pc_set)
            .ok()
            .map(|(_, q)| format!("{:?}", q))
    };

    if let Some(ref ql) = quality_label {
        println!("Quality: {}", ql);
    }

    // Compute canonical voicings
    let families: Vec<Vec<Voicing>> = if n == 3 {
        ThreeNoteChordQuality::voicings(&notes)
    } else {
        FourNoteChordQuality::voicings(&notes)
    };

    let total: usize = families.iter().map(|f| f.len()).sum();
    let limit = args.limit.unwrap_or(total);

    println!();
    println!("{} families, {} voicings total", families.len(), total);

    let mut count = 0;
    for (fi, family) in families.iter().enumerate() {
        if count >= limit {
            break;
        }
        println!();
        println!("--- Family {} ({} inversions) ---", fi + 1, family.len());

        for (ii, voicing) in family.iter().enumerate() {
            if count >= limit {
                break;
            }
            count += 1;
            let si = StackedIntervals::from(voicing);
            let mut line = String::new();
            write!(line, "  {}. {} | intervals: {}", count, format_voicing(voicing), format_intervals(&si)).unwrap();

            if si.has_wide_intervals() {
                write!(line, " [wide]").unwrap();
            }

            if args.verbose {
                let span = voicing.span();
                if let Some((lo, hi)) = span {
                    write!(line, " | span: {}–{}", lo, hi).unwrap();
                }
                write!(line, " | inversion {}", ii).unwrap();
            }

            println!("{}", line);
        }
    }

    if count < total {
        println!();
        println!("(showing {}/{}, use --limit to see more)", count, total);
    }

    println!();
    println!("Total: {} voicings", std::cmp::min(count, limit));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_triad_has_2_families() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = ThreeNoteChordQuality::voicings(&notes);
        assert_eq!(families.len(), 2, "triads should have 2 voicing families");
    }

    #[test]
    fn c_major_triad_has_3_inversions_per_family() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = ThreeNoteChordQuality::voicings(&notes);
        for family in &families {
            assert_eq!(family.len(), 3, "each triad family should have 3 inversions");
        }
    }

    #[test]
    fn cmaj7_has_6_families() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into(), "B".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = FourNoteChordQuality::voicings(&notes);
        assert_eq!(families.len(), 6, "four-note chords should have 6 voicing families");
    }

    #[test]
    fn cmaj7_has_4_inversions_per_family() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into(), "B".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = FourNoteChordQuality::voicings(&notes);
        for family in &families {
            assert_eq!(family.len(), 4, "each 4-note family should have 4 inversions");
        }
    }

    #[test]
    fn voicings_are_sorted_low_to_high() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = ThreeNoteChordQuality::voicings(&notes);
        for family in &families {
            for voicing in family {
                let pitches: Vec<_> = voicing.iter().collect();
                for w in pitches.windows(2) {
                    assert!(w[0] <= w[1], "pitches should be sorted low to high: {:?}", voicing);
                }
            }
        }
    }

    #[test]
    fn stacked_intervals_sum_reasonable() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = ThreeNoteChordQuality::voicings(&notes);
        for family in &families {
            for voicing in family {
                let si = StackedIntervals::from(voicing);
                let sum: u8 = si.iter().sum();
                // Sum should be reasonable (< 3 octaves for a triad)
                assert!(sum > 0, "intervals should be non-zero");
                assert!(sum <= 36, "intervals shouldn't exceed 3 octaves for a triad");
            }
        }
    }

    #[test]
    fn rejects_two_note_input() {
        let result = run(VoicingsArgs {
            input: vec!["C".into(), "E".into()],
            limit: None,
            verbose: false,
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("3- or 4-note"), "error should mention size requirement: {}", msg);
    }

    #[test]
    fn rejects_five_note_input() {
        let result = run(VoicingsArgs {
            input: vec!["C".into(), "D".into(), "E".into(), "G".into(), "A".into()],
            limit: None,
            verbose: false,
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("3- or 4-note"), "error should mention size requirement: {}", msg);
    }

    #[test]
    fn format_intervals_display() {
        let si = StackedIntervals::new(vec![4, 3, 5]);
        assert_eq!(format_intervals(&si), "[4, 3, 5]");
    }
}
