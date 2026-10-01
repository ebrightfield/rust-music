use anyhow::{bail, Context, Result};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality_detailed;
use music::note_collections::chord_name::quality::chord::QualityAmbiguity;
use music::note_collections::chord_name::{
    ChordName, ChordNameDisplayConfig, ChordQuality, ExtensionStyle, MajNotation, NamingConfig,
    TonalSpecification,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::str::FromStr;

use super::input::{parse_pc, pc_label};
use super::output::{write_output, OutputFormat};

pub struct NameArgs {
    pub pcs: Vec<String>,
    pub root: Option<String>,
    pub bass: Option<String>,
    pub slash_threshold: Option<usize>,
    pub naming_style: String,
    pub prefer_add: Option<bool>,
    pub show_omissions: Option<bool>,
    pub distinguish_sixth: Option<bool>,
    pub report_ambiguities: Option<bool>,
    pub extension_style: String,
    pub major_symbol: String,
    pub accidentals: String,
    pub explicit_sus4: Option<bool>,
    pub root_spacing: usize,
    pub quality_slash_spacing: usize,
    pub slash_spacing: usize,
    pub format: Option<String>,
    pub output: Option<String>,
    pub verbose: bool,
}

fn normalize_to_root(pcs: &[Pc], root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(&root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

fn parse_input_to_notes(tokens: &[String]) -> Result<Vec<Note>> {
    let mut notes = Vec::new();
    for token in tokens {
        for part in token
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            let note = Note::from_str(part)
                .or_else(|_| {
                    parse_pc(part)
                        .map(|pc| pc.default_sharp_spelling())
                        .ok_or(())
                })
                .map_err(|_| anyhow::anyhow!("invalid note or pitch class: '{part}'"))?;
            notes.push(note);
        }
    }
    anyhow::ensure!(!notes.is_empty(), "no pitch classes provided");
    Ok(notes)
}

fn naming_config(args: &NameArgs) -> Result<NamingConfig> {
    let mut config = match args.naming_style.as_str() {
        "default" => NamingConfig::default(),
        "strict" => NamingConfig::strict(),
        "jazz" => NamingConfig::jazz(),
        "pop" => NamingConfig::pop(),
        other => bail!("unsupported naming style '{other}' (options: default, strict, jazz, pop)"),
    };
    if let Some(value) = args.prefer_add {
        config.prefer_add_notation = value;
    }
    if let Some(value) = args.show_omissions {
        config.show_omissions = value;
    }
    if let Some(value) = args.distinguish_sixth {
        config.distinguish_sixth_from_thirteenth = value;
    }
    if let Some(value) = args.report_ambiguities {
        config.report_ambiguities = value;
    }
    if let Some(value) = args.slash_threshold {
        config.slash_chord_threshold = value;
    }
    Ok(config)
}

fn display_config(args: &NameArgs) -> Result<ChordNameDisplayConfig> {
    let extension_style = match args.extension_style.as_str() {
        "none" => ExtensionStyle::None,
        "strict" => ExtensionStyle::Strict,
        "highest" => ExtensionStyle::Highest,
        "highest-unless-one" => ExtensionStyle::HighestUnlessOne,
        other => bail!(
            "unsupported extension style '{other}' (options: none, strict, highest, highest-unless-one)"
        ),
    };
    let maj_notation = match args.major_symbol.as_str() {
        "delta" => MajNotation::Delta,
        "maj" => MajNotation::Maj,
        "capital-m" => MajNotation::MajCap,
        "lower-maj" => MajNotation::LowerMaj,
        other => {
            bail!("unsupported major symbol '{other}' (options: delta, maj, capital-m, lower-maj)")
        }
    };
    let utf8_accidentals = match args.accidentals.as_str() {
        "unicode" | "utf8" => true,
        "ascii" => false,
        other => bail!("unsupported accidentals '{other}' (options: unicode, ascii)"),
    };
    Ok(ChordNameDisplayConfig {
        explicit_sus4: args.explicit_sus4.unwrap_or(false),
        utf8_accidentals,
        space_between_root_and_quality: args.root_spacing,
        space_between_quality_and_slash: args.quality_slash_spacing,
        space_after_slash: args.slash_spacing,
        extension_style,
        maj_notation,
    })
}

fn ambiguity_text(ambiguity: &QualityAmbiguity) -> String {
    match ambiguity {
        QualityAmbiguity::DuplicateScaleDegree { degree, intervals } => format!(
            "duplicate scale degree {degree} at intervals {}",
            intervals
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        QualityAmbiguity::MultipleInterpretations(names) => {
            format!("multiple interpretations: {}", names.join(", "))
        }
        QualityAmbiguity::SixthVsThirteenth { has_seventh } => format!(
            "sixth versus thirteenth (seventh {})",
            if *has_seventh { "present" } else { "absent" }
        ),
    }
}

fn ambiguity_json(ambiguity: &QualityAmbiguity) -> Value {
    match ambiguity {
        QualityAmbiguity::DuplicateScaleDegree { degree, intervals } => json!({
            "kind": "duplicate-scale-degree",
            "degree": degree,
            "intervals": intervals,
        }),
        QualityAmbiguity::MultipleInterpretations(names) => json!({
            "kind": "multiple-interpretations",
            "names": names,
        }),
        QualityAmbiguity::SixthVsThirteenth { has_seventh } => json!({
            "kind": "sixth-vs-thirteenth",
            "has_seventh": has_seventh,
        }),
    }
}

fn config_json(naming: &NamingConfig, display: &ChordNameDisplayConfig) -> Value {
    json!({
        "naming": {
            "prefer_add_notation": naming.prefer_add_notation,
            "show_omissions": naming.show_omissions,
            "distinguish_sixth_from_thirteenth": naming.distinguish_sixth_from_thirteenth,
            "report_ambiguities": naming.report_ambiguities,
            "slash_chord_threshold": naming.slash_chord_threshold,
        },
        "display": {
            "explicit_sus4": display.explicit_sus4,
            "utf8_accidentals": display.utf8_accidentals,
            "root_spacing": display.space_between_root_and_quality,
            "quality_slash_spacing": display.space_between_quality_and_slash,
            "slash_spacing": display.space_after_slash,
        },
    })
}

pub fn run(args: NameArgs) -> Result<()> {
    let notes = parse_input_to_notes(&args.pcs)?;
    let pcs: Vec<Pc> = notes.iter().map(Pc::from).collect();
    let naming = naming_config(&args)?;
    let display = display_config(&args)?;
    let bass = args
        .bass
        .as_deref()
        .map(|value| {
            Note::from_str(value).with_context(|| format!("invalid note '{value}' for --bass"))
        })
        .transpose()?;
    let explicit_root = args
        .root
        .as_deref()
        .map(|value| {
            Note::from_str(value).with_context(|| format!("invalid note '{value}' for --root"))
        })
        .transpose()?;

    let (root_pc, root_note, bass_note, tonality, normalized, quality) = match bass {
        Some(bass) if explicit_root.is_none() => {
            let inferred = ChordName::infer(&notes, bass, &naming)
                .context("could not infer a bass-aware chord name")?;
            let root = match inferred.tonality {
                TonalSpecification::RootPosition(root)
                | TonalSpecification::SlashChord { root, .. } => root,
                TonalSpecification::None(_) => {
                    unreachable!("bass-aware inference always has a root")
                }
            };
            let root_pc = Pc::from(&root);
            let normalized = inferred.pc_shape.iter().copied().collect();
            (
                root_pc,
                Some(root),
                Some(bass),
                inferred.tonality,
                normalized,
                inferred.quality,
            )
        }
        Some(bass) => {
            let root = explicit_root.expect("guarded above");
            let root_pc = Pc::from(&root);
            anyhow::ensure!(
                pcs.contains(&root_pc),
                "--root must be a member of the chord when --bass is supplied"
            );
            anyhow::ensure!(
                pcs.contains(&Pc::from(&bass)),
                "--bass must be a member of the chord"
            );
            let normalized = normalize_to_root(&pcs, root_pc);
            let (quality, _) = infer_chord_quality_detailed(&normalized, &naming);
            let quality = quality.context("could not identify chord quality at explicit root")?;
            let tonality = if Pc::from(&bass) == root_pc {
                TonalSpecification::RootPosition(root)
            } else {
                TonalSpecification::SlashChord { bass, root }
            };
            (
                root_pc,
                Some(root),
                Some(bass),
                tonality,
                normalized,
                quality,
            )
        }
        None => {
            let root_pc = explicit_root.map_or(pcs[0], |root| Pc::from(&root));
            let normalized = normalize_to_root(&pcs, root_pc);
            let (quality, _) = infer_chord_quality_detailed(&normalized, &naming);
            let quality = quality.with_context(|| {
                format!(
                    "could not identify chord quality for pitch classes: {:?}",
                    pcs.iter().map(u8::from).collect::<Vec<_>>()
                )
            })?;
            (
                root_pc,
                explicit_root,
                None,
                TonalSpecification::RootPosition(
                    explicit_root.unwrap_or_else(|| root_pc.default_sharp_spelling()),
                ),
                normalized,
                quality,
            )
        }
    };

    let (_, ambiguities) = infer_chord_quality_detailed(&normalized, &naming);
    let quality_text = quality.to_string(&display);
    let root_label = root_note
        .map(|note| note.to_string())
        .unwrap_or_else(|| pc_label(root_pc).to_string());
    let mut name = format!(
        "{}{}{}",
        root_label,
        " ".repeat(display.space_between_root_and_quality),
        quality_text
    );
    if let TonalSpecification::SlashChord { bass, .. } = tonality {
        name.push_str(&" ".repeat(display.space_between_quality_and_slash));
        name.push('/');
        name.push_str(&" ".repeat(display.space_after_slash));
        name.push_str(&bass.to_string());
    }

    let format = OutputFormat::resolve(
        args.format.as_deref(),
        args.output.as_deref(),
        OutputFormat::Text,
    )?;
    anyhow::ensure!(
        matches!(format, OutputFormat::Text | OutputFormat::Json),
        "name supports text and JSON output"
    );

    let rendered = match format {
        OutputFormat::Text => {
            let mut text = format!("{name}\n");
            for ambiguity in &ambiguities {
                text.push_str(&format!("Ambiguity: {}\n", ambiguity_text(ambiguity)));
            }
            text
        }
        OutputFormat::Json => {
            let mut normalized_values: Vec<u8> = normalized.iter().map(u8::from).collect();
            normalized_values.sort_unstable();
            let inversion = matches!(tonality, TonalSpecification::SlashChord { .. });
            serde_json::to_string_pretty(&json!({
                "name": name,
                "root": root_label,
                "root_pc": u8::from(root_pc),
                "bass": bass_note.map(|note| note.to_string()),
                "bass_pc": bass_note.map(|note| u8::from(Pc::from(&note))),
                "inversion": inversion,
                "quality": quality_text,
                "pitch_classes": pcs.iter().map(u8::from).collect::<Vec<_>>(),
                "normalized": normalized_values,
                "ambiguities": ambiguities.iter().map(ambiguity_json).collect::<Vec<_>>(),
                "config": config_json(&naming, &display),
            }))? + "\n"
        }
        _ => unreachable!(),
    };
    write_output(format, args.output.as_deref(), rendered.as_bytes())?;

    if args.verbose {
        print_detail(&pcs, root_pc, &quality, &display);
    }
    Ok(())
}

fn print_detail(pcs: &[Pc], root: Pc, quality: &ChordQuality, cfg: &ChordNameDisplayConfig) {
    let pc_ints: Vec<u8> = pcs.iter().map(u8::from).collect();
    eprintln!("  pitch classes: {pc_ints:?}");
    let root_val = u8::from(&root);
    let intervals: Vec<u8> = pcs
        .iter()
        .map(|pitch| (u8::from(pitch) + 12 - root_val) % 12)
        .collect();
    eprintln!("  intervals from root: {intervals:?}");
    eprintln!("  quality: {}", quality.to_string(cfg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(pcs: &[&str]) -> NameArgs {
        NameArgs {
            pcs: pcs.iter().map(|value| (*value).to_string()).collect(),
            root: None,
            bass: None,
            slash_threshold: None,
            naming_style: "default".into(),
            prefer_add: None,
            show_omissions: None,
            distinguish_sixth: None,
            report_ambiguities: None,
            extension_style: "none".into(),
            major_symbol: "maj".into(),
            accidentals: "unicode".into(),
            explicit_sus4: None,
            root_spacing: 0,
            quality_slash_spacing: 0,
            slash_spacing: 0,
            format: None,
            output: Some("/dev/null".into()),
            verbose: false,
        }
    }

    #[test]
    fn normalize_a_minor() {
        let normalized = normalize_to_root(&[Pc::Pc9, Pc::Pc0, Pc::Pc4], Pc::Pc9);
        assert_eq!(
            normalized,
            [Pc::Pc0, Pc::Pc3, Pc::Pc7].into_iter().collect()
        );
    }

    #[test]
    fn names_common_chords() {
        run(args(&["C", "E", "G"])).unwrap();
        run(args(&["D", "F", "A"])).unwrap();
        run(args(&["G", "B", "D", "F"])).unwrap();
    }

    #[test]
    fn explicit_root_is_supported() {
        let mut args = args(&["A", "C", "E"]);
        args.root = Some("A".into());
        run(args).unwrap();
    }
}
