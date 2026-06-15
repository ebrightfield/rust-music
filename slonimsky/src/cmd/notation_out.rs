//! Shared staff-notation output for melody-producing commands.
//!
//! Converts a generated melody (`Vec<MelodicEvent>`) into a `music-engraver`
//! `ScoreBuilder` and renders it to SVG or PNG, dispatched by output-file
//! extension. Also provides clef parsing and a key → key-signature mapper.
//!
//! This is the bridge that lets `sight-reading`, `rhythm-drill`, etc. emit real
//! staff notation in-process via the native engraver (replacing external
//! notation pipelines).

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use music::melody::sequencer::MelodicEvent;
use music::notation::clef::Clef;
use music::note::note::Note;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

/// Ticks in one 4/4 measure (whole note = 128 ticks in the `music` rhythm model).
const TICKS_PER_4_4_MEASURE: usize = 128;

/// User-selectable clef for notation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClefChoice {
    Treble,
    /// Octave-down treble clef (treble-8) — the standard guitar clef.
    Treble8,
    Bass,
}

impl ClefChoice {
    /// Parse a `--clef` value. Accepts `treble`, `treble-8`/`treble8`/`guitar`,
    /// and `bass`. Returns an error for unsupported clefs (e.g. alto/tenor,
    /// which `music::Clef` does not model).
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s.unwrap_or("treble").to_lowercase().replace('_', "-").as_str() {
            "treble" | "g" => Ok(ClefChoice::Treble),
            "treble-8" | "treble8" | "guitar" => Ok(ClefChoice::Treble8),
            "bass" | "f" => Ok(ClefChoice::Bass),
            "alto" | "tenor" => anyhow::bail!(
                "clef '{}' is not supported (music::Clef models only treble, treble-8, bass)",
                s.unwrap_or("")
            ),
            other => anyhow::bail!(
                "unknown clef: '{other}' (options: treble, treble-8, bass)"
            ),
        }
    }

    fn to_music_clef(self) -> Clef {
        match self {
            ClefChoice::Treble => Clef::Treble,
            ClefChoice::Treble8 => Clef::Treble8ba,
            ClefChoice::Bass => Clef::Bass,
        }
    }
}

/// Output format inferred from a file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Svg,
    Png,
}

impl OutputFormat {
    fn from_path(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "svg" => Ok(OutputFormat::Svg),
            "png" => Ok(OutputFormat::Png),
            "ly" => anyhow::bail!(
                "LilyPond (.ly) output is not wired into this command; use .svg or .png \
                 for native engraver output (LilyPond document generation lives behind \
                 the workspace `lilypond` feature)"
            ),
            "" => anyhow::bail!("output path '{}' has no extension; use .svg or .png", path.display()),
            other => anyhow::bail!("unsupported output format '.{other}' (use .svg or .png)"),
        }
    }
}

/// Map a key root + whether the scale is minor-flavored to a `KeySignature`.
///
/// Diatonic key signatures follow the circle of fifths. For non-major scales we
/// base the signature on the parent major key:
/// - melodic/harmonic minor → relative major of the natural minor (root + 3 semitones),
///   matching the conventional practice of notating minor with its key signature
///   and writing raised degrees as accidentals.
/// - harmonic major → same signature as major.
///
/// `root` is the spelled tonic note (its letter+accidental determine the key).
pub fn key_signature_for(root: Note, minor_flavored: bool) -> KeySignature {
    // Circle-of-fifths position for each *major* key, by spelled note.
    // Positive = sharps, negative = flats.
    let major_root = if minor_flavored {
        relative_major_root(root)
    } else {
        root
    };
    match fifths_position(major_root) {
        Some(0) => KeySignature::Open,
        Some(n) if n > 0 => KeySignature::Sharps(n as u8),
        Some(n) => KeySignature::Flats((-n) as u8),
        None => KeySignature::Open, // exotic spelling with no standard signature
    }
}

/// The relative-major tonic of a (natural) minor key: a minor third up.
/// Spelled to land on a standard key-signature note where possible.
fn relative_major_root(minor_root: Note) -> Note {
    use Note::*;
    // Map common minor tonics to their relative major tonic.
    match minor_root {
        A => C,
        E => G,
        B => D,
        Fis => A,
        Cis => E,
        Gis => B,
        Dis => Fis,
        Ais => Cis,
        D => F,
        G => Bes,
        C => Ees,
        F => Aes,
        Bes => Des,
        Ees => Ges,
        Aes => Ces,
        // Fallback: treat as no signature rather than guessing an exotic key.
        other => other,
    }
}

/// Number of sharps (+) or flats (−) for a major key with the given spelled
/// tonic. Returns `None` for spellings outside the standard 15 major keys.
fn fifths_position(major_root: Note) -> Option<i32> {
    use Note::*;
    Some(match major_root {
        C => 0,
        G => 1,
        D => 2,
        A => 3,
        E => 4,
        B => 5,
        Fis => 6,
        Cis => 7,
        F => -1,
        Bes => -2,
        Ees => -3,
        Aes => -4,
        Des => -5,
        Ges => -6,
        Ces => -7,
        _ => return None,
    })
}

/// Build a `ScoreBuilder` from a melody, splitting it into 4/4 measures by tick
/// total. Notes are emitted in order; ties are preserved.
fn build_score(
    melody: &[MelodicEvent],
    clef: ClefChoice,
    key_sig: KeySignature,
) -> ScoreBuilder {
    let mut sb = ScoreBuilder::new()
        .clef(clef.to_music_clef())
        .key_signature(key_sig)
        .time_signature(4, 4)
        .auto_line_breaks();

    let mut ticks_in_measure = 0usize;
    for (i, ev) in melody.iter().enumerate() {
        sb = sb.note(ev.pitch, ev.duration);
        if ev.tied {
            sb = sb.tie();
        }
        ticks_in_measure += ev.duration.ticks();
        // Close the measure once it's full (and we're not at the very end —
        // the final barline is added after the loop).
        if ticks_in_measure >= TICKS_PER_4_4_MEASURE && i + 1 < melody.len() {
            sb = sb.barline();
            ticks_in_measure = 0;
        }
    }
    sb.end_barline()
}

/// Render a melody to the given output path as SVG or PNG (by extension).
///
/// Returns the number of bytes written.
pub fn render_melody_to_file(
    melody: &[MelodicEvent],
    clef: ClefChoice,
    key_sig: KeySignature,
    path: &str,
) -> Result<usize> {
    let p = Path::new(path);
    let format = OutputFormat::from_path(p)?;
    let sb = build_score(melody, clef, key_sig);

    let bytes: Vec<u8> = match format {
        OutputFormat::Svg => sb
            .try_render_svg()
            .map_err(|e| anyhow::anyhow!("SVG render failed: {e}"))?
            .into_bytes(),
        OutputFormat::Png => sb
            .try_render_png(2.0)
            .map_err(|e| anyhow::anyhow!("PNG render failed: {e}"))?,
    };

    fs::write(p, &bytes).with_context(|| format!("failed to write {path}"))?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::notation::rhythm::duration::Duration;
    use music::note::pitch::Pitch;

    fn ev(note: Note, octave: i8, dur: Duration) -> MelodicEvent {
        MelodicEvent {
            pitch: Pitch::new(note, octave),
            duration: dur,
            tied: false,
        }
    }

    #[test]
    fn clef_parsing() {
        assert_eq!(ClefChoice::from_str_opt(None).unwrap(), ClefChoice::Treble);
        assert_eq!(ClefChoice::from_str_opt(Some("treble")).unwrap(), ClefChoice::Treble);
        assert_eq!(ClefChoice::from_str_opt(Some("treble-8")).unwrap(), ClefChoice::Treble8);
        assert_eq!(ClefChoice::from_str_opt(Some("treble8")).unwrap(), ClefChoice::Treble8);
        assert_eq!(ClefChoice::from_str_opt(Some("guitar")).unwrap(), ClefChoice::Treble8);
        assert_eq!(ClefChoice::from_str_opt(Some("bass")).unwrap(), ClefChoice::Bass);
        assert!(ClefChoice::from_str_opt(Some("alto")).is_err());
        assert!(ClefChoice::from_str_opt(Some("nonsense")).is_err());
    }

    #[test]
    fn major_key_signatures() {
        assert_eq!(key_signature_for(Note::C, false), KeySignature::Open);
        assert_eq!(key_signature_for(Note::G, false), KeySignature::Sharps(1));
        assert_eq!(key_signature_for(Note::D, false), KeySignature::Sharps(2));
        assert_eq!(key_signature_for(Note::F, false), KeySignature::Flats(1));
        assert_eq!(key_signature_for(Note::Bes, false), KeySignature::Flats(2));
        assert_eq!(key_signature_for(Note::Ees, false), KeySignature::Flats(3));
        assert_eq!(key_signature_for(Note::Cis, false), KeySignature::Sharps(7));
    }

    #[test]
    fn minor_key_signatures_use_relative_major() {
        // A minor → C major signature (open)
        assert_eq!(key_signature_for(Note::A, true), KeySignature::Open);
        // E minor → G major (1 sharp)
        assert_eq!(key_signature_for(Note::E, true), KeySignature::Sharps(1));
        // G minor → Bb major (2 flats)
        assert_eq!(key_signature_for(Note::G, true), KeySignature::Flats(2));
        // D minor → F major (1 flat)
        assert_eq!(key_signature_for(Note::D, true), KeySignature::Flats(1));
    }

    #[test]
    fn output_format_from_extension() {
        assert!(OutputFormat::from_path(Path::new("x.svg")).is_ok());
        assert!(OutputFormat::from_path(Path::new("x.png")).is_ok());
        assert!(OutputFormat::from_path(Path::new("x.ly")).is_err());
        assert!(OutputFormat::from_path(Path::new("x")).is_err());
        assert!(OutputFormat::from_path(Path::new("x.txt")).is_err());
    }

    #[test]
    fn renders_svg() {
        let melody = vec![
            ev(Note::C, 4, Duration::QTR),
            ev(Note::D, 4, Duration::QTR),
            ev(Note::E, 4, Duration::QTR),
            ev(Note::F, 4, Duration::QTR),
            ev(Note::G, 4, Duration::QTR),
        ];
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("m.svg");
        let n = render_melody_to_file(
            &melody,
            ClefChoice::Treble,
            KeySignature::Open,
            out.to_str().unwrap(),
        )
        .unwrap();
        assert!(n > 0);
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with("<svg") || content.contains("<svg"));
        assert!(content.contains("</svg>"));
    }

    #[test]
    fn renders_png() {
        let melody = vec![
            ev(Note::C, 4, Duration::HALF),
            ev(Note::E, 4, Duration::HALF),
        ];
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("m.png");
        let n = render_melody_to_file(
            &melody,
            ClefChoice::Bass,
            KeySignature::Sharps(2),
            out.to_str().unwrap(),
        )
        .unwrap();
        assert!(n > 0);
        let bytes = std::fs::read(&out).unwrap();
        // PNG magic number.
        assert_eq!(&bytes[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}
