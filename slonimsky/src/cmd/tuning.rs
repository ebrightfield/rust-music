use anyhow::{bail, Context, Result};
use music::fretboard::{
    Fretboard, BASS_4, BASS_5, DADGAD, DROP_D, OPEN_G, STANDARD_7, STD_6STR_GTR,
};
use music::note::note::Note;
use music::note::pitch::Pitch;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// A resolved fretboard tuning and the stable label used in command output.
#[derive(Clone, Debug)]
pub struct TuningSpec {
    pub fretboard: Fretboard,
    pub label: String,
}

impl TuningSpec {
    /// Resolve a named tuning, an inline comma-separated pitch list, or `@path`.
    ///
    /// Pitch tokens may be spelled pitches with octaves (`E2`, `Bb3`) or MIDI
    /// note numbers (`40`, `58`). Files use the same comma/whitespace-separated
    /// token syntax as inline values.
    pub fn parse(value: &str) -> Result<Self> {
        let value = value.trim();
        if value.is_empty() {
            bail!("tuning cannot be empty");
        }

        if let Some(fretboard) = named_tuning(value) {
            return Ok(Self {
                fretboard: fretboard.clone(),
                label: canonical_name(value).to_string(),
            });
        }

        if let Some(path) = value.strip_prefix('@') {
            if path.is_empty() {
                bail!("tuning file path after '@' cannot be empty");
            }
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read tuning file '{path}'"))?;
            let fretboard = parse_pitch_list(&contents)
                .with_context(|| format!("invalid tuning file '{path}'"))?;
            return Ok(Self {
                fretboard,
                label: format!("@{}", Path::new(path).display()),
            });
        }

        Ok(Self {
            fretboard: parse_pitch_list(value).with_context(|| {
                format!(
                    "invalid tuning '{value}'; use a named tuning, comma-separated pitches/MIDI values, or @path"
                )
            })?,
            label: value.to_string(),
        })
    }
}

fn named_tuning(value: &str) -> Option<&'static Fretboard> {
    match value.to_ascii_lowercase().as_str() {
        "standard" => Some(&STD_6STR_GTR),
        "drop-d" => Some(&DROP_D),
        "dadgad" => Some(&DADGAD),
        "open-g" => Some(&OPEN_G),
        "7-string" => Some(&STANDARD_7),
        "bass-4" => Some(&BASS_4),
        "bass-5" => Some(&BASS_5),
        _ => None,
    }
}

fn canonical_name(value: &str) -> &str {
    match value.to_ascii_lowercase().as_str() {
        "standard" => "standard",
        "drop-d" => "drop-d",
        "dadgad" => "dadgad",
        "open-g" => "open-g",
        "7-string" => "7-string",
        "bass-4" => "bass-4",
        "bass-5" => "bass-5",
        _ => unreachable!("only called after named_tuning"),
    }
}

fn parse_pitch_list(value: &str) -> Result<Fretboard> {
    let tokens: Vec<&str> = value
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect();
    if tokens.is_empty() {
        bail!("tuning contains no pitches");
    }

    let open_strings = tokens
        .into_iter()
        .map(parse_pitch)
        .collect::<Result<Vec<_>>>()?;
    if open_strings.len() > u8::MAX as usize {
        bail!("tuning has too many strings");
    }
    Ok(Fretboard { open_strings })
}

fn parse_pitch(token: &str) -> Result<Pitch> {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        let midi = token
            .parse::<u8>()
            .with_context(|| format!("MIDI pitch '{token}' must be between 0 and 127"))?;
        return Pitch::from_midi(midi)
            .with_context(|| format!("MIDI pitch '{token}' must be between 0 and 127"));
    }

    let octave_start = token
        .char_indices()
        .find(|(index, ch)| {
            ch.is_ascii_digit()
                || (*ch == '-'
                    && token[*index + ch.len_utf8()..]
                        .starts_with(|next: char| next.is_ascii_digit()))
        })
        .map(|(index, _)| index)
        .ok_or_else(|| anyhow::anyhow!("pitch '{token}' needs an octave, for example E2 or Bb3"))?;
    let (note, octave) = token.split_at(octave_start);
    let note = Note::from_str(note).with_context(|| format!("invalid note in pitch '{token}'"))?;
    let octave = octave
        .parse::<i8>()
        .with_context(|| format!("invalid octave in pitch '{token}'"))?;
    Pitch::try_new(note, octave).with_context(|| format!("pitch '{token}' is outside MIDI range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_tuning() {
        let spec = TuningSpec::parse("drop-d").unwrap();
        assert_eq!(spec.fretboard.num_strings(), 6);
        assert_eq!(spec.fretboard.open_strings[0].to_string(), "D3");
    }

    #[test]
    fn parses_spelled_inline_tuning() {
        let spec = TuningSpec::parse("E2,A2,D3,G3").unwrap();
        assert_eq!(spec.fretboard.num_strings(), 4);
        assert_eq!(spec.fretboard.open_strings[1].midi_note, 45);
    }

    #[test]
    fn parses_midi_inline_tuning() {
        let spec = TuningSpec::parse("40,45,50,55").unwrap();
        assert_eq!(spec.fretboard.num_strings(), 4);
        assert_eq!(spec.fretboard.open_strings[3].to_string(), "G3");
    }

    #[test]
    fn parses_tuning_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("tuning.txt");
        fs::write(&path, "D2 A2\nD3 G3 B3 E4\n").unwrap();
        let spec = TuningSpec::parse(&format!("@{}", path.display())).unwrap();
        assert_eq!(spec.fretboard.num_strings(), 6);
        assert_eq!(spec.fretboard.open_strings[0].to_string(), "D2");
    }

    #[test]
    fn rejects_pitch_without_octave() {
        assert!(TuningSpec::parse("E,A,D,G").is_err());
    }
}
