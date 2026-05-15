//! Chord symbol layout — harmony labels above the staff.
//!
//! Standard engraving convention: chord symbols (e.g. "Cmaj7", "Am", "F#dim")
//! appear above the staff in a bold sans-serif or serif font, centered on the
//! beat they apply to. Positioned above rehearsal marks and tempo markings
//! when those are absent, or at a consistent height above the staff.
//!
//! Two layout paths are provided:
//!
//! - [`layout_chord_symbol`] — single-text-element layout. The chord symbol is
//!   rendered as one `<text>` element including any ASCII `#` / `b` accidental
//!   characters as plain text. Stable, font-agnostic; the original v1 path.
//! - [`layout_chord_symbol_composite`] — text + SMuFL accidental glyph
//!   composition. The symbol is parsed into [`ChordSymbolSegment`]s; text runs
//!   are emitted as `<text>` and accidentals (`#`, `b`-as-flat, `♯`, `♭`, `♮`)
//!   become SMuFL glyph paths (`accidentalSharp`, `accidentalFlat`,
//!   `accidentalNatural`) sized at ~70 % of the text font size and
//!   baseline-raised slightly — the engraving convention used by all
//!   high-quality engraving software.

use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// Result of laying out a chord symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordSymbolLayout {
    /// The chord symbol text content (e.g. "Cmaj7", "Am").
    pub text: String,
    /// Center x-position (aligned with the note/beat it applies to).
    pub x_center: f64,
    /// Baseline y-position of the text (above the staff).
    pub y_baseline: f64,
    /// Font size in font design units.
    pub font_size: f64,
}

/// Distance above the top staff line for chord symbol placement, in staff spaces.
/// Above rehearsal marks (2.5ss) to avoid collision. Chord symbols are the
/// topmost text layer in standard engraving.
pub const CHORD_SYMBOL_ABOVE_STAFF_SS: f64 = 3.5;

/// Font size for chord symbols, in staff spaces.
const CHORD_SYMBOL_FONT_SIZE_SS: f64 = 1.6;

/// Lay out a chord symbol above the staff.
///
/// `text` is the chord symbol content (e.g. "Cmaj7", "Am7", "F#dim").
/// `note_center_x` is the horizontal center of the note/beat it applies to.
/// `staff` provides vertical reference for placement above the top staff line.
/// `staff_space` is the staff space size in font design units.
pub fn layout_chord_symbol(
    text: &str,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> ChordSymbolLayout {
    let font_size = CHORD_SYMBOL_FONT_SIZE_SS * staff_space;
    let above_offset = CHORD_SYMBOL_ABOVE_STAFF_SS * staff_space;

    // Top staff line y (smaller y = higher in SVG)
    let top_line_y = staff.y_of(8);
    let y_baseline = top_line_y - above_offset;

    ChordSymbolLayout {
        text: text.to_string(),
        x_center: note_center_x,
        y_baseline,
        font_size,
    }
}

// ---------------------------------------------------------------------------
// Composite layout with SMuFL accidental glyph substitution
// ---------------------------------------------------------------------------

/// A single component of a parsed chord symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordSymbolSegment {
    /// A run of plain text (letters, digits, punctuation other than accidentals).
    Text(String),
    /// A sharp accidental — `#`, `♯` (U+266F), or contextual.
    Sharp,
    /// A flat accidental — `b` (in flat context), `♭` (U+266D).
    Flat,
    /// A natural accidental — `♮` (U+266E).
    Natural,
}

impl ChordSymbolSegment {
    /// Whether this segment is an accidental (Sharp, Flat, or Natural).
    ///
    /// Returns `false` for `Text`. Useful for callers that need to decide
    /// whether to apply the inter-segment side bearing
    /// ([`ACCIDENTAL_SIDE_BEARING_FACTOR`]) on either side of a boundary
    /// without re-pattern-matching the variant.
    pub const fn is_accidental(&self) -> bool {
        matches!(self, Self::Sharp | Self::Flat | Self::Natural)
    }

    /// The SMuFL glyph that depicts this accidental segment in chord-symbol
    /// context, or `None` for `Text`.
    ///
    /// All three chord-symbol accidentals (`Sharp`, `Flat`, `Natural`) map to
    /// the standard SMuFL accidental glyphs: `AccidentalSharp`, `AccidentalFlat`,
    /// `AccidentalNatural`. The mapping is fixed for chord symbols — variant
    /// SMuFL glyphs (small/raised) are reserved for inline notation
    /// (key signatures, alterations on noteheads), not for chord-symbol use.
    pub const fn glyph(&self) -> Option<Glyph> {
        match self {
            Self::Text(_) => None,
            Self::Sharp => Some(Glyph::AccidentalSharp),
            Self::Flat => Some(Glyph::AccidentalFlat),
            Self::Natural => Some(Glyph::AccidentalNatural),
        }
    }
}

/// Parse a chord-symbol string into a sequence of text runs and accidental tokens.
///
/// Accidental resolution rules — driven by chord-symbol convention, not by the
/// generic English language:
///
/// - ASCII `#` is always [`ChordSymbolSegment::Sharp`].
/// - Unicode `♯` (U+266F), `♭` (U+266D), `♮` (U+266E) are always the
///   corresponding accidentals.
/// - ASCII lowercase `b` is contextual. In a chord symbol it is a flat when:
///     - immediately preceded by an uppercase root letter `A`–`G` (e.g. `Bb`, `Eb`),
///     - immediately preceded by a digit (e.g. `b5`, `b9`, `7b5`, `13b9`),
///     - immediately preceded by `(`, `+`, `-`, `/`, or `,`
///       (e.g. `C(b5)`, `C7+b9`, `D/Bb`),
///     - or it is the first character of the symbol (rare but defensible — keeps
///       a leading `b5` token from being lost).
///
///   Otherwise `b` is kept as plain text.
/// - All other characters accumulate into a [`ChordSymbolSegment::Text`] run.
///
/// The returned segments are contiguous in source order; adjacent text segments
/// are merged.
pub fn parse_chord_symbol_segments(text: &str) -> Vec<ChordSymbolSegment> {
    let mut out: Vec<ChordSymbolSegment> = Vec::new();
    let mut text_run = String::new();
    let mut prev_char: Option<char> = None;

    // Flush any accumulated text into the output.
    fn flush_text(text_run: &mut String, out: &mut Vec<ChordSymbolSegment>) {
        if !text_run.is_empty() {
            out.push(ChordSymbolSegment::Text(std::mem::take(text_run)));
        }
    }

    for c in text.chars() {
        match c {
            '#' | '\u{266F}' => {
                flush_text(&mut text_run, &mut out);
                out.push(ChordSymbolSegment::Sharp);
            }
            '\u{266D}' => {
                flush_text(&mut text_run, &mut out);
                out.push(ChordSymbolSegment::Flat);
            }
            '\u{266E}' => {
                flush_text(&mut text_run, &mut out);
                out.push(ChordSymbolSegment::Natural);
            }
            'b' if is_flat_context(prev_char) => {
                flush_text(&mut text_run, &mut out);
                out.push(ChordSymbolSegment::Flat);
            }
            _ => {
                text_run.push(c);
            }
        }
        prev_char = Some(c);
    }
    flush_text(&mut text_run, &mut out);
    out
}

/// Decide whether a lowercase `b` at the current position should be interpreted
/// as a flat accidental based on the preceding character.
fn is_flat_context(prev: Option<char>) -> bool {
    match prev {
        // First character of the symbol — treat as flat (rare but defensible).
        None => true,
        Some(p) => {
            matches!(p, 'A'..='G')
                || p.is_ascii_digit()
                || matches!(p, '(' | '+' | '-' | '/' | ',')
        }
    }
}

/// A laid-out segment of a composite chord symbol.
///
/// Coordinates are absolute, in font design units (the same coordinate system
/// the SVG viewBox uses elsewhere in the renderer).
#[derive(Debug, Clone, PartialEq)]
pub struct ChordSymbolSegmentBox {
    /// What this segment is — text run or accidental glyph.
    pub segment: ChordSymbolSegment,
    /// Left x-coordinate of the segment's bounding box.
    pub x_left: f64,
    /// Baseline y-coordinate. For accidentals this is the glyph's font-baseline
    /// (already baseline-raised relative to the text baseline; see
    /// [`ACCIDENTAL_BASELINE_RAISE_FACTOR`]).
    pub y_baseline: f64,
    /// Font size applied to this segment. For text this is the chord symbol's
    /// text font size; for accidentals it is the reduced accidental font size
    /// ([`ACCIDENTAL_SIZE_FACTOR`] × text font size).
    pub font_size: f64,
    /// Estimated horizontal advance width of this segment in font design units.
    /// Text widths are approximations (no text font is bundled — see
    /// [`CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR`]); accidental widths come from
    /// the music font's glyph metrics scaled by the accidental font size.
    pub width: f64,
}

/// Composite chord-symbol layout — text and accidental glyphs together.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordSymbolCompositeLayout {
    /// Per-segment layout boxes, in source order.
    pub boxes: Vec<ChordSymbolSegmentBox>,
    /// Center x of the entire composite (the value passed in to layout).
    pub x_center: f64,
    /// Baseline y of the text portion. Accidentals are baseline-raised from
    /// this value; their box stores its own raised baseline.
    pub y_baseline: f64,
    /// Text font size in font design units.
    pub font_size: f64,
    /// Total estimated width of all segments combined, in font design units.
    pub total_width: f64,
}

/// Estimated text character width factor.
///
/// Multiplied by the text font size to estimate the advance width of one
/// character. The chord-symbol text font is not bundled — the layout uses
/// a serif/sans-serif fallback via SVG `font-family="serif"`, so the actual
/// rendered widths depend on the viewer. 0.55 em is a conservative average
/// across common chord-symbol fonts (Times, Helvetica, Bravura Text); it is
/// only used to space the SMuFL accidental glyphs *between* text runs.
pub const CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR: f64 = 0.55;

/// Scale factor applied to the music-font's natural accidental glyph size
/// when used in a chord symbol.
///
/// Chord-symbol accidentals are typographically smaller than the surrounding
/// chord letter so they don't visually overwhelm a `Bb` or `F#`. 0.70 is the
/// standard convention used by Sibelius, Finale, and Dorico.
pub const ACCIDENTAL_SIZE_FACTOR: f64 = 0.70;

/// Vertical offset of the accidental glyph baseline above the text baseline,
/// as a fraction of the text font size.
///
/// SMuFL accidental glyphs naturally extend mostly upward from their baseline,
/// but a small upward shift visually centers them on the text x-height rather
/// than letting them sit on the descender line. 0.20 em produces a balanced
/// alignment with a typical 0.7-em x-height serif/sans-serif body font.
pub const ACCIDENTAL_BASELINE_RAISE_FACTOR: f64 = 0.20;

/// Spacing between an accidental glyph and the adjacent text run, as a fraction
/// of the text font size. Applied on both sides of every accidental.
///
/// A small gap (~0.08 em) prevents the glyph from kissing the next character
/// without visually disconnecting from the chord letter it modifies.
pub const ACCIDENTAL_SIDE_BEARING_FACTOR: f64 = 0.08;

/// Estimated advance width of a single chord-symbol segment, in font design units.
///
/// This is a courtesy helper for external callers (e.g. higher-level layout
/// code that needs to budget chord-symbol width without invoking
/// [`layout_chord_symbol_composite`] and walking its `boxes`). It returns the
/// segment's intrinsic width only — **inter-segment side bearings are not
/// included**, because side bearings depend on the *neighbors* of a segment,
/// not on the segment in isolation. To get a total composite width including
/// gaps between accidentals and adjacent text, use the
/// [`ChordSymbolCompositeLayout::total_width`] field instead.
///
/// Formulas (identical to the ones used by [`layout_chord_symbol_composite`]):
///
/// - `Text(s)`: `s.chars().count() * text_font_size * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR`
/// - `Sharp` / `Flat` / `Natural`: `accidental_advance(glyph) * text_font_size * ACCIDENTAL_SIZE_FACTOR / units_per_em`
///
/// `units_per_em` is floored at 1 to avoid divide-by-zero on junk font input.
pub fn chord_symbol_segment_advance(
    seg: &ChordSymbolSegment,
    text_font_size: f64,
    units_per_em: u16,
    accidental_advance: impl Fn(Glyph) -> u16,
) -> f64 {
    let upe = units_per_em.max(1) as f64;
    let accidental_font_size = text_font_size * ACCIDENTAL_SIZE_FACTOR;
    match seg {
        ChordSymbolSegment::Text(s) => {
            s.chars().count() as f64 * text_font_size * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR
        }
        ChordSymbolSegment::Sharp => {
            accidental_advance(Glyph::AccidentalSharp) as f64 * accidental_font_size / upe
        }
        ChordSymbolSegment::Flat => {
            accidental_advance(Glyph::AccidentalFlat) as f64 * accidental_font_size / upe
        }
        ChordSymbolSegment::Natural => {
            accidental_advance(Glyph::AccidentalNatural) as f64 * accidental_font_size / upe
        }
    }
}

/// Lay out a chord symbol with SMuFL accidental glyph composition.
///
/// `text` is the chord symbol source (e.g. `"F#m7b5"`).
/// `note_center_x` is the horizontal center the composite should be centered on.
/// `staff` provides the vertical reference (top staff line).
/// `staff_space` is the staff space size in font design units.
/// `units_per_em` is the music font's units-per-em (used to scale accidental
/// glyph paths from the music font's design-unit space into the SVG's
/// font-unit space).
/// `accidental_advance` is a callback returning the music font's advance width
/// (in design units) for a given accidental glyph. Keeping the metric source
/// as a callback preserves the layout module's font-agnostic stance — the
/// renderer wires Bravura's metrics in at the call site, but a Petaluma or
/// Leland substitution requires no layout changes.
pub fn layout_chord_symbol_composite(
    text: &str,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
    units_per_em: u16,
    accidental_advance: impl Fn(Glyph) -> u16,
) -> ChordSymbolCompositeLayout {
    let font_size = CHORD_SYMBOL_FONT_SIZE_SS * staff_space;
    let above_offset = CHORD_SYMBOL_ABOVE_STAFF_SS * staff_space;
    let top_line_y = staff.y_of(8);
    let y_baseline = top_line_y - above_offset;

    let segments = parse_chord_symbol_segments(text);
    let accidental_font_size = font_size * ACCIDENTAL_SIZE_FACTOR;
    let accidental_y_baseline = y_baseline - font_size * ACCIDENTAL_BASELINE_RAISE_FACTOR;
    let accidental_side_bearing = font_size * ACCIDENTAL_SIDE_BEARING_FACTOR;

    // Pre-compute per-segment widths and per-segment side bearings (extra
    // gap added BEFORE this segment) in a single pass. Side bearings live
    // between an accidental and any adjacent segment.
    let mut widths: Vec<f64> = Vec::with_capacity(segments.len());
    let mut leading_gaps: Vec<f64> = Vec::with_capacity(segments.len());
    for (i, seg) in segments.iter().enumerate() {
        widths.push(chord_symbol_segment_advance(
            seg,
            font_size,
            units_per_em,
            &accidental_advance,
        ));
        let needs_gap_before =
            i > 0 && (seg.is_accidental() || segments[i - 1].is_accidental());
        leading_gaps.push(if needs_gap_before { accidental_side_bearing } else { 0.0 });
    }

    let total_width: f64 = widths.iter().sum::<f64>() + leading_gaps.iter().sum::<f64>();
    let mut cursor = note_center_x - total_width / 2.0;
    let mut boxes: Vec<ChordSymbolSegmentBox> = Vec::with_capacity(segments.len());

    for ((seg, width), gap) in segments.into_iter().zip(widths.iter()).zip(leading_gaps.iter()) {
        cursor += *gap;
        let (seg_font_size, seg_baseline) = match seg {
            ChordSymbolSegment::Text(_) => (font_size, y_baseline),
            ChordSymbolSegment::Sharp
            | ChordSymbolSegment::Flat
            | ChordSymbolSegment::Natural => (accidental_font_size, accidental_y_baseline),
        };
        boxes.push(ChordSymbolSegmentBox {
            segment: seg,
            x_left: cursor,
            y_baseline: seg_baseline,
            font_size: seg_font_size,
            width: *width,
        });
        cursor += *width;
    }

    ChordSymbolCompositeLayout {
        boxes,
        x_center: note_center_x,
        y_baseline,
        font_size,
        total_width,
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
        let layout = layout_chord_symbol("Cmaj7", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "Cmaj7");
    }

    #[test]
    fn x_center_preserved() {
        let layout = layout_chord_symbol("Am", 1234.0, &test_staff(), 250.0);
        assert_eq!(layout.x_center, 1234.0);
    }

    #[test]
    fn baseline_above_top_staff_line() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout = layout_chord_symbol("G7", 0.0, &staff, 250.0);
        assert!(
            layout.y_baseline < top_y,
            "baseline {} should be above top line {}",
            layout.y_baseline,
            top_y
        );
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_chord_symbol("C", 0.0, &test_staff(), 125.0);
        let large = layout_chord_symbol("C", 0.0, &test_staff(), 250.0);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn different_positions_produce_different_layouts() {
        let a = layout_chord_symbol("C", 100.0, &test_staff(), 250.0);
        let b = layout_chord_symbol("C", 900.0, &test_staff(), 250.0);
        assert!((a.x_center - b.x_center).abs() > 700.0);
    }

    #[test]
    fn font_size_is_positive() {
        let layout = layout_chord_symbol("X", 0.0, &test_staff(), 250.0);
        assert!(layout.font_size > 0.0);
    }

    #[test]
    fn above_staff_offset_is_correct() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout = layout_chord_symbol("Dm", 0.0, &staff, 250.0);
        let expected = top_y - CHORD_SYMBOL_ABOVE_STAFF_SS * 250.0;
        assert!(
            (layout.y_baseline - expected).abs() < 0.01,
            "expected y={expected}, got {}",
            layout.y_baseline
        );
    }

    #[test]
    fn different_texts_produce_different_layouts() {
        let a = layout_chord_symbol("C", 500.0, &test_staff(), 250.0);
        let b = layout_chord_symbol("Am7", 500.0, &test_staff(), 250.0);
        assert_ne!(a.text, b.text);
    }

    #[test]
    fn empty_text_allowed() {
        let layout = layout_chord_symbol("", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "");
    }

    #[test]
    fn complex_symbol_preserved() {
        let layout = layout_chord_symbol("F#m7b5", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "F#m7b5");
    }

    #[test]
    fn higher_than_rehearsal_marks() {
        // Chord symbols at 3.5ss above vs rehearsal at 2.5ss above
        let staff = test_staff();
        let ss = 250.0;
        let chord = layout_chord_symbol("C", 500.0, &staff, ss);
        let rehearsal_y = staff.y_of(8) - 2.5 * ss;
        assert!(
            chord.y_baseline < rehearsal_y,
            "chord symbol y {} should be higher (smaller) than rehearsal y {}",
            chord.y_baseline,
            rehearsal_y
        );
    }

    // ---------------- parse_chord_symbol_segments ----------------

    use ChordSymbolSegment as S;

    #[test]
    fn parse_empty_string_yields_no_segments() {
        assert!(parse_chord_symbol_segments("").is_empty());
    }

    #[test]
    fn parse_plain_text_yields_single_text_segment() {
        assert_eq!(
            parse_chord_symbol_segments("Cmaj7"),
            vec![S::Text("Cmaj7".into())]
        );
    }

    #[test]
    fn parse_ascii_hash_is_sharp() {
        assert_eq!(
            parse_chord_symbol_segments("F#"),
            vec![S::Text("F".into()), S::Sharp]
        );
    }

    #[test]
    fn parse_unicode_sharp_is_sharp() {
        assert_eq!(
            parse_chord_symbol_segments("F\u{266F}"),
            vec![S::Text("F".into()), S::Sharp]
        );
    }

    #[test]
    fn parse_unicode_flat_is_flat() {
        assert_eq!(
            parse_chord_symbol_segments("B\u{266D}"),
            vec![S::Text("B".into()), S::Flat]
        );
    }

    #[test]
    fn parse_unicode_natural_is_natural() {
        assert_eq!(
            parse_chord_symbol_segments("F\u{266E}"),
            vec![S::Text("F".into()), S::Natural]
        );
    }

    #[test]
    fn parse_b_after_root_letter_is_flat() {
        // The classic case: "Bb" means B-flat root.
        assert_eq!(
            parse_chord_symbol_segments("Bb"),
            vec![S::Text("B".into()), S::Flat]
        );
        assert_eq!(
            parse_chord_symbol_segments("Eb"),
            vec![S::Text("E".into()), S::Flat]
        );
    }

    #[test]
    fn parse_b_after_digit_is_flat() {
        assert_eq!(
            parse_chord_symbol_segments("7b5"),
            vec![S::Text("7".into()), S::Flat, S::Text("5".into())]
        );
        assert_eq!(
            parse_chord_symbol_segments("13b9"),
            vec![S::Text("13".into()), S::Flat, S::Text("9".into())]
        );
    }

    #[test]
    fn parse_b_after_paren_or_alteration_marker_is_flat() {
        assert_eq!(
            parse_chord_symbol_segments("C(b5)"),
            vec![
                S::Text("C(".into()),
                S::Flat,
                S::Text("5)".into()),
            ]
        );
        assert_eq!(
            parse_chord_symbol_segments("C+b9"),
            vec![S::Text("C+".into()), S::Flat, S::Text("9".into())]
        );
        assert_eq!(
            parse_chord_symbol_segments("D/Bb"),
            vec![S::Text("D/B".into()), S::Flat]
        );
    }

    #[test]
    fn parse_b_at_start_of_symbol_is_flat() {
        // Defensive case: a bare leading "b5" alteration label, isolated.
        assert_eq!(
            parse_chord_symbol_segments("b5"),
            vec![S::Flat, S::Text("5".into())]
        );
    }

    #[test]
    fn parse_complex_chord_symbol() {
        // F#m7b5 — half-diminished. Should split into 5 segments:
        // "F", #, "m7", b, "5"
        assert_eq!(
            parse_chord_symbol_segments("F#m7b5"),
            vec![
                S::Text("F".into()),
                S::Sharp,
                S::Text("m7".into()),
                S::Flat,
                S::Text("5".into()),
            ]
        );
    }

    #[test]
    fn parse_multiple_sharps_and_flats_preserved() {
        // F#7#9 — altered dominant with both ♯7 (no, that's wrong terminology — #9
        // is the alteration). Either way, two sharps in a single symbol.
        assert_eq!(
            parse_chord_symbol_segments("F#7#9"),
            vec![
                S::Text("F".into()),
                S::Sharp,
                S::Text("7".into()),
                S::Sharp,
                S::Text("9".into()),
            ]
        );
    }

    #[test]
    fn parse_consecutive_accidentals_distinct() {
        // Edge case: "##" — two adjacent sharps. Don't merge or drop.
        let segs = parse_chord_symbol_segments("C##");
        assert_eq!(segs, vec![S::Text("C".into()), S::Sharp, S::Sharp]);
    }

    #[test]
    fn parse_text_with_no_accidentals_is_single_text() {
        // Sanity guard: anything that doesn't trigger the b-context rule stays text.
        assert_eq!(
            parse_chord_symbol_segments("Cmaj7"),
            vec![S::Text("Cmaj7".into())]
        );
        assert_eq!(
            parse_chord_symbol_segments("Am"),
            vec![S::Text("Am".into())]
        );
        assert_eq!(
            parse_chord_symbol_segments("G7sus4"),
            vec![S::Text("G7sus4".into())]
        );
    }

    #[test]
    fn parse_unicode_natural_and_flat_in_same_symbol() {
        // Defensive: handle multiple Unicode accidentals in one symbol.
        assert_eq!(
            parse_chord_symbol_segments("C\u{266E}7\u{266D}5"),
            vec![
                S::Text("C".into()),
                S::Natural,
                S::Text("7".into()),
                S::Flat,
                S::Text("5".into()),
            ]
        );
    }

    // ---------------- layout_chord_symbol_composite ----------------

    /// Synthetic advance-width callback that returns a fixed value for every
    /// accidental. Lets layout tests reason about widths without a real font.
    fn fixed_advance(_g: smufl::Glyph) -> u16 {
        200
    }

    #[test]
    fn composite_plain_text_yields_one_text_segment() {
        let layout = layout_chord_symbol_composite(
            "Cmaj7",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        assert_eq!(layout.boxes.len(), 1);
        assert!(matches!(
            layout.boxes[0].segment,
            ChordSymbolSegment::Text(ref t) if t == "Cmaj7"
        ));
    }

    #[test]
    fn composite_f_sharp_yields_text_plus_sharp() {
        let layout = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        assert_eq!(layout.boxes.len(), 2);
        assert!(matches!(
            layout.boxes[0].segment,
            ChordSymbolSegment::Text(ref t) if t == "F"
        ));
        assert_eq!(layout.boxes[1].segment, ChordSymbolSegment::Sharp);
    }

    #[test]
    fn composite_b_flat_yields_text_plus_flat() {
        let layout = layout_chord_symbol_composite(
            "Bb",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        assert_eq!(layout.boxes.len(), 2);
        assert_eq!(layout.boxes[1].segment, ChordSymbolSegment::Flat);
    }

    #[test]
    fn composite_accidentals_use_reduced_font_size() {
        let layout = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        // Box[0] is text — full font size; Box[1] is sharp — reduced.
        let text_size = layout.boxes[0].font_size;
        let acc_size = layout.boxes[1].font_size;
        let expected = text_size * ACCIDENTAL_SIZE_FACTOR;
        assert!(
            (acc_size - expected).abs() < 0.01,
            "accidental font_size {acc_size} should equal text {text_size} × {ACCIDENTAL_SIZE_FACTOR} = {expected}"
        );
    }

    #[test]
    fn composite_accidentals_baseline_raised() {
        let layout = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        // Sharp box baseline should be y_baseline minus the configured raise.
        let text_baseline = layout.boxes[0].y_baseline;
        let acc_baseline = layout.boxes[1].y_baseline;
        let expected_raise = layout.font_size * ACCIDENTAL_BASELINE_RAISE_FACTOR;
        let observed_raise = text_baseline - acc_baseline;
        assert!(
            (observed_raise - expected_raise).abs() < 0.01,
            "accidental baseline should be {expected_raise} above text baseline, got {observed_raise}"
        );
    }

    #[test]
    fn composite_segments_are_left_to_right() {
        let layout = layout_chord_symbol_composite(
            "F#m7b5",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        assert_eq!(layout.boxes.len(), 5);
        for i in 0..(layout.boxes.len() - 1) {
            assert!(
                layout.boxes[i].x_left < layout.boxes[i + 1].x_left,
                "boxes should be ordered left to right; box[{i}]={:?} >= box[{}]={:?}",
                layout.boxes[i],
                i + 1,
                layout.boxes[i + 1]
            );
        }
    }

    #[test]
    fn composite_total_width_matches_sum_of_widths_plus_gaps() {
        let layout = layout_chord_symbol_composite(
            "F#m7b5",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        // The last segment's right edge minus the first segment's left edge
        // should equal total_width (no leading gap on first segment).
        let leftmost = layout.boxes.first().unwrap().x_left;
        let rightmost = {
            let last = layout.boxes.last().unwrap();
            last.x_left + last.width
        };
        let span = rightmost - leftmost;
        assert!(
            (span - layout.total_width).abs() < 0.01,
            "rightmost - leftmost ({span}) should equal total_width ({})",
            layout.total_width
        );
    }

    #[test]
    fn composite_is_centered_on_note_center_x() {
        let center = 500.0;
        let layout = layout_chord_symbol_composite(
            "F#m7b5",
            center,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        let leftmost = layout.boxes.first().unwrap().x_left;
        let rightmost = {
            let last = layout.boxes.last().unwrap();
            last.x_left + last.width
        };
        let observed_center = (leftmost + rightmost) / 2.0;
        assert!(
            (observed_center - center).abs() < 0.5,
            "composite should be centered on {center}, observed center is {observed_center}"
        );
    }

    #[test]
    fn composite_accidental_width_scales_with_font_advance() {
        // Doubling the accidental advance width must double the sharp segment's width.
        let big_advance = |_g: smufl::Glyph| 400u16;
        let small_advance = |_g: smufl::Glyph| 200u16;
        let layout_big = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            1000,
            big_advance,
        );
        let layout_small = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            1000,
            small_advance,
        );
        let big = layout_big.boxes[1].width;
        let small = layout_small.boxes[1].width;
        assert!(
            (big - 2.0 * small).abs() < 0.01,
            "sharp width should scale linearly with advance width: big={big}, small={small}"
        );
    }

    #[test]
    fn composite_empty_string_yields_empty_layout() {
        let layout = layout_chord_symbol_composite(
            "",
            500.0,
            &test_staff(),
            250.0,
            1000,
            fixed_advance,
        );
        assert!(layout.boxes.is_empty());
        assert_eq!(layout.total_width, 0.0);
        assert_eq!(layout.x_center, 500.0);
    }

    #[test]
    fn composite_baseline_matches_simple_layout() {
        // Byte equivalence cousin: the composite layout's text baseline must
        // match the simple layout's baseline for the same input position.
        let staff = test_staff();
        let simple = layout_chord_symbol("Cmaj7", 500.0, &staff, 250.0);
        let composite = layout_chord_symbol_composite(
            "Cmaj7",
            500.0,
            &staff,
            250.0,
            1000,
            fixed_advance,
        );
        assert!((simple.y_baseline - composite.y_baseline).abs() < 0.01);
        assert!((simple.font_size - composite.font_size).abs() < 0.01);
    }

    #[test]
    fn composite_units_per_em_zero_does_not_panic() {
        // Defensive: a junk units_per_em of 0 must not divide-by-zero.
        let layout = layout_chord_symbol_composite(
            "F#",
            500.0,
            &test_staff(),
            250.0,
            0,
            fixed_advance,
        );
        // No panic; layout produced.
        assert_eq!(layout.boxes.len(), 2);
    }

    // ---------------- ChordSymbolSegment::is_accidental / glyph ----------------

    #[test]
    fn is_accidental_text_is_false() {
        assert!(!S::Text("Cmaj7".into()).is_accidental());
        assert!(!S::Text("".into()).is_accidental());
    }

    #[test]
    fn is_accidental_all_variants() {
        assert!(S::Sharp.is_accidental());
        assert!(S::Flat.is_accidental());
        assert!(S::Natural.is_accidental());
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn is_accidental_is_const_callable() {
        // Compile-fail canary: removing `const fn` from is_accidental breaks
        // the three const-item initializers below. Const context can't
        // construct a `String`, so Text is excluded — the body's exhaustive
        // match keeps Text in lockstep with the no-payload variants.
        // (Clippy correctly notes the assertions are statically true; that
        // tautology *is* the test — we're checking the value resolves at
        // compile time, not at runtime.)
        const SHARP_IS_ACCIDENTAL: bool = S::Sharp.is_accidental();
        const FLAT_IS_ACCIDENTAL: bool = S::Flat.is_accidental();
        const NATURAL_IS_ACCIDENTAL: bool = S::Natural.is_accidental();
        assert!(SHARP_IS_ACCIDENTAL);
        assert!(FLAT_IS_ACCIDENTAL);
        assert!(NATURAL_IS_ACCIDENTAL);
    }

    #[test]
    fn glyph_text_is_none() {
        assert_eq!(S::Text("Cmaj7".into()).glyph(), None);
        assert_eq!(S::Text("".into()).glyph(), None);
    }

    #[test]
    fn glyph_sharp_maps_to_accidental_sharp() {
        assert_eq!(S::Sharp.glyph(), Some(Glyph::AccidentalSharp));
    }

    #[test]
    fn glyph_flat_maps_to_accidental_flat() {
        assert_eq!(S::Flat.glyph(), Some(Glyph::AccidentalFlat));
    }

    #[test]
    fn glyph_natural_maps_to_accidental_natural() {
        assert_eq!(S::Natural.glyph(), Some(Glyph::AccidentalNatural));
    }

    #[test]
    fn glyph_is_const_callable() {
        // Compile-fail canary for `const fn` on glyph().
        const G: Option<Glyph> = S::Sharp.glyph();
        assert_eq!(G, Some(Glyph::AccidentalSharp));
    }

    // ---------------- chord_symbol_segment_advance ----------------

    #[test]
    fn segment_advance_text_uses_char_width_factor() {
        // "Am" — 2 chars; width = 2 * font_size * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR.
        let w = chord_symbol_segment_advance(
            &S::Text("Am".into()),
            400.0,
            1000,
            fixed_advance,
        );
        let expected = 2.0 * 400.0 * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR;
        assert!(
            (w - expected).abs() < 0.01,
            "expected {expected}, got {w}"
        );
    }

    #[test]
    fn segment_advance_text_scales_linearly_with_length() {
        let single = chord_symbol_segment_advance(
            &S::Text("C".into()),
            400.0,
            1000,
            fixed_advance,
        );
        let triple = chord_symbol_segment_advance(
            &S::Text("CCC".into()),
            400.0,
            1000,
            fixed_advance,
        );
        assert!(
            (triple - 3.0 * single).abs() < 0.01,
            "3-char width {triple} should equal 3 × 1-char width {single}"
        );
    }

    #[test]
    fn segment_advance_text_scales_linearly_with_font_size() {
        let small = chord_symbol_segment_advance(
            &S::Text("Am".into()),
            200.0,
            1000,
            fixed_advance,
        );
        let large = chord_symbol_segment_advance(
            &S::Text("Am".into()),
            400.0,
            1000,
            fixed_advance,
        );
        assert!(
            (large - 2.0 * small).abs() < 0.01,
            "doubling font_size must double text width: small={small}, large={large}"
        );
    }

    #[test]
    fn segment_advance_empty_text_is_zero() {
        let w = chord_symbol_segment_advance(
            &S::Text(String::new()),
            400.0,
            1000,
            fixed_advance,
        );
        assert_eq!(w, 0.0);
    }

    #[test]
    fn segment_advance_text_does_not_invoke_advance_callback() {
        // Tracks whether the callback is called. For Text segments it MUST NOT
        // be — text widths are estimated from char count alone.
        let calls = std::cell::Cell::new(0usize);
        let counting = |_g: smufl::Glyph| {
            calls.set(calls.get() + 1);
            200
        };
        let _ = chord_symbol_segment_advance(
            &S::Text("Cmaj7".into()),
            400.0,
            1000,
            counting,
        );
        assert_eq!(
            calls.get(),
            0,
            "text-segment advance must not consult the accidental advance callback"
        );
    }

    #[test]
    fn segment_advance_sharp_uses_advance_callback() {
        // advance=200, font_size=400, units_per_em=1000, ACCIDENTAL_SIZE_FACTOR=0.70
        // expected = 200 * (400 * 0.70) / 1000 = 200 * 280 / 1000 = 56
        let w = chord_symbol_segment_advance(
            &S::Sharp,
            400.0,
            1000,
            fixed_advance,
        );
        let expected = 200.0 * 400.0 * ACCIDENTAL_SIZE_FACTOR / 1000.0;
        assert!(
            (w - expected).abs() < 0.01,
            "expected {expected}, got {w}"
        );
    }

    #[test]
    fn segment_advance_scales_linearly_with_advance() {
        // Doubling the advance callback's return must double the segment width.
        let small = chord_symbol_segment_advance(
            &S::Sharp,
            400.0,
            1000,
            |_g| 200u16,
        );
        let big = chord_symbol_segment_advance(
            &S::Sharp,
            400.0,
            1000,
            |_g| 400u16,
        );
        assert!(
            (big - 2.0 * small).abs() < 0.01,
            "doubling advance must double width: small={small}, big={big}"
        );
    }

    #[test]
    fn segment_advance_accidentals_use_their_own_glyph() {
        // Callback returns different widths per glyph — verifies the function
        // queries the right glyph for each accidental variant.
        let differentiating = |g: smufl::Glyph| -> u16 {
            match g {
                smufl::Glyph::AccidentalSharp => 100,
                smufl::Glyph::AccidentalFlat => 200,
                smufl::Glyph::AccidentalNatural => 300,
                _ => 999,
            }
        };
        let sharp = chord_symbol_segment_advance(&S::Sharp, 400.0, 1000, differentiating);
        let flat = chord_symbol_segment_advance(&S::Flat, 400.0, 1000, differentiating);
        let natural =
            chord_symbol_segment_advance(&S::Natural, 400.0, 1000, differentiating);
        // Locked-in ordering: flat (200) = 2× sharp (100); natural (300) = 3× sharp.
        assert!((flat - 2.0 * sharp).abs() < 0.01);
        assert!((natural - 3.0 * sharp).abs() < 0.01);
        // Sanity: 999 (the catch-all in `differentiating`) is never reached.
        // If a refactor accidentally called the callback with a non-accidental
        // glyph, sharp would equal ~999*280/1000 = 280, not 100*280/1000 = 28.
        assert!(sharp < 50.0, "sharp width {sharp} should reflect advance=100, not 999");
    }

    #[test]
    fn segment_advance_units_per_em_zero_does_not_panic() {
        // Defensive: divide-by-zero floor — units_per_em.max(1) must hold.
        let w = chord_symbol_segment_advance(
            &S::Sharp,
            400.0,
            0,
            fixed_advance,
        );
        // The floor turns the divisor into 1, so the width is just
        // advance * accidental_font_size — large but finite, not NaN/Inf.
        assert!(w.is_finite());
        assert!(w > 0.0);
    }

    #[test]
    fn segment_advance_matches_composite_layout_widths() {
        // The most important contract: this helper and
        // layout_chord_symbol_composite agree on each segment's width.
        // F#m7b5 covers Text + Sharp + Text + Flat + Text in one symbol.
        let staff = test_staff();
        let staff_space = 250.0;
        let text_font_size = CHORD_SYMBOL_FONT_SIZE_SS * staff_space;
        let composite = layout_chord_symbol_composite(
            "F#m7b5",
            500.0,
            &staff,
            staff_space,
            1000,
            fixed_advance,
        );
        let segments = parse_chord_symbol_segments("F#m7b5");
        assert_eq!(composite.boxes.len(), segments.len());
        for (i, seg) in segments.iter().enumerate() {
            let direct = chord_symbol_segment_advance(seg, text_font_size, 1000, fixed_advance);
            let from_layout = composite.boxes[i].width;
            assert!(
                (direct - from_layout).abs() < 0.01,
                "segment[{i}] {seg:?}: helper={direct}, composite={from_layout}"
            );
        }
    }

    #[test]
    fn segment_advance_callable_repeatedly_with_same_callback() {
        // The `impl Fn` signature must accept callbacks called many times. A
        // closure that captures non-Copy state (here, a `Vec`) is itself non-
        // Copy; reusing it across multiple calls requires passing by reference.
        // This guards against an accidental switch to `FnOnce` (which would
        // move the closure on the first call and refuse the second) — that
        // change would break the renderer call site at the same time.
        let lookup: Box<[u16]> = Box::new([150u16, 200, 250]);
        let closure = move |_g: smufl::Glyph| lookup[0];
        let a = chord_symbol_segment_advance(&S::Sharp, 400.0, 1000, &closure);
        let b = chord_symbol_segment_advance(&S::Flat, 400.0, 1000, &closure);
        let c = chord_symbol_segment_advance(&S::Natural, 400.0, 1000, &closure);
        // All three should produce equal widths since the callback is constant.
        assert!((a - b).abs() < 0.01);
        assert!((b - c).abs() < 0.01);
    }
}
