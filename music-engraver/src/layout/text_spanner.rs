//! Text spanners: a text label followed by a dashed, solid or absent line
//! that runs to a later event ("rit. - - -", "cresc. - - -", "dim").
//!
//! LilyPond's `\startTextSpan` / `\stopTextSpan` and the dynamic text
//! spanners (`\crescTextCresc`, `\decrescendoText`) are all this one shape.
//! The label sits at the start event and the line runs along the label
//! baseline to the end event. Across a system break, the source system
//! keeps the label and the line runs to its right edge; the next system
//! carries a label-less continuation (see
//! [`layout_text_spanner_continuation`]).

use crate::layout::placement::Placement;
use crate::layout::staff::StaffLayout;
use crate::layout::text_script::TextFont;

/// Line drawn after a text spanner's label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpannerLine {
    /// Dashed continuation line (LilyPond `dashed-line`, the default).
    #[default]
    Dashed,
    /// Solid continuation line.
    Solid,
    /// Label only, no line.
    None,
}

/// A text spanner's content and style.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextSpanner {
    /// The label ("rit.", "cresc.", "dim").
    pub text: String,
    /// Continuation line style.
    pub line: SpannerLine,
    /// Side of the staff.
    pub placement: Placement,
    /// Typeface style of the label.
    pub font: TextFont,
}

impl TextSpanner {
    /// An italic label with a `line`, on `placement`'s side of the staff.
    pub fn new(text: impl Into<String>, line: SpannerLine, placement: Placement) -> Self {
        Self {
            text: text.into(),
            line,
            placement,
            font: TextFont::Italic,
        }
    }

    /// Dashed "cresc." below the staff.
    pub fn cresc() -> Self {
        Self::new("cresc.", SpannerLine::Dashed, Placement::Below)
    }

    /// Dashed "decresc." below the staff.
    pub fn decresc() -> Self {
        Self::new("decresc.", SpannerLine::Dashed, Placement::Below)
    }

    /// Dashed "dim." below the staff.
    pub fn dim() -> Self {
        Self::new("dim.", SpannerLine::Dashed, Placement::Below)
    }

    /// Dashed "rit." above the staff.
    pub fn rit() -> Self {
        Self::new("rit.", SpannerLine::Dashed, Placement::Above)
    }

    /// Move the spanner to `placement`'s side of the staff.
    pub fn placed(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Set the label's typeface style.
    pub fn font(mut self, font: TextFont) -> Self {
        self.font = font;
        self
    }
}

/// Geometry of one system's segment of a text spanner.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpannerLayout {
    /// Label text.
    pub label: String,
    /// Label typeface style.
    pub font: TextFont,
    /// Left edge of the segment (label x when `has_label`).
    pub x_start: f64,
    /// Right end of the line.
    pub x_end: f64,
    /// Where the line starts: after the label, or `x_start` without one.
    pub x_line_start: f64,
    /// Baseline of the label; the line is drawn along it.
    pub y_baseline: f64,
    /// Label font size in font design units.
    pub font_size: f64,
    /// Line stroke width.
    pub line_thickness: f64,
    /// Dash length (dashed lines).
    pub dash_length: f64,
    /// Gap between dashes (dashed lines).
    pub dash_gap: f64,
    /// Line style.
    pub line: SpannerLine,
    /// Whether this segment draws the label (false on continuation
    /// segments after a system break).
    pub has_label: bool,
}

/// Distance from the bottom staff line down to a below-staff spanner's
/// baseline, in staff spaces.
pub const TEXT_SPANNER_BELOW_STAFF_SS: f64 = 3.5;

/// Distance from the top staff line up to an above-staff spanner's
/// baseline, in staff spaces.
pub const TEXT_SPANNER_ABOVE_STAFF_SS: f64 = 1.6;

/// Label font size, in staff spaces.
pub const TEXT_SPANNER_FONT_SIZE_SS: f64 = 1.4;

/// Estimated label advance per character, in staff spaces.
pub const TEXT_SPANNER_LABEL_WIDTH_PER_CHAR_SS: f64 = 0.6;

/// Gap between the label and the start of the line, in staff spaces.
pub const TEXT_SPANNER_LABEL_PADDING_SS: f64 = 0.25;

/// Line stroke width, in staff spaces.
pub const TEXT_SPANNER_LINE_THICKNESS_SS: f64 = 0.12;

/// Dash length, in staff spaces.
pub const TEXT_SPANNER_DASH_LENGTH_SS: f64 = 0.8;

/// Gap between dashes, in staff spaces.
pub const TEXT_SPANNER_DASH_GAP_SS: f64 = 0.4;

/// Baseline y of a spanner on `placement`'s side of `staff`.
pub fn text_spanner_baseline(placement: Placement, staff: &StaffLayout, staff_space: f64) -> f64 {
    match placement {
        Placement::Below => staff.y_of(0) + TEXT_SPANNER_BELOW_STAFF_SS * staff_space,
        Placement::Above => staff.y_of(8) - TEXT_SPANNER_ABOVE_STAFF_SS * staff_space,
    }
}

fn segment(
    spanner: &TextSpanner,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
    has_label: bool,
) -> TextSpannerLayout {
    let x_line_start = if has_label {
        x_start
            + spanner.text.chars().count() as f64
                * TEXT_SPANNER_LABEL_WIDTH_PER_CHAR_SS
                * staff_space
            + TEXT_SPANNER_LABEL_PADDING_SS * staff_space
    } else {
        x_start
    };
    TextSpannerLayout {
        label: spanner.text.clone(),
        font: spanner.font,
        x_start,
        x_end,
        x_line_start,
        y_baseline: text_spanner_baseline(spanner.placement, staff, staff_space),
        font_size: TEXT_SPANNER_FONT_SIZE_SS * staff_space,
        line_thickness: TEXT_SPANNER_LINE_THICKNESS_SS * staff_space,
        dash_length: TEXT_SPANNER_DASH_LENGTH_SS * staff_space,
        dash_gap: TEXT_SPANNER_DASH_GAP_SS * staff_space,
        line: spanner.line,
        has_label,
    }
}

/// Lay out the labelled segment of a spanner from `x_start` (label left
/// edge) to `x_end` (line end).
pub fn layout_text_spanner(
    spanner: &TextSpanner,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> TextSpannerLayout {
    segment(spanner, x_start, x_end, staff, staff_space, true)
}

/// Lay out a label-less continuation segment (the part of a spanner on the
/// system after a break): the line alone from `x_start` to `x_end`.
pub fn layout_text_spanner_continuation(
    spanner: &TextSpanner,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> TextSpannerLayout {
    segment(spanner, x_start, x_end, staff, staff_space, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0;

    fn staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, SS)
    }

    #[test]
    fn presets_keep_the_dynamic_text_labels_below_and_rit_above() {
        for (spanner, label) in [
            (TextSpanner::cresc(), "cresc."),
            (TextSpanner::decresc(), "decresc."),
            (TextSpanner::dim(), "dim."),
        ] {
            assert_eq!(spanner.text, label);
            assert_eq!(spanner.placement, Placement::Below);
            assert_eq!(spanner.line, SpannerLine::Dashed);
            assert_eq!(spanner.font, TextFont::Italic);
        }
        assert_eq!(TextSpanner::rit().placement, Placement::Above);
    }

    #[test]
    fn below_and_above_baselines_mirror_around_the_staff() {
        let s = staff();
        let below = layout_text_spanner(&TextSpanner::cresc(), 100.0, 900.0, &s, SS);
        let above = layout_text_spanner(&TextSpanner::rit(), 100.0, 900.0, &s, SS);
        assert!((below.y_baseline - (s.y_of(0) + 3.5 * SS)).abs() < 1e-9);
        assert!((above.y_baseline - (s.y_of(8) - 1.6 * SS)).abs() < 1e-9);
    }

    #[test]
    fn line_starts_after_the_label_estimate_and_continuations_drop_the_label() {
        let s = staff();
        let rit = TextSpanner::rit();
        let first = layout_text_spanner(&rit, 100.0, 900.0, &s, SS);
        assert!(first.has_label);
        assert!((first.x_line_start - (100.0 + 4.0 * 0.6 * SS + 0.25 * SS)).abs() < 1e-9);
        let cont = layout_text_spanner_continuation(&rit, 50.0, 400.0, &s, SS);
        assert!(!cont.has_label);
        assert_eq!(cont.x_line_start, 50.0);
        assert_eq!(cont.y_baseline, first.y_baseline);
    }
}
