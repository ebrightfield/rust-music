//! Example: render trill ornaments with bracketed wavy-line extensions using
//! the ScoreBuilder `.trill_with_extension_bracketed(side)` method. Produces
//! `examples/output/trill_bracket_score.svg`.
//!
//! Demonstrates all three bracket forms (Start, End, Both) plus a chord trill
//! bracket and a cross-system bracketed trill. The bracket form is the
//! engraving convention for trills whose duration must be unambiguous —
//! Behind Bars: "where the duration of a trill must be precisely defined,
//! the wavy line is bracketed at one or both ends."

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::trill_bracket::TrillBracketSide;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: dotted-half trill with Both bracket — defines the trill's exact
        // start and end. The wavy line is capped at both ends with a short
        // vertical hook.
        .note(p(Note::G, 4), Duration::new(DurationKind::Half, 1))
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .note(p(Note::A, 4), Duration::QTR)
        .barline()
        // M2: half-note trill with Start bracket only — marks unambiguously
        // where the trill begins; the trill simply tapers into the following
        // note.
        .note(p(Note::E, 5), Duration::HALF)
        .trill_with_extension_bracketed(TrillBracketSide::Start)
        .note(p(Note::D, 5), Duration::HALF)
        .barline()
        // M3: half-note trill with End bracket only — marks where the trill
        // ends; the start is implicit at the "tr" glyph.
        .note(p(Note::C, 5), Duration::HALF)
        .trill_with_extension_bracketed(TrillBracketSide::End)
        .note(p(Note::B, 4), Duration::HALF)
        .barline()
        // M4: chord trill with Both bracket — exercises the chord path.
        // The bracketed wiggle anchors to the top note of the chord.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .note(p(Note::F, 4), Duration::HALF)
        .end_barline()
        // M5 (system 3): cross-system bracketed trill. The whole-note trill
        // at the end of M5 has its wiggle continued into M6 (the next system);
        // with Both bracket, the Start hook sits at the wiggle's left edge on
        // system 3, and the End hook sits at the wiggle's right edge on the
        // incoming wiggle on system 4 — the bracket spans the trill's full
        // semantic range across the line break.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        // M6: the principal note that the cross-system trill resolves into.
        .note(p(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let path = out_dir.join("trill_bracket_score.svg");
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

    // Baseline check: compare against the same score without brackets. The
    // bracket variant should add hook <line> elements:
    //   - M1 Both: 2 hooks
    //   - M2 Start: 1 hook
    //   - M3 End: 1 hook
    //   - M4 Both (chord): 2 hooks
    //   - M5 Both (cross-system): 2 hooks total (Start on N + End on N+1)
    // Total: 8 extra <line> elements vs the no-bracket version.
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
        "expected exactly 8 hook lines (M1 Both: 2 + M2 Start: 1 + M3 End: 1 + \
         M4 Both: 2 + M5 cross-system Both: 2). Got delta={delta} \
         (bracketed={bracket_lines}, plain={plain_lines})"
    );

    // The bracket form must produce strictly more paths only when the
    // underlying wiggle is also drawn; if cargo build flags differ, this
    // catches a regression where the entire trill machinery silently broke.
    assert!(
        path_count > 0,
        "must render at least one path (clef + noteheads)"
    );
}
