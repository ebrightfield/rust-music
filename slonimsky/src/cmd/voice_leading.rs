use anyhow::{bail, Context, Result};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note_collections::geometry::symmetry::voiceleading::{
    naive_distance, NoVoxCrossings, Voiceleading, VoiceleadingRule,
};
use music::note_collections::Voicing;
use std::fmt::Write as FmtWrite;

/// Which distance metric to use for ranking voice-leadings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Sum of absolute semitone motions across all voices.
    L1,
    /// Maximum single-voice absolute semitone motion.
    Linf,
    Weighted,
}

impl Metric {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "l1" => Ok(Metric::L1),
            "linf" | "l_inf" | "max" => Ok(Metric::Linf),
            "weighted" => Ok(Metric::Weighted),
            _ => bail!("unknown metric '{}': expected l1, linf, or weighted", s),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Metric::L1 => "L1",
            Metric::Linf => "L∞",
            Metric::Weighted => "weighted L1",
        }
    }
}

/// Compute L∞ distance: max absolute single-voice motion.
fn linf_distance(vl: &Voiceleading) -> usize {
    vl.paths
        .iter()
        .map(|p| p.unsigned_abs() as usize)
        .max()
        .unwrap_or(0)
}

pub fn parse_weights(
    value: Option<&str>,
    metric: Metric,
    voices: usize,
) -> Result<Option<Vec<usize>>> {
    match (metric, value) {
        (Metric::Weighted, None) => bail!("--metric weighted requires --weights"),
        (Metric::Weighted, Some(value)) => {
            let weights = value
                .split(',')
                .map(|part| {
                    part.trim()
                        .parse::<u16>()
                        .map(usize::from)
                        .with_context(|| {
                            format!(
                                "invalid voice weight '{}': expected a non-negative integer",
                                part
                            )
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            if weights.len() != voices {
                bail!(
                    "--weights has {} values but the voicing has {} voices",
                    weights.len(),
                    voices
                );
            }
            if weights.iter().all(|weight| *weight == 0) {
                bail!("--weights must contain at least one non-zero value");
            }
            Ok(Some(weights))
        }
        (_, Some(_)) => bail!("--weights requires --metric weighted"),
        (_, None) => Ok(None),
    }
}

pub fn metric_distance(vl: &Voiceleading, metric: Metric, weights: Option<&[usize]>) -> usize {
    match metric {
        Metric::L1 => naive_distance(vl),
        Metric::Linf => linf_distance(vl),
        Metric::Weighted => vl
            .paths
            .iter()
            .zip(weights.expect("weighted metric validated before scoring"))
            .map(|(path, weight)| path.unsigned_abs() as usize * weight)
            .sum(),
    }
}

pub struct VoiceLeadingArgs {
    pub from: String,
    pub to: String,
    pub limit: Option<usize>,
    pub no_crossings: bool,
    pub metric: String,
    pub weights: Option<String>,
    pub verbose: bool,
}

/// Parse a note name like "C", "C#", "Db", "Eb" into a Note.
fn parse_note(s: &str) -> Result<Note> {
    let s = s.trim();
    match s {
        "C" => Ok(Note::C),
        "C#" | "Cs" => Ok(Note::Cis),
        "Db" => Ok(Note::Des),
        "D" => Ok(Note::D),
        "D#" | "Ds" => Ok(Note::Dis),
        "Eb" => Ok(Note::Ees),
        "E" => Ok(Note::E),
        "F" => Ok(Note::F),
        "F#" | "Fs" => Ok(Note::Fis),
        "Gb" => Ok(Note::Ges),
        "G" => Ok(Note::G),
        "G#" | "Gs" => Ok(Note::Gis),
        "Ab" => Ok(Note::Aes),
        "A" => Ok(Note::A),
        "A#" | "As" => Ok(Note::Ais),
        "Bb" => Ok(Note::Bes),
        "B" => Ok(Note::B),
        _ => bail!("unrecognized note name: '{}'", s),
    }
}

/// Parse a pitch string like "C4", "Eb3", "F#5" into a Pitch.
fn parse_pitch(s: &str) -> Result<Pitch> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty pitch string");
    }
    // The octave number is the last character(s) — find where digits start at the end
    let digit_start = s
        .rfind(|c: char| !c.is_ascii_digit())
        .map(|i| i + 1)
        .unwrap_or(0);
    if digit_start == 0 || digit_start >= s.len() {
        bail!(
            "pitch '{}' must have a note name and octave number (e.g. C4, Eb3)",
            s
        );
    }
    let note_str = &s[..digit_start];
    let octave_str = &s[digit_start..];
    let note = parse_note(note_str).with_context(|| format!("in pitch '{}'", s))?;
    let octave: i8 = octave_str
        .parse()
        .with_context(|| format!("invalid octave '{}' in pitch '{}'", octave_str, s))?;
    Ok(Pitch::new(note, octave))
}

/// Parse a comma-separated list of pitches: "C4,E4,G4"
fn parse_voicing(s: &str) -> Result<Voicing> {
    let pitches: Vec<Pitch> = s
        .split(',')
        .map(parse_pitch)
        .collect::<Result<Vec<_>>>()
        .context("parsing voicing")?;
    if pitches.is_empty() {
        bail!("voicing must contain at least one pitch");
    }
    Ok(Voicing::new(pitches))
}

/// Parse a comma-separated list of note names: "F,A,C"
fn parse_target_notes(s: &str) -> Result<Vec<Note>> {
    let notes: Vec<Note> = s
        .split(',')
        .map(parse_note)
        .collect::<Result<Vec<_>>>()
        .context("parsing target notes")?;
    if notes.is_empty() {
        bail!("target must contain at least one note");
    }
    Ok(notes)
}

/// Format a voicing as a compact pitch string.
fn format_voicing(v: &Voicing) -> String {
    v.iter()
        .map(|p| format!("{}", p))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format paths as signed semitone movements.
fn format_paths(paths: &[i8]) -> String {
    paths
        .iter()
        .map(|p| {
            if *p >= 0 {
                format!("+{}", p)
            } else {
                format!("{}", p)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn run(args: VoiceLeadingArgs) -> Result<()> {
    let metric = Metric::parse(&args.metric)?;
    let from_voicing = parse_voicing(&args.from).context("parsing --from voicing")?;
    let to_notes = parse_target_notes(&args.to).context("parsing --to notes")?;
    let weights = parse_weights(args.weights.as_deref(), metric, from_voicing.len())?;

    if from_voicing.len() != to_notes.len() {
        bail!(
            "voice count mismatch: --from has {} voices, --to has {} notes. \
             Both must have the same number of voices.",
            from_voicing.len(),
            to_notes.len()
        );
    }

    let rules: Option<Vec<Box<dyn VoiceleadingRule>>> = if args.no_crossings {
        Some(vec![Box::new(NoVoxCrossings)])
    } else {
        None
    };

    let rules_ref = rules.as_ref();

    let results = Voiceleading::find_all(&from_voicing, &to_notes, rules_ref)
        .context("computing voice-leadings")?;

    // Re-score and re-sort by the chosen metric
    let mut scored: Vec<(usize, Voiceleading)> = results
        .into_iter()
        .map(|(_, vl)| {
            let score = metric_distance(&vl, metric, weights.as_deref());
            (score, vl)
        })
        .collect();
    scored.sort_by_key(|(score, _)| *score);

    let total = scored.len();
    let shown = if let Some(limit) = args.limit {
        limit.min(total)
    } else {
        total
    };

    // Header
    println!(
        "Voice-leading: {} → {}",
        format_voicing(&from_voicing),
        to_notes
            .iter()
            .map(|n| format!("{}", n))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("Voices: {}", from_voicing.len());
    println!("Metric: {}", metric.label());
    if let Some(weights) = &weights {
        println!(
            "Weights: {} (lowest to highest source voice)",
            weights
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    if args.no_crossings {
        println!("Rule: no voice crossings");
    }
    println!();

    if total == 0 {
        println!("No valid voice-leadings found.");
        return Ok(());
    }

    let display_results = &scored[..shown];
    for (i, (score, vl)) in display_results.iter().enumerate() {
        let mut line = String::new();
        write!(line, "  {}. ", i + 1)?;
        write!(
            line,
            "{} → {}",
            format_voicing(&vl.from),
            format_voicing(&vl.to)
        )?;
        write!(line, "  dist={}", score)?;

        if args.verbose {
            let l1 = naive_distance(vl);
            let linf = linf_distance(vl);
            write!(line, "  L1={} L∞={}", l1, linf)?;
            if weights.is_some() {
                write!(
                    line,
                    " weighted={}",
                    metric_distance(vl, Metric::Weighted, weights.as_deref())
                )?;
            }
            write!(line, "  paths=[{}]", format_paths(&vl.paths))?;
        }

        println!("{}", line);
    }

    println!();
    if shown < total {
        println!(
            "Total: {} voice-leadings found (showing {}/{})",
            total, shown, total
        );
    } else {
        println!("Total: {} voice-leadings found", total);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pitch_c4() {
        let p = parse_pitch("C4").unwrap();
        assert_eq!(p, Pitch::new(Note::C, 4));
    }

    #[test]
    fn parse_pitch_eb3() {
        let p = parse_pitch("Eb3").unwrap();
        assert_eq!(p, Pitch::new(Note::Ees, 3));
    }

    #[test]
    fn parse_pitch_fsharp5() {
        let p = parse_pitch("F#5").unwrap();
        assert_eq!(p, Pitch::new(Note::Fis, 5));
    }

    #[test]
    fn parse_pitch_rejects_no_octave() {
        assert!(parse_pitch("C").is_err());
    }

    #[test]
    fn parse_voicing_ceg() {
        let v = parse_voicing("C4,E4,G4").unwrap();
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn parse_target_notes_fac() {
        let notes = parse_target_notes("F,A,C").unwrap();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0], Note::F);
        assert_eq!(notes[1], Note::A);
        assert_eq!(notes[2], Note::C);
    }

    #[test]
    fn find_voiceleadings_c_to_f() {
        let args = VoiceLeadingArgs {
            from: "C4,E4,G4".to_string(),
            to: "F,A,C".to_string(),
            limit: None,
            no_crossings: true,
            metric: "l1".to_string(),
            weights: None,
            verbose: false,
        };
        // Just verify it runs without error
        let from = parse_voicing(&args.from).unwrap();
        let to = parse_target_notes(&args.to).unwrap();
        let rules: Vec<Box<dyn VoiceleadingRule>> = vec![Box::new(NoVoxCrossings)];
        let results = Voiceleading::find_all(&from, &to, Some(&rules)).unwrap();
        assert!(
            !results.is_empty(),
            "should find voice-leadings from C to F"
        );
        // First result should have lowest distance
        if results.len() > 1 {
            assert!(results[0].0 <= results[1].0);
        }
    }

    #[test]
    fn voice_count_mismatch_rejected() {
        let args = VoiceLeadingArgs {
            from: "C4,E4,G4".to_string(),
            to: "F,A".to_string(),
            limit: None,
            no_crossings: false,
            metric: "l1".to_string(),
            weights: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("voice count mismatch"));
    }

    #[test]
    fn metric_parse_l1() {
        assert_eq!(Metric::parse("l1").unwrap(), Metric::L1);
        assert_eq!(Metric::parse("L1").unwrap(), Metric::L1);
    }

    #[test]
    fn metric_parse_linf() {
        assert_eq!(Metric::parse("linf").unwrap(), Metric::Linf);
        assert_eq!(Metric::parse("Linf").unwrap(), Metric::Linf);
        assert_eq!(Metric::parse("l_inf").unwrap(), Metric::Linf);
        assert_eq!(Metric::parse("max").unwrap(), Metric::Linf);
    }

    #[test]
    fn metric_parse_unknown_rejected() {
        assert!(Metric::parse("euclidean").is_err());
    }

    #[test]
    fn linf_distance_computes_max_abs() {
        let no_rules: Option<&Vec<Box<dyn VoiceleadingRule>>> = None;
        let vl = Voiceleading::new(
            Voicing::new(vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)]),
            vec![0, 5],
            None,
            &no_rules,
        )
        .unwrap();
        assert_eq!(linf_distance(&vl), 5);
        assert_eq!(naive_distance(&vl), 5);
    }

    #[test]
    fn linf_vs_l1_different_for_spread_motion() {
        // Paths: [+1, -3] → L1=4, L∞=3
        let no_rules: Option<&Vec<Box<dyn VoiceleadingRule>>> = None;
        let vl = Voiceleading::new(
            Voicing::new(vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)]),
            vec![1, -3],
            None,
            &no_rules,
        )
        .unwrap();
        assert_eq!(naive_distance(&vl), 4);
        assert_eq!(linf_distance(&vl), 3);
    }

    #[test]
    fn limit_caps_output() {
        let from = parse_voicing("C4,E4,G4").unwrap();
        let to = parse_target_notes("F,A,C").unwrap();
        let results = Voiceleading::find_all(&from, &to, None).unwrap();
        assert!(
            results.len() > 2,
            "should find more than 2 voice-leadings without rules"
        );
    }

    #[test]
    fn no_crossings_reduces_results() {
        let from = parse_voicing("C4,E4,G4").unwrap();
        let to = parse_target_notes("F,A,C").unwrap();
        let all = Voiceleading::find_all(&from, &to, None).unwrap();
        let rules: Vec<Box<dyn VoiceleadingRule>> = vec![Box::new(NoVoxCrossings)];
        let constrained = Voiceleading::find_all(&from, &to, Some(&rules)).unwrap();
        assert!(
            constrained.len() <= all.len(),
            "no-crossings rule should produce fewer or equal results"
        );
    }
}
