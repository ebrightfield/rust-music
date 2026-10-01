use anyhow::{Context, Result};
use music::ascii::ToAsciiFretboard;
use music::fretboard::{FretboardShape, StringConvention};
use music::svg::{FretboardBuilder, Orientation};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::resolve_theme;
use super::tuning::TuningSpec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Svg,
    FretSpec,
    Positions,
}

impl OutputFormat {
    fn resolve(explicit: Option<&str>, output: Option<&str>) -> Result<Self> {
        let value = explicit.or_else(|| {
            output.and_then(|path| Path::new(path).extension().and_then(|ext| ext.to_str()))
        });
        match value.unwrap_or("svg").to_ascii_lowercase().as_str() {
            "svg" => Ok(Self::Svg),
            "ascii" | "positions" | "txt" => Ok(Self::Positions),
            "fret-spec" | "frets" => Ok(Self::FretSpec),
            other => anyhow::bail!(
                "unsupported fretboard output format '{other}' (options: svg, positions, fret-spec)"
            ),
        }
    }
}

fn resolve_orientation(name: &str) -> Result<Orientation> {
    match name.to_lowercase().as_str() {
        "vertical" | "up-down" => Ok(Orientation::Vertical),
        "horizontal" | "right-left" => Ok(Orientation::Horizontal),
        other => anyhow::bail!("unknown orientation: '{other}' (options: vertical, horizontal)"),
    }
}

pub struct FretboardArgs {
    pub frets: String,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub format: Option<String>,
    pub notes: bool,
    pub high_to_low: bool,
    pub tuning: String,
    pub orientation: Option<String>,
    pub title: Option<String>,
    pub num_frets: Option<u8>,
    pub verbose: bool,
}

pub fn run(args: FretboardArgs) -> Result<()> {
    let tuning = TuningSpec::parse(&args.tuning)?;
    let format = OutputFormat::resolve(args.format.as_deref(), args.output.as_deref())?;
    let shape = FretboardShape::from_string(&args.frets, &tuning.fretboard)
        .with_context(|| format!("invalid fret notation: '{}'", args.frets))?;

    if args.verbose {
        let (lo, hi) = shape.span();
        eprintln!(
            "fretboard: frets='{}', tuning={}, strings={}, span={}-{}, playable={}",
            args.frets,
            tuning.label,
            tuning.fretboard.num_strings(),
            lo,
            hi,
            shape.is_playable()
        );
    }

    if format != OutputFormat::Svg {
        anyhow::ensure!(
            args.orientation.is_none() && args.title.is_none() && args.num_frets.is_none(),
            "--orientation, --title, and --num-frets only apply to SVG output"
        );
        let convention = if args.high_to_low {
            StringConvention::OneIndexedFromHigh
        } else {
            StringConvention::ZeroIndexedFromLow
        };
        let builder = shape
            .to_ascii()
            .show_notes(args.notes)
            .string_convention(convention);
        let text = match format {
            OutputFormat::FretSpec => builder.fret_spec(),
            OutputFormat::Positions => builder.position_list(),
            OutputFormat::Svg => unreachable!(),
        } + "\n";
        return match &args.output {
            Some(path) => {
                fs::write(path, text).with_context(|| format!("failed to write {path}"))?;
                Ok(())
            }
            None => {
                io::stdout().write_all(text.as_bytes())?;
                Ok(())
            }
        };
    }

    let theme = resolve_theme(args.theme.as_deref())?;
    let mut builder = FretboardBuilder::new().from_shape(&shape).theme(theme);
    if let Some(title) = &args.title {
        builder = builder.title(title);
    }
    if let Some(orientation) = &args.orientation {
        builder = builder.orientation(resolve_orientation(orientation)?);
    }
    if let Some(num_frets) = args.num_frets {
        builder = builder.num_frets(num_frets);
    }
    let svg = builder.build();

    match &args.output {
        Some(path) => {
            let ext = Path::new(path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("svg");
            anyhow::ensure!(
                ext == "svg",
                "SVG fretboard output requires a .svg file (got .{ext})"
            );
            fs::write(path, &svg).with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", svg.len());
            }
        }
        None => io::stdout().write_all(svg.as_bytes())?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_c_major_shape() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_fretboard.svg");
        let args = FretboardArgs {
            frets: "x-3-2-0-1-0".into(),
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            format: None,
            notes: false,
            high_to_low: false,
            tuning: "standard".into(),
            orientation: None,
            title: Some("C Major".into()),
            num_frets: None,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "output should be SVG");
        assert!(content.contains("</svg>"), "SVG should be closed");
        assert!(content.contains("C Major"), "title should appear in SVG");
        // A 6-string chord should have fret markers or circles
        assert!(
            content.contains("<circle") || content.contains("<rect"),
            "SVG should contain fret position markers"
        );
    }

    #[test]
    fn run_with_dark_theme() {
        let args = FretboardArgs {
            frets: "0-2-2-1-0-0".into(),
            output: None,
            theme: Some("dark".into()),
            format: None,
            notes: false,
            high_to_low: false,
            tuning: "standard".into(),
            orientation: None,
            title: None,
            num_frets: None,
            verbose: false,
        };
        // Should not error; output goes to stdout
        run(args).unwrap();
    }

    #[test]
    fn reject_bad_fret_notation() {
        let args = FretboardArgs {
            frets: "not-valid-frets".into(),
            output: None,
            theme: None,
            format: None,
            notes: false,
            high_to_low: false,
            tuning: "standard".into(),
            orientation: None,
            title: None,
            num_frets: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
