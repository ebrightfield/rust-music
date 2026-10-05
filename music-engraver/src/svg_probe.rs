//! Test-only parsing of the engraver's own SVG output: text runs, glyph
//! placements and lines, with numeric coordinates, so tests can assert
//! positions instead of substrings.

use smufl::Glyph;

use crate::font::MusicFont;

/// A `<text>` element.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SvgText {
    pub x: f64,
    pub y: f64,
    pub anchor: String,
    pub weight: String,
    pub style: String,
    pub size: f64,
    pub content: String,
}

/// A `<line>` element.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SvgLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub dashed: bool,
}

/// Placement of a glyph path: origin and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphPlacement {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

fn attr<'a>(element: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = element.find(&key)? + key.len();
    let end = element[start..].find('"')? + start;
    Some(&element[start..end])
}

fn num(element: &str, name: &str) -> f64 {
    attr(element, name)
        .unwrap_or_else(|| panic!("missing {name} in {element}"))
        .parse()
        .unwrap_or_else(|_| panic!("non-numeric {name} in {element}"))
}

fn elements<'a>(svg: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag} ");
    svg.match_indices(&open)
        .map(|(i, _)| {
            let rest = &svg[i..];
            let end = rest.find('\n').unwrap_or(rest.len());
            &rest[..end]
        })
        .collect()
}

/// Every `<text>` element in document order.
pub(crate) fn texts(svg: &str) -> Vec<SvgText> {
    elements(svg, "text")
        .into_iter()
        .map(|e| {
            let content_start = e.find('>').expect("text open tag") + 1;
            let content_end = e.rfind("</text>").expect("text close tag");
            SvgText {
                x: num(e, "x"),
                y: num(e, "y"),
                anchor: attr(e, "text-anchor").unwrap_or("").to_string(),
                weight: attr(e, "font-weight").unwrap_or("").to_string(),
                style: attr(e, "font-style").unwrap_or("").to_string(),
                size: num(e, "font-size"),
                content: e[content_start..content_end].to_string(),
            }
        })
        .collect()
}

/// The single `<text>` whose content is `content`.
pub(crate) fn text(svg: &str, content: &str) -> SvgText {
    let found: Vec<SvgText> = texts(svg)
        .into_iter()
        .filter(|t| t.content == content)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected one <text> {content:?}, found {found:?}"
    );
    found.into_iter().next().unwrap()
}

/// Every `<line>` element in document order.
pub(crate) fn lines(svg: &str) -> Vec<SvgLine> {
    elements(svg, "line")
        .into_iter()
        .map(|e| SvgLine {
            x1: num(e, "x1"),
            y1: num(e, "y1"),
            x2: num(e, "x2"),
            y2: num(e, "y2"),
            dashed: attr(e, "stroke-dasharray").is_some(),
        })
        .collect()
}

/// Every placement of `glyph` (identified by its exact outline path data).
pub(crate) fn glyphs(svg: &str, font: &MusicFont, glyph: Glyph) -> Vec<GlyphPlacement> {
    let data = font.glyph_outline(glyph).expect("glyph outline").path_data;
    let needle = format!("<path d=\"{data}\"");
    svg.match_indices(&needle)
        .map(|(i, _)| {
            let rest = &svg[i..];
            let element = &rest[..rest.find('\n').unwrap_or(rest.len())];
            let transform = attr(element, "transform").expect("glyph transform");
            let inner = transform
                .strip_prefix("translate(")
                .expect("translate transform");
            let (translate, scale) = match inner.split_once(") scale(") {
                Some((t, s)) => (t, s.trim_end_matches(')')),
                None => (inner.trim_end_matches(')'), "1,1"),
            };
            let mut xy = translate
                .split(',')
                .map(|v| v.trim().parse::<f64>().unwrap());
            let scale = scale.split(',').next().unwrap().trim().parse().unwrap();
            GlyphPlacement {
                x: xy.next().unwrap(),
                y: xy.next().unwrap(),
                scale,
            }
        })
        .collect()
}

/// The single placement of `glyph`.
pub(crate) fn glyph(svg: &str, font: &MusicFont, glyph: Glyph) -> GlyphPlacement {
    let found = glyphs(svg, font, glyph);
    assert_eq!(found.len(), 1, "expected one {glyph:?}, found {found:?}");
    found[0]
}

/// Assert `a` and `b` agree to within 1e-6.
#[track_caller]
pub(crate) fn assert_close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}
