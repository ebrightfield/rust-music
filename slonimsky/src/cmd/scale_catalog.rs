use anyhow::{Context, Result};
use music::note_collections::geometry::symmetry::transpositional::Modes;
use music::note_collections::pc_set::{AsPcSlice, PcShape};
use music::note_collections::OctavePartition;
use musical_combinatorics::seven_note_scales::SevenNoteScaleQuality;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, pc_label};

pub struct ScaleCatalogArgs {
    pub scale: Option<String>,
    pub identify: Vec<String>,
    pub modes: bool,
    pub format: Option<String>,
    pub output: Option<String>,
    pub verbose: bool,
}

pub const MAJOR_MODE_NAMES: &[&str] = &[
    "Ionian",
    "Dorian",
    "Phrygian",
    "Lydian",
    "Mixolydian",
    "Aeolian",
    "Locrian",
];
pub const MELODIC_MINOR_MODE_NAMES: &[&str] = &[
    "Melodic Minor",
    "Dorian b2",
    "Lydian Augmented",
    "Lydian Dominant",
    "Mixolydian b6",
    "Locrian #2",
    "Altered",
];
pub const HARMONIC_MINOR_MODE_NAMES: &[&str] = &[
    "Harmonic Minor",
    "Locrian #6",
    "Ionian Augmented",
    "Dorian #4",
    "Phrygian Dominant",
    "Lydian #2",
    "Ultralocrian",
];
pub const HARMONIC_MAJOR_MODE_NAMES: &[&str] = &[
    "Harmonic Major",
    "Dorian b5",
    "Phrygian b4",
    "Lydian b3",
    "Mixolydian b2",
    "Lydian Augmented #2",
    "Locrian bb7",
];

pub struct CatalogScale {
    pub slug: &'static str,
    pub name: &'static str,
    pub quality: SevenNoteScaleQuality,
    pub mode_names: Option<&'static [&'static str]>,
}

pub fn catalog() -> Vec<CatalogScale> {
    use SevenNoteScaleQuality::*;
    vec![
        entry("major", "Major", Major, Some(MAJOR_MODE_NAMES)),
        entry(
            "melodic-minor",
            "Melodic Minor",
            MelodicMinor,
            Some(MELODIC_MINOR_MODE_NAMES),
        ),
        entry("major-b9-b13", "Major b9 b13", MajorFlat9Flat13, None),
        entry(
            "mixolydian-sharp11-b13",
            "Mixolydian #11 b13",
            MixolydianSharp11Flat13,
            None,
        ),
        entry(
            "harmonic-minor",
            "Harmonic Minor",
            HarmonicMinor,
            Some(HARMONIC_MINOR_MODE_NAMES),
        ),
        entry(
            "harmonic-major",
            "Harmonic Major",
            HarmonicMajor,
            Some(HARMONIC_MAJOR_MODE_NAMES),
        ),
        entry("major-b9", "Major b9", MajorFlat9, None),
        entry("major-sharp9", "Major #9", MajorSharp9, None),
        entry("mixolydian-sharp9", "Mixolydian #9", MixolydianSharp9, None),
        entry("lydian-b9", "Lydian b9", LydianFlat9, None),
        entry(
            "mixolydian-sharp9-sharp11",
            "Mixolydian #9 #11",
            MixolydianSharp9Sharp11,
            None,
        ),
        entry(
            "mixolydian-b9-sharp11",
            "Mixolydian b9 #11",
            MixolydianFlat9Sharp11,
            None,
        ),
        entry(
            "melodic-minor-b9-sharp11",
            "Melodic Minor b9 #11",
            MelodicMinorFlat9Sharp11,
            None,
        ),
        entry(
            "mixolydian-b9-sharp11-b13",
            "Mixolydian b9 #11 b13",
            MixolydianFlat9Sharp11Flat13,
            None,
        ),
        entry(
            "mixolydian-sharp9-b13",
            "Mixolydian #9 b13",
            MixolydianSharp9Flat13,
            None,
        ),
        entry(
            "melodic-minor-b11",
            "Melodic Minor b11",
            MelodicMinorFlat11,
            None,
        ),
        entry(
            "mixolydian-sharp9-sharp11-b13",
            "Mixolydian #9 #11 b13",
            MixolydianSharp9Sharp11Flat13,
            None,
        ),
        entry(
            "melodic-minor-b9-b11",
            "Melodic Minor b9 b11",
            MelodicMinorFlat9Flat11,
            None,
        ),
        entry("lydian-b9-b13", "Lydian b9 b13", LydianFlat9Flat13, None),
        entry(
            "melodic-minor-b9-sharp11-b13",
            "Melodic Minor b9 #11 b13",
            MelodicMinorFlat9Sharp11Flat13,
            None,
        ),
        entry(
            "lydian-sharp9-b13",
            "Lydian #9 b13",
            LydianSharp9Flat13,
            None,
        ),
        entry("major-sharp9-b13", "Major #9 b13", MajorSharp9Flat13, None),
    ]
}

fn entry(
    slug: &'static str,
    name: &'static str,
    quality: SevenNoteScaleQuality,
    mode_names: Option<&'static [&'static str]>,
) -> CatalogScale {
    CatalogScale {
        slug,
        name,
        quality,
        mode_names,
    }
}

fn normalized_name(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .replace(' ', "-")
}

pub fn resolve_scale(value: &str) -> Result<CatalogScale> {
    let normalized = normalized_name(value);
    catalog()
        .into_iter()
        .find(|scale| scale.slug == normalized || (normalized == "ionian" && scale.slug == "major"))
        .with_context(|| format!("unknown scale '{value}'; use `scale-catalog` to list names"))
}

pub fn mode_name(scale: &CatalogScale, index: usize) -> String {
    scale
        .mode_names
        .and_then(|names| names.get(index))
        .map(|name| (*name).to_owned())
        .unwrap_or_else(|| format!("{} mode {}", scale.name, index + 1))
}

fn shape(scale: &CatalogScale) -> PcShape {
    PcShape::from(&OctavePartition::from(&scale.quality))
}

fn numbers(shape: &PcShape) -> Vec<u8> {
    shape.as_pc_slice().iter().map(|pc| u8::from(*pc)).collect()
}

fn notes(shape: &PcShape) -> Vec<&'static str> {
    shape.as_pc_slice().iter().map(|pc| pc_label(*pc)).collect()
}

fn selected(scale: Option<&str>) -> Result<Vec<CatalogScale>> {
    match scale {
        Some(value) => Ok(vec![resolve_scale(value)?]),
        None => Ok(catalog()),
    }
}

fn render_catalog(
    scales: &[CatalogScale],
    include_modes: bool,
    json_output: bool,
) -> Result<String> {
    if json_output {
        let rows = scales
            .iter()
            .map(|scale| {
                let parent = shape(scale);
                let modes = include_modes.then(|| {
                    parent
                        .modes()
                        .iter()
                        .enumerate()
                        .map(|(index, mode)| {
                            json!({
                                "number": index + 1,
                                "name": mode_name(scale, index),
                                "pitch_classes": numbers(mode),
                                "notes_from_c": notes(mode),
                            })
                        })
                        .collect::<Vec<_>>()
                });
                json!({
                    "slug": scale.slug,
                    "name": scale.name,
                    "pitch_classes": numbers(&parent),
                    "notes_from_c": notes(&parent),
                    "modes": modes,
                })
            })
            .collect::<Vec<_>>();
        return Ok(serde_json::to_string_pretty(&json!({"scales": rows}))? + "\n");
    }

    let mut out = format!("Seven-note scale catalog ({} families)\n", scales.len());
    for scale in scales {
        let parent = shape(scale);
        out.push_str(&format!(
            "\n{} [{}]\n  pcs: {}\n  C: {}\n",
            scale.name,
            scale.slug,
            numbers(&parent)
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(" "),
            notes(&parent).join(" ")
        ));
        if include_modes {
            for (index, mode) in parent.modes().iter().enumerate() {
                out.push_str(&format!(
                    "  {:>2}. {:<34} {}\n",
                    index + 1,
                    mode_name(scale, index),
                    numbers(mode)
                        .iter()
                        .map(u8::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
        }
    }
    Ok(out)
}

fn identify(input: &[String], json_output: bool) -> Result<String> {
    let pcs = parse_input_to_pcs(input)?;
    anyhow::ensure!(
        pcs.len() == 7,
        "scale identification requires exactly seven pitch classes"
    );
    let tonic = pcs[0];
    let tonic_pc = u8::from(tonic);
    let mut relative = pcs
        .iter()
        .map(|pc| (u8::from(*pc) + 12 - tonic_pc) % 12)
        .collect::<Vec<_>>();
    relative.sort_unstable();
    relative.dedup();
    anyhow::ensure!(
        relative.len() == 7,
        "scale identification requires seven distinct pitch classes"
    );

    let mut matches = Vec::new();
    for scale in catalog() {
        for (index, mode) in shape(&scale).modes().iter().enumerate() {
            if numbers(mode) == relative {
                matches.push((scale.slug, scale.name, index + 1, mode_name(&scale, index)));
            }
        }
    }
    anyhow::ensure!(
        !matches.is_empty(),
        "seven-note scale is not in the catalog"
    );

    if json_output {
        let values = matches
            .iter()
            .map(|(slug, family, number, name)| {
                json!({
                    "family": slug, "family_name": family, "mode": number, "mode_name": name,
                })
            })
            .collect::<Vec<_>>();
        return Ok(serde_json::to_string_pretty(&json!({
            "tonic": pc_label(tonic), "tonic_pc": tonic_pc, "shape": relative, "matches": values,
        }))? + "\n");
    }

    let mut out = format!(
        "Tonic: {} ({})\nShape: {}\n",
        pc_label(tonic),
        tonic_pc,
        relative
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    );
    for (slug, family, number, name) in matches {
        out.push_str(&format!(
            "Match: {family} [{slug}], mode {number}: {name}\n"
        ));
    }
    Ok(out)
}

pub fn run(args: ScaleCatalogArgs) -> Result<()> {
    let format = args
        .format
        .as_deref()
        .or_else(|| {
            args.output
                .as_deref()
                .and_then(|path| Path::new(path).extension()?.to_str())
        })
        .unwrap_or("text");
    let json_output = match format.to_ascii_lowercase().as_str() {
        "text" | "txt" => false,
        "json" => true,
        other => anyhow::bail!("unsupported scale-catalog format '{other}' (options: text, json)"),
    };
    anyhow::ensure!(
        args.identify.is_empty() || args.scale.is_none(),
        "a scale selector cannot be combined with --identify"
    );
    anyhow::ensure!(
        args.identify.is_empty() || !args.modes,
        "--modes cannot be combined with --identify"
    );

    let rendered = if args.identify.is_empty() {
        render_catalog(&selected(args.scale.as_deref())?, args.modes, json_output)?
    } else {
        identify(&args.identify, json_output)?
    };
    match &args.output {
        Some(path) => {
            fs::write(path, &rendered).with_context(|| format!("failed to write {path}"))?
        }
        None => io::stdout().write_all(rendered.as_bytes())?,
    }
    if args.verbose {
        eprintln!(
            "scale-catalog: {}",
            if args.identify.is_empty() {
                "catalog"
            } else {
                "identification"
            }
        );
    }
    Ok(())
}
