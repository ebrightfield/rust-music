//! Free text attached to events and barlines, and the one-line
//! text-plus-glyph composition shared by text scripts, tempo marks and
//! custom dynamics.
//!
//! A [`TextScript`] is LilyPond's `^\markup` / `_\markup` (and, attached to
//! a barline, `\textMark` / `\textEndMark`): a line of text runs and SMuFL
//! glyphs in one font style and size, placed above or below the staff.
//!
//! Text is emitted as SVG `<text>` and glyphs as outline paths, so text
//! widths are only estimated. [`layout_text_line`] confines the estimate to
//! where it cannot be avoided: a text run that touches a glyph is anchored on
//! that glyph's exact edge (`text-anchor="end"` before a glyph, `"start"`
//! after it), so joins such as "(" + note + "= 60)" are exact.

use smufl::Glyph;

use crate::font::{FontError, MusicFont};
use crate::layout::placement::Placement;

/// Typeface style of a text run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFont {
    /// Roman (upright, normal weight).
    #[default]
    Upright,
    /// Italic, normal weight.
    Italic,
    /// Bold upright.
    Bold,
    /// Bold italic.
    BoldItalic,
}

impl TextFont {
    /// SVG `font-weight` value.
    pub fn svg_weight(self) -> &'static str {
        match self {
            Self::Upright | Self::Italic => "normal",
            Self::Bold | Self::BoldItalic => "bold",
        }
    }

    /// SVG `font-style` value.
    pub fn svg_style(self) -> &'static str {
        match self {
            Self::Upright | Self::Bold => "normal",
            Self::Italic | Self::BoldItalic => "italic",
        }
    }

    fn width_factor(self) -> f64 {
        match self {
            Self::Upright => 1.0,
            Self::Italic => 0.96,
            Self::Bold => 1.06,
            Self::BoldItalic => 1.02,
        }
    }
}

/// Relative size of a text script (LilyPond `\small`, `\tiny`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextSize {
    /// Default size.
    #[default]
    Normal,
    /// One LilyPond font-size step down (`\small`, ×2^(−1/6)).
    Small,
    /// Two steps down (`\tiny`, ×2^(−2/6)).
    Tiny,
}

impl TextSize {
    /// Scale factor applied to the base font size and to inline glyphs.
    pub fn scale(self) -> f64 {
        match self {
            Self::Normal => 1.0,
            Self::Small => 2f64.powf(-1.0 / 6.0),
            Self::Tiny => 2f64.powf(-2.0 / 6.0),
        }
    }
}

/// Horizontal alignment of a text script relative to its anchor event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Left edge at the event's left edge (LilyPond's default for text
    /// scripts).
    #[default]
    Left,
    /// Centered on the event.
    Center,
    /// Right edge at the event's right edge.
    Right,
}

/// One piece of a text script: a text run or a SMuFL glyph
/// (LilyPond `\musicglyph`).
#[derive(Clone, Debug, PartialEq)]
pub enum TextItem {
    /// A run of text in the script's font style.
    Text(String),
    /// A SMuFL glyph drawn inline at the script's size.
    Glyph(Glyph),
}

/// Free text above or below the staff, attached to a note, chord, rest or
/// barline.
///
/// ```
/// use music_engraver::layout::text_script::TextScript;
/// let a_tempo = TextScript::above("a tempo").italic();
/// let label = TextScript::above("a)");
/// let gliss = TextScript::below("gliss.").italic();
/// # let _ = (a_tempo, label, gliss);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TextScript {
    /// Text runs and glyphs, left to right.
    pub items: Vec<TextItem>,
    /// Side of the staff.
    pub placement: Placement,
    /// Typeface style of every text run.
    pub font: TextFont,
    /// Relative size.
    pub size: TextSize,
    /// Horizontal alignment on the anchor.
    pub align: TextAlign,
}

impl TextScript {
    /// Upright, left-aligned text on `placement`'s side of the staff.
    pub fn new(text: impl Into<String>, placement: Placement) -> Self {
        Self {
            items: vec![TextItem::Text(text.into())],
            placement,
            font: TextFont::Upright,
            size: TextSize::Normal,
            align: TextAlign::Left,
        }
    }

    /// Upright text above the staff (LilyPond `^"text"`).
    pub fn above(text: impl Into<String>) -> Self {
        Self::new(text, Placement::Above)
    }

    /// Upright text below the staff (LilyPond `_"text"`).
    pub fn below(text: impl Into<String>) -> Self {
        Self::new(text, Placement::Below)
    }

    /// A lone SMuFL glyph (e.g. a fermata in a `\textMark`), centered.
    pub fn glyph(glyph: Glyph, placement: Placement) -> Self {
        Self {
            items: vec![TextItem::Glyph(glyph)],
            placement,
            font: TextFont::Upright,
            size: TextSize::Normal,
            align: TextAlign::Center,
        }
    }

    /// Append a text run.
    pub fn then_text(mut self, text: impl Into<String>) -> Self {
        self.items.push(TextItem::Text(text.into()));
        self
    }

    /// Append an inline SMuFL glyph.
    pub fn then_glyph(mut self, glyph: Glyph) -> Self {
        self.items.push(TextItem::Glyph(glyph));
        self
    }

    /// Set the typeface style.
    pub fn font(mut self, font: TextFont) -> Self {
        self.font = font;
        self
    }

    /// Italic text.
    pub fn italic(self) -> Self {
        self.font(TextFont::Italic)
    }

    /// Bold text.
    pub fn bold(self) -> Self {
        self.font(TextFont::Bold)
    }

    /// Set the relative size.
    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }

    /// `\small` text.
    pub fn small(self) -> Self {
        self.size(TextSize::Small)
    }

    /// `\tiny` text.
    pub fn tiny(self) -> Self {
        self.size(TextSize::Tiny)
    }

    /// Set the horizontal alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Center the script on its anchor.
    pub fn centered(self) -> Self {
        self.align(TextAlign::Center)
    }

    /// Lay the script out as one line at its own size.
    pub fn layout(&self, font: &MusicFont, staff_space: f64) -> Result<TextLineLayout, FontError> {
        let scale = self.size.scale();
        let font_size = TEXT_SCRIPT_FONT_SIZE_SS * staff_space * scale;
        let items: Vec<LineItem> = self
            .items
            .iter()
            .map(|item| match item {
                TextItem::Text(text) => LineItem::Text {
                    text: text.clone(),
                    font: self.font,
                    font_size,
                },
                TextItem::Glyph(glyph) => LineItem::Glyph {
                    glyph: *glyph,
                    scale,
                    dy: 0.0,
                },
            })
            .collect();
        layout_text_line(&items, font)
    }
}

/// Base font size of text scripts, in staff spaces.
pub const TEXT_SCRIPT_FONT_SIZE_SS: f64 = 1.4;

/// Distance from the bottom staff line down to the default baseline of a
/// below-staff text script, in staff spaces (below the dynamics band).
pub const TEXT_SCRIPT_BELOW_STAFF_SS: f64 = 4.0;

/// Clearance between the staff (or the outermost notehead/stem) and the
/// nearest edge of a text script, in staff spaces.
pub const TEXT_SCRIPT_PADDING_SS: f64 = 0.5;

/// Vertical gap between stacked marks on one side of an event, in staff
/// spaces.
pub const TEXT_SCRIPT_STACK_GAP_SS: f64 = 0.3;

/// Ascent of serif text above its baseline, as a fraction of font size.
pub const TEXT_ASCENT_RATIO: f64 = 0.70;

/// Descent of serif text below its baseline, as a fraction of font size.
pub const TEXT_DESCENT_RATIO: f64 = 0.22;

/// Width of a word space, as a fraction of font size.
const SPACE_WIDTH_EM: f64 = 0.25;

/// Estimated advance of one character of a Times-like serif face, in ems.
fn char_width_em(c: char) -> f64 {
    match c {
        ' ' | '.' | ',' | ':' | ';' | '\'' | '!' | '|' => 0.25,
        'i' | 'j' | 'l' => 0.278,
        '(' | ')' | '[' | ']' | '-' | 'f' | 't' | 'r' | 'I' | 'J' => 0.333,
        's' => 0.389,
        'm' | 'M' => 0.82,
        'w' | 'W' => 0.8,
        '0'..='9' | 'a'..='z' => 0.5,
        '=' | '+' | '<' | '>' => 0.564,
        'A'..='Z' => 0.68,
        '\u{2013}' => 0.5,
        _ => 0.5,
    }
}

/// Estimated rendered width of `text` at `font_size` in `font`.
pub fn estimate_text_width(text: &str, font_size: f64, font: TextFont) -> f64 {
    text.chars().map(char_width_em).sum::<f64>() * font_size * font.width_factor()
}

/// One item of a composed text line.
#[derive(Clone, Debug, PartialEq)]
pub enum LineItem {
    /// A text run.
    Text {
        /// The text; leading/trailing spaces become exact gaps.
        text: String,
        /// Typeface style.
        font: TextFont,
        /// Font size in font design units.
        font_size: f64,
    },
    /// A SMuFL glyph, origin on the line's baseline.
    Glyph {
        /// The glyph.
        glyph: Glyph,
        /// Scale relative to staff size.
        scale: f64,
        /// Vertical offset of the glyph origin from the baseline (SVG y-down).
        dy: f64,
    },
    /// Horizontal space, in font design units.
    Gap(f64),
}

/// A positioned item of a [`TextLineLayout`], relative to the line origin
/// (left edge, baseline).
#[derive(Clone, Debug, PartialEq)]
pub enum PlacedLineItem {
    /// A text run.
    Text {
        /// Anchor x: the run's left edge, or its right edge when
        /// `anchor_end` is set.
        x: f64,
        /// The text (trimmed).
        text: String,
        /// Typeface style.
        font: TextFont,
        /// Font size in font design units.
        font_size: f64,
        /// Whether `x` is the run's right edge (it abuts a following glyph).
        anchor_end: bool,
    },
    /// A glyph whose origin sits at (`x`, `dy`).
    Glyph {
        /// Origin x.
        x: f64,
        /// The glyph.
        glyph: Glyph,
        /// Scale relative to staff size.
        scale: f64,
        /// Vertical offset from the baseline (SVG y-down).
        dy: f64,
    },
}

/// A composed line of text runs and glyphs.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLineLayout {
    /// Items relative to the line origin.
    pub items: Vec<PlacedLineItem>,
    /// Total advance width.
    pub width: f64,
    /// Extent above the baseline (positive).
    pub ascent: f64,
    /// Extent below the baseline (positive).
    pub descent: f64,
    /// Origin x of the first glyph, if any.
    pub first_glyph_x: Option<f64>,
}

impl TextLineLayout {
    /// The x of the line's left edge so that it is aligned `align` on an
    /// anchor spanning `left..left + width`.
    pub fn left_for(&self, align: TextAlign, left: f64, width: f64) -> f64 {
        match align {
            TextAlign::Left => left,
            TextAlign::Center => left + (width - self.width) / 2.0,
            TextAlign::Right => left + width - self.width,
        }
    }
}

fn split_edge_spaces(text: &str, font_size: f64) -> (f64, &str, f64) {
    let space = SPACE_WIDTH_EM * font_size;
    let trimmed_start = text.trim_start();
    let leading = (text.len() - trimmed_start.len()) as f64 * space;
    let trimmed = trimmed_start.trim_end();
    let trailing = (trimmed_start.len() - trimmed.len()) as f64 * space;
    (leading, trimmed, trailing)
}

/// Compose `items` left to right from x = 0 on a common baseline.
///
/// Glyph advances and heights come from the font; text widths are
/// estimated with [`estimate_text_width`]. A text run whose nearest
/// non-gap neighbour on the right is a glyph (and on the left is not) is
/// anchored by its right edge, so the join is exact.
pub fn layout_text_line(items: &[LineItem], font: &MusicFont) -> Result<TextLineLayout, FontError> {
    // Normalize: text edge spaces become gaps so SVG whitespace collapsing
    // cannot eat them.
    let mut normalized: Vec<LineItem> = Vec::with_capacity(items.len());
    for item in items {
        match item {
            LineItem::Text {
                text,
                font: tf,
                font_size,
            } => {
                let (lead, body, trail) = split_edge_spaces(text, *font_size);
                if lead > 0.0 {
                    normalized.push(LineItem::Gap(lead));
                }
                if !body.is_empty() {
                    normalized.push(LineItem::Text {
                        text: body.to_string(),
                        font: *tf,
                        font_size: *font_size,
                    });
                }
                if trail > 0.0 {
                    normalized.push(LineItem::Gap(trail));
                }
            }
            other => normalized.push(other.clone()),
        }
    }

    let is_glyph = |i: Option<&LineItem>| matches!(i, Some(LineItem::Glyph { .. }));
    let prev_solid = |i: usize| {
        normalized[..i]
            .iter()
            .rev()
            .find(|it| !matches!(it, LineItem::Gap(_)))
    };
    let next_solid = |i: usize| {
        normalized[i + 1..]
            .iter()
            .find(|it| !matches!(it, LineItem::Gap(_)))
    };

    let mut placed = Vec::with_capacity(normalized.len());
    let mut x = 0.0_f64;
    let mut ascent = 0.0_f64;
    let mut descent = 0.0_f64;
    let mut first_glyph_x = None;
    for (i, item) in normalized.iter().enumerate() {
        match item {
            LineItem::Text {
                text,
                font: tf,
                font_size,
            } => {
                let width = estimate_text_width(text, *font_size, *tf);
                let anchor_end = is_glyph(next_solid(i)) && !is_glyph(prev_solid(i));
                placed.push(PlacedLineItem::Text {
                    x: if anchor_end { x + width } else { x },
                    text: text.clone(),
                    font: *tf,
                    font_size: *font_size,
                    anchor_end,
                });
                ascent = ascent.max(TEXT_ASCENT_RATIO * font_size);
                descent = descent.max(TEXT_DESCENT_RATIO * font_size);
                x += width;
            }
            LineItem::Glyph { glyph, scale, dy } => {
                let advance = f64::from(font.glyph_advance(*glyph)?) * scale;
                if let Some(bbox) = font.glyph_bbox_design_units(*glyph) {
                    ascent = ascent.max(-(bbox.y_top * scale + dy));
                    descent = descent.max(bbox.y_bottom * scale + dy);
                }
                first_glyph_x.get_or_insert(x);
                placed.push(PlacedLineItem::Glyph {
                    x,
                    glyph: *glyph,
                    scale: *scale,
                    dy: *dy,
                });
                x += advance;
            }
            LineItem::Gap(width) => x += width,
        }
    }

    Ok(TextLineLayout {
        items: placed,
        width: x,
        ascent,
        descent,
        first_glyph_x,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn text(t: &str) -> LineItem {
        LineItem::Text {
            text: t.into(),
            font: TextFont::Bold,
            font_size: 400.0,
        }
    }

    fn glyph(g: Glyph) -> LineItem {
        LineItem::Glyph {
            glyph: g,
            scale: 1.0,
            dy: 0.0,
        }
    }

    #[test]
    fn text_before_glyph_is_end_anchored_on_the_glyph_origin() {
        let font = bravura_font();
        let line = layout_text_line(
            &[text("("), glyph(Glyph::MetNoteQuarterUp), text(" = 60)")],
            &font,
        )
        .unwrap();
        let PlacedLineItem::Text {
            x: paren_x,
            anchor_end,
            ..
        } = &line.items[0]
        else {
            panic!("first item must be text: {:?}", line.items);
        };
        let PlacedLineItem::Glyph { x: glyph_x, .. } = &line.items[1] else {
            panic!("second item must be the glyph: {:?}", line.items);
        };
        assert!(*anchor_end);
        assert_eq!(
            paren_x, glyph_x,
            "\"(\" must end exactly where the note starts"
        );
        // The leading space of " = 60)" becomes an exact gap after the glyph.
        let advance = f64::from(font.glyph_advance(Glyph::MetNoteQuarterUp).unwrap());
        let PlacedLineItem::Text {
            x: eq_x,
            text,
            anchor_end,
            ..
        } = &line.items[2]
        else {
            panic!("third item must be text: {:?}", line.items);
        };
        assert!(!anchor_end);
        assert_eq!(text, "= 60)");
        assert!((eq_x - (glyph_x + advance + SPACE_WIDTH_EM * 400.0)).abs() < 1e-9);
    }

    #[test]
    fn script_size_scales_font_and_glyphs() {
        let font = bravura_font();
        let normal = TextScript::above("8").layout(&font, 250.0).unwrap();
        let tiny = TextScript::above("8").tiny().layout(&font, 250.0).unwrap();
        let PlacedLineItem::Text { font_size: n, .. } = &normal.items[0] else {
            panic!()
        };
        let PlacedLineItem::Text { font_size: t, .. } = &tiny.items[0] else {
            panic!()
        };
        assert!((n - TEXT_SCRIPT_FONT_SIZE_SS * 250.0).abs() < 1e-9);
        assert!((t / n - 2f64.powf(-2.0 / 6.0)).abs() < 1e-9);
    }

    #[test]
    fn alignment_offsets_against_anchor() {
        let font = bravura_font();
        let line = TextScript::above("a tempo").layout(&font, 250.0).unwrap();
        assert_eq!(line.left_for(TextAlign::Left, 100.0, 300.0), 100.0);
        assert!(
            (line.left_for(TextAlign::Center, 100.0, 300.0) - (250.0 - line.width / 2.0)).abs()
                < 1e-9
        );
        assert!(
            (line.left_for(TextAlign::Right, 100.0, 300.0) - (400.0 - line.width)).abs() < 1e-9
        );
    }
}
