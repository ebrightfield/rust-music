use anyhow::{Context, Result};
use music::fretboard::Fretboard;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::svg::{IntervalBuilder, PitchCircleBuilder};
use music_ron::ast::{Document, OwnedPitch, OwnedTuning};
use music_ron::convert::{
    convert_chord_progression, convert_fretboard_shape, convert_interval_matrix,
    convert_pitch_circle, convert_scale_diagram, convert_snippet, convert_tab,
    IntervalMatrixIdentity, IntervalStyle, PitchCircleIdentity,
};
use std::path::PathBuf;

use super::output::{svg_bytes, write_output, OutputFormat};

pub struct RenderArgs {
    pub input: String,
    pub format: Option<String>,
    pub output: Option<String>,
    pub dpi: f32,
    pub verbose: bool,
}

fn read_document(input: &str) -> Result<Document> {
    if input == "-" {
        let mut source = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut source)
            .context("failed to read RON document from stdin")?;
        music_ron::parse(&source).context("invalid RON music document")
    } else {
        music_ron::parse_path(&PathBuf::from(input))
            .with_context(|| format!("failed to parse RON document '{input}'"))
    }
}

fn owned_pitch(value: &OwnedPitch) -> Result<Pitch> {
    match value {
        OwnedPitch::Long { note, octave } => Ok(Pitch::new(
            *note,
            i8::try_from(*octave).context("tuning octave is outside the supported range")?,
        )),
        OwnedPitch::Shorthand(s) => {
            let wrapper = format!(
                r#"(kind: "Snippet", clef: "treble", events: [Note(pitch: "{s}", duration: "4")])"#
            );
            let Document::Snippet(snippet) = music_ron::parse(&wrapper)? else {
                unreachable!()
            };
            let resolved = convert_snippet(&snippet)?;
            match &resolved.events[0].event {
                music::notation::rhythm::NotatedEvent::SingleEvent(
                    music::notation::rhythm::SingleEvent::Pitch(pitch),
                    _,
                ) => Ok(*pitch),
                _ => unreachable!(),
            }
        }
    }
}

fn tuning(value: &OwnedTuning) -> Result<Fretboard> {
    match value {
        OwnedTuning::Named(name) => Ok(Fretboard {
            open_strings: music_ron::tuning_registry::resolve(name, "tuning")?
                .open_strings
                .clone(),
        }),
        OwnedTuning::Inline { pitches } => Ok(Fretboard {
            open_strings: pitches
                .iter()
                .map(owned_pitch)
                .collect::<Result<Vec<_>>>()?,
        }),
    }
}

fn validate(document: &Document) -> Result<()> {
    match document {
        Document::Snippet(value) => {
            convert_snippet(value)?;
        }
        Document::Tab(value) => {
            let board = tuning(&value.tuning)?;
            convert_tab(value, &board)?;
        }
        Document::FretboardShape(value) => {
            convert_fretboard_shape(value)?;
        }
        Document::PitchCircle(value) => {
            convert_pitch_circle(value)?;
        }
        Document::ChordProgression(value) => {
            convert_chord_progression(value)?;
        }
        Document::ScaleDiagram(value) => {
            convert_scale_diagram(value)?;
            tuning(&value.tuning)?;
        }
        Document::IntervalMatrix(value) => {
            convert_interval_matrix(value)?;
        }
        Document::Score(_) => {
            anyhow::bail!("Score documents are not supported by slonimsky render");
        }
    }
    Ok(())
}

fn identity_pcs(identity: &PitchCircleIdentity) -> Result<(Vec<Pc>, Option<Pc>)> {
    match identity {
        PitchCircleIdentity::Chord { root, shape, .. } => {
            Ok((shape.iter().copied().collect(), Some(Pc::from(root))))
        }
        PitchCircleIdentity::Pcs(pcs) => Ok((pcs.clone(), None)),
        PitchCircleIdentity::Scale(name) => Ok((scale_pcs(name)?, None)),
    }
}

fn matrix_pcs(identity: &IntervalMatrixIdentity) -> Result<Vec<Pc>> {
    match identity {
        IntervalMatrixIdentity::Chord { shape, .. } => Ok(shape.iter().copied().collect()),
        IntervalMatrixIdentity::Pcs(pcs) => Ok(pcs.clone()),
        IntervalMatrixIdentity::Scale(name) => scale_pcs(name),
    }
}

fn scale_pcs(name: &str) -> Result<Vec<Pc>> {
    let values: &[u8] = match name.to_ascii_lowercase().replace('_', "-").as_str() {
        "major" | "ionian" => &[0, 2, 4, 5, 7, 9, 11],
        "minor" | "natural-minor" | "aeolian" => &[0, 2, 3, 5, 7, 8, 10],
        "harmonic-minor" => &[0, 2, 3, 5, 7, 8, 11],
        "melodic-minor" => &[0, 2, 3, 5, 7, 9, 11],
        other => anyhow::bail!("unknown scale '{other}'"),
    };
    Ok(values.iter().copied().map(Pc::from).collect())
}

fn escaped(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn generic_svg(document: &Document) -> String {
    let json = serde_json::to_string_pretty(document).unwrap_or_default();
    let lines = json
        .lines()
        .take(28)
        .enumerate()
        .map(|(i, line)| {
            format!(
                r#"<text x="24" y="{}" font-family="monospace" font-size="14">{}</text>"#,
                58 + i * 18,
                escaped(line)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="900" height="620" viewBox="0 0 900 620"><rect width="900" height="620" fill="white"/><text x="24" y="30" font-family="sans-serif" font-size="20" font-weight="bold">{}</text>{}</svg>"#,
        document.kind_str(),
        lines
    )
}

fn render_svg(document: &Document) -> Result<String> {
    match document {
        Document::PitchCircle(value) => {
            let resolved = convert_pitch_circle(value)?;
            let (pcs, identity_root) = identity_pcs(&resolved.identity)?;
            let mut builder = PitchCircleBuilder::new().pitches(pcs);
            if let Some(root) = resolved.root.map(|n| Pc::from(&n)).or(identity_root) {
                builder = builder.root(root);
            }
            Ok(builder.build())
        }
        Document::IntervalMatrix(value) => {
            let resolved = convert_interval_matrix(value)?;
            let mut builder = IntervalBuilder::new().pitches(matrix_pcs(&resolved.identity)?);
            if let Some(title) = resolved.title {
                builder = builder.title(title);
            }
            Ok(match resolved.style {
                IntervalStyle::Vector => builder.build_vector(),
                IntervalStyle::FullVector => builder.build_full_vector(),
                IntervalStyle::Matrix => builder.build_matrix(),
                IntervalStyle::Linear => builder.build_linear(),
            })
        }
        _ => Ok(generic_svg(document)),
    }
}

pub fn run(args: RenderArgs) -> Result<()> {
    let document = read_document(&args.input)?;
    validate(&document)
        .with_context(|| format!("{} document failed validation", document.kind_str()))?;
    let format = OutputFormat::resolve(
        args.format.as_deref(),
        args.output.as_deref(),
        OutputFormat::Text,
    )?;
    let bytes = match format {
        OutputFormat::Text => {
            format!("kind: {}\n{document:#?}\n", document.kind_str()).into_bytes()
        }
        OutputFormat::Json => serde_json::to_vec_pretty(&document)?,
        OutputFormat::Svg | OutputFormat::Png | OutputFormat::Pdf => {
            svg_bytes(render_svg(&document)?, format, args.dpi)?
        }
    };
    write_output(format, args.output.as_deref(), &bytes)?;
    if args.verbose {
        eprintln!(
            "render: input={}, kind={}, format={format:?}, bytes={}",
            args.input,
            document.kind_str(),
            bytes.len()
        );
    }
    Ok(())
}
