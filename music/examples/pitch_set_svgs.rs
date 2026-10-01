//! Generates guitar-fretboard scale-pattern SVGs for pc-sets of varying cardinality.
//!
//! For each pc-set we run the library's melodic-shape search (3-notes-per-string
//! and 2-notes-per-string families) and emit one SVG per starting-note pattern,
//! plus the open-position shape. Additionally emits:
//!   - `all_shapes.svg`        — every tile stacked in one tall SVG.
//!   - `page_NN.svg`           — tiles packed into A4-sized SVGs, one per page.
//!     A companion shell script (`gen_fretboard_svgs.sh`) converts these to a PDF.
//!
//! Run with:
//!     cargo run --example pitch_set_svgs
//!
//! Outputs go to `target/pitch_set_svgs/`.

use std::fs;
use std::panic;
use std::path::PathBuf;

use music::fretboard::fretboard_shape::melodic_shape_search::{
    find_open_scale_shape, n_note_per_string_shape, MelodicFretboardShape, ScaleShapeSearchResult,
};
use music::fretboard::STD_6STR_GTR;
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcShape;
use music::svg::fretboard::{FretPosition, FretboardBuilder, Orientation};
use music::svg::SvgTheme;

struct Entry {
    slug: &'static str,
    title: &'static str,
    root: Note,
    pcs: Vec<Pc>,
}

/// One rendered fretboard diagram plus its dimensions, to be placed into a page.
struct Tile {
    svg_body: String,
    width: u32,
    height: u32,
}

/// Either a shape tile or a group heading inserted between cardinality groups.
enum DocItem {
    Heading(String),
    Tile(Tile),
}

// A4 at 96 DPI. Portrait.
const A4_W: u32 = 794;
const A4_H: u32 = 1123;
const PAGE_MARGIN: u32 = 40;
const TILE_V_GAP: u32 = 18;
const HEADING_HEIGHT: u32 = 44;
const HEADING_FONT: u32 = 22;
const USABLE_W: u32 = A4_W - 2 * PAGE_MARGIN;
const USABLE_H: u32 = A4_H - 2 * PAGE_MARGIN;

fn main() {
    let out_dir = PathBuf::from("target").join("pitch_set_svgs");
    fs::create_dir_all(&out_dir).expect("create output dir");
    // Remove any stale .svg files from a prior run so we never merge old pages.
    if let Ok(entries) = fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "svg") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let groups: Vec<(&str, Vec<Entry>)> = vec![
        (
            "3-note",
            vec![
                Entry {
                    slug: "major-triad",
                    title: "C Major Triad",
                    root: Note::C,
                    pcs: vec![Pc::Pc0, Pc::Pc4, Pc::Pc7],
                },
                Entry {
                    slug: "minor-triad",
                    title: "A Minor Triad",
                    root: Note::A,
                    pcs: vec![Pc::Pc0, Pc::Pc3, Pc::Pc7],
                },
                Entry {
                    slug: "sus4-triad",
                    title: "D Sus4 Triad",
                    root: Note::D,
                    pcs: vec![Pc::Pc0, Pc::Pc5, Pc::Pc7],
                },
            ],
        ),
        (
            "4-note",
            vec![
                Entry {
                    slug: "maj7-arpeggio",
                    title: "Gmaj7 Arpeggio",
                    root: Note::G,
                    pcs: vec![Pc::Pc0, Pc::Pc4, Pc::Pc7, Pc::Pc11],
                },
                Entry {
                    slug: "dom7-arpeggio",
                    title: "E7 Arpeggio",
                    root: Note::E,
                    pcs: vec![Pc::Pc0, Pc::Pc4, Pc::Pc7, Pc::Pc10],
                },
                Entry {
                    slug: "m7b5-arpeggio",
                    title: "Bm7b5 Arpeggio",
                    root: Note::B,
                    pcs: vec![Pc::Pc0, Pc::Pc3, Pc::Pc6, Pc::Pc10],
                },
            ],
        ),
        (
            "5-note",
            vec![
                Entry {
                    slug: "major-pentatonic",
                    title: "G Major Pentatonic",
                    root: Note::G,
                    pcs: vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc7, Pc::Pc9],
                },
                Entry {
                    slug: "minor-pentatonic",
                    title: "A Minor Pentatonic",
                    root: Note::A,
                    pcs: vec![Pc::Pc0, Pc::Pc3, Pc::Pc5, Pc::Pc7, Pc::Pc10],
                },
                Entry {
                    slug: "in-sen",
                    title: "E In-sen",
                    root: Note::E,
                    pcs: vec![Pc::Pc0, Pc::Pc1, Pc::Pc5, Pc::Pc7, Pc::Pc10],
                },
            ],
        ),
        (
            "6-note",
            vec![
                Entry {
                    slug: "blues",
                    title: "A Blues Scale",
                    root: Note::A,
                    pcs: vec![Pc::Pc0, Pc::Pc3, Pc::Pc5, Pc::Pc6, Pc::Pc7, Pc::Pc10],
                },
                Entry {
                    slug: "whole-tone",
                    title: "C Whole-Tone",
                    root: Note::C,
                    pcs: vec![Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc6, Pc::Pc8, Pc::Pc10],
                },
                Entry {
                    slug: "hexatonic-augmented",
                    title: "C Augmented Hexatonic",
                    root: Note::C,
                    pcs: vec![Pc::Pc0, Pc::Pc3, Pc::Pc4, Pc::Pc7, Pc::Pc8, Pc::Pc11],
                },
            ],
        ),
        (
            "7-note",
            vec![
                Entry {
                    slug: "major",
                    title: "C Major / Ionian",
                    root: Note::C,
                    pcs: vec![
                        Pc::Pc0,
                        Pc::Pc2,
                        Pc::Pc4,
                        Pc::Pc5,
                        Pc::Pc7,
                        Pc::Pc9,
                        Pc::Pc11,
                    ],
                },
                Entry {
                    slug: "dorian",
                    title: "D Dorian",
                    root: Note::D,
                    pcs: vec![
                        Pc::Pc0,
                        Pc::Pc2,
                        Pc::Pc3,
                        Pc::Pc5,
                        Pc::Pc7,
                        Pc::Pc9,
                        Pc::Pc10,
                    ],
                },
                Entry {
                    slug: "harmonic-minor",
                    title: "A Harmonic Minor",
                    root: Note::A,
                    pcs: vec![
                        Pc::Pc0,
                        Pc::Pc2,
                        Pc::Pc3,
                        Pc::Pc5,
                        Pc::Pc7,
                        Pc::Pc8,
                        Pc::Pc11,
                    ],
                },
                Entry {
                    slug: "melodic-minor",
                    title: "C Melodic Minor",
                    root: Note::C,
                    pcs: vec![
                        Pc::Pc0,
                        Pc::Pc2,
                        Pc::Pc3,
                        Pc::Pc5,
                        Pc::Pc7,
                        Pc::Pc9,
                        Pc::Pc11,
                    ],
                },
            ],
        ),
    ];

    let theme = SvgTheme::default();
    let fretboard = &*STD_6STR_GTR;
    let mut items: Vec<DocItem> = vec![];
    let mut total = 0usize;

    for (cardinality, entries) in &groups {
        items.push(DocItem::Heading((*cardinality).to_string()));
        for entry in entries {
            let pc_shape = PcShape::new(entry.pcs.clone());
            let spelled = match pc_shape.try_spell(&entry.root) {
                Ok(notes) => notes,
                Err(err) => {
                    eprintln!("skip {}: spelling failed: {:?}", entry.slug, err);
                    continue;
                }
            };

            // Open position shape.
            let spelled_open = spelled.clone();
            let open = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                find_open_scale_shape(&spelled_open, fretboard)
            }));
            if let Ok(Ok(shape)) = open {
                if !shape.shape.is_empty() {
                    let filename = format!("{}-{}-open.svg", cardinality, entry.slug);
                    let title = format!("{} — open position", entry.title);
                    let tile = render_shape(&shape, &entry.root, &title, &theme);
                    fs::write(out_dir.join(&filename), &tile.svg_body).expect("write svg");
                    println!("wrote {}", out_dir.join(&filename).display());
                    items.push(DocItem::Tile(tile));
                    total += 1;
                }
            }

            // N-notes-per-string patterns.
            for &n_cfg in &[(2usize, 2usize), (2, 3), (3, 3)] {
                let label = match n_cfg {
                    (2, 2) => "2nps",
                    (2, 3) => "2-3nps",
                    (3, 3) => "3nps",
                    _ => "nps",
                };
                for start in &spelled {
                    let spelled_for_nps = spelled.clone();
                    let start_owned = *start;
                    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        n_note_per_string_shape(n_cfg, &spelled_for_nps, &start_owned, fretboard)
                    }));
                    if let Ok(Ok(shape)) = res {
                        let filename = format!(
                            "{}-{}-{}-from-{}.svg",
                            cardinality,
                            entry.slug,
                            label,
                            note_slug(start)
                        );
                        let title = format!("{} — {} from {}", entry.title, label, start);
                        let tile = render_shape(&shape, &entry.root, &title, &theme);
                        fs::write(out_dir.join(&filename), &tile.svg_body).expect("write svg");
                        println!("wrote {}", out_dir.join(&filename).display());
                        items.push(DocItem::Tile(tile));
                        total += 1;
                    }
                }
            }

            // Simple (low-cost positional) shapes: the closest to CAGED boxes.
            let spelled_simple = spelled.clone();
            let simple_res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                ScaleShapeSearchResult::from_raw_search_result(&spelled_simple, fretboard)
            }));
            if let Ok(Ok(result)) = simple_res {
                for (idx, shape) in result.simple.iter().enumerate() {
                    let (min_fret, _) = shape.span();
                    let filename =
                        format!("{}-{}-simple-{:02}.svg", cardinality, entry.slug, idx + 1);
                    let title = format!(
                        "{} — simple shape {} (fret {}+)",
                        entry.title,
                        idx + 1,
                        min_fret
                    );
                    let tile = render_shape(shape, &entry.root, &title, &theme);
                    fs::write(out_dir.join(&filename), &tile.svg_body).expect("write svg");
                    println!("wrote {}", out_dir.join(&filename).display());
                    items.push(DocItem::Tile(tile));
                    total += 1;
                }
            }
        }
    }

    // Combined SVG: every item stacked vertically, widest tile determines width.
    write_combined_svg(&items, &out_dir.join("all_shapes.svg"));
    // Paginated A4 SVGs for PDF conversion.
    let page_count = write_pages(&items, &out_dir);

    println!(
        "\nGenerated {} tile SVGs, 1 combined SVG, {} page SVGs in {}",
        total,
        page_count,
        out_dir.display()
    );
}

fn note_slug(note: &Note) -> String {
    format!("{}", note)
        .replace('#', "s")
        .replace('b', "f")
        .to_lowercase()
}

fn render_shape(shape: &MelodicFretboardShape, root: &Note, title: &str, theme: &SvgTheme) -> Tile {
    let (min_fret, max_fret) = shape.span();
    let has_open = shape.shape.iter().any(|n| n.fret == 0);

    let (start_fret, num_frets) = if has_open || min_fret == 0 {
        (0u8, max_fret.max(5))
    } else {
        // start_fret = N means the first visible space is fret N+1. For a shape
        // whose lowest fret is 1, that's start_fret = 0 (show the nut).
        let start = min_fret.saturating_sub(1);
        let n = (max_fret - start).max(5);
        (start, n)
    };

    let root_pc = Pc::from(root);
    let mut builder = FretboardBuilder::new()
        .num_frets(num_frets)
        .start_fret(start_fret)
        .orientation(Orientation::Horizontal)
        .title(title)
        .theme(theme.clone());

    for note in &shape.shape {
        let pos = if note.fret == 0 {
            FretPosition::Open {
                string: note.string,
            }
        } else {
            FretPosition::Fretted {
                string: note.string,
                fret: note.fret,
            }
        };
        builder = builder.position(pos);
        if Pc::from(&note.pitch.note) == root_pc {
            builder = builder.root_at(note.string, note.fret);
        }
    }

    let svg_body = builder.build();
    let (width, height) = parse_svg_dims(&svg_body);
    Tile {
        svg_body,
        width,
        height,
    }
}

/// Parse `width="N"` and `height="N"` out of an `<svg ...>` root tag.
fn parse_svg_dims(svg: &str) -> (u32, u32) {
    fn find_attr(s: &str, key: &str) -> u32 {
        let needle = format!("{}=\"", key);
        let idx = s.find(&needle).expect("attr present");
        let rest = &s[idx + needle.len()..];
        let end = rest.find('"').expect("closing quote");
        rest[..end].parse().expect("numeric")
    }
    // Only look in the first 400 bytes to avoid matching later text attributes.
    let head = &svg[..svg.len().min(400)];
    (find_attr(head, "width"), find_attr(head, "height"))
}

/// Wrap an `<svg>` document body so it can be embedded as a `<g>` inside a
/// larger SVG at the given offset. Strips the outer `<svg ...>` and `</svg>`.
fn svg_inner(svg: &str) -> &str {
    let first_gt = svg.find('>').expect("svg open");
    let close = svg.rfind("</svg>").expect("svg close");
    &svg[first_gt + 1..close]
}

/// Uniform scale factor to fit `tile` within `max_w`. `1.0` if the tile already
/// fits; otherwise `max_w / tile.width` so the whole tile shrinks proportionally.
fn tile_scale(tile: &Tile, max_w: u32) -> f64 {
    if tile.width <= max_w {
        1.0
    } else {
        max_w as f64 / tile.width as f64
    }
}

/// Scaled dimensions (rounded up) of a tile when constrained to `max_w`.
fn scaled_dims(tile: &Tile, max_w: u32) -> (u32, u32) {
    let s = tile_scale(tile, max_w);
    let w = (tile.width as f64 * s).ceil() as u32;
    let h = (tile.height as f64 * s).ceil() as u32;
    (w, h)
}

fn write_combined_svg(items: &[DocItem], path: &std::path::Path) {
    let max_w = items
        .iter()
        .filter_map(|i| match i {
            DocItem::Tile(t) => Some(t.width),
            _ => None,
        })
        .max()
        .unwrap_or(600);
    let total_h: u32 = items
        .iter()
        .map(|i| match i {
            DocItem::Heading(_) => HEADING_HEIGHT + TILE_V_GAP,
            DocItem::Tile(t) => t.height + TILE_V_GAP,
        })
        .sum::<u32>()
        + PAGE_MARGIN * 2;
    let doc_w = max_w + PAGE_MARGIN * 2;

    let mut s = String::new();
    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
"#,
        w = doc_w,
        h = total_h
    ));
    s.push_str(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##);
    s.push('\n');

    let mut y = PAGE_MARGIN;
    for item in items {
        match item {
            DocItem::Heading(text) => {
                s.push_str(&format!(
                    r##"<text x="{x}" y="{y}" font-family="sans-serif" font-size="{fs}" font-weight="bold" fill="#222">{t}</text>"##,
                    x = PAGE_MARGIN,
                    y = y + HEADING_FONT,
                    fs = HEADING_FONT,
                    t = escape_xml(text)
                ));
                s.push('\n');
                y += HEADING_HEIGHT + TILE_V_GAP;
            }
            DocItem::Tile(tile) => {
                let x_offset = PAGE_MARGIN;
                s.push_str(&format!(r#"<g transform="translate({},{})">"#, x_offset, y));
                s.push_str(svg_inner(&tile.svg_body));
                s.push_str("</g>\n");
                y += tile.height + TILE_V_GAP;
            }
        }
    }
    s.push_str("</svg>\n");
    fs::write(path, s).expect("write combined svg");
    println!("wrote {}", path.display());
}

/// Pack items into A4 pages, writing `page_NN.svg` each. Returns page count.
/// Never splits a tile across pages. Avoids orphaned headings (heading at
/// bottom of page with no tile after it is deferred to the next page).
fn write_pages(items: &[DocItem], out_dir: &std::path::Path) -> usize {
    // Pre-compute item heights, using scaled height for oversized tiles.
    let item_height = |item: &DocItem| -> u32 {
        match item {
            DocItem::Heading(_) => HEADING_HEIGHT,
            DocItem::Tile(t) => scaled_dims(t, USABLE_W).1,
        }
    };

    let mut page_items: Vec<Vec<usize>> = vec![vec![]];
    let mut cur_y: u32 = 0;
    let mut i = 0;
    while i < items.len() {
        let h = item_height(&items[i]);
        let h_with_gap = if page_items.last().unwrap().is_empty() {
            h
        } else {
            h + TILE_V_GAP
        };

        // Guard: if this item is a Heading and no tile follows on this page
        // (i.e. the tile after wouldn't fit), push heading to next page.
        // We check by tentatively placing heading + next tile.
        let is_heading = matches!(&items[i], DocItem::Heading(_));

        let fits = cur_y + h_with_gap <= USABLE_H;
        if !fits {
            // Start a new page.
            page_items.push(vec![]);
            cur_y = 0;
            continue;
        }

        if is_heading && i + 1 < items.len() {
            let next_h = item_height(&items[i + 1]);
            let gap_for_next = TILE_V_GAP;
            let combined = cur_y + h_with_gap + gap_for_next + next_h;
            if combined > USABLE_H && cur_y > 0 {
                // Heading would orphan — push whole heading to next page.
                page_items.push(vec![]);
                cur_y = 0;
                continue;
            }
        }

        page_items.last_mut().unwrap().push(i);
        cur_y += h_with_gap;
        i += 1;
    }

    for (page_idx, indices) in page_items.iter().enumerate() {
        if indices.is_empty() {
            continue;
        }
        let page_path = out_dir.join(format!("page_{:02}.svg", page_idx + 1));
        let mut s = String::new();
        s.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
"#,
            w = A4_W,
            h = A4_H
        ));
        s.push_str(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##);
        s.push('\n');

        let mut y = PAGE_MARGIN;
        let mut first_on_page = true;
        for &idx in indices {
            if !first_on_page {
                y += TILE_V_GAP;
            }
            first_on_page = false;
            match &items[idx] {
                DocItem::Heading(text) => {
                    s.push_str(&format!(
                        r##"<text x="{x}" y="{y}" font-family="sans-serif" font-size="{fs}" font-weight="bold" fill="#222">{t}</text>"##,
                        x = PAGE_MARGIN,
                        y = y + HEADING_FONT,
                        fs = HEADING_FONT,
                        t = escape_xml(text)
                    ));
                    s.push('\n');
                    y += HEADING_HEIGHT;
                }
                DocItem::Tile(tile) => {
                    let scale = tile_scale(tile, USABLE_W);
                    let (scaled_w, scaled_h) = scaled_dims(tile, USABLE_W);
                    // Center horizontally within the usable width.
                    let x = PAGE_MARGIN + USABLE_W.saturating_sub(scaled_w) / 2;
                    if (scale - 1.0).abs() < f64::EPSILON {
                        s.push_str(&format!(r#"<g transform="translate({},{})">"#, x, y));
                    } else {
                        s.push_str(&format!(
                            r#"<g transform="translate({},{}) scale({:.4})">"#,
                            x, y, scale
                        ));
                    }
                    s.push_str(svg_inner(&tile.svg_body));
                    s.push_str("</g>\n");
                    y += scaled_h;
                }
            }
        }
        s.push_str("</svg>\n");
        fs::write(&page_path, s).expect("write page svg");
        println!("wrote {}", page_path.display());
    }

    page_items.iter().filter(|p| !p.is_empty()).count()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
