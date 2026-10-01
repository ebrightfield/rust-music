use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;

use anyhow::{Context, Result};
use music::fretboard::fretboard_shape::melodic_shape_search::{
    find_all_scale_shapes, find_open_scale_shape, n_note_per_string_shape, MelodicFretboardShape,
    ScaleShapeSearchResult,
};
use music::fretboard::Fretboard;
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::svg::fretboard::{FretPosition, FretboardBuilder, Orientation};
use music::svg::SvgTheme;
use music_engraver::render::png::{dpi_to_scale, svg_to_png};
use serde_json::{json, Value};

use super::input::{parse_pc, resolve_theme};
use super::tuning::TuningSpec;

const CATEGORY_NAMES: &[&str] = &[
    "open",
    "simple",
    "2nps",
    "2-3nps",
    "3-2nps",
    "3nps",
    "exhaustive",
];

pub struct MelodicShapesArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub format: Option<String>,
    pub categories: Vec<String>,
    pub starting_notes: Vec<String>,
    pub tuning: String,
    pub root: Option<String>,
    pub max_score: Option<usize>,
    pub max_span: Option<u8>,
    pub limit: Option<usize>,
    pub sort: String,
    pub deduplicate: bool,
    pub orientation: String,
    pub columns: usize,
    pub tile_width: u32,
    pub tile_height: u32,
    pub gap: u32,
    pub start_fret: Option<u8>,
    pub num_frets: Option<u8>,
    pub fret_padding: u8,
    pub no_titles: bool,
    pub no_root_markers: bool,
    pub no_fret_numbers: bool,
    pub no_string_names: bool,
    pub dpi: f32,
    pub theme: Option<String>,
    pub title: Option<String>,
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Text,
    Json,
    Svg,
    Png,
    Pdf,
}

impl OutputFormat {
    fn resolve(explicit: Option<&str>, output: Option<&str>) -> Result<Self> {
        let raw = explicit.or_else(|| {
            output.and_then(|path| Path::new(path).extension().and_then(|ext| ext.to_str()))
        });
        match raw.unwrap_or("text").to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "svg" => Ok(Self::Svg),
            "png" => Ok(Self::Png),
            "pdf" => Ok(Self::Pdf),
            other => anyhow::bail!(
                "unsupported melodic-shapes output format '{other}' (options: text, json, svg, png, pdf)"
            ),
        }
    }

    fn is_binary(self) -> bool {
        matches!(self, Self::Png | Self::Pdf)
    }
}

#[derive(Clone)]
struct ShapeEntry<'a> {
    category: &'static str,
    starting_note: Note,
    rank: usize,
    shape: MelodicFretboardShape<'a>,
}

fn parse_notes(tokens: &[String]) -> Result<Vec<Note>> {
    let mut notes = Vec::new();
    for token in tokens {
        for part in token.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let note = Note::from_str(part)
                .or_else(|_| parse_pc(part).map(|pc| pc.notes()[0]).ok_or(()))
                .map_err(|_| anyhow::anyhow!("invalid note or pitch class: '{part}'"))?;
            if !notes
                .iter()
                .any(|existing| Pc::from(existing) == Pc::from(&note))
            {
                notes.push(note);
            }
        }
    }
    anyhow::ensure!(
        notes.len() >= 2,
        "melodic shape search requires at least two distinct notes"
    );
    Ok(notes)
}

fn parse_note_filters(tokens: &[String], chord: &[Note]) -> Result<HashSet<Pc>> {
    if tokens.is_empty() {
        return Ok(chord.iter().map(Pc::from).collect());
    }
    let notes = parse_notes_allow_one(tokens)?;
    let chord_pcs: HashSet<Pc> = chord.iter().map(Pc::from).collect();
    let selected: HashSet<Pc> = notes.iter().map(Pc::from).collect();
    for pc in &selected {
        anyhow::ensure!(
            chord_pcs.contains(pc),
            "starting note {} is not in the input set",
            pc.notes()[0]
        );
    }
    Ok(selected)
}

fn parse_notes_allow_one(tokens: &[String]) -> Result<Vec<Note>> {
    let mut notes = Vec::new();
    for token in tokens {
        for part in token.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let note = Note::from_str(part)
                .or_else(|_| parse_pc(part).map(|pc| pc.notes()[0]).ok_or(()))
                .map_err(|_| anyhow::anyhow!("invalid note or pitch class: '{part}'"))?;
            notes.push(note);
        }
    }
    anyhow::ensure!(!notes.is_empty(), "no starting notes provided");
    Ok(notes)
}

fn parse_categories(raw: &[String]) -> Result<HashSet<&'static str>> {
    let mut selected = HashSet::new();
    if raw.is_empty() {
        selected.extend(CATEGORY_NAMES.iter().copied());
        return Ok(selected);
    }
    for token in raw {
        for part in token.split(',') {
            let normalized = part.trim().to_ascii_lowercase();
            if normalized == "all" {
                selected.extend(CATEGORY_NAMES.iter().copied());
                continue;
            }
            let canonical = match normalized.as_str() {
                "open" => "open",
                "simple" | "caged" => "simple",
                "2" | "2nps" | "2-2" | "2-2nps" => "2nps",
                "2-3" | "2+3" | "2-3nps" => "2-3nps",
                "3-2" | "3+2" | "3-2nps" => "3-2nps",
                "3" | "3nps" | "3-3" | "3-3nps" => "3nps",
                "other" | "all-shapes" | "exhaustive" => "exhaustive",
                _ => anyhow::bail!(
                    "unknown category '{part}' (options: all, open, simple, 2nps, 2-3nps, 3-2nps, 3nps, exhaustive)"
                ),
            };
            selected.insert(canonical);
        }
    }
    anyhow::ensure!(!selected.is_empty(), "no shape categories selected");
    Ok(selected)
}

fn nps_category(category: &str) -> Option<(usize, usize)> {
    match category {
        "2nps" => Some((2, 2)),
        "2-3nps" => Some((2, 3)),
        "3-2nps" => Some((3, 2)),
        "3nps" => Some((3, 3)),
        _ => None,
    }
}

fn collect_shapes<'a>(
    chord: &Vec<Note>,
    fretboard: &'a Fretboard,
    categories: &HashSet<&str>,
    starting_pcs: &HashSet<Pc>,
) -> Result<Vec<ShapeEntry<'a>>> {
    let mut entries = Vec::new();

    if categories.contains("open") {
        let shape =
            find_open_scale_shape(chord, fretboard).context("open-position search failed")?;
        let starting_note = shape
            .shape
            .first()
            .map(|n| n.pitch.note)
            .unwrap_or(chord[0]);
        if starting_pcs.contains(&Pc::from(&starting_note)) {
            entries.push(ShapeEntry {
                category: "open",
                starting_note,
                rank: 1,
                shape,
            });
        }
    }

    if categories.contains("simple") {
        let result = ScaleShapeSearchResult::from_raw_search_result(chord, fretboard)
            .context("simple-shape classification failed")?;
        for shape in result.simple {
            let starting_note = shape
                .shape
                .first()
                .map(|n| n.pitch.note)
                .unwrap_or(chord[0]);
            if starting_pcs.contains(&Pc::from(&starting_note)) {
                entries.push(ShapeEntry {
                    category: "simple",
                    starting_note,
                    rank: 0,
                    shape,
                });
            }
        }
    }

    for category in ["2nps", "2-3nps", "3-2nps", "3nps"] {
        if !categories.contains(category) {
            continue;
        }
        let config = nps_category(category).unwrap();
        for starting_note in chord {
            if !starting_pcs.contains(&Pc::from(starting_note)) {
                continue;
            }
            if let Ok(shape) = n_note_per_string_shape(config, chord, starting_note, fretboard) {
                entries.push(ShapeEntry {
                    category,
                    starting_note: *starting_note,
                    rank: 0,
                    shape,
                });
            }
        }
    }

    if categories.contains("exhaustive") {
        let all = find_all_scale_shapes(chord, fretboard);
        for starting_note in chord {
            if !starting_pcs.contains(&Pc::from(starting_note)) {
                continue;
            }
            if let Some(shapes) = all.get(starting_note) {
                for (index, shape) in shapes.iter().enumerate() {
                    entries.push(ShapeEntry {
                        category: "exhaustive",
                        starting_note: *starting_note,
                        rank: index + 1,
                        shape: shape.clone(),
                    });
                }
            }
        }
    }

    Ok(entries)
}

fn shape_key(shape: &MelodicFretboardShape<'_>) -> String {
    shape
        .shape
        .iter()
        .map(|note| format!("{}:{}", note.string, note.fret))
        .collect::<Vec<_>>()
        .join(";")
}

fn apply_filters<'a>(
    mut entries: Vec<ShapeEntry<'a>>,
    max_score: Option<usize>,
    max_span: Option<u8>,
    deduplicate: bool,
    sort: &str,
    limit: Option<usize>,
) -> Result<Vec<ShapeEntry<'a>>> {
    if let Some(max_score) = max_score {
        entries.retain(|entry| entry.shape.score <= max_score);
    }
    if let Some(max_span) = max_span {
        entries.retain(|entry| {
            let (low, high) = entry.shape.span();
            high.saturating_sub(low) <= max_span
        });
    }
    if deduplicate {
        let mut seen = HashSet::new();
        entries.retain(|entry| seen.insert(shape_key(&entry.shape)));
    }
    match sort.to_ascii_lowercase().as_str() {
        "category" => entries.sort_by_key(|entry| (entry.category, entry.rank)),
        "score" => entries.sort_by_key(|entry| {
            let (low, high) = entry.shape.span();
            (entry.shape.score, high.saturating_sub(low), low)
        }),
        "span" => entries.sort_by_key(|entry| {
            let (low, high) = entry.shape.span();
            (high.saturating_sub(low), entry.shape.score, low)
        }),
        "position" | "fret" => entries.sort_by_key(|entry| {
            let (low, high) = entry.shape.span();
            (low, high, entry.shape.score)
        }),
        other => {
            anyhow::bail!("unknown sort order '{other}' (options: category, score, span, position)")
        }
    }
    if let Some(limit) = limit {
        entries.truncate(limit);
    }
    for (index, entry) in entries.iter_mut().enumerate() {
        if entry.rank == 0 {
            entry.rank = index + 1;
        }
    }
    Ok(entries)
}

fn orientation(value: &str) -> Result<Orientation> {
    match value.to_ascii_lowercase().as_str() {
        "vertical" | "portrait" => Ok(Orientation::Vertical),
        "horizontal" | "landscape" => Ok(Orientation::Horizontal),
        other => anyhow::bail!("unknown orientation '{other}' (options: vertical, horizontal)"),
    }
}

fn entry_title(entry: &ShapeEntry<'_>) -> String {
    let (low, high) = entry.shape.span();
    format!(
        "{} from {} · rank {} · score {} · span {}",
        entry.category,
        entry.starting_note,
        entry.rank,
        entry.shape.score,
        high.saturating_sub(low)
    )
}

fn render_entry(
    entry: &ShapeEntry<'_>,
    root_pc: Pc,
    theme: &SvgTheme,
    orientation: Orientation,
    args: &MelodicShapesArgs,
) -> String {
    let (min_fret, max_fret) = entry.shape.span();
    let automatic_start = if min_fret == 0 {
        0
    } else {
        min_fret.saturating_sub(args.fret_padding)
    };
    let start_fret = args.start_fret.unwrap_or(automatic_start);
    let automatic_count = max_fret
        .saturating_sub(start_fret)
        .saturating_add(args.fret_padding)
        .max(1);
    let num_frets = args.num_frets.unwrap_or(automatic_count);

    let mut builder = FretboardBuilder::new()
        .start_fret(start_fret)
        .num_frets(num_frets)
        .orientation(orientation)
        .theme(theme.clone())
        .show_fret_numbers(!args.no_fret_numbers)
        .show_string_names(!args.no_string_names);
    if !args.no_titles {
        builder = builder.title(entry_title(entry));
    }
    for note in &entry.shape.shape {
        let position = if note.fret == 0 {
            FretPosition::Open {
                string: note.string,
            }
        } else {
            FretPosition::Fretted {
                string: note.string,
                fret: note.fret,
            }
        };
        builder = builder.position(position);
        if !args.no_root_markers && Pc::from(&note.pitch.note) == root_pc {
            builder = builder.root_at(note.string, note.fret);
        }
    }
    builder.build()
}

fn svg_inner(svg: &str) -> &str {
    let start = svg.find('>').map(|index| index + 1).unwrap_or(0);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    &svg[start..end]
}

fn build_svg(
    entries: &[ShapeEntry<'_>],
    root_pc: Pc,
    theme: &SvgTheme,
    orientation: Orientation,
    args: &MelodicShapesArgs,
    heading: &str,
) -> String {
    let columns = args.columns.max(1).min(entries.len().max(1));
    let rows = entries.len().max(1).div_ceil(columns);
    let header_height = 56u32;
    let width = args.gap + columns as u32 * (args.tile_width + args.gap);
    let height = header_height + args.gap + rows as u32 * (args.tile_height + args.gap);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n<text x=\"{}\" y=\"36\" font-family=\"sans-serif\" font-size=\"24\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
        theme.background_color,
        args.gap,
        theme.text_color,
        xml_escape(heading),
    );
    for (index, entry) in entries.iter().enumerate() {
        let source = render_entry(entry, root_pc, theme, orientation.clone(), args);
        let (source_width, source_height) = svg_dimensions(&source);
        let scale = (args.tile_width as f32 / source_width as f32)
            .min(args.tile_height as f32 / source_height as f32);
        let x = args.gap + (index % columns) as u32 * (args.tile_width + args.gap);
        let y = header_height + args.gap + (index / columns) as u32 * (args.tile_height + args.gap);
        svg.push_str(&format!(
            "<g transform=\"translate({x} {y}) scale({scale:.5})\">{}</g>\n",
            svg_inner(&source)
        ));
    }
    svg.push_str("</svg>\n");
    svg
}

fn svg_dimensions(svg: &str) -> (u32, u32) {
    fn attr(svg: &str, name: &str) -> Option<u32> {
        let needle = format!("{name}=\"");
        let rest = svg.get(svg.find(&needle)? + needle.len()..)?;
        rest.get(..rest.find('"')?)?.parse().ok()
    }
    (
        attr(svg, "width").unwrap_or(240),
        attr(svg, "height").unwrap_or(240),
    )
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text_report(entries: &[ShapeEntry<'_>], chord: &[Note], args: &MelodicShapesArgs) -> String {
    let mut out = format!(
        "Melodic fretboard shapes: {}\nTuning: {} · Shapes: {}\n\n",
        chord
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
        args.tuning,
        entries.len()
    );
    for (index, entry) in entries.iter().enumerate() {
        let (low, high) = entry.shape.span();
        out.push_str(&format!(
            "{:>3}. {:<10} from {:<4} score={:<3} span={}-{}  {}\n",
            index + 1,
            entry.category,
            entry.starting_note,
            entry.shape.score,
            low,
            high,
            entry.shape
        ));
    }
    out
}

fn json_report(
    entries: &[ShapeEntry<'_>],
    chord: &[Note],
    args: &MelodicShapesArgs,
) -> Result<String> {
    let shapes: Vec<Value> = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let (low, high) = entry.shape.span();
            let notes: Vec<Value> = entry
                .shape
                .shape
                .iter()
                .map(|note| {
                    json!({
                        "string": note.string + 1,
                        "fret": note.fret,
                        "note": note.pitch.note.to_string(),
                        "midi": note.pitch.midi_note,
                    })
                })
                .collect();
            json!({
                "index": index + 1,
                "category": entry.category,
                "starting_note": entry.starting_note.to_string(),
                "rank": entry.rank,
                "score": entry.shape.score,
                "span": { "low": low, "high": high, "width": high.saturating_sub(low) },
                "complete": entry.shape.is_complete(),
                "notes": notes,
            })
        })
        .collect();
    Ok(serde_json::to_string_pretty(&json!({
        "pitch_set": chord.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "tuning": args.tuning,
        "count": entries.len(),
        "shapes": shapes,
    }))?)
}

fn write_output(format: OutputFormat, output: Option<&str>, bytes: &[u8]) -> Result<()> {
    if format.is_binary() {
        let path = output.context("PNG/PDF output requires --output <path>")?;
        fs::write(path, bytes).with_context(|| format!("failed to write {path}"))?;
    } else if let Some(path) = output {
        fs::write(path, bytes).with_context(|| format!("failed to write {path}"))?;
    } else {
        io::stdout().write_all(bytes)?;
    }
    Ok(())
}

fn svg_to_pdf(svg: &str) -> Result<Vec<u8>> {
    let mut child = Command::new("rsvg-convert")
        .args(["--format", "pdf"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("PDF output requires the 'rsvg-convert' executable")?;
    child
        .stdin
        .as_mut()
        .context("failed to open rsvg-convert stdin")?
        .write_all(svg.as_bytes())
        .context("failed to send SVG to rsvg-convert")?;
    let output = child
        .wait_with_output()
        .context("failed to wait for rsvg-convert")?;
    anyhow::ensure!(
        output.status.success(),
        "rsvg-convert failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

pub fn run(args: MelodicShapesArgs) -> Result<()> {
    anyhow::ensure!(args.columns > 0, "--columns must be at least 1");
    anyhow::ensure!(
        args.tile_width > 0 && args.tile_height > 0,
        "tile dimensions must be positive"
    );
    anyhow::ensure!(args.dpi > 0.0, "--dpi must be positive");
    let chord = parse_notes(&args.input)?;
    let tuning = TuningSpec::parse(&args.tuning)?;
    let fretboard = &tuning.fretboard;
    let categories = parse_categories(&args.categories)?;
    let starting_pcs = parse_note_filters(&args.starting_notes, &chord)?;
    let root = match args.root.as_deref() {
        Some(root) => parse_notes_allow_one(&[root.to_string()])?[0],
        None => chord[0],
    };
    anyhow::ensure!(
        chord.iter().any(|note| Pc::from(note) == Pc::from(&root)),
        "root {root} is not in the input set"
    );
    let format = OutputFormat::resolve(args.format.as_deref(), args.output.as_deref())?;
    let orientation = orientation(&args.orientation)?;
    let theme = resolve_theme(args.theme.as_deref())?;

    let entries = collect_shapes(&chord, fretboard, &categories, &starting_pcs)?;
    let entries = apply_filters(
        entries,
        args.max_score,
        args.max_span,
        args.deduplicate,
        &args.sort,
        args.limit,
    )?;
    anyhow::ensure!(
        !entries.is_empty(),
        "no melodic shapes matched the requested selection and filters"
    );

    let heading = args.title.clone().unwrap_or_else(|| {
        format!(
            "Melodic shapes: {}",
            chord
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        )
    });
    let bytes = match format {
        OutputFormat::Text => text_report(&entries, &chord, &args).into_bytes(),
        OutputFormat::Json => json_report(&entries, &chord, &args)?.into_bytes(),
        OutputFormat::Svg => build_svg(
            &entries,
            Pc::from(&root),
            &theme,
            orientation.clone(),
            &args,
            &heading,
        )
        .into_bytes(),
        OutputFormat::Png => {
            let svg = build_svg(
                &entries,
                Pc::from(&root),
                &theme,
                orientation.clone(),
                &args,
                &heading,
            );
            svg_to_png(&svg, dpi_to_scale(args.dpi))
                .context("failed to rasterize melodic-shapes SVG")?
        }
        OutputFormat::Pdf => {
            let svg = build_svg(
                &entries,
                Pc::from(&root),
                &theme,
                orientation,
                &args,
                &heading,
            );
            svg_to_pdf(&svg)?
        }
    };
    write_output(format, args.output.as_deref(), &bytes)?;
    if args.verbose {
        eprintln!(
            "melodic-shapes: notes=[{}], tuning={}, categories={}, selected={}, format={:?}",
            chord
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            args.tuning,
            categories.iter().copied().collect::<Vec<_>>().join(","),
            entries.len(),
            format,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::fretboard::STD_6STR_GTR;

    fn args() -> MelodicShapesArgs {
        MelodicShapesArgs {
            input: vec!["B,D#,F#,A#".into()],
            output: None,
            format: Some("json".into()),
            categories: vec!["exhaustive".into()],
            starting_notes: vec![],
            tuning: "standard".into(),
            root: Some("B".into()),
            max_score: None,
            max_span: None,
            limit: None,
            sort: "category".into(),
            deduplicate: false,
            orientation: "horizontal".into(),
            columns: 3,
            tile_width: 320,
            tile_height: 220,
            gap: 20,
            start_fret: None,
            num_frets: None,
            fret_padding: 1,
            no_titles: false,
            no_root_markers: false,
            no_fret_numbers: false,
            no_string_names: false,
            dpi: 144.0,
            theme: None,
            title: None,
            verbose: false,
        }
    }

    #[test]
    fn bmaj7_exhaustive_search_is_stable() {
        let args = args();
        let chord = parse_notes(&args.input).unwrap();
        let categories = parse_categories(&args.categories).unwrap();
        let starts = parse_note_filters(&args.starting_notes, &chord).unwrap();
        let entries = collect_shapes(&chord, &STD_6STR_GTR, &categories, &starts).unwrap();
        assert_eq!(entries.len(), 44);
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.starting_note == Note::B)
                .count(),
            12
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.starting_note == Note::Dis)
                .count(),
            11
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.starting_note == Note::Fis)
                .count(),
            9
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.starting_note == Note::Ais)
                .count(),
            12
        );
    }

    #[test]
    fn category_aliases_and_filters_work() {
        let selected = parse_categories(&["caged,2+3,other".into()]).unwrap();
        assert!(selected.contains("simple"));
        assert!(selected.contains("2-3nps"));
        assert!(selected.contains("exhaustive"));
    }

    #[test]
    fn output_format_prefers_explicit_value() {
        assert_eq!(
            OutputFormat::resolve(Some("json"), Some("out.svg")).unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::resolve(Some("pdf"), None).unwrap(),
            OutputFormat::Pdf
        );
    }
}
