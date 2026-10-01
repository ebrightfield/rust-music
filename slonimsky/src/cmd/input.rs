use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::svg::SvgTheme;

/// Parse a single input token into a Pc.
/// Accepts integer (0–11), note name (C, C#, Db, …), or returns None.
pub fn parse_pc(token: &str) -> Option<Pc> {
    // Try integer first
    if let Ok(n) = token.parse::<u8>() {
        if n < 12 {
            return Some(Pc::from(n));
        }
    }
    // Try note names (case-insensitive first char, then accidentals)
    let t = token.trim();
    match t.to_lowercase().as_str() {
        "c" => Some(Pc::Pc0),
        "c#" | "cis" | "c♯" => Some(Pc::Pc1),
        "db" | "des" | "d♭" => Some(Pc::Pc1),
        "d" => Some(Pc::Pc2),
        "d#" | "dis" | "d♯" => Some(Pc::Pc3),
        "eb" | "es" | "e♭" => Some(Pc::Pc3),
        "e" => Some(Pc::Pc4),
        "f" => Some(Pc::Pc5),
        "f#" | "fis" | "f♯" => Some(Pc::Pc6),
        "gb" | "ges" | "g♭" => Some(Pc::Pc6),
        "g" => Some(Pc::Pc7),
        "g#" | "gis" | "g♯" => Some(Pc::Pc8),
        "ab" | "as" | "aes" | "a♭" => Some(Pc::Pc8),
        "a" => Some(Pc::Pc9),
        "a#" | "ais" | "a♯" => Some(Pc::Pc10),
        "bb" | "bes" | "b♭" => Some(Pc::Pc10),
        "b" => Some(Pc::Pc11),
        _ => None,
    }
}

/// Parse the input token list into a Vec<Pc>.
/// Tokens may be integers, note names, or comma-separated groups.
pub fn parse_input_to_pcs(tokens: &[String]) -> Result<Vec<Pc>> {
    let mut pcs = Vec::new();
    for token in tokens {
        // Split on commas to allow `0,4,7` as a single arg
        for part in token.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let pc =
                parse_pc(part).with_context(|| format!("unrecognized pitch class: '{part}'"))?;
            pcs.push(pc);
        }
    }
    anyhow::ensure!(!pcs.is_empty(), "no pitch classes provided");
    Ok(pcs)
}

/// Map a pitch class to a human-readable label (e.g. "C", "C#/Db").
pub fn pc_label(pc: Pc) -> &'static str {
    match pc {
        Pc::Pc0 => "C",
        Pc::Pc1 => "C#/Db",
        Pc::Pc2 => "D",
        Pc::Pc3 => "D#/Eb",
        Pc::Pc4 => "E",
        Pc::Pc5 => "F",
        Pc::Pc6 => "F#/Gb",
        Pc::Pc7 => "G",
        Pc::Pc8 => "G#/Ab",
        Pc::Pc9 => "A",
        Pc::Pc10 => "A#/Bb",
        Pc::Pc11 => "B",
    }
}

/// Resolve a theme name string to an SvgTheme.
pub fn resolve_theme(name: Option<&str>) -> Result<SvgTheme> {
    match name {
        None | Some("default") => Ok(SvgTheme::default()),
        Some("dark") => Ok(SvgTheme::dark()),
        Some("print") => Ok(SvgTheme::print()),
        Some("colorful") => Ok(SvgTheme::colorful()),
        Some(other) => {
            anyhow::bail!("unknown theme: '{other}' (options: default, dark, print, colorful)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integers() {
        let pcs = parse_input_to_pcs(&["0".into(), "4".into(), "7".into()]).unwrap();
        assert_eq!(pcs, vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
    }

    #[test]
    fn parse_note_names() {
        let pcs = parse_input_to_pcs(&["C".into(), "E".into(), "G".into()]).unwrap();
        assert_eq!(pcs, vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
    }

    #[test]
    fn parse_comma_separated() {
        let pcs = parse_input_to_pcs(&["0,4,7".into()]).unwrap();
        assert_eq!(pcs, vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
    }

    #[test]
    fn parse_sharps_flats() {
        let pcs = parse_input_to_pcs(&["C#".into(), "Eb".into(), "Bb".into()]).unwrap();
        assert_eq!(pcs, vec![Pc::Pc1, Pc::Pc3, Pc::Pc10]);
    }

    #[test]
    fn parse_rejects_unknown() {
        assert!(parse_input_to_pcs(&["xyz".into()]).is_err());
    }

    #[test]
    fn parse_empty_rejects() {
        assert!(parse_input_to_pcs(&[]).is_err());
    }

    #[test]
    fn parse_pc_boundary_values() {
        assert_eq!(parse_pc("0"), Some(Pc::Pc0));
        assert_eq!(parse_pc("11"), Some(Pc::Pc11));
        assert_eq!(parse_pc("12"), None);
    }

    #[test]
    fn pc_label_all_12() {
        let labels: Vec<&str> = (0..12u8).map(|i| pc_label(Pc::from(i))).collect();
        assert_eq!(labels[0], "C");
        assert_eq!(labels[6], "F#/Gb");
        assert_eq!(labels[11], "B");
        assert_eq!(labels.len(), 12);
        // All labels are non-empty and unique
        let unique: std::collections::HashSet<&str> = labels.iter().copied().collect();
        assert_eq!(unique.len(), 12);
    }

    #[test]
    fn resolve_theme_all_variants() {
        assert!(resolve_theme(None).is_ok());
        assert!(resolve_theme(Some("default")).is_ok());
        assert!(resolve_theme(Some("dark")).is_ok());
        assert!(resolve_theme(Some("print")).is_ok());
        assert!(resolve_theme(Some("colorful")).is_ok());
        assert!(resolve_theme(Some("nope")).is_err());
    }
}
