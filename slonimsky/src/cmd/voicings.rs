use anyhow::{Context, Result};
use music::fretboard::Fretboard;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcShape;
use music::note_collections::{StackedIntervals, Voicing};
use musical_combinatorics::canonical_voicings::CanonicalVoicings;
use musical_combinatorics::FourNoteChordQuality;
use musical_combinatorics::ThreeNoteChordQuality;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

use super::input::{parse_input_to_pcs, parse_pc, pc_label};
use super::tuning::TuningSpec;

pub struct VoicingsArgs {
    pub input: Vec<String>,
    pub limit: Option<usize>,
    pub range: Option<String>,
    pub min_spacing: Option<u8>,
    pub max_spacing: Option<u8>,
    pub strings: Option<usize>,
    pub tuning: Option<String>,
    pub doubling: String,
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

fn parse_pitch(value: &str) -> Result<Pitch> {
    let value = value.trim();
    let octave_start = value
        .char_indices()
        .skip(1)
        .find_map(|(index, ch)| (ch.is_ascii_digit() || ch == '-').then_some(index))
        .with_context(|| format!("pitch '{value}' must include an octave (for example, C3)"))?;
    let (note, octave) = value.split_at(octave_start);
    let pc =
        parse_pc(note).with_context(|| format!("unrecognized note name in pitch '{value}'"))?;
    let octave = octave
        .parse::<i8>()
        .with_context(|| format!("invalid octave in pitch '{value}'"))?;
    music::note::pitch::Pitch::try_new(pc.notes()[0], octave)
        .with_context(|| format!("pitch '{value}' is outside the MIDI range"))
}

fn parse_range(value: Option<&str>) -> Result<Option<(Pitch, Pitch)>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let (low, high) = value.split_once("..").with_context(|| {
        format!("invalid range '{value}': expected LOW..HIGH (for example, C3..C6)")
    })?;
    let low = parse_pitch(low)?;
    let high = parse_pitch(high)?;
    anyhow::ensure!(
        low <= high,
        "range lower bound {low} exceeds upper bound {high}"
    );
    Ok(Some((low, high)))
}

fn register_placements(voicing: &Voicing, range: Option<(Pitch, Pitch)>) -> Vec<Voicing> {
    let Some((low, high)) = range else {
        return vec![voicing.clone()];
    };
    let (voicing_low, voicing_high) = voicing.span().expect("canonical voicings are non-empty");
    (-10isize..=10)
        .filter(|shift| {
            let low_midi = voicing_low.midi_note as isize + shift * 12;
            let high_midi = voicing_high.midi_note as isize + shift * 12;
            low_midi >= low.midi_note as isize
                && high_midi <= high.midi_note as isize
                && low_midi >= 0
                && high_midi <= 127
        })
        .map(|shift| {
            voicing
                .move_by_octaves(shift)
                .expect("validated octave shift must remain in MIDI range")
        })
        .collect()
}

fn spacing_matches(voicing: &Voicing, min: Option<u8>, max: Option<u8>) -> bool {
    let intervals = StackedIntervals::from(voicing);
    intervals.iter().all(|interval| {
        min.is_none_or(|min| *interval >= min) && max.is_none_or(|max| *interval <= max)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoublingPolicy {
    Allow,
    Forbid,
    Require,
}

impl DoublingPolicy {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "allow" => Ok(Self::Allow),
            "forbid" => Ok(Self::Forbid),
            "require" => Ok(Self::Require),
            _ => {
                anyhow::bail!("unknown doubling policy '{value}' (options: allow, forbid, require)")
            }
        }
    }
}

struct InstrumentSearch<'a> {
    options: &'a [Vec<Pitch>],
    required_pcs: &'a HashSet<Pc>,
    sounded_strings: usize,
    policy: DoublingPolicy,
    min_spacing: Option<u8>,
    max_spacing: Option<u8>,
    seen: HashSet<Vec<u8>>,
    results: Vec<Voicing>,
}

impl InstrumentSearch<'_> {
    fn visit(&mut self, string: usize, pitches: &mut Vec<Pitch>) {
        if pitches.len() > self.sounded_strings
            || pitches.len() + self.options.len().saturating_sub(string) < self.sounded_strings
        {
            return;
        }
        if string == self.options.len() {
            if pitches.len() != self.sounded_strings {
                return;
            }
            let present = pitches
                .iter()
                .map(|pitch| Pc::from(&pitch.note))
                .collect::<HashSet<_>>();
            if present != *self.required_pcs {
                return;
            }
            let has_doubling = pitches.len() > present.len();
            if matches!(self.policy, DoublingPolicy::Forbid) && has_doubling
                || matches!(self.policy, DoublingPolicy::Require) && !has_doubling
            {
                return;
            }
            let voicing = Voicing::new(pitches.clone());
            if !spacing_matches(&voicing, self.min_spacing, self.max_spacing) {
                return;
            }
            let key = voicing
                .iter()
                .map(|pitch| pitch.midi_note)
                .collect::<Vec<_>>();
            if self.seen.insert(key) {
                self.results.push(voicing);
            }
            return;
        }

        self.visit(string + 1, pitches);
        for pitch in &self.options[string] {
            pitches.push(*pitch);
            self.visit(string + 1, pitches);
            pitches.pop();
        }
    }
}

fn instrument_voicings(
    pcs: &[Pc],
    fretboard: &Fretboard,
    sounded_strings: usize,
    policy: DoublingPolicy,
    range: Option<(Pitch, Pitch)>,
    min_spacing: Option<u8>,
    max_spacing: Option<u8>,
) -> Vec<Voicing> {
    let required_pcs = pcs.iter().copied().collect::<HashSet<_>>();
    let options = fretboard
        .open_strings
        .iter()
        .map(|open| {
            (0..=Fretboard::MAX)
                .filter_map(|fret| open.midi_note.checked_add(fret))
                .filter(|midi| *midi <= 127 && required_pcs.contains(&Pc::from(*midi % 12)))
                .filter(|midi| {
                    range
                        .is_none_or(|(low, high)| *midi >= low.midi_note && *midi <= high.midi_note)
                })
                .filter_map(|midi| Pitch::from_midi(midi).ok())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut search = InstrumentSearch {
        options: &options,
        required_pcs: &required_pcs,
        sounded_strings,
        policy,
        min_spacing,
        max_spacing,
        seen: HashSet::new(),
        results: Vec::new(),
    };
    search.visit(0, &mut Vec::with_capacity(sounded_strings));
    search.results.sort_by_key(|voicing| {
        voicing
            .iter()
            .map(|pitch| pitch.midi_note)
            .collect::<Vec<_>>()
    });
    search.results
}

pub fn run(args: VoicingsArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input).context("parsing input")?;
    let n = pcs.len();
    let range = parse_range(args.range.as_deref())?;
    if let (Some(min), Some(max)) = (args.min_spacing, args.max_spacing) {
        anyhow::ensure!(
            min <= max,
            "--min-spacing ({min}) cannot exceed --max-spacing ({max})"
        );
    }
    let doubling = DoublingPolicy::parse(&args.doubling)?;
    let instrument_mode =
        args.strings.is_some() || args.tuning.is_some() || doubling != DoublingPolicy::Forbid;

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
    println!(
        "Voicings for: {} (PcSet {{{}}})",
        pc_strs.join(" "),
        pcs.iter()
            .map(|pc| format!("{}", *pc as u8))
            .collect::<Vec<_>>()
            .join(", ")
    );

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

    let mut instrument_description = None;
    let families: Vec<(usize, Vec<Voicing>)> = if instrument_mode {
        let tuning = TuningSpec::parse(args.tuning.as_deref().unwrap_or("standard"))?;
        let sounded_strings = args.strings.unwrap_or(n);
        anyhow::ensure!(sounded_strings > 0, "--strings must be greater than zero");
        anyhow::ensure!(
            sounded_strings <= tuning.fretboard.num_strings() as usize,
            "--strings ({sounded_strings}) exceeds tuning '{}' string count ({})",
            tuning.label,
            tuning.fretboard.num_strings()
        );
        anyhow::ensure!(
            sounded_strings >= n,
            "--strings ({sounded_strings}) cannot cover all {n} chord tones"
        );
        match doubling {
            DoublingPolicy::Forbid => anyhow::ensure!(
                sounded_strings == n,
                "--doubling forbid requires --strings to equal the {n} chord tones"
            ),
            DoublingPolicy::Require => anyhow::ensure!(
                sounded_strings > n,
                "--doubling require needs more than {n} sounded strings"
            ),
            DoublingPolicy::Allow => {}
        }
        let placements = instrument_voicings(
            &pcs,
            &tuning.fretboard,
            sounded_strings,
            doubling,
            range,
            args.min_spacing,
            args.max_spacing,
        );
        instrument_description = Some((tuning.label, sounded_strings));
        if placements.is_empty() {
            Vec::new()
        } else {
            vec![(0, placements)]
        }
    } else {
        // Expand canonical forms into octave placements, then apply adjacent-voice spacing.
        let canonical_families: Vec<Vec<Voicing>> = if n == 3 {
            ThreeNoteChordQuality::voicings(&notes)
        } else {
            FourNoteChordQuality::voicings(&notes)
        };
        canonical_families
            .iter()
            .enumerate()
            .filter_map(|(index, family)| {
                let placements = family
                    .iter()
                    .flat_map(|voicing| register_placements(voicing, range))
                    .filter(|voicing| spacing_matches(voicing, args.min_spacing, args.max_spacing))
                    .collect::<Vec<_>>();
                (!placements.is_empty()).then_some((index, placements))
            })
            .collect()
    };

    let total: usize = families.iter().map(|(_, family)| family.len()).sum();
    let limit = args.limit.unwrap_or(total);

    println!();
    if instrument_mode {
        println!("{total} instrument voicings total");
    } else {
        println!("{} families, {} voicings total", families.len(), total);
    }
    if let Some((tuning, strings)) = &instrument_description {
        println!(
            "Tuning: {tuning}, sounded strings: {strings}, doubling: {}",
            args.doubling
        );
    }
    if let Some((low, high)) = range {
        println!("Range: {low}..{high}");
    }
    if args.min_spacing.is_some() || args.max_spacing.is_some() {
        println!(
            "Adjacent spacing: {}..{} semitones",
            args.min_spacing
                .map_or_else(|| "0".to_string(), |v| v.to_string()),
            args.max_spacing
                .map_or_else(|| "unbounded".to_string(), |v| v.to_string())
        );
    }

    let mut count = 0;
    for (family_index, family) in &families {
        if count >= limit {
            break;
        }
        println!();
        if instrument_mode {
            println!("--- Instrument placements ({} voicings) ---", family.len());
        } else {
            let member_label = if range.is_some() {
                "placements"
            } else {
                "inversions"
            };
            println!(
                "--- Family {} ({} {}) ---",
                family_index + 1,
                family.len(),
                member_label
            );
        }

        for (ii, voicing) in family.iter().enumerate() {
            if count >= limit {
                break;
            }
            count += 1;
            let si = StackedIntervals::from(voicing);
            let mut line = String::new();
            write!(
                line,
                "  {}. {} | intervals: {}",
                count,
                format_voicing(voicing),
                format_intervals(&si)
            )
            .unwrap();

            if si.has_wide_intervals() {
                write!(line, " [wide]").unwrap();
            }

            if args.verbose {
                let span = voicing.span();
                if let Some((lo, hi)) = span {
                    write!(line, " | span: {}–{}", lo, hi).unwrap();
                }
                if !instrument_mode {
                    write!(line, " | inversion {}", ii).unwrap();
                }
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
            assert_eq!(
                family.len(),
                3,
                "each triad family should have 3 inversions"
            );
        }
    }

    #[test]
    fn cmaj7_has_6_families() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into(), "B".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = FourNoteChordQuality::voicings(&notes);
        assert_eq!(
            families.len(),
            6,
            "four-note chords should have 6 voicing families"
        );
    }

    #[test]
    fn cmaj7_has_4_inversions_per_family() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into(), "B".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|&pc| pc_to_note(pc)).collect();
        let families = FourNoteChordQuality::voicings(&notes);
        for family in &families {
            assert_eq!(
                family.len(),
                4,
                "each 4-note family should have 4 inversions"
            );
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
                    assert!(
                        w[0] <= w[1],
                        "pitches should be sorted low to high: {:?}",
                        voicing
                    );
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
                assert!(
                    sum <= 36,
                    "intervals shouldn't exceed 3 octaves for a triad"
                );
            }
        }
    }

    #[test]
    fn rejects_two_note_input() {
        let result = run(VoicingsArgs {
            input: vec!["C".into(), "E".into()],
            limit: None,
            range: None,
            min_spacing: None,
            max_spacing: None,
            strings: None,
            tuning: None,
            doubling: "forbid".to_string(),
            verbose: false,
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("3- or 4-note"),
            "error should mention size requirement: {}",
            msg
        );
    }

    #[test]
    fn rejects_five_note_input() {
        let result = run(VoicingsArgs {
            input: vec!["C".into(), "D".into(), "E".into(), "G".into(), "A".into()],
            limit: None,
            range: None,
            min_spacing: None,
            max_spacing: None,
            strings: None,
            tuning: None,
            doubling: "forbid".to_string(),
            verbose: false,
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("3- or 4-note"),
            "error should mention size requirement: {}",
            msg
        );
    }

    #[test]
    fn format_intervals_display() {
        let si = StackedIntervals::new(vec![4, 3, 5]);
        assert_eq!(format_intervals(&si), "[4, 3, 5]");
    }
}
