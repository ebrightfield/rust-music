/// Lyric text layout — syllables positioned below the staff under each note.
///
/// Standard engraving convention: lyrics appear below the staff in roman
/// (upright) text, centered on the note they belong to. Syllables that
/// continue to the next note show a trailing hyphen; melismatic extensions
/// show an underscore/extender line. Lyrics sit below dynamics and expression
/// text to avoid collision.
use crate::layout::staff::StaffLayout;

/// The continuation style after a lyric syllable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LyricContinuation {
    /// No continuation — the word ends with this syllable.
    None,
    /// A hyphen follows, indicating the word continues on the next note.
    Hyphen,
    /// An extender line follows, indicating the syllable is sustained
    /// (melisma) across one or more subsequent notes.
    Extender,
}

/// A lyric syllable with optional continuation.
#[derive(Clone, Debug, PartialEq)]
pub struct LyricSyllable {
    /// The text content (e.g. "Hap", "py", "day").
    pub text: String,
    /// How this syllable connects to the next note's lyric.
    pub continuation: LyricContinuation,
}

impl LyricSyllable {
    /// Create a standalone syllable (end of word, no continuation).
    pub fn word(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::None,
        }
    }

    /// Create a syllable followed by a hyphen (word continues).
    pub fn with_hyphen(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::Hyphen,
        }
    }

    /// Create a syllable with a melisma extender.
    pub fn with_extender(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::Extender,
        }
    }
}

/// Result of laying out a lyric syllable.
#[derive(Clone, Debug, PartialEq)]
pub struct LyricLayout {
    /// The syllable text content.
    pub text: String,
    /// Center x-position (aligned with the note).
    pub x_center: f64,
    /// Baseline y-position (below the staff).
    pub y_baseline: f64,
    /// Font size in font design units.
    pub font_size: f64,
    /// Continuation style (hyphen, extender, or none).
    pub continuation: LyricContinuation,
}

/// Distance below the bottom staff line for lyric text placement, in staff spaces.
/// Below expression text (4.0ss) and dynamics (2.5ss) to avoid collision.
pub const LYRIC_BELOW_STAFF_SS: f64 = 5.5;

/// Font size for lyric text, in staff spaces.
const LYRIC_FONT_SIZE_SS: f64 = 1.4;

/// Lay out a lyric syllable below the staff.
///
/// `syllable` is the lyric content with continuation info.
/// `note_center_x` is the horizontal center of the note it belongs to.
/// `staff` provides vertical reference for placement below the bottom staff line.
/// `staff_space` is the staff space size in font design units.
pub fn layout_lyric(
    syllable: &LyricSyllable,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> LyricLayout {
    let font_size = LYRIC_FONT_SIZE_SS * staff_space;
    let below_offset = LYRIC_BELOW_STAFF_SS * staff_space;

    let bottom_line_y = staff.y_of(0);
    let y_baseline = bottom_line_y + below_offset;

    LyricLayout {
        text: syllable.text.clone(),
        x_center: note_center_x,
        y_baseline,
        font_size,
        continuation: syllable.continuation.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    #[test]
    fn text_content_preserved() {
        let syl = LyricSyllable::word("day");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "day");
    }

    #[test]
    fn x_center_preserved() {
        let syl = LyricSyllable::word("sing");
        let layout = layout_lyric(&syl, 1234.0, &test_staff(), 250.0);
        assert_eq!(layout.x_center, 1234.0);
    }

    #[test]
    fn baseline_below_bottom_staff_line() {
        let staff = test_staff();
        let bottom_y = staff.y_of(0);
        let syl = LyricSyllable::word("la");
        let layout = layout_lyric(&syl, 0.0, &staff, 250.0);
        assert!(
            layout.y_baseline > bottom_y,
            "baseline {} should be below bottom line {}",
            layout.y_baseline,
            bottom_y
        );
    }

    #[test]
    fn below_staff_offset_is_correct() {
        let staff = test_staff();
        let bottom_y = staff.y_of(0);
        let syl = LyricSyllable::word("test");
        let layout = layout_lyric(&syl, 0.0, &staff, 250.0);
        let expected = bottom_y + LYRIC_BELOW_STAFF_SS * 250.0;
        assert!(
            (layout.y_baseline - expected).abs() < 0.01,
            "expected y={expected}, got {}",
            layout.y_baseline
        );
    }

    #[test]
    fn lyrics_below_expression_text() {
        let staff = test_staff();
        let syl = LyricSyllable::word("test");
        let lyric_layout = layout_lyric(&syl, 0.0, &staff, 250.0);

        // Expression text is at 4.0ss below bottom line
        let expr_y = staff.y_of(0) + 4.0 * 250.0;
        assert!(
            lyric_layout.y_baseline > expr_y,
            "lyrics ({}) should be below expression text ({})",
            lyric_layout.y_baseline,
            expr_y
        );
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let syl = LyricSyllable::word("a");
        let small = layout_lyric(&syl, 0.0, &test_staff(), 125.0);
        let large = layout_lyric(&syl, 0.0, &test_staff(), 250.0);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn font_size_is_positive() {
        let syl = LyricSyllable::word("x");
        let layout = layout_lyric(&syl, 0.0, &test_staff(), 250.0);
        assert!(layout.font_size > 0.0);
    }

    #[test]
    fn different_positions_produce_different_layouts() {
        let syl = LyricSyllable::word("do");
        let a = layout_lyric(&syl, 100.0, &test_staff(), 250.0);
        let b = layout_lyric(&syl, 900.0, &test_staff(), 250.0);
        assert!((a.x_center - b.x_center).abs() > 700.0);
    }

    #[test]
    fn word_constructor_no_continuation() {
        let syl = LyricSyllable::word("day");
        assert_eq!(syl.continuation, LyricContinuation::None);
        assert_eq!(syl.text, "day");
    }

    #[test]
    fn hyphen_constructor() {
        let syl = LyricSyllable::with_hyphen("hap");
        assert_eq!(syl.continuation, LyricContinuation::Hyphen);
        assert_eq!(syl.text, "hap");
    }

    #[test]
    fn extender_constructor() {
        let syl = LyricSyllable::with_extender("love");
        assert_eq!(syl.continuation, LyricContinuation::Extender);
        assert_eq!(syl.text, "love");
    }

    #[test]
    fn continuation_preserved_in_layout() {
        let syl_h = LyricSyllable::with_hyphen("hap");
        let layout_h = layout_lyric(&syl_h, 0.0, &test_staff(), 250.0);
        assert_eq!(layout_h.continuation, LyricContinuation::Hyphen);

        let syl_e = LyricSyllable::with_extender("love");
        let layout_e = layout_lyric(&syl_e, 0.0, &test_staff(), 250.0);
        assert_eq!(layout_e.continuation, LyricContinuation::Extender);

        let syl_n = LyricSyllable::word("day");
        let layout_n = layout_lyric(&syl_n, 0.0, &test_staff(), 250.0);
        assert_eq!(layout_n.continuation, LyricContinuation::None);
    }
}
