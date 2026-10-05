//! Tempo marks above the staff: tempo words, metronome marks and
//! note = note equations.
//!
//! A [`TempoMark`] composes up to three lines' worth of content on the
//! LilyPond model of `\tempo \markup { ... }`:
//!
//! - an optional text before the metronome ("Allegro", "Vals"),
//! - an optional [`MetronomeMark`]: a note value, "=", then a BPM, a BPM
//!   range ("116-112"), or a second note value (a metric-modulation
//!   equation "♩ = ♩"); optionally approximate ("c. 60") and parenthesized,
//! - optional text after it on the same line ("( Agitato)", "Monodia"),
//! - an optional text stacked below it ("Alla gavotta").
//!
//! Text is bold by default (LilyPond's tempo font); note values are SMuFL
//! `metNote*` glyphs scaled to the text, with `metAugmentationDot` dots.

use smufl::Glyph;

use crate::font::{FontError, MusicFont};
use crate::layout::staff::StaffLayout;
use crate::layout::text_script::{layout_text_line, LineItem, TextFont, TextLineLayout};

/// The note value printed in a metronome mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetronomeNoteKind {
    /// Whole note.
    Whole,
    /// Half note.
    Half,
    /// Quarter note.
    Quarter,
    /// Eighth note.
    Eighth,
    /// Sixteenth note.
    Sixteenth,
    /// Thirty-second note.
    ThirtySecond,
}

impl MetronomeNoteKind {
    /// SMuFL metronome glyph (stem up) for this note value.
    pub fn notehead_glyph(self) -> Glyph {
        match self {
            Self::Whole => Glyph::MetNoteWhole,
            Self::Half => Glyph::MetNoteHalfUp,
            Self::Quarter => Glyph::MetNoteQuarterUp,
            Self::Eighth => Glyph::MetNote8thUp,
            Self::Sixteenth => Glyph::MetNote16thUp,
            Self::ThirtySecond => Glyph::MetNote32ndUp,
        }
    }
}

/// A (possibly dotted) note value in a metronome mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetronomeUnit {
    /// The note value.
    pub kind: MetronomeNoteKind,
    /// Augmentation dots (0 for an undotted value).
    pub dots: u8,
}

impl MetronomeUnit {
    /// An undotted note value.
    pub fn new(kind: MetronomeNoteKind) -> Self {
        Self { kind, dots: 0 }
    }

    /// A single-dotted note value ("♩.").
    pub fn dotted(kind: MetronomeNoteKind) -> Self {
        Self { kind, dots: 1 }
    }
}

impl From<MetronomeNoteKind> for MetronomeUnit {
    fn from(kind: MetronomeNoteKind) -> Self {
        Self::new(kind)
    }
}

/// The right-hand side of a metronome mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetronomeValue {
    /// Beats per minute ("= 120").
    Bpm(u16),
    /// A range of beats per minute, printed as written ("= 116-112").
    Range(u16, u16),
    /// Another note value: a metric-modulation equation ("♩ = ♩.").
    Unit(MetronomeUnit),
}

/// A metronome mark: `unit = value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetronomeMark {
    /// The note value on the left of "=".
    pub unit: MetronomeUnit,
    /// The right-hand side.
    pub value: MetronomeValue,
    /// Prefix a BPM value with "c." (circa).
    pub approximate: bool,
    /// Enclose the whole mark in parentheses.
    pub parenthesized: bool,
}

impl MetronomeMark {
    /// `unit = bpm`.
    pub fn bpm(unit: impl Into<MetronomeUnit>, bpm: u16) -> Self {
        Self {
            unit: unit.into(),
            value: MetronomeValue::Bpm(bpm),
            approximate: false,
            parenthesized: false,
        }
    }

    /// `unit = from-to`.
    pub fn range(unit: impl Into<MetronomeUnit>, from: u16, to: u16) -> Self {
        Self {
            value: MetronomeValue::Range(from, to),
            ..Self::bpm(unit, from)
        }
    }

    /// `left = right` (metric modulation).
    pub fn equation(left: impl Into<MetronomeUnit>, right: impl Into<MetronomeUnit>) -> Self {
        Self {
            unit: left.into(),
            value: MetronomeValue::Unit(right.into()),
            approximate: false,
            parenthesized: false,
        }
    }

    /// Mark the BPM as approximate ("c. 60").
    pub fn approx(mut self) -> Self {
        self.approximate = true;
        self
    }

    /// Enclose the mark in parentheses ("(♩ = 60)").
    pub fn parenthesized(mut self) -> Self {
        self.parenthesized = true;
        self
    }
}

/// A run of tempo text with its typeface style (bold by default).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TempoText {
    /// The words.
    pub text: String,
    /// Typeface style.
    pub font: TextFont,
}

impl TempoText {
    /// Bold tempo text (LilyPond's default tempo font).
    pub fn bold(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: TextFont::Bold,
        }
    }

    /// Roman tempo text (LilyPond `\normal-text`).
    pub fn upright(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: TextFont::Upright,
        }
    }
}

impl From<&str> for TempoText {
    fn from(text: &str) -> Self {
        Self::bold(text)
    }
}

impl From<String> for TempoText {
    fn from(text: String) -> Self {
        Self::bold(text)
    }
}

/// A tempo marking: words, a metronome mark, or both.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TempoMark {
    /// Text before the metronome mark on its line ("Allegro").
    pub text: Option<TempoText>,
    /// The metronome mark or note = note equation.
    pub metronome: Option<MetronomeMark>,
    /// Text after the metronome mark on its line ("Monodia").
    pub text_after: Option<TempoText>,
    /// Text on a line of its own below ("Alla gavotta").
    pub text_below: Option<TempoText>,
}

impl TempoMark {
    /// Words only ("Allegro").
    pub fn text(text: impl Into<TempoText>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    /// A metronome mark only.
    pub fn metronome(mark: MetronomeMark) -> Self {
        Self {
            metronome: Some(mark),
            ..Self::default()
        }
    }

    /// Set the text before the metronome mark.
    pub fn with_text(mut self, text: impl Into<TempoText>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the text after the metronome mark.
    pub fn with_text_after(mut self, text: impl Into<TempoText>) -> Self {
        self.text_after = Some(text.into());
        self
    }

    /// Set the text stacked below.
    pub fn with_text_below(mut self, text: impl Into<TempoText>) -> Self {
        self.text_below = Some(text.into());
        self
    }
}

/// One positioned line of a tempo mark.
#[derive(Debug, Clone, PartialEq)]
pub struct TempoLine {
    /// The composed line.
    pub line: TextLineLayout,
    /// Absolute baseline y.
    pub baseline_y: f64,
}

/// Result of laying out a tempo marking.
#[derive(Debug, Clone, PartialEq)]
pub struct TempoMarkLayout {
    /// Left edge of every line.
    pub x_left: f64,
    /// Lines from top to bottom.
    pub lines: Vec<TempoLine>,
}

impl TempoMarkLayout {
    /// Topmost y reached by the mark (SVG y-down: the smallest y).
    pub fn top_y(&self) -> f64 {
        self.lines
            .iter()
            .map(|l| l.baseline_y - l.line.ascent)
            .fold(f64::INFINITY, f64::min)
    }
}

/// Distance above the top staff line of the bottom line's baseline, in
/// staff spaces.
pub const TEMPO_ABOVE_STAFF_SS: f64 = 2.8;

/// Font size for tempo text, in staff spaces.
pub const TEMPO_FONT_SIZE_SS: f64 = 1.6;

/// Scale of metronome note glyphs relative to staff size (LilyPond sets
/// tempo notes `\fontsize #-2` and smaller).
pub const METRONOME_NOTE_SCALE: f64 = 0.7;

/// Gap between a note glyph and its augmentation dot, in staff spaces.
const METRONOME_DOT_GAP_SS: f64 = 0.15;

/// Gap between tempo words and the metronome mark, in staff spaces.
pub const TEXT_METRONOME_GAP_SS: f64 = 0.5;

/// Line spacing of stacked tempo text, as a multiple of the font size.
const TEMPO_LINE_SPACING: f64 = 1.2;

/// Default baseline of a tempo mark's bottom line above `staff`.
pub fn tempo_baseline(staff: &StaffLayout, staff_space: f64) -> f64 {
    staff.y_of(8) - TEMPO_ABOVE_STAFF_SS * staff_space
}

fn text_item(text: &TempoText, font_size: f64) -> LineItem {
    LineItem::Text {
        text: text.text.clone(),
        font: text.font,
        font_size,
    }
}

fn unit_items(unit: MetronomeUnit, staff_space: f64, items: &mut Vec<LineItem>) {
    items.push(LineItem::Glyph {
        glyph: unit.kind.notehead_glyph(),
        scale: METRONOME_NOTE_SCALE,
        dy: 0.0,
    });
    for _ in 0..unit.dots {
        items.push(LineItem::Gap(METRONOME_DOT_GAP_SS * staff_space));
        items.push(LineItem::Glyph {
            glyph: Glyph::MetAugmentationDot,
            scale: METRONOME_NOTE_SCALE,
            dy: 0.0,
        });
    }
}

/// Line items of a metronome mark, in bold at `font_size`.
fn metronome_items(mark: &MetronomeMark, font_size: f64, staff_space: f64) -> Vec<LineItem> {
    let bold = |text: String| LineItem::Text {
        text,
        font: TextFont::Bold,
        font_size,
    };
    let mut items = Vec::new();
    if mark.parenthesized {
        items.push(bold("(".into()));
    }
    unit_items(mark.unit, staff_space, &mut items);
    let approx = if mark.approximate { "c. " } else { "" };
    let close = if mark.parenthesized { ")" } else { "" };
    match mark.value {
        MetronomeValue::Bpm(bpm) => items.push(bold(format!(" = {approx}{bpm}{close}"))),
        MetronomeValue::Range(from, to) => {
            items.push(bold(format!(" = {approx}{from}-{to}{close}")))
        }
        MetronomeValue::Unit(right) => {
            items.push(bold(" = ".into()));
            unit_items(right, staff_space, &mut items);
            if mark.parenthesized {
                items.push(bold(")".into()));
            }
        }
    }
    items
}

/// Lay out a tempo marking with its left edge at `x_left` and its bottom
/// line's baseline at `bottom_baseline_y` (see [`tempo_baseline`]).
pub fn layout_tempo_mark(
    mark: &TempoMark,
    x_left: f64,
    bottom_baseline_y: f64,
    font: &MusicFont,
    staff_space: f64,
) -> Result<TempoMarkLayout, FontError> {
    let font_size = TEMPO_FONT_SIZE_SS * staff_space;
    let gap = TEXT_METRONOME_GAP_SS * staff_space;

    let mut main = Vec::new();
    if let Some(text) = &mark.text {
        main.push(text_item(text, font_size));
    }
    if let Some(metronome) = &mark.metronome {
        if !main.is_empty() {
            main.push(LineItem::Gap(gap));
        }
        main.extend(metronome_items(metronome, font_size, staff_space));
    }
    if let Some(text) = &mark.text_after {
        if !main.is_empty() {
            main.push(LineItem::Gap(gap));
        }
        main.push(text_item(text, font_size));
    }

    let mut lines = Vec::new();
    if !main.is_empty() {
        lines.push(layout_text_line(&main, font)?);
    }
    if let Some(text) = &mark.text_below {
        lines.push(layout_text_line(&[text_item(text, font_size)], font)?);
    }

    let line_step = TEMPO_LINE_SPACING * font_size;
    let count = lines.len();
    let lines = lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| TempoLine {
            line,
            baseline_y: bottom_baseline_y - (count - 1 - i) as f64 * line_step,
        })
        .collect();
    Ok(TempoMarkLayout { x_left, lines })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::text_script::PlacedLineItem;

    const SS: f64 = 250.0;

    fn glyphs(line: &TextLineLayout) -> Vec<Glyph> {
        line.items
            .iter()
            .filter_map(|i| match i {
                PlacedLineItem::Glyph { glyph, .. } => Some(*glyph),
                PlacedLineItem::Text { .. } => None,
            })
            .collect()
    }

    fn texts(line: &TextLineLayout) -> Vec<String> {
        line.items
            .iter()
            .filter_map(|i| match i {
                PlacedLineItem::Text { text, .. } => Some(text.clone()),
                PlacedLineItem::Glyph { .. } => None,
            })
            .collect()
    }

    #[test]
    fn parenthesized_approximate_dotted_range_composes_in_source_order() {
        let font = bravura_font();
        let mark = TempoMark::metronome(
            MetronomeMark::range(MetronomeUnit::dotted(MetronomeNoteKind::Quarter), 58, 56)
                .approx()
                .parenthesized(),
        );
        let layout = layout_tempo_mark(&mark, 100.0, -700.0, &font, SS).unwrap();
        assert_eq!(layout.lines.len(), 1);
        let line = &layout.lines[0].line;
        // "(" ♩ . "= c. 58-56)"
        let order: Vec<String> = line
            .items
            .iter()
            .map(|i| match i {
                PlacedLineItem::Text { text, .. } => text.clone(),
                PlacedLineItem::Glyph { glyph, .. } => format!("{glyph:?}"),
            })
            .collect();
        assert_eq!(
            order,
            ["(", "MetNoteQuarterUp", "MetAugmentationDot", "= c. 58-56)"]
        );
        // Strictly increasing x: the dot sits right of the note, the text
        // right of the dot.
        let xs: Vec<f64> = line
            .items
            .iter()
            .map(|i| match i {
                PlacedLineItem::Text { x, .. } | PlacedLineItem::Glyph { x, .. } => *x,
            })
            .collect();
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "{xs:?}");
        assert_eq!(layout.lines[0].baseline_y, -700.0);
    }

    #[test]
    fn equation_has_two_note_glyphs_and_no_bpm() {
        let font = bravura_font();
        let mark = TempoMark::metronome(MetronomeMark::equation(
            MetronomeNoteKind::Quarter,
            MetronomeNoteKind::Quarter,
        ));
        let layout = layout_tempo_mark(&mark, 0.0, 0.0, &font, SS).unwrap();
        let line = &layout.lines[0].line;
        assert_eq!(
            glyphs(line),
            [Glyph::MetNoteQuarterUp, Glyph::MetNoteQuarterUp]
        );
        assert_eq!(texts(line), ["="]);
    }

    #[test]
    fn text_after_and_text_below_stack_with_the_bottom_line_on_the_baseline() {
        let font = bravura_font();
        let mark = TempoMark::metronome(
            MetronomeMark::bpm(MetronomeNoteKind::Half, 100)
                .approx()
                .parenthesized(),
        )
        .with_text_after(TempoText::upright("Monodia"))
        .with_text_below("Alla gavotta");
        let layout = layout_tempo_mark(&mark, 0.0, -700.0, &font, SS).unwrap();
        assert_eq!(layout.lines.len(), 2);
        assert_eq!(texts(&layout.lines[0].line), ["(", "= c. 100)", "Monodia"]);
        assert_eq!(texts(&layout.lines[1].line), ["Alla gavotta"]);
        assert_eq!(layout.lines[1].baseline_y, -700.0);
        let step = 1.2 * TEMPO_FONT_SIZE_SS * SS;
        assert!((layout.lines[0].baseline_y - (-700.0 - step)).abs() < 1e-9);
        let PlacedLineItem::Text { font: f, .. } = layout.lines[0].line.items.last().unwrap()
        else {
            panic!()
        };
        assert_eq!(*f, TextFont::Upright);
    }

    #[test]
    fn every_note_kind_has_a_metronome_glyph() {
        assert_eq!(
            MetronomeNoteKind::ThirtySecond.notehead_glyph(),
            Glyph::MetNote32ndUp
        );
        assert_eq!(
            MetronomeNoteKind::Whole.notehead_glyph(),
            Glyph::MetNoteWhole
        );
    }
}
