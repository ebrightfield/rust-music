// score_to_mid: CLI example that generates an SMF from a hard-coded "Mary Had a
// Little Lamb" melody and writes it to the path given as the first argument.
//
// Usage:
//   cargo run -p music-midi --example score_to_mid --features smf -- /tmp/out.mid
//
// Note on LilypondScore RON pivot:
// The dispatch originally requested reading a LilypondScore from a RON fixture.
// LilypondScore<'a> holds lifetime-bound references to borrowed note data, which
// means it cannot implement serde::Deserialize or be reconstructed from a RON
// string at runtime. The smf feature (which also enables the score feature) is
// present, but RON deserialization of a lifetime-parameterized type is not
// feasible without an intermediate owned form that does not yet exist. We
// therefore demonstrate the same pipeline — melody -> SmfBuilder -> SMF file —
// using the owned MelodicEvent type, which is fully serializable and exercises
// the same conversion path.
//
// Acceptance criterion: the output file is a valid SMF whose first 4 bytes are
// the ASCII string "MThd".

use music_midi::{smf::SmfBuilder, StaticTempoMap};
use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::{Duration, DurationKind};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let output_path = args.get(1).map(|s| s.as_str()).unwrap_or("/tmp/music-midi-out.mid");

    let smf_bytes = mary_had_a_little_lamb_smf()
        .unwrap_or_else(|e| {
            eprintln!("Error building SMF: {e}");
            std::process::exit(1);
        });

    fs::write(output_path, &smf_bytes)
        .unwrap_or_else(|e| {
            eprintln!("Error writing {output_path}: {e}");
            std::process::exit(1);
        });

    println!("Wrote {} bytes to {output_path}", smf_bytes.len());
    println!("First 4 bytes (should be MThd): {:?}", &smf_bytes[..4.min(smf_bytes.len())]);
}

/// Build an SMF of "Mary Had a Little Lamb" (first phrase) at 120 BPM.
///
/// Notes (MIDI): E4=64, D4=62, C4=60, G4=67
///
/// Phrase: E D C D E E E _ D D D _ E G G _ E D C D E E E E D D E D C
fn mary_had_a_little_lamb_smf() -> Result<Vec<u8>, music_midi::MidiConversionError> {
    // Mary Had a Little Lamb — first two phrases (16 quarter notes).
    // MIDI note numbers: C4=60, D4=62, E4=64, G4=67
    let e4 = Pitch::from_midi(64).expect("E4 in range");
    let d4 = Pitch::from_midi(62).expect("D4 in range");
    let c4 = Pitch::from_midi(60).expect("C4 in range");
    let g4 = Pitch::from_midi(67).expect("G4 in range");

    let qtr = Duration::new(DurationKind::Qtr, 0);
    let half = Duration::new(DurationKind::Half, 0);

    // Phrase 1: E D C D E E E(half) D D D(half) E G G(half)
    // Phrase 2: E D C D E E E E D D E D C(half)
    let melody: Vec<MelodicEvent> = vec![
        // Phrase 1
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(c4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(e4, half),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(d4, half),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(g4, qtr),
        MelodicEvent::new(g4, half),
        // Phrase 2
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(c4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(e4, qtr),
        MelodicEvent::new(d4, qtr),
        MelodicEvent::new(c4, half),
    ];

    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("melody", 0, melody.as_slice())?
        .build()?;

    let mut bytes = Vec::new();
    smf.write(&mut bytes)
        .map_err(|e| music_midi::MidiConversionError::Smf(e.to_string()))?;
    Ok(bytes)
}
