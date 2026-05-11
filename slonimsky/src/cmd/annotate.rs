use anyhow::{bail, Context, Result};
use music::note::note::Note;
use music::note::pitch::Pitch;
use std::fmt::Write as FmtWrite;
use std::fs;

pub struct AnnotateArgs {
    pub chords: Vec<String>,
    pub key: Option<String>,
    #[allow(dead_code)]
    pub scale: Option<String>,
    pub no_crossings: bool,
    pub format: OutputFormat,
    pub output: Option<String>,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("text") => Ok(Self::Text),
            Some("json") => Ok(Self::Json),
            Some(other) => bail!("unknown format: '{other}' (options: text, json)"),
        }
    }

    /// Infer format from file extension when `--format` is not given.
    pub fn infer_from_output(path: &str, explicit: Self) -> Self {
        if explicit != Self::Text {
            return explicit;
        }
        if path.ends_with(".json") {
            Self::Json
        } else {
            Self::Text
        }
    }
}

/// Parse a note name like "C", "C#", "Db" into a Note.
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
    let octave: u8 = octave_str
        .parse()
        .with_context(|| format!("invalid octave '{}' in pitch '{}'", octave_str, s))?;
    Ok(Pitch::new(note, octave))
}

/// Parse a comma-separated group of pitched notes: "C4,E4,G4"
fn parse_chord_voicing(s: &str) -> Result<Vec<Pitch>> {
    let pitches: Vec<Pitch> = s
        .split(',')
        .map(parse_pitch)
        .collect::<Result<Vec<_>>>()
        .with_context(|| format!("parsing chord '{}'", s))?;
    if pitches.is_empty() {
        bail!("chord voicing must contain at least one pitch");
    }
    Ok(pitches)
}

/// MIDI note number for a Pitch (C4 = 60).
fn midi_number(p: &Pitch) -> i32 {
    use music::note::pitch_class::Pc;
    let pc: Pc = (&p.note).into();
    let pc_val = pc as i32;
    pc_val + (p.octave as i32) * 12
}

/// Compute per-voice semitone motion (signed).
fn voice_paths(from: &[Pitch], to: &[Pitch]) -> Vec<i32> {
    from.iter()
        .zip(to.iter())
        .map(|(f, t)| midi_number(t) - midi_number(f))
        .collect()
}

/// L1 distance (sum of absolute semitone motions).
fn l1_cost(paths: &[i32]) -> i32 {
    paths.iter().map(|p| p.abs()).sum()
}

/// L∞ distance (max absolute semitone motion).
fn linf_cost(paths: &[i32]) -> i32 {
    paths.iter().map(|p| p.abs()).max().unwrap_or(0)
}

/// Check for voice crossings: voices should not swap register positions.
fn count_crossings(from: &[Pitch], to: &[Pitch]) -> usize {
    let mut crossings = 0;
    for i in 0..from.len() {
        for j in (i + 1)..from.len() {
            let from_order = midi_number(&from[i]).cmp(&midi_number(&from[j]));
            let to_order = midi_number(&to[i]).cmp(&midi_number(&to[j]));
            if from_order != to_order
                && from_order != std::cmp::Ordering::Equal
                && to_order != std::cmp::Ordering::Equal
            {
                crossings += 1;
            }
        }
    }
    crossings
}

/// Classify voice motion type.
fn motion_label(semitones: i32) -> &'static str {
    match semitones.abs() {
        0 => "common tone",
        1 | 2 => "step",
        3 | 4 => "skip",
        _ => "leap",
    }
}

/// Smoothness rating for a step, scaled by voice count.
fn smoothness_rating(l1: i32, voice_count: usize) -> &'static str {
    let scale = voice_count as f64 / 3.0;
    let threshold_excellent = (4.0 * scale) as i32;
    let threshold_good = (7.0 * scale) as i32;
    let threshold_fair = (10.0 * scale) as i32;
    if l1 <= threshold_excellent {
        "excellent"
    } else if l1 <= threshold_good {
        "good"
    } else if l1 <= threshold_fair {
        "fair"
    } else {
        "poor"
    }
}

fn format_pitch(p: &Pitch) -> String {
    format!("{}{}", p.note, p.octave)
}

fn format_signed(v: i32) -> String {
    if v >= 0 {
        format!("+{}", v)
    } else {
        format!("{}", v)
    }
}

pub fn run(args: AnnotateArgs) -> Result<()> {
    if args.chords.len() < 2 {
        bail!("annotate requires at least 2 chords (got {})", args.chords.len());
    }

    let voicings: Vec<Vec<Pitch>> = args
        .chords
        .iter()
        .map(|s| parse_chord_voicing(s))
        .collect::<Result<Vec<_>>>()?;

    // Validate equal voice count
    let voice_count = voicings[0].len();
    for (i, v) in voicings.iter().enumerate().skip(1) {
        if v.len() != voice_count {
            bail!(
                "all chords must have the same number of voices: chord 1 has {}, chord {} has {}",
                voice_count,
                i + 1,
                v.len()
            );
        }
    }

    // Infer format from output path when --format wasn't explicitly set
    let format = match &args.output {
        Some(path) => OutputFormat::infer_from_output(path, args.format),
        None => args.format,
    };

    let output_str = match format {
        OutputFormat::Text => format_text(&voicings, voice_count, &args.key, args.no_crossings),
        OutputFormat::Json => format_json_str(&voicings, voice_count, args.no_crossings),
    };

    match &args.output {
        Some(path) => {
            fs::write(path, &output_str)
                .with_context(|| format!("writing output to '{}'", path))?;
        }
        None => {
            print!("{}", output_str);
        }
    }

    if args.verbose {
        let steps = voicings.len() - 1;
        let total_l1: i32 = (0..steps)
            .map(|i| l1_cost(&voice_paths(&voicings[i], &voicings[i + 1])))
            .sum();
        let total_crossings: usize = (0..steps)
            .map(|i| count_crossings(&voicings[i], &voicings[i + 1]))
            .sum();
        eprintln!(
            "[annotate] {} steps, {} voices, total L1={}, crossings={}",
            steps, voice_count, total_l1, total_crossings
        );
    }

    Ok(())
}

fn format_text(
    voicings: &[Vec<Pitch>],
    voice_count: usize,
    key: &Option<String>,
    flag_crossings: bool,
) -> String {
    let mut out = String::new();

    let key_label = key.as_deref().unwrap_or("(not specified)");
    writeln!(
        out,
        "Key: {}",
        if key.is_some() {
            key_label
        } else {
            "(not specified)"
        }
    )
    .unwrap();
    writeln!(out).unwrap();

    let mut total_l1 = 0i32;
    let mut total_linf = 0i32;
    let mut total_crossings = 0usize;
    let steps = voicings.len() - 1;

    for step in 0..steps {
        let from = &voicings[step];
        let to = &voicings[step + 1];
        let paths = voice_paths(from, to);
        let step_l1 = l1_cost(&paths);
        let step_linf = linf_cost(&paths);
        let crossings = count_crossings(from, to);

        total_l1 += step_l1;
        total_linf = total_linf.max(step_linf);
        total_crossings += crossings;

        let from_str: Vec<String> = from.iter().map(format_pitch).collect();
        let to_str: Vec<String> = to.iter().map(format_pitch).collect();

        writeln!(
            out,
            "Step {}: {} → {}",
            step + 1,
            from_str.join(","),
            to_str.join(",")
        )
        .unwrap();

        for v in 0..voice_count {
            let semitones = paths[v];
            let motion = motion_label(semitones);
            writeln!(
                out,
                "  Voice {}: {} → {}  ({} st, {})",
                v + 1,
                from_str[v],
                to_str[v],
                format_signed(semitones),
                motion
            )
            .unwrap();
        }

        let rating = smoothness_rating(step_l1, voice_count);
        writeln!(
            out,
            "  L1 cost: {}   L∞ cost: {}   Crossings: {}",
            step_l1,
            step_linf,
            if crossings == 0 {
                "none".to_string()
            } else {
                crossings.to_string()
            }
        )
        .unwrap();
        writeln!(out, "  Smoothness: {}", rating).unwrap();

        if flag_crossings && crossings > 0 {
            writeln!(out, "  ⚠ Voice crossing detected!").unwrap();
        }

        writeln!(out).unwrap();
    }

    // Summary
    writeln!(out, "Summary:").unwrap();
    writeln!(out, "  Total L1 cost: {}", total_l1).unwrap();
    if steps > 0 {
        writeln!(
            out,
            "  Average L1 per step: {:.1}",
            total_l1 as f64 / steps as f64
        )
        .unwrap();
    }
    writeln!(out, "  Max L∞ cost: {}", total_linf).unwrap();
    writeln!(out, "  Voice crossings: {}", total_crossings).unwrap();
    let overall = smoothness_rating(
        if steps > 0 {
            total_l1 / steps as i32
        } else {
            total_l1
        },
        voice_count,
    );
    writeln!(out, "  Smoothness rating: {}", overall).unwrap();

    out
}

fn format_json_str(
    voicings: &[Vec<Pitch>],
    voice_count: usize,
    _flag_crossings: bool,
) -> String {
    let steps = voicings.len() - 1;
    let mut json = String::new();
    writeln!(json, "{{").unwrap();
    writeln!(json, "  \"voices\": {},", voice_count).unwrap();
    writeln!(json, "  \"steps\": [").unwrap();

    let mut total_l1 = 0i32;
    let mut total_crossings = 0usize;

    for step in 0..steps {
        let from = &voicings[step];
        let to = &voicings[step + 1];
        let paths = voice_paths(from, to);
        let step_l1 = l1_cost(&paths);
        let step_linf = linf_cost(&paths);
        let crossings = count_crossings(from, to);
        total_l1 += step_l1;
        total_crossings += crossings;

        let from_str: Vec<String> = from.iter().map(|p| format!("\"{}\"", format_pitch(p))).collect();
        let to_str: Vec<String> = to.iter().map(|p| format!("\"{}\"", format_pitch(p))).collect();
        let paths_str: Vec<String> = paths.iter().map(|p| p.to_string()).collect();

        let comma = if step < steps - 1 { "," } else { "" };
        writeln!(json, "    {{").unwrap();
        writeln!(json, "      \"from\": [{}],", from_str.join(", ")).unwrap();
        writeln!(json, "      \"to\": [{}],", to_str.join(", ")).unwrap();
        writeln!(json, "      \"paths\": [{}],", paths_str.join(", ")).unwrap();
        writeln!(json, "      \"l1_cost\": {},", step_l1).unwrap();
        writeln!(json, "      \"linf_cost\": {},", step_linf).unwrap();
        writeln!(json, "      \"crossings\": {},", crossings).unwrap();
        writeln!(
            json,
            "      \"smoothness\": \"{}\"",
            smoothness_rating(step_l1, voice_count)
        )
        .unwrap();
        writeln!(json, "    }}{}", comma).unwrap();
    }

    let avg_l1 = if steps > 0 {
        total_l1 as f64 / steps as f64
    } else {
        0.0
    };
    let overall = smoothness_rating(
        if steps > 0 {
            total_l1 / steps as i32
        } else {
            total_l1
        },
        voice_count,
    );

    writeln!(json, "  ],").unwrap();
    writeln!(json, "  \"summary\": {{").unwrap();
    writeln!(json, "    \"total_l1\": {},", total_l1).unwrap();
    writeln!(json, "    \"average_l1\": {:.1},", avg_l1).unwrap();
    writeln!(json, "    \"total_crossings\": {},", total_crossings).unwrap();
    writeln!(json, "    \"smoothness\": \"{}\"", overall).unwrap();
    writeln!(json, "  }}").unwrap();
    write!(json, "}}").unwrap();

    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chord_voicing_3_voices() {
        let v = parse_chord_voicing("C4,E4,G4").unwrap();
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], Pitch::new(Note::C, 4));
    }

    #[test]
    fn voice_paths_common_tone() {
        let from = vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)];
        let to = vec![Pitch::new(Note::C, 4), Pitch::new(Note::F, 4)];
        let paths = voice_paths(&from, &to);
        assert_eq!(paths[0], 0); // common tone
        assert_eq!(paths[1], 1); // E→F = +1
    }

    #[test]
    fn l1_cost_sum() {
        assert_eq!(l1_cost(&[0, 1, -2]), 3);
    }

    #[test]
    fn linf_cost_max() {
        assert_eq!(linf_cost(&[0, 1, -2]), 2);
    }

    #[test]
    fn smoothness_3_voice_excellent() {
        assert_eq!(smoothness_rating(3, 3), "excellent");
    }

    #[test]
    fn smoothness_3_voice_poor() {
        assert_eq!(smoothness_rating(15, 3), "poor");
    }

    #[test]
    fn smoothness_scales_with_voices() {
        // 6 voices → threshold_excellent = 4 * 6/3 = 8
        assert_eq!(smoothness_rating(7, 6), "excellent");
        // Same L1 with 3 voices would be "poor"
        assert_eq!(smoothness_rating(7, 3), "good");
    }

    #[test]
    fn no_crossings_detected() {
        let from = vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)];
        let to = vec![Pitch::new(Note::D, 4), Pitch::new(Note::F, 4)];
        assert_eq!(count_crossings(&from, &to), 0);
    }

    #[test]
    fn crossing_detected() {
        let from = vec![Pitch::new(Note::C, 4), Pitch::new(Note::G, 4)];
        // Voice 1 goes up to A4, Voice 2 goes down to D4 → they cross
        let to = vec![Pitch::new(Note::A, 4), Pitch::new(Note::D, 4)];
        assert_eq!(count_crossings(&from, &to), 1);
    }

    #[test]
    fn motion_labels_correct() {
        assert_eq!(motion_label(0), "common tone");
        assert_eq!(motion_label(1), "step");
        assert_eq!(motion_label(-2), "step");
        assert_eq!(motion_label(3), "skip");
        assert_eq!(motion_label(-4), "skip");
        assert_eq!(motion_label(5), "leap");
    }

    #[test]
    fn rejects_single_chord() {
        let args = AnnotateArgs {
            chords: vec!["C4,E4,G4".to_string()],
            key: None,
            scale: None,
            no_crossings: false,
            format: OutputFormat::Text,
            output: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("at least 2 chords"));
    }

    #[test]
    fn rejects_mismatched_voice_count() {
        let args = AnnotateArgs {
            chords: vec!["C4,E4,G4".to_string(), "F4,A4".to_string()],
            key: None,
            scale: None,
            no_crossings: false,
            format: OutputFormat::Text,
            output: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("same number of voices"));
    }
}
