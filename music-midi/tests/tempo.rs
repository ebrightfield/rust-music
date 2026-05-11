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

// I18: invalid BPM inputs to public constructors must clamp, not panic.

#[test]
fn constant_with_zero_bpm_coerces_to_120() {
    let t = StaticTempoMap::constant(0.0);
    assert_eq!(t.bpm_at(0), 120.0);
    assert_eq!(t.change_points().len(), 1);
}

#[test]
fn constant_with_nan_bpm_does_not_panic() {
    let t = StaticTempoMap::constant(f32::NAN);
    // Coerced to 120.0 — must be a finite, positive value.
    let bpm = t.bpm_at(0);
    assert!(bpm.is_finite() && bpm > 0.0);
    assert_eq!(bpm, 120.0);
}

#[test]
fn constant_with_negative_bpm_coerces_to_120() {
    let t = StaticTempoMap::constant(-42.0);
    assert_eq!(t.bpm_at(0), 120.0);
}

#[test]
fn constant_with_infinity_bpm_coerces_to_120() {
    let t = StaticTempoMap::constant(f32::INFINITY);
    assert_eq!(t.bpm_at(0), 120.0);
}

#[test]
fn push_with_invalid_bpm_clamps_instead_of_panic() {
    let mut t = StaticTempoMap::constant(120.0);
    t.push(480, f32::NAN);
    assert_eq!(t.bpm_at(480), 120.0);
    t.push(960, -10.0);
    assert_eq!(t.bpm_at(960), 120.0);
}
