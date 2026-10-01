/// Generate MIDI ear-training quizzes (intervals, chords, or progressions).
///
/// Each quiz item is a pair of events (e.g. two notes for interval ID)
/// written sequentially into a MIDI file with brief silence between items.
/// The answer key is printed to stdout as labeled text (or JSON with `-o answers.json`).
use anyhow::{bail, Result};
use music::note::pitch::Pitch;
use rand::Rng;

const INTERVAL_NAMES: [&str; 13] = [
    "Unison (P1)",
    "Minor 2nd (m2)",
    "Major 2nd (M2)",
    "Minor 3rd (m3)",
    "Major 3rd (M3)",
    "Perfect 4th (P4)",
    "Tritone (TT)",
    "Perfect 5th (P5)",
    "Minor 6th (m6)",
    "Major 6th (M6)",
    "Minor 7th (m7)",
    "Major 7th (M7)",
    "Octave (P8)",
];

pub struct EarTrainingArgs {
    pub quiz_type: QuizType,
    pub count: usize,
    pub output: Option<String>,
    pub verbose: bool,
    pub seed: Option<u64>,
}

#[derive(Clone, Copy)]
pub enum QuizType {
    Intervals,
}

impl QuizType {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None | Some("intervals") => Ok(Self::Intervals),
            Some(other) => bail!("Unknown quiz type '{}'. Options: intervals", other),
        }
    }
}

pub fn run(args: EarTrainingArgs) -> Result<()> {
    match args.quiz_type {
        QuizType::Intervals => run_intervals(args),
    }
}

/// A note event with absolute tick position.
struct NoteEvent {
    tick: u64,
    key: u8,
    velocity: u8,
    is_on: bool,
}

fn run_intervals(args: EarTrainingArgs) -> Result<()> {
    let mut rng = {
        use rand::SeedableRng;
        match args.seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        }
    };

    let ppq: u16 = 480;
    let bpm: f64 = 90.0;

    // Each quiz item: root note + interval note, then rest.
    // Root chosen from C3–C5 (MIDI 48–72), interval 0–12 semitones.
    let mut events: Vec<NoteEvent> = Vec::new();
    let mut answers: Vec<(usize, u8, String, String, String)> = Vec::new();
    let mut tick: u64 = 0;
    let quarter = ppq as u64;
    let mut item_num = 0usize;

    for _ in 0..args.count {
        let root_midi: u8 = rng.gen_range(48..=72);
        let interval: u8 = rng.gen_range(0..=12);
        let target_midi = root_midi + interval;

        if target_midi >= 108 {
            continue;
        }

        item_num += 1;
        let root = Pitch::from_midi(root_midi)?;
        let target = Pitch::from_midi(target_midi)?;

        // Play root for 1 quarter note
        events.push(NoteEvent {
            tick,
            key: root_midi,
            velocity: 80,
            is_on: true,
        });
        tick += quarter;
        events.push(NoteEvent {
            tick,
            key: root_midi,
            velocity: 64,
            is_on: false,
        });

        // Brief pause (half quarter)
        tick += quarter / 2;

        // Play target for 1 quarter note
        events.push(NoteEvent {
            tick,
            key: target_midi,
            velocity: 80,
            is_on: true,
        });
        tick += quarter;
        events.push(NoteEvent {
            tick,
            key: target_midi,
            velocity: 64,
            is_on: false,
        });

        // 2 beats rest between items
        tick += quarter * 2;

        let name = INTERVAL_NAMES[interval as usize];
        answers.push((
            item_num,
            interval,
            name.to_string(),
            format!("{}", root),
            format!("{}", target),
        ));
    }

    let smf = build_smf(ppq, bpm, &events)?;

    // Write MIDI output
    let output_path = args.output.as_deref().unwrap_or("ear_training.mid");
    if output_path.ends_with(".json") {
        let json = build_json_answers(&answers);
        std::fs::write(output_path, json)?;
        if args.verbose {
            eprintln!("Wrote JSON answer key to {}", output_path);
        }
    } else {
        let mut bytes = Vec::new();
        smf.write(&mut bytes)
            .map_err(|e| anyhow::anyhow!("MIDI write error: {}", e))?;
        std::fs::write(output_path, &bytes)?;

        if args.verbose {
            eprintln!(
                "Wrote {} quiz items to {} ({} bytes)",
                answers.len(),
                output_path,
                bytes.len()
            );
        }

        // Print answer key to stdout
        println!("Ear Training — Interval Identification");
        println!("=======================================");
        println!("{} items, tempo = {} BPM\n", answers.len(), bpm);
        for (num, semitones, name, root, target) in &answers {
            println!(
                "  #{:>2}: {} → {}  =  {} semitones — {}",
                num, root, target, semitones, name
            );
        }
    }

    Ok(())
}

fn build_json_answers(answers: &[(usize, u8, String, String, String)]) -> String {
    let mut out = String::from("[\n");
    for (i, (num, semitones, name, root, target)) in answers.iter().enumerate() {
        out.push_str(&format!(
            "  {{\"item\": {}, \"root\": \"{}\", \"target\": \"{}\", \"semitones\": {}, \"interval\": \"{}\"}}",
            num, root, target, semitones, name
        ));
        if i + 1 < answers.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push(']');
    out
}

/// Build an SMF directly from pre-computed NoteEvents using midly.
fn build_smf(ppq: u16, bpm: f64, events: &[NoteEvent]) -> Result<midly::Smf<'static>> {
    use midly::{
        num::{u15, u24, u28, u4, u7},
        Format, Header, MetaMessage, MidiMessage as MM, Smf, Timing, TrackEvent, TrackEventKind,
    };

    let header = Header {
        format: Format::Parallel,
        timing: Timing::Metrical(u15::new(ppq)),
    };

    // Conductor track: single tempo event
    let usec_per_qn = (60_000_000.0 / bpm).round() as u32;
    let conductor = vec![
        TrackEvent {
            delta: u28::from(0u32),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(usec_per_qn))),
        },
        TrackEvent {
            delta: u28::from(0u32),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        },
    ];

    // Instrument track: convert absolute-tick events to delta-tick
    let mut track_events: Vec<TrackEvent<'static>> = Vec::new();
    track_events.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"ear-training")),
    });

    let mut last_tick: u64 = 0;
    for ev in events {
        let delta = (ev.tick - last_tick) as u32;
        last_tick = ev.tick;

        let kind = if ev.is_on {
            TrackEventKind::Midi {
                channel: u4::new(0),
                message: MM::NoteOn {
                    key: u7::new(ev.key),
                    vel: u7::new(ev.velocity),
                },
            }
        } else {
            TrackEventKind::Midi {
                channel: u4::new(0),
                message: MM::NoteOff {
                    key: u7::new(ev.key),
                    vel: u7::new(ev.velocity),
                },
            }
        };

        track_events.push(TrackEvent {
            delta: u28::from(delta),
            kind,
        });
    }
    track_events.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(Smf {
        header,
        tracks: vec![conductor, track_events],
    })
}
