use anyhow::{Context, Result};
use music::melody::context::ChordProgression;
use music::melody::pattern::IntervalPattern;
use music::melody::sequencer::{MelodicEvent, MelodicSequencer, MelodicSequencerConfig};
use music::melody::{Direction, PitchBounds, TurnaroundMode};
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcShape;
use music::note_collections::spelling::spell_shape;
use music::note_collections::NoteSet;
use music::note_collections::OctavePartition;
use musical_combinatorics::seven_note_scales::SevenNoteScaleQuality;

use super::input::parse_pc;
use super::notation_out::{key_signature_for, render_melody_to_file, ClefChoice};

/// Whether a scale quality should be notated with a minor-style key signature
/// (based on its parent natural minor). Melodic/harmonic minor are
/// minor-flavored; major and harmonic major use the major signature.
fn is_minor_flavored(quality: &SevenNoteScaleQuality) -> bool {
    matches!(
        quality,
        SevenNoteScaleQuality::MelodicMinor | SevenNoteScaleQuality::HarmonicMinor
    )
}

pub struct SightReadingArgs {
    pub key: Option<String>,
    pub scale: Option<String>,
    pub difficulty: u8,
    pub measures: usize,
    pub seed: Option<u64>,
    pub clef: Option<String>,
    pub output: Option<String>,
    pub verbose: bool,
}

/// Resolve a key name to a root `Note`, preserving the user's spelling intent.
///
/// When the user types a note name (`Bb`, `F#`, `Eb`, …) we honor that exact
/// spelling so the key reads and notates conventionally (`Bb major`, not
/// `A# major`). For an integer pitch-class input we fall back to the
/// conventional key spelling for that pitch class (flat-preferring for the keys
/// usually written with flats).
fn resolve_key(key_str: Option<&str>) -> Result<Note> {
    let s = key_str.unwrap_or("C").trim();
    // First, honor an explicit spelled note name.
    if let Some(note) = parse_key_note(s) {
        return Ok(note);
    }
    // Otherwise it must be an integer pitch class — map to a conventional key.
    let pc = parse_pc(s).with_context(|| format!("Invalid key: '{s}'"))?;
    Ok(conventional_key_note(pc))
}

/// Parse an explicit note-name key into a spelled `Note`, or `None` if the
/// token isn't a recognized note name (e.g. an integer pitch class).
fn parse_key_note(token: &str) -> Option<Note> {
    match token.to_lowercase().as_str() {
        "c" => Some(Note::C),
        "c#" | "cis" | "c♯" => Some(Note::Cis),
        "db" | "des" | "d♭" => Some(Note::Des),
        "d" => Some(Note::D),
        "d#" | "dis" | "d♯" => Some(Note::Dis),
        "eb" | "es" | "ees" | "e♭" => Some(Note::Ees),
        "e" => Some(Note::E),
        "f" => Some(Note::F),
        "f#" | "fis" | "f♯" => Some(Note::Fis),
        "gb" | "ges" | "g♭" => Some(Note::Ges),
        "g" => Some(Note::G),
        "g#" | "gis" | "g♯" => Some(Note::Gis),
        "ab" | "as" | "aes" | "a♭" => Some(Note::Aes),
        "a" => Some(Note::A),
        "a#" | "ais" | "a♯" => Some(Note::Ais),
        "bb" | "bes" | "b♭" => Some(Note::Bes),
        "b" => Some(Note::B),
        _ => None,
    }
}

/// Conventional major-key spelling for a pitch class given only an integer.
/// Prefers the flat spelling for the keys usually written with flats
/// (Db, Eb, Ab, Bb) and sharps elsewhere — matching how players name keys.
fn conventional_key_note(pc: Pc) -> Note {
    match u8::from(&pc) {
        0 => Note::C,
        1 => Note::Des, // Db major
        2 => Note::D,
        3 => Note::Ees, // Eb major
        4 => Note::E,
        5 => Note::F,
        6 => Note::Fis, // F# major (could be Gb; F# is the common choice)
        7 => Note::G,
        8 => Note::Aes, // Ab major
        9 => Note::A,
        10 => Note::Bes, // Bb major
        _ => Note::B,
    }
}

/// Resolve scale name to a SevenNoteScaleQuality.
fn resolve_scale(name: Option<&str>) -> Result<SevenNoteScaleQuality> {
    let s = name.unwrap_or("major");
    match s.to_lowercase().replace('-', " ").as_str() {
        "major" | "ionian" => Ok(SevenNoteScaleQuality::Major),
        "melodic minor" | "melodic_minor" => Ok(SevenNoteScaleQuality::MelodicMinor),
        "harmonic minor" | "harmonic_minor" => Ok(SevenNoteScaleQuality::HarmonicMinor),
        "harmonic major" | "harmonic_major" => Ok(SevenNoteScaleQuality::HarmonicMajor),
        _ => anyhow::bail!(
            "unknown scale: '{s}' (options: major, melodic-minor, harmonic-minor, harmonic-major)"
        ),
    }
}

/// Build a NoteSet for the given key and scale quality, with key-aware
/// enharmonic spelling.
///
/// The scale's interval template (a root-relative [`PcShape`]) is spelled
/// against `root` via [`spell_shape`], which assigns each scale degree a
/// distinct letter name and the correct accidental for the key (e.g. Bb major
/// → `Bb C D Eb F G A`, not `A# C D D# F G A`). The resulting notes are anchored
/// to `root` so the sequencer's step logic starts on the tonic.
///
/// Errors only when `root` is a double-accidental spelling (not a valid
/// practice key), propagated from [`spell_shape`].
fn build_scale_noteset(root: Note, quality: SevenNoteScaleQuality) -> Result<NoteSet> {
    let partition = OctavePartition::from(&quality);
    let parent_shape = PcShape::from(&partition);
    let notes = spell_shape(&root, &parent_shape)
        .map_err(|e| anyhow::anyhow!("cannot spell {root} scale: {e:?}"))?;
    Ok(NoteSet::with_root(notes, &root))
}

/// Build an IntervalPattern and rhythm pattern based on difficulty (1–5).
fn difficulty_config(difficulty: u8, seed: Option<u64>) -> (IntervalPattern, Vec<Duration>) {
    // Use seed for deterministic pseudo-random shuffling of patterns
    let seed_val = seed.unwrap_or(0);
    // Rotate the pattern based on seed to get variety
    let offset = (seed_val % 8) as usize;

    match difficulty {
        1 => {
            // Stepwise only, quarter notes — simplest reading exercise
            let intervals = rotate_pattern(&[1, 1, 1, 1, -1, 1, -1, -1], offset);
            let pattern = IntervalPattern::simple(intervals, 1);
            let rhythm = vec![Duration::QTR];
            (pattern, rhythm)
        }
        2 => {
            // Steps + one skip, quarter + eighth notes
            let intervals = rotate_pattern(&[1, 1, 2, 1, -1, 1, -1, -2], offset);
            let pattern = IntervalPattern::simple(intervals, 1);
            let rhythm = vec![Duration::QTR, Duration::EIGHTH, Duration::EIGHTH];
            (pattern, rhythm)
        }
        3 => {
            // Steps + skips + thirds, mixed rhythms
            let intervals = rotate_pattern(&[1, 2, -1, 3, 1, -2, 1, -1], offset);
            let pattern = IntervalPattern::simple(intervals, 1);
            let rhythm = vec![
                Duration::QTR,
                Duration::EIGHTH,
                Duration::EIGHTH,
                Duration::HALF,
                Duration::QTR,
            ];
            (pattern, rhythm)
        }
        4 => {
            // Wider intervals including fourths, syncopated rhythms
            // Multi-level pattern: level 0 does step-skip-step, level 1 leaps a third
            let rotated = rotate_pattern(&[1, 2, -1, 3, -1, 2, -2, 1], offset);
            let half = rotated.len() / 2;
            let pattern =
                IntervalPattern::new(vec![rotated[..half].to_vec(), rotated[half..].to_vec()], 1);
            let rhythm = vec![
                Duration::EIGHTH,
                Duration::QTR,
                Duration::EIGHTH,
                Duration::HALF,
                Duration::EIGHTH,
                Duration::EIGHTH,
            ];
            (pattern, rhythm)
        }
        _ => {
            // Level 5: wide leaps including fifths, complex rhythms
            let rotated = rotate_pattern(&[2, -1, 3, -2, 4, 1, -3, 2], offset);
            let third = rotated.len() / 3;
            let pattern = IntervalPattern::new(
                vec![
                    rotated[..third].to_vec(),
                    rotated[third..2 * third].to_vec(),
                ],
                rotated[2 * third],
            );
            let rhythm = vec![
                Duration::EIGHTH,
                Duration::EIGHTH,
                Duration::QTR,
                Duration::new(DurationKind::Qtr, 1), // dotted quarter
                Duration::EIGHTH,
                Duration::HALF,
            ];
            (pattern, rhythm)
        }
    }
}

/// Rotate a pattern by offset positions (for variety with different seeds).
fn rotate_pattern(base: &[i8], offset: usize) -> Vec<i8> {
    let len = base.len();
    let offset = offset % len;
    let mut result = Vec::with_capacity(len);
    result.extend_from_slice(&base[offset..]);
    result.extend_from_slice(&base[..offset]);
    result
}

/// Format a Duration as a human-readable string.
fn format_duration(dur: &Duration) -> &'static str {
    let ticks = dur.ticks();
    match ticks {
        128 => "whole",
        96 => "dotted-half",
        64 => "half",
        48 => "dotted-quarter",
        32 => "quarter",
        24 => "dotted-eighth",
        16 => "eighth",
        8 => "sixteenth",
        _ => "note",
    }
}

/// Format a Note for display (e.g. "C", "F#", "Bb").
fn format_note(note: &Note) -> String {
    format!("{note}")
}

/// Format a Pitch for display (e.g. "C4", "F#5").
fn format_pitch(pitch: &Pitch) -> String {
    format!("{}{}", format_note(&pitch.note), pitch.octave)
}

pub fn run(args: SightReadingArgs) -> Result<()> {
    let root = resolve_key(args.key.as_deref())?;
    let quality = resolve_scale(args.scale.as_deref())?;
    // Capture before `quality` is moved into the noteset builder.
    let minor_flavored = is_minor_flavored(&quality);
    let scale = build_scale_noteset(root, quality)?;

    if args.difficulty < 1 || args.difficulty > 5 {
        anyhow::bail!("difficulty must be 1–5 (got {})", args.difficulty);
    }

    // Validate every argument before generating anything, so a bad `--clef`
    // fails whether or not `-o` was passed. Deferring this to the output branch
    // let an unsupported clef print a normal text sheet and exit 0.
    let clef = ClefChoice::from_str_opt(args.clef.as_deref())?;

    // Notes per measure: assume 4/4 time, difficulty affects density
    let notes_per_measure = match args.difficulty {
        1 => 4, // all quarter notes
        2 => 6, // mix of quarter + eighth
        3 => 5, // varied
        4 => 6, // syncopated
        _ => 7, // complex
    };
    let total_notes = args.measures * notes_per_measure;

    let (pattern, rhythm) = difficulty_config(args.difficulty, args.seed);

    let starting_pitch = Pitch::new(root, 4);
    let bounds = PitchBounds::try_new(Pitch::new(Note::C, 3), Pitch::new(Note::C, 6))?;

    let config = MelodicSequencerConfig {
        chord_progression: ChordProgression::static_chord(scale),
        interval_pattern: pattern,
        rhythm_pattern: rhythm,
        bounds,
        starting_pitch,
        direction: Direction::Up,
        turnaround_mode: TurnaroundMode::Reflect,
        max_length: total_notes,
    };

    let mut sequencer = MelodicSequencer::new(config);
    let melody = sequencer
        .generate()
        .map_err(|e| anyhow::anyhow!("Melody generation failed: {e:?}"))?;

    // If an output file was requested, render staff notation via the engraver
    // and return — the text report is the no-output default.
    if let Some(ref path) = args.output {
        let key_sig = key_signature_for(root, minor_flavored);
        let n = render_melody_to_file(&melody, clef, key_sig, path)?;
        if args.verbose {
            eprintln!("wrote {path} ({n} bytes)");
        } else {
            println!("Wrote {path}");
        }
        return Ok(());
    }

    // Print header
    let scale_name = args.scale.as_deref().unwrap_or("major");
    let key_name = format_note(&root);
    println!("=== Sight-Reading Exercise ===");
    println!("Key: {} {}", key_name, scale_name);
    println!("Difficulty: {}/5", args.difficulty);
    println!("Measures: {} (4/4 time)", args.measures);
    println!("Notes: {}", melody.len());
    println!();

    // Print melody as a readable sequence
    print_melody_text(&melody, args.measures);

    if args.verbose {
        eprintln!(
            "Generated {} events in {} {} over {} measures",
            melody.len(),
            key_name,
            scale_name,
            args.measures
        );
        eprintln!(
            "Pitch range: {}–{}",
            format_pitch(
                &melody
                    .iter()
                    .min_by_key(|e| e.pitch.midi_note)
                    .unwrap()
                    .pitch
            ),
            format_pitch(
                &melody
                    .iter()
                    .max_by_key(|e| e.pitch.midi_note)
                    .unwrap()
                    .pitch
            ),
        );
    }

    Ok(())
}

/// Print the melody organized into measures.
fn print_melody_text(melody: &[MelodicEvent], measures: usize) {
    let notes_per_measure = melody.len() / measures.max(1);
    let remainder = melody.len() % measures.max(1);

    let mut idx = 0;
    for m in 0..measures {
        let count = if m < remainder {
            notes_per_measure + 1
        } else {
            notes_per_measure
        };
        let end = (idx + count).min(melody.len());
        let bar: Vec<String> = melody[idx..end]
            .iter()
            .map(|e| {
                let tied = if e.tied { "~" } else { "" };
                format!(
                    "{}{} ({})",
                    format_pitch(&e.pitch),
                    tied,
                    format_duration(&e.duration)
                )
            })
            .collect();
        println!("  m{}: {}", m + 1, bar.join("  "));
        idx = end;
    }
    // Any remaining notes
    if idx < melody.len() {
        let bar: Vec<String> = melody[idx..]
            .iter()
            .map(|e| {
                format!(
                    "{} ({})",
                    format_pitch(&e.pitch),
                    format_duration(&e.duration)
                )
            })
            .collect();
        println!("  extra: {}", bar.join("  "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_key_c() {
        let note = resolve_key(Some("C")).unwrap();
        assert_eq!(Pc::from(&note), Pc::from(0u8));
    }

    #[test]
    fn resolve_key_g() {
        let note = resolve_key(Some("G")).unwrap();
        assert_eq!(Pc::from(&note), Pc::from(7u8));
    }

    #[test]
    fn resolve_key_default() {
        let note = resolve_key(None).unwrap();
        assert_eq!(Pc::from(&note), Pc::from(0u8));
    }

    #[test]
    fn resolve_key_preserves_flat_spelling() {
        // Bb must stay Bb, not collapse to the sharpward A#.
        assert_eq!(resolve_key(Some("Bb")).unwrap(), Note::Bes);
        assert_eq!(resolve_key(Some("Eb")).unwrap(), Note::Ees);
        assert_eq!(resolve_key(Some("Ab")).unwrap(), Note::Aes);
        assert_eq!(resolve_key(Some("F#")).unwrap(), Note::Fis);
    }

    #[test]
    fn resolve_key_integer_uses_conventional_spelling() {
        // Pc 10 → Bb (flat key), Pc 6 → F#.
        assert_eq!(resolve_key(Some("10")).unwrap(), Note::Bes);
        assert_eq!(resolve_key(Some("3")).unwrap(), Note::Ees);
        assert_eq!(resolve_key(Some("6")).unwrap(), Note::Fis);
    }

    #[test]
    fn bb_major_scale_is_flat_spelled() {
        // The headline gaps-doc bug: Bb major must spell Bb C D Eb F G A,
        // never A# / D#.
        let scale = build_scale_noteset(Note::Bes, SevenNoteScaleQuality::Major).unwrap();
        let notes: Vec<Note> = scale.iter().copied().collect();
        assert!(notes.contains(&Note::Bes), "should contain Bb");
        assert!(notes.contains(&Note::Ees), "should contain Eb");
        assert!(!notes.contains(&Note::Ais), "must NOT contain A#");
        assert!(!notes.contains(&Note::Dis), "must NOT contain D#");
    }

    #[test]
    fn g_melodic_minor_has_correct_accidentals() {
        // G melodic minor: G A Bb C D E F# — flat 3, raised 7.
        let scale = build_scale_noteset(Note::G, SevenNoteScaleQuality::MelodicMinor).unwrap();
        let notes: Vec<Note> = scale.iter().copied().collect();
        assert!(notes.contains(&Note::Bes), "b3 should be Bb");
        assert!(notes.contains(&Note::Fis), "raised 7 should be F#");
        assert!(!notes.contains(&Note::Ais), "must NOT contain A#");
    }

    #[test]
    fn fsharp_major_spells_eis() {
        // F# major's 7th degree is E#, not F natural.
        let scale = build_scale_noteset(Note::Fis, SevenNoteScaleQuality::Major).unwrap();
        let notes: Vec<Note> = scale.iter().copied().collect();
        assert!(notes.contains(&Note::Eis), "leading tone should be E#");
    }

    #[test]
    fn resolve_scale_major() {
        let q = resolve_scale(Some("major")).unwrap();
        assert_eq!(q, SevenNoteScaleQuality::Major);
    }

    #[test]
    fn resolve_scale_melodic_minor() {
        let q = resolve_scale(Some("melodic-minor")).unwrap();
        assert_eq!(q, SevenNoteScaleQuality::MelodicMinor);
    }

    #[test]
    fn resolve_scale_unknown_rejects() {
        assert!(resolve_scale(Some("pentatonic")).is_err());
    }

    #[test]
    fn difficulty_config_returns_nonempty() {
        for d in 1..=5 {
            let (pattern, rhythm) = difficulty_config(d, Some(42));
            assert!(!rhythm.is_empty());
            // Pattern should yield intervals
            let mut p = pattern;
            let (interval, _) = p.next_interval();
            assert!(interval != 0 || d == 1); // level 1 can have 0 intervals via rotation
        }
    }

    #[test]
    fn generate_melody_produces_events() {
        let root = resolve_key(Some("C")).unwrap();
        let quality = resolve_scale(Some("major")).unwrap();
        let scale = build_scale_noteset(root, quality).unwrap();
        let (pattern, rhythm) = difficulty_config(2, Some(42));

        let config = MelodicSequencerConfig {
            chord_progression: ChordProgression::static_chord(scale),
            interval_pattern: pattern,
            rhythm_pattern: rhythm,
            bounds: PitchBounds::try_new(Pitch::new(Note::C, 3), Pitch::new(Note::C, 6)).unwrap(),
            starting_pitch: Pitch::new(Note::C, 4),
            direction: Direction::Up,
            turnaround_mode: TurnaroundMode::Reflect,
            max_length: 16,
        };

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();
        assert_eq!(melody.len(), 16);
        // First note should be the starting pitch
        assert_eq!(melody[0].pitch, Pitch::new(Note::C, 4));
        // All pitches within bounds
        for event in &melody {
            assert!(event.pitch.midi_note >= Pitch::new(Note::C, 3).midi_note);
            assert!(event.pitch.midi_note <= Pitch::new(Note::C, 6).midi_note);
        }
    }

    #[test]
    fn format_duration_known() {
        assert_eq!(format_duration(&Duration::QTR), "quarter");
        assert_eq!(format_duration(&Duration::EIGHTH), "eighth");
        assert_eq!(format_duration(&Duration::HALF), "half");
        assert_eq!(format_duration(&Duration::WHOLE), "whole");
    }

    #[test]
    fn rotate_pattern_works() {
        let base = [1, 2, 3, 4];
        assert_eq!(rotate_pattern(&base, 0), vec![1, 2, 3, 4]);
        assert_eq!(rotate_pattern(&base, 1), vec![2, 3, 4, 1]);
        assert_eq!(rotate_pattern(&base, 4), vec![1, 2, 3, 4]); // wraps
    }
}
