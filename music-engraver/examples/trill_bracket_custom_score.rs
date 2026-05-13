//! Example: render trill ornaments with bracketed wavy-line extensions using
//! the custom-options ScoreBuilder method
//! `.trill_with_extension_bracketed_custom(side, direction, length_ss)`.
//! Produces `examples/output/trill_bracket_custom_score.svg`.
//!
//! Demonstrates the two extra knobs over the plain `.trill_with_extension_bracketed`:
//!
//! - `HookDirection::Up` for trills whose wiggle should still point a hook
//!   back toward the affected notes — useful for trills rendered below the
//!   staff or for layouts where the conventional Down hook would crash into
//!   surrounding marks.
//! - User-chosen hook length in staff spaces — Behind Bars shows hooks
//!   ranging from roughly 0.5 to 1.0 staff spaces depending on the density
//!   of the surrounding engraving.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: long-tail Both bracket — 1.2 ss tall hook makes the bracket
        // visually emphatic. The standard length is 0.75 ss; this is at the
        // upper end of Behind Bars' practical range.
        .note(p(Note::G, 4), Duration::new(DurationKind::Half, 1))
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Both,
            HookDirection::Down,
            1.2,
        )
        .note(p(Note::A, 4), Duration::QTR)
        .barline()
        // M2: short Start bracket — 0.5 ss is at the lower end of the
        // practical range; subtle but readable.
        .note(p(Note::E, 5), Duration::HALF)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Start,
            HookDirection::Down,
            0.5,
        )
        .note(p(Note::D, 5), Duration::HALF)
        .barline()
        // M3: Up-direction End bracket — the hook points upward away from the
        // baseline. Combined with the wiggle's "tr" glyph above the staff,
        // this gives the bracket an open visual gesture.
        .note(p(Note::C, 5), Duration::HALF)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::End,
            HookDirection::Up,
            0.9,
        )
        .note(p(Note::B, 4), Duration::HALF)
        .barline()
        // M4: chord trill with Both, Up direction, default length — exercises
        // the chord code path with the new direction parameter.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Both,
            HookDirection::Up,
            0.75,
        )
        .note(p(Note::F, 4), Duration::HALF)
        .end_barline()
        // M5 (system 3): cross-system Both bracket with Up direction and a
        // 1.0 ss hook. The Start hook sits on system 3, the End hook on the
        // incoming wiggle on system 4 — both must honor the same direction
        // and length so the bracket reads as a coherent pair across the line
        // break.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Both,
            HookDirection::Up,
            1.0,
        )
        .end_barline()
        // M6: the principal note that the cross-system trill resolves into.
        .note(p(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let path = out_dir.join("trill_bracket_custom_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count
    );

    // Structural assertions.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // The same chord/note structure with no brackets must produce strictly
    // fewer <line> elements — 8 fewer in this score, exactly matching the
    // breakdown:
    //   - M1 Both: 2 hooks
    //   - M2 Start: 1 hook
    //   - M3 End: 1 hook
    //   - M4 Both (chord): 2 hooks
    //   - M5 Both (cross-system): 2 hooks (Start on N + End on N+1)
    // Total: 8 extra <line> elements over the no-bracket variant.
    let no_bracket = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::new(DurationKind::Half, 1))
        .trill_with_extension()
        .note(p(Note::A, 4), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::HALF)
        .trill_with_extension()
        .note(p(Note::D, 5), Duration::HALF)
        .barline()
        .note(p(Note::C, 5), Duration::HALF)
        .trill_with_extension()
        .note(p(Note::B, 4), Duration::HALF)
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .trill_with_extension()
        .note(p(Note::F, 4), Duration::HALF)
        .end_barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let bracket_lines = svg.matches("<line ").count();
    let plain_lines = no_bracket.matches("<line ").count();
    let delta = bracket_lines - plain_lines;
    assert_eq!(
        delta, 8,
        "expected exactly 8 hook lines: M1 Both:2 + M2 Start:1 + M3 End:1 + \
         M4 Both:2 + M5 cross-system Both:2. Got delta={delta} \
         (bracketed={bracket_lines}, plain={plain_lines})"
    );

    // Compare with the default-hook variant (same sides, Down direction,
    // 0.75 ss length): the custom version must differ byte-for-byte because
    // M1's length, M2's length, M3's direction+length, M4's direction, and
    // M5's direction+length all differ from the defaults.
    let default_hooks = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::new(DurationKind::Half, 1))
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .note(p(Note::A, 4), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::HALF)
        .trill_with_extension_bracketed(TrillBracketSide::Start)
        .note(p(Note::D, 5), Duration::HALF)
        .barline()
        .note(p(Note::C, 5), Duration::HALF)
        .trill_with_extension_bracketed(TrillBracketSide::End)
        .note(p(Note::B, 4), Duration::HALF)
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .note(p(Note::F, 4), Duration::HALF)
        .end_barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_ne!(
        svg, default_hooks,
        "custom-options score must differ from the default-options score"
    );
    // But the line counts must match — the custom knobs only change geometry,
    // not element count.
    assert_eq!(
        svg.matches("<line ").count(),
        default_hooks.matches("<line ").count(),
        "custom knobs must not add/remove <line> elements"
    );

    // Sanity: at least the clef path and one notehead per measure are present.
    assert!(
        path_count > 6,
        "must render at least 7 paths (clef + noteheads + ornament glyphs)"
    );
}
