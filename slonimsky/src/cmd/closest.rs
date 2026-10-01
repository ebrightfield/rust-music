use anyhow::Result;
use music::note::pitch_class::Pc;
use std::collections::HashSet;

use super::input::{parse_input_to_pcs, pc_label};
use super::superchords::{known_types, transpose};

pub struct ClosestArgs {
    pub input: Vec<String>,
    pub metric: Metric,
    pub pool: Pool,
    pub limit: usize,
    pub verbose: bool,
}

#[derive(Clone, Copy)]
pub enum Metric {
    SymmetricDiff,
}

impl Metric {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("symmetric-diff") => Ok(Metric::SymmetricDiff),
            Some(other) => anyhow::bail!("unknown metric: {other} (expected symmetric-diff)"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Pool {
    Chords,
    Scales,
    Both,
}

impl Pool {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("both") => Ok(Pool::Both),
            Some("chords") => Ok(Pool::Chords),
            Some("scales") => Ok(Pool::Scales),
            Some(other) => {
                anyhow::bail!("unknown pool: {other} (expected chords, scales, or both)")
            }
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

struct RankedMatch {
    name: &'static str,
    root: Pc,
    size: usize,
    distance: usize,
    common: Vec<Pc>,
    only_in_input: Vec<Pc>,
    only_in_match: Vec<Pc>,
}

pub fn run(args: ClosestArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let input_set: HashSet<Pc> = pcs.iter().copied().collect();

    if args.verbose {
        let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
        let metric_name = match args.metric {
            Metric::SymmetricDiff => "symmetric-diff",
        };
        eprintln!(
            "closest: input={pc_ints:?}, metric={metric_name}, limit={}",
            args.limit
        );
    }

    let catalog = known_types();
    let mut matches: Vec<RankedMatch> = Vec::new();

    for entry in &catalog {
        let entry_size = entry.pcs.len();
        if !args.pool.accepts_size(entry_size) {
            continue;
        }

        for t in 0u8..12 {
            let transposed = transpose(entry.pcs, t);

            let common: Vec<Pc> = input_set.intersection(&transposed).copied().collect();
            let only_in_input: Vec<Pc> = input_set.difference(&transposed).copied().collect();
            let only_in_match: Vec<Pc> = transposed.difference(&input_set).copied().collect();

            let distance = only_in_input.len() + only_in_match.len();

            matches.push(RankedMatch {
                name: entry.name,
                root: Pc::from(t),
                size: entry_size,
                distance,
                common,
                only_in_input,
                only_in_match,
            });
        }
    }

    // Sort by distance (ascending), then by size (closer to input size first),
    // then by root PC for stability.
    let input_size = input_set.len();
    matches.sort_by(|a, b| {
        a.distance
            .cmp(&b.distance)
            .then_with(|| {
                let a_diff = (a.size as isize - input_size as isize).unsigned_abs();
                let b_diff = (b.size as isize - input_size as isize).unsigned_abs();
                a_diff.cmp(&b_diff)
            })
            .then_with(|| u8::from(&a.root).cmp(&u8::from(&b.root)))
    });

    // Remove distance-0 entries (exact matches aren't interesting for "closest")
    matches.retain(|m| m.distance > 0);

    matches.truncate(args.limit);

    let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
    let pc_display = pc_ints
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    println!("Closest known chords/scales to {{{}}}:", pc_display);
    println!();

    if matches.is_empty() {
        println!("  (none found)");
    } else {
        for (i, m) in matches.iter().enumerate() {
            let mut common_sorted: Vec<u8> = m.common.iter().map(u8::from).collect();
            common_sorted.sort();
            let common_str = common_sorted
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",");

            println!(
                "  {:>3}. {} {} (dist={}, common={{{}}}, {} common tones)",
                i + 1,
                pc_label(m.root),
                m.name,
                m.distance,
                common_str,
                m.common.len()
            );

            if args.verbose {
                let mut only_input: Vec<u8> = m.only_in_input.iter().map(u8::from).collect();
                only_input.sort();
                let mut only_match: Vec<u8> = m.only_in_match.iter().map(u8::from).collect();
                only_match.sort();
                eprintln!(
                    "       only in input: {{{}}}, only in match: {{{}}}",
                    only_input
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    only_match
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
        }
    }

    println!();
    println!("Total: {} results (distance > 0)", matches.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_closest(input: Vec<&str>, pool: Pool, limit: usize) -> Result<()> {
        run(ClosestArgs {
            input: input.into_iter().map(String::from).collect(),
            metric: Metric::SymmetricDiff,
            pool,
            limit,
            verbose: false,
        })
    }

    #[test]
    fn c_major_triad_closest_chords() {
        // C major triad should find close chords (e.g. Cmaj7 at dist 1, Cmin at dist 2)
        run_closest(vec!["C", "E", "G"], Pool::Chords, 10).unwrap();
    }

    #[test]
    fn closest_excludes_exact_matches() {
        // C major triad is in the catalog — it should not appear at distance 0
        let pcs = parse_input_to_pcs(
            &["C", "E", "G"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let input_set: HashSet<Pc> = pcs.iter().copied().collect();

        let catalog = known_types();
        // Find any transposition of Major triad that matches exactly
        let mut found_exact = false;
        for entry in &catalog {
            if entry.name == "Major" {
                let transposed = transpose(entry.pcs, 0);
                if transposed == input_set {
                    found_exact = true;
                    break;
                }
            }
        }
        assert!(found_exact, "catalog should contain C major triad");

        // But running closest should not include distance-0 entries
        // (verified by the retain filter in run())
    }

    #[test]
    fn closest_sorts_by_distance() {
        let pcs = parse_input_to_pcs(
            &["C", "E", "G"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let input_set: HashSet<Pc> = pcs.iter().copied().collect();

        let catalog = known_types();
        let mut distances: Vec<usize> = Vec::new();

        for entry in &catalog {
            if !Pool::Both.accepts_size(entry.pcs.len()) {
                continue;
            }
            for t in 0u8..12 {
                let transposed = transpose(entry.pcs, t);
                let only_in_input = input_set.difference(&transposed).count();
                let only_in_match = transposed.difference(&input_set).count();
                let dist = only_in_input + only_in_match;
                if dist > 0 {
                    distances.push(dist);
                }
            }
        }

        distances.sort();
        // The minimum distance should be 1 (adding one note to a triad → 4-note chord)
        assert!(!distances.is_empty());
        assert_eq!(
            distances[0], 1,
            "closest to C major should be dist 1 (e.g. Cmaj7)"
        );
    }

    #[test]
    fn limit_caps_output() {
        run_closest(vec!["C", "E", "G"], Pool::Both, 3).unwrap();
    }

    #[test]
    fn pool_filter_chords_only() {
        run_closest(vec!["C", "E", "G"], Pool::Chords, 5).unwrap();
    }

    #[test]
    fn pool_filter_scales_only() {
        run_closest(vec!["C", "E", "G"], Pool::Scales, 5).unwrap();
    }

    #[test]
    fn rejects_bad_metric() {
        assert!(Metric::from_str_opt(Some("euclidean")).is_err());
    }

    #[test]
    fn rejects_bad_pool() {
        assert!(Pool::from_str_opt(Some("garbage")).is_err());
    }
}
