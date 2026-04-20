// REQ-O11

use music_midi::tempo::{StaticTempoMap, TempoSource};

#[test]
fn constant_one_change_point() {
    let t = StaticTempoMap::constant(120.0);
    assert_eq!(t.change_points().len(), 1);
    assert_eq!(t.bpm_at(0), 120.0);
    assert_eq!(t.bpm_at(1_000_000), 120.0);
}

#[test]
fn ticks_to_seconds_120bpm_quarter_is_half_second() {
    let t = StaticTempoMap::constant(120.0);
    let s = t.ticks_to_seconds(480);  // one quarter @ PPQ=480
    assert!((s - 0.5).abs() < 1e-9);
}

#[test]
fn push_changes_bpm() {
    let mut t = StaticTempoMap::constant(120.0);
    t.push(960, 60.0);
    assert_eq!(t.bpm_at(959), 120.0);
    assert_eq!(t.bpm_at(960), 60.0);
}
