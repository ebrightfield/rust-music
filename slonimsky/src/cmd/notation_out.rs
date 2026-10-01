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
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

/// Ticks in a whole note in the `music` rhythm model.
const TICKS_PER_WHOLE: usize = 128;

/// Ticks in one quarter note.
const TICKS_PER_QUARTER: usize = TICKS_PER_WHOLE / 4;

/// Measures per system for generated exercise sheets. Four is the usual
/// practice-sheet grid and keeps a 4-bar phrase on one line.
const MEASURES_PER_SYSTEM: usize = 4;

/// One staff space in Bravura font design units.
const STAFF_SPACE_FU: f64 = 250.0;

/// System width in staff spaces. Wide enough that four subdivision-heavy
/// measures fit without the engraver compressing them.
const SYSTEM_WIDTH_SS: f64 = 140.0;

/// One notated event: a pitched note or a rest.
///
/// The melody types in `music` carry only pitched events, so rests would
/// otherwise have to be dropped — which silently desynchronizes measures from
/// the rhythm they are supposed to notate. Commands build a stream of these
/// instead so rests reach the engraver as rests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotationEvent {
    Note {
        pitch: Pitch,
        duration: Duration,
        /// Tied to the following note.
        tied: bool,
    },
    Rest {
        duration: Duration,
    },
}

impl NotationEvent {
    pub fn note(pitch: Pitch, duration: Duration) -> Self {
        NotationEvent::Note {
            pitch,
            duration,
            tied: false,
        }
    }

    pub fn rest(duration: Duration) -> Self {
        NotationEvent::Rest { duration }
    }

    pub fn duration(&self) -> Duration {
        match self {
            NotationEvent::Note { duration, .. } => *duration,
            NotationEvent::Rest { duration } => *duration,
        }
    }

    fn ticks(&self) -> usize {
        self.duration().ticks()
    }

    /// Whether this event is short enough to be beamed (eighth or shorter).
    fn beamable(&self) -> bool {
        matches!(self, NotationEvent::Note { .. })
            && self.duration().ticks() < TICKS_PER_QUARTER
            && !matches!(
                self.duration().kind(),
                DurationKind::Whole | DurationKind::Half
            )
    }
}

impl From<&MelodicEvent> for NotationEvent {
    fn from(ev: &MelodicEvent) -> Self {
        NotationEvent::Note {
            pitch: ev.pitch,
            duration: ev.duration,
            tied: ev.tied,
        }
    }
}

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
        match s
            .unwrap_or("treble")
            .to_lowercase()
            .replace('_', "-")
            .as_str()
        {
            "treble" | "g" => Ok(ClefChoice::Treble),
            "treble-8" | "treble8" | "guitar" => Ok(ClefChoice::Treble8),
            "bass" | "f" => Ok(ClefChoice::Bass),
            "alto" | "tenor" => anyhow::bail!(
                "clef '{}' is not supported (music::Clef models only treble, treble-8, bass)",
                s.unwrap_or("")
            ),
            other => anyhow::bail!("unknown clef: '{other}' (options: treble, treble-8, bass)"),
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
            "" => anyhow::bail!(
                "output path '{}' has no extension; use .svg or .png",
                path.display()
            ),
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

/// Split an event stream into measures of `ticks_per_measure`.
///
/// Events are assumed to align to measure boundaries (the generators build them
/// beat by beat). Any trailing partial measure is kept as its own measure so
/// nothing is silently dropped.
fn split_measures(events: &[NotationEvent], ticks_per_measure: usize) -> Vec<Vec<NotationEvent>> {
    let mut measures = Vec::new();
    let mut current: Vec<NotationEvent> = Vec::new();
    let mut ticks = 0usize;

    for ev in events {
        current.push(*ev);
        ticks += ev.ticks();
        if ticks >= ticks_per_measure {
            measures.push(std::mem::take(&mut current));
            ticks = 0;
        }
    }
    if !current.is_empty() {
        measures.push(current);
    }
    measures
}

/// Partition one measure into beat-aligned runs of beamable notes.
///
/// Returns a list of runs; each run is either a single event (rendered on its
/// own, with a flag if it is short) or two-or-more consecutive beamable notes
/// that share a beam. Beams never cross a beat boundary, which is the
/// conventional grouping for simple meters and keeps `♫`-style pairs together.
fn beam_runs(measure: &[NotationEvent], beat_ticks: usize) -> Vec<Vec<NotationEvent>> {
    let mut runs: Vec<Vec<NotationEvent>> = Vec::new();
    let mut run: Vec<NotationEvent> = Vec::new();
    // Tick offset of the current event from the start of the measure.
    let mut offset = 0usize;

    for ev in measure {
        let starts_new_beat = beat_ticks > 0 && offset % beat_ticks == 0;
        let breaks_run = !ev.beamable()
            // A beam group must stay inside one beat.
            || (starts_new_beat && !run.is_empty());

        if breaks_run && !run.is_empty() {
            runs.push(std::mem::take(&mut run));
        }

        if ev.beamable() {
            run.push(*ev);
        } else {
            runs.push(vec![*ev]);
        }

        offset += ev.ticks();
    }
    if !run.is_empty() {
        runs.push(run);
    }
    runs
}

/// Build a `ScoreBuilder` from a notation event stream.
///
/// Measures are split by the given time signature, subdivisions are beamed
/// within each beat, and rests are emitted as rests. Ties are preserved.
fn build_score(
    events: &[NotationEvent],
    clef: ClefChoice,
    key_sig: KeySignature,
    (num, den): (u8, u8),
) -> ScoreBuilder {
    // Fixed measures per system, and a system wide enough to hold them.
    //
    // Gourlay spacing gives a subdivision-heavy measure a natural width close
    // to the default 40-staff-space system, which would put one measure on each
    // line and break systems at inconsistent places. Practice sheets want a
    // predictable grid instead, so ask for a set number of measures per system
    // and widen the system to fit them.
    let mut sb = ScoreBuilder::new()
        .clef(clef.to_music_clef())
        .key_signature(key_sig)
        .time_signature(num, den)
        .measures_per_system(MEASURES_PER_SYSTEM)
        .system_width_fu(SYSTEM_WIDTH_SS * STAFF_SPACE_FU);

    // Measure length and beat length, in ticks, from the time signature.
    let ticks_per_measure = TICKS_PER_WHOLE * num as usize / den as usize;
    let beat_ticks = TICKS_PER_WHOLE / den as usize;

    let measures = split_measures(events, ticks_per_measure);
    let last = measures.len().saturating_sub(1);

    for (m_idx, measure) in measures.iter().enumerate() {
        for run in beam_runs(measure, beat_ticks) {
            if run.len() > 1 {
                // Two or more beamable notes in the same beat → one beam group.
                let notes: Vec<(Pitch, Duration)> = run
                    .iter()
                    .filter_map(|ev| match ev {
                        NotationEvent::Note {
                            pitch, duration, ..
                        } => Some((*pitch, *duration)),
                        NotationEvent::Rest { .. } => None,
                    })
                    .collect();
                sb = sb.beam_group(notes);
                continue;
            }

            match run[0] {
                NotationEvent::Note {
                    pitch,
                    duration,
                    tied,
                } => {
                    sb = sb.note(pitch, duration);
                    if tied {
                        sb = sb.tie();
                    }
                }
                NotationEvent::Rest { duration } => {
                    sb = sb.rest(duration);
                }
            }
        }

        // Barline at every measure boundary; the last one is the final double bar.
        sb = if m_idx == last {
            sb.end_barline()
        } else {
            sb.barline()
        };
    }

    // An empty stream still needs a closed measure to render a valid staff.
    if measures.is_empty() {
        sb = sb.end_barline();
    }
    sb
}

/// Render a melody to the given output path as SVG or PNG (by extension).
///
/// Assumes 4/4; use [`render_events_to_file`] to supply a time signature or to
/// include rests.
///
/// Returns the number of bytes written.
pub fn render_melody_to_file(
    melody: &[MelodicEvent],
    clef: ClefChoice,
    key_sig: KeySignature,
    path: &str,
) -> Result<usize> {
    let events: Vec<NotationEvent> = melody.iter().map(NotationEvent::from).collect();
    render_events_to_file(&events, clef, key_sig, (4, 4), path)
}

/// Render a notation event stream (notes and rests) to SVG or PNG by extension.
///
/// `time_sig` drives both measure splitting and beam grouping.
///
/// Returns the number of bytes written.
pub fn render_events_to_file(
    events: &[NotationEvent],
    clef: ClefChoice,
    key_sig: KeySignature,
    time_sig: (u8, u8),
    path: &str,
) -> Result<usize> {
    let p = Path::new(path);
    let format = OutputFormat::from_path(p)?;
    let sb = build_score(events, clef, key_sig, time_sig);

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

    /// A note event on a fixed pitch, for rhythm-shaped tests.
    fn n(dur: Duration) -> NotationEvent {
        NotationEvent::note(Pitch::new(Note::B, 4), dur)
    }

    fn r(dur: Duration) -> NotationEvent {
        NotationEvent::rest(dur)
    }

    fn dotted_eighth() -> Duration {
        Duration::new(DurationKind::Eighth, 1)
    }

    /// A 4/4 measure's worth of events becomes exactly one measure.
    #[test]
    fn splits_on_measure_boundaries() {
        let four_quarters = vec![n(Duration::QTR); 4];
        let measures = split_measures(&four_quarters, TICKS_PER_WHOLE);
        assert_eq!(measures.len(), 1);

        let two_bars = vec![n(Duration::QTR); 8];
        assert_eq!(split_measures(&two_bars, TICKS_PER_WHOLE).len(), 2);
    }

    /// Rests count toward the measure just like notes — dropping them used to
    /// stretch every measure that contained one.
    #[test]
    fn rests_fill_measures_like_notes() {
        // Quarter, eighth, eighth-rest, quarter, quarter = one 4/4 bar.
        let bar = vec![
            n(Duration::QTR),
            n(Duration::EIGHTH),
            r(Duration::EIGHTH),
            n(Duration::QTR),
            n(Duration::QTR),
        ];
        let total: usize = bar.iter().map(|e| e.ticks()).sum();
        assert_eq!(total, TICKS_PER_WHOLE);
        assert_eq!(split_measures(&bar, TICKS_PER_WHOLE).len(), 1);
    }

    /// Consecutive eighths inside one beat beam together; quarters never do.
    #[test]
    fn beams_group_within_a_beat() {
        let bar = vec![
            n(Duration::QTR),    // beat 1: alone
            n(Duration::EIGHTH), // beat 2: pair
            n(Duration::EIGHTH),
            n(Duration::QTR),    // beat 3: alone
            n(Duration::EIGHTH), // beat 4: pair
            n(Duration::EIGHTH),
        ];
        let runs = beam_runs(&bar, TICKS_PER_QUARTER);
        let sizes: Vec<usize> = runs.iter().map(|r| r.len()).collect();
        assert_eq!(
            sizes,
            vec![1, 2, 1, 2],
            "expected quarter, beam, quarter, beam"
        );
    }

    /// A beam never spans a beat boundary: eight straight eighths in 4/4 make
    /// four beamed pairs, not one eight-note beam.
    #[test]
    fn beams_do_not_cross_beats() {
        let bar = vec![n(Duration::EIGHTH); 8];
        let runs = beam_runs(&bar, TICKS_PER_QUARTER);
        assert_eq!(runs.len(), 4);
        assert!(runs.iter().all(|r| r.len() == 2));
    }

    /// Rests break a beam group rather than being swept into it.
    #[test]
    fn rests_break_beam_groups() {
        // Eighth-rest then eighth (a syncopated cell): no beam, two runs.
        let beat = vec![r(Duration::EIGHTH), n(Duration::EIGHTH)];
        let runs = beam_runs(&beat, TICKS_PER_QUARTER);
        assert_eq!(runs.len(), 2);
        assert!(runs.iter().all(|r| r.len() == 1));
    }

    /// Four sixteenths in a beat beam as one group; a dotted-eighth + sixteenth
    /// also beams (the engraver draws the fractional stub).
    #[test]
    fn sixteenth_figures_beam() {
        let four = vec![n(Duration::SIXTEENTH); 4];
        assert_eq!(beam_runs(&four, TICKS_PER_QUARTER)[0].len(), 4);

        let dotted = vec![n(dotted_eighth()), n(Duration::SIXTEENTH)];
        let runs = beam_runs(&dotted, TICKS_PER_QUARTER);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].len(), 2);
    }

    /// Quarters and longer are never beamed.
    #[test]
    fn long_notes_are_not_beamable() {
        assert!(!n(Duration::QTR).beamable());
        assert!(!n(Duration::HALF).beamable());
        assert!(!n(Duration::WHOLE).beamable());
        assert!(n(Duration::EIGHTH).beamable());
        assert!(n(Duration::SIXTEENTH).beamable());
        assert!(!r(Duration::EIGHTH).beamable(), "rests are never beamed");
    }

    /// A rhythm containing rests renders, and every measure gets a barline —
    /// the engraved bar count must match the requested measure count.
    #[test]
    fn renders_rests_and_barlines() {
        let mut events = Vec::new();
        for _ in 0..3 {
            // beat 1: quarter · beat 2: beamed eighth pair
            // beat 3: eighth + eighth-rest · beat 4: quarter
            events.push(n(Duration::QTR));
            events.push(n(Duration::EIGHTH));
            events.push(n(Duration::EIGHTH));
            events.push(n(Duration::EIGHTH));
            events.push(r(Duration::EIGHTH));
            events.push(n(Duration::QTR));
        }
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("rests.svg");
        render_events_to_file(
            &events,
            ClefChoice::Treble,
            KeySignature::Open,
            (4, 4),
            out.to_str().unwrap(),
        )
        .unwrap();
        let svg = std::fs::read_to_string(&out).unwrap();
        // Beams render as filled polygons; rests and noteheads as paths.
        assert!(svg.contains("<polygon"), "expected at least one beam");
        assert!(svg.contains("<svg") && svg.contains("</svg>"));
    }

    /// 3/4 splits into three-beat measures rather than four.
    #[test]
    fn honors_time_signature_for_measures() {
        let bar_3_4 = vec![n(Duration::QTR); 3];
        let ticks_3_4 = TICKS_PER_WHOLE * 3 / 4;
        assert_eq!(split_measures(&bar_3_4, ticks_3_4).len(), 1);
        let two_bars = vec![n(Duration::QTR); 6];
        assert_eq!(split_measures(&two_bars, ticks_3_4).len(), 2);
    }

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
        assert_eq!(
            ClefChoice::from_str_opt(Some("treble")).unwrap(),
            ClefChoice::Treble
        );
        assert_eq!(
            ClefChoice::from_str_opt(Some("treble-8")).unwrap(),
            ClefChoice::Treble8
        );
        assert_eq!(
            ClefChoice::from_str_opt(Some("treble8")).unwrap(),
            ClefChoice::Treble8
        );
        assert_eq!(
            ClefChoice::from_str_opt(Some("guitar")).unwrap(),
            ClefChoice::Treble8
        );
        assert_eq!(
            ClefChoice::from_str_opt(Some("bass")).unwrap(),
            ClefChoice::Bass
        );
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
        assert_eq!(
            &bytes[..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
