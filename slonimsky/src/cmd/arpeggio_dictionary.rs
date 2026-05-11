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

/// Transpose a set of PCs by the given semitone offset.
fn transpose(pcs: &[Pc], semitones: u8) -> Vec<Pc> {
    pcs.iter()
        .map(|&pc| Pc::from((u8::from(pc) + semitones) % 12))
        .collect()
}

pub struct ArpeggioDictionaryArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub tuning: String,
    pub keys: Option<String>,
    pub positions: usize,
    pub max_span: u8,
    pub verbose: bool,
}

/// Resolve a comma-separated key list or default to all 12.
fn resolve_keys(keys_str: Option<&str>) -> Result<Vec<Pc>> {
    use super::input::parse_pc;
    match keys_str {
        None | Some("all") => Ok((0..12u8).map(Pc::from).collect()),
        Some(s) => {
            let mut keys = Vec::new();
            for part in s.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let pc = parse_pc(part)
                    .with_context(|| format!("invalid key: '{part}'"))?;
                keys.push(pc);
            }
            anyhow::ensure!(!keys.is_empty(), "no keys specified");
            Ok(keys)
        }
    }
}

/// Find arpeggio shapes for a PcSet on a fretboard, returning up to `positions`
/// shapes sorted by lowest fret and deduplicated.
fn find_arpeggio_shapes<'a>(
    pcs: &[Pc],
    fretboard: &'a Fretboard,
    max_span: u8,
    positions: usize,
) -> Result<Vec<FretboardShape<'a>>> {
    let notes: Vec<Note> = pcs.iter().map(|pc| pc_to_note(*pc)).collect();
    let results = find_chord_shapes(&notes, fretboard)
        .with_context(|| "chord shape search failed")?;

    let mut shapes: Vec<FretboardShape> = Vec::new();

    for shape_vec in results.playable.values() {
        for shape in shape_vec {
            let (lo, hi) = shape.span();
            let span = hi.saturating_sub(lo);
            if span <= max_span {
                shapes.push(shape.clone());
            }
        }
    }
    for shape_vec in results.nontransposable.values() {
        for shape in shape_vec {
            let (lo, hi) = shape.span();
            let span = hi.saturating_sub(lo);
            if span <= max_span {
                shapes.push(shape.clone());
            }
        }
    }

    shapes.sort_by_key(|s| s.span().0);

    let mut seen = std::collections::HashSet::new();
    shapes.retain(|s| seen.insert(format!("{}", s)));

    shapes.truncate(positions);
    Ok(shapes)
}

pub fn run(args: ArpeggioDictionaryArgs) -> Result<()> {
    let base_pcs = parse_input_to_pcs(&args.input)?;
    let keys = resolve_keys(args.keys.as_deref())?;
    let tuning = resolve_tuning(&args.tuning)?;
    let theme = resolve_theme(args.theme.as_deref())?;

    let base_label: String = base_pcs.iter().map(|pc| pc_label(*pc)).collect::<Vec<_>>().join(", ");

    if args.verbose {
        eprintln!(
            "arpeggio-dictionary: chord=[{}], keys={}, positions={}, tuning={}, max-span={}",
            base_label,
            keys.len(),
            args.positions,
            args.tuning,
            args.max_span,
        );
    }

    // Validate output extension
    if let Some(ref path) = args.output {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        anyhow::ensure!(
            ext == "svg" || ext == "txt",
            "arpeggio-dictionary supports .svg or .txt output (got .{ext})"
        );
    }

    let is_svg = args.output.as_ref().is_some_and(|p| {
        Path::new(p)
            .extension()
            .and_then(|e| e.to_str()) == Some("svg")
    });

    // Compute arpeggio shapes per key
    let mut per_key: Vec<(Pc, Vec<FretboardShape<'_>>)> = Vec::new();
    let mut total_shapes = 0usize;

    for &key in &keys {
        let offset = u8::from(key);
        let transposed = transpose(&base_pcs, offset);
        let shapes = find_arpeggio_shapes(&transposed, tuning, args.max_span, args.positions)?;
        total_shapes += shapes.len();
        per_key.push((key, shapes));
    }

    if is_svg {
        let path = args.output.as_ref().unwrap();
        let svg = build_grid_svg(&per_key, &base_label, args.positions, &theme);
        fs::write(path, &svg)
            .with_context(|| format!("failed to write {path}"))?;
        if args.verbose {
            eprintln!("wrote {path} ({} bytes, {} keys × up to {} positions)", svg.len(), keys.len(), args.positions);
        }
    } else {
        let text = build_text_report(&per_key, &base_label, &args.tuning, args.max_span, total_shapes);
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
    }

    Ok(())
}

fn build_text_report(
    per_key: &[(Pc, Vec<FretboardShape<'_>>)],
    base_label: &str,
    tuning: &str,
    max_span: u8,
    total_shapes: usize,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Arpeggio Dictionary: [{}]\nTuning: {}, Max span: {} frets\n",
        base_label, tuning, max_span,
    ));
    out.push_str(&format!(
        "{} keys, {} total shapes\n",
        per_key.len(),
        total_shapes,
    ));
    out.push_str(&format!("{}\n", "=".repeat(60)));

    for (key, shapes) in per_key {
        let key_name = pc_label(*key);
        out.push_str(&format!("\nKey: {key_name}\n"));
        out.push_str(&format!("{}\n", "-".repeat(40)));

        if shapes.is_empty() {
            out.push_str("  (no shapes found)\n");
        } else {
            for (i, shape) in shapes.iter().enumerate() {
                let (lo, hi) = shape.span();
                out.push_str(&format!(
                    "  {:>3}. {}  (frets {}-{})\n",
                    i + 1,
                    shape,
                    lo,
                    hi,
                ));
            }
        }
    }

    out.push_str(&format!("\nTotal: {} shapes across {} keys\n", total_shapes, per_key.len()));
    out
}

fn build_grid_svg(
    per_key: &[(Pc, Vec<FretboardShape<'_>>)],
    base_label: &str,
    max_positions: usize,
    theme: &music::svg::SvgTheme,
) -> String {
    let cell_w = 200u32;
    let cell_h = 200u32;
    let header_h = 40u32;
    let row_label_w = 60u32;
    let cols = max_positions.max(1) as u32;
    let rows = per_key.len() as u32;
    let total_w = row_label_w + cols * cell_w;
    let total_h = header_h + rows * cell_h;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}">
<style>
  text {{ font-family: sans-serif; }}
  .title {{ font-size: 14px; font-weight: bold; }}
  .header {{ font-size: 11px; font-weight: bold; }}
  .label {{ font-size: 11px; font-weight: bold; }}
</style>
<text x="{}" y="20" class="title">Arpeggio Dictionary: [{base_label}] ({rows} keys × {cols} positions)</text>
"#,
        total_w / 2,
    );

    // Column headers
    for col in 0..cols {
        let x = row_label_w + col * cell_w + cell_w / 2;
        svg.push_str(&format!(
            r#"<text x="{x}" y="{}" text-anchor="middle" class="header">Pos {}</text>
"#,
            header_h - 5,
            col + 1,
        ));
    }

    // Rows: one per key
    for (row, (key, shapes)) in per_key.iter().enumerate() {
        let y_base = header_h + (row as u32) * cell_h;

        // Row label
        svg.push_str(&format!(
            r#"<text x="5" y="{}" class="label">{}</text>
"#,
            y_base + cell_h / 2,
            pc_label(*key),
        ));

        for (col, shape) in shapes.iter().enumerate() {
            let x_base = row_label_w + (col as u32) * cell_w;

            let inner_svg = FretboardBuilder::new()
                .from_shape(shape)
                .theme(theme.clone())
                .title(format!("{}", shape))
                .build();

            let inner_content = strip_svg_wrapper(&inner_svg);
            svg.push_str(&format!(
                r#"  <g transform="translate({x_base},{y_base}) scale(0.4)">{inner_content}</g>
"#,
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

fn strip_svg_wrapper(svg: &str) -> &str {
    let start = svg.find('>').map(|i| i + 1).unwrap_or(0);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    &svg[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_keys_all() {
        let keys = resolve_keys(None).unwrap();
        assert_eq!(keys.len(), 12);
    }

    #[test]
    fn resolve_keys_subset() {
        let keys = resolve_keys(Some("C,G,D")).unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], Pc::Pc0);
        assert_eq!(keys[1], Pc::Pc7);
    }

    #[test]
    fn transpose_basic() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let t = transpose(&pcs, 7);
        assert_eq!(t, vec![Pc::Pc7, Pc::Pc11, Pc::Pc2]);
    }

    #[test]
    fn find_shapes_for_c_major() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let shapes = find_arpeggio_shapes(&pcs, &STD_6STR_GTR, 4, 5).unwrap();
        assert!(!shapes.is_empty(), "C major should have arpeggio shapes");
        assert!(shapes.len() <= 5, "should cap at 5 positions");
    }

    #[test]
    fn text_output_has_keys_and_total() {
        let args = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: None,
            theme: None,
            tuning: "standard".into(),
            keys: Some("C,G".into()),
            positions: 3,
            max_span: 4,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn single_key_produces_output() {
        let dir = std::env::temp_dir().join("slonimsky_test_arp");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("test_arp.txt");
        let args = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            keys: Some("C".into()),
            positions: 5,
            max_span: 4,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("Arpeggio Dictionary:"), "should have header");
        assert!(content.contains("Key: C"), "should have key header");
        assert!(content.contains("Total:"), "should have total line");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn svg_output_valid() {
        let dir = std::env::temp_dir().join("slonimsky_test_arp");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("test_arp.svg");
        let args = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            keys: Some("C,G".into()),
            positions: 3,
            max_span: 4,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "should start with <svg");
        assert!(content.contains("</svg>"), "should close with </svg>");
        assert!(content.contains("Arpeggio Dictionary:"), "should have title");
        assert!(content.contains("<g transform"), "should have positioned groups");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn reject_bad_extension() {
        let dir = std::env::temp_dir().join("slonimsky_test_arp");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("test_arp.pdf");
        let args = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            keys: Some("C".into()),
            positions: 3,
            max_span: 4,
            verbose: false,
        };
        assert!(run(args).is_err());
    }

    #[test]
    fn multiple_keys_multiply_shapes() {
        let dir = std::env::temp_dir().join("slonimsky_test_arp");
        let _ = std::fs::create_dir_all(&dir);
        let out1 = dir.join("test_arp_1key.txt");
        let out3 = dir.join("test_arp_3keys.txt");
        let args1 = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out1.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            keys: Some("C".into()),
            positions: 5,
            max_span: 4,
            verbose: false,
        };
        let args3 = ArpeggioDictionaryArgs {
            input: vec!["C".into(), "E".into(), "G".into()],
            output: Some(out3.to_string_lossy().into_owned()),
            theme: None,
            tuning: "standard".into(),
            keys: Some("C,G,D".into()),
            positions: 5,
            max_span: 4,
            verbose: false,
        };
        run(args1).unwrap();
        run(args3).unwrap();
        let content1 = std::fs::read_to_string(&out1).unwrap();
        let content3 = std::fs::read_to_string(&out3).unwrap();
        // 3-key report should be longer than 1-key
        assert!(content3.len() > content1.len(), "3-key report should be larger");
        // 3-key should mention all 3 keys
        assert!(content3.contains("Key: C"));
        assert!(content3.contains("Key: G"));
        assert!(content3.contains("Key: D"));
        let _ = std::fs::remove_file(&out1);
        let _ = std::fs::remove_file(&out3);
    }
}
