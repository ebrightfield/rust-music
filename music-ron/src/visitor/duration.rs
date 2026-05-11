use music::notation::rhythm::duration::DurationKind;

/// Parse a duration shorthand string like "4", "8.", "16.." into a
/// `(DurationKind, dot_count)` pair. Rejects unknown bases and >2 dots.
pub(crate) fn parse_duration_shorthand(input: &str) -> Result<(DurationKind, u8), ()> {
    let (base_str, dots_str) = match input.find('.') {
        Some(i) => (&input[..i], &input[i..]),
        None => (input, ""),
    };
    let kind = match base_str {
        "1" => DurationKind::Whole,
        "2" => DurationKind::Half,
        "4" => DurationKind::Qtr,
        "8" => DurationKind::Eighth,
        "16" => DurationKind::Sixteenth,
        "32" => DurationKind::ThirtySecond,
        "64" => DurationKind::SixtyFourth,
        "128" => DurationKind::OneTwentyEighth,
        _ => return Err(()),
    };
    let dots = match dots_str {
        "" => 0u8,
        "." => 1,
        ".." => 2,
        _ => return Err(()),
    };
    Ok((kind, dots))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_note() {
        assert_eq!(parse_duration_shorthand("1"), Ok((DurationKind::Whole, 0)));
    }

    #[test]
    fn quarter_note() {
        assert_eq!(parse_duration_shorthand("4"), Ok((DurationKind::Qtr, 0)));
    }

    #[test]
    fn dotted_eighth() {
        assert_eq!(
            parse_duration_shorthand("8."),
            Ok((DurationKind::Eighth, 1))
        );
    }

    #[test]
    fn double_dotted_half() {
        assert_eq!(
            parse_duration_shorthand("2.."),
            Ok((DurationKind::Half, 2))
        );
    }

    #[test]
    fn sixteenth() {
        assert_eq!(
            parse_duration_shorthand("16"),
            Ok((DurationKind::Sixteenth, 0))
        );
    }

    #[test]
    fn one_twenty_eighth() {
        assert_eq!(
            parse_duration_shorthand("128"),
            Ok((DurationKind::OneTwentyEighth, 0))
        );
    }

    #[test]
    fn triple_dot_rejected() {
        assert_eq!(parse_duration_shorthand("4..."), Err(()));
    }

    #[test]
    fn unknown_base_rejected() {
        assert_eq!(parse_duration_shorthand("3"), Err(()));
    }

    #[test]
    fn empty_rejected() {
        assert_eq!(parse_duration_shorthand(""), Err(()));
    }

    #[test]
    fn only_dots_rejected() {
        assert_eq!(parse_duration_shorthand("."), Err(()));
    }
}
