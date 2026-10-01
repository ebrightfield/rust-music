use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{Context, Result};
use music::fretboard::fretboard_shape::chord_shape_search::{
    find_chord_shapes, ChordShapeSearchResult,
};
use music::fretboard::fretted_note::FrettedNote;
use music::fretboard::FretboardShape;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::svg::FretboardBuilder;
use serde_json::{json, Value};

use super::input::{parse_input_to_pcs, pc_label, resolve_theme};
use super::output::{write_output, OutputFormat};
use super::tuning::TuningSpec;

const CLASSIFICATIONS: &[&str] = &[
    "playable",
    "wide",
    "nontransposable",
    "high-fret",
    "unplayable",
];

pub struct ChordDictionaryArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub format: Option<String>,
    pub theme: Option<String>,
    pub tuning: String,
    pub classifications: Vec<String>,
    pub voicing: Option<String>,
    pub family: Option<String>,
    pub bass: Option<String>,
    pub open_strings: String,
    pub min_fret: Option<u8>,
    pub max_fret: Option<u8>,
    pub max_span: Option<u8>,
    pub max_results: Option<usize>,
    pub verbose: bool,
}

#[derive(Clone)]
struct ShapeEntry<'a> {
    classification: &'static str,
    voicing: Vec<Pitch>,
    shape: FretboardShape<'a>,
}

fn pc_to_note(pc: Pc) -> Note {
    pc.notes()[0]
}

fn parse_classifications(raw: &[String]) -> Result<HashSet<&'static str>> {
    let mut selected = HashSet::new();
    if raw.is_empty() {
        selected.extend(CLASSIFICATIONS.iter().copied());
        return Ok(selected);
    }
    for token in raw {
        for value in token.split(',') {
            let value = value.trim().to_ascii_lowercase();
            if value == "all" {
                selected.extend(CLASSIFICATIONS.iter().copied());
            } else if let Some(name) = CLASSIFICATIONS.iter().find(|name| **name == value) {
                selected.insert(*name);
            } else {
                anyhow::bail!(
                    "unknown classification '{value}' (options: all, {})",
                    CLASSIFICATIONS.join(", ")
                );
            }
        }
    }
    Ok(selected)
}

fn parse_note_sequence(raw: &str, option: &str) -> Result<Vec<Note>> {
    let notes = raw
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            Note::from_str(part).with_context(|| format!("invalid note '{part}' for {option}"))
        })
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(!notes.is_empty(), "{option} requires at least one note");
    Ok(notes)
}

fn parse_pitch(raw: &str) -> Result<Pitch> {
    let octave_start = raw
        .char_indices()
        .find_map(|(index, ch)| (ch.is_ascii_digit() || ch == '-').then_some(index))
        .with_context(|| format!("pitch '{raw}' needs an octave, for example C4 or Bb3"))?;
    let (note, octave) = raw.split_at(octave_start);
    let note = Note::from_str(note).with_context(|| format!("invalid note in pitch '{raw}'"))?;
    let octave = octave
        .parse::<i8>()
        .with_context(|| format!("invalid octave in pitch '{raw}'"))?;
    Pitch::try_new(note, octave).with_context(|| format!("pitch '{raw}' is outside MIDI range"))
}

fn parse_voicing(raw: Option<&str>) -> Result<Option<Vec<Pitch>>> {
    raw.map(|raw| {
        let pitches = raw
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(parse_pitch)
            .collect::<Result<Vec<_>>>()?;
        anyhow::ensure!(!pitches.is_empty(), "--voicing requires at least one pitch");
        Ok(pitches)
    })
    .transpose()
}

fn collect_map<'a>(
    entries: &mut Vec<ShapeEntry<'a>>,
    classification: &'static str,
    map: &std::collections::HashMap<
        music::note_collections::voicing::Voicing,
        Vec<FretboardShape<'a>>,
    >,
) {
    for shapes in map.values() {
        for shape in shapes {
            let voicing = shape
                .iter()
                .filter_map(|note| note.pitch())
                .collect::<Vec<_>>();
            entries.push(ShapeEntry {
                classification,
                voicing,
                shape: shape.clone(),
            });
        }
    }
}

fn collect_entries<'a>(
    results: &ChordShapeSearchResult<'a>,
    selected: &HashSet<&str>,
) -> Vec<ShapeEntry<'a>> {
    let mut entries = Vec::new();
    if selected.contains("playable") {
        collect_map(&mut entries, "playable", &results.playable);
    }
    if selected.contains("wide") {
        collect_map(&mut entries, "wide", &results.wide_intervals);
    }
    if selected.contains("nontransposable") {
        collect_map(&mut entries, "nontransposable", &results.nontransposable);
    }
    if selected.contains("high-fret") {
        collect_map(&mut entries, "high-fret", &results.all_above_12th_fret);
    }
    if selected.contains("unplayable") {
        collect_map(&mut entries, "unplayable", &results.unplayable);
    }
    entries
}

fn pitch_names(voicing: &[Pitch]) -> Vec<String> {
    voicing.iter().map(ToString::to_string).collect()
}

fn family_names(voicing: &[Pitch]) -> Vec<String> {
    voicing.iter().map(|pitch| pitch.note.to_string()).collect()
}

fn classification_rank(name: &str) -> usize {
    CLASSIFICATIONS
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap()
}

fn text_output(
    label: &str,
    tuning: &str,
    max_span: Option<u8>,
    entries: &[ShapeEntry<'_>],
) -> String {
    let span = max_span.map_or_else(|| "unlimited".to_string(), |span| span.to_string());
    let mut out =
        format!("Chord dictionary: [{label}]\nTuning: {tuning}, Max span: {span} frets\n\n");
    if entries.is_empty() {
        out.push_str("(no matching shapes found)\n");
    }
    for (index, entry) in entries.iter().enumerate() {
        let (low, high) = entry.shape.span();
        out.push_str(&format!(
            "  {:>3}. {}  (frets {}-{}, classification {}, bass {}, family {}, voicing {})\n",
            index + 1,
            entry.shape,
            low,
            high,
            entry.classification,
            entry
                .voicing
                .first()
                .map(|pitch| pitch.note.to_string())
                .unwrap_or_default(),
            family_names(&entry.voicing).join(","),
            pitch_names(&entry.voicing).join(","),
        ));
    }
    out.push_str(&format!("\nTotal: {} shapes\n", entries.len()));
    out
}

fn json_output(label: &str, tuning: &str, entries: &[ShapeEntry<'_>]) -> Result<Vec<u8>> {
    let shapes: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let (min_fret, max_fret) = entry.shape.span();
            json!({
                "classification": entry.classification,
                "shape": entry.shape.to_string(),
                "frets": entry.shape.iter().map(|note| match note {
                    FrettedNote::Muted { .. } => Value::Null,
                    FrettedNote::Sounded(note) => json!(note.fret),
                }).collect::<Vec<_>>(),
                "min_fret": min_fret,
                "max_fret": max_fret,
                "span": max_fret.saturating_sub(min_fret),
                "contains_open_strings": entry.shape.contains_open_strings(),
                "bass": entry.voicing.first().map(|pitch| pitch.note.to_string()),
                "family": family_names(&entry.voicing),
                "voicing": pitch_names(&entry.voicing),
            })
        })
        .collect();
    Ok(serde_json::to_vec_pretty(&json!({
        "input": label,
        "tuning": tuning,
        "count": entries.len(),
        "shapes": shapes,
    }))?)
}

fn svg_output(entries: &[ShapeEntry<'_>], theme: music::svg::SvgTheme) -> Result<Vec<u8>> {
    anyhow::ensure!(!entries.is_empty(), "no matching shapes found");
    let columns = 4u32;
    let tile_width = 200u32;
    let tile_height = 200u32;
    let rows = (entries.len() as u32).div_ceil(columns);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        tile_width * columns,
        tile_height * rows
    );
    for (index, entry) in entries.iter().enumerate() {
        let inner = FretboardBuilder::new()
            .from_shape(&entry.shape)
            .theme(theme.clone())
            .title(format!("{}: {}", entry.classification, entry.shape))
            .build();
        let content = inner
            .find('>')
            .and_then(|start| inner.rfind("</svg>").map(|end| &inner[start + 1..end]))
            .context("generated fretboard SVG had an invalid envelope")?;
        svg.push_str(&format!(
            "<g transform=\"translate({}, {})\">{}</g>\n",
            index as u32 % columns * tile_width,
            index as u32 / columns * tile_height,
            content
        ));
    }
    svg.push_str("</svg>\n");
    Ok(svg.into_bytes())
}

pub fn run(args: ChordDictionaryArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let tuning = TuningSpec::parse(&args.tuning)?;
    let selected = parse_classifications(&args.classifications)?;
    let voicing_filter = parse_voicing(args.voicing.as_deref())?;
    let family_filter = args
        .family
        .as_deref()
        .map(|raw| parse_note_sequence(raw, "--family"))
        .transpose()?;
    let bass_filter = args
        .bass
        .as_deref()
        .map(|raw| Note::from_str(raw).with_context(|| format!("invalid note '{raw}' for --bass")))
        .transpose()?;
    let open_filter = match args.open_strings.as_str() {
        "any" => None,
        "required" => Some(true),
        "excluded" => Some(false),
        other => anyhow::bail!(
            "unknown --open-strings value '{other}' (options: any, required, excluded)"
        ),
    };
    if let (Some(min), Some(max)) = (args.min_fret, args.max_fret) {
        anyhow::ensure!(min <= max, "--min-fret cannot exceed --max-fret");
    }

    let label = pcs
        .iter()
        .map(|pc| pc_label(*pc))
        .collect::<Vec<_>>()
        .join(", ");
    let notes = pcs.iter().map(|pc| pc_to_note(*pc)).collect::<Vec<_>>();
    let results = find_chord_shapes(&notes, &tuning.fretboard)
        .with_context(|| format!("chord shape search failed for [{label}]"))?;
    let mut entries = collect_entries(&results, &selected);
    entries.retain(|entry| {
        let (low, high) = entry.shape.span();
        args.min_fret.is_none_or(|min| low >= min)
            && args.max_fret.is_none_or(|max| high <= max)
            && args
                .max_span
                .is_none_or(|span| high.saturating_sub(low) <= span)
            && open_filter.is_none_or(|required| entry.shape.contains_open_strings() == required)
            && bass_filter.is_none_or(|bass| {
                entry
                    .voicing
                    .first()
                    .is_some_and(|pitch| pitch.note == bass)
            })
            && family_filter.as_ref().is_none_or(|family| {
                entry.voicing.len() == family.len()
                    && entry
                        .voicing
                        .iter()
                        .zip(family)
                        .all(|(pitch, note)| Pc::from(&pitch.note) == Pc::from(note))
            })
            && voicing_filter
                .as_ref()
                .is_none_or(|voicing| entry.voicing == *voicing)
    });
    entries.sort_by(|left, right| {
        let left_span = left.shape.span();
        let right_span = right.shape.span();
        (
            classification_rank(left.classification),
            left_span.0,
            left_span.1,
            left.shape.to_string(),
        )
            .cmp(&(
                classification_rank(right.classification),
                right_span.0,
                right_span.1,
                right.shape.to_string(),
            ))
    });
    let mut seen = HashSet::new();
    entries.retain(|entry| seen.insert((entry.classification, entry.shape.to_string())));
    if let Some(limit) = args.max_results {
        entries.truncate(limit);
    }

    let format = OutputFormat::resolve(
        args.format.as_deref(),
        args.output.as_deref(),
        OutputFormat::Text,
    )?;
    anyhow::ensure!(
        matches!(
            format,
            OutputFormat::Text | OutputFormat::Json | OutputFormat::Svg
        ),
        "chord-dictionary supports text, json, or svg output"
    );
    let bytes = match format {
        OutputFormat::Text => {
            text_output(&label, &args.tuning, args.max_span, &entries).into_bytes()
        }
        OutputFormat::Json => json_output(&label, &args.tuning, &entries)?,
        OutputFormat::Svg => svg_output(&entries, resolve_theme(args.theme.as_deref())?)?,
        _ => unreachable!(),
    };
    write_output(format, args.output.as_deref(), &bytes)?;
    if args.verbose {
        eprintln!(
            "chord-dictionary: input=[{label}], tuning={}, found {} matching shapes",
            args.tuning,
            entries.len()
        );
    }
    Ok(())
}
