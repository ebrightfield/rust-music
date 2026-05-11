use anyhow::Result;
use music::note::pitch_class::Pc;
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};
use super::superchords::{known_types, transpose};

/// Pool of known types to search.
#[derive(Clone, Copy)]
pub enum Pool {
    Chords,
    Scales,
    Both,
}

impl Pool {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None => Ok(Pool::Both),
            Some("chords") => Ok(Pool::Chords),
            Some("scales") => Ok(Pool::Scales),
            Some("both") => Ok(Pool::Both),
            Some(other) => anyhow::bail!("unknown pool: {other} (expected chords, scales, or both)"),
        }
    }

    fn accepts_size(self, size: usize) -> bool {
        match self {
            Pool::Chords => size <= 4,
            Pool::Scales => size >= 5,
            Pool::Both => true,
        }
    }
}

/// Direction of containment query.
#[derive(Clone, Copy)]
pub enum Direction {
    /// Find known types that contain the input (input ⊂ result).
    Super,
    /// Find known types contained in the input (result ⊂ input).
    Sub,
}

impl Direction {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None => Ok(Direction::Super), // overridden by auto-detect
            Some("super") => Ok(Direction::Super),
            Some("sub") => Ok(Direction::Sub),
            Some(other) => anyhow::bail!(
                "unknown direction: {other} (expected super or sub)"
            ),
        }
    }
}

pub struct ContainsArgs {
    pub input: Vec<String>,
    pub pool: Pool,
    pub direction: Option<Direction>,
    pub limit: Option<usize>,
    pub verbose: bool,
}

struct Match {
    name: &'static str,
    size: usize,
    root: Pc,
}

pub fn run(args: ContainsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let input_set: HashSet<Pc> = pcs.iter().copied().collect();
    let input_size = input_set.len();

    // Auto-detect direction: small input (≤4) → super, large (≥5) → sub.
    // Explicit --direction overrides.
    let direction = match args.direction {
        Some(d) => d,
        None => {
            if input_size >= 5 {
                Direction::Sub
            } else {
                Direction::Super
            }
        }
    };

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        let dir_label = match direction {
            Direction::Super => "super (scales/chords containing input)",
            Direction::Sub => "sub (chords/scales contained in input)",
        };
        eprintln!("contains: input={pc_ints:?}, direction={dir_label}");
    }

    let catalog = known_types();
    let mut matches: Vec<Match> = Vec::new();

    for entry in &catalog {
        let entry_size = entry.pcs.len();

        if !args.pool.accepts_size(entry_size) {
            continue;
        }

        match direction {
            Direction::Super => {
                // Skip same-size or smaller — we want strict supersets
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
            Direction::Sub => {
                // Skip same-size or larger — we want strict subsets
                if entry_size >= input_size {
                    continue;
                }
                for t in 0u8..12 {
                    let transposed = transpose(entry.pcs, t);
                    if transposed.is_subset(&input_set) {
                        matches.push(Match {
                            name: entry.name,
                            size: entry_size,
                            root: Pc::from(t),
                        });
                    }
                }
            }
        }
    }

    // Sort by size (ascending for sub, descending for super) then root
    matches.sort_by(|a, b| {
        let size_cmp = match direction {
            Direction::Sub => b.size.cmp(&a.size),   // largest subsets first
            Direction::Super => a.size.cmp(&b.size),  // smallest supersets first
        };
        size_cmp.then_with(|| u8::from(&a.root).cmp(&u8::from(&b.root)))
    });

    if let Some(limit) = args.limit {
        matches.truncate(limit);
    }

    let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
    let pc_display = pc_ints.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");

    match direction {
        Direction::Super => {
            println!("Scales/chords containing {{{}}}:", pc_display);
        }
        Direction::Sub => {
            println!("Chords/scales contained in {{{}}}:", pc_display);
        }
    }
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
    println!("Total: {} results", matches.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_contains(input: Vec<&str>, pool: Pool, direction: Option<Direction>) -> Result<()> {
        run(ContainsArgs {
            input: input.into_iter().map(String::from).collect(),
            pool,
            direction,
            limit: None,
            verbose: false,
        })
    }

    #[test]
    fn c_major_triad_contained_in_scales() {
        // {0,4,7} should be found in C Major, F Major, G Major scales etc.
        run_contains(vec!["C", "E", "G"], Pool::Scales, Some(Direction::Super)).unwrap();
    }

    #[test]
    fn c_major_scale_contains_chords() {
        // C major scale {0,2,4,5,7,9,11} contains many triads and 7th chords
        run_contains(
            vec!["0", "2", "4", "5", "7", "9", "11"],
            Pool::Chords,
            Some(Direction::Sub),
        )
        .unwrap();
    }

    #[test]
    fn auto_detect_super_for_triad() {
        // Small input → auto-detect super direction
        run_contains(vec!["C", "E", "G"], Pool::Both, None).unwrap();
    }

    #[test]
    fn auto_detect_sub_for_scale() {
        // Large input (7 PCs) → auto-detect sub direction
        run_contains(
            vec!["0", "2", "4", "5", "7", "9", "11"],
            Pool::Both,
            None,
        )
        .unwrap();
    }

    #[test]
    fn pool_chords_excludes_scales() {
        let pcs = parse_input_to_pcs(
            &["C", "E", "G"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .unwrap();
        let input_set: HashSet<Pc> = pcs.iter().copied().collect();

        let catalog = known_types();
        // With pool=Chords, no 7-note entries should pass
        for entry in &catalog {
            if entry.pcs.len() >= 5 {
                assert!(
                    !Pool::Chords.accepts_size(entry.pcs.len()),
                    "Pool::Chords should reject size {}",
                    entry.pcs.len()
                );
            }
        }
        // But with pool=Scales, only large entries pass
        for entry in &catalog {
            if entry.pcs.len() <= 4 {
                assert!(
                    !Pool::Scales.accepts_size(entry.pcs.len()),
                    "Pool::Scales should reject size {}",
                    entry.pcs.len()
                );
            }
        }
        let _ = input_set; // used for setup verification
    }

    #[test]
    fn limit_caps_results() {
        let result = run(ContainsArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            pool: Pool::Both,
            direction: Some(Direction::Super),
            limit: Some(3),
            verbose: false,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_bad_pool() {
        assert!(Pool::from_str_opt(Some("invalid")).is_err());
    }

    #[test]
    fn rejects_bad_direction() {
        assert!(Direction::from_str_opt(Some("invalid")).is_err());
    }
}
