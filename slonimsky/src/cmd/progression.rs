use anyhow::{bail, Context, Result};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::note_collections::geometry::symmetry::voiceleading::{
    NoVoxCrossings, Voiceleading, VoiceleadingRule,
};
use music::note_collections::Voicing;
use std::fmt::Write as FmtWrite;

use super::input::{parse_input_to_pcs, pc_label};
use super::voice_leading::{metric_distance, parse_weights, Metric};

pub struct ProgressionArgs {
    pub chords: Vec<String>,
    pub no_crossings: bool,
    pub metric: String,
    pub weights: Option<String>,
    pub output: Option<String>,
    pub verbose: bool,
}

/// Map a Pc to its most natural Note spelling (prefer naturals/sharps).
fn pc_to_note(pc: Pc) -> Note {
    match u8::from(pc) {
        0 => Note::C,
        1 => Note::Cis,
        2 => Note::D,
        3 => Note::Ees,
        4 => Note::E,
        5 => Note::F,
        6 => Note::Fis,
        7 => Note::G,
        8 => Note::Aes,
        9 => Note::A,
        10 => Note::Bes,
        11 => Note::B,
        _ => unreachable!(),
    }
}

/// Place a set of PCs as a close-position voicing in octave 4.
/// PCs are placed ascending from octave 4, wrapping into octave 5
/// only to maintain ascending order.
fn initial_voicing(pcs: &[Pc]) -> Voicing {
    let mut pitches = Vec::with_capacity(pcs.len());
    let mut last_midi: u8 = 0;
    for (i, pc) in pcs.iter().enumerate() {
        let pc_val = u8::from(*pc);
        // Base MIDI for octave 4: C4 = 60
        let mut midi = 60 + pc_val;
        if i > 0 && midi <= last_midi {
            // Bump up by octaves until above the previous pitch
            while midi <= last_midi {
                midi += 12;
            }
        }
        last_midi = midi;
        let pitch = Pitch::from_midi_as(midi, pc_to_note(*pc))
            .expect("close voicing from C4 stays within MIDI range");
        pitches.push(pitch);
    }
    Voicing::new(pitches)
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

/// Parse a chord argument into a list of PCs.
/// Accepts comma-separated groups like "C,E,G" or space-separated like "C E G"
/// or integer PCs like "0,4,7".
fn parse_chord(s: &str) -> Result<Vec<Pc>> {
    // Treat commas as the primary delimiter for a single chord
    let tokens: Vec<String> = s.split(',').map(|t| t.trim().to_string()).collect();
    parse_input_to_pcs(&tokens)
}

/// Score a voice-leading by the chosen metric.
fn score_voiceleading(vl: &Voiceleading, metric: Metric, weights: Option<&[usize]>) -> usize {
    metric_distance(vl, metric, weights)
}

/// A step in the progression chain, for output formatting.
struct ProgressionStep {
    cost: usize,
    voicing: Voicing,
    paths: Vec<i8>,
}

pub fn run(args: ProgressionArgs) -> Result<()> {
    if args.chords.len() < 2 {
        bail!("progression requires at least 2 chords");
    }

    let metric = Metric::parse(&args.metric)?;

    // Parse all chords
    let mut chord_pcs: Vec<Vec<Pc>> = Vec::with_capacity(args.chords.len());
    for (i, s) in args.chords.iter().enumerate() {
        let pcs = parse_chord(s).with_context(|| format!("parsing chord {} ('{}')", i + 1, s))?;
        chord_pcs.push(pcs);
    }

    // Verify all chords have the same cardinality
    let voice_count = chord_pcs[0].len();
    for (i, pcs) in chord_pcs.iter().enumerate() {
        if pcs.len() != voice_count {
            bail!(
                "all chords must have the same number of notes. \
                 Chord 1 has {} notes, chord {} has {} notes.",
                voice_count,
                i + 1,
                pcs.len()
            );
        }
    }

    let weights = parse_weights(args.weights.as_deref(), metric, voice_count)?;

    if voice_count < 2 {
        bail!("chords must have at least 2 notes for voice-leading");
    }

    let rules: Option<Vec<Box<dyn VoiceleadingRule>>> = if args.no_crossings {
        Some(vec![Box::new(NoVoxCrossings)])
    } else {
        None
    };
    let rules_ref = rules.as_ref();

    // Build chord labels
    let chord_labels: Vec<String> = chord_pcs
        .iter()
        .map(|pcs| {
            pcs.iter()
                .map(|pc| pc_label(*pc).to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();

    // Compute the voice-leading chain
    let start_voicing = initial_voicing(&chord_pcs[0]);
    let mut current_voicing = start_voicing.clone();
    let mut steps: Vec<ProgressionStep> = Vec::new();
    let mut total_cost: usize = 0;

    for i in 1..chord_pcs.len() {
        let target_notes: Vec<Note> = chord_pcs[i].iter().map(|pc| pc_to_note(*pc)).collect();

        let results = Voiceleading::find_all(&current_voicing, &target_notes, rules_ref)
            .with_context(|| {
                format!(
                    "computing voice-leading from chord {} to chord {}",
                    i,
                    i + 1
                )
            })?;

        if results.is_empty() {
            bail!(
                "no valid voice-leading from {} to {}",
                chord_labels[i - 1],
                chord_labels[i]
            );
        }

        // Re-score by chosen metric and pick the best
        let mut scored: Vec<(usize, &Voiceleading)> = results
            .iter()
            .map(|(_l1, vl)| (score_voiceleading(vl, metric, weights.as_deref()), vl))
            .collect();
        scored.sort_by_key(|(s, _)| *s);

        let (cost, best) = scored[0];
        total_cost += cost;

        steps.push(ProgressionStep {
            cost,
            voicing: best.to.clone(),
            paths: best.paths.clone(),
        });

        current_voicing = best.to.clone();
    }

    // Determine output format from -o extension
    let output_is_midi = args
        .output
        .as_deref()
        .map(|p| p.ends_with(".mid") || p.ends_with(".midi"))
        .unwrap_or(false);

    #[cfg(feature = "midi")]
    if output_is_midi {
        let output_path = args.output.as_deref().unwrap();
        write_midi(output_path, &start_voicing, &steps, args.verbose)?;
        // Also print text summary to stdout
        print_text_summary(
            &chord_labels,
            voice_count,
            metric,
            args.no_crossings,
            weights.as_deref(),
            &start_voicing,
            &steps,
            total_cost,
            args.verbose,
        );
        return Ok(());
    }

    #[cfg(not(feature = "midi"))]
    if output_is_midi {
        bail!(
            "MIDI output requires the 'midi' feature. \
             Rebuild with: cargo build -p slonimsky --features midi"
        );
    }

    // Default: text output (stdout or file)
    let text = format_text(
        &chord_labels,
        voice_count,
        metric,
        args.no_crossings,
        weights.as_deref(),
        &start_voicing,
        &steps,
        total_cost,
        args.verbose,
    );

    if let Some(ref path) = args.output {
        std::fs::write(path, &text).with_context(|| format!("writing output to {}", path))?;
        if args.verbose {
            eprintln!("Wrote text output to {}", path);
        }
    } else {
        print!("{}", text);
    }

    if args.verbose {
        let avg = total_cost as f64 / (chord_pcs.len() - 1) as f64;
        eprintln!("Average cost per step: {:.1}", avg);
    }

    Ok(())
}

/// Format the progression as text.
fn format_text(
    chord_labels: &[String],
    voice_count: usize,
    metric: Metric,
    no_crossings: bool,
    weights: Option<&[usize]>,
    start_voicing: &Voicing,
    steps: &[ProgressionStep],
    total_cost: usize,
    verbose: bool,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Progression: {}", chord_labels.join(" → "));
    let _ = writeln!(out, "Voices: {}", voice_count);
    let _ = writeln!(out, "Metric: {}", metric.label());
    if let Some(weights) = weights {
        let _ = writeln!(
            out,
            "Weights: {} (lowest to highest voice)",
            weights
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    if no_crossings {
        let _ = writeln!(out, "Rule: no voice crossings");
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  Start: {} ({})",
        format_voicing(start_voicing),
        chord_labels[0]
    );

    for (i, step) in steps.iter().enumerate() {
        let from_v = if i == 0 {
            start_voicing.clone()
        } else {
            steps[i - 1].voicing.clone()
        };
        let _ = write!(
            out,
            "  Step {}: {} → {}  dist={}",
            i + 1,
            format_voicing(&from_v),
            format_voicing(&step.voicing),
            step.cost
        );
        if verbose {
            let l1: usize = step.paths.iter().map(|p| p.unsigned_abs() as usize).sum();
            let linf = step
                .paths
                .iter()
                .map(|p| p.unsigned_abs() as usize)
                .max()
                .unwrap_or(0);
            let _ = write!(out, "  L1={} L∞={}", l1, linf);
            if let Some(weights) = weights {
                let weighted: usize = step
                    .paths
                    .iter()
                    .zip(weights)
                    .map(|(path, weight)| path.unsigned_abs() as usize * weight)
                    .sum();
                let _ = write!(out, " weighted={}", weighted);
            }
            let _ = write!(out, "  paths=[{}]", format_paths(&step.paths));
        }
        let _ = writeln!(out, "  ({})", chord_labels[i + 1]);
    }

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Total voice-leading cost: {} ({}, {} steps)",
        total_cost,
        metric.label(),
        steps.len()
    );

    out
}

/// Print text summary (used when MIDI is the primary output).
#[cfg(feature = "midi")]
fn print_text_summary(
    chord_labels: &[String],
    voice_count: usize,
    metric: Metric,
    no_crossings: bool,
    weights: Option<&[usize]>,
    start_voicing: &Voicing,
    steps: &[ProgressionStep],
    total_cost: usize,
    verbose: bool,
) {
    let text = format_text(
        chord_labels,
        voice_count,
        metric,
        no_crossings,
        weights,
        start_voicing,
        steps,
        total_cost,
        verbose,
    );
    print!("{}", text);
}

/// Write the progression as a MIDI file. Each chord is played as a block chord
/// for 2 beats, with the voice-leading chain from start through all steps.
#[cfg(feature = "midi")]
fn write_midi(
    path: &str,
    start_voicing: &Voicing,
    steps: &[ProgressionStep],
    verbose: bool,
) -> Result<()> {
    use midly::{
        num::{u15, u24, u28, u4, u7},
        Format, Header, MetaMessage, MidiMessage as MM, Smf, Timing, TrackEvent, TrackEventKind,
    };

    let ppq: u16 = 480;
    let bpm: f64 = 100.0;
    let chord_duration = ppq as u32 * 2; // 2 quarter notes per chord
    let gap = ppq as u32 / 4; // Brief gap between chords

    let header = Header {
        format: Format::Parallel,
        timing: Timing::Metrical(u15::new(ppq)),
    };

    // Conductor track: tempo
    let usec_per_qn = (60_000_000.0 / bpm).round() as u32;
    let conductor = vec![
        TrackEvent {
            delta: u28::from(0u32),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(usec_per_qn))),
        },
        TrackEvent {
            delta: u28::from(0u32),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        },
    ];

    // Instrument track: chord-by-chord voice-leading
    let mut track: Vec<TrackEvent<'static>> = Vec::new();
    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"progression")),
    });

    // Collect all voicings: start + each step's target
    let mut voicings: Vec<&Voicing> = Vec::with_capacity(steps.len() + 1);
    voicings.push(start_voicing);
    for step in steps {
        voicings.push(&step.voicing);
    }

    for (chord_idx, voicing) in voicings.iter().enumerate() {
        // Note-on for all voices simultaneously
        for (vi, pitch) in voicing.iter().enumerate() {
            let midi_key = pitch.midi_note;
            let delta = if vi == 0 && chord_idx > 0 {
                gap // gap after previous chord's note-offs
            } else {
                0
            };
            track.push(TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MM::NoteOn {
                        key: u7::new(midi_key),
                        vel: u7::new(80),
                    },
                },
            });
        }

        // Note-off after chord_duration
        for (vi, pitch) in voicing.iter().enumerate() {
            let midi_key = pitch.midi_note;
            let delta = if vi == 0 { chord_duration } else { 0 };
            track.push(TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MM::NoteOff {
                        key: u7::new(midi_key),
                        vel: u7::new(64),
                    },
                },
            });
        }
    }

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header,
        tracks: vec![conductor, track],
    };

    let mut bytes = Vec::new();
    smf.write(&mut bytes)
        .map_err(|e| anyhow::anyhow!("MIDI write error: {}", e))?;
    std::fs::write(path, &bytes).with_context(|| format!("writing MIDI to {}", path))?;

    if verbose {
        eprintln!(
            "Wrote MIDI progression to {} ({} bytes, {} chords, {} BPM)",
            path,
            bytes.len(),
            steps.len() + 1,
            bpm
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chord_notes() {
        let pcs = parse_chord("C,E,G").unwrap();
        assert_eq!(pcs.len(), 3);
        assert_eq!(u8::from(pcs[0]), 0); // C
        assert_eq!(u8::from(pcs[1]), 4); // E
        assert_eq!(u8::from(pcs[2]), 7); // G
    }

    #[test]
    fn parse_chord_integers() {
        let pcs = parse_chord("0,4,7").unwrap();
        assert_eq!(pcs.len(), 3);
        assert_eq!(u8::from(pcs[0]), 0);
    }

    #[test]
    fn initial_voicing_ascending() {
        let pcs = parse_chord("C,E,G").unwrap();
        let v = initial_voicing(&pcs);
        assert_eq!(v.len(), 3);
        // Pitches should be ascending
        let pitches: Vec<_> = v.iter().collect();
        for i in 1..pitches.len() {
            assert!(pitches[i] > pitches[i - 1], "voicing should be ascending");
        }
    }

    #[test]
    fn initial_voicing_wraps_octave() {
        // B,D,F — B4 should come before D, so we get B4 D5 F5
        let pcs = parse_chord("B,D,F").unwrap();
        let v = initial_voicing(&pcs);
        assert_eq!(v.len(), 3);
        let pitches: Vec<_> = v.iter().collect();
        for i in 1..pitches.len() {
            assert!(pitches[i] > pitches[i - 1]);
        }
    }

    #[test]
    fn progression_two_chords() {
        let args = ProgressionArgs {
            chords: vec!["C,E,G".to_string(), "F,A,C".to_string()],
            no_crossings: false,
            metric: "l1".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn progression_three_chords() {
        let args = ProgressionArgs {
            chords: vec![
                "C,E,G".to_string(),
                "F,A,C".to_string(),
                "G,B,D".to_string(),
            ],
            no_crossings: true,
            metric: "l1".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn progression_linf_metric() {
        let args = ProgressionArgs {
            chords: vec!["C,E,G".to_string(), "F,A,C".to_string()],
            no_crossings: false,
            metric: "linf".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn progression_rejects_single_chord() {
        let args = ProgressionArgs {
            chords: vec!["C,E,G".to_string()],
            no_crossings: false,
            metric: "l1".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("at least 2 chords"));
    }

    #[test]
    fn progression_rejects_mismatched_cardinality() {
        let args = ProgressionArgs {
            chords: vec!["C,E,G".to_string(), "D,F,A,C".to_string()],
            no_crossings: false,
            metric: "l1".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("same number of notes"));
    }

    #[test]
    fn progression_rejects_bad_metric() {
        let args = ProgressionArgs {
            chords: vec!["C,E,G".to_string(), "F,A,C".to_string()],
            no_crossings: false,
            metric: "euclidean".to_string(),
            weights: None,
            output: None,
            verbose: false,
        };
        let result = run(args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unknown metric"));
    }

    #[test]
    #[cfg(feature = "midi")]
    fn progression_midi_output() {
        let tmp = tempfile::TempDir::new().unwrap();
        let midi_path = tmp.path().join("test_prog.mid");
        let args = ProgressionArgs {
            chords: vec![
                "C,E,G".to_string(),
                "F,A,C".to_string(),
                "G,B,D".to_string(),
            ],
            no_crossings: false,
            metric: "l1".to_string(),
            weights: None,
            output: Some(midi_path.to_string_lossy().to_string()),
            verbose: false,
        };
        assert!(run(args).is_ok());
        let bytes = std::fs::read(&midi_path).expect("MIDI file should exist");
        // Check MThd magic
        assert_eq!(&bytes[0..4], b"MThd", "MIDI file should start with MThd");
        // Should have 2 tracks (conductor + instrument)
        assert!(
            bytes.len() > 30,
            "MIDI file should be non-trivial (got {} bytes)",
            bytes.len()
        );
        // Check MTrk marker exists at least twice
        let mtrk_count = bytes.windows(4).filter(|w| w == b"MTrk").count();
        assert_eq!(mtrk_count, 2, "should have 2 tracks (conductor + notes)");
    }

    #[test]
    fn progression_total_cost_positive() {
        // C→F→G — each step should have some cost
        let pcs1 = parse_chord("C,E,G").unwrap();
        let v1 = initial_voicing(&pcs1);
        let target = vec![
            pc_to_note(Pc::from(5)),
            pc_to_note(Pc::from(9)),
            pc_to_note(Pc::from(0)),
        ];
        let results = Voiceleading::find_all(&v1, &target, None).unwrap();
        assert!(!results.is_empty());
        // Best voice-leading should have cost > 0 (different chords)
        assert!(
            results[0].0 > 0,
            "different chords should have positive cost"
        );
    }
}
