use anyhow::{Context, Result};
use music::fretboard::{
    Fretboard, FretboardShape, BASS_4, BASS_5, DADGAD, DROP_D, OPEN_G, STANDARD_7, STD_6STR_GTR,
};
use music::svg::{FretboardBuilder, Orientation};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::resolve_theme;

fn resolve_tuning(name: &str) -> Result<&'static Fretboard> {
    match name.to_lowercase().as_str() {
        "standard" => Ok(&STD_6STR_GTR),
        "drop-d" | "dropd" => Ok(&DROP_D),
        "dadgad" => Ok(&DADGAD),
        "open-g" | "openg" => Ok(&OPEN_G),
        "7-string" | "7string" => Ok(&STANDARD_7),
        "bass-4" | "bass4" => Ok(&BASS_4),
        "bass-5" | "bass5" => Ok(&BASS_5),
        other => anyhow::bail!(
            "unknown tuning: '{other}' (options: standard, drop-d, dadgad, open-g, 7-string, bass-4, bass-5)"
        ),
    }
}

fn resolve_orientation(name: &str) -> Result<Orientation> {
    match name.to_lowercase().as_str() {
        "vertical" | "up-down" => Ok(Orientation::Vertical),
        "horizontal" | "right-left" => Ok(Orientation::Horizontal),
        other => anyhow::bail!(
            "unknown orientation: '{other}' (options: vertical, horizontal)"
        ),
    }
}

pub struct FretboardArgs {
    pub frets: String,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub tuning: String,
    pub orientation: Option<String>,
    pub title: Option<String>,
    pub num_frets: Option<u8>,
    pub verbose: bool,
}

pub fn run(args: FretboardArgs) -> Result<()> {
    let tuning = resolve_tuning(&args.tuning)?;
    let theme = resolve_theme(args.theme.as_deref())?;

    let shape = FretboardShape::from_string(&args.frets, tuning)
        .with_context(|| format!("invalid fret notation: '{}'", args.frets))?;

    if args.verbose {
        let (lo, hi) = shape.span();
        eprintln!(
            "fretboard: frets='{}', tuning={}, strings={}, span={}-{}, playable={}",
            args.frets,
            args.tuning,
            tuning.num_strings(),
            lo,
            hi,
            shape.is_playable()
        );
    }

    let mut builder = FretboardBuilder::new()
        .from_shape(&shape)
        .theme(theme);

    if let Some(ref t) = args.title {
        builder = builder.title(t.as_str());
    }
    if let Some(ref o) = args.orientation {
        builder = builder.orientation(resolve_orientation(o)?);
    }
    if let Some(n) = args.num_frets {
        builder = builder.num_frets(n);
    }

    let svg = builder.build();

    match args.output {
        Some(ref path) => {
            let p = Path::new(path);
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("svg");
            anyhow::ensure!(
                ext == "svg",
                "fretboard only supports .svg output (got .{ext})"
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
    fn resolve_standard_tuning() {
        let fb = resolve_tuning("standard").unwrap();
        assert_eq!(fb.num_strings(), 6);
    }

    #[test]
    fn resolve_drop_d_tuning() {
        let fb = resolve_tuning("drop-d").unwrap();
        assert_eq!(fb.num_strings(), 6);
    }

    #[test]
    fn resolve_7string_tuning() {
        let fb = resolve_tuning("7-string").unwrap();
        assert_eq!(fb.num_strings(), 7);
    }

    #[test]
    fn reject_unknown_tuning() {
        assert!(resolve_tuning("ukulele").is_err());
    }

    #[test]
    fn run_c_major_shape() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_fretboard.svg");
        let args = FretboardArgs {
            frets: "x-3-2-0-1-0".into(),
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
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
            tuning: "standard".into(),
            orientation: None,
            title: None,
            num_frets: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
