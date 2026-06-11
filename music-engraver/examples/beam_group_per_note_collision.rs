//! Example: per-note collision detection inside additional-voice beam groups.
//!
//! When an additional voice contains a beamed run of notes and only *some*
//! of those notes share a beat (and a colliding staff position) with the
//! primary voice, only the colliding noteheads shift — the rest of the
//! beam group, its stems, and the beam line itself stay anchored at the
//! original beat positions.
//!
//! Layout:
//! - Measure 1: Voice 0 plays two quarters (C5, then E5). Voice 1 plays a
//!   beam of four eighths starting on C5 — note 0 collides at beat 1
//!   (unison with the primary's C5); note 2 collides at beat 2 (unison
//!   with primary's E5 since the beam-group's note 2 is E5); notes 1 and
//!   3 sit on staff positions that are a third away from the primary at
//!   their respective beats so no collision fires.
//! - Measure 2: Voice 0 plays one quarter (G5). Voice 1 plays a beam of
//!   four eighths — none of which collide with the primary's G5. The
//!   beam group renders normally; this measure is the no-collision
//!   control.
//!
//! Produces `examples/output/beam_group_per_note_collision.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: per-note collision targets in a beam group.
        // Voice 0 (primary): C5 half, E5 half. Stems up by default.
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .note(Pitch::new(Note::E, 5), Duration::HALF)
        .voice(1)
        // Voice 1: beam of four eighth notes. Beat positions:
        //  beat 0: C5 → unison with primary's C5 → collision (note 0)
        //  beat 0.5: A4 → primary has no element at this x → no collision
        //  beat 1: E5 → unison with primary's E5 → collision (note 2)
        //  beat 1.5: G4 → primary has no element at this x → no collision
        .beam_group(vec![
            (Pitch::new(Note::C, 5), Duration::EIGHTH),
            (Pitch::new(Note::A, 4), Duration::EIGHTH),
            (Pitch::new(Note::E, 5), Duration::EIGHTH),
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
        ])
        .voice(0)
        .barline()
        // Measure 2: no-collision control — beam group sits below the
        // primary's note so nothing collides.
        .note(Pitch::new(Note::G, 5), Duration::WHOLE)
        .voice(1)
        .beam_group(vec![
            (Pitch::new(Note::D, 4), Duration::EIGHTH),
            (Pitch::new(Note::E, 4), Duration::EIGHTH),
            (Pitch::new(Note::F, 4), Duration::EIGHTH),
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
        ])
        .voice(0)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/beam_group_per_note_collision.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/beam_group_per_note_collision.svg ({} bytes)",
        svg.len()
    );

    // Structural assertions.
    assert!(svg.starts_with("<svg"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let polygon_count = svg.matches("<polygon").count();
    println!("  paths: {path_count}, lines: {line_count}, polygons: {polygon_count}");

    // Expect at minimum: 2 primary noteheads (M1) + 1 primary whole (M2) +
    // 4 voice-1 eighths (M1) + 4 voice-1 eighths (M2) + 1 clef = 12 paths.
    // Allow extra for time-signature glyphs and tweaks; lower bound only.
    assert!(path_count >= 12, "expected ≥12 paths, got {path_count}");
    // Lines: 5 staff × N systems + stems + barlines.
    assert!(line_count >= 10, "expected ≥10 lines, got {line_count}");
    // Polygons: at least one beam in M1 + one in M2.
    assert!(polygon_count >= 2, "expected ≥2 beam polygons, got {polygon_count}");
}
