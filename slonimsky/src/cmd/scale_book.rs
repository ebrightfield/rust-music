use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::{AsPcSlice, PcShape};
use music::note_collections::geometry::symmetry::transpositional::Modes;
use music::svg::PitchCircleBuilder;
use musical_combinatorics::seven_note_scales::SevenNoteScaleQuality;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_pc, pc_label, resolve_theme};

pub struct ScaleBookArgs {
    pub scale: String,
    pub keys: Option<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub verbose: bool,
}

/// Names for the 7 modes of the major scale.
pub const MAJOR_MODE_NAMES: &[&str] = &[
    "Ionian", "Dorian", "Phrygian", "Lydian",
    "Mixolydian", "Aeolian", "Locrian",
];

/// Names for the 7 modes of melodic minor.
pub const MELODIC_MINOR_MODE_NAMES: &[&str] = &[
    "Melodic Minor", "Dorian b2", "Lydian Augmented",
    "Lydian Dominant", "Mixolydian b6", "Locrian #2",
    "Altered",
];

/// Names for the 7 modes of harmonic minor.
pub const HARMONIC_MINOR_MODE_NAMES: &[&str] = &[
    "Harmonic Minor", "Locrian #6", "Ionian Augmented",
    "Dorian #4", "Phrygian Dominant", "Lydian #2",
    "Ultralocrian",
];

/// Names for the 7 modes of harmonic major.
pub const HARMONIC_MAJOR_MODE_NAMES: &[&str] = &[
    "Harmonic Major", "Dorian b5", "Phrygian b4",
    "Lydian b3", "Mixolydian b2", "Lydian Augmented #2",
    "Locrian bb7",
];

struct ScaleInfo {
    quality: SevenNoteScaleQuality,
    mode_names: &'static [&'static str],
}

fn resolve_scale(name: &str) -> Result<ScaleInfo> {
    match name.to_lowercase().replace('-', " ").as_str() {
        "major" | "ionian" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::Major,
            mode_names: MAJOR_MODE_NAMES,
        }),
        "melodic minor" | "melodic_minor" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::MelodicMinor,
            mode_names: MELODIC_MINOR_MODE_NAMES,
        }),
        "harmonic minor" | "harmonic_minor" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::HarmonicMinor,
            mode_names: HARMONIC_MINOR_MODE_NAMES,
        }),
        "harmonic major" | "harmonic_major" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::HarmonicMajor,
            mode_names: HARMONIC_MAJOR_MODE_NAMES,
        }),
        _ => anyhow::bail!(
            "unknown scale: '{name}' (options: major, melodic-minor, harmonic-minor, harmonic-major)"
        ),
    }
}

fn resolve_keys(keys_str: Option<&str>) -> Result<Vec<Pc>> {
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

/// Transpose a PcSet by the given number of semitones (preserving cardinality).
fn transpose_pcs(pcs: &[Pc], semitones: u8) -> Vec<Pc> {
    pcs.iter()
        .map(|&pc| Pc::from((u8::from(pc) + semitones) % 12))
        .collect()
}

pub fn run(args: ScaleBookArgs) -> Result<()> {
    let scale_info = resolve_scale(&args.scale)?;
    let keys = resolve_keys(args.keys.as_deref())?;
    let theme = resolve_theme(args.theme.as_deref())?;

    // Get parent scale PcShape (rooted at C = Pc0)
    let parent_partition = music::note_collections::OctavePartition::from(&scale_info.quality);
    let parent_pcs = PcShape::from(&parent_partition);
    let modes = parent_pcs.modes();

    if args.verbose {
        eprintln!(
            "scale-book: scale={}, modes={}, keys={}, theme={}",
            args.scale,
            modes.len(),
            keys.len(),
            args.theme.as_deref().unwrap_or("default")
        );
    }

    match args.output {
        Some(ref path) => {
            let p = Path::new(path);
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("svg");
            anyhow::ensure!(
                ext == "svg",
                "scale-book only supports .svg output (got .{ext})"
            );
            let svg = build_grid_svg(&modes, scale_info.mode_names, &keys, &theme, &args.scale);
            fs::write(p, &svg)
                .with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", svg.len());
            }
        }
        None => {
            // Text mode: print mode info for each key
            print_text_report(&modes, scale_info.mode_names, &keys, &args.scale)?;
        }
    }

    Ok(())
}

fn print_text_report(
    modes: &[PcShape],
    mode_names: &[&str],
    keys: &[Pc],
    scale_name: &str,
) -> Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "Scale Book: {} ({} modes × {} keys = {} entries)",
        scale_name, modes.len(), keys.len(), modes.len() * keys.len())?;
    writeln!(out, "{}", "=".repeat(60))?;

    for (mode_idx, mode) in modes.iter().enumerate() {
        let name = mode_names.get(mode_idx).unwrap_or(&"?");
        writeln!(out)?;
        writeln!(out, "Mode {}: {}", mode_idx + 1, name)?;
        writeln!(out, "  Parent intervals: {mode}")?;
        writeln!(out, "{}", "-".repeat(40))?;

        for &key in keys {
            let key_label = pc_label(key);
            let transposed = transpose_pcs(mode.as_pc_slice(), u8::from(key));
            let labels: Vec<&str> = transposed.iter().map(|&pc| pc_label(pc)).collect();
            writeln!(out, "  {key_label:>5} {name}: {}", labels.join(" "))?;
        }
    }

    writeln!(out)?;
    writeln!(out, "Total: {} entries", modes.len() * keys.len())?;
    Ok(())
}

fn build_grid_svg(
    modes: &[PcShape],
    mode_names: &[&str],
    keys: &[Pc],
    theme: &music::svg::SvgTheme,
    scale_name: &str,
) -> String {
    let cell_w = 220;
    let cell_h = 230;
    let cols = keys.len();
    let rows = modes.len();
    let header_h = 40;
    let total_w = cols * cell_w + 80; // extra left margin for row labels
    let total_h = rows * cell_h + header_h + 20;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}">
<style>
  text {{ font-family: sans-serif; }}
  .title {{ font-size: 16px; font-weight: bold; }}
  .header {{ font-size: 12px; font-weight: bold; }}
  .label {{ font-size: 10px; }}
</style>
<text x="{}" y="20" class="title">{scale_name} — Scale Book ({rows} modes × {cols} keys)</text>
"#,
        total_w / 2,
    );

    // Column headers (key labels)
    for (col, &key) in keys.iter().enumerate() {
        let x = 80 + col * cell_w + cell_w / 2;
        svg.push_str(&format!(
            r#"<text x="{x}" y="{}" text-anchor="middle" class="header">{}</text>
"#,
            header_h - 5,
            pc_label(key),
        ));
    }

    // Grid of pitch circles
    for (row, mode) in modes.iter().enumerate() {
        let name = mode_names.get(row).unwrap_or(&"?");
        let y_base = header_h + row * cell_h;

        // Row label
        svg.push_str(&format!(
            r#"<text x="5" y="{}" class="label">{name}</text>
"#,
            y_base + cell_h / 2,
        ));

        for (col, &key) in keys.iter().enumerate() {
            let x_base = 80 + col * cell_w;
            let transposed = transpose_pcs(mode.as_pc_slice(), u8::from(key));

            let title = format!("{} {name}", pc_label(key));
            let cell_svg = PitchCircleBuilder::new()
                .pitches(transposed.iter().cloned())
                .root(key)
                .show_intervals(true)
                .theme(theme.clone())
                .title(&title)
                .build();

            // Embed as a scaled group
            svg.push_str(&format!(
                r#"<g transform="translate({x_base},{y_base}) scale(0.45)">{cell_svg}</g>
"#,
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_scale_major() {
        let info = resolve_scale("major").unwrap();
        assert_eq!(info.mode_names.len(), 7);
        assert_eq!(info.mode_names[0], "Ionian");
        assert_eq!(info.mode_names[6], "Locrian");
    }

    #[test]
    fn resolve_scale_aliases() {
        assert!(resolve_scale("melodic-minor").is_ok());
        assert!(resolve_scale("harmonic-minor").is_ok());
        assert!(resolve_scale("harmonic-major").is_ok());
        assert!(resolve_scale("ionian").is_ok());
    }

    #[test]
    fn resolve_scale_rejects_unknown() {
        assert!(resolve_scale("pentatonic").is_err());
    }

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
        assert_eq!(keys[2], Pc::Pc2);
    }

    #[test]
    fn transpose_pcs_wraps() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let t = transpose_pcs(&pcs, 7); // G major
        assert_eq!(t, vec![Pc::Pc7, Pc::Pc11, Pc::Pc2]);
    }

    #[test]
    fn major_modes_count() {
        let info = resolve_scale("major").unwrap();
        let partition = music::note_collections::OctavePartition::from(&info.quality);
        let parent = PcShape::from(&partition);
        let modes = parent.modes();
        assert_eq!(modes.len(), 7);
    }

    #[test]
    fn text_report_contains_modes() {
        let args = ScaleBookArgs {
            scale: "major".into(),
            keys: Some("C".into()),
            output: None,
            theme: None,
            verbose: false,
        };
        // Capture output by running the function — just verify no panic
        run(args).unwrap();
    }

    #[test]
    fn svg_output_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test.svg");
        let args = ScaleBookArgs {
            scale: "major".into(),
            keys: Some("C,G".into()),
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "output should be SVG");
        assert!(content.contains("</svg>"), "SVG should be closed");
        assert!(content.contains("Ionian"), "should contain mode name");
        assert!(content.contains("Locrian"), "should contain last mode");
        assert!(content.contains("Scale Book"), "should contain title");
    }
}
