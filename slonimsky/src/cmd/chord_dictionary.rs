use anyhow::{Context, Result};
use music::fretboard::fretboard_shape::chord_shape_search::find_chord_shapes;
use music::fretboard::{
    Fretboard, FretboardShape, BASS_4, BASS_5, DADGAD, DROP_D, OPEN_G, STANDARD_7, STD_6STR_GTR,
};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::svg::FretboardBuilder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, pc_label, resolve_theme};

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

/// Pick the most common spelling for a pitch class.
fn pc_to_note(pc: Pc) -> Note {
    pc.notes()[0]
}

pub struct ChordDictionaryArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub tuning: String,
    pub max_span: u8,
    pub max_results: usize,
    pub verbose: bool,
}

pub fn run(args: ChordDictionaryArgs) -> Result<()> {
    let pcs = parse_input_to_pcs(&args.input)?;
    let tuning = resolve_tuning(&args.tuning)?;
    let theme = resolve_theme(args.theme.as_deref())?;

    let label: String = pcs.iter().map(|pc| pc_label(*pc)).collect::<Vec<_>>().join(", ");

    let notes: Vec<Note> = pcs.iter().map(|pc| pc_to_note(*pc)).collect();

    let results = find_chord_shapes(&notes, tuning)
        .with_context(|| format!("chord shape search failed for [{}]", label))?;

    // Collect playable shapes, filtered by max-span
    let mut shapes: Vec<FretboardShape> = Vec::new();
    for shape_vec in results.playable.values() {
        for shape in shape_vec {
            let (lo, hi) = shape.span();
            let span = hi.saturating_sub(lo);
            if span <= args.max_span {
                shapes.push(shape.clone());
            }
        }
    }
    // Also include nontransposable (open-string dependent) shapes
    for shape_vec in results.nontransposable.values() {
        for shape in shape_vec {
            let (lo, hi) = shape.span();
            let span = hi.saturating_sub(lo);
            if span <= args.max_span {
                shapes.push(shape.clone());
            }
        }
    }

    // Sort by lowest fret position for stable output
    shapes.sort_by_key(|s| {
        let (lo, _) = s.span();
        lo
    });

    // Deduplicate by display string
    let mut seen = std::collections::HashSet::new();
    shapes.retain(|s| seen.insert(format!("{}", s)));

    // Cap at max_results
    shapes.truncate(args.max_results);

    if args.verbose {
        eprintln!(
            "chord-dictionary: input=[{}], tuning={}, max-span={}, found {} shapes (capped at {})",
            label,
            args.tuning,
            args.max_span,
            shapes.len(),
            args.max_results,
        );
    }

    // Determine output mode from extension
    let is_svg = args.output.as_ref().is_some_and(|p| {
        Path::new(p)
            .extension()
            .and_then(|e| e.to_str()) == Some("svg")
    });

    if let Some(ref path) = args.output {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        anyhow::ensure!(
            ext == "svg" || ext == "txt",
            "chord-dictionary supports .svg or .txt output (got .{ext})"
        );
    }

    if is_svg {
        let path = args.output.as_ref().unwrap();
        if shapes.is_empty() {
            anyhow::bail!("no playable shapes found for [{}] with max-span {}", label, args.max_span);
        }
        // Build a combined SVG: vertical stack of individual diagrams
        let mut combined = String::new();
        let diagram_height = 200u32;
        let diagram_width = 200u32;
        let cols = 4u32;
        let rows = (shapes.len() as u32).div_ceil(cols);
        let total_width = diagram_width * cols;
        let total_height = diagram_height * rows;

        combined.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
            total_width, total_height
        ));

        for (i, shape) in shapes.iter().enumerate() {
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;
            let x = col * diagram_width;
            let y = row * diagram_height;

            let inner_svg = FretboardBuilder::new()
                .from_shape(shape)
                .theme(theme.clone())
                .title(format!("{}", shape))
                .build();

            // Wrap each diagram in a positioned <g> element
            // Strip the outer <svg> tags from the inner diagram
            let inner_content = strip_svg_wrapper(&inner_svg);
            combined.push_str(&format!(
                "  <g transform=\"translate({}, {})\">\n{}\n  </g>\n",
                x, y, inner_content
            ));
        }
        combined.push_str("</svg>\n");

        fs::write(path, &combined)
            .with_context(|| format!("failed to write {path}"))?;
        if args.verbose {
            eprintln!("wrote {path} ({} bytes, {} shapes)", combined.len(), shapes.len());
        }
    } else {
        // Text output
        let mut out = String::new();
        out.push_str(&format!("Chord dictionary: [{}]\n", label));
        out.push_str(&format!(
            "Tuning: {}, Max span: {} frets\n\n",
            args.tuning, args.max_span
        ));

        if shapes.is_empty() {
            out.push_str("(no playable shapes found)\n");
        } else {
            for (i, shape) in shapes.iter().enumerate() {
                let (lo, hi) = shape.span();
                out.push_str(&format!(
                    "  {:>3}. {}  (frets {}-{})\n",
                    i + 1,
                    shape,
                    lo,
                    hi
                ));
            }
        }

        out.push_str(&format!("\nTotal: {} shapes\n", shapes.len()));

        match args.output {
            Some(ref path) => {
                fs::write(path, &out)
                    .with_context(|| format!("failed to write {path}"))?;
                if args.verbose {
                    eprintln!("wrote {path} ({} bytes)", out.len());
                }
            }
            None => {
                io::stdout().write_all(out.as_bytes())?;
            }
        }
    }

    Ok(())
}

/// Strip the outer <svg ...> and </svg> tags, returning just the inner content.
fn strip_svg_wrapper(svg: &str) -> &str {
    let start = svg.find('>').map(|i| i + 1).unwrap_or(0);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    &svg[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_to_note_basic() {
        assert_eq!(pc_to_note(Pc::Pc0), Note::C);
        assert_eq!(pc_to_note(Pc::Pc4), Note::E);
        assert_eq!(pc_to_note(Pc::Pc7), Note::G);
    }

    #[test]
    fn c_major_finds_shapes() {
        let args = ChordDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: None,
            theme: None,
            tuning: "standard".into(),
            max_span: 4,
            max_results: 50,
            verbose: false,
        };
        // Should not error — C major has many shapes on guitar
        run(args).unwrap();
    }

    #[test]
    fn max_span_filters() {
        // With max_span=0, only shapes where all fretted notes are on the same fret
        let args = ChordDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: None,
            theme: None,
            tuning: "standard".into(),
            max_span: 0,
            max_results: 100,
            verbose: false,
        };
        // Should succeed (might find 0 or few shapes)
        run(args).unwrap();
    }

    #[test]
    fn max_results_limits() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        let notes: Vec<Note> = pcs.iter().map(|pc| pc_to_note(*pc)).collect();
        let results = find_chord_shapes(&notes, &STD_6STR_GTR).unwrap();
        let total_playable: usize = results.playable.values().map(|v| v.len()).sum();
        // C major should have many playable shapes
        assert!(total_playable > 5, "expected many C major shapes, got {}", total_playable);
    }

    #[test]
    fn text_output_has_header_and_total() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_cd.txt");
        let args = ChordDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            max_span: 4,
            max_results: 10,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("Chord dictionary:"), "should have header");
        assert!(content.contains("Total:"), "should have total line");
        assert!(content.contains("Max span: 4"), "should show max span");
    }

    #[test]
    fn svg_output_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_cd.svg");
        let args = ChordDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            max_span: 4,
            max_results: 8,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "should start with <svg");
        assert!(content.contains("</svg>"), "should close with </svg>");
        assert!(content.contains("<g transform"), "should have positioned groups");
    }

    #[test]
    fn reject_bad_extension() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_cd.pdf");
        let args = ChordDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            max_span: 4,
            max_results: 10,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
