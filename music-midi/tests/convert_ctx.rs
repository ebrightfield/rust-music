// REQ-O9, X7
// Note: rescale_quarter_note_at_480 cannot live here (rescale_ticks is pub(crate)).
// That test is placed in a #[cfg(test)] module inside convert/mod.rs.

use music_midi::*;
use music_midi::tempo::StaticTempoMap;

fn instr(_s: &str) -> u8 { 0 }

#[test]
fn rejects_ppq_not_multiple_of_32() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(481, &t, VelocityPolicy::Fixed(80), None, &instr);
    assert!(matches!(ctx, Err(MidiConversionError::InvalidPpq(481))));
}

#[test]
fn accepts_default_ppq_480() {
    let t = StaticTempoMap::constant(120.0);
    assert!(ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).is_ok());
}
