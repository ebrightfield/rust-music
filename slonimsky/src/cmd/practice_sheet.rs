use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::{ChordNameDisplayConfig, MajNotation};
use music::note_collections::geometry::symmetry::transpositional::Modes;
use music::note_collections::pc_set::{AsPcSlice, PcContent, PcShape};
use music::note_collections::OctavePartition;
use music::svg::PitchCircleBuilder;
use musical_combinatorics::seven_note_scales::SevenNoteScaleQuality;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, pc_label, resolve_theme};
use super::scale_book::{
    HARMONIC_MAJOR_MODE_NAMES, HARMONIC_MINOR_MODE_NAMES, MAJOR_MODE_NAMES,
    MELODIC_MINOR_MODE_NAMES,
};

pub struct PracticeSheetArgs {
    pub key: Option<String>,
    pub scale: Option<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub verbose: bool,
}

struct ScaleInfo {
    quality: SevenNoteScaleQuality,
    mode_names: &'static [&'static str],
    label: &'static str,
}

/// Bundled data for rendering a practice sheet (avoids too-many-arguments).
struct SheetData {
    scale_pcs: PcContent,
    modes: Vec<PcShape>,
    mode_names: &'static [&'static str],
    key: Pc,
    scale_label: &'static str,
    iv: Vec<usize>,
    triads: Vec<(Vec<Pc>, Option<String>)>,
    sevenths: Vec<(Vec<Pc>, Option<String>)>,
}

fn resolve_scale(name: &str) -> Result<ScaleInfo> {
    match name.to_lowercase().replace('-', " ").as_str() {
        "major" | "ionian" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::Major,
            mode_names: MAJOR_MODE_NAMES,
            label: "Major",
        }),
        "melodic minor" | "melodic_minor" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::MelodicMinor,
            mode_names: MELODIC_MINOR_MODE_NAMES,
            label: "Melodic Minor",
        }),
        "harmonic minor" | "harmonic_minor" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::HarmonicMinor,
            mode_names: HARMONIC_MINOR_MODE_NAMES,
            label: "Harmonic Minor",
        }),
        "harmonic major" | "harmonic_major" => Ok(ScaleInfo {
            quality: SevenNoteScaleQuality::HarmonicMajor,
            mode_names: HARMONIC_MAJOR_MODE_NAMES,
            label: "Harmonic Major",
        }),
        _ => anyhow::bail!(
            "unknown scale: '{name}' (options: major, melodic-minor, harmonic-minor, harmonic-major)"
        ),
    }
}

fn resolve_key(key_str: Option<&str>) -> Result<Pc> {
    match key_str {
        None | Some("C") | Some("c") => Ok(Pc::Pc0),
        Some(s) => {
            let pcs = parse_input_to_pcs(&[s.to_string()])?;
            anyhow::ensure!(
                pcs.len() == 1,
                "key must be a single note, got {}",
                pcs.len()
            );
            Ok(pcs[0])
        }
    }
}

/// Transpose a slice of pitch classes by semitones.
fn transpose_pcs(pcs: &[Pc], semitones: u8) -> Vec<Pc> {
    pcs.iter()
        .map(|&pc| Pc::from((u8::from(pc) + semitones) % 12))
        .collect()
}

/// Rotate a sorted slice of PCs so the entry matching `root` comes first,
/// preserving the ascending cyclic order. If `root` is not in the slice,
/// returns the original order unchanged.
fn rotate_to_root(pcs: &[Pc], root: Pc) -> Vec<Pc> {
    if let Some(pos) = pcs.iter().position(|&pc| pc == root) {
        let mut rotated = Vec::with_capacity(pcs.len());
        rotated.extend_from_slice(&pcs[pos..]);
        rotated.extend_from_slice(&pcs[..pos]);
        rotated
    } else {
        pcs.to_vec()
    }
}

/// Transpose a set of Pcs so that `root` becomes Pc0.
fn normalize_to_root(pcs: &[Pc], root: Pc) -> HashSet<Pc> {
    let offset = 12 - u8::from(&root);
    pcs.iter()
        .map(|pc| Pc::from((u8::from(pc) + offset) % 12))
        .collect()
}

/// Try to name a subset given as raw Pcs, treating the first Pc as root.
fn try_name_subset(pcs: &[Pc]) -> Option<String> {
    if pcs.is_empty() {
        return None;
    }
    let root = pcs[0];
    let normalized = normalize_to_root(pcs, root);

    let display_cfg = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        utf8_accidentals: true,
        ..Default::default()
    };

    match infer_chord_quality(&normalized) {
        Some((_heuristic, Some(quality))) => {
            let quality_str = quality.to_string(&display_cfg);
            Some(format!("{}{}", pc_label(root), quality_str))
        }
        _ => None,
    }
}

/// Compute every k-subset of `pcs`, preserving the input order. This mirrors
/// the behaviour of `itertools::combinations` and lets us reason about
/// subchords in their absolute (key-rooted) form without zero-anchoring.
fn combinations<T: Clone>(pcs: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    if k > pcs.len() {
        return out;
    }
    for (i, head) in pcs.iter().enumerate() {
        for mut tail in combinations(&pcs[i + 1..], k - 1) {
            tail.insert(0, head.clone());
            out.push(tail);
        }
    }
    out
}

/// Get subchords of a given size from a pitch-class set, with optional names.
///
/// `pcs` carries the absolute (un-zeroed) pitch classes for the scale in the
/// requested key. We enumerate size-`size` subsets directly on the absolute
/// PCs so the returned subsets retain their key spellings (e.g. the I chord
/// in G major returns [G, B, D] rather than [0, 4, 7]).
fn get_named_subchords(pcs: &PcContent, size: u8) -> Vec<(Vec<Pc>, Option<String>)> {
    // Match the old `get_subchords` constraints: size must be at least 3
    // and strictly less than the cardinality of the set.
    if size < 3 || (size as usize) >= pcs.len() {
        return Vec::new();
    }
    combinations(pcs.as_pc_slice(), size as usize)
        .into_iter()
        .map(|sub| {
            let name = try_name_subset(&sub);
            (sub, name)
        })
        .collect()
}

pub fn run(args: PracticeSheetArgs) -> Result<()> {
    let key = resolve_key(args.key.as_deref())?;
    let scale_name = args.scale.as_deref().unwrap_or("major");
    let scale_info = resolve_scale(scale_name)?;
    let theme = resolve_theme(args.theme.as_deref())?;

    // Build the scale pitch-class set in the requested key
    let parent_partition = OctavePartition::from(&scale_info.quality);
    let parent_pcs_c = PcShape::from(&parent_partition);
    let transposed = transpose_pcs(parent_pcs_c.as_pc_slice(), u8::from(key));
    let scale_pcs = PcContent::new(transposed.clone());

    // Get all modes (relative to C), we'll transpose for display
    let modes = parent_pcs_c.modes();

    // Compute interval vector (reduced) — IntervalMatrix is a shape-level
    // computation, so derive the shape from the key-transposed content.
    let scale_shape = scale_pcs.to_shape();
    let matrix = music::note_collections::geometry::IntervalMatrix::new(&scale_shape);
    let iv = matrix.reduced_interval_vector();

    // Get triads and seventh chords in the scale
    let triads = get_named_subchords(&scale_pcs, 3);
    let sevenths = get_named_subchords(&scale_pcs, 4);

    let data = SheetData {
        scale_pcs,
        modes,
        mode_names: scale_info.mode_names,
        key,
        scale_label: scale_info.label,
        iv: iv.to_vec(),
        triads,
        sevenths,
    };

    if args.verbose {
        eprintln!(
            "practice-sheet: key={}, scale={}, pcs={:?}, modes={}, triads={}, sevenths={}",
            pc_label(data.key),
            data.scale_label,
            data.scale_pcs,
            data.modes.len(),
            data.triads.len(),
            data.sevenths.len(),
        );
    }

    match args.output {
        Some(ref path) => {
            let p = Path::new(path);
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("svg");
            anyhow::ensure!(
                ext == "svg",
                "practice-sheet only supports .svg output (got .{ext})"
            );
            let svg = build_svg(&data, &theme);
            fs::write(p, &svg).with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", svg.len());
            }
        }
        None => {
            print_text_report(&data)?;
        }
    }

    Ok(())
}

fn print_text_report(data: &SheetData) -> Result<()> {
    let mut out = io::stdout().lock();
    let key_label = pc_label(data.key);

    writeln!(out, "Practice Sheet: {} {}", key_label, data.scale_label)?;
    writeln!(out, "{}", "=".repeat(60))?;

    // Section 1: Scale overview (rotated to start from key root)
    writeln!(out)?;
    writeln!(out, "SCALE")?;
    writeln!(out, "-----")?;
    let rooted = rotate_to_root(data.scale_pcs.as_pc_slice(), data.key);
    let note_labels: Vec<&str> = rooted.iter().map(|&pc| pc_label(pc)).collect();
    writeln!(out, "  Notes: {}", note_labels.join(" "))?;
    let pc_ints: Vec<String> = rooted.iter().map(|&pc| u8::from(pc).to_string()).collect();
    writeln!(out, "  PCs:   {{{}}}", pc_ints.join(", "))?;
    writeln!(
        out,
        "  Interval Vector: <{}>",
        data.iv
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )?;

    // Section 2: Modes
    writeln!(out)?;
    writeln!(out, "MODES")?;
    writeln!(out, "-----")?;
    for (i, mode) in data.modes.iter().enumerate() {
        let name = data.mode_names.get(i).unwrap_or(&"?");
        let transposed = transpose_pcs(mode.as_pc_slice(), u8::from(data.key));
        let labels: Vec<&str> = transposed.iter().map(|&pc| pc_label(pc)).collect();
        writeln!(out, "  {:2}. {:20} {}", i + 1, name, labels.join(" "))?;
    }

    // Section 3: All 3-note subchords of the scale
    writeln!(out)?;
    writeln!(out, "3-NOTE SUBCHORDS ({} total)", data.triads.len())?;
    writeln!(out, "----------------")?;
    for (i, (sub, name)) in data.triads.iter().enumerate() {
        let labels: Vec<&str> = sub.iter().map(|&pc| pc_label(pc)).collect();
        let name_str = name.as_deref().unwrap_or("?");
        writeln!(out, "  {:2}. {:12} {}", i + 1, name_str, labels.join(" "))?;
    }

    // Section 4: All 4-note subchords of the scale
    writeln!(out)?;
    writeln!(out, "4-NOTE SUBCHORDS ({} total)", data.sevenths.len())?;
    writeln!(out, "----------------")?;
    for (i, (sub, name)) in data.sevenths.iter().enumerate() {
        let labels: Vec<&str> = sub.iter().map(|&pc| pc_label(pc)).collect();
        let name_str = name.as_deref().unwrap_or("?");
        writeln!(out, "  {:2}. {:12} {}", i + 1, name_str, labels.join(" "))?;
    }

    // Section 5: Practice suggestions
    writeln!(out)?;
    writeln!(out, "PRACTICE SUGGESTIONS")?;
    writeln!(out, "--------------------")?;
    writeln!(
        out,
        "  1. Play the scale ascending and descending across all strings"
    )?;
    writeln!(out, "  2. Play each mode starting from its root degree")?;
    writeln!(out, "  3. Arpeggiate each diatonic triad through the scale")?;
    writeln!(
        out,
        "  4. Arpeggiate each diatonic seventh chord through the scale"
    )?;
    writeln!(out, "  5. Play the ii-V-I progression in this key:")?;

    // Compute ii-V-I suggestion
    let degree_2 = (u8::from(data.key) + 2) % 12; // whole step up
    let degree_5 = (u8::from(data.key) + 7) % 12; // perfect 5th up
    writeln!(
        out,
        "     ii:  {} minor  →  V:  {} dom7  →  I:  {} major",
        pc_label(Pc::from(degree_2)),
        pc_label(Pc::from(degree_5)),
        key_label,
    )?;

    Ok(())
}

fn build_svg(data: &SheetData, theme: &music::svg::SvgTheme) -> String {
    let key_label = pc_label(data.key);
    let scale_label = data.scale_label;
    let width = 800;
    let height = 600 + (data.triads.len() + data.sevenths.len()) * 18 + data.modes.len() * 18;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<style>
  text {{ font-family: sans-serif; }}
  .title {{ font-size: 20px; font-weight: bold; }}
  .section {{ font-size: 14px; font-weight: bold; }}
  .body {{ font-size: 12px; }}
  .small {{ font-size: 10px; }}
</style>
<rect width="{width}" height="{height}" fill="{}"/>
"#,
        if theme.background_color == "#FFFFFF" {
            "#FAFAFA"
        } else {
            &theme.background_color
        },
    );

    let text_color = &theme.text_color;

    // Title
    svg.push_str(&format!(
        r#"<text x="400" y="30" text-anchor="middle" class="title" fill="{text_color}">{key_label} {scale_label} — Practice Sheet</text>
"#,
    ));

    // Pitch circle (embedded as scaled group, left side)
    let circle_svg = PitchCircleBuilder::new()
        .pitches(data.scale_pcs.iter().cloned())
        .root(data.key)
        .show_intervals(true)
        .theme(theme.clone())
        .title(format!("{key_label} {scale_label}"))
        .build();
    svg.push_str(&format!(
        r#"<g transform="translate(20,50) scale(0.55)">{circle_svg}</g>
"#,
    ));

    // Text sections on the right side
    let rx = 340; // right column x
    let mut y = 80;

    // Scale notes (rotated to start from key root)
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="section" fill="{text_color}">Scale Notes</text>
"#,
    ));
    y += 20;
    let rooted = rotate_to_root(data.scale_pcs.as_pc_slice(), data.key);
    let note_labels: Vec<&str> = rooted.iter().map(|&pc| pc_label(pc)).collect();
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="body" fill="{text_color}">{}</text>
"#,
        note_labels.join("  "),
    ));
    y += 20;

    // Interval vector
    let iv_str = data
        .iv
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="body" fill="{text_color}">Interval Vector: &lt;{iv_str}&gt;</text>
"#,
    ));
    y += 30;

    // Modes
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="section" fill="{text_color}">Modes</text>
"#,
    ));
    y += 18;
    for (i, mode) in data.modes.iter().enumerate() {
        let name = data.mode_names.get(i).unwrap_or(&"?");
        let transposed = transpose_pcs(mode.as_pc_slice(), u8::from(data.key));
        let labels: Vec<&str> = transposed.iter().map(|&pc| pc_label(pc)).collect();
        svg.push_str(&format!(
            r#"<text x="{rx}" y="{y}" class="small" fill="{text_color}">{}. {} — {}</text>
"#,
            i + 1,
            name,
            labels.join(" "),
        ));
        y += 16;
    }
    y += 10;

    // 3-note subchords
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="section" fill="{text_color}">3-Note Subchords</text>
"#,
    ));
    y += 18;
    for (sub, name) in &data.triads {
        let labels: Vec<&str> = sub.iter().map(|&pc| pc_label(pc)).collect();
        let name_str = name.as_deref().unwrap_or("?");
        svg.push_str(&format!(
            r#"<text x="{rx}" y="{y}" class="small" fill="{text_color}">{:12} {}</text>
"#,
            name_str,
            labels.join(" "),
        ));
        y += 16;
    }
    y += 10;

    // 4-note subchords
    svg.push_str(&format!(
        r#"<text x="{rx}" y="{y}" class="section" fill="{text_color}">4-Note Subchords</text>
"#,
    ));
    y += 18;
    for (sub, name) in &data.sevenths {
        let labels: Vec<&str> = sub.iter().map(|&pc| pc_label(pc)).collect();
        let name_str = name.as_deref().unwrap_or("?");
        svg.push_str(&format!(
            r#"<text x="{rx}" y="{y}" class="small" fill="{text_color}">{:12} {}</text>
"#,
            name_str,
            labels.join(" "),
        ));
        y += 16;
    }

    svg.push_str("</svg>\n");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_key_default() {
        assert_eq!(resolve_key(None).unwrap(), Pc::Pc0);
    }

    #[test]
    fn resolve_key_named() {
        assert_eq!(resolve_key(Some("G")).unwrap(), Pc::Pc7);
        assert_eq!(resolve_key(Some("F#")).unwrap(), Pc::Pc6);
    }

    #[test]
    fn resolve_scale_works() {
        let info = resolve_scale("major").unwrap();
        assert_eq!(info.label, "Major");
        assert_eq!(info.mode_names.len(), 7);
    }

    #[test]
    fn text_report_produces_output() {
        let args = PracticeSheetArgs {
            key: Some("C".into()),
            scale: Some("major".into()),
            output: None,
            theme: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn text_report_g_melodic_minor() {
        let args = PracticeSheetArgs {
            key: Some("G".into()),
            scale: Some("melodic-minor".into()),
            output: None,
            theme: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn svg_output_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test.svg");
        let args = PracticeSheetArgs {
            key: Some("C".into()),
            scale: Some("major".into()),
            output: Some(out.to_string_lossy().into_owned()),
            theme: None,
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"), "output should be SVG");
        assert!(content.contains("</svg>"), "SVG should be closed");
        assert!(content.contains("Practice Sheet"), "should contain title");
        assert!(
            content.contains("Scale Notes"),
            "should contain scale section"
        );
        assert!(content.contains("Modes"), "should contain modes section");
        assert!(
            content.contains("3-Note Subchords"),
            "should contain triads"
        );
        assert!(
            content.contains("4-Note Subchords"),
            "should contain sevenths"
        );
    }

    #[test]
    fn svg_dark_theme() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("test_dark.svg");
        let args = PracticeSheetArgs {
            key: Some("D".into()),
            scale: Some("harmonic-minor".into()),
            output: Some(out.to_string_lossy().into_owned()),
            theme: Some("dark".into()),
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg"));
        assert!(content.contains("D Harmonic Minor"));
    }

    #[test]
    fn rejects_unknown_scale() {
        let args = PracticeSheetArgs {
            key: Some("C".into()),
            scale: Some("pentatonic".into()),
            output: None,
            theme: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }

    #[test]
    fn rotate_to_root_works() {
        let pcs = vec![
            Pc::Pc0,
            Pc::Pc2,
            Pc::Pc4,
            Pc::Pc5,
            Pc::Pc7,
            Pc::Pc9,
            Pc::Pc11,
        ];
        // G major should start from G (Pc7)
        let rotated = rotate_to_root(&pcs, Pc::Pc7);
        assert_eq!(rotated[0], Pc::Pc7, "should start from G");
        assert_eq!(rotated.len(), 7);
        assert_eq!(
            rotated,
            vec![
                Pc::Pc7,
                Pc::Pc9,
                Pc::Pc11,
                Pc::Pc0,
                Pc::Pc2,
                Pc::Pc4,
                Pc::Pc5
            ]
        );
    }

    #[test]
    fn rotate_to_root_c_unchanged() {
        let pcs = vec![
            Pc::Pc0,
            Pc::Pc2,
            Pc::Pc4,
            Pc::Pc5,
            Pc::Pc7,
            Pc::Pc9,
            Pc::Pc11,
        ];
        let rotated = rotate_to_root(&pcs, Pc::Pc0);
        assert_eq!(rotated, pcs, "C root should be unchanged");
    }

    #[test]
    fn triad_naming_works() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        let name = try_name_subset(&pcs);
        assert!(name.is_some());
        assert!(name.unwrap().contains("Maj"));
    }
}
