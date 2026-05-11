// REQ-O5, O6, O7, O8, O15: SmfBuilder integration tests
// Fix M1: use smf.write(&mut bytes) (midly::io::Write on Vec<u8>)
// instead of write_std (std::io::Write).

#![cfg(feature = "smf")]

use music::note::pitch::Pitch;
use music_midi::{
    MidiConversionError,
    smf::SmfBuilder,
    tempo::StaticTempoMap,
};

/// REQ-O15: build without a tempo source returns TempoSourceEmpty.
#[test]
fn requires_tempo() {
    let p = Pitch::from_midi(60).unwrap();
    let result = SmfBuilder::new()
        .add_track("m", 0, &p)
        .unwrap()
        .build();
    assert!(
        matches!(result, Err(MidiConversionError::TempoSourceEmpty)),
        "expected TempoSourceEmpty, got: {:?}",
        result
    );
}

/// REQ-O8: round-trip single note through midly.
/// Builds SMF, writes to bytes, parses back — confirms 2 tracks (conductor + melody).
#[test]
fn round_trip_single_note_through_midly() {
    let p = Pitch::from_midi(60).unwrap();
    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("m", 0, &p)
        .unwrap()
        .build()
        .unwrap();
    let mut bytes = Vec::new();
    // Fix M1: Vec<u8> implements midly::io::Write; use .write() not .write_std()
    smf.write(&mut bytes).unwrap();
    let parsed = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(parsed.tracks.len(), 2, "expected conductor + melody tracks");
}

/// REQ-X7: ppq not a multiple of 32 is rejected at build time.
#[test]
fn rejects_ppq_not_multiple_of_32() {
    let p = Pitch::from_midi(60).unwrap();
    let result = SmfBuilder::new()
        .ppq(481)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("m", 0, &p)
        .unwrap()
        .build();
    assert!(
        matches!(result, Err(MidiConversionError::InvalidPpq(481))),
        "expected InvalidPpq(481), got: {:?}",
        result
    );
}

/// REQ-O18: adding a 17th track returns TooManyVoices(17).
#[test]
fn more_than_16_tracks_errors() {
    let p = Pitch::from_midi(60).unwrap();
    let mut builder = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0));
    for i in 0u8..16 {
        builder = builder.add_track(&format!("t{}", i), i, &p).unwrap();
    }
    let err = builder.add_track("over", 0, &p);
    assert!(
        matches!(err, Err(MidiConversionError::TooManyVoices(17))),
        "expected TooManyVoices(17)",
    );
}

/// REQ-O5, O16: SmfBuilder::meter() splices a TimeSignature meta into the
/// conductor track that survives the midly round-trip.
#[test]
fn time_signature_meta_reaches_conductor() {
    use music::notation::rhythm::meter::{Meter, MeterDenominator};
    use midly::{MetaMessage, TrackEvent, TrackEventKind};
    // 7/8 time signature via builder
    let p = Pitch::from_midi(60).unwrap();
    let meter = Meter::new(7, MeterDenominator::Eight, None);
    let owned = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .meter(meter)
        .add_track("m", 0, &p)
        .unwrap()
        .build()
        .unwrap();
    let mut bytes = Vec::new();
    owned.write(&mut bytes).unwrap();
    let parsed = midly::Smf::parse(&bytes).unwrap();
    let ts = parsed.tracks[0].iter().find(|e| matches!(
        e.kind, TrackEventKind::Meta(MetaMessage::TimeSignature(..))
    ));
    match ts {
        Some(TrackEvent { kind: TrackEventKind::Meta(MetaMessage::TimeSignature(n, d, c, s)), .. }) => {
            assert_eq!(*n, 7);
            assert_eq!(*d, 3);  // log2(8) = 3
            assert_eq!(*c, 24);
            assert_eq!(*s, 8);
        }
        _ => panic!("expected TimeSignature meta in conductor track, got {:?}", ts),
    }
}

/// REQ-O16: builder.meter() auto-wires TimeSignature meta in conductor.
#[test]
fn builder_meter_emits_time_signature() {
    use music::notation::rhythm::meter::{Meter, MeterDenominator};

    let c4 = Pitch::from_midi(60).unwrap();
    let owned = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .meter(Meter::new(4, MeterDenominator::Four, None))
        .add_track("piano", 0, &c4).unwrap()
        .build().unwrap();

    let smf = owned.as_smf();
    let conductor = &smf.tracks[0];
    let found = conductor.iter().any(|ev| matches!(
        &ev.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::TimeSignature(4, 2, 24, 8))
    ));
    assert!(found, "conductor must contain TimeSignature(4, 2, 24, 8) when meter() is called");
}

/// REQ-O15 negative: no TimeSignature meta when meter() is not called.
#[test]
fn builder_no_meter_omits_time_signature() {
    let c4 = Pitch::from_midi(60).unwrap();
    let owned = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("piano", 0, &c4).unwrap()
        .build().unwrap();
    let smf = owned.as_smf();
    let has_ts = smf.tracks[0].iter().any(|ev| matches!(
        &ev.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::TimeSignature(..))
    ));
    assert!(!has_ts, "no TimeSignature should appear when meter() is not called");
}

/// W8: tempo change points whose tick delta exceeds the SMF u28 maximum
/// (0x0FFF_FFFF) must surface a MidiConversionError::Smf rather than silently
/// truncating the delta or panicking inside midly.
#[test]
fn oversize_tempo_tick_delta_errors_not_panics() {
    let p = Pitch::from_midi(60).unwrap();
    let mut tempo = StaticTempoMap::constant(120.0);
    // 0x1000_0000 = 268_435_456, one past u28::max_value() = 0x0FFF_FFFF.
    tempo.push(0x1000_0000, 180.0);
    let result = SmfBuilder::new()
        .ppq(480)
        .tempo(tempo)
        .add_track("m", 0, &p)
        .unwrap()
        .build();
    match result {
        Err(MidiConversionError::Smf(msg)) => {
            assert!(
                msg.contains("u28") || msg.contains("268435455"),
                "expected Smf error to mention u28 overflow, got: {}",
                msg
            );
        }
        other => panic!(
            "expected MidiConversionError::Smf for oversize tick delta, got: {:?}",
            other
        ),
    }
}

/// SMF with 2 tempo change points produces a conductor track with 2 tempo metas.
#[test]
fn multi_tempo_conductor_track() {
    let p = Pitch::from_midi(60).unwrap();
    let mut tempo = StaticTempoMap::constant(120.0);
    tempo.push(1920, 180.0); // tempo change at tick 1920
    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(tempo)
        .add_track("m", 0, &p)
        .unwrap()
        .build()
        .unwrap();
    let mut bytes = Vec::new();
    smf.write(&mut bytes).unwrap();
    let parsed = midly::Smf::parse(&bytes).unwrap();
    // conductor track: 2 tempo metas + EndOfTrack
    let conductor = &parsed.tracks[0];
    let tempo_events: Vec<_> = conductor
        .iter()
        .filter(|e| matches!(e.kind, midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))))
        .collect();
    assert_eq!(tempo_events.len(), 2, "expected 2 tempo meta events");
}
