#[cfg(feature = "midi")]
use crate::cmd::midi_output as cmd_midi;
use anyhow::{bail, Context, Result};
use music::melody::{
    ChordProgression, Direction, IntervalPattern, MelodicEvent, MelodicSequencer,
    MelodicSequencerConfig, PitchBounds, TimedChord, TurnaroundMode,
};
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note_collections::NoteSet;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::str::FromStr;

pub struct SequenceArgs {
    pub harmony: Vec<String>,
    pub chord_durations: String,
    pub pattern: String,
    pub master_step: i8,
    pub rhythm: String,
    pub start: String,
    pub low: String,
    pub high: String,
    pub direction: String,
    pub turnaround: String,
    pub length: usize,
    pub format: Option<String>,
    pub output: Option<String>,
    pub verbose: bool,
    pub bpm: f32,
    pub ppq: u16,
    pub soundfont: Option<String>,
    pub offline: bool,
    pub sample_rate: u32,
    pub tail: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    #[cfg(feature = "midi")]
    Midi(cmd_midi::MidiOutputFormat),
}

fn parse_pitch(value: &str) -> Result<Pitch> {
    let value = value.trim();
    let octave_start = value
        .char_indices()
        .find(|(index, ch)| {
            ch.is_ascii_digit()
                || (*ch == '-'
                    && value[*index + ch.len_utf8()..].starts_with(|c: char| c.is_ascii_digit()))
        })
        .map(|(index, _)| index)
        .ok_or_else(|| anyhow::anyhow!("pitch '{value}' needs an octave, for example C4 or Bb3"))?;
    let (note, octave) = value.split_at(octave_start);
    let note = Note::from_str(note).with_context(|| format!("invalid note in pitch '{value}'"))?;
    let octave = octave
        .parse::<i8>()
        .with_context(|| format!("invalid octave in pitch '{value}'"))?;
    Pitch::try_new(note, octave).with_context(|| format!("pitch '{value}' is outside MIDI range"))
}

fn parse_duration(value: &str) -> Result<Duration> {
    let value = value.trim().to_ascii_lowercase();
    let dot_count = value.chars().rev().take_while(|ch| *ch == '.').count();
    if dot_count > 5 {
        bail!("duration '{value}' has more than five dots");
    }
    let base = &value[..value.len() - dot_count];
    let kind = match base {
        "breve" | "2" => DurationKind::Breve,
        "whole" | "w" | "1" => DurationKind::Whole,
        "half" | "h" => DurationKind::Half,
        "quarter" | "q" | "4" => DurationKind::Qtr,
        "eighth" | "e" | "8" => DurationKind::Eighth,
        "sixteenth" | "s" | "16" => DurationKind::Sixteenth,
        "32" => DurationKind::ThirtySecond,
        "64" => DurationKind::SixtyFourth,
        "128" => DurationKind::OneTwentyEighth,
        _ => bail!(
            "invalid duration '{value}' (expected breve, w, h, q, 8, 16, 32, 64, or 128, optionally dotted)"
        ),
    };
    Ok(Duration::new(kind, dot_count as u8))
}

fn duration_label(duration: Duration) -> String {
    let base = match duration.kind() {
        DurationKind::Breve => "breve",
        DurationKind::Whole => "w",
        DurationKind::Half => "h",
        DurationKind::Qtr => "q",
        DurationKind::Eighth => "8",
        DurationKind::Sixteenth => "16",
        DurationKind::ThirtySecond => "32",
        DurationKind::SixtyFourth => "64",
        DurationKind::OneTwentyEighth => "128",
    };
    format!("{base}{}", ".".repeat(duration.num_dots() as usize))
}

fn parse_duration_list(value: &str, option: &str) -> Result<Vec<Duration>> {
    let values: Vec<_> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if values.is_empty() {
        bail!("{option} must contain at least one duration");
    }
    values
        .into_iter()
        .map(|part| parse_duration(part).with_context(|| format!("parsing {option}")))
        .collect()
}

fn parse_pattern(value: &str) -> Result<Vec<Vec<i8>>> {
    let mut levels = Vec::new();
    for (index, level) in value.split('/').enumerate() {
        let intervals = level
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| {
                part.parse::<i8>().with_context(|| {
                    format!("invalid interval '{part}' in pattern level {}", index + 1)
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if intervals.is_empty() {
            bail!("pattern level {} is empty", index + 1);
        }
        levels.push(intervals);
    }
    if levels.is_empty() {
        bail!("--pattern must contain at least one interval");
    }
    Ok(levels)
}

fn parse_harmony(value: &str) -> Result<NoteSet> {
    let notes = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| Note::from_str(part).with_context(|| format!("invalid harmony note '{part}'")))
        .collect::<Result<Vec<_>>>()?;
    if notes.is_empty() {
        bail!("harmony '{value}' contains no notes");
    }
    Ok(NoteSet::new(notes))
}

fn parse_direction(value: &str) -> Result<Direction> {
    match value {
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        _ => bail!("invalid direction '{value}' (expected up or down)"),
    }
}

fn parse_turnaround(value: &str) -> Result<TurnaroundMode> {
    match value {
        "reflect" => Ok(TurnaroundMode::Reflect),
        "ricochet" => Ok(TurnaroundMode::Ricochet),
        "start-over" => Ok(TurnaroundMode::StartOver),
        "wrap" => Ok(TurnaroundMode::Wrap),
        "stop" => Ok(TurnaroundMode::Stop),
        _ => bail!(
            "invalid turnaround '{value}' (expected reflect, ricochet, start-over, wrap, or stop)"
        ),
    }
}

fn output_format(explicit: Option<&str>, output: Option<&str>) -> Result<OutputFormat> {
    let value = explicit.map(str::to_owned).or_else(|| {
        output.and_then(|path| {
            std::path::Path::new(path)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_owned)
        })
    });
    match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
        None | Some("text" | "txt") => Ok(OutputFormat::Text),
        Some("json") => Ok(OutputFormat::Json),
        #[cfg(feature = "midi")]
        Some(value) if cmd_midi::MidiOutputFormat::parse(value).is_some() => Ok(
            OutputFormat::Midi(cmd_midi::MidiOutputFormat::parse(value).unwrap()),
        ),
        #[cfg(not(feature = "midi"))]
        Some("midi" | "mid" | "wav" | "wave" | "play") => {
            bail!("MIDI, WAV, and playback output require --features midi")
        }
        Some(other) => bail!("invalid format '{other}' (expected text, json, midi, wav, or play)"),
    }
}

fn text_output(events: &[MelodicEvent]) -> String {
    let mut output = String::from("index\tpitch\tmidi\tduration\tticks\ttied\n");
    for (index, event) in events.iter().enumerate() {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}",
            index + 1,
            event.pitch,
            event.pitch.midi_note,
            duration_label(event.duration),
            event.duration.ticks(),
            event.tied
        )
        .expect("writing to String cannot fail");
    }
    output
}

fn json_output(events: &[MelodicEvent]) -> Result<String> {
    let values: Vec<_> = events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            serde_json::json!({
                "index": index + 1,
                "pitch": event.pitch.to_string(),
                "midi": event.pitch.midi_note,
                "duration": duration_label(event.duration),
                "ticks": event.duration.ticks(),
                "tied": event.tied,
            })
        })
        .collect();
    Ok(format!("{}\n", serde_json::to_string_pretty(&values)?))
}

pub fn run(args: SequenceArgs) -> Result<()> {
    #[cfg(not(feature = "midi"))]
    let _ = (
        args.bpm,
        args.ppq,
        args.soundfont.as_deref(),
        args.offline,
        args.sample_rate,
        args.tail,
    );
    if args.length == 0 {
        bail!("--length must be at least 1");
    }

    let start = parse_pitch(&args.start)?;
    let low = parse_pitch(&args.low)?;
    let high = parse_pitch(&args.high)?;
    let bounds = PitchBounds::try_new(low, high).context("invalid pitch bounds")?;
    if !bounds.contains(&start) {
        bail!("starting pitch {start} is outside bounds {low}..{high}");
    }

    let chord_durations = parse_duration_list(&args.chord_durations, "--chord-durations")?;
    if chord_durations.len() != 1 && chord_durations.len() != args.harmony.len() {
        bail!(
            "--chord-durations requires one value or one per harmony (got {} for {})",
            chord_durations.len(),
            args.harmony.len()
        );
    }
    let chords = args
        .harmony
        .iter()
        .enumerate()
        .map(|(index, harmony)| {
            let duration = if chord_durations.len() == 1 {
                chord_durations[0]
            } else {
                chord_durations[index]
            };
            Ok(TimedChord::new(
                parse_harmony(harmony).with_context(|| format!("parsing harmony {}", index + 1))?,
                duration,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let config = MelodicSequencerConfig {
        chord_progression: ChordProgression::new(chords),
        interval_pattern: IntervalPattern::new(parse_pattern(&args.pattern)?, args.master_step),
        rhythm_pattern: parse_duration_list(&args.rhythm, "--rhythm")?,
        bounds,
        starting_pitch: start,
        direction: parse_direction(&args.direction)?,
        turnaround_mode: parse_turnaround(&args.turnaround)?,
        max_length: args.length,
    };
    let events = MelodicSequencer::new(config)
        .generate()
        .context("generating melodic sequence")?;
    let format = output_format(args.format.as_deref(), args.output.as_deref())?;
    match format {
        OutputFormat::Text | OutputFormat::Json => {
            let rendered = match format {
                OutputFormat::Text => text_output(&events),
                OutputFormat::Json => json_output(&events)?,
                #[cfg(feature = "midi")]
                OutputFormat::Midi(_) => unreachable!(),
            };
            if let Some(path) = args.output {
                fs::write(&path, rendered).with_context(|| format!("writing {path}"))?;
                if args.verbose {
                    eprintln!("Wrote {} events to {path}", events.len());
                }
            } else {
                print!("{rendered}");
            }
        }
        #[cfg(feature = "midi")]
        OutputFormat::Midi(format) => {
            let output = args.output.as_deref();
            cmd_midi::render_melody(
                &events,
                cmd_midi::MidiOutputArgs {
                    format,
                    output,
                    bpm: args.bpm,
                    ppq: args.ppq,
                    soundfont: args.soundfont.as_deref(),
                    offline: args.offline,
                    sample_rate: args.sample_rate,
                    tail: args.tail,
                },
            )?;
            if args.verbose {
                match output {
                    Some(path) => eprintln!("Wrote {} events to {path}", events.len()),
                    None => eprintln!("Played {} events", events.len()),
                }
            }
        }
    }
    Ok(())
}
