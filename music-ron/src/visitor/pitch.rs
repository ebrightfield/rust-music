use music::note::note::Note;
use music::note::spelling::{Accidental, Letter, Spelling};

/// Strict left-to-right pitch shorthand parser:
/// letter (1 lowercase a–g), accidental (longest-match over {"eses","ss","es","s",""}),
/// octave (single digit 0–8). Returns `(Note, octave)` on success.
pub(crate) fn parse_pitch_shorthand(input: &str) -> Result<(Note, i32), (String, ())> {
    let mut chars = input.chars();
    let letter_char = match chars.next() {
        Some(c @ 'a'..='g') => c,
        _ => return Err((input.into(), ())),
    };
    let rest: &str = &input[1..];

    // Longest-match accidental tokenization.
    // "eses" before "es" ensures double-flat wins; "ss" before "s" ensures double-sharp wins.
    let (acc, tail) = if let Some(t) = rest.strip_prefix("eses") {
        (Accidental::DoubleFlat, t)
    } else if let Some(t) = rest.strip_prefix("ss") {
        (Accidental::DoubleSharp, t)
    } else if let Some(t) = rest.strip_prefix("es") {
        (Accidental::Flat, t)
    } else if let Some(t) = rest.strip_prefix("s") {
        (Accidental::Sharp, t)
    } else {
        (Accidental::Natural, rest)
    };

    // Exactly one remaining char, must be a digit 0–8.
    if tail.len() != 1 {
        return Err((input.into(), ()));
    }
    let oct_char = tail.as_bytes()[0];
    let octave: i32 = match oct_char {
        b'0'..=b'8' => (oct_char - b'0') as i32,
        _ => return Err((input.into(), ())),
    };

    let letter: Letter = match letter_char {
        'a' => Letter::A,
        'b' => Letter::B,
        'c' => Letter::C,
        'd' => Letter::D,
        'e' => Letter::E,
        'f' => Letter::F,
        'g' => Letter::G,
        _ => unreachable!(),
    };
    let note: Note = Spelling { letter, acc }
        .try_into()
        .map_err(|_| (input.into(), ()))?;

    Ok((note, octave))
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::note::Note;

    #[test]
    fn parses_c4() {
        assert_eq!(parse_pitch_shorthand("c4").unwrap(), (Note::C, 4));
    }

    #[test]
    fn parses_cs4_cis() {
        assert_eq!(parse_pitch_shorthand("cs4").unwrap(), (Note::Cis, 4));
    }

    #[test]
    fn parses_des4() {
        assert_eq!(parse_pitch_shorthand("des4").unwrap(), (Note::Des, 4));
    }

    #[test]
    fn parses_ess4_longest_match() {
        // Tokenizes as "e" + "ss" (double-sharp) + "4".
        // Note::Eisis doesn't exist (E double-sharp is an ExcessiveAccidental),
        // so the parser correctly rejects this spelling.
        assert!(parse_pitch_shorthand("ess4").is_err());
    }

    #[test]
    fn parses_aeses1_doubleflat() {
        assert_eq!(
            parse_pitch_shorthand("aeses1").unwrap(),
            (Note::Aeses, 1)
        );
    }

    #[test]
    fn parses_cisis0_doublesharp() {
        assert_eq!(
            parse_pitch_shorthand("css0").unwrap(),
            (Note::Cisis, 0)
        );
    }

    #[test]
    fn parses_ees3_flat() {
        assert_eq!(parse_pitch_shorthand("ees3").unwrap(), (Note::Ees, 3));
    }

    #[test]
    fn rejects_c12_double_digit_octave() {
        assert!(parse_pitch_shorthand("c12").is_err());
    }

    #[test]
    fn rejects_capital_letters() {
        assert!(parse_pitch_shorthand("C4").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_pitch_shorthand("").is_err());
    }

    #[test]
    fn rejects_octave_9() {
        assert!(parse_pitch_shorthand("c9").is_err());
    }
}
