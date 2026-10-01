use anyhow::{bail, Context, Result};
use music::note::{Note, Pitch};
use music::note_collections::geometry::contour::ContourSequence;
use serde_json::{json, Value};

use super::output::{write_output, OutputFormat};

pub struct ContourArgs {
    pub input: Vec<String>,
    pub compare: Vec<String>,
    pub transform: String,
    pub output: Option<String>,
    pub format: Option<String>,
    pub verbose: bool,
}

fn parse_pitch(token: &str) -> Result<Pitch> {
    let token = token.trim();
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        let midi = token
            .parse::<u8>()
            .with_context(|| format!("MIDI pitch '{token}' must be between 0 and 127"))?;
        return Pitch::from_midi(midi)
            .with_context(|| format!("MIDI pitch '{token}' must be between 0 and 127"));
    }

    let octave_start = token
        .char_indices()
        .find_map(|(index, ch)| (ch.is_ascii_digit() || ch == '-').then_some(index))
        .with_context(|| format!("pitch '{token}' requires an octave (for example, C4)"))?;
    let note = token[..octave_start]
        .parse::<Note>()
        .with_context(|| format!("invalid note in pitch '{token}'"))?;
    let octave = token[octave_start..]
        .parse::<i8>()
        .with_context(|| format!("invalid octave in pitch '{token}'"))?;
    Pitch::try_new(note, octave).with_context(|| format!("pitch '{token}' is outside MIDI range"))
}

fn parse_pitches(values: &[String], label: &str) -> Result<Vec<Pitch>> {
    let pitches = values
        .iter()
        .flat_map(|value| value.split(|ch: char| ch == ',' || ch.is_whitespace()))
        .filter(|token| !token.is_empty())
        .map(parse_pitch)
        .collect::<Result<Vec<_>>>()?;
    if pitches.len() < 2 {
        bail!("{label} requires at least two pitches");
    }
    Ok(pitches)
}

fn transform(contour: &ContourSequence, name: &str) -> Result<ContourSequence> {
    match name {
        "original" | "identity" => Ok(contour.clone()),
        "retrograde" => Ok(contour.retrograde()),
        "inversion" => Ok(contour.inversion()),
        "retrograde-inversion" | "ri" => Ok(contour.retrograde_inversion()),
        _ => bail!(
            "unsupported transformation '{name}' (options: original, retrograde, inversion, retrograde-inversion)"
        ),
    }
}

fn direction_names(contour: &ContourSequence) -> Vec<&'static str> {
    contour
        .to_numeric_vec()
        .into_iter()
        .map(|movement| match movement {
            1 => "ascending",
            -1 => "descending",
            _ => "repeat",
        })
        .collect()
}

fn intervals(pitches: &[Pitch]) -> Vec<i16> {
    pitches
        .windows(2)
        .map(|pair| pair[1].midi_note as i16 - pair[0].midi_note as i16)
        .collect()
}

fn interval_changes(intervals: &[i16]) -> Vec<&'static str> {
    intervals
        .windows(2)
        .map(|pair| match pair[1].abs().cmp(&pair[0].abs()) {
            std::cmp::Ordering::Greater => "expanding",
            std::cmp::Ordering::Less => "contracting",
            std::cmp::Ordering::Equal => "unchanged",
        })
        .collect()
}

fn contour_json(pitches: &[Pitch], contour: &ContourSequence) -> Value {
    let melodic_intervals = intervals(pitches);
    json!({
        "pitches": pitches.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "midi": pitches.iter().map(|pitch| pitch.midi_note).collect::<Vec<_>>(),
        "directions": direction_names(contour),
        "numeric": contour.to_numeric_vec(),
        "intervals": melodic_intervals,
        "interval_changes": interval_changes(&melodic_intervals),
    })
}

fn render_text(
    input: &[Pitch],
    selected_name: &str,
    selected: &ContourSequence,
    comparison: Option<(&[Pitch], &ContourSequence)>,
) -> String {
    let source = ContourSequence::from_pitches(input);
    let melodic_intervals = intervals(input);
    let mut text = format!(
        "Pitches: {}\nDirections: {}\nNumeric: {}\nIntervals: {}\nInterval changes: {}\nTransformation: {}\nTransformed directions: {}\nTransformed numeric: {}\nRetrograde: {}\nInversion: {}\nRetrograde-inversion: {}\n",
        input.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
        direction_names(&source).join(" "),
        source.to_numeric_vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
        melodic_intervals.iter().map(|value| format!("{value:+}")).collect::<Vec<_>>().join(" "),
        interval_changes(&melodic_intervals).join(" "),
        selected_name,
        direction_names(selected).join(" "),
        selected.to_numeric_vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
        source.retrograde().to_numeric_vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
        source.inversion().to_numeric_vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
        source.retrograde_inversion().to_numeric_vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
    );
    if let Some((other_pitches, other)) = comparison {
        text.push_str(&format!(
            "Comparison: {}\nSimilarity: {:.6}\nMaximum transformed similarity: {:.6}\nEquivalent under transformation: {}\n",
            other_pitches.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "),
            source.similarity(other),
            source.max_similarity(other),
            source.is_equivalent(other),
        ));
    }
    text
}

pub fn run(args: ContourArgs) -> Result<()> {
    let pitches = parse_pitches(&args.input, "contour")?;
    let contour = ContourSequence::from_pitches(&pitches);
    let selected = transform(&contour, &args.transform)?;
    let comparison = if args.compare.is_empty() {
        None
    } else {
        let pitches = parse_pitches(&args.compare, "--compare")?;
        let contour = ContourSequence::from_pitches(&pitches);
        Some((pitches, contour))
    };
    let format = OutputFormat::resolve(
        args.format.as_deref(),
        args.output.as_deref(),
        OutputFormat::Text,
    )?;
    anyhow::ensure!(
        matches!(format, OutputFormat::Text | OutputFormat::Json),
        "contour supports text and JSON output"
    );

    let rendered = match format {
        OutputFormat::Text => render_text(
            &pitches,
            &args.transform,
            &selected,
            comparison
                .as_ref()
                .map(|(pitches, contour)| (pitches.as_slice(), contour)),
        ),
        OutputFormat::Json => {
            let mut value = json!({
                "source": contour_json(&pitches, &contour),
                "transformation": args.transform,
                "transformed": {
                    "directions": direction_names(&selected),
                    "numeric": selected.to_numeric_vec(),
                },
                "transformations": {
                    "retrograde": contour.retrograde().to_numeric_vec(),
                    "inversion": contour.inversion().to_numeric_vec(),
                    "retrograde_inversion": contour.retrograde_inversion().to_numeric_vec(),
                },
            });
            if let Some((other_pitches, other)) = &comparison {
                value["comparison"] = json!({
                    "contour": contour_json(other_pitches, other),
                    "similarity": contour.similarity(other),
                    "max_similarity": contour.max_similarity(other),
                    "equivalent": contour.is_equivalent(other),
                });
            }
            serde_json::to_string_pretty(&value)? + "\n"
        }
        _ => unreachable!(),
    };
    write_output(format, args.output.as_deref(), rendered.as_bytes())?;
    if args.verbose {
        eprintln!(
            "contour: {} pitches, {} movements",
            pitches.len(),
            contour.len()
        );
    }
    Ok(())
}
