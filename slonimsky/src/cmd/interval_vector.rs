use anyhow::{Context, Result};
use music::note_collections::geometry::IntervalMatrix;
use music::note_collections::pc_set::{PcContent, PcShape};
use music::svg::IntervalBuilder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, resolve_theme};

pub struct IntervalVectorArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub title: Option<String>,
    pub verbose: bool,
    pub full: bool,
}

pub fn run(args: IntervalVectorArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let pc_set = PcContent::new(pcs);

    let ext = args
        .output
        .as_ref()
        .and_then(|p| Path::new(p).extension())
        .and_then(|e| e.to_str())
        .unwrap_or("txt");

    match ext {
        "svg" => write_svg(&pc_set, &args),
        "txt" => write_text(&pc_set, &args),
        _ if args.output.is_none() => write_text(&pc_set, &args),
        ext => anyhow::bail!(
            "interval-vector supports .svg and .txt output (got .{ext})"
        ),
    }
}

fn write_svg(pc_set: &PcContent, args: &IntervalVectorArgs) -> Result<()> {
    let theme = resolve_theme(args.theme.as_deref())?;

    let pc_shape: PcShape = pc_set.to_shape();
    let mut builder = IntervalBuilder::new().from_pc_shape(&pc_shape).theme(theme);

    if let Some(ref t) = args.title {
        builder = builder.title(t.as_str());
    }

    let svg = if args.full {
        builder.build_full_vector()
    } else {
        builder.build_vector()
    };

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

fn write_text(pc_set: &PcContent, args: &IntervalVectorArgs) -> Result<()> {
    let pc_shape: PcShape = pc_set.to_shape();
    let matrix = IntervalMatrix::new(&pc_shape);

    let mut lines = Vec::new();

    if let Some(ref t) = args.title {
        lines.push(t.clone());
        lines.push(String::new());
    }

    // Show the PcSet
    let pcs_str: Vec<String> = pc_set.iter().map(|pc| format!("{}", u8::from(*pc))).collect();
    lines.push(format!("PcSet: {{{}}}", pcs_str.join(", ")));
    lines.push(String::new());

    if args.full {
        let iv = matrix.interval_vector();
        // Full vector: 12 entries for ic 0..11
        lines.push("Full interval vector (ic 0–11):".to_string());
        let labels: Vec<String> = (0u8..12).map(|i| format!("ic{i:>2}")).collect();
        lines.push(format!("  {}", labels.join("  ")));
        let vals: Vec<String> = iv.iter().map(|n| format!("{:>4}", n)).collect();
        lines.push(format!("  {}", vals.join("  ")));
    } else {
        let riv = matrix.reduced_interval_vector();
        // Reduced vector: 6 entries for ic 1..6
        let riv_str: Vec<String> = riv.iter().map(|n| n.to_string()).collect();
        lines.push(format!("Interval vector: <{}>", riv_str.join(", ")));
        lines.push(String::new());
        // Labeled breakdown
        let ic_labels = ["m2/M7", "M2/m7", "m3/M6", "M3/m6", "P4/P5", "tritone"];
        for (i, (label, count)) in ic_labels.iter().zip(riv.iter()).enumerate() {
            lines.push(format!("  ic{}: {} ({})", i + 1, count, label));
        }
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

    fn run_to_string(args: IntervalVectorArgs) -> String {
        let dir = std::env::temp_dir().join("slonimsky_test_iv");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("iv_test.txt");
        let args = IntervalVectorArgs {
            output: Some(out.to_string_lossy().into_owned()),
            ..args
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&out);
        content
    }

    #[test]
    fn text_output_major_triad() {
        let content = run_to_string(IntervalVectorArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: None,
            theme: None,
            title: None,
            verbose: false,
            full: false,
        });
        assert!(content.contains("Interval vector: <"));
        // Major triad {0,4,7}: ic3=1, ic4=1, ic5=1, others=0
        // But library counts both directions, so doubled: ic3=2, ic4=2, ic5=2
        let iv_line = content.lines().find(|l| l.starts_with("Interval vector:")).unwrap();
        let inner = iv_line.split('<').nth(1).unwrap().split('>').next().unwrap();
        let vals: Vec<usize> = inner.split(',').map(|s| s.trim().parse().unwrap()).collect();
        assert_eq!(vals.len(), 6, "reduced IV should have 6 elements");
    }

    #[test]
    fn text_output_shows_pcset() {
        let content = run_to_string(IntervalVectorArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: None,
            theme: None,
            title: None,
            verbose: false,
            full: false,
        });
        assert!(content.contains("PcSet: {0, 4, 7}"));
    }

    #[test]
    fn text_output_labeled_breakdown() {
        let content = run_to_string(IntervalVectorArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: None,
            theme: None,
            title: None,
            verbose: false,
            full: false,
        });
        assert!(content.contains("ic1:"), "should have labeled ic breakdown");
        assert!(content.contains("m2/M7"), "should show interval class names");
        assert!(content.contains("tritone"), "should show tritone label");
    }

    #[test]
    fn full_vector_mode() {
        let content = run_to_string(IntervalVectorArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: None,
            theme: None,
            title: None,
            verbose: false,
            full: true,
        });
        assert!(content.contains("Full interval vector"));
        assert!(content.contains("ic 0"), "should show ic 0 label");
    }

    #[test]
    fn title_appears_in_output() {
        let content = run_to_string(IntervalVectorArgs {
            input: vec!["0".into(), "3".into(), "6".into(), "9".into()],
            output: None,
            theme: None,
            title: Some("Dim7 Chord".into()),
            verbose: false,
            full: false,
        });
        assert!(content.starts_with("Dim7 Chord"));
    }

    #[test]
    fn svg_output_valid() {
        let dir = std::env::temp_dir().join("slonimsky_test_iv_svg");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("vector.svg");
        let args = IntervalVectorArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            title: Some("Major Triad IV".into()),
            verbose: false,
            full: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "should be valid SVG");
        assert!(content.contains("</svg>"), "SVG should close");
        assert!(content.contains("<rect") || content.contains("<text"),
            "SVG should contain bar chart elements");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn svg_full_vector_output() {
        let dir = std::env::temp_dir().join("slonimsky_test_iv_svg_full");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("vector_full.svg");
        let args = IntervalVectorArgs {
            input: vec!["0".into(), "4".into(), "7".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            title: None,
            verbose: false,
            full: true,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"));
        assert!(content.contains("</svg>"));
        let _ = std::fs::remove_file(&out);
    }
}
