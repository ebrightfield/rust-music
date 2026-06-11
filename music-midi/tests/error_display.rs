// REQ-O19

use music_midi::MidiConversionError;

#[test]
fn display_follows_standard_format() {
    let e = MidiConversionError::InvalidPpq(481);
    let s = e.to_string();
    assert!(s.starts_with("music-midi: "));
    assert!(s.contains("481"));
    assert!(s.contains("multiple of 32"));
}

#[test]
fn display_never_contains_prohibited_phrases() {
    for e in [
        MidiConversionError::PitchOutOfRange(200),
        MidiConversionError::InvalidPpq(7),
        MidiConversionError::TempoSourceEmpty,
        MidiConversionError::TooManyVoices(20),
    ] {
        let s = e.to_string();
        for bad in ["Error occurred", "Operation failed", "Something went wrong"] {
            assert!(!s.contains(bad), "variant `{:?}` violates error message standard: {}", e, s);
        }
    }
}
