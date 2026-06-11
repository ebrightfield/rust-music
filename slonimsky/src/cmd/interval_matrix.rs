use anyhow::{Context, Result};
use music::note_collections::geometry::IntervalMatrix;
use music::note_collections::pc_set::{PcContent, PcShape};
use music::svg::IntervalBuilder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, resolve_theme};

pub struct IntervalMatrixArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub title: Option<String>,
    pub verbose: bool,
    pub full: bool,
}

pub fn run(args: IntervalMatrixArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = PcContent::new(pcs);

    let output_is_svg = args
        .output
        .as_ref()
        .map(|p| {
            Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("txt")
        })
        .unwrap_or("txt");

    match output_is_svg {
        "svg" => write_svg(&pc_set, &args),
        "txt" => write_text(&pc_set, &args),
        _ if args.output.is_none() => write_text(&pc_set, &args),
        ext => anyhow::bail!(
            "interval-matrix supports .svg and .txt output (got .{ext})"
        ),
    }
}

fn write_svg(pc_set: &PcContent, args: &IntervalMatrixArgs) -> Result<()> {
    let theme = resolve_theme(args.theme.as_deref())?;

    let pc_shape: PcShape = pc_set.to_shape();
    let mut builder = IntervalBuilder::new().from_pc_shape(&pc_shape).theme(theme);

    if let Some(ref t) = args.title {
        builder = builder.title(t.as_str());
    }

    let svg = builder.build_matrix();

    match args.output {
        Some(ref path) => {
            fs::write(path, &svg)
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

fn write_text(pc_set: &PcContent, args: &IntervalMatrixArgs) -> Result<()> {
    let pc_shape: PcShape = pc_set.to_shape();
    let matrix = IntervalMatrix::new(&pc_shape);
    let dim = matrix.dimension();
    let pcs_display: Vec<String> = pc_set
        .iter()
        .map(|pc| format!("{:>2}", u8::from(*pc)))
        .collect();

    let mut lines = Vec::new();

    if let Some(ref t) = args.title {
        lines.push(t.clone());
        lines.push(String::new());
    }

    // Header row
    let header = format!("     {}", pcs_display.join("  "));
    lines.push(header);
    lines.push(format!("    {}", "----".repeat(dim)));

    // Matrix rows
    for (row, label) in pcs_display.iter().enumerate() {
        let mut cells = Vec::new();
        for col in 0..dim {
            if let Some(ic) = matrix.get(row, col) {
                cells.push(format!("{:>3}", { let val: u8 = ic.into(); val }));
            }
        }
        lines.push(format!("{} |{}", label, cells.join(" ")));
    }

    // Interval vector summary
    lines.push(String::new());
    if args.full {
        let iv = matrix.interval_vector();
        let iv_str: Vec<String> = iv.iter().map(|n| n.to_string()).collect();
        lines.push(format!("Full interval vector: [{}]", iv_str.join(", ")));
    } else {
        let riv = matrix.reduced_interval_vector();
        let riv_str: Vec<String> = riv.iter().map(|n| n.to_string()).collect();
        lines.push(format!("Interval vector: <{}>", riv_str.join(", ")));
    }

    let text = lines.join("\n") + "\n";

    match args.output {
        Some(ref path) => {
            fs::write(path, &text)
                .with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", text.len());
            }
        }
        None => {
            io::stdout().write_all(text.as_bytes())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::pitch_class::Pc;

    #[test]
    fn text_output_contains_interval_vector() {
        let args = IntervalMatrixArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: None,
            theme: None,
            title: None,
            verbose: false,
            full: false,
        };
        // Capture by writing to file
        let dir = std::env::temp_dir().join("slonimsky_test_im");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("matrix.txt");
        let args = IntervalMatrixArgs {
            output: Some(out.to_string_lossy().into_owned()),
            ..args
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        // C major triad {0,4,7}: ic3 (from 0→4=4→ic4… actually:
        // pairs: (0,4)=4→ic4, (0,7)=7→ic5, (4,7)=3→ic3; each counted in both directions
        assert!(
            content.contains("Interval vector: <"),
            "expected reduced interval vector in output, got:\n{content}"
        );
        // Should contain 6 comma-separated values
        let iv_line = content.lines().find(|l| l.contains("Interval vector:")).unwrap();
        let inner = iv_line.split('<').nth(1).unwrap().split('>').next().unwrap();
        let vals: Vec<usize> = inner.split(',').map(|s| s.trim().parse().unwrap()).collect();
        assert_eq!(vals.len(), 6, "reduced IV should have 6 elements");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn text_output_with_title() {
        let dir = std::env::temp_dir().join("slonimsky_test_im2");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("titled.txt");
        let args = IntervalMatrixArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            title: Some("C Major Triad".into()),
            verbose: false,
            full: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("C Major Triad"));
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn svg_output_valid() {
        let dir = std::env::temp_dir().join("slonimsky_test_im3");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("matrix.svg");
        let args = IntervalMatrixArgs {
            input: vec!["0".into(), "3".into(), "6".into(), "9".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            title: Some("Dim7".into()),
            verbose: false,
            full: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "should be valid SVG");
        assert!(content.contains("</svg>"), "SVG should close");
        // Matrix has 4x4 = 16 cells; SVG should have rect or text elements
        assert!(
            content.contains("<text") || content.contains("<rect"),
            "SVG should contain matrix elements"
        );
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn full_vector_mode() {
        let dir = std::env::temp_dir().join("slonimsky_test_im4");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("full.txt");
        let args = IntervalMatrixArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            title: None,
            verbose: false,
            full: true,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(
            content.contains("Full interval vector:"),
            "should show full vector"
        );
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn matrix_dimensions_match_input() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let pc_set = PcContent::new(pcs);
        let pc_shape = pc_set.to_shape();
        let matrix = IntervalMatrix::new(&pc_shape);
        assert_eq!(matrix.dimension(), 3);
        // Diagonal should be 0 (unison)
        let v0: u8 = matrix.get(0, 0).unwrap().into();
        let v1: u8 = matrix.get(1, 1).unwrap().into();
        assert_eq!(v0, 0);
        assert_eq!(v1, 0);
    }
}
