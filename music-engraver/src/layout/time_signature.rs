use smufl::Glyph;

/// Time signature display style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimeSignatureKind {
    /// Numeric time signature (e.g. 4/4, 6/8, 7/16).
    Numeric { numerator: u8, denominator: u8 },
    /// Common time symbol (C).
    Common,
    /// Cut time / alla breve symbol (₵).
    CutCommon,
}

/// A resolved time signature glyph sequence with vertical positions.
///
/// Numerator digits are centred on staff position 6 (third space from bottom).
/// Denominator digits are centred on staff position 2 (first space from bottom).
/// Common/CutCommon glyphs are centred on the middle line (position 4).
#[derive(Clone, Debug)]
pub struct TimeSignatureLayout {
    /// Glyphs to render. Each entry is (glyph, staff_position, x_offset from
    /// the start of the time signature group).
    pub glyphs: Vec<(Glyph, i8, f64)>,
    /// Total advance width of the time signature in font design units.
    pub width: f64,
}

/// Map a decimal digit 0–9 to the corresponding SMuFL time signature glyph.
pub fn digit_glyph(d: u8) -> Option<Glyph> {
    match d {
        0 => Some(Glyph::TimeSig0),
        1 => Some(Glyph::TimeSig1),
        2 => Some(Glyph::TimeSig2),
        3 => Some(Glyph::TimeSig3),
        4 => Some(Glyph::TimeSig4),
        5 => Some(Glyph::TimeSig5),
        6 => Some(Glyph::TimeSig6),
        7 => Some(Glyph::TimeSig7),
        8 => Some(Glyph::TimeSig8),
        9 => Some(Glyph::TimeSig9),
        _ => None,
    }
}

/// Convert a number (1–99) to a sequence of SMuFL time-sig digit glyphs.
fn number_to_digit_glyphs(n: u8) -> Vec<Glyph> {
    if n >= 10 {
        let tens = n / 10;
        let ones = n % 10;
        vec![digit_glyph(tens).unwrap(), digit_glyph(ones).unwrap()]
    } else {
        vec![digit_glyph(n).unwrap()]
    }
}

/// Compute the layout for a time signature.
///
/// `advance_of` returns the advance width (in font design units) for a given glyph.
/// Numerator is centred on staff position 6, denominator on staff position 2.
/// For common/cut-common, the single glyph is at staff position 4 (middle line).
pub fn time_signature_layout(
    kind: &TimeSignatureKind,
    advance_of: impl Fn(Glyph) -> f64,
) -> TimeSignatureLayout {
    match kind {
        TimeSignatureKind::Numeric {
            numerator,
            denominator,
        } => {
            let num_glyphs = number_to_digit_glyphs(*numerator);
            let den_glyphs = number_to_digit_glyphs(*denominator);

            let num_width: f64 = num_glyphs.iter().map(|g| advance_of(*g)).sum();
            let den_width: f64 = den_glyphs.iter().map(|g| advance_of(*g)).sum();
            let total_width = num_width.max(den_width);

            // Centre numerator row
            let num_offset = (total_width - num_width) / 2.0;
            let den_offset = (total_width - den_width) / 2.0;

            let mut glyphs = Vec::new();
            let mut x = num_offset;
            for g in &num_glyphs {
                glyphs.push((*g, 6_i8, x));
                x += advance_of(*g);
            }
            x = den_offset;
            for g in &den_glyphs {
                glyphs.push((*g, 2_i8, x));
                x += advance_of(*g);
            }

            TimeSignatureLayout {
                glyphs,
                width: total_width,
            }
        }
        TimeSignatureKind::Common => {
            let glyph = Glyph::TimeSigCommon;
            let w = advance_of(glyph);
            TimeSignatureLayout {
                glyphs: vec![(glyph, 4, 0.0)],
                width: w,
            }
        }
        TimeSignatureKind::CutCommon => {
            let glyph = Glyph::TimeSigCutCommon;
            let w = advance_of(glyph);
            TimeSignatureLayout {
                glyphs: vec![(glyph, 4, 0.0)],
                width: w,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stub advance: every glyph is 250 units wide.
    fn fixed_advance(_g: Glyph) -> f64 {
        250.0
    }

    #[test]
    fn common_time_single_glyph() {
        let layout = time_signature_layout(&TimeSignatureKind::Common, fixed_advance);
        assert_eq!(layout.glyphs.len(), 1);
        assert_eq!(layout.glyphs[0].0, Glyph::TimeSigCommon);
        assert_eq!(layout.glyphs[0].1, 4);
        assert!((layout.width - 250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cut_common_single_glyph() {
        let layout = time_signature_layout(&TimeSignatureKind::CutCommon, fixed_advance);
        assert_eq!(layout.glyphs.len(), 1);
        assert_eq!(layout.glyphs[0].0, Glyph::TimeSigCutCommon);
        assert_eq!(layout.glyphs[0].1, 4);
    }

    #[test]
    fn four_four_has_two_glyphs() {
        let kind = TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        // 1 digit numerator + 1 digit denominator = 2 glyphs
        assert_eq!(layout.glyphs.len(), 2);
        let num = &layout.glyphs[0];
        let den = &layout.glyphs[1];
        assert_eq!(num.0, Glyph::TimeSig4);
        assert_eq!(num.1, 6);
        assert_eq!(den.0, Glyph::TimeSig4);
        assert_eq!(den.1, 2);
    }

    #[test]
    fn six_eight_correct_glyphs() {
        let kind = TimeSignatureKind::Numeric {
            numerator: 6,
            denominator: 8,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        assert_eq!(layout.glyphs.len(), 2);
        assert_eq!(layout.glyphs[0].0, Glyph::TimeSig6);
        assert_eq!(layout.glyphs[1].0, Glyph::TimeSig8);
    }

    #[test]
    fn twelve_eight_has_four_glyphs() {
        let kind = TimeSignatureKind::Numeric {
            numerator: 12,
            denominator: 8,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        // 2 numerator digits + 1 denominator digit = 3 glyphs
        assert_eq!(layout.glyphs.len(), 3);
        assert_eq!(layout.glyphs[0].0, Glyph::TimeSig1);
        assert_eq!(layout.glyphs[0].1, 6);
        assert_eq!(layout.glyphs[1].0, Glyph::TimeSig2);
        assert_eq!(layout.glyphs[1].1, 6);
        assert_eq!(layout.glyphs[2].0, Glyph::TimeSig8);
        assert_eq!(layout.glyphs[2].1, 2);
    }

    #[test]
    fn numerator_and_denominator_centred_when_different_widths() {
        // Numerator "12" = 2 glyphs × 250 = 500, denominator "8" = 1 glyph × 250
        let kind = TimeSignatureKind::Numeric {
            numerator: 12,
            denominator: 8,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        let total_width = layout.width;
        // Width should be max(500, 250) = 500
        assert!((total_width - 500.0).abs() < f64::EPSILON);
        // Denominator "8" should be centred: offset = (500-250)/2 = 125
        let den = &layout.glyphs[2];
        assert!((den.2 - 125.0).abs() < f64::EPSILON);
        // First numerator digit should be at offset 0 (centred, but full width)
        assert!((layout.glyphs[0].2 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn denominator_wider_centres_numerator() {
        // 3/16: numerator 1 digit (250), denominator 2 digits (500)
        let kind = TimeSignatureKind::Numeric {
            numerator: 3,
            denominator: 16,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        assert!((layout.width - 500.0).abs() < f64::EPSILON);
        // Numerator "3" centred: offset = (500-250)/2 = 125
        assert!((layout.glyphs[0].2 - 125.0).abs() < f64::EPSILON);
        // Denominator "1" at offset 0
        assert!((layout.glyphs[1].2 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn digit_glyph_maps_all_digits() {
        for d in 0..=9u8 {
            assert!(digit_glyph(d).is_some(), "digit {} should map", d);
        }
        assert!(digit_glyph(10).is_none());
        assert!(digit_glyph(255).is_none());
    }

    #[test]
    fn digit_glyph_returns_correct_variants() {
        assert_eq!(digit_glyph(0), Some(Glyph::TimeSig0));
        assert_eq!(digit_glyph(5), Some(Glyph::TimeSig5));
        assert_eq!(digit_glyph(9), Some(Glyph::TimeSig9));
    }

    #[test]
    fn all_glyphs_at_correct_staff_positions() {
        let kind = TimeSignatureKind::Numeric {
            numerator: 7,
            denominator: 16,
        };
        let layout = time_signature_layout(&kind, fixed_advance);
        // Numerator glyphs at position 6
        assert_eq!(layout.glyphs[0].1, 6); // "7"
        // Denominator glyphs at position 2
        assert_eq!(layout.glyphs[1].1, 2); // "1"
        assert_eq!(layout.glyphs[2].1, 2); // "6"
    }

    #[test]
    fn width_is_positive_for_all_kinds() {
        let kinds = [
            TimeSignatureKind::Common,
            TimeSignatureKind::CutCommon,
            TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            },
            TimeSignatureKind::Numeric {
                numerator: 12,
                denominator: 8,
            },
        ];
        for kind in &kinds {
            let layout = time_signature_layout(kind, fixed_advance);
            assert!(layout.width > 0.0, "{:?} should have positive width", kind);
        }
    }
}
