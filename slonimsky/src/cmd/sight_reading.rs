use anyhow::{Context, Result};
use music::melody::context::ChordProgression;
use music::melody::pattern::IntervalPattern;
use music::melody::sequencer::{MelodicEvent, MelodicSequencer, MelodicSequencerConfig};
use music::melody::{Direction, PitchBounds, TurnaroundMode};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::{AsPcSlice, PcShape};
use music::note_collections::NoteSet;
use music::note_collections::OctavePartition;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::{Duration, DurationKind};
use musical_combinatorics::seven_note_scales::SevenNoteScaleQuality;

use super::input::parse_pc;

pub struct SightReadingArgs {
    pub key: Option<String>,
    pub scale: Option<String>,
    pub difficulty: u8,
    pub measures: usize,
    pub seed: Option<u64>,
    pub verbose: bool,
}

/// Resolve key name to a root Note.
fn resolve_key(key_str: Option<&str>) -> Result<Note> {
    let s = key_str.unwrap_or("C");
    let pc = parse_pc(s).with_context(|| format!("Invalid key: '{s}'"))?;
    // Pick the first (sharpward) spelling for the root
    let notes = pc.notes();
    Ok(notes[0])
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

/// Transpose a slice of pitch classes by semitones.
fn transpose_pcs(pcs: &[Pc], semitones: u8) -> Vec<Pc> {
    pcs.iter()
        .map(|&pc| Pc::from((u8::from(pc) + semitones) % 12))
        .collect()
}

/// Build a NoteSet for the given key and scale quality.
fn build_scale_noteset(root: Note, quality: SevenNoteScaleQuality) -> NoteSet {
    let partition = OctavePartition::from(&quality);
    let parent_pcs = PcShape::from(&partition);
    let root_pc = Pc::from(&root);
    let transposed = transpose_pcs(parent_pcs.as_pc_slice(), u8::from(root_pc));
    // Map PCs to Notes via first available spelling
    let notes: Vec<Note> = transposed.iter().map(|pc| pc.notes()[0]).collect();
    NoteSet::new(notes)
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
            let pattern = IntervalPattern::new(
                vec![rotated[..half].to_vec(), rotated[half..].to_vec()],
                1,
            );
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
                vec![rotated[..third].to_vec(), rotated[third..2*third].to_vec()],
                rotated[2*third],
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
    let scale = build_scale_noteset(root, quality);

    if args.difficulty < 1 || args.difficulty > 5 {
        anyhow::bail!("difficulty must be 1–5 (got {})", args.difficulty);
    }

    // Notes per measure: assume 4/4 time, difficulty affects density
    let notes_per_measure = match args.difficulty {
        1 => 4,       // all quarter notes
        2 => 6,       // mix of quarter + eighth
        3 => 5,       // varied
        4 => 6,       // syncopated
        _ => 7,       // complex
    };
    let total_notes = args.measures * notes_per_measure;

    let (pattern, rhythm) = difficulty_config(args.difficulty, args.seed);

    let starting_pitch = Pitch::new(root, 4);
    let bounds = PitchBounds::try_new(
        Pitch::new(Note::C, 3),
        Pitch::new(Note::C, 6),
    )?;

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
    let melody = sequencer.generate()
        .map_err(|e| anyhow::anyhow!("Melody generation failed: {e:?}"))?;

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
        eprintln!("Generated {} events in {} {} over {} measures",
            melody.len(), key_name, scale_name, args.measures);
        eprintln!("Pitch range: {}–{}",
            format_pitch(&melody.iter().min_by_key(|e| e.pitch.midi_note).unwrap().pitch),
            format_pitch(&melody.iter().max_by_key(|e| e.pitch.midi_note).unwrap().pitch),
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
                format!("{}{} ({})", format_pitch(&e.pitch), tied, format_duration(&e.duration))
            })
            .collect();
        println!("  m{}: {}", m + 1, bar.join("  "));
        idx = end;
    }
    // Any remaining notes
    if idx < melody.len() {
        let bar: Vec<String> = melody[idx..]
            .iter()
            .map(|e| format!("{} ({})", format_pitch(&e.pitch), format_duration(&e.duration)))
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
        let scale = build_scale_noteset(root, quality);
        let (pattern, rhythm) = difficulty_config(2, Some(42));

        let config = MelodicSequencerConfig {
            chord_progression: ChordProgression::static_chord(scale),
            interval_pattern: pattern,
            rhythm_pattern: rhythm,
            bounds: PitchBounds::try_new(
                Pitch::new(Note::C, 3),
                Pitch::new(Note::C, 6),
            ).unwrap(),
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
