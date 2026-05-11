use anyhow::{Context, Result};
use music::note_collections::PcSet;
use music::svg::PitchCircleBuilder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, parse_pc, resolve_theme};

pub struct PitchCircleArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub root: Option<String>,
    pub show_intervals: bool,
    pub title: Option<String>,
    pub verbose: bool,
}

pub fn run(args: PitchCircleArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;

    let root = match &args.root {
        Some(r) => Some(parse_pc(r).with_context(|| format!("invalid root: '{r}'"))?),
        None => Some(pcs[0]),
    };

    let pc_set = PcSet::from_unzeroed(pcs.clone());
    let theme = resolve_theme(args.theme.as_deref())?;

    if args.verbose {
        eprintln!("pitch-circle: pcs={pc_set:?}, root={root:?}, theme={}", args.theme.as_deref().unwrap_or("default"));
    }

    let mut builder = PitchCircleBuilder::new()
        .from_pc_set(&pc_set)
        .show_intervals(args.show_intervals)
        .theme(theme);

    if let Some(r) = root {
        builder = builder.root(r);
    }
    if let Some(ref t) = args.title {
        builder = builder.title(t.as_str());
    }

    let svg = builder.build();

    // Output: file or stdout
    match args.output {
        Some(ref path) => {
            let p = Path::new(path);
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("svg");
            anyhow::ensure!(
                ext == "svg",
                "pitch-circle only supports .svg output (got .{ext})"
            );
            fs::write(p, &svg)
                .with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", svg.len());
            }
        }
        None => {
            io::stdout().write_all(svg.as_bytes())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_produces_svg_to_file() {
        let dir = std::env::temp_dir().join("slonimsky_test_pc");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("test.svg");
        let args = PitchCircleArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            root: None,
            show_intervals: true,
            title: Some("C Major".into()),
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "output should be SVG");
        assert!(content.contains("</svg>"), "SVG should be closed");
        assert!(content.contains("<circle") || content.contains("<text"),
            "SVG should contain graphical elements");
        // Clean up
        let _ = std::fs::remove_file(&out);
    }
}
