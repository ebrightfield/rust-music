use smufl::Glyph;

use crate::font::EngravingConfig;

/// Number of lines in a standard guitar tablature staff.
pub const TAB_LINE_COUNT: u8 = 6;

/// Number of lines in a 4-string tablature staff (bass, ukulele).
pub const TAB_4_STRING_LINE_COUNT: u8 = 4;

/// Geometric model for a tablature staff.
///
/// Like [`super::staff::StaffLayout`] but supports variable line counts
/// (typically 6 for guitar, 4 for bass/ukulele). The coordinate system
/// uses SVG conventions: y increases downward. `y_origin` is the
/// y-coordinate of the top line.
///
/// String numbering follows guitar convention: string 1 is the highest
/// pitch (bottom line in TAB), string N is the lowest pitch (top line).
#[derive(Clone, Debug)]
pub struct TabStaffLayout {
    /// X-coordinate of the left edge.
    pub x: f64,
    /// Y-coordinate of the top staff line in the enclosing canvas.
    pub y_origin: f64,
    /// Total width of the staff lines.
    pub width: f64,
    /// Distance between adjacent staff lines, in font design units.
    pub staff_space: f64,
    /// Number of strings/lines (6 for guitar, 4 for bass/ukulele).
    pub line_count: u8,
}

impl TabStaffLayout {
    /// Create a tab staff layout with explicit geometry.
    pub fn new(x: f64, y_origin: f64, width: f64, staff_space: f64, line_count: u8) -> Self {
        Self {
            x,
            y_origin,
            width,
            staff_space,
            line_count,
        }
    }

    /// Standard 6-string guitar tab staff.
    pub fn guitar(x: f64, y_origin: f64, width: f64, config: &EngravingConfig) -> Self {
        Self::new(x, y_origin, width, config.staff_space, TAB_LINE_COUNT)
    }

    /// 4-string tab staff (bass guitar, ukulele).
    pub fn four_string(x: f64, y_origin: f64, width: f64, config: &EngravingConfig) -> Self {
        Self::new(x, y_origin, width, config.staff_space, TAB_4_STRING_LINE_COUNT)
    }

    /// Total height from top line to bottom line.
    pub fn height(&self) -> f64 {
        self.staff_space * (self.line_count.saturating_sub(1)) as f64
    }

    /// Y-coordinate of the bottom staff line.
    pub fn bottom_y(&self) -> f64 {
        self.y_origin + self.height()
    }

    /// Y-coordinates of all staff lines, from top to bottom.
    pub fn line_ys(&self) -> Vec<f64> {
        (0..self.line_count)
            .map(|i| self.y_origin + i as f64 * self.staff_space)
            .collect()
    }

    /// Y-coordinate for a given string number (1-based, 1 = highest pitch = bottom line).
    ///
    /// String 1 is at the bottom, string N is at the top — matching guitar
    /// convention where string 1 is the thinnest (high E).
    pub fn string_y(&self, string: u8) -> f64 {
        // String 1 → bottom line, string line_count → top line.
        // Bottom line index = line_count - 1, top line index = 0.
        let line_index = self.line_count.saturating_sub(string);
        self.y_origin + line_index as f64 * self.staff_space
    }

    /// Vertical center of the staff (for centering the TAB clef glyph).
    pub fn center_y(&self) -> f64 {
        self.y_origin + self.height() / 2.0
    }
}

/// Returns the SMuFL glyph for the TAB clef based on string count.
///
/// 6-string TAB uses `_6StringTabClef`, 4-string uses `_4StringTabClef`.
/// Other counts default to the 6-string clef.
pub fn tab_clef_glyph(line_count: u8) -> Glyph {
    match line_count {
        4 => Glyph::_4StringTabClef,
        _ => Glyph::_6StringTabClef,
    }
}

/// Font size for fret numbers relative to staff space. Slightly smaller than
/// a full staff space for clean centering on the line.
pub const FRET_NUMBER_FONT_SIZE_RATIO: f64 = 1.4;

/// Layout result for a single fret number on a tab staff.
#[derive(Clone, Debug)]
pub struct FretNumberLayout {
    /// X-coordinate (horizontal center of the number).
    pub x: f64,
    /// Y-coordinate (vertical center, on the string line).
    pub y: f64,
    /// The fret number text (e.g. "0", "12", "24").
    pub text: String,
    /// Computed font size in font design units.
    pub font_size: f64,
    /// Half-width of a white background rect to mask the staff line behind the number.
    pub bg_half_width: f64,
    /// Half-height of a white background rect.
    pub bg_half_height: f64,
}

/// Compute layout for a fret number on a tab staff.
///
/// `string` is 1-based (1 = highest pitch = bottom line).
/// `fret` is the fret number (0 = open string, up to 24+).
/// `x` is the horizontal position of the note event.
pub fn layout_fret_number(
    tab_staff: &TabStaffLayout,
    string: u8,
    fret: u8,
    x: f64,
) -> FretNumberLayout {
    let y = tab_staff.string_y(string);
    let font_size = tab_staff.staff_space * FRET_NUMBER_FONT_SIZE_RATIO;
    let text = fret.to_string();

    // Background rect sized to mask the line behind the number.
    // Wider for 2-digit numbers.
    let digit_count = if fret >= 10 { 2.0 } else { 1.0 };
    let bg_half_width = font_size * 0.35 * digit_count;
    let bg_half_height = font_size * 0.45;

    FretNumberLayout {
        x,
        y,
        text,
        font_size,
        bg_half_width,
        bg_half_height,
    }
}

/// Layout result for a muted/dead string marker ("x") on a tab staff.
///
/// Reuses `FretNumberLayout` with text "x" — the rendering is identical
/// to a fret number (white background rect + centered bold text) except
/// the text content indicates the string is muted rather than fretted.
pub fn layout_muted_string(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
) -> FretNumberLayout {
    let y = tab_staff.string_y(string);
    let font_size = tab_staff.staff_space * FRET_NUMBER_FONT_SIZE_RATIO;

    // Single-character "x" uses the same sizing as a single-digit fret number.
    let bg_half_width = font_size * 0.35;
    let bg_half_height = font_size * 0.45;

    FretNumberLayout {
        x,
        y,
        text: "x".to_string(),
        font_size,
        bg_half_width,
        bg_half_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn guitar_staff() -> TabStaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config)
    }

    #[test]
    fn guitar_has_six_lines() {
        let staff = guitar_staff();
        assert_eq!(staff.line_count, 6);
        assert_eq!(staff.line_ys().len(), 6);
    }

    #[test]
    fn four_string_has_four_lines() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::four_string(0.0, 0.0, 5000.0, &config);
        assert_eq!(staff.line_count, 4);
        assert_eq!(staff.line_ys().len(), 4);
    }

    #[test]
    fn height_is_five_spaces_for_six_lines() {
        let staff = guitar_staff();
        let expected = staff.staff_space * 5.0;
        assert!((staff.height() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn height_is_three_spaces_for_four_lines() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::four_string(0.0, 0.0, 5000.0, &config);
        let expected = staff.staff_space * 3.0;
        assert!((staff.height() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn line_ys_evenly_spaced() {
        let staff = guitar_staff();
        let ys = staff.line_ys();
        for i in 1..ys.len() {
            let gap = ys[i] - ys[i - 1];
            assert!(
                (gap - staff.staff_space).abs() < f64::EPSILON,
                "gap between line {i} and {} should be one staff space",
                i - 1
            );
        }
    }

    #[test]
    fn top_line_at_y_origin() {
        let staff = guitar_staff();
        let ys = staff.line_ys();
        assert!((ys[0] - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bottom_line_at_height() {
        let staff = guitar_staff();
        let ys = staff.line_ys();
        assert!(
            (ys[5] - staff.height()).abs() < f64::EPSILON,
            "bottom line should be at height"
        );
    }

    #[test]
    fn string_1_is_bottom_line() {
        let staff = guitar_staff();
        let bottom = *staff.line_ys().last().unwrap();
        assert!(
            (staff.string_y(1) - bottom).abs() < f64::EPSILON,
            "string 1 (high E) should be the bottom line"
        );
    }

    #[test]
    fn string_6_is_top_line() {
        let staff = guitar_staff();
        let top = staff.line_ys()[0];
        assert!(
            (staff.string_y(6) - top).abs() < f64::EPSILON,
            "string 6 (low E) should be the top line"
        );
    }

    #[test]
    fn string_3_is_fourth_line_from_top() {
        let staff = guitar_staff();
        // String 3 (G) → line index = 6-3 = 3 → y = 3 * staff_space
        let expected = staff.staff_space * 3.0;
        assert!((staff.string_y(3) - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn center_y_is_midpoint() {
        let staff = guitar_staff();
        let mid = (staff.y_origin + staff.bottom_y()) / 2.0;
        assert!((staff.center_y() - mid).abs() < f64::EPSILON);
    }

    #[test]
    fn tab_clef_glyph_six_string() {
        assert_eq!(tab_clef_glyph(6), Glyph::_6StringTabClef);
    }

    #[test]
    fn tab_clef_glyph_four_string() {
        assert_eq!(tab_clef_glyph(4), Glyph::_4StringTabClef);
    }

    #[test]
    fn tab_clef_glyph_other_defaults_to_six() {
        assert_eq!(tab_clef_glyph(5), Glyph::_6StringTabClef);
        assert_eq!(tab_clef_glyph(7), Glyph::_6StringTabClef);
    }

    #[test]
    fn fret_number_layout_position() {
        let staff = guitar_staff();
        let layout = layout_fret_number(&staff, 1, 5, 1000.0);
        assert!((layout.x - 1000.0).abs() < f64::EPSILON);
        assert!((layout.y - staff.string_y(1)).abs() < f64::EPSILON);
        assert_eq!(layout.text, "5");
    }

    #[test]
    fn fret_number_layout_open_string() {
        let staff = guitar_staff();
        let layout = layout_fret_number(&staff, 3, 0, 500.0);
        assert_eq!(layout.text, "0");
    }

    #[test]
    fn fret_number_layout_two_digits_wider_bg() {
        let staff = guitar_staff();
        let single = layout_fret_number(&staff, 1, 5, 100.0);
        let double = layout_fret_number(&staff, 1, 12, 100.0);
        assert!(
            double.bg_half_width > single.bg_half_width,
            "two-digit fret numbers need wider background"
        );
    }

    #[test]
    fn fret_number_font_size_scales_with_staff_space() {
        let staff = guitar_staff();
        let layout = layout_fret_number(&staff, 1, 7, 100.0);
        let expected = staff.staff_space * FRET_NUMBER_FONT_SIZE_RATIO;
        assert!((layout.font_size - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn nonzero_origin_shifts_all_ys() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config);
        let ys = staff.line_ys();
        assert!((ys[0] - 500.0).abs() < f64::EPSILON);
        assert!((staff.string_y(6) - 500.0).abs() < f64::EPSILON);
        let fret = layout_fret_number(&staff, 6, 3, 100.0);
        assert!((fret.y - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn muted_string_text_is_x() {
        let staff = guitar_staff();
        let layout = layout_muted_string(&staff, 1, 100.0);
        assert_eq!(layout.text, "x");
    }

    #[test]
    fn muted_string_y_matches_string_position() {
        let staff = guitar_staff();
        let layout = layout_muted_string(&staff, 3, 100.0);
        let expected_y = staff.string_y(3);
        assert!(
            (layout.y - expected_y).abs() < f64::EPSILON,
            "muted string y should match string_y"
        );
    }

    #[test]
    fn muted_string_x_preserved() {
        let staff = guitar_staff();
        let layout = layout_muted_string(&staff, 1, 500.0);
        assert!((layout.x - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn muted_string_font_size_matches_fret_number() {
        let staff = guitar_staff();
        let muted = layout_muted_string(&staff, 1, 100.0);
        let fret = layout_fret_number(&staff, 1, 5, 100.0);
        assert!(
            (muted.font_size - fret.font_size).abs() < f64::EPSILON,
            "muted and fret number font sizes should match"
        );
    }

    #[test]
    fn muted_string_bg_same_as_single_digit_fret() {
        let staff = guitar_staff();
        let muted = layout_muted_string(&staff, 1, 100.0);
        let fret = layout_fret_number(&staff, 1, 5, 100.0);
        assert!(
            (muted.bg_half_width - fret.bg_half_width).abs() < f64::EPSILON,
            "muted 'x' should have same background width as single-digit fret"
        );
    }

    #[test]
    fn muted_string_different_strings_have_different_y() {
        let staff = guitar_staff();
        let m1 = layout_muted_string(&staff, 1, 100.0);
        let m6 = layout_muted_string(&staff, 6, 100.0);
        assert!(
            (m1.y - m6.y).abs() > 1.0,
            "different strings should have different y positions"
        );
    }
}
