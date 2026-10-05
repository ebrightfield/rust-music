/// Lyric text layout — syllables positioned below the staff under each note.
///
/// Verses occupy independent baselines below the staff, centered under their
/// notes. Each verse can use upright or italic serif text independently;
/// continuations draw hyphens or melisma lines without crossing verse lanes.
/// Lyrics sit below dynamics and expression text to avoid collision.
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
    /// Continue the word without printing a hyphen (LilyPond `\once \hide LyricHyphen`).
    HyphenHidden,
}

/// A lyric syllable with optional continuation.
#[derive(Clone, Debug, PartialEq)]
pub struct LyricSyllable {
    /// The text content (e.g. "Hap", "py", "day").
    pub text: String,
    /// How this syllable connects to the next note's lyric.
    pub continuation: LyricContinuation,
    /// A timed lyric skip: reserves an anchor but draws no text.
    pub skip: bool,
}

impl LyricSyllable {
    /// Create a standalone syllable (end of word, no continuation).
    pub fn word(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::None,
            skip: false,
        }
    }

    /// Create a syllable followed by a hyphen (word continues).
    pub fn with_hyphen(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::Hyphen,
            skip: false,
        }
    }

    /// Create a syllable with a melisma extender.
    pub fn with_extender(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::Extender,
            skip: false,
        }
    }

    /// Continue a word without a visible hyphen.
    pub fn with_hidden_hyphen(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            continuation: LyricContinuation::HyphenHidden,
            skip: false,
        }
    }

    /// Advance this verse past one note without drawing a syllable.
    pub fn skip() -> Self {
        Self {
            text: String::new(),
            continuation: LyricContinuation::None,
            skip: true,
        }
    }
}

/// Lyric typography shares the serif text-script metrics and SVG font shape.
pub use crate::layout::text_script::TextFont as LyricStyle;

/// One verse's lyric event at a note or chord. Verse numbers start at 1.
#[derive(Clone, Debug, PartialEq)]
pub struct VerseLyric {
    pub verse: u16,
    pub syllable: LyricSyllable,
    pub style: LyricStyle,
}

/// Baseline distance between adjacent lyric verses, in staff spaces.
pub const LYRIC_VERSE_GAP_SS: f64 = 2.0;

/// Baseline for a numbered verse below the bottom staff line.
pub fn verse_baseline(staff: &StaffLayout, staff_space: f64, verse: u16) -> f64 {
    staff.y_of(0)
        + (LYRIC_BELOW_STAFF_SS + f64::from(verse.saturating_sub(1)) * LYRIC_VERSE_GAP_SS)
            * staff_space
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
    /// Font shape of this verse.
    pub style: LyricStyle,
}

/// Distance below the bottom staff line for lyric text placement, in staff spaces.
/// Below expression text (4.0ss) and dynamics (2.5ss) to avoid collision.
pub const LYRIC_BELOW_STAFF_SS: f64 = 5.5;

/// Font size for lyric text, in staff spaces. Public so the system renderer
/// can compute hyphen font size to match the lyric text without re-running
/// `layout_lyric`.
pub const LYRIC_FONT_SIZE_SS: f64 = 1.4;

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
    layout_lyric_verse(
        syllable,
        note_center_x,
        staff,
        staff_space,
        1,
        LyricStyle::Upright,
    )
}

/// Lay out one verse's syllable on its independent baseline.
pub fn layout_lyric_verse(
    syllable: &LyricSyllable,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
    verse: u16,
    style: LyricStyle,
) -> LyricLayout {
    LyricLayout {
        text: syllable.text.clone(),
        x_center: note_center_x,
        y_baseline: verse_baseline(staff, staff_space, verse),
        font_size: LYRIC_FONT_SIZE_SS * staff_space,
        continuation: syllable.continuation.clone(),
        style,
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
