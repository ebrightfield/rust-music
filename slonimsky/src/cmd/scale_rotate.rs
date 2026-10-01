use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, pc_label};

pub struct ScaleRotateArgs {
    pub input: Vec<String>,
    pub steps: isize,
    pub output: Option<String>,
    pub format: Option<String>,
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn resolve(explicit: Option<&str>, output: Option<&str>) -> Result<Self> {
        let value = explicit.or_else(|| {
            output.and_then(|path| Path::new(path).extension().and_then(|ext| ext.to_str()))
        });
        match value.unwrap_or("text").to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => {
                anyhow::bail!("unsupported scale-rotate format '{other}' (options: text, json)")
            }
        }
    }
}

pub fn run(args: ScaleRotateArgs) -> Result<()> {
    let mut pcs = parse_input_to_pcs(&args.input)?;
    pcs.sort_by_key(|pc| u8::from(*pc));
    pcs.dedup();
    anyhow::ensure!(
        pcs.len() >= 2,
        "scale rotation needs at least two distinct pitches"
    );

    let steps = args.steps.rem_euclid(pcs.len() as isize) as usize;
    pcs.rotate_left(steps);
    let root = pcs[0];
    let relative: Vec<u8> = pcs
        .iter()
        .map(|pc| (u8::from(*pc) + 12 - u8::from(root)) % 12)
        .collect();
    let notes: Vec<&str> = pcs.iter().map(|pc| pc_label(*pc)).collect();
    let format = OutputFormat::resolve(args.format.as_deref(), args.output.as_deref())?;
    let rendered = match format {
        OutputFormat::Text => format!(
            "Rotation: {}\nRoot: {} ({})\nNotes: {}\nShape: {}\n",
            args.steps,
            pc_label(root),
            u8::from(root),
            notes.join(" "),
            relative
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        OutputFormat::Json => {
            serde_json::to_string_pretty(&json!({
                "rotation": args.steps,
                "normalized_rotation": steps,
                "root": pc_label(root),
                "root_pc": u8::from(root),
                "notes": notes,
                "pitch_classes": pcs.iter().map(|pc| u8::from(*pc)).collect::<Vec<_>>(),
                "shape": relative,
            }))? + "\n"
        }
    };

    match &args.output {
        Some(path) => {
            fs::write(path, &rendered).with_context(|| format!("failed to write {path}"))?
        }
        None => io::stdout().write_all(rendered.as_bytes())?,
    }
    if args.verbose {
        eprintln!("scale-rotate: {} notes, root {}", pcs.len(), pc_label(root));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use music::note::pitch_class::Pc;
    #[test]
    fn dorian_rotation_rebases_major_scale() {
        let mut pcs = vec![
            Pc::Pc0,
            Pc::Pc2,
            Pc::Pc4,
            Pc::Pc5,
            Pc::Pc7,
            Pc::Pc9,
            Pc::Pc11,
        ];
        pcs.rotate_left(1);
        let root = pcs[0];
        let relative: Vec<u8> = pcs
            .iter()
            .map(|pc| (u8::from(*pc) + 12 - u8::from(root)) % 12)
            .collect();
        assert_eq!(relative, vec![0, 2, 3, 5, 7, 9, 10]);
    }
}
