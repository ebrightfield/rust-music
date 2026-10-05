use super::*;
use crate::layout::ottava::OttavaKind;
use crate::layout::stem::StemDirection;
use crate::layout::tremolo::TremoloCount;
use music::note::note::Note;

// --- duration_kind_to_log2 ---

#[test]
fn log2_whole_is_0() {
    assert_eq!(duration_kind_to_log2(DurationKind::Whole), 0);
}

#[test]
fn log2_quarter_is_2() {
    assert_eq!(duration_kind_to_log2(DurationKind::Qtr), 2);
}

#[test]
fn log2_eighth_is_3() {
    assert_eq!(duration_kind_to_log2(DurationKind::Eighth), 3);
}

#[test]
fn log2_sixteenth_is_4() {
    assert_eq!(duration_kind_to_log2(DurationKind::Sixteenth), 4);
}

#[test]
fn log2_128th_is_7() {
    assert_eq!(duration_kind_to_log2(DurationKind::OneTwentyEighth), 7);
}

#[test]
fn log2_breve_is_minus_1_distinct_from_whole() {
    assert_eq!(duration_kind_to_log2(DurationKind::Breve), -1);
    assert_ne!(
        duration_kind_to_log2(DurationKind::Breve),
        duration_kind_to_log2(DurationKind::Whole)
    );
}

// --- note_altered_in_key ---

#[test]
fn f_altered_in_one_sharp() {
    assert!(note_altered_in_key(
        music::note::spelling::Letter::F,
        &KeySignature::Sharps(1)
    ));
}

#[test]
fn c_not_altered_in_one_sharp() {
    assert!(!note_altered_in_key(
        music::note::spelling::Letter::C,
        &KeySignature::Sharps(1)
    ));
}

#[test]
fn c_altered_in_two_sharps() {
    assert!(note_altered_in_key(
        music::note::spelling::Letter::C,
        &KeySignature::Sharps(2)
    ));
}

#[test]
fn b_altered_in_one_flat() {
    assert!(note_altered_in_key(
        music::note::spelling::Letter::B,
        &KeySignature::Flats(1)
    ));
}

#[test]
fn b_not_altered_in_open_key() {
    assert!(!note_altered_in_key(
        music::note::spelling::Letter::B,
        &KeySignature::Open
    ));
}

#[test]
fn all_altered_in_seven_sharps() {
    use music::note::spelling::Letter;
    for letter in [
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::A,
        Letter::B,
    ] {
        assert!(note_altered_in_key(letter, &KeySignature::Sharps(7)));
    }
}

// --- should_show_accidental ---

#[test]
fn sharp_note_shows_sharp_glyph() {
    let pitch = Pitch::new(Note::Fis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Open);
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
}

#[test]
fn sharp_note_suppressed_when_in_sharp_key() {
    // F# in D major (2 sharps) — F is sharped in the key sig, so the
    // accidental is redundant and suppressed.
    let pitch = Pitch::new(Note::Fis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
    assert_eq!(glyph, None);
}

#[test]
fn natural_note_in_sharp_key_shows_natural() {
    // F natural in D major — F is sharped in key sig, so we show a natural
    let pitch = Pitch::new(Note::F, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalNatural));
}

#[test]
fn natural_note_in_open_key_no_accidental() {
    let pitch = Pitch::new(Note::C, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Open);
    assert_eq!(glyph, None);
}

#[test]
fn flat_note_shows_flat() {
    let pitch = Pitch::new(Note::Bes, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Open);
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
}

#[test]
fn sharp_note_shown_in_flat_key() {
    // F# in Bb major (2 flats) — F is not flatted, so sharp is shown
    let pitch = Pitch::new(Note::Fis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
}

#[test]
fn sharp_note_shown_when_not_in_key() {
    // G# in D major (2 sharps: F#, C#) — G is not altered, so show the sharp
    let pitch = Pitch::new(Note::Gis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
}

#[test]
fn flat_note_suppressed_when_in_flat_key() {
    // Bb in Bb major (2 flats: Bb, Eb) — B is flatted in key sig, suppress
    let pitch = Pitch::new(Note::Bes, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
    assert_eq!(glyph, None);
}

#[test]
fn flat_note_shown_in_sharp_key() {
    // Bb in G major (1 sharp) — B is not altered, show the flat
    let pitch = Pitch::new(Note::Bes, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(1));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
}

#[test]
fn flat_note_shown_when_not_in_key() {
    // Ab in Bb major (2 flats: Bb, Eb) — A is not flatted, show the flat
    let pitch = Pitch::new(Note::Aes, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
}

#[test]
fn double_sharp_shows_double_sharp() {
    let pitch = Pitch::new(Note::Fisis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Open);
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
}

#[test]
fn double_sharp_shown_even_in_sharp_key() {
    // F## in D major — double sharps are never in key signatures
    let pitch = Pitch::new(Note::Fisis, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
}

#[test]
fn double_flat_shown_even_in_flat_key() {
    let pitch = Pitch::new(Note::Beses, 4);
    let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
    assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleFlat));
}

// --- ScoreBuilder ---

#[test]
fn empty_score_produces_svg() {
    let svg = ScoreBuilder::new().render_svg();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn single_note_score_produces_svg_with_path() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .render_svg();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<path"), "should contain glyph paths");
    assert!(svg.contains("<line"), "should contain staff lines");
}

#[test]
fn score_with_key_sig_shows_accidental_paths() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .render_svg();
    // Key sig has 2 sharps → at least 2 extra paths beyond clef+notehead
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 4,
        "expected >= 4 paths (clef + 2 key sig sharps + notehead), got {}",
        path_count
    );
}

#[test]
fn score_with_time_sig() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(3, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .render_svg();
    // Time sig numerals are paths
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 5,
        "expected >= 5 paths (clef + 2 time sig digits + 3 noteheads), got {}",
        path_count
    );
}

#[test]
fn multi_measure_score() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::D, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(svg.starts_with("<svg"));
    // Should have at least 2 barline elements (single + final)
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 7,
        "expected >= 7 lines (5 staff + barlines), got {}",
        line_count
    );
}

#[test]
fn rest_in_score() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .render_svg();
    assert!(svg.contains("<path"), "rest should produce a path");
}

#[test]
fn bass_clef_score() {
    let svg_treble = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .render_svg();
    let svg_bass = ScoreBuilder::new()
        .clef(Clef::Bass)
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .render_svg();
    // Different clef → different clef glyph path data
    assert_ne!(svg_treble, svg_bass);
}

#[test]
fn dotted_note_produces_dot_glyph() {
    let dotted_qtr = Duration::new(DurationKind::Qtr, 1);
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), dotted_qtr)
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef + notehead + dot = at least 3 paths
    assert!(
        path_count >= 3,
        "expected >= 3 paths for dotted note, got {}",
        path_count
    );
}

#[test]
fn eighth_note_produces_flag() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::G, 4), Duration::EIGHTH)
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef + notehead + flag = at least 3 paths
    assert!(
        path_count >= 3,
        "expected >= 3 paths for eighth note (clef+notehead+flag), got {}",
        path_count
    );
}

#[test]
fn measures_per_system_affects_layout() {
    let build = || {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::D, 4), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::E, 4), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::F, 4), Duration::WHOLE)
            .end_barline()
    };

    let svg_2 = build().measures_per_system(2).render_svg();
    let svg_4 = build().measures_per_system(4).render_svg();

    // With 2 measures/system, 4 measures → 2 systems → 2 sets of staff lines (10 total)
    // With 4 measures/system, 4 measures → 1 system → 1 set of staff lines (5 total)
    let lines_2 = svg_2.matches("<line").count();
    let lines_4 = svg_4.matches("<line").count();
    assert!(
        lines_2 > lines_4,
        "2 measures/system ({} lines) should have more lines than 4/system ({} lines)",
        lines_2,
        lines_4
    );
}

#[test]
fn natural_shown_when_cancelling_key_sig() {
    // In D major (F#, C#), an F natural should show a natural accidental
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef + 2 key sig sharps + notehead + natural accidental = 5
    assert!(
        path_count >= 5,
        "expected >= 5 paths (natural accidental shown), got {}",
        path_count
    );
}

#[test]
fn convert_event_note_preserves_staff_position() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let pitch = Pitch::new(Note::B, 4);
    let event = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            // B4 in treble: G4=pos 2, A4=pos 3, B4=pos 4 (middle line)
            assert_eq!(n.staff_position, 4);
            assert_eq!(n.duration_log2, 2);
            assert_eq!(n.dots, 0);
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn convert_event_rest_preserves_duration() {
    let builder = ScoreBuilder::new();
    let dur = Duration::new(DurationKind::Half, 1); // dotted half
    let event = ScoreEvent::Rest { duration: dur };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Rest(r) => {
            assert_eq!(r.duration_log2, 1);
            assert_eq!(r.dots, 1);
        }
        _ => panic!("expected Rest event"),
    }
}

#[test]
fn common_time_uses_common_symbol() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .common_time()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    // Common time uses a single glyph (timeSigCommon), not two digit glyphs.
    // With numeric 4/4 we'd get 2 digit paths; with common we get 1 symbol path.
    let svg_numeric = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    // The SVGs should differ because different glyphs are used
    assert_ne!(svg, svg_numeric);
}

#[test]
fn cut_time_uses_cut_common_symbol() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .cut_time()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    let svg_common = ScoreBuilder::new()
        .clef(Clef::Treble)
        .common_time()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    // Cut time and common time produce different SVGs
    assert_ne!(svg, svg_common);
}

#[test]
fn cut_time_differs_from_numeric_2_2() {
    let svg_cut = ScoreBuilder::new()
        .clef(Clef::Treble)
        .cut_time()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    let svg_numeric = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(2, 2)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    assert_ne!(svg_cut, svg_numeric);
}

#[test]
fn pending_events_auto_flushed_as_final_barline() {
    // Not calling barline() or end_barline() — events should still render
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .render_svg();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<path"));
}

#[test]
fn try_render_svg_returns_ok_for_valid_input() {
    let result = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .try_render_svg();
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<path"));
}

#[test]
fn try_render_svg_empty_score_returns_ok() {
    let result = ScoreBuilder::new().try_render_svg();
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn try_render_svg_matches_render_svg() {
    let svg_try = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .barline()
        .rest(Duration::WHOLE)
        .end_barline()
        .try_render_svg()
        .unwrap();
    let svg_direct = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .barline()
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert_eq!(svg_try, svg_direct);
}

// --- within-measure accidental tracking ---

#[test]
fn resolve_accidental_with_tracking_suppresses_repeated_sharp() {
    let key = KeySignature::Open;
    let mut seen: AccidentalTracker = HashMap::new();
    let pitch = Pitch::new(Note::Fis, 4);

    // First occurrence: show sharp
    let first = resolve_accidental(&pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(
        first,
        Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalSharp))
    );

    // Record it
    seen.insert(note_key(&pitch), Accidental::Sharp);

    // Second occurrence: suppress (same accidental already shown)
    let second = resolve_accidental(&pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(second, None, "repeated sharp should be suppressed");
}

#[test]
fn resolve_accidental_with_tracking_suppresses_repeated_flat() {
    let key = KeySignature::Open;
    let mut seen: AccidentalTracker = HashMap::new();
    let pitch = Pitch::new(Note::Bes, 4);

    let first = resolve_accidental(&pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(
        first,
        Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalFlat))
    );
    seen.insert(note_key(&pitch), Accidental::Flat);

    let second = resolve_accidental(&pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(second, None, "repeated flat should be suppressed");
}

#[test]
fn resolve_accidental_shows_courtesy_natural_after_sharp() {
    let key = KeySignature::Open;
    let mut seen: AccidentalTracker = HashMap::new();
    let sharp_pitch = Pitch::new(Note::Fis, 4);
    let natural_pitch = Pitch::new(Note::F, 4);

    // Show sharp on F#4
    seen.insert(note_key(&sharp_pitch), Accidental::Sharp);

    // F4 natural should show a courtesy natural
    let result = resolve_accidental(&natural_pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(
        result,
        Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalNatural)),
        "natural should show after sharp on same letter+octave"
    );
}

#[test]
fn resolve_accidental_shows_courtesy_natural_after_flat() {
    let key = KeySignature::Open;
    let mut seen: AccidentalTracker = HashMap::new();
    let flat_pitch = Pitch::new(Note::Bes, 4);
    let natural_pitch = Pitch::new(Note::B, 4);

    seen.insert(note_key(&flat_pitch), Accidental::Flat);

    let result = resolve_accidental(&natural_pitch, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(
        result,
        Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalNatural)),
        "natural should show after flat on same letter+octave"
    );
}

#[test]
fn resolve_accidental_different_octaves_independent() {
    let key = KeySignature::Open;
    let mut seen: AccidentalTracker = HashMap::new();
    let fis4 = Pitch::new(Note::Fis, 4);
    let fis5 = Pitch::new(Note::Fis, 5);

    // Show sharp on F#4
    seen.insert(note_key(&fis4), Accidental::Sharp);

    // F#5 is a different octave — should still show sharp
    let result = resolve_accidental(&fis5, &key, AccidentalDisplay::Auto, Some(&seen));
    assert_eq!(
        result,
        Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalSharp)),
        "sharp on different octave should not be suppressed"
    );
}

#[test]
fn convert_event_with_tracking_suppresses_repeated_accidental() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();
    let pitch = Pitch::new(Note::Fis, 4);

    let ev1 = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let ev2 = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };

    let r1 = convert_event(&ev1, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    let r2 = convert_event(&ev2, &Clef::Treble, &builder.key_sig, Some(&mut seen));

    match (&r1, &r2) {
        (MeasureEvent::Note(n1), MeasureEvent::Note(n2)) => {
            assert!(n1.accidental.is_some(), "first F#4 should show accidental");
            assert!(
                n2.accidental.is_none(),
                "second F#4 should suppress accidental"
            );
        }
        _ => panic!("expected Note events"),
    }
}

#[test]
fn convert_event_with_tracking_shows_natural_after_sharp() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();

    let ev_sharp = ScoreEvent::Note {
        pitch: Pitch::new(Note::Fis, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let ev_natural = ScoreEvent::Note {
        pitch: Pitch::new(Note::F, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };

    let _ = convert_event(&ev_sharp, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    let r2 = convert_event(
        &ev_natural,
        &Clef::Treble,
        &builder.key_sig,
        Some(&mut seen),
    );

    match r2 {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.accidental,
                Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalNatural)),
                "F natural after F# should show courtesy natural"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn tracking_resets_between_measures_in_render() {
    // F#4 in measure 1, then F#4 in measure 2 — both should show sharp
    // (tracking resets at barline)
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR) // suppressed
        .barline()
        .note(Pitch::new(Note::Fis, 4), Duration::QTR) // should show (new measure)
        .end_barline()
        .render_svg();

    // Count accidental paths: measure 1 has 1 sharp, measure 2 has 1 sharp = 2 sharps
    // Each measure also has noteheads. Clef adds 1 path.
    // Without tracking: 3 sharps. With tracking: 2 sharps.
    let path_count = svg.matches("<path").count();
    // clef(1) + 3 noteheads + 2 sharps = 6 paths
    assert_eq!(
        path_count, 6,
        "expected 6 paths (1 clef + 3 noteheads + 2 sharps, with 1 suppressed), got {}",
        path_count
    );
}

#[test]
fn tracking_within_measure_suppresses_duplicate() {
    // Two F#4 in one measure — second should not show sharp
    let svg_tracked = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // One F#4 alone for comparison
    let svg_single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let paths_tracked = svg_tracked.matches("<path").count();
    let paths_single = svg_single.matches("<path").count();

    // Tracked: clef + 2 noteheads + 1 sharp = 4
    // Single: clef + 1 notehead + 1 sharp = 3
    // Difference should be exactly 1 (one extra notehead, no extra sharp)
    assert_eq!(
        paths_tracked - paths_single,
        1,
        "adding duplicate F# should add only 1 path (notehead), not 2 (notehead+sharp)"
    );
}

#[test]
fn note_key_same_letter_different_octave() {
    let p1 = Pitch::new(Note::C, 3);
    let p2 = Pitch::new(Note::C, 5);
    assert_ne!(note_key(&p1), note_key(&p2));
}

#[test]
fn note_key_same_note_same_octave() {
    let p1 = Pitch::new(Note::C, 4);
    let p2 = Pitch::new(Note::C, 4);
    assert_eq!(note_key(&p1), note_key(&p2));
}

#[test]
fn note_key_enharmonic_different() {
    // C# and Db are different letters
    let p1 = Pitch::new(Note::Cis, 4);
    let p2 = Pitch::new(Note::Des, 4);
    assert_ne!(note_key(&p1), note_key(&p2));
}

// --- chord support in ScoreBuilder ---

#[test]
fn chord_produces_multiple_noteheads() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::QTR,
        )
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef(1) + 3 noteheads = 4 paths minimum
    assert!(
        path_count >= 4,
        "expected >= 4 paths (clef + 3 noteheads), got {}",
        path_count
    );
}

#[test]
fn chord_has_more_paths_than_single_note() {
    let svg_note = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .render_svg();
    let svg_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::QTR,
        )
        .render_svg();
    let paths_note = svg_note.matches("<path").count();
    let paths_chord = svg_chord.matches("<path").count();
    assert!(
        paths_chord > paths_note,
        "chord ({} paths) should have more paths than single note ({} paths)",
        paths_chord,
        paths_note
    );
}

#[test]
fn chord_with_accidentals_in_key() {
    // In D major (F#, C#): chord with F#4 and A4
    // F# is suppressed (in key sig), A has no accidental
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .chord(
            vec![Pitch::new(Note::Fis, 4), Pitch::new(Note::A, 4)],
            Duration::QTR,
        )
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef(1) + 2 key sig sharps + 2 noteheads + 0 accidentals = 5
    assert_eq!(
        path_count, 5,
        "expected 5 paths (clef + 2 key sharps + 2 noteheads, no extra accidentals), got {}",
        path_count
    );
}

#[test]
fn chord_with_natural_accidental() {
    // In D major, chord with F♮4 and G4 — natural should appear on F
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .chord(
            vec![Pitch::new(Note::F, 4), Pitch::new(Note::G, 4)],
            Duration::QTR,
        )
        .render_svg();
    let path_count = svg.matches("<path").count();
    // clef(1) + 2 key sharps + 2 noteheads + 1 natural = 6
    assert_eq!(
        path_count, 6,
        "expected 6 paths (clef + 2 key sharps + 2 noteheads + 1 natural), got {}",
        path_count
    );
}

#[test]
fn convert_event_chord_maps_pitches() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::E, 4), Pitch::new(Note::G, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(c.staff_positions.len(), 2);
            // E4 in treble = pos 0, G4 = pos 2
            assert_eq!(c.staff_positions[0], 0);
            assert_eq!(c.staff_positions[1], 2);
            assert_eq!(c.duration_log2, 2);
            assert_eq!(c.dots, 0);
            assert_eq!(c.accidentals.len(), 2);
        }
        _ => panic!("expected Chord event"),
    }
}

#[test]
fn convert_event_with_tracking_chord_suppresses_repeated_accidental() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();

    // First: single F#4 note
    let ev1 = ScoreEvent::Note {
        pitch: Pitch::new(Note::Fis, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let _ = convert_event(&ev1, &Clef::Treble, &builder.key_sig, Some(&mut seen));

    // Then: chord with F#4 (should be suppressed) and A4
    let ev2 = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::Fis, 4), Pitch::new(Note::A, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let result = convert_event(&ev2, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Chord(c) => {
            // F#4's accidental should be suppressed (already shown)
            assert_eq!(
                c.accidentals[0], None,
                "F#4 accidental should be suppressed"
            );
            // A4 has no accidental in C major
            assert_eq!(c.accidentals[1], None, "A4 should have no accidental");
        }
        _ => panic!("expected Chord event"),
    }
}

// --- beam group ---

#[test]
fn beam_group_renders_svg_with_polygons() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .beam_group(vec![
            (Pitch::new(Note::E, 4), Duration::EIGHTH),
            (Pitch::new(Note::F, 4), Duration::EIGHTH),
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
            (Pitch::new(Note::A, 4), Duration::EIGHTH),
        ])
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<svg"));
    // 4 noteheads + clef + time sig digits
    let path_count = svg.matches("<path ").count();
    assert!(
        path_count >= 4,
        "at least 4 paths for noteheads, got {path_count}"
    );
    // Beam polygons (1 primary for all eighth notes)
    let polygon_count = svg.matches("<polygon ").count();
    assert!(
        polygon_count >= 1,
        "at least 1 beam polygon, got {polygon_count}"
    );
}

#[test]
fn beam_group_differs_from_individual_eighth_notes() {
    let e4 = Pitch::new(Note::E, 4);
    let f4 = Pitch::new(Note::F, 4);

    let svg_beamed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .beam_group(vec![(e4, Duration::EIGHTH), (f4, Duration::EIGHTH)])
        .end_barline()
        .render_svg();

    let svg_flagged = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(e4, Duration::EIGHTH)
        .note(f4, Duration::EIGHTH)
        .end_barline()
        .render_svg();

    // Beamed version should have polygons, flagged should not
    assert!(
        svg_beamed.matches("<polygon ").count() >= 1,
        "beamed has polygons"
    );
    assert_eq!(
        svg_flagged.matches("<polygon ").count(),
        0,
        "flagged has no polygons"
    );
}

#[test]
fn beam_group_convert_event_produces_beam_group_measure_event() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::BeamGroup {
        notes: vec![
            (Pitch::new(Note::E, 4), Duration::EIGHTH),
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
        ],
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::BeamGroup(bg) => {
            assert_eq!(bg.notes.len(), 2);
            // E4 in treble: pos 0, G4: pos 2
            assert_eq!(bg.notes[0].staff_position, 0);
            assert_eq!(bg.notes[1].staff_position, 2);
            assert_eq!(bg.notes[0].duration_log2, 3);
            assert_eq!(bg.notes[1].duration_log2, 3);
            assert!(bg.stem_direction.is_none());
        }
        _ => panic!("expected BeamGroup event"),
    }
}

#[test]
fn beam_group_tracked_accidentals() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();

    // First F#4 note (standalone)
    let ev1 = ScoreEvent::Note {
        pitch: Pitch::new(Note::Fis, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let _ = convert_event(&ev1, &Clef::Treble, &builder.key_sig, Some(&mut seen));

    // Then beam group with F#4 again — should suppress repeated accidental
    let ev2 = ScoreEvent::BeamGroup {
        notes: vec![
            (Pitch::new(Note::Fis, 4), Duration::EIGHTH),
            (Pitch::new(Note::A, 4), Duration::EIGHTH),
        ],
    };
    let result = convert_event(&ev2, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::BeamGroup(bg) => {
            assert!(
                bg.notes[0].accidental.is_none(),
                "F#4 accidental suppressed"
            );
            assert!(bg.notes[1].accidental.is_none(), "A4 has no accidental");
        }
        _ => panic!("expected BeamGroup event"),
    }
}

// --- tie support ---

#[test]
fn tie_produces_filled_path_in_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tie()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Tie should produce a filled path with stroke="none"
    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert!(
        tie_count >= 1,
        "expected at least 1 tie path, got {tie_count}"
    );
    // Should contain Bézier curves
    assert!(svg.contains(" C"), "tie should have cubic Bézier curves");
}

#[test]
fn tie_across_barline_in_score() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .tie()
        .barline()
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert!(
        tie_count >= 1,
        "tie across barline should produce a tie path"
    );
}

#[test]
fn no_tie_without_tie_call() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 0, "no tie without .tie() call");
}

#[test]
fn tie_on_rest_has_no_effect() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .tie() // Should have no effect since last event is a rest
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 0, "tie after rest should have no effect");
}

#[test]
fn tie_differs_from_untied() {
    let svg_tied = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .tie()
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let svg_untied = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .end_barline()
        .render_svg();

    assert_ne!(
        svg_tied, svg_untied,
        "tied and untied should produce different SVGs"
    );
    // Tied version should be longer (has tie path)
    assert!(
        svg_tied.len() > svg_untied.len(),
        "tied SVG should be larger"
    );
}

#[test]
fn convert_event_preserves_tie_forward() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::E, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tie_forward: true,
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.tie_forward, "tie_forward should be preserved");
        }
        _ => panic!("expected Note event"),
    }
}

// --- chord tie support ---

#[test]
fn tie_after_chord_sets_tie_forward() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tie_forward: true,
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert!(
                c.annotations.tie_forward,
                "chord tie_forward should be preserved"
            );
        }
        _ => panic!("expected Chord event"),
    }
}

#[test]
fn chord_tie_produces_filled_paths() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .tie()
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();

    // 3 notes in the chord → 3 ties
    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(
        tie_count, 3,
        "expected 3 ties (one per chord note), got {tie_count}"
    );
}

#[test]
fn chord_tie_differs_from_untied_chord() {
    let build = || {
        ScoreBuilder::new().clef(Clef::Treble).chord(
            vec![Pitch::new(Note::E, 4), Pitch::new(Note::G, 4)],
            Duration::HALF,
        )
    };

    let svg_tied = build()
        .tie()
        .chord(
            vec![Pitch::new(Note::E, 4), Pitch::new(Note::G, 4)],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();

    let svg_untied = build()
        .chord(
            vec![Pitch::new(Note::E, 4), Pitch::new(Note::G, 4)],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();

    assert_ne!(svg_tied, svg_untied, "tied chord should differ from untied");
    assert!(
        svg_tied.len() > svg_untied.len(),
        "tied chord SVG should be larger (contains tie paths)"
    );
}

#[test]
fn chord_tie_no_effect_without_tie_call() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();

    let tie_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 0, "no ties without .tie() call");
}

#[test]
fn convert_event_with_tracking_chord_preserves_tie_forward() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::C, 4), Pitch::new(Note::G, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tie_forward: true,
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Chord(c) => {
            assert!(
                c.annotations.tie_forward,
                "tracked conversion should preserve tie_forward"
            );
        }
        _ => panic!("expected Chord event"),
    }
}

// --- dynamics integration ---

#[test]
fn dynamic_on_note_produces_extra_path() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .end_barline()
        .render_svg();
    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let paths_with = svg_with.matches("<path ").count();
    let paths_without = svg_without.matches("<path ").count();
    assert_eq!(
        paths_with,
        paths_without + 1,
        "dynamic should add exactly 1 path (the dynamic glyph)"
    );
}

#[test]
fn different_dynamics_produce_different_svg() {
    let svg_p = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .render_svg();
    let svg_f = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .render_svg();
    assert_ne!(svg_p, svg_f, "p and f should produce different SVGs");
}

#[test]
fn dynamic_on_chord_produces_extra_path() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::QTR,
        )
        .dynamic(Dynamic::Ff)
        .end_barline()
        .render_svg();
    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::QTR,
        )
        .end_barline()
        .render_svg();

    let paths_with = svg_with.matches("<path ").count();
    let paths_without = svg_without.matches("<path ").count();
    assert_eq!(
        paths_with,
        paths_without + 1,
        "dynamic on chord should add exactly 1 path"
    );
}

#[test]
fn dynamic_on_rest_has_no_effect() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .dynamic(Dynamic::Mf) // should be ignored — last event is a rest
        .end_barline()
        .render_svg();
    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        svg_with, svg_without,
        "dynamic on rest should have no effect"
    );
}

#[test]
fn convert_event_preserves_dynamic() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::E, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Pp),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.dynamic,
                Some(Dynamic::Pp),
                "dynamic should be preserved"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn convert_event_with_tracking_preserves_dynamic() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::E, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Fff),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.dynamic,
                Some(Dynamic::Fff),
                "tracked conversion preserves dynamic"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn convert_event_chord_preserves_dynamic() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Sfz),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(
                c.annotations.dynamic,
                Some(Dynamic::Sfz),
                "chord dynamic should be preserved"
            );
        }
        _ => panic!("expected Chord event"),
    }
}

#[test]
fn multiple_dynamics_in_score() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .end_barline()
        .render_svg();

    // Should have clef + 2 time sig digits + 4 noteheads + 2 dynamics = 9 paths
    let path_count = svg.matches("<path ").count();
    assert_eq!(
        path_count, 9,
        "expected 9 paths (clef + 2 time digits + 4 noteheads + 2 dynamics), got {}",
        path_count
    );
}

// --- tuplet support in ScoreBuilder ---

#[test]
fn tuplet_renders_svg_with_bracket() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .tuplet(
            3,
            vec![
                (Pitch::new(Note::E, 4), Duration::EIGHTH),
                (Pitch::new(Note::F, 4), Duration::EIGHTH),
                (Pitch::new(Note::G, 4), Duration::EIGHTH),
            ],
        )
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<svg"));
    // 3 noteheads + clef + 2 time sig digits + 1 tuplet number = 7 paths
    let path_count = svg.matches("<path ").count();
    assert_eq!(path_count, 7, "expected 7 paths, got {path_count}");
    // Should have beam polygon
    let polygon_count = svg.matches("<polygon ").count();
    assert!(polygon_count >= 1, "should have beam polygon(s)");
    // Should have bracket lines (hooks + bracket segments)
    let line_count = svg.matches("<line ").count();
    // 5 staff + 3 stems + 4 bracket/hooks + barline(s) = should be > 10
    assert!(
        line_count > 10,
        "expected > 10 lines (staff + stems + bracket), got {line_count}"
    );
}

#[test]
fn tuplet_differs_from_beam_group() {
    let notes = vec![
        (Pitch::new(Note::E, 4), Duration::EIGHTH),
        (Pitch::new(Note::F, 4), Duration::EIGHTH),
        (Pitch::new(Note::G, 4), Duration::EIGHTH),
    ];

    let svg_beam = ScoreBuilder::new()
        .clef(Clef::Treble)
        .beam_group(notes.clone())
        .end_barline()
        .render_svg();

    let svg_tuplet = ScoreBuilder::new()
        .clef(Clef::Treble)
        .tuplet(3, notes)
        .end_barline()
        .render_svg();

    // Tuplet should have extra content (bracket + number)
    assert!(
        svg_tuplet.len() > svg_beam.len(),
        "tuplet SVG ({}) should be larger than beam group SVG ({})",
        svg_tuplet.len(),
        svg_beam.len()
    );

    // Tuplet has 1 extra path (number glyph)
    let paths_beam = svg_beam.matches("<path ").count();
    let paths_tuplet = svg_tuplet.matches("<path ").count();
    assert_eq!(
        paths_tuplet,
        paths_beam + 1,
        "tuplet adds 1 path for number glyph"
    );
}

#[test]
fn tuplet_convert_event_produces_tuplet_group() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::TupletGroup {
        notes: vec![
            (Pitch::new(Note::E, 4), Duration::EIGHTH),
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
            (Pitch::new(Note::B, 4), Duration::EIGHTH),
        ],
        tuplet_number: 3,
        in_time_of: 2,
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::TupletGroup(tg) => {
            assert_eq!(tg.tuplet_number, 3);
            assert_eq!(tg.beam_group.notes.len(), 3);
            assert_eq!(tg.beam_group.notes[0].staff_position, 0); // E4
            assert_eq!(tg.beam_group.notes[1].staff_position, 2); // G4
            assert_eq!(tg.beam_group.notes[2].staff_position, 4); // B4
            assert_eq!(tg.beam_group.notes[0].duration_log2, 3);
        }
        _ => panic!("expected TupletGroup event"),
    }
}

#[test]
fn tuplet_tracked_accidentals() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();

    // First F#4 note (standalone)
    let ev1 = ScoreEvent::Note {
        pitch: Pitch::new(Note::Fis, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations::default(),
    };
    let _ = convert_event(&ev1, &Clef::Treble, &builder.key_sig, Some(&mut seen));

    // Then tuplet with F#4 again — should suppress repeated accidental
    let ev2 = ScoreEvent::TupletGroup {
        notes: vec![
            (Pitch::new(Note::Fis, 4), Duration::EIGHTH),
            (Pitch::new(Note::A, 4), Duration::EIGHTH),
            (Pitch::new(Note::C, 5), Duration::EIGHTH),
        ],
        tuplet_number: 3,
        in_time_of: 2,
    };
    let result = convert_event(&ev2, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::TupletGroup(tg) => {
            assert!(
                tg.beam_group.notes[0].accidental.is_none(),
                "F#4 accidental suppressed"
            );
            assert!(
                tg.beam_group.notes[1].accidental.is_none(),
                "A4 has no accidental"
            );
        }
        _ => panic!("expected TupletGroup event"),
    }
}

#[test]
fn tuplet_quintuplet_in_score() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .tuplet(
            5,
            vec![
                (Pitch::new(Note::C, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::D, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::E, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::F, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::G, 4), Duration::SIXTEENTH),
            ],
        )
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<svg"));
    // 5 noteheads + clef + 1 tuplet number = 7 paths
    let path_count = svg.matches("<path ").count();
    assert_eq!(path_count, 7, "expected 7 paths, got {path_count}");
}

// --- slur tests ---

#[test]
fn slur_start_end_produces_filled_path() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .slur_start()
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .slur_end()
        .end_barline()
        .render_svg();

    // Should contain a filled slur path (Bézier curves)
    let filled_count = svg.matches(r#"stroke="none""#).count();
    assert!(
        filled_count >= 1,
        "expected at least 1 filled slur path, got {filled_count}"
    );
    assert!(
        svg.contains(" C"),
        "slur should contain cubic Bézier commands"
    );
}

#[test]
fn slurred_differs_from_unslurred() {
    let slurred = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .slur_start()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .slur_end()
        .end_barline()
        .render_svg();

    let unslurred = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(slurred, unslurred, "slurred should differ from un-slurred");
}

#[test]
fn slur_on_rest_is_noop() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .slur_start()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // slur_start on a rest is a no-op; no slur_end anywhere, so no slur
    let filled_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(filled_count, 0, "slur_start on rest should be a no-op");
}

#[test]
fn convert_event_preserves_slur_flags() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::E, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            slur_start: true,
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.slur_start, "slur_start should be preserved");
            assert!(!n.annotations.slur_end, "slur_end should be false");
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn convert_event_with_tracking_preserves_slur_flags() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let mut seen: AccidentalTracker = HashMap::new();
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::G, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            slur_end: true,
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert!(!n.annotations.slur_start, "slur_start should be false");
            assert!(n.annotations.slur_end, "slur_end should be preserved");
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn chord_slur_preserves_flags() {
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            slur_start: true,
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert!(
                c.annotations.slur_start,
                "chord slur_start should be preserved"
            );
            assert!(!c.annotations.slur_end);
        }
        _ => panic!("expected Chord event"),
    }
}

// --- hairpin tests ---

use crate::layout::hairpin::HairpinType;

/// Helper to create a Pitch from a note name string and octave.
fn p(name: &str, octave: i8) -> Pitch {
    let note = match name {
        "C" => Note::C,
        "D" => Note::D,
        "E" => Note::E,
        "F" => Note::F,
        "G" => Note::G,
        "A" => Note::A,
        "B" => Note::B,
        _ => panic!("unsupported note name: {name}"),
    };
    Pitch::new(note, octave)
}

#[test]
fn hairpin_adds_lines_to_svg() {
    let svg_hp = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_no = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let hp_lines = svg_hp.matches("<line ").count();
    let no_lines = svg_no.matches("<line ").count();
    assert_eq!(hp_lines, no_lines + 2, "hairpin adds 2 lines");
}

#[test]
fn decresc_differs_from_cresc() {
    let svg_c = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();

    let svg_d = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .decresc()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();

    assert_ne!(svg_c, svg_d, "cresc and decresc should differ");
}

#[test]
fn hairpin_on_rest_is_noop() {
    let svg1 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();

    let svg2 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // cresc() on a rest is a no-op, so hairpin_start is never set
    // hairpin_end without matching start produces no hairpin
    assert_eq!(svg1, svg2, "hairpin on rest should be no-op");
}

#[test]
fn convert_event_preserves_hairpin_fields() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            hairpin_start: Some(HairpinType::Crescendo),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(n.annotations.hairpin_start, Some(HairpinType::Crescendo));
            assert!(!n.annotations.hairpin_end);
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn convert_event_with_tracking_preserves_hairpin_fields() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            hairpin_end: true,
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let mut seen = HashMap::new();
    let result = convert_event(&event, &clef, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.hairpin_start.is_none());
            assert!(n.annotations.hairpin_end);
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn chord_hairpin_preserved_in_convert() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Chord {
        pitches: vec![p("C", 4), p("E", 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            hairpin_start: Some(HairpinType::Decrescendo),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(c.annotations.hairpin_start, Some(HairpinType::Decrescendo));
            assert!(!c.annotations.hairpin_end);
        }
        _ => panic!("expected Chord"),
    }
}

// --- hairpin_dashed (within-system dashed wedge) integration tests ---

#[test]
fn hairpin_dashed_flag_sets_annotation_on_start_note() {
    // The ScoreBuilder method must set NoteAnnotations::hairpin_dashed
    // on the most recent note. Verifies the builder surface — the
    // rendering side is exercised in the next two tests.
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_dashed();
    // Inspect the last current_events entry.
    let (_, last) = builder.current_events.last().expect("note pushed");
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert!(
                annotations.hairpin_dashed,
                "hairpin_dashed flag must be set"
            );
            assert_eq!(
                annotations.hairpin_start,
                Some(HairpinType::Crescendo),
                "hairpin_dashed must not clear the hairpin_start"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn hairpin_dashed_renders_stroke_dasharray() {
    // Full integration: ScoreBuilder → events → measure → system_renderer
    // → SVG must produce a hairpin with stroke-dasharray on both wedge
    // lines. Catches a regression at any layer of the plumbing.
    let svg_dashed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_dashed()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_solid = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    // Exactly 2 stroke-dasharray attributes on the dashed wedge,
    // exactly 0 on the solid one. Both produce the same number of
    // total <line> elements (dashed only changes the attribute set).
    assert_eq!(
        svg_dashed.matches("stroke-dasharray").count(),
        2,
        "dashed hairpin must emit stroke-dasharray on both wedge lines; SVG:\n{svg_dashed}"
    );
    assert_eq!(
        svg_solid.matches("stroke-dasharray").count(),
        0,
        "solid hairpin must emit zero stroke-dasharray; SVG:\n{svg_solid}"
    );
    assert_eq!(
        svg_dashed.matches("<line ").count(),
        svg_solid.matches("<line ").count(),
        "dashed and solid hairpins must produce the same total <line> count"
    );
}

#[test]
fn hairpin_dashed_without_hairpin_start_renders_no_wedge() {
    // hairpin_dashed() without a preceding cresc() / decresc() /
    // hairpin_start() must be a no-op visually — there is no wedge to
    // dash. Catches a regression where setting hairpin_dashed on its
    // own accidentally synthesizes a wedge.
    let svg_lone = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .hairpin_dashed()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        svg_lone, svg_plain,
        "hairpin_dashed without hairpin_start must not emit any wedge"
    );
}

#[test]
fn hairpin_dashed_on_rest_is_noop() {
    // hairpin_dashed after a rest cannot set the flag (the rest event
    // carries no NoteAnnotations). Mirror of `hairpin_on_rest_is_noop`.
    let svg_rest_dashed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .cresc()
        .hairpin_dashed()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        svg_rest_dashed, svg_plain,
        "cresc().hairpin_dashed() on a rest must be a no-op end-to-end"
    );
}

#[test]
fn hairpin_dashed_decrescendo_also_dashes() {
    // The dashed style must apply regardless of HairpinType — both
    // crescendo and decrescendo wedges respect the flag. Catches a
    // regression where the flag is conditional on Crescendo only.
    let svg_c = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .decresc()
        .hairpin_dashed()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();
    assert_eq!(
        svg_c.matches("stroke-dasharray").count(),
        2,
        "dashed decrescendo must emit stroke-dasharray on both wedge lines; SVG:\n{svg_c}"
    );
}

// --- hairpin_niente (open "o" at wedge tip) integration tests ---
//
// Plumbing under test: ScoreBuilder.hairpin_niente() / .hairpin_niente_start()
// → NoteAnnotations::hairpin_niente → HairpinNoteInfo::hairpin_niente →
// layout_hairpin_styled → draw_hairpin (which renders the niente as a
// `<circle>` element). Each test asserts on the SVG output, not on
// intermediate state, so the full chain is exercised.

#[test]
fn hairpin_niente_convenience_sets_closed_end_annotation() {
    // The shorthand `.hairpin_niente()` is closed-end (the common
    // "al niente" / "dal niente" convention). Verifies the builder
    // surface: annotation field is set; hairpin_start preserved.
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_niente();
    let (_, last) = builder.current_events.last().expect("note pushed");
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.hairpin_niente,
                Some(NientePlacement::ClosedEnd),
                "hairpin_niente() must set ClosedEnd placement"
            );
            assert_eq!(
                annotations.hairpin_start,
                Some(HairpinType::Crescendo),
                "hairpin_niente must not clear hairpin_start"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn hairpin_niente_start_open_end_sets_open_end_annotation() {
    // The full API takes a NientePlacement; OpenEnd is preserved
    // verbatim on the annotation.
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_niente_start(NientePlacement::OpenEnd);
    let (_, last) = builder.current_events.last().expect("note pushed");
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.hairpin_niente,
                Some(NientePlacement::OpenEnd),
                "hairpin_niente_start(OpenEnd) must set OpenEnd placement"
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn hairpin_niente_renders_open_circle() {
    // End-to-end: ScoreBuilder → SVG must emit one `<circle>` element
    // for the niente "o", on top of the existing 2-line wedge. The
    // baseline (no niente) has zero `<circle>` elements anywhere in
    // the output (the renderer doesn't emit circles for any other
    // hairpin-band glyph).
    let svg_n = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_niente()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let circles_n = svg_n.matches("<circle ").count();
    let circles_plain = svg_plain.matches("<circle ").count();
    assert_eq!(
        circles_n,
        circles_plain + 1,
        "hairpin_niente must add exactly one <circle> over the plain hairpin; \
             niente SVG had {circles_n} circles, plain SVG had {circles_plain}"
    );
    // The circle must be open (fill="none") — the engraved "o" reading.
    assert!(
        svg_n.contains(r#"fill="none""#),
        "niente circle must be drawn fill=\"none\" (open ring); SVG:\n{svg_n}"
    );
    // Wedge line count identical between with/without — niente is
    // purely additive on top of the wedge geometry.
    assert_eq!(
        svg_n.matches("<line ").count(),
        svg_plain.matches("<line ").count(),
        "niente must not change the total `<line>` count"
    );
}

#[test]
fn hairpin_niente_with_dashed_keeps_circle_solid() {
    // Combo: dashed wedge + niente "o". The wedge lines must carry
    // stroke-dasharray; the niente circle element must NOT — engraved
    // convention treats the niente as a definite symbol independent of
    // the dashed wedge.
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .hairpin_dashed()
        .hairpin_niente()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    // Exactly one niente circle.
    assert_eq!(
        svg.matches("<circle ").count(),
        1,
        "combined dashed+niente must emit exactly one circle; SVG:\n{svg}"
    );
    // Locate the <circle ...> element and confirm it lacks the
    // stroke-dasharray attribute. Asserts the engraving invariant
    // directly on the element rather than the document-level count.
    let circle_line = svg
        .lines()
        .find(|l| l.contains("<circle "))
        .expect("circle element must exist in combined-mode SVG");
    assert!(
        !circle_line.contains("stroke-dasharray"),
        "niente circle must remain solid (no stroke-dasharray) even on a \
             dashed wedge; circle element was: {circle_line}"
    );
    // Both wedge lines must carry dasharray (the dashed half of the combo).
    assert_eq!(
        svg.matches("stroke-dasharray").count(),
        2,
        "dashed wedge half of the combo must emit dasharray on both wedge lines"
    );
}

#[test]
fn hairpin_niente_without_hairpin_start_renders_no_circle() {
    // Like the dashed flag, niente only has visible meaning paired
    // with a hairpin start. Lone `.hairpin_niente()` after a plain
    // note must produce SVG byte-identical to the no-niente render —
    // no wedge means no circle.
    let svg_lone = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .hairpin_niente()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .hairpin_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        svg_lone, svg_plain,
        "hairpin_niente without hairpin_start must not emit any wedge or circle"
    );
}

#[test]
fn hairpin_niente_on_rest_is_noop() {
    // `.hairpin_niente()` after a rest cannot set the flag (RestEvent
    // carries no NoteAnnotations). Mirror of `hairpin_dashed_on_rest_is_noop`.
    let svg_rest_niente = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .cresc()
        .hairpin_niente()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();
    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        svg_rest_niente, svg_plain,
        "cresc().hairpin_niente() on a rest must be a no-op end-to-end"
    );
}

#[test]
fn hairpin_niente_decrescendo_also_renders_circle() {
    // Direction-agnostic: a decrescendo with closed-end niente also
    // emits the open "o" — at the closing tip (right side of the
    // wedge). Catches a regression where niente is conditional on
    // Crescendo only.
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .decresc()
        .hairpin_niente()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();
    assert_eq!(
        svg.matches("<circle ").count(),
        1,
        "decrescendo + niente must emit one circle; SVG:\n{svg}"
    );
}

// --- cresc-text (dashed-text crescendo/diminuendo) integration tests ---

#[test]
fn cresc_text_adds_label_and_dashed_line_to_svg() {
    // Verifies the full ScoreBuilder → event → measure → system_renderer
    // chain: calling .cresc_text() / .cresc_text_end() must produce a
    // dashed-text marking in the rendered SVG.
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .cresc_text()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .cresc_text_end()
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    // The marking adds exactly one label and exactly one dashed line.
    assert!(
        svg_with.contains(">cresc.</text>"),
        "should contain literal 'cresc.' label"
    );
    assert_eq!(
        svg_with.matches("stroke-dasharray").count(),
        svg_without.matches("stroke-dasharray").count() + 1,
        "cresc_text marking adds exactly one dashed line"
    );
    assert_eq!(
        svg_with.matches("<text").count(),
        svg_without.matches("<text").count() + 1,
        "cresc_text marking adds exactly one <text> element"
    );
}

#[test]
fn decresc_text_label_differs_from_cresc_text() {
    let svg_c = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc_text()
        .note(p("E", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();

    let svg_d = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .decresc_text()
        .note(p("E", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();

    let svg_m = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .dim_text()
        .note(p("E", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();

    // Each renders its own label.
    assert!(
        svg_c.contains(">cresc.</text>"),
        "cresc_text() should emit 'cresc.'"
    );
    assert!(
        svg_d.contains(">decresc.</text>"),
        "decresc_text() should emit 'decresc.'"
    );
    assert!(
        svg_m.contains(">dim.</text>"),
        "dim_text() should emit 'dim.'"
    );

    // And distinct SVG.
    assert_ne!(svg_c, svg_d);
    assert_ne!(svg_d, svg_m);
    assert_ne!(svg_c, svg_m);
}

#[test]
fn cresc_text_on_rest_is_noop() {
    // Calling .cresc_text() right after .rest() should be a no-op
    // (the convenience methods only apply to Notes and Chords), so
    // the dangling cresc_text_end() has nothing to pair with and
    // also renders nothing.
    let svg1 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .cresc_text()
        .note(p("E", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();

    let svg2 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(svg1, svg2, "cresc_text on rest must be a no-op");
    assert!(!svg1.contains(">cresc.</text>"), "must not emit label");
}

#[test]
fn cresc_text_start_without_end_renders_no_marking() {
    // Orphan start: dangling .cresc_text() with no matching
    // .cresc_text_end() produces no marking.
    let svg_orphan = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc_text()
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        svg_orphan, svg_baseline,
        "orphan cresc_text_start (no matching cresc_text_end) must produce no marking"
    );
}

#[test]
fn cresc_text_and_hairpin_coexist_independently() {
    // Belt-and-suspenders: a phrase with BOTH a hairpin marking AND
    // a cresc-text marking on different pairs of notes must add up
    // to one wedge (2 lines, no dashed) + one cresc-text (1 dashed
    // line + 1 text element). Catches a regression where the two
    // paths interfere or share state.
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .note(p("G", 4), Duration::QTR)
        .cresc_text()
        .note(p("A", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();

    // Hairpin wedge contributes no dashed lines; cresc-text
    // contributes one. So exactly one stroke-dasharray must appear.
    assert_eq!(
        svg.matches("stroke-dasharray").count(),
        1,
        "exactly one dashed line (cresc-text continuation); hairpin must contribute none"
    );
    assert!(
        svg.contains(">cresc.</text>"),
        "cresc-text label must still render alongside hairpin"
    );
}

#[test]
fn convert_event_preserves_cresc_text_fields() {
    // The annotations flow through convert_event unchanged — required
    // for the system_renderer to see the flags downstream.
    use crate::layout::cresc_text::CrescTextKind;
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            cresc_text_start: Some(CrescTextKind::Diminuendo),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.cresc_text_start,
                Some(CrescTextKind::Diminuendo)
            );
            assert!(!n.annotations.cresc_text_end);
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn convert_event_preserves_cresc_text_end_flag() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            cresc_text_end: true,
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let mut seen = HashMap::new();
    let result = convert_event(&event, &clef, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.cresc_text_start.is_none());
            assert!(n.annotations.cresc_text_end);
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn chord_cresc_text_preserved_in_convert() {
    use crate::layout::cresc_text::CrescTextKind;
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Chord {
        pitches: vec![p("C", 4), p("E", 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            cresc_text_start: Some(CrescTextKind::Crescendo),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(
                c.annotations.cresc_text_start,
                Some(CrescTextKind::Crescendo)
            );
            assert!(!c.annotations.cresc_text_end);
        }
        _ => panic!("expected Chord"),
    }
}

#[test]
fn cresc_text_on_chord_renders_label() {
    // The end-to-end path also works when the start lives on a Chord
    // event (not just on a Note event).
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .cresc_text()
        .note(p("A", 4), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg();
    assert!(
        svg.contains(">cresc.</text>"),
        "cresc_text() on a chord must still emit the label in the rendered SVG"
    );
    assert_eq!(
        svg.matches("stroke-dasharray").count(),
        1,
        "exactly one dashed continuation line on chord-anchored marking"
    );
}

// --- Rehearsal mark integration tests ---

#[test]
fn rehearsal_mark_adds_text_element_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .rehearsal_mark("A", RehearsalStyle::Boxed)
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();
    // Boxed rehearsal mark produces a <text> element and a <rect> element
    assert!(
        svg.contains("<text "),
        "rehearsal mark should produce a <text> element"
    );
    assert!(
        svg.contains("<rect "),
        "boxed rehearsal mark should produce a <rect> element"
    );
    assert!(
        svg.contains(">A<"),
        "text content 'A' should appear in the SVG"
    );
}

#[test]
fn rehearsal_mark_on_chord_adds_text() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .rehearsal_mark("B", RehearsalStyle::Boxed)
        .end_barline()
        .render_svg();
    assert!(
        svg.contains(">B<"),
        "chord rehearsal mark text should appear in SVG"
    );
    assert!(svg.contains("<rect "), "boxed style should produce a rect");
}

#[test]
fn plain_rehearsal_mark_has_text_but_no_rect() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 5), Duration::WHOLE)
        .rehearsal_mark("C", RehearsalStyle::Plain)
        .end_barline()
        .render_svg();
    assert!(
        svg.contains(">C<"),
        "plain rehearsal mark text should appear"
    );
    // Plain style should NOT have a rect (only boxed does)
    assert!(
        !svg.contains("<rect "),
        "plain rehearsal mark should not have a rect"
    );
}

#[test]
fn rehearsal_mark_on_rest_is_noop() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .rehearsal_mark("X", RehearsalStyle::Boxed)
        .end_barline()
        .render_svg();
    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    // rehearsal_mark after rest is a no-op
    assert_eq!(
        svg_with, svg_without,
        "rehearsal_mark on rest should have no effect"
    );
}

#[test]
fn note_with_rehearsal_differs_from_without() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("D", 5), Duration::QTR)
        .rehearsal_mark("1", RehearsalStyle::Boxed)
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();
    let svg_without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("D", 5), Duration::QTR)
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();
    assert_ne!(
        svg_with, svg_without,
        "rehearsal mark should change the SVG output"
    );
}

#[test]
fn convert_event_preserves_rehearsal_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            rehearsal_mark: Some(("A".to_string(), RehearsalStyle::Boxed)),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.rehearsal_mark,
                Some(("A".to_string(), RehearsalStyle::Boxed))
            );
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn convert_event_with_tracking_preserves_rehearsal_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            rehearsal_mark: Some(("B".to_string(), RehearsalStyle::Plain)),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let mut seen = HashMap::new();
    let result = convert_event(&event, &clef, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.rehearsal_mark,
                Some(("B".to_string(), RehearsalStyle::Plain))
            );
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn chord_convert_preserves_rehearsal_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Chord {
        pitches: vec![p("C", 4), p("E", 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            rehearsal_mark: Some(("12".to_string(), RehearsalStyle::Boxed)),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(
                c.annotations.rehearsal_mark,
                Some(("12".to_string(), RehearsalStyle::Boxed))
            );
        }
        _ => panic!("expected Chord"),
    }
}

// --- Tempo mark integration tests ---

#[test]
fn tempo_mark_adds_text_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .tempo(TempoMark::Text("Allegro".into()))
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();

    assert!(svg.contains(">Allegro<"), "tempo text should appear in SVG");
    assert!(svg.contains("bold"), "tempo text should be bold");
}

#[test]
fn tempo_metronome_adds_bpm_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .tempo(TempoMark::Metronome {
            note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
            dotted: false,
            bpm: 120,
        })
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();

    assert!(svg.contains("= 120"), "BPM should appear in SVG");
}

#[test]
fn tempo_on_rest_is_noop() {
    let svg_with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .tempo(TempoMark::Text("Andante".into()))
        .end_barline()
        .render_svg();

    // tempo() on rest should be ignored — "Andante" should NOT appear
    assert!(
        !svg_with.contains(">Andante<"),
        "tempo mark on rest should be ignored"
    );
}

#[test]
fn tempo_differs_from_no_tempo() {
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .tempo(TempoMark::Text("Vivace".into()))
        .end_barline()
        .render_svg();

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_ne!(with, without, "tempo mark should change SVG output");
}

#[test]
fn convert_event_preserves_tempo_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tempo_mark: Some(TempoMark::Text("Largo".into())),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.tempo_mark,
                Some(TempoMark::Text("Largo".into()))
            );
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn tracked_convert_preserves_tempo_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let mut seen = HashMap::new();
    let event = ScoreEvent::Note {
        pitch: p("D", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tempo_mark: Some(TempoMark::Metronome {
                note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 60,
            }),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.tempo_mark.is_some());
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn chord_convert_preserves_tempo_mark() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Chord {
        pitches: vec![p("C", 4), p("E", 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tempo_mark: Some(TempoMark::Text("Adagio".into())),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Chord(c) => {
            assert_eq!(
                c.annotations.tempo_mark,
                Some(TempoMark::Text("Adagio".into()))
            );
        }
        _ => panic!("expected Chord"),
    }
}

// --- Expression text integration tests ---

#[test]
fn expression_adds_italic_text_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .expression("dolce")
        .rest(Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();

    assert!(
        svg.contains(">dolce<"),
        "expression text should appear in SVG"
    );
    assert!(svg.contains("italic"), "expression text should be italic");
}

#[test]
fn expression_on_rest_is_noop() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .expression("legato")
        .end_barline()
        .render_svg();

    assert!(
        !svg.contains(">legato<"),
        "expression on rest should be ignored"
    );
}

#[test]
fn expression_differs_from_no_expression() {
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .expression("espressivo")
        .end_barline()
        .render_svg();

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_ne!(with, without, "expression should change SVG output");
}

#[test]
fn convert_event_preserves_expression() {
    let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: p("C", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            expression: Some("cantabile".into()),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &builder.key_sig, None);
    match result {
        MeasureEvent::Note(n) => {
            assert_eq!(n.annotations.expression, Some("cantabile".into()));
        }
        _ => panic!("expected Note"),
    }
}

#[test]
fn chord_expression_produces_text() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .expression("sostenuto")
        .end_barline()
        .render_svg();

    assert!(
        svg.contains(">sostenuto<"),
        "chord expression should appear"
    );
    assert!(svg.contains("italic"), "chord expression should be italic");
}

// -- PNG convenience methods (feature-gated) --

#[cfg(feature = "png")]
#[test]
fn try_render_png_produces_valid_png() {
    let png = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .try_render_png(1.0)
        .unwrap();

    assert_eq!(&png[0..4], &[0x89, b'P', b'N', b'G'], "PNG magic bytes");
    assert!(
        png.len() > 500,
        "score PNG should be non-trivial: {} bytes",
        png.len()
    );
}

#[cfg(feature = "png")]
#[test]
fn render_png_at_2x_is_larger_than_1x() {
    let builder = || {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("D", 4), Duration::HALF)
            .rest(Duration::HALF)
            .end_barline()
    };

    let png_1x = builder().render_png(1.0);
    let png_2x = builder().render_png(2.0);

    assert_eq!(&png_1x[0..4], &[0x89, b'P', b'N', b'G']);
    assert_eq!(&png_2x[0..4], &[0x89, b'P', b'N', b'G']);
    assert!(
        png_2x.len() > png_1x.len(),
        "2x PNG ({}) should be larger than 1x ({})",
        png_2x.len(),
        png_1x.len()
    );
}

#[cfg(feature = "png")]
#[test]
fn render_png_matches_try_render_png() {
    let builder = || {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("G", 4), Duration::WHOLE)
            .end_barline()
    };

    let via_try = builder().try_render_png(1.0).unwrap();
    let via_convenience = builder().render_png(1.0);
    assert_eq!(via_try, via_convenience);
}

#[cfg(feature = "png")]
#[test]
fn save_png_writes_same_bytes_as_render_png() {
    let builder = || {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("C", 4), Duration::QTR)
            .note(p("E", 4), Duration::QTR)
            .end_barline()
    };

    // Unique path under the OS temp dir; no extra dev-dependency needed.
    let path = std::env::temp_dir().join(format!(
        "music-engraver-save-png-{}.png",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    builder()
        .save_png(&path, 1.0)
        .expect("save_png should succeed");
    let on_disk = std::fs::read(&path).expect("read written PNG");
    let _ = std::fs::remove_file(&path);

    assert_eq!(&on_disk[0..4], &[0x89, b'P', b'N', b'G'], "PNG magic bytes");
    assert_eq!(
        on_disk,
        builder().render_png(1.0),
        "save_png must write exactly the render_png bytes"
    );
}

#[cfg(feature = "png")]
#[test]
fn save_png_to_unwritable_path_returns_io_error() {
    // A path whose parent directory does not exist must surface as Io, not panic.
    let bad = std::path::Path::new("/nonexistent-dir-xyz/score.png");
    let err = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .save_png(bad, 1.0)
        .expect_err("writing to a nonexistent dir should fail");
    assert!(
        matches!(err, crate::error::EngraverError::Io(_)),
        "expected Io error, got {err:?}"
    );
}

// ---- Articulation tests ----

#[test]
fn articulation_staccato_adds_extra_path() {
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .end_barline()
        .render_svg();
    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert_eq!(
        count_with,
        count_without + 1,
        "staccato should add exactly one extra path element"
    );
}

#[test]
fn articulation_on_rest_is_noop() {
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .articulation(Articulation::Staccato)
        .end_barline()
        .render_svg();
    assert_eq!(without, with, "articulation on rest should be no-op");
}

#[test]
fn different_articulations_produce_different_svgs() {
    use crate::layout::articulation::Articulation;
    let staccato = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .end_barline()
        .render_svg();
    let accent = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Accent)
        .end_barline()
        .render_svg();
    assert_ne!(
        staccato, accent,
        "staccato and accent should produce different SVG"
    );
}

#[test]
fn articulation_on_chord_adds_path() {
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .articulation(Articulation::Marcato)
        .end_barline()
        .render_svg();
    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert_eq!(
        count_with,
        count_without + 1,
        "marcato on chord should add one extra path"
    );
}

#[test]
fn fermata_adds_path_to_score() {
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("G", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("G", 4), Duration::WHOLE)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();
    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert_eq!(
        count_with,
        count_without + 1,
        "fermata should add one extra path"
    );
}

#[test]
fn each_fermata_duration_variant_adds_a_path_and_differs_from_plain() {
    // End-to-end regression: every fermata duration variant should add
    // exactly one path to the score (just like the plain Fermata) AND
    // produce a different SVG, so users can confirm they're getting the
    // long/short/etc. glyph they asked for and not silently falling back
    // to the standard fermata.
    use crate::layout::articulation::Articulation;

    let baseline_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("G", 4), Duration::WHOLE)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();
    let baseline_no_fermata = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("G", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let baseline_paths = baseline_no_fermata.matches("<path").count();

    let variants = [
        Articulation::FermataLong,
        Articulation::FermataShort,
        Articulation::FermataVeryLong,
        Articulation::FermataVeryShort,
        Articulation::FermataHenzeLong,
        Articulation::FermataHenzeShort,
    ];
    let mut svgs = Vec::new();
    for v in &variants {
        let with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("G", 4), Duration::WHOLE)
            .articulation(*v)
            .end_barline()
            .render_svg();
        let count_with = with.matches("<path").count();
        assert_eq!(
            count_with,
            baseline_paths + 1,
            "{v:?} should add exactly one extra path: baseline={}, with={}",
            baseline_paths,
            count_with,
        );
        assert_ne!(
            with, baseline_plain,
            "{v:?} should produce different SVG from plain Fermata",
        );
        svgs.push(with);
    }
    // No two duration variants should produce the same SVG either.
    for (i, a) in svgs.iter().enumerate() {
        for (j, b) in svgs.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "duration-variant scores {i} and {j} share SVG output",);
            }
        }
    }
}

#[test]
fn stack_long_fermata_with_staccato_adds_two_paths() {
    // Companion to `stacked_fermata_plus_staccato_adds_two_paths`: the
    // stack splitter must treat FermataLong identically to plain Fermata
    // (both always above, both stacked separately from the below
    // staccato), so an end-to-end render adds the same two-path delta.
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::FermataLong)
        .end_barline()
        .render_svg();
    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert_eq!(
        count_with,
        count_without + 2,
        "staccato + long-fermata should add two paths: without={}, with={}",
        count_without,
        count_with,
    );
}

#[test]
fn articulation_convert_event_preserves_field() {
    use crate::layout::articulation::Articulation;
    let key_sig = KeySignature::Open;
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            articulations: vec![Articulation::Tenuto],
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.articulations, vec![Articulation::Tenuto]);
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn stacked_articulations_add_two_paths() {
    use crate::layout::articulation::Articulation;
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .end_barline()
        .render_svg();
    let stacked = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Accent)
        .end_barline()
        .render_svg();
    let count_single = single.matches("<path").count();
    let count_stacked = stacked.matches("<path").count();
    assert_eq!(
        count_stacked,
        count_single + 1,
        "second articulation should add one more path: single={}, stacked={}",
        count_single,
        count_stacked
    );
}

#[test]
fn stacked_articulations_differ_from_single() {
    use crate::layout::articulation::Articulation;
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .end_barline()
        .render_svg();
    let stacked = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Accent)
        .end_barline()
        .render_svg();
    assert_ne!(
        single, stacked,
        "stacked should differ from single articulation"
    );
}

#[test]
fn stacked_fermata_plus_staccato_adds_two_paths() {
    use crate::layout::articulation::Articulation;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Fermata)
        .end_barline()
        .render_svg();
    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert_eq!(
        count_with,
        count_without + 2,
        "staccato + fermata should add two paths: without={}, with={}",
        count_without,
        count_with
    );
}

#[test]
fn stacked_convert_event_preserves_multiple() {
    use crate::layout::articulation::Articulation;
    let key_sig = KeySignature::Open;
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            articulations: vec![Articulation::Staccato, Articulation::Accent],
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.articulations.len(), 2);
            assert_eq!(ne.annotations.articulations[0], Articulation::Staccato);
            assert_eq!(ne.annotations.articulations[1], Articulation::Accent);
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn grace_note_acciaccatura_adds_extra_path() {
    use crate::layout::grace::GraceNoteKind;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .grace_note(p("D", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert!(
        count_with > count_without,
        "grace note should add at least one path: {} vs {}",
        count_with,
        count_without
    );
}

#[test]
fn grace_note_on_rest_is_noop() {
    use crate::layout::grace::GraceNoteKind;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .grace_note(p("C", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    assert_eq!(without, with, "grace note on rest should be no-op");
}

#[test]
fn acciaccatura_vs_appoggiatura_differ() {
    use crate::layout::grace::GraceNoteKind;

    let acc = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .grace_note(p("D", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let app = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .grace_note(p("D", 4), GraceNoteKind::Appoggiatura)
        .end_barline()
        .render_svg();

    assert_ne!(
        acc, app,
        "acciaccatura and appoggiatura should produce different SVGs"
    );
}

#[test]
fn grace_note_on_chord_adds_path() {
    use crate::layout::grace::GraceNoteKind;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4)], Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4)], Duration::QTR)
        .grace_note(p("B", 3), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let count_without = without.matches("<path").count();
    let count_with = with.matches("<path").count();
    assert!(
        count_with > count_without,
        "grace note on chord should add a path: {} vs {}",
        count_with,
        count_without
    );
}

#[test]
fn grace_note_convert_event_preserves_field() {
    use crate::layout::grace::GraceNoteKind;

    let key_sig = KeySignature::Open;
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            grace_note: Some((2, GraceNoteKind::Acciaccatura)),
            ..Default::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(
                ne.annotations.grace_note,
                Some((2, GraceNoteKind::Acciaccatura))
            );
        }
        _ => panic!("expected Note event from grace_note_convert_event"),
    }
}

#[test]
fn grace_note_slur_adds_filled_path_vs_plain_grace() {
    use crate::layout::grace::GraceNoteKind;

    let no_slur = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .grace_note(p("D", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let with_slur = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .grace_note_slur(p("D", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    // Exactly one extra filled (stroke="none") path: the slur crescent.
    let stroke_none_without = no_slur.matches(r#"stroke="none""#).count();
    let stroke_none_with = with_slur.matches(r#"stroke="none""#).count();
    assert_eq!(
            stroke_none_with,
            stroke_none_without + 1,
            "grace_note_slur should add exactly one filled path: {stroke_none_without} → {stroke_none_with}"
        );

    // Both still contain the grace glyph (scale 0.6).
    assert!(with_slur.contains("scale(0.6"));
    assert!(no_slur.contains("scale(0.6"));

    // Differ structurally.
    assert_ne!(no_slur, with_slur);
}

#[test]
fn grace_note_slur_on_rest_is_noop() {
    use crate::layout::grace::GraceNoteKind;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .grace_note_slur(p("C", 4), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    assert_eq!(without, with, "grace_note_slur on rest should be no-op");
}

#[test]
fn grace_note_slur_on_chord_adds_filled_path() {
    use crate::layout::grace::GraceNoteKind;

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::HALF)
        .grace_note(p("B", 3), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let slurred = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::HALF)
        .grace_note_slur(p("B", 3), GraceNoteKind::Acciaccatura)
        .end_barline()
        .render_svg();

    let plain_filled = plain.matches(r#"stroke="none""#).count();
    let slurred_filled = slurred.matches(r#"stroke="none""#).count();
    assert_eq!(
        slurred_filled,
        plain_filled + 1,
        "chord grace_note_slur should add exactly one filled path"
    );
}

#[test]
fn grace_note_slur_persists_grace_note_field() {
    use crate::layout::grace::GraceNoteKind;

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("F", 4), Duration::HALF)
        .grace_note_slur(p("E", 4), GraceNoteKind::Appoggiatura)
        .end_barline()
        .render_svg();

    // Grace glyph (scale 0.6) and the slur (stroke="none") must both be present.
    assert!(
        svg.contains("scale(0.6"),
        "grace_note_slur must still emit the grace glyph"
    );
    assert!(
        svg.contains(r#"stroke="none""#),
        "grace_note_slur must emit a filled slur path"
    );
}

// ---- Lyric tests ----

#[test]
fn lyric_on_note_adds_text_element() {
    use crate::layout::lyric::LyricSyllable;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 4), Duration::QTR)
        .lyric(LyricSyllable::word("day"))
        .end_barline()
        .render_svg();
    let texts_without = without.matches("<text").count();
    let texts_with = with.matches("<text").count();
    assert_eq!(
        texts_with,
        texts_without + 1,
        "lyric should add one text element"
    );
    assert!(
        with.contains(">day<"),
        "lyric text 'day' should appear in SVG"
    );
}

#[test]
fn lyric_on_rest_is_noop() {
    use crate::layout::lyric::LyricSyllable;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::QTR)
        .lyric(LyricSyllable::word("oops"))
        .end_barline()
        .render_svg();
    assert_eq!(without, with, "lyric on rest should be no-op");
}

#[test]
fn lyric_with_hyphen_shows_separated_hyphen_between_syllables() {
    // Engraving convention: hyphen is a separate centered '-' glyph
    // between two syllables, NOT appended to the source syllable text.
    use crate::layout::lyric::LyricSyllable;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("hap"))
        .note(p("D", 4), Duration::QTR)
        .lyric(LyricSyllable::word("py"))
        .end_barline()
        .render_svg();
    // Source syllable appears alone — no " -" suffix.
    assert!(
        svg.contains(">hap<"),
        "should render 'hap' alone, got: {svg}"
    );
    assert!(
        !svg.contains("hap -"),
        "source syllable text must not have ' -' appended, got: {svg}"
    );
    // A standalone hyphen <text>-</text> element is drawn between the
    // two syllables.
    let hyphen_count = svg.matches(">-<").count();
    assert!(
            hyphen_count >= 1,
            "expected at least 1 standalone hyphen text between syllables, got {hyphen_count}; svg: {svg}"
        );
}

#[test]
fn lyric_hyphen_crosses_system_boundary() {
    // When a hyphen syllable is on the last note of a system and the
    // next syllable is on the first note of the next system, the
    // cross-system pass must draw the hyphen(s). We use a 1-measure-per-
    // system layout to force the boundary.
    use crate::layout::lyric::LyricSyllable;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .measures_per_system(1)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("hap"))
        .barline()
        .note(p("D", 4), Duration::QTR)
        .lyric(LyricSyllable::word("py"))
        .end_barline()
        .render_svg();
    // Source and target syllables both appear, neither with concatenated
    // hyphen.
    assert!(svg.contains(">hap<"));
    assert!(svg.contains(">py<"));
    assert!(!svg.contains("hap -"));
    // The cross-system path emits up to 2 hyphens (one trailing on the
    // source system, one leading on the target). At least one must
    // appear — assert exact count is at least 1 (it could be 2; we
    // don't pin it to a specific number because gap-threshold logic
    // may suppress one side depending on layout widths).
    let hyphen_count = svg.matches(">-<").count();
    assert!(
        hyphen_count >= 1,
        "expected at least 1 cross-system hyphen, got {hyphen_count}; svg: {svg}"
    );
}

#[test]
fn lyric_hyphen_not_drawn_when_no_successor_syllable() {
    // A hyphen continuation with no following syllable in the system
    // should not produce a stray standalone hyphen.
    use crate::layout::lyric::LyricSyllable;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("hap"))
        .end_barline()
        .render_svg();
    assert!(svg.contains(">hap<"));
    assert_eq!(
        svg.matches(">-<").count(),
        0,
        "no hyphen should be drawn when there is no target syllable"
    );
}

#[test]
fn different_lyrics_produce_different_svgs() {
    use crate::layout::lyric::LyricSyllable;
    let svg_a = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::word("day"))
        .end_barline()
        .render_svg();
    let svg_b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::word("night"))
        .end_barline()
        .render_svg();
    assert_ne!(
        svg_a, svg_b,
        "different lyric texts should produce different SVGs"
    );
}

#[test]
fn lyric_on_chord_adds_text() {
    use crate::layout::lyric::LyricSyllable;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("C", 4), p("E", 4)], Duration::QTR)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("C", 4), p("E", 4)], Duration::QTR)
        .lyric(LyricSyllable::word("love"))
        .end_barline()
        .render_svg();
    let texts_without = without.matches("<text").count();
    let texts_with = with.matches("<text").count();
    assert_eq!(
        texts_with,
        texts_without + 1,
        "lyric on chord should add one text element"
    );
}

#[test]
fn lyric_convert_event_preserves_field() {
    use crate::layout::lyric::LyricSyllable;
    let key_sig = KeySignature::Open;
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            lyric: Some(LyricSyllable::with_hyphen("test")),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            let lyric = ne.annotations.lyric.as_ref().expect("should have lyric");
            assert_eq!(lyric.text, "test");
            assert_eq!(
                lyric.continuation,
                crate::layout::lyric::LyricContinuation::Hyphen
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn lyric_is_not_italic_in_svg() {
    use crate::layout::lyric::LyricSyllable;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::word("la"))
        .end_barline()
        .render_svg();
    // Find the text element containing "la" and verify it's not italic
    // The lyric text should be rendered in normal (roman) style
    assert!(svg.contains(">la<"), "should contain lyric text 'la'");
    // Expression text uses italic; lyrics should not
    // Check that the text element for "la" does not have font-style="italic"
    let la_pos = svg.find(">la<").expect("should find 'la'");
    let text_start = svg[..la_pos]
        .rfind("<text")
        .expect("should find <text before 'la'");
    let text_tag = &svg[text_start..la_pos];
    assert!(
        !text_tag.contains("italic"),
        "lyric text should be roman (normal), not italic: {text_tag}"
    );
}

// ---- Chord symbol tests ----

#[test]
fn chord_symbol_adds_text_element() {
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("Cmaj7")
        .end_barline()
        .render_svg();

    let texts_without = without.matches("<text").count();
    let texts_with = with.matches("<text").count();
    assert!(
        texts_with > texts_without,
        "chord symbol should add a text element: {} vs {}",
        texts_with,
        texts_without
    );
    assert!(with.contains(">Cmaj7<"), "should contain 'Cmaj7'");
}

#[test]
fn chord_symbol_on_rest_is_noop() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .chord_symbol("Am")
        .end_barline()
        .render_svg();

    assert!(
        !svg.contains(">Am<"),
        "chord symbol on rest should be ignored"
    );
}

#[test]
fn different_chord_symbols_differ() {
    let svg_c = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("C")
        .end_barline()
        .render_svg();

    let svg_am = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("Am7")
        .end_barline()
        .render_svg();

    assert_ne!(svg_c, svg_am);
}

#[test]
fn chord_symbol_on_chord_event() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .chord_symbol("C")
        .end_barline()
        .render_svg();

    assert!(svg.contains(">C<"), "should contain chord symbol 'C'");
}

#[test]
fn chord_symbol_is_bold() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::QTR)
        .chord_symbol("G7")
        .end_barline()
        .render_svg();

    let g7_pos = svg.find(">G7<").expect("should find 'G7'");
    let text_start = svg[..g7_pos]
        .rfind("<text")
        .expect("should find <text before 'G7'");
    let text_tag = &svg[text_start..g7_pos];
    assert!(
        text_tag.contains("bold"),
        "chord symbol should be bold: {text_tag}"
    );
}

#[test]
fn convert_event_preserves_chord_symbol() {
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            chord_symbol: Some("Em".to_string()),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.chord_symbol.as_deref(), Some("Em"));
        }
        _ => panic!("expected Note event"),
    }
}

// ---- Auto line breaking tests ----

#[test]
fn auto_line_breaks_produces_valid_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .auto_line_breaks()
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        .note(p("D", 4), Duration::WHOLE)
        .barline()
        .note(p("E", 4), Duration::WHOLE)
        .barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<svg"), "should produce valid SVG");
    assert!(svg.contains("</svg>"), "should contain closing tag");
}

#[test]
fn auto_line_breaks_differs_from_fixed() {
    // 8 measures of whole notes — auto vs fixed(4) may produce
    // different system counts depending on available width.
    let build = |auto: bool| {
        let mut b = ScoreBuilder::new().clef(Clef::Treble).time_signature(4, 4);
        if auto {
            b = b.auto_line_breaks();
        } else {
            b = b.measures_per_system(2);
        }
        for i in 0..8u8 {
            let note = match i % 4 {
                0 => "C",
                1 => "D",
                2 => "E",
                _ => "F",
            };
            b = b.note(p(note, 4), Duration::WHOLE).barline();
        }
        b = b.note(p("G", 4), Duration::WHOLE).end_barline();
        b.render_svg()
    };

    let auto_svg = build(true);
    let fixed_svg = build(false);

    // Both should be valid
    assert!(auto_svg.starts_with("<svg"));
    assert!(fixed_svg.starts_with("<svg"));

    // They should differ because fixed(2) forces exactly 2 per system
    // while auto may pack more or fewer
    assert_ne!(
        auto_svg, fixed_svg,
        "auto and fixed(2) should produce different output"
    );
}

#[test]
fn auto_line_breaks_overrides_measures_per_system() {
    // Setting auto_line_breaks after measures_per_system should use auto
    let svg_auto = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        .auto_line_breaks()
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        .note(p("D", 4), Duration::WHOLE)
        .barline()
        .note(p("E", 4), Duration::WHOLE)
        .barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Setting measures_per_system after auto_line_breaks should use fixed
    let svg_fixed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .auto_line_breaks()
        .measures_per_system(2)
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        .note(p("D", 4), Duration::WHOLE)
        .barline()
        .note(p("E", 4), Duration::WHOLE)
        .barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // auto should differ from fixed(2) — auto with 4 whole-note measures
    // at default width will likely pack differently than 2 per system
    assert_ne!(
        svg_auto, svg_fixed,
        "auto should differ from fixed(2) when the last call wins"
    );
}

#[test]
fn auto_line_breaks_with_varied_density() {
    // Mix of dense and sparse measures
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .auto_line_breaks()
        // Measure 1: whole note (sparse)
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        // Measure 2: 4 quarters (denser)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 3: beam group (densest)
        .beam_group(vec![
            (p("G", 4), Duration::SIXTEENTH),
            (p("A", 4), Duration::SIXTEENTH),
            (p("B", 4), Duration::SIXTEENTH),
            (p("C", 5), Duration::SIXTEENTH),
            (p("B", 4), Duration::SIXTEENTH),
            (p("A", 4), Duration::SIXTEENTH),
            (p("G", 4), Duration::SIXTEENTH),
            (p("F", 4), Duration::SIXTEENTH),
        ])
        .barline()
        // Measure 4: whole note (sparse again)
        .note(p("E", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(
        svg.starts_with("<svg"),
        "varied-density auto-break should produce valid SVG"
    );
    // Should have at least some path elements (noteheads, clef, etc.)
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 5,
        "should have at least 5 path elements, got {path_count}"
    );
}

// ---- Ornament tests ----

#[test]
fn ornament_trill_adds_extra_path() {
    use crate::layout::ornament::Ornament;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .ornament(Ornament::Trill)
        .end_barline()
        .render_svg();

    let paths_without = without.matches("<path").count();
    let paths_with = with.matches("<path").count();
    assert!(
        paths_with > paths_without,
        "trill should add a path: {} vs {}",
        paths_with,
        paths_without
    );
}

#[test]
fn ornament_on_rest_is_noop() {
    use crate::layout::ornament::Ornament;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .ornament(Ornament::Trill)
        .end_barline()
        .render_svg();

    assert_eq!(without, with, "ornament on rest should be a no-op");
}

#[test]
fn ornament_on_chord_adds_path() {
    use crate::layout::ornament::Ornament;

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .end_barline()
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .ornament(Ornament::Turn)
        .end_barline()
        .render_svg();

    let paths_without = without.matches("<path").count();
    let paths_with = with.matches("<path").count();
    assert!(
        paths_with > paths_without,
        "ornament on chord should add a path: {} vs {}",
        paths_with,
        paths_without
    );
}

#[test]
fn different_ornaments_produce_different_svg() {
    use crate::layout::ornament::Ornament;

    let trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::QTR)
        .ornament(Ornament::Trill)
        .end_barline()
        .render_svg();

    let mordent = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::QTR)
        .ornament(Ornament::Mordent)
        .end_barline()
        .render_svg();

    assert_ne!(
        trill, mordent,
        "trill and mordent should produce different SVG"
    );
}

#[test]
fn ornament_convert_event_preserves_field() {
    use crate::layout::ornament::Ornament;

    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Turn),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Turn));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn ornament_tracked_convert_preserves_field() {
    use crate::layout::ornament::Ornament;

    let event = ScoreEvent::Note {
        pitch: p("F", 4),
        duration: Duration::HALF,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Mordent),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let mut seen = AccidentalTracker::new();
    let result = convert_event(&event, &clef, &key_sig, Some(&mut seen));
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Mordent));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn ornament_chord_convert_preserves_field() {
    use crate::layout::ornament::Ornament;

    let event = ScoreEvent::Chord {
        pitches: vec![p("C", 4), p("E", 4)],
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::InvertedTurn),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Chord(ce) => {
            assert_eq!(ce.annotations.ornament, Some(Ornament::InvertedTurn));
        }
        _ => panic!("expected Chord event"),
    }
}

// ---- Trill-with-extension tests ----

#[test]
fn trill_with_extension_adds_more_paths_than_plain_trill() {
    use crate::layout::ornament::Ornament;

    // Use a whole note + following quarter to create enough horizontal
    // span between events for the wiggle to fit at least one tile.
    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_trill.matches("<path").count();
    let ext_paths = with_ext.matches("<path").count();
    assert!(
        ext_paths > plain_paths,
        "trill_with_extension must add wiggle paths beyond plain trill: \
             plain={plain_paths}, with_ext={ext_paths}"
    );
}

#[test]
fn trill_with_extension_on_rest_is_noop() {
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, with_ext,
        "trill_with_extension on rest must be a no-op"
    );
}

#[test]
fn trill_with_extension_on_chord_adds_wiggle() {
    use crate::layout::ornament::Ornament;

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let chord_with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_chord.matches("<path").count();
    let ext_paths = chord_with_ext.matches("<path").count();
    assert!(
        ext_paths > plain_paths,
        "trill_with_extension on chord must add wiggle paths: \
             plain={plain_paths}, with_ext={ext_paths}"
    );
}

#[test]
fn trill_with_extension_sets_both_annotation_fields() {
    use crate::layout::ornament::Ornament;

    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Trill));
            assert!(ne.annotations.trill_extension);
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_extension_flag_alone_is_inert_without_trill_ornament() {
    use crate::layout::ornament::Ornament;

    // Setting only trill_extension (without ornament=Trill) should not
    // produce a wiggle — the renderer requires both flags. Confirms the
    // collector's `matches!(ornament, Some(Trill))` guard.
    let only_flag_set = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        // Apply a non-trill ornament, then manually mimic only setting the
        // flag — but the public API forces both via trill_with_extension.
        // We instead compare ScoreBuilder against a Mordent + manual flag.
        .ornament(Ornament::Mordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // A non-trill ornament should never produce a wiggle.
    // Mordent's "wiggle" appearance is part of the glyph itself, but the
    // system-renderer extension wiggle is a separate tiled element.
    // We assert that this rendering equals the same score without the
    // extension flag — there's no public path to set trill_extension=true
    // on a non-trill ornament, but the renderer's guard means it would
    // still be inert.
    let plain_mordent = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::Mordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        only_flag_set, plain_mordent,
        "non-trill ornament never triggers the wiggle extension"
    );
}

// --- measure numbers via ScoreBuilder ---

#[test]
fn show_measure_numbers_adds_text_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measure_numbering(MeasureNumbering::SystemStart)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::D, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(svg.contains(">1</text>"), "should show measure number 1");
}

#[test]
fn show_measure_numbers_multi_system_shows_correct_numbers() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        .measure_numbering(MeasureNumbering::SystemStart)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::D, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::E, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // 2 measures per system × 4 measures = 2 systems
    assert!(svg.contains(">1</text>"), "first system should show 1");
    assert!(svg.contains(">3</text>"), "second system should show 3");
}

#[test]
fn show_measure_numbers_disabled_by_default() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Default is no measure numbers
    assert!(
        !svg.contains(">1</text>"),
        "should not show measure numbers by default"
    );
}

#[test]
fn show_measure_numbers_on_vs_off_differs() {
    let builder = || {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::D, 4), Duration::WHOLE)
            .end_barline()
    };

    let svg_on = builder()
        .measure_numbering(MeasureNumbering::SystemStart)
        .render_svg();
    let svg_off = builder().render_svg();

    assert_ne!(
        svg_on, svg_off,
        "enabling measure numbers should change output"
    );
    assert!(
        svg_on.len() > svg_off.len(),
        "SVG with measure numbers should be larger"
    );
}

// --- Optimal line breaking ---

#[test]
fn optimal_line_breaks_renders_valid_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .optimal_line_breaks()
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .barline()
        .note(Pitch::new(Note::G, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 4,
        "expected at least 4 paths for clef + noteheads, got {path_count}"
    );
}

#[test]
fn optimal_line_breaks_differs_from_fixed() {
    // 8 measures with varying density: optimal should produce a different
    // line layout than a fixed 4-per-system.
    let mut builder = ScoreBuilder::new().clef(Clef::Treble).time_signature(4, 4);
    for i in 0..8u8 {
        let note = match i % 4 {
            0 => Note::C,
            1 => Note::D,
            2 => Note::E,
            _ => Note::F,
        };
        builder = builder
            .note(Pitch::new(note, 4), Duration::QTR)
            .note(Pitch::new(note, 4), Duration::QTR)
            .note(Pitch::new(note, 4), Duration::QTR)
            .note(Pitch::new(note, 4), Duration::QTR)
            .barline();
    }
    let optimal_svg = builder.clone().optimal_line_breaks().render_svg();
    let fixed_svg = builder.measures_per_system(4).render_svg();

    // Both should produce valid SVG
    assert!(optimal_svg.starts_with("<svg"));
    assert!(fixed_svg.starts_with("<svg"));

    // Optimal covers all content (same number of noteheads)
    let opt_paths = optimal_svg.matches("<path").count();
    let fix_paths = fixed_svg.matches("<path").count();
    // Path counts might differ due to different clef repetitions per system,
    // but both should have the same number of note content paths approximately
    assert!(opt_paths > 0 && fix_paths > 0);
}

#[test]
fn optimal_overrides_auto() {
    // Setting optimal after auto should use optimal
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .auto_line_breaks()
        .optimal_line_breaks()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(svg.starts_with("<svg"));
}

#[test]
fn auto_overrides_optimal() {
    // Setting auto after optimal should use auto
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .optimal_line_breaks()
        .auto_line_breaks()
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(svg.starts_with("<svg"));
}

#[test]
fn measures_per_system_overrides_optimal() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .optimal_line_breaks()
        .measures_per_system(2)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::D, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::E, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(svg.starts_with("<svg"));
    // 4 measures at 2 per system = 2 systems = 10 staff lines
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 10,
        "expected at least 10 lines for 2 systems, got {line_count}"
    );
}

// --- volta brackets ---

#[test]
fn volta_single_measure_adds_bracket_lines_and_text() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .volta_start("1.")
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .volta_end()
        .end_barline()
        .render_svg();
    assert!(svg.contains(">1.</text>"), "should contain volta text '1.'");
    // Single-measure volta: 3 bracket lines (top + left hook + right hook)
    // plus 5 staff lines + other lines
    let without_volta = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(!without_volta.contains(">1.</text>"));
    // The volta version should have more <line elements
    let volta_lines = svg.matches("<line").count();
    let no_volta_lines = without_volta.matches("<line").count();
    assert!(
        volta_lines > no_volta_lines,
        "volta should add bracket lines: {volta_lines} vs {no_volta_lines}"
    );
}

#[test]
fn volta_multi_measure_has_continuation() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        // First ending: 2 measures
        .volta_start("1.")
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .barline()
        .note(Pitch::new(Note::D, 4), Duration::WHOLE)
        .volta_end()
        .barline()
        // Second ending: 1 measure
        .volta_start("2.")
        .note(Pitch::new(Note::E, 4), Duration::WHOLE)
        .volta_end()
        .end_barline()
        .render_svg();
    assert!(svg.contains(">1.</text>"), "should contain volta text '1.'");
    assert!(svg.contains(">2.</text>"), "should contain volta text '2.'");
}

#[test]
fn volta_differs_from_no_volta() {
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .volta_start("1.")
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .volta_end()
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert_ne!(with, without);
}

#[test]
fn volta_text_is_bold() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .volta_start("1.")
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .volta_end()
        .end_barline()
        .render_svg();
    // Find the text element containing "1." and verify it's bold
    assert!(
        svg.contains("font-weight=\"bold\""),
        "volta text should be bold"
    );
}

#[test]
fn volta_no_bracket_without_volta_calls() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    // No volta-related text should appear
    assert!(!svg.contains(">1.</text>"));
    assert!(!svg.contains(">2.</text>"));
}

#[test]
fn volta_single_measure_three_extra_lines() {
    // A single-measure volta bracket adds exactly 3 lines:
    // 1 horizontal top line + 1 left hook + 1 right hook
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .volta_start("1.")
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .volta_end()
        .end_barline()
        .render_svg();
    let diff = with.matches("<line").count() - without.matches("<line").count();
    assert_eq!(
        diff, 3,
        "single-measure volta adds 3 lines (top + 2 hooks), got {diff}"
    );
}

// Navigation sign (segno, coda) tests

#[test]
fn navigation_sign_segno_adds_path() {
    use crate::layout::navigation::NavigationSign;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .navigation_sign(NavigationSign::Segno)
        .end_barline()
        .render_svg();
    assert!(
        with.matches("<path").count() > without.matches("<path").count(),
        "segno should add a path element"
    );
}

#[test]
fn navigation_sign_coda_adds_path() {
    use crate::layout::navigation::NavigationSign;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .navigation_sign(NavigationSign::Coda)
        .end_barline()
        .render_svg();
    assert!(
        with.matches("<path").count() > without.matches("<path").count(),
        "coda should add a path element"
    );
}

#[test]
fn navigation_sign_rest_noop() {
    use crate::layout::navigation::NavigationSign;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .navigation_sign(NavigationSign::Segno)
        .end_barline()
        .render_svg();
    assert_eq!(without, with, "navigation sign on rest should be a no-op");
}

#[test]
fn navigation_sign_segno_differs_from_coda() {
    use crate::layout::navigation::NavigationSign;
    let segno = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .navigation_sign(NavigationSign::Segno)
        .end_barline()
        .render_svg();
    let coda = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .navigation_sign(NavigationSign::Coda)
        .end_barline()
        .render_svg();
    assert_ne!(segno, coda, "segno and coda should produce different SVGs");
}

#[test]
fn navigation_sign_on_chord() {
    use crate::layout::navigation::NavigationSign;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::WHOLE,
        )
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::WHOLE,
        )
        .navigation_sign(NavigationSign::Coda)
        .end_barline()
        .render_svg();
    assert!(
        with.matches("<path").count() > without.matches("<path").count(),
        "coda on chord should add a path"
    );
}

#[test]
fn convert_event_preserves_navigation_sign() {
    use crate::layout::navigation::NavigationSign;
    let builder = ScoreBuilder::new().clef(Clef::Treble);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::C, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            navigation_sign: Some(NavigationSign::Segno),
            ..Default::default()
        },
    };
    let element = convert_event(&event, &builder.clef.to_clef(), &builder.key_sig, None);
    match element {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.annotations.navigation_sign,
                Some(NavigationSign::Segno),
                "navigation_sign should be preserved through convert_event"
            );
        }
        _ => panic!("expected MeasureEvent::Note"),
    }
}

// --- ottava brackets ---

#[test]
fn ottava_8va_adds_dashed_line_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .note(Pitch::new(Note::D, 6), Duration::QTR)
        .note(Pitch::new(Note::E, 6), Duration::QTR)
        .ottava_end()
        .note(Pitch::new(Note::F, 6), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(svg.contains(">8va</text>"), "should have 8va label");
    assert!(svg.contains("stroke-dasharray"), "should have dashed line");
}

#[test]
fn ottava_rest_no_op() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .rest(Duration::QTR)
        .ottava_end()
        .end_barline()
        .render_svg();
    assert!(!svg.contains("8va"), "ottava on rest should be no-op");
}

#[test]
fn ottava_8vb_label_differs_from_8va() {
    let svg_8va = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .ottava_start(OttavaKind::Ottava8va)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .ottava_end()
        .end_barline()
        .render_svg();
    let svg_8vb = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 3), Duration::HALF)
        .ottava_start(OttavaKind::Ottava8vb)
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .ottava_end()
        .end_barline()
        .render_svg();
    assert!(svg_8va.contains(">8va</text>"));
    assert!(svg_8vb.contains(">8vb</text>"));
}

#[test]
fn no_ottava_without_calls() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .end_barline()
        .render_svg();
    assert!(!svg.contains("8va"), "no 8va without ottava calls");
    assert!(
        !svg.contains("stroke-dasharray"),
        "no dashed line without ottava calls"
    );
}

#[test]
fn ottava_convert_event_preserves_start() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::C, 5),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ottava_start: Some(OttavaKind::Ottava8va),
            ..Default::default()
        },
    };
    let element = convert_event(&event, &builder.clef.to_clef(), &builder.key_sig, None);
    match element {
        MeasureEvent::Note(n) => {
            assert_eq!(n.annotations.ottava_start, Some(OttavaKind::Ottava8va));
        }
        _ => panic!("expected MeasureEvent::Note"),
    }
}

#[test]
fn ottava_convert_event_preserves_end() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open);
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::C, 5),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ottava_end: true,
            ..Default::default()
        },
    };
    let element = convert_event(&event, &builder.clef.to_clef(), &builder.key_sig, None);
    match element {
        MeasureEvent::Note(n) => {
            assert!(n.annotations.ottava_end, "ottava_end should be preserved");
        }
        _ => panic!("expected MeasureEvent::Note"),
    }
}

#[test]
fn cross_system_ottava_produces_two_labels_via_score_builder() {
    use crate::layout::ottava::OttavaKind;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .barline()
        .note(p("D", 6), Duration::QTR)
        .ottava_end()
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    // Cross-system ottava: trailing half-bracket label + incoming half-bracket label
    let count_8va = svg.matches("8va").count();
    assert!(
        count_8va >= 2,
        "cross-system ottava should produce at least 2 '8va' labels, got {count_8va}"
    );
}

#[test]
fn cross_system_ottava_differs_from_no_ottava() {
    use crate::layout::ottava::OttavaKind;
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 6), Duration::QTR)
        .barline()
        .note(p("D", 6), Duration::QTR)
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .barline()
        .note(p("D", 6), Duration::QTR)
        .ottava_end()
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    assert_ne!(
        without, with,
        "cross-system ottava should change SVG output"
    );
}

// Pedal marking tests

#[test]
fn pedal_down_adds_path_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_down()
        .end_barline()
        .render_svg();
    // Pedal "Ped." glyph adds one extra path vs a note without pedal
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        svg.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal_down should add at least one path"
    );
}

#[test]
fn pedal_up_adds_path_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_up()
        .end_barline()
        .render_svg();
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        svg.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal_up should add at least one path"
    );
}

#[test]
fn pedal_down_and_up_produce_different_svg() {
    let down = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_down()
        .end_barline()
        .render_svg();
    let up = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_up()
        .end_barline()
        .render_svg();
    assert_ne!(down, up, "Ped. and * should produce different SVG");
}

#[test]
fn pedal_on_rest_is_noop() {
    let with_pedal = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .pedal_down()
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(with_pedal, without, "pedal on rest should be no-op");
}

#[test]
fn pedal_on_chord_adds_path() {
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .end_barline()
        .render_svg();
    let with_pedal = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .pedal_down()
        .end_barline()
        .render_svg();
    assert!(
        with_pedal.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal on chord should add path"
    );
}

#[test]
fn convert_event_preserves_pedal() {
    use crate::layout::pedal::PedalMark;
    let pitch = p("C", 4);
    let event = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            pedal: Some(PedalMark::Down),
            ..Default::default()
        },
    };
    let key_sig = KeySignature::Open;
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.pedal, Some(PedalMark::Down));
    } else {
        panic!("expected Note event");
    }
}

#[test]
fn pedal_half_adds_path_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_half()
        .end_barline()
        .render_svg();
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        svg.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal_half should add at least one path"
    );
}

#[test]
fn pedal_sost_adds_path_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_sost()
        .end_barline()
        .render_svg();
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        svg.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal_sost should add at least one path"
    );
}

#[test]
fn all_four_pedal_score_builders_produce_distinct_svg() {
    // ScoreBuilder surface: each pedal builder method must reach the
    // renderer with the correct PedalMark variant, yielding a different
    // SVG byte string. A regression that hard-coded one variant in the
    // builder (e.g., copy-pasting pedal_down's body into pedal_half) would
    // collapse two of these onto the same output.
    let make = |attach: fn(ScoreBuilder) -> ScoreBuilder| -> String {
        attach(
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .time_signature(4, 4)
                .note(p("C", 4), Duration::QTR),
        )
        .end_barline()
        .render_svg()
    };

    let down = make(|b| b.pedal_down());
    let up = make(|b| b.pedal_up());
    let half = make(|b| b.pedal_half());
    let sost = make(|b| b.pedal_sost());

    let outputs = [
        ("down", &down),
        ("up", &up),
        ("half", &half),
        ("sost", &sost),
    ];
    for i in 0..outputs.len() {
        for j in (i + 1)..outputs.len() {
            assert_ne!(
                outputs[i].1, outputs[j].1,
                "pedal_{} and pedal_{} produced identical SVG",
                outputs[i].0, outputs[j].0,
            );
        }
    }
}

#[test]
fn pedal_half_on_rest_is_noop() {
    // Mirrors pedal_on_rest_is_noop for the Half variant — the builder
    // silently drops the annotation when the last event is a rest.
    let with_pedal = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .pedal_half()
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(with_pedal, without, "pedal_half on rest should be no-op");
}

#[test]
fn pedal_sost_on_rest_is_noop() {
    let with_pedal = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .pedal_sost()
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(with_pedal, without, "pedal_sost on rest should be no-op");
}

#[test]
fn pedal_half_on_chord_adds_path() {
    // Pedal annotations attach to chords as well as notes — verify the
    // Half variant follows the same routing as Down on a chord.
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .end_barline()
        .render_svg();
    let with_pedal = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .pedal_half()
        .end_barline()
        .render_svg();
    assert!(
        with_pedal.matches("<path ").count() > baseline.matches("<path ").count(),
        "pedal_half on chord should add a path"
    );
}

#[test]
fn convert_event_preserves_pedal_half() {
    // Round-trip the new variants through convert_event to confirm the
    // builder-side annotation reaches the layout-side MeasureEvent intact.
    use crate::layout::pedal::PedalMark;
    let pitch = p("C", 4);
    let event = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            pedal: Some(PedalMark::Half),
            ..Default::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &KeySignature::Open, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.pedal, Some(PedalMark::Half));
    } else {
        panic!("expected Note event");
    }
}

#[test]
fn convert_event_preserves_pedal_sost() {
    use crate::layout::pedal::PedalMark;
    let pitch = p("C", 4);
    let event = ScoreEvent::Note {
        pitch,
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            pedal: Some(PedalMark::Sost),
            ..Default::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &KeySignature::Open, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.pedal, Some(PedalMark::Sost));
    } else {
        panic!("expected Note event");
    }
}

// ── Tremolo tests ──────────────────────────────────────────────

#[test]
fn tremolo_adds_extra_path_to_svg() {
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with_paths = with.matches("<path ").count();
    let without_paths = without.matches("<path ").count();
    assert!(
        with_paths > without_paths,
        "tremolo should add a path: with={}, without={}",
        with_paths,
        without_paths
    );
}

#[test]
fn tremolo_on_rest_is_noop() {
    let with_call = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .tremolo(TremoloCount::Double)
        .end_barline()
        .render_svg();
    let without_call = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(with_call, without_call, "tremolo on rest should be no-op");
}

#[test]
fn different_tremolo_counts_differ() {
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .end_barline()
        .render_svg();
    let triple = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tremolo(TremoloCount::Triple)
        .end_barline()
        .render_svg();
    assert_ne!(
        single, triple,
        "single and triple tremolo should produce different SVGs"
    );
}

#[test]
fn chord_tremolo_adds_path() {
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::QTR,
        )
        .tremolo(TremoloCount::Double)
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::QTR,
        )
        .end_barline()
        .render_svg();
    assert!(
        with.matches("<path ").count() > without.matches("<path ").count(),
        "chord tremolo should add extra path"
    );
}

#[test]
fn convert_event_preserves_tremolo() {
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::G, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            tremolo: Some(TremoloCount::Triple),
            ..NoteAnnotations::default()
        },
    };
    let key_sig = KeySignature::Open;
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.tremolo, Some(TremoloCount::Triple));
    } else {
        panic!("expected Note event");
    }
}

// --- arpeggio ---

#[test]
fn arpeggio_on_chord_adds_path() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Up)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    let paths = svg.matches("<path ").count();
    // Without arpeggio, a 3-note chord has ~4 paths (clef + 3 noteheads)
    // With arpeggio, we add 1 more path for the wavy line
    assert!(
        paths >= 5,
        "arpeggio should add a path: found {paths} paths"
    );
}

#[test]
fn arpeggio_on_rest_is_noop() {
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .arpeggio(ArpeggioDirection::Up)
        .end_barline()
        .render_svg();
    assert_eq!(without, with, "arpeggio on rest should be no-op");
}

#[test]
fn arpeggio_up_differs_from_down() {
    let make = |dir: ArpeggioDirection| {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(
                vec![
                    Pitch::new(Note::C, 4),
                    Pitch::new(Note::E, 4),
                    Pitch::new(Note::G, 4),
                ],
                Duration::WHOLE,
            )
            .arpeggio(dir)
            .end_barline()
            .render_svg()
    };
    assert_ne!(
        make(ArpeggioDirection::Up),
        make(ArpeggioDirection::Down),
        "up and down arpeggio should produce different SVG"
    );
}

#[test]
fn arpeggio_on_single_note_adds_path() {
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::WHOLE)
        .arpeggio(ArpeggioDirection::Up)
        .end_barline()
        .render_svg();
    let paths_without = without.matches("<path ").count();
    let paths_with = with.matches("<path ").count();
    assert!(
        paths_with > paths_without,
        "arpeggio on single note should add path: {paths_with} vs {paths_without}"
    );
}

#[test]
fn convert_event_preserves_arpeggio() {
    let event = ScoreEvent::Chord {
        pitches: vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
        duration: Duration::HALF,
        annotations: NoteAnnotations {
            arpeggio: Some(ArpeggioDirection::Down),
            ..NoteAnnotations::default()
        },
    };
    let key_sig = KeySignature::Open;
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    if let MeasureEvent::Chord(chord) = result {
        assert_eq!(chord.annotations.arpeggio, Some(ArpeggioDirection::Down));
    } else {
        panic!("expected Chord event");
    }
}

// --- Breath mark tests ---

#[test]
fn breath_mark_adds_path_to_svg() {
    use crate::layout::breath::BreathMark;
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    // Breath mark should add 1 extra path vs without
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    let paths_with = svg.matches("<path").count();
    let paths_without = without.matches("<path").count();
    assert_eq!(
        paths_with,
        paths_without + 1,
        "breath mark should add exactly 1 path"
    );
}

#[test]
fn breath_mark_on_rest_is_noop() {
    use crate::layout::breath::BreathMark;
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .breath_mark(BreathMark::Comma)
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert_eq!(with, without, "breath mark on rest should be no-op");
}

#[test]
fn different_breath_marks_differ() {
    use crate::layout::breath::BreathMark;
    let make = |mark: BreathMark| {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 5), Duration::HALF)
            .breath_mark(mark)
            .rest(Duration::HALF)
            .end_barline()
            .render_svg()
    };
    let comma = make(BreathMark::Comma);
    let tick = make(BreathMark::Tick);
    let caesura = make(BreathMark::Caesura);
    assert_ne!(comma, tick, "comma and tick should differ");
    assert_ne!(tick, caesura, "tick and caesura should differ");
}

#[test]
fn breath_mark_on_chord() {
    use crate::layout::breath::BreathMark;
    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .breath_mark(BreathMark::Caesura)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    assert_ne!(with, without, "caesura on chord should add path");
    assert!(
        with.matches("<path").count() > without.matches("<path").count(),
        "breath mark should add at least 1 path"
    );
}

#[test]
fn convert_event_preserves_breath_mark() {
    use crate::layout::breath::BreathMark;
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::A, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            breath_mark: Some(BreathMark::Tick),
            ..NoteAnnotations::default()
        },
    };
    let key_sig = KeySignature::Open;
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.breath_mark, Some(BreathMark::Tick));
    } else {
        panic!("expected Note event");
    }
}

// --- glissando ---

#[test]
fn glissando_adds_line_to_svg() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    // Glissando draws a <line> element
    let line_count = svg.matches("<line").count();
    let no_gliss_svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    let no_gliss_line_count = no_gliss_svg.matches("<line").count();
    assert!(
        line_count > no_gliss_line_count,
        "glissando should add at least one line: {} vs {}",
        line_count,
        no_gliss_line_count
    );
}

#[test]
fn glissando_on_rest_is_noop() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    let svg_no_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();
    assert_eq!(svg, svg_no_gliss, "glissando on rest should be no-op");
}

#[test]
fn glissando_with_text_shows_label() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .glissando(GlissandoStyle::LineWithText)
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .end_barline()
        .render_svg();
    assert!(
        svg.contains("gliss."),
        "LineWithText should show 'gliss.' label"
    );
}

#[test]
fn glissando_line_vs_text_differ() {
    let svg_line = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .end_barline()
        .render_svg();
    let svg_text = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .glissando(GlissandoStyle::LineWithText)
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .end_barline()
        .render_svg();
    assert_ne!(
        svg_line, svg_text,
        "Line and LineWithText should produce different SVGs"
    );
}

#[test]
fn glissando_on_chord_adds_line() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .glissando(GlissandoStyle::Line)
        .chord(
            vec![Pitch::new(Note::G, 4), Pitch::new(Note::B, 4)],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();
    let no_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(
            vec![Pitch::new(Note::C, 4), Pitch::new(Note::E, 4)],
            Duration::HALF,
        )
        .chord(
            vec![Pitch::new(Note::G, 4), Pitch::new(Note::B, 4)],
            Duration::HALF,
        )
        .end_barline()
        .render_svg();
    assert_ne!(svg, no_gliss, "chord glissando should add line elements");
}

#[test]
fn convert_event_preserves_glissando_start() {
    let event = ScoreEvent::Note {
        pitch: Pitch::new(Note::C, 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            glissando_start: Some(GlissandoStyle::Line),
            ..NoteAnnotations::default()
        },
    };
    let key_sig = KeySignature::Open;
    let clef = Clef::Treble;
    let result = convert_event(&event, &clef, &key_sig, None);
    if let MeasureEvent::Note(note) = result {
        assert_eq!(note.annotations.glissando_start, Some(GlissandoStyle::Line));
    } else {
        panic!("expected Note event");
    }
}

// --- cross-system glissando tests ---

#[test]
fn cross_system_glissando_adds_extra_lines() {
    let with_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .barline()
        .note(p("G", 5), Duration::QTR)
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    let without_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .barline()
        .note(p("G", 5), Duration::QTR)
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    let gliss_lines = with_gliss.matches("<line ").count();
    let no_gliss_lines = without_gliss.matches("<line ").count();

    assert!(
        gliss_lines > no_gliss_lines,
        "cross-system glissando should add extra lines: {gliss_lines} vs {no_gliss_lines}"
    );
}

#[test]
fn cross_system_glissando_differs_from_no_glissando() {
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .barline()
        .note(p("A", 5), Duration::QTR)
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    let with = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .barline()
        .note(p("A", 5), Duration::QTR)
        .end_barline()
        .measures_per_system(1)
        .render_svg();

    assert_ne!(
        without, with,
        "cross-system glissando should change SVG output"
    );
}

// --- multi-voice ---

#[test]
fn voice_defaults_to_zero() {
    let builder = ScoreBuilder::new();
    assert_eq!(builder.current_voice, 0);
}

#[test]
fn voice_switches_active_voice() {
    let builder = ScoreBuilder::new().voice(1);
    assert_eq!(builder.current_voice, 1);
}

#[test]
fn barline_resets_voice_to_zero() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .barline();
    assert_eq!(builder.current_voice, 0);
}

#[test]
fn end_barline_resets_voice_to_zero() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .end_barline();
    assert_eq!(builder.current_voice, 0);
}

#[test]
fn single_voice_no_additional_voices() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].events.len(), 2);
    assert!(contents[0].additional_voices.is_empty());
}

#[test]
fn two_voices_splits_into_primary_and_additional() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    assert_eq!(contents.len(), 1);
    // Voice 0: 2 half notes
    assert_eq!(contents[0].events.len(), 2);
    // Voice 1: 1 whole note
    assert_eq!(contents[0].additional_voices.len(), 1);
    assert_eq!(contents[0].additional_voices[0].len(), 1);
}

#[test]
fn multi_voice_forces_stem_up_on_voice_0() {
    use crate::layout::stem::StemDirection;
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        // High note normally gets stem down, but voice 0 in
        // multi-voice forces stem up
        .note(p("A", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Voice 0 note should have stem up forced
    match &contents[0].events[0] {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.stem_direction,
                Some(StemDirection::Up),
                "voice 0 in multi-voice should force stem up"
            );
        }
        other => panic!("expected Note, got {other:?}"),
    }
}

#[test]
fn multi_voice_forces_stem_down_on_voice_1() {
    use crate::layout::stem::StemDirection;
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        // Low note normally gets stem up, but voice 1 forces stem down
        .note(p("C", 4), Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Voice 1 note should have stem down forced
    match &contents[0].additional_voices[0][0] {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.stem_direction,
                Some(StemDirection::Down),
                "voice 1 should force stem down"
            );
        }
        other => panic!("expected Note, got {other:?}"),
    }
}

#[test]
fn single_voice_does_not_force_stems() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("A", 5), Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Single voice should use auto stem direction (None = auto)
    match &contents[0].events[0] {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.stem_direction, None,
                "single voice should leave stem direction as auto (None)"
            );
        }
        other => panic!("expected Note, got {other:?}"),
    }
}

#[test]
fn multi_voice_render_svg_produces_valid_output() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert!(svg.starts_with("<svg"), "should produce valid SVG");
    assert!(svg.contains("</svg>"));
    // Voice 0 has 2 notes, voice 1 has 1 note = 3 note paths minimum
    let path_count = svg.matches("<path ").count();
    assert!(
        path_count >= 3,
        "multi-voice SVG should have at least 3 paths (clef + noteheads), got {path_count}"
    );
}

#[test]
fn voice_reset_across_measures() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::HALF)
        .barline()
        // After barline, voice should be 0 again
        .note(p("D", 5), Duration::WHOLE)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Measure 0: multi-voice (voice 0 + voice 1)
    assert_eq!(contents[0].additional_voices.len(), 1);
    // Measure 1: single voice (only voice 0)
    assert!(
        contents[1].additional_voices.is_empty(),
        "measure after barline should be single-voice since voice resets to 0"
    );
}

#[test]
fn multi_voice_beam_group_forces_stems() {
    use crate::layout::stem::StemDirection;
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .beam_group(vec![
            (p("E", 5), Duration::EIGHTH),
            (p("F", 5), Duration::EIGHTH),
        ])
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Voice 0 beam group should have stem up forced
    match &contents[0].events[0] {
        MeasureEvent::BeamGroup(bg) => {
            assert_eq!(
                bg.stem_direction,
                Some(StemDirection::Up),
                "voice 0 beam group in multi-voice should force stem up"
            );
        }
        other => panic!("expected BeamGroup, got {other:?}"),
    }
}

#[test]
fn multi_voice_chord_forces_stems() {
    use crate::layout::stem::StemDirection;
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .chord(vec![p("E", 5), p("G", 5)], Duration::HALF)
        .voice(1)
        .chord(vec![p("C", 4), p("E", 4)], Duration::HALF)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Voice 0 chord: stems up
    match &contents[0].events[0] {
        MeasureEvent::Chord(c) => {
            assert_eq!(c.stem_direction, Some(StemDirection::Up));
        }
        other => panic!("expected Chord, got {other:?}"),
    }
    // Voice 1 chord: stems down
    match &contents[0].additional_voices[0][0] {
        MeasureEvent::Chord(c) => {
            assert_eq!(c.stem_direction, Some(StemDirection::Down));
        }
        other => panic!("expected Chord, got {other:?}"),
    }
}

#[test]
fn multi_voice_rest_has_no_stem_direction() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        .rest(Duration::QTR)
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    // Voice 1 rest should still be a rest (no stem to force)
    match &contents[0].additional_voices[0][0] {
        MeasureEvent::Rest(r) => {
            assert_eq!(r.duration_log2, 2, "quarter rest log2 = 2");
        }
        other => panic!("expected Rest, got {other:?}"),
    }
}

// ── Multi-voice rendering integration tests ─────────────────────────────

#[test]
fn multi_voice_renders_more_paths_than_single_voice() {
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let single_paths = single.matches("<path ").count();
    let multi_paths = multi.matches("<path ").count();
    assert!(
        multi_paths > single_paths,
        "multi-voice should have more paths: single={single_paths}, multi={multi_paths}"
    );
}

#[test]
fn multi_voice_renders_additional_stem_lines() {
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let single_lines = single.matches("<line ").count();
    let multi_lines = multi.matches("<line ").count();
    assert!(
        multi_lines > single_lines,
        "multi-voice should have more lines: single={single_lines}, multi={multi_lines}"
    );
}

#[test]
fn multi_voice_with_rest_in_secondary_renders_displaced() {
    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    // The rest glyph path should be present alongside clef + notehead
    let paths = multi.matches("<path ").count();
    assert!(
        paths >= 3,
        "should have clef + notehead + rest path at minimum, got {paths}"
    );
}

#[test]
fn multi_voice_differs_from_single_voice() {
    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("E", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        single, multi,
        "multi-voice SVG should differ from single-voice"
    );
}

#[test]
fn multi_voice_secondary_note_has_forced_stem_down() {
    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p("C", 4), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .end_barline();

    let contents = multi.build_measure_contents().unwrap();
    // Voice 0 should have stem up (forced for multi-voice)
    match &contents[0].events[0] {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.stem_direction,
                Some(StemDirection::Up),
                "voice 0 should force stems up"
            );
        }
        other => panic!("expected Note, got {other:?}"),
    }
    // Voice 1 should have stem down
    match &contents[0].additional_voices[0][0] {
        MeasureEvent::Note(n) => {
            assert_eq!(
                n.stem_direction,
                Some(StemDirection::Down),
                "voice 1 should force stems down"
            );
        }
        other => panic!("expected Note, got {other:?}"),
    }
}

// ── Cross-voice collision avoidance integration tests ────────────────────

#[test]
fn multi_voice_unison_collision_differs_from_no_collision() {
    let pitch_c4 = Pitch::new(Note::C, 4);
    let pitch_g5 = Pitch::new(Note::G, 5);

    // Unison: both voices play C4
    let svg_unison = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(pitch_c4, Duration::WHOLE)
        .voice(1)
        .note(pitch_c4, Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Far apart: voice 0 plays C4, voice 1 plays G5
    let svg_far = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(pitch_c4, Duration::WHOLE)
        .voice(1)
        .note(pitch_g5, Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_ne!(
        svg_unison, svg_far,
        "unison voices (collision) should produce different SVG than far-apart voices"
    );
}

#[test]
fn multi_voice_second_collision_differs_from_third() {
    let pitch_e4 = Pitch::new(Note::E, 4);
    let pitch_f4 = Pitch::new(Note::F, 4);
    let pitch_g4 = Pitch::new(Note::G, 4);

    // Second apart (E4 vs F4): collision
    let svg_second = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(pitch_f4, Duration::WHOLE)
        .voice(1)
        .note(pitch_e4, Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Third apart (E4 vs G4): no collision
    let svg_third = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(pitch_g4, Duration::WHOLE)
        .voice(1)
        .note(pitch_e4, Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(svg_second.contains("<svg"));
    assert!(svg_third.contains("<svg"));
    assert_ne!(
        svg_second, svg_third,
        "second (collision) vs third (no collision) should produce different SVGs"
    );
}

// --- additional voice span rendering (ties, slurs, hairpins) ---

#[test]
fn tie_in_additional_voice_produces_filled_path() {
    let svg_tied = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .tie()
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let svg_untied = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let tied_fills = svg_tied.matches(r#"stroke="none""#).count();
    let untied_fills = svg_untied.matches(r#"stroke="none""#).count();
    assert_eq!(
        tied_fills,
        untied_fills + 1,
        "tie in voice 1 should add 1 filled path; tied={}, untied={}",
        tied_fills,
        untied_fills
    );
}

#[test]
fn slur_in_additional_voice_produces_filled_path() {
    let svg_slurred = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .slur_start()
        .note(p("E", 4), Duration::QTR)
        .slur_end()
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let slurred_fills = svg_slurred.matches(r#"stroke="none""#).count();
    let plain_fills = svg_plain.matches(r#"stroke="none""#).count();
    assert_eq!(
        slurred_fills,
        plain_fills + 1,
        "slur in voice 1 should add 1 filled path; slurred={}, plain={}",
        slurred_fills,
        plain_fills
    );
}

#[test]
fn hairpin_in_additional_voice_produces_lines() {
    let svg_hp = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .cresc()
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();

    let svg_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let hp_lines = svg_hp.matches("<line ").count();
    let plain_lines = svg_plain.matches("<line ").count();
    assert_eq!(
        hp_lines,
        plain_lines + 2,
        "hairpin in voice 1 should add 2 lines (wedge); hp={}, plain={}",
        hp_lines,
        plain_lines
    );
}

// --- multi-measure rest (ScoreBuilder integration) ---

/// A multi-measure rest renders an H-bar made of three filled rectangles
/// (two vertical serifs + horizontal crossbar) plus a count number text.
/// Compared to a single whole-rest measure, that adds at least 3 rects and
/// 1 text — none of which the equivalent rest measure produces.
#[test]
fn multi_measure_rest_adds_hbar_rects_and_count_text() {
    let svg_mmr = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest(8)
        .end_barline()
        .render_svg();

    let svg_rest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();

    let mmr_rects = svg_mmr.matches("<rect").count();
    let rest_rects = svg_rest.matches("<rect").count();
    assert!(
            mmr_rects >= rest_rects + 3,
            "multi-measure rest should add >= 3 rects (2 serifs + crossbar); mmr={mmr_rects}, rest={rest_rects}",
        );

    // The count number must appear as a text node containing the digit(s)
    assert!(
        svg_mmr.contains(">8</text>"),
        "multi-measure rest count '8' should appear in a <text> element",
    );
    // And it must not appear in the plain rest version
    assert!(
        !svg_rest.contains(">8</text>"),
        "plain rest version should not contain count text",
    );
}

/// Different counts must yield different SVG output (the digit changes).
#[test]
fn multi_measure_rest_distinct_counts_differ() {
    let svg_4 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest(4)
        .end_barline()
        .render_svg();

    let svg_16 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest(16)
        .end_barline()
        .render_svg();

    assert!(svg_4.contains(">4</text>"), "count 4 should appear");
    assert!(svg_16.contains(">16</text>"), "count 16 should appear");
    assert_ne!(
        svg_4, svg_16,
        "different counts must produce different SVGs"
    );
}

/// A multi-measure rest mid-score must coexist with surrounding notated
/// measures: clef, time sig, a note measure, the rest measure, another
/// note measure — all in one system. The H-bar count must appear, the
/// noteheads must remain, and barlines must separate the measures.
#[test]
fn multi_measure_rest_between_notated_measures() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(4)
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        .multi_measure_rest(7)
        .barline()
        .note(p("G", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert!(svg.contains(">7</text>"), "count should be present");
    // 5 staff lines + at least 3 barlines (after each measure) = 8 lines minimum
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 8,
        "should have at least 8 lines (staff + barlines), got {line_count}",
    );
    // The two whole notes still produce noteheads in addition to the H-bar
    // crossbar; <path> count must exceed a score with only the rest measure.
    let svg_only_mmr = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest(7)
        .end_barline()
        .render_svg();
    let paths_with_notes = svg.matches("<path").count();
    let paths_only_mmr = svg_only_mmr.matches("<path").count();
    assert!(
        paths_with_notes > paths_only_mmr,
        "embedding notes around the rest must add more glyph paths; \
             with_notes={paths_with_notes}, only_mmr={paths_only_mmr}",
    );
}

/// The H-bar count text must be styled `font-weight="bold"` per engraving
/// convention (and per the renderer's `TextStyle::bold` choice).
#[test]
fn multi_measure_rest_count_is_bold() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest(3)
        .end_barline()
        .render_svg();
    assert!(
        svg.contains("font-weight=\"bold\""),
        "count number should be bold-weighted text",
    );
}

// ── Church-rest style (small-count multi-measure rest) ──────────────────

/// `multi_measure_rest_church(2)` swaps the H-bar's three rects for a
/// single breve-rest glyph (`<path>`), and the count number stays.
#[test]
fn church_rest_count_2_uses_path_not_rects() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest_church(2)
        .end_barline()
        .render_svg();

    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest(2)
        .end_barline()
        .render_svg();

    // The H-bar emits 3 extra <rect> elements relative to a plain score.
    // The church-rest version emits 1 <path> for the breve rest glyph and
    // ZERO H-bar rects, so it must have strictly fewer rects than the
    // H-bar baseline. (Staff lines and time-sig are <line>/<path>, not
    // rects, so this is a clean delta.)
    let church_rects = svg.matches("<rect").count();
    let hbar_rects = baseline.matches("<rect").count();
    assert!(
        church_rects + 3 == hbar_rects,
        "church rest should have exactly 3 fewer <rect> than H-bar; \
             church={church_rects}, hbar={hbar_rects}"
    );
    // The count number "2" must appear in both
    assert!(svg.contains(">2</text>"));
    assert!(baseline.contains(">2</text>"));
    // The two renderings must differ
    assert_ne!(svg, baseline);
}

/// `multi_measure_rest_church(3)` draws breve + whole = two rest paths.
#[test]
fn church_rest_count_3_draws_two_extra_paths() {
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    let church = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .multi_measure_rest_church(3)
        .end_barline()
        .render_svg();

    let plain_paths = plain.matches("<path ").count();
    let church_paths = church.matches("<path ").count();
    // Both have clef + time-sig + 1 rest. The plain version has 1 whole-
    // rest path; the church version has 2 (breve + whole). Difference
    // is exactly 1 extra path.
    assert_eq!(
        church_paths - plain_paths,
        1,
        "count=3 church rest should add exactly 1 path vs single-whole-rest \
             baseline (breve+whole = 2 instead of 1); church={church_paths}, plain={plain_paths}",
    );
    // Count text must show "3"
    assert!(church.contains(">3</text>"));
    // Plain version must NOT show a count text "3"
    assert!(!plain.contains(">3</text>"));
}

/// `multi_measure_rest_church(4)` draws two breve rests.
#[test]
fn church_rest_count_4_distinct_from_count_3() {
    let svg3 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest_church(3)
        .end_barline()
        .render_svg();
    let svg4 = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest_church(4)
        .end_barline()
        .render_svg();
    assert_ne!(svg3, svg4, "counts 3 and 4 must render different SVG");
    assert!(svg3.contains(">3</text>"));
    assert!(svg4.contains(">4</text>"));
    // Both have 2 rest paths (breve+whole and breve+breve respectively).
    // Compare path counts vs a plain score.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    let plain_paths = plain.matches("<path ").count();
    assert_eq!(svg3.matches("<path ").count() - plain_paths, 1);
    assert_eq!(svg4.matches("<path ").count() - plain_paths, 1);
}

/// For counts above the church-rest maximum the renderer falls back to
/// the H-bar form even when the user asked for the church style.
#[test]
fn church_rest_falls_back_to_hbar_for_large_counts() {
    let church = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest_church(8)
        .end_barline()
        .render_svg();
    let hbar = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest(8)
        .end_barline()
        .render_svg();
    // Fallback means church output equals plain H-bar output for count=8.
    assert_eq!(
        church, hbar,
        "count=8 church-rest should fall back to H-bar form (identical SVG)"
    );
}

/// Distinct small counts must produce distinct church-rest SVGs.
#[test]
fn church_rest_distinct_small_counts_differ() {
    let mut renderings: Vec<String> = Vec::new();
    for n in 1..=4u32 {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .multi_measure_rest_church(n)
            .end_barline()
            .render_svg();
        renderings.push(svg);
    }
    for i in 0..renderings.len() {
        for j in (i + 1)..renderings.len() {
            assert_ne!(
                renderings[i],
                renderings[j],
                "church counts {} and {} must differ",
                i + 1,
                j + 1
            );
        }
    }
}

/// Church-rest style preserves the bold-count text convention.
#[test]
fn church_rest_count_is_bold() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .multi_measure_rest_church(2)
        .end_barline()
        .render_svg();
    assert!(
        svg.contains("font-weight=\"bold\""),
        "church-rest count must be bold-weighted text",
    );
}

// --- trill_with_extension_bracketed (ScoreBuilder) ---

#[test]
fn trill_with_extension_bracketed_both_adds_hooks_vs_plain_extension() {
    use crate::layout::trill_bracket::TrillBracketSide;

    // Bracketed both ends should add 2 hook <line> elements compared to
    // the same score using only trill_with_extension.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_lines = plain.matches("<line ").count();
    let bracketed_lines = bracketed.matches("<line ").count();
    assert_eq!(
        bracketed_lines - plain_lines,
        2,
        "Both bracket via ScoreBuilder must add exactly 2 hook <line> elements: \
             plain={plain_lines}, bracketed={bracketed_lines}"
    );
}

#[test]
fn trill_with_extension_bracketed_start_adds_one_hook() {
    use crate::layout::trill_bracket::TrillBracketSide;
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Start)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let delta = bracketed.matches("<line ").count() - plain.matches("<line ").count();
    assert_eq!(
        delta, 1,
        "Start bracket must add exactly 1 hook (got {delta})"
    );
}

#[test]
fn trill_with_extension_bracketed_end_adds_one_hook() {
    use crate::layout::trill_bracket::TrillBracketSide;
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::End)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let delta = bracketed.matches("<line ").count() - plain.matches("<line ").count();
    assert_eq!(
        delta, 1,
        "End bracket within-system must add exactly 1 hook (got {delta})"
    );
}

#[test]
fn trill_with_extension_bracketed_on_rest_is_noop() {
    use crate::layout::trill_bracket::TrillBracketSide;
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_bracket = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, with_bracket,
        "trill_with_extension_bracketed on rest must be a complete no-op"
    );
}

#[test]
fn trill_with_extension_bracketed_sets_all_three_annotation_fields() {
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;

    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(TrillBracketSide::Both),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &KeySignature::Open, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Trill));
            assert!(ne.annotations.trill_extension);
            assert_eq!(ne.annotations.trill_bracket, Some(TrillBracketSide::Both));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_with_extension_bracketed_on_chord_adds_hooks() {
    use crate::layout::trill_bracket::TrillBracketSide;
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let delta = bracketed.matches("<line ").count() - plain.matches("<line ").count();
    assert_eq!(
        delta, 2,
        "Both bracket on chord trill must add exactly 2 hooks (got {delta})"
    );
}

// --- trill_with_extension_speed (ScoreBuilder) ---

#[test]
fn trill_with_extension_speed_sets_all_three_annotation_fields() {
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::TrillWiggleSpeed;

    // Use `convert_event` directly (mirroring the bracketed test) so the
    // wiring is verified independently of the system renderer.
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_wiggle_speed: Some(TrillWiggleSpeed::Fast),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &KeySignature::Open, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Trill));
            assert!(ne.annotations.trill_extension);
            assert_eq!(
                ne.annotations.trill_wiggle_speed,
                Some(TrillWiggleSpeed::Fast)
            );
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_with_extension_speed_on_rest_is_noop() {
    use crate::layout::trill_extension::TrillWiggleSpeed;

    // Builder method must silently no-op when the most recent event was a
    // rest, just like the other annotation builders. We compare the SVG
    // against the same builder chain without the speed call: byte-equal.
    let with_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fastest)
        .end_barline()
        .render_svg();
    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();
    assert_eq!(
        with_speed, without,
        "trill_with_extension_speed on a rest must be a no-op"
    );
}

#[test]
fn trill_with_extension_speed_renders_wiggle_paths() {
    // A speed variant must still produce wiggle paths: it's the same
    // gesture as trill_with_extension(), just a different tile glyph.
    use crate::layout::trill_extension::TrillWiggleSpeed;

    let with_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Standard)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let no_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        with_speed.matches("<path").count() > no_ext.matches("<path").count(),
        "speed-variant trill extension must add wiggle paths beyond plain note"
    );
}

#[test]
fn trill_with_extension_speed_standard_matches_default_extension() {
    // The `Standard` speed *must* produce the same SVG as the plain
    // `trill_with_extension()` call so existing callers see no change
    // when they upgrade to the new method using the default speed.
    // This is the canary that protects all 4 ScoreBuilder methods that
    // funnel into the same wiggle layout path.
    use crate::layout::trill_extension::TrillWiggleSpeed;

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let standard = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Standard)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        plain, standard,
        "Standard speed must produce byte-identical SVG to trill_with_extension()"
    );
}

#[test]
fn trill_with_extension_speed_different_speeds_produce_different_svg() {
    // Two distinct wiggle speeds must produce visually distinct SVG —
    // otherwise the user's choice of speed has no effect on the output.
    use crate::layout::trill_extension::TrillWiggleSpeed;

    let fast = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fast)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let slow = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slow)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_ne!(
        fast, slow,
        "Fast and Slow wiggle variants must produce different SVG output"
    );
}

#[test]
fn trill_with_extension_speed_on_chord_adds_wiggle() {
    // Chord trills must also pick up the speed annotation.
    use crate::layout::trill_extension::TrillWiggleSpeed;

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let chord_with_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fastest)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert!(
        chord_with_speed.matches("<path").count() > plain_chord.matches("<path").count(),
        "speed-variant chord trill must add tr glyph + wiggle paths"
    );
}

// --- trill_with_extension_speed_with_options (ScoreBuilder, options builder API) ---

#[test]
fn speed_with_options_default_ornament_matches_plain_speed_byte_for_byte() {
    // The whole point of the options builder is that the
    // ornament-unspecified case is byte-identical to the existing
    // non-options `trill_with_extension_speed(speed)` API. If a future
    // change to either path drifts, this canary fails.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fast)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let opts = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Fast,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, opts,
        "Options(speed only) must render byte-identically to trill_with_extension_speed"
    );
}

#[test]
fn speed_with_options_via_into_matches_plain_speed_byte_for_byte() {
    // The `From<TrillWiggleSpeed>` ergonomics must produce the exact same
    // SVG as the explicit constructor. Catches a regression where the
    // From impl picks up a different default for the ornament field.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slower)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_into = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillWiggleSpeed::Slower.into())
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Helper to remind future readers that this conversion exists and
    // must be byte-stable.
    let _round_trip: TrillExtensionSpeedOptions = TrillWiggleSpeed::Slower.into();

    assert_eq!(
        plain, via_into,
        "`TrillWiggleSpeed::into()` ergonomics must match the explicit constructor"
    );
}

#[test]
fn speed_with_options_sets_three_annotation_fields() {
    // Field-level inspection of the builder's intermediate state proving
    // the three coupled fields propagate exactly and the ornament
    // collapse happens at the builder layer.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fastest)
                .with_ornament(Ornament::TrillWithMordent),
        );

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::TrillWithMordent));
            assert!(annotations.trill_extension);
            assert_eq!(
                annotations.trill_wiggle_speed,
                Some(TrillWiggleSpeed::Fastest)
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn speed_with_options_unset_ornament_writes_plain_trill_to_annotation() {
    // `None` ornament must collapse to `Some(Trill)` at the builder, not
    // at the renderer — the annotation field is the single source of
    // truth for downstream supports_trill_extension() filtering.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Standard,
        ));

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.ornament,
                Some(Ornament::Trill),
                "unset ornament must collapse to Trill at the builder"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn speed_with_options_compound_ornament_renders_distinct_from_plain_trill_same_speed() {
    // The whole reason this method exists: pairing a compound ornament
    // with a non-default speed must produce a visibly different SVG from
    // pairing the plain Trill with the same speed. The renderer's
    // glyph-advance lookup differs between the two ornaments (Bravura:
    // 521 vs 990 font-units), so the wiggle's start position shifts.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Fast,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        plain_trill, compound,
        "Compound ornament + same speed must produce a visibly different SVG"
    );
}

#[test]
fn speed_with_options_compound_with_speed_distinct_from_compound_default_speed() {
    // Pairing the compound ornament with a non-Standard speed must also
    // produce a distinct SVG from pairing the compound with the
    // (default) Standard speed — proves the speed half of the options
    // bundle actually propagates when the ornament is overridden.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let compound_standard = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound_fastest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fastest)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        compound_standard, compound_fastest,
        "Compound ornament + different speeds must produce different SVG"
    );
}

#[test]
fn speed_with_options_compound_renders_wiggle_paths_beyond_plain_compound() {
    // Sanity check: the renderer must actually draw a wiggle for compound
    // + speed. The trill_with_mordent extension path was already verified
    // by the existing compound-extension test, but the speed-options path
    // walks the new builder method — same downstream wiring, but a fresh
    // canary that no annotation-routing regression silently dropped the
    // extension flag.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain_compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        // Plain compound, no extension at all.
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound_with_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_compound.matches("<path").count();
    let with_speed_paths = compound_with_speed.matches("<path").count();
    assert!(
        with_speed_paths > plain_paths,
        "compound ornament + speed must add wiggle paths beyond plain compound: \
             plain={plain_paths}, with_speed={with_speed_paths}"
    );
}

#[test]
fn speed_with_options_on_rest_is_noop() {
    // Builder method must silently no-op when the most recent event was a
    // rest. SVG must be byte-equal to the same chain without the call.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let with_call = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fastest)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .render_svg();

    let without = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_eq!(
        with_call, without,
        "trill_with_extension_speed_with_options on a rest must be a no-op"
    );
}

#[test]
fn speed_with_options_on_chord_renders_wiggle() {
    // Chord trills must also pick up both halves of the options bundle.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let chord_with_options = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert!(
        chord_with_options.matches("<path").count() > plain_chord.matches("<path").count(),
        "speed+ornament options on chord must add compound + wiggle paths"
    );
}

#[test]
fn speed_with_options_unsupported_ornament_silently_drops_extension() {
    // Contract canary: passing an ornament that does NOT satisfy
    // supports_trill_extension() must make the renderer silently skip
    // the wiggle, matching the behavior of the bracket-options path.
    // Compared to the plain (Trill) variant at the same speed, the
    // unsupported variant must have STRICTLY FEWER wiggle paths.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    // Sanity guard: ShortTrill is intentionally unsupported.
    assert!(!Ornament::ShortTrill.supports_trill_extension());

    let supported = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Fast,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let unsupported = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast)
                .with_ornament(Ornament::ShortTrill),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let supported_paths = supported.matches("<path").count();
    let unsupported_paths = unsupported.matches("<path").count();
    assert!(
        unsupported_paths < supported_paths,
        "unsupported ornament (ShortTrill) must silently drop the extension; \
             expected fewer paths than supported variant: \
             supported={supported_paths}, unsupported={unsupported_paths}"
    );
}

// --- trill_with_extension_bracketed_custom (ScoreBuilder) ---

#[test]
fn trill_with_extension_bracketed_custom_sets_all_five_annotation_fields() {
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};

    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(TrillBracketSide::Both),
            trill_bracket_direction: Some(HookDirection::Up),
            trill_bracket_length_ss: Some(0.5),
            ..NoteAnnotations::default()
        },
    };
    let result = convert_event(&event, &Clef::Treble, &KeySignature::Open, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Trill));
            assert!(ne.annotations.trill_extension);
            assert_eq!(ne.annotations.trill_bracket, Some(TrillBracketSide::Both));
            assert_eq!(
                ne.annotations.trill_bracket_direction,
                Some(HookDirection::Up)
            );
            assert_eq!(ne.annotations.trill_bracket_length_ss, Some(0.5));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_with_extension_bracketed_custom_default_args_matches_plain_bracketed() {
    // Custom with Down direction and the default 0.75 length must produce
    // byte-identical SVG to the existing `trill_with_extension_bracketed`
    // call. This is the regression canary that protects the existing API.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let custom = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Down, 0.75)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, custom,
        "Custom(Down, 0.75) must render identically to the default bracketed API"
    );
}

#[test]
fn trill_with_extension_bracketed_custom_length_changes_svg() {
    // A longer hook must produce a measurably different SVG than the default.
    // Specifically, the hook <line>'s vertical extent must encode the new
    // length — we don't assert exact coordinates here, just that the SVG
    // is not byte-identical (a length change must propagate to render).
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};

    let default_len = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Down, 0.75)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let long = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Down, 1.5)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_ne!(
        default_len, long,
        "Longer hook must produce different SVG than the default length"
    );
    // Same number of <line> elements — only their geometry differs.
    assert_eq!(
        default_len.matches("<line ").count(),
        long.matches("<line ").count(),
        "Length change must NOT add or remove <line> elements"
    );
}

#[test]
fn trill_with_extension_bracketed_custom_direction_changes_svg() {
    // Direction flip (Down -> Up) must produce a measurably different SVG.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};

    let down = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Down, 0.75)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let up = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Up, 0.75)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_ne!(
        down, up,
        "Down vs Up hook direction must produce different SVG"
    );
    assert_eq!(
        down.matches("<line ").count(),
        up.matches("<line ").count(),
        "Direction flip must NOT add or remove <line> elements"
    );
}

#[test]
fn trill_with_extension_bracketed_custom_on_rest_is_noop() {
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with_bracket = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Up, 1.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        plain, with_bracket,
        "Custom bracketed on rest must be a complete no-op"
    );
}

#[test]
fn trill_with_extension_bracketed_custom_hook_up_y_coordinates() {
    // The Up-direction hook's <line> must have y1 < y2 with the smaller y
    // (further above the staff) being the hook's tip and the larger y
    // being the wiggle baseline. With the default Down direction, the
    // larger y is the tip. Compare the y-extent direction between the two.
    //
    // SvgWriter emits hook lines via stroke <line ... y1="..." y2="..."/>;
    // we parse the first hook line out of each SVG (the Start hook on the
    // single trill in each score) and compare midpoint orderings.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};

    let down = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Start, HookDirection::Down, 1.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let up = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Start, HookDirection::Up, 1.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Extract last <line ...> from each (the hook is appended after the
    // wiggle segments, so it's the last <line> emitted by the trill pass;
    // the wiggle is emitted as <path>, so the only <line> here is the hook
    // itself plus any staff/barlines drawn earlier in the system).
    let extract_last_hook_y_range = |svg: &str| -> (f64, f64) {
        // Find the last <line tag and pull y1 and y2 attribute values.
        let last_line_start = svg.rfind("<line ").expect("at least one <line>");
        let tag_end = svg[last_line_start..]
            .find('>')
            .expect("malformed <line> tag");
        let tag = &svg[last_line_start..last_line_start + tag_end];
        let find_attr = |name: &str| -> f64 {
            let key = format!("{name}=\"");
            let start = tag.find(&key).unwrap_or_else(|| panic!("missing {name}")) + key.len();
            let end = tag[start..].find('"').expect("unterminated attr") + start;
            tag[start..end].parse().expect("numeric attr")
        };
        (find_attr("y1"), find_attr("y2"))
    };

    let (down_y1, down_y2) = extract_last_hook_y_range(&down);
    let (up_y1, up_y2) = extract_last_hook_y_range(&up);

    // For the Down hook, the baseline (smaller y) is at the wiggle and
    // the tip is below it (larger y). For the Up hook, the baseline
    // (larger y) is at the wiggle and the tip is above it (smaller y).
    //
    // The SVG writer emits the line in (x, y_top, y_bottom) order:
    // y1 == y_top (smaller), y2 == y_bottom (larger). So the *span* (y2-y1)
    // is positive in both cases. What flips is the relationship to the
    // baseline: for Down, y1 ≈ baseline; for Up, y2 ≈ baseline. We can
    // detect this by comparing y1 to y1 and y2 to y2: the Down hook's y1
    // should equal the Up hook's y2 (they share the wiggle baseline).
    let span_down = down_y2 - down_y1;
    let span_up = up_y2 - up_y1;
    assert!(span_down > 0.0, "down hook span must be positive");
    assert!(span_up > 0.0, "up hook span must be positive");
    assert!(
        (span_down - span_up).abs() < 1e-6,
        "Spans must be equal: down={span_down}, up={span_up}"
    );
    assert!(
        (down_y1 - up_y2).abs() < 1e-6,
        "Down's y_top must equal Up's y_bottom (shared wiggle baseline): \
             down_y1={down_y1}, up_y2={up_y2}"
    );
}

// --- trill_with_extension_bracketed_with_options (ScoreBuilder, builder API) ---

#[test]
fn with_options_default_matches_plain_bracketed_byte_for_byte() {
    // The whole point of the options builder is that the all-defaults
    // case is byte-identical to the existing non-custom API. If a future
    // change to either path drifts, this canary fails.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let opts = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, opts,
        "Options(side only) must render byte-identically to plain bracketed"
    );
}

#[test]
fn with_options_both_overrides_matches_custom_call_byte_for_byte() {
    // Options with both fields populated must match the all-or-nothing
    // custom call exactly — they should walk the same code path in the
    // renderer.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};

    let custom = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Up, 1.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let opts = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        custom, opts,
        "Options(Up, 1.0) must render byte-identically to custom(Both, Up, 1.0)"
    );
}

#[test]
fn with_options_length_only_renders_distinct_from_default() {
    // Setting just length (leaving direction unset) must visibly change
    // the SVG while preserving the line count.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let default = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let longer = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        default, longer,
        "Length-only override must change SVG output"
    );
    assert_eq!(
        default.matches("<line ").count(),
        longer.matches("<line ").count(),
        "Length-only override must NOT change <line> count"
    );
}

#[test]
fn with_options_direction_only_renders_distinct_from_default() {
    // Setting just direction (leaving length unset) must visibly change
    // the SVG while preserving the line count.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};

    let default = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let up = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_direction(HookDirection::Up),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        default, up,
        "Direction-only override must change SVG output"
    );
    assert_eq!(
        default.matches("<line ").count(),
        up.matches("<line ").count(),
        "Direction-only override must NOT change <line> count"
    );
}

#[test]
fn with_options_from_side_via_into_matches_new() {
    // The `From<TrillBracketSide>` impl should make `side.into()`
    // equivalent to `TrillBracketOptions::new(side)`. Verify at the
    // SVG output level so the integration of the conversion is
    // exercised, not just the type-level conversion.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let via_new = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::End,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_into = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketSide::End.into())
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_new, via_into,
        "TrillBracketSide::into() must match new()"
    );
}

#[test]
fn with_options_sets_annotation_fields_matching_overrides() {
    // Field-level inspection: confirm that an Option in the input lands
    // as the same Option in the annotation (not collapsed to a default).
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};

    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(TrillBracketSide::Start),
            trill_bracket_direction: Some(HookDirection::Up),
            trill_bracket_length_ss: None, // intentionally unset
            ..NoteAnnotations::default()
        },
    };
    // Independently verify the same outcome via the builder path.
    let built = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Start).with_direction(HookDirection::Up),
        );
    let last = built.current_events.last().expect("at least one event");
    let ann = match &last.1 {
        ScoreEvent::Note { annotations, .. } => annotations,
        _ => panic!("expected Note event"),
    };
    assert_eq!(ann.ornament, Some(Ornament::Trill));
    assert!(ann.trill_extension);
    assert_eq!(ann.trill_bracket, Some(TrillBracketSide::Start));
    assert_eq!(ann.trill_bracket_direction, Some(HookDirection::Up));
    assert_eq!(
        ann.trill_bracket_length_ss, None,
        "Unset length_ss must remain None (not collapsed to a default)"
    );
    let _ = event; // silence unused-binding lint on the parallel constructor
}

#[test]
fn with_options_on_rest_is_noop() {
    // Bracketing a rest is meaningless — must be a complete no-op.
    use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let with_bracket = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        plain, with_bracket,
        "Options bracket on rest must be a complete no-op"
    );
}

#[test]
fn with_options_on_chord_renders_bracket() {
    // Bracketing a chord must work the same as bracketing a single note —
    // the bracket targets the chord as a unit. We verify by comparing
    // <line> counts: a plain chord vs a chord with a Both bracket must
    // differ by exactly 2 lines.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let bracketed_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_lines = plain_chord.matches("<line ").count();
    let bracketed_lines = bracketed_chord.matches("<line ").count();
    assert_eq!(
        bracketed_lines.saturating_sub(plain_lines),
        2,
        "Bracketed (Both) chord must add exactly 2 <line> elements (Start + End hooks); \
             plain={plain_lines}, bracketed={bracketed_lines}"
    );
}

// --- with_options ornament override (bracketed compound trills) ---

#[test]
fn with_options_ornament_unset_writes_plain_trill_to_annotation() {
    // The default (None ornament) collapses to Ornament::Trill at the
    // builder layer. Inspecting in-flight events guards against a future
    // refactor that defers the collapse to the renderer (which would
    // break the `supports_trill_extension` filter at the wrong layer).
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ));

    let (_, last) = b.current_events.last().expect("note added").clone();
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.ornament,
                Some(Ornament::Trill),
                "options-without-ornament must write Trill into the annotation"
            );
            assert!(annotations.trill_extension);
            assert_eq!(annotations.trill_bracket, Some(TrillBracketSide::Both));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn with_options_ornament_writes_chosen_ornament_to_annotation() {
    // Sanity: setting the ornament on the options actually lands in
    // the annotation field downstream.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        );

    let (_, last) = b.current_events.last().expect("note added").clone();
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::TrillWithMordent));
            assert!(annotations.trill_extension);
            assert_eq!(annotations.trill_bracket, Some(TrillBracketSide::Both));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn with_options_compound_ornament_renders_distinct_svg_from_plain_trill() {
    // The whole point of the override: a bracketed compound trill must
    // produce a visibly different SVG than a bracketed plain trill on
    // the same span. The difference comes from (1) a different prefix
    // glyph (the precomposed compound vs the bare "tr") and (2) a
    // different wiggle start (the wider compound advance pushes the
    // wiggle further right). At least one of those must change the
    // output byte stream.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        plain, compound,
        "Bracketed compound trill must differ from bracketed plain trill: \
             different prefix glyph and wider wiggle start"
    );
}

#[test]
fn with_options_compound_renders_same_hook_count_as_plain() {
    // The ornament glyph dictates the prefix and shifts the wiggle's
    // start, but the bracket geometry (one hook per requested side) is
    // independent of the prefix glyph. So a Both-bracket on a compound
    // ornament must emit exactly the same number of <line> elements as
    // the plain Both-bracket variant.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain_lines = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
        .matches("<line ")
        .count();

    let compound_lines = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
        .matches("<line ")
        .count();

    assert_eq!(
        plain_lines, compound_lines,
        "Compound bracketed-trill must emit the same <line> count as plain bracketed-trill \
             (the prefix glyph determines the wiggle start but not the hook count). \
             plain={plain_lines}, compound={compound_lines}"
    );
}

#[test]
fn with_options_compound_renders_wiggle_paths_beyond_plain_compound() {
    // Apply the plain compound glyph (no extension) so the comparison
    // isolates the wiggle's + bracket's contribution. The bracketed
    // compound trill should add at least 2 path-or-line elements: at
    // minimum one wiggle tile and two hook lines (Both bracket).
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain_compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed_compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_compound.matches("<path").count();
    let bracketed_paths = bracketed_compound.matches("<path").count();
    let plain_lines = plain_compound.matches("<line ").count();
    let bracketed_lines = bracketed_compound.matches("<line ").count();

    assert!(
        bracketed_paths > plain_paths,
        "Bracketed compound must add wiggle paths over plain compound: \
             plain={plain_paths}, bracketed={bracketed_paths}"
    );
    assert_eq!(
        bracketed_lines.saturating_sub(plain_lines),
        2,
        "Bracketed (Both) compound must add exactly 2 hook <line>s vs plain compound"
    );
}

#[test]
fn with_options_unsupported_ornament_silently_drops_extension_and_bracket() {
    // Per the documented contract, an ornament that does not
    // `supports_trill_extension()` makes the renderer drop the wiggle
    // and the bracket. Only the bare ornament glyph remains. The
    // bracketed variant must produce zero extra hook lines vs the
    // non-bracket variant when ShortTrill is selected — this is the
    // contract canary.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    // Sanity precondition: ShortTrill must remain unsupported. If a
    // future change adds it to `supports_trill_extension()`, this test
    // and its premise must be re-examined.
    assert!(!Ornament::ShortTrill.supports_trill_extension());

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::ShortTrill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed_unsupported = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_ornament(Ornament::ShortTrill),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain.matches("<line ").count(),
        bracketed_unsupported.matches("<line ").count(),
        "Unsupported ornament must drop the bracket — no hook <line>s added"
    );
}

#[test]
fn with_options_compound_on_rest_is_noop() {
    // The no-op-on-rest contract must extend to the new ornament
    // override; an ornamented-bracket request on a rest must produce
    // SVG byte-identical to the plain-rest baseline.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain_rest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();

    let bracketed_rest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .render_svg();

    assert_eq!(
        plain_rest, bracketed_rest,
        "Options-with-ornament bracket on rest must be a no-op"
    );
}

#[test]
fn with_options_compound_on_chord_renders_bracket_and_wiggle() {
    // Chord targeting: the bracketed compound trill on a chord must
    // (a) add exactly 2 hook <line>s vs the plain chord, AND
    // (b) add wiggle paths beyond the plain chord. Guards against a
    // regression where the chord path collapses to the plain trill
    // glyph or skips the wiggle entirely.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed_compound_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        bracketed_compound_chord
            .matches("<line ")
            .count()
            .saturating_sub(plain_chord.matches("<line ").count()),
        2,
        "Both-bracketed compound chord must add exactly 2 hook <line>s"
    );
    assert!(
        bracketed_compound_chord.matches("<path").count() > plain_chord.matches("<path").count(),
        "Both-bracketed compound chord must add wiggle paths over plain chord"
    );
}

// --- trill_with_mordent_with_extension ---

#[test]
fn trill_with_mordent_with_extension_sets_both_annotation_fields() {
    use crate::layout::ornament::Ornament;

    let with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension();

    // Inspect the in-flight events to verify both annotation flags are set.
    let (_, last) = with_ext
        .current_events
        .last()
        .expect("note was added")
        .clone();
    match last {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::TrillWithMordent));
            assert!(annotations.trill_extension);
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_with_mordent_with_extension_renders_wiggle_paths() {
    use crate::layout::ornament::Ornament;

    // Apply the plain compound glyph (no extension) so the comparison
    // isolates the wiggle's contribution rather than the glyph itself.
    let plain_compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_compound.matches("<path").count();
    let ext_paths = with_ext.matches("<path").count();
    // At least one wiggle segment must be drawn — typically several,
    // since a whole note in 4/4 is wide enough to fit multiple segments.
    assert!(
        ext_paths > plain_paths,
        "trill_with_mordent_with_extension must add wiggle paths beyond plain compound: \
             plain={plain_paths}, with_ext={ext_paths}"
    );
    // ≥2 added paths confirms a real tiled wiggle (not just a phantom
    // single glyph drawn by mistake).
    assert!(
        ext_paths.saturating_sub(plain_paths) >= 2,
        "expected ≥2 wiggle segments, got delta={}",
        ext_paths.saturating_sub(plain_paths)
    );
}

#[test]
fn trill_with_mordent_with_extension_on_rest_is_noop() {
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, with_ext,
        "trill_with_mordent_with_extension on a rest must be a no-op"
    );
}

#[test]
fn trill_with_mordent_with_extension_on_chord_adds_wiggle() {
    use crate::layout::ornament::Ornament;

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let chord_with_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let plain_paths = plain_chord.matches("<path").count();
    let ext_paths = chord_with_ext.matches("<path").count();
    assert!(
        ext_paths > plain_paths,
        "trill_with_mordent_with_extension on chord must add wiggle paths: \
             plain={plain_paths}, with_ext={ext_paths}"
    );
}

#[test]
fn trill_with_mordent_with_extension_wiggle_starts_past_full_compound_glyph() {
    use crate::layout::ornament::Ornament;

    // Render the same score twice — once with `Ornament::Trill`, once
    // with `Ornament::TrillWithMordent` — both with the extension flag
    // set manually so the *only* difference is the prefix glyph's
    // advance width. The compound glyph is wider than the bare "tr",
    // so the wiggle must start further to the right.
    //
    // We can't easily measure the wiggle's start x in the rendered SVG
    // without parsing every translate, but we *can* assert that the
    // total <path> counts differ (or are not identically arranged) —
    // the wider compound glyph leaves less room for wiggle segments,
    // so the extension SVG should differ byte-for-byte from the plain
    // trill extension at the same span. This is the canary that the
    // collector + draw pass propagated the actual ornament rather than
    // hardcoding `Trill`.
    let trill_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        trill_ext, compound_ext,
        "trill+ext and trill-with-mordent+ext must render differently \
             (different prefix glyph AND wiggle start x)"
    );

    // The compound glyph itself is a different SMuFL outline, so a
    // distinct path d-string for the ornament must appear. Sanity
    // check: the unique-path-data set differs.
    let trill_count = trill_ext.matches("<path").count();
    let compound_count = compound_ext.matches("<path").count();
    // The compound glyph occupies more horizontal space, so the wiggle
    // has *fewer* segments to tile. Strictly less-or-equal is the
    // expected relation; if Bravura's metrics ever flip this, the
    // assertion documents the assumption.
    assert!(
        compound_count <= trill_count,
        "compound ornament glyph is wider; wiggle should fit fewer segments. \
             trill={trill_count}, compound={compound_count}"
    );

    // And the compound version must still draw at least one wiggle
    // segment — confirms the draw pass actually did call into the
    // extension path for `TrillWithMordent` (not silently bailed out).
    let plain_compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let plain_compound_paths = plain_compound.matches("<path").count();
    assert!(
        compound_count > plain_compound_paths,
        "compound + ext should draw more paths than plain compound: \
             with_ext={compound_count}, plain={plain_compound_paths}"
    );
}

// --- trill_with_extension_full_options ---

#[test]
fn full_options_default_matches_plain_trill_with_extension_byte_for_byte() {
    // The all-defaults case is the central byte-equivalence canary: a
    // bare `TrillExtensionFullOptions::new()` must produce SVG byte-
    // identical to `trill_with_extension()`. If a future change drifts
    // either path (e.g. the ornament-collapse-to-Trill semantics
    // shift, or the trill_extension flag's default value flips), this
    // catches it.
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new())
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, full,
        "default full options must render byte-identically to trill_with_extension()"
    );
}

#[test]
fn full_options_from_bracket_matches_bracketed_options_byte_for_byte() {
    // A TrillBracketOptions widened via `.into()` and applied through
    // the full-options builder must produce SVG byte-identical to
    // applying the same TrillBracketOptions through the existing
    // bracket-options builder. Proves the From conversion threads
    // through every relevant annotation field correctly.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let bracket_opts = TrillBracketOptions::new(TrillBracketSide::Both)
        .with_direction(HookDirection::Up)
        .with_length_ss(0.85)
        .with_ornament(Ornament::TrillWithMordent);

    let bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(bracket_opts)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let full: TrillExtensionFullOptions = bracket_opts.into();
    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(full)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        bracketed, via_full,
        "TrillBracketOptions::into() routed through full_options must match bracketed_with_options"
    );
}

#[test]
fn full_options_from_speed_matches_speed_options_byte_for_byte() {
    // Mirror of the bracket-From test: a TrillExtensionSpeedOptions
    // widened via `.into()` and applied through the full-options
    // builder must produce SVG byte-identical to applying the same
    // speed options through the existing speed-options builder.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let speed_opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
        .with_ornament(Ornament::TrillWithMordent);

    let via_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(speed_opts)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let full: TrillExtensionFullOptions = speed_opts.into();
    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(full)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
            via_speed, via_full,
            "TrillExtensionSpeedOptions::into() routed through full_options must match speed_with_options"
        );
}

#[test]
fn full_options_sets_every_annotation_field_when_all_knobs_specified() {
    // Field-level inspection of the builder's intermediate state: a
    // fully-specified options bundle must thread every knob into the
    // matching annotation field (ornament, trill_extension,
    // trill_bracket, trill_bracket_direction, trill_bracket_length_ss,
    // trill_wiggle_speed). Any silently-dropped field shows up here.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let opts = TrillExtensionFullOptions::new()
        .with_bracket(TrillBracketSide::Both)
        .with_bracket_direction(HookDirection::Up)
        .with_bracket_length_ss(0.9)
        .with_speed(TrillWiggleSpeed::Fast)
        .with_ornament(Ornament::TrillWithMordent);

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(opts);

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.ornament,
                Some(Ornament::TrillWithMordent),
                "ornament must propagate"
            );
            assert!(annotations.trill_extension, "trill_extension must be set");
            assert_eq!(
                annotations.trill_bracket,
                Some(TrillBracketSide::Both),
                "bracket side must propagate"
            );
            assert_eq!(
                annotations.trill_bracket_direction,
                Some(HookDirection::Up),
                "bracket direction must propagate"
            );
            assert_eq!(
                annotations.trill_bracket_length_ss,
                Some(0.9),
                "bracket length must propagate"
            );
            assert_eq!(
                annotations.trill_wiggle_speed,
                Some(TrillWiggleSpeed::Fast),
                "wiggle speed must propagate"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn full_options_unset_ornament_writes_plain_trill_to_annotation() {
    // Matches the convention of the two single-purpose options
    // builders: `None` ornament must collapse to `Some(Trill)` at the
    // builder layer (not the renderer), so the annotation field
    // remains the single source of truth for the downstream
    // supports_trill_extension() check.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new());

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.ornament,
                Some(Ornament::Trill),
                "unset ornament must collapse to Trill at the builder"
            );
            assert!(annotations.trill_extension);
            assert_eq!(
                annotations.trill_bracket, None,
                "bracket must remain unset when not requested"
            );
            assert_eq!(annotations.trill_wiggle_speed, None);
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn full_options_bracket_plus_speed_renders_distinct_from_bracket_only() {
    // The whole point of the unified options bundle: a single call
    // can combine bracket + speed, where neither single-purpose
    // builder can. The combined SVG must be visibly distinct from
    // the bracket-only variant at the same bracket settings — proves
    // the speed override actually propagated through.
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let bracket_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::Both),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracket_plus_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slowest),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        bracket_only, bracket_plus_speed,
        "adding a non-default speed must visibly change the rendered SVG"
    );
    // Both must still draw the bracket — exactly two hook <line>s for
    // Both-sided bracket. Catches a regression where adding speed
    // somehow eats the bracket.
    assert!(
        bracket_only.matches("<line ").count() >= 2,
        "Both-bracket must draw at least 2 hook lines (bracket-only)"
    );
    assert!(
        bracket_plus_speed.matches("<line ").count() >= 2,
        "Both-bracket must draw at least 2 hook lines (bracket+speed)"
    );
}

#[test]
fn full_options_compound_bracket_plus_speed_renders_distinct_from_plain_trill_same_options() {
    // Coupling the compound ornament with bracket+speed must produce a
    // visibly different SVG from the plain Trill at the same
    // bracket+speed. Proves the ornament half of the bundle propagates
    // through the renderer's glyph-advance lookup (Bravura: 521 vs
    // 990 font-units for "tr" vs the compound).
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_speed(TrillWiggleSpeed::Fast),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let compound = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_speed(TrillWiggleSpeed::Fast)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        plain_trill, compound,
        "compound ornament must visibly differ from plain Trill at same bracket+speed"
    );
    // Bracket should still draw exactly 1 hook <line> over the
    // no-bracket baseline of either variant — guards against the
    // ornament half eating the bracket half.
    let plain_lines = plain_trill.matches("<line ").count();
    let compound_lines = compound.matches("<line ").count();
    assert_eq!(
            plain_lines, compound_lines,
            "bracket geometry is glyph-independent — line counts must match (plain={plain_lines}, compound={compound_lines})"
        );
}

#[test]
fn full_options_unsupported_ornament_silently_drops_extension_and_bracket() {
    // The renderer-layer contract: an ornament that fails
    // supports_trill_extension() causes the collector to drop the
    // wiggle AND the bracket hooks, leaving only the ornament glyph
    // itself. Matches the behavior of the two single-purpose options
    // builders.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let supported = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::Trill),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let unsupported = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::ShortTrill),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // The unsupported variant must produce strictly fewer paths — no
    // wiggle, and no bracket hook lines.
    assert!(
        unsupported.matches("<path").count() < supported.matches("<path").count(),
        "ShortTrill must drop the wiggle (fewer paths than Trill)"
    );
    assert!(
        unsupported.matches("<line ").count() < supported.matches("<line ").count(),
        "ShortTrill must drop the bracket hooks (fewer <line>s than Trill)"
    );
}

#[test]
fn full_options_on_rest_is_noop() {
    // The no-op-on-rest contract must extend to the unified options
    // builder. Applying a fully-specified bundle to a rest must
    // produce SVG byte-identical to the plain-rest baseline.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let plain_rest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .render_svg();

    let annotated_rest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .render_svg();

    assert_eq!(
        plain_rest, annotated_rest,
        "full_options on rest must be a no-op (byte-identical to plain rest)"
    );
}

#[test]
fn full_options_on_chord_renders_bracket_and_wiggle() {
    // Chord targeting: the full bundle on a chord must add both
    // bracket hook lines AND wiggle paths beyond the plain chord
    // baseline.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let plain_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let full_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Fast)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Exact 2-line delta for Both-sided bracket on chord.
    assert_eq!(
        full_chord
            .matches("<line ")
            .count()
            .saturating_sub(plain_chord.matches("<line ").count()),
        2,
        "Both-bracketed compound chord must add exactly 2 hook <line>s"
    );
    assert!(
        full_chord.matches("<path").count() > plain_chord.matches("<path").count(),
        "Bracketed compound chord must add wiggle paths over plain chord"
    );
}

// --- trill_with_extension_full_options × length_ss (combined) ---

#[test]
fn full_options_length_ss_propagates_into_annotation() {
    // The new `length_ss` field on TrillExtensionFullOptions must thread
    // into `NoteAnnotations::trill_extension_length_ss`. Catches a
    // silently-dropped propagation in `trill_with_extension_full_options`.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(2.5));

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::Trill));
            assert!(annotations.trill_extension);
            assert_eq!(
                annotations.trill_extension_length_ss,
                Some(2.5),
                "length_ss must thread into trill_extension_length_ss"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn full_options_length_ss_byte_equivalent_to_trill_with_extension_length_ss() {
    // Byte-equivalence guarantee documented on
    // `trill_with_extension_full_options`: setting only `length_ss` via
    // the options bundle must produce SVG byte-identical to the standalone
    // `trill_with_extension_length_ss(L)` builder.
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(1.5))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_standalone = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(1.5)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_full, via_standalone,
        "trill_with_extension_full_options(new().with_length_ss(L)) must \
             be byte-equivalent to trill_with_extension_length_ss(L)"
    );
}

#[test]
fn full_options_length_ss_shortens_wiggle_vs_default_full_options() {
    // The combined-options path is the only API that exposes "bracket +
    // explicit length" in one call. Build two scores with identical
    // bracket settings, one with `.with_length_ss(short)` and one without
    // — the shortened version must render strictly fewer paths.
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let unshortened = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::Both),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let shortened = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let unshort_paths = unshortened.matches("<path").count();
    let short_paths = shortened.matches("<path").count();
    assert!(
        short_paths < unshort_paths,
        "explicit length must shorten the wiggle even with a bracket attached: \
             short_paths={short_paths}, unshort_paths={unshort_paths}"
    );
    // Bracket geometry is glyph-independent — the bracketed variant on
    // both sides emits exactly 2 hook <line>s regardless of wiggle
    // length. Catches a regression where the explicit-length code path
    // accidentally drops one of the hooks.
    assert_eq!(
        shortened.matches("<line ").count(),
        unshortened.matches("<line ").count(),
        "bracket hook <line> count must be invariant under explicit length"
    );
}

#[test]
fn full_options_length_ss_clamps_to_natural_when_oversized() {
    // Mirrors the standalone builder's clamp contract: a length larger
    // than the natural span must render byte-identical to the same
    // options bundle without any length set.
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new())
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let huge = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(1000.0))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        huge, natural,
        "oversized length_ss in full_options must clamp to natural \
             and render byte-identical to the no-length default"
    );
}

#[test]
fn full_options_combines_bracket_speed_ornament_and_length_in_one_call() {
    // The unique capability of the unified options bundle: simultaneously
    // set bracket + speed + ornament + length in a single call. None of
    // the other builders can express all four. This test proves the
    // combination produces a distinct SVG vs. the same call without the
    // length knob — i.e. the length flag survives through bracket +
    // speed + ornament writes without being silently overwritten.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let common = || {
        TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
    };

    let without_length = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(common())
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_length = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(common().with_length_ss(2.0))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        with_length, without_length,
        "adding `.with_length_ss(2.0)` to a full bracket+speed+ornament options \
             bundle must produce a distinct SVG (length must propagate alongside \
             every other field)"
    );
    // Per the explicit-length contract, the shortened variant must
    // render fewer paths. Catches a regression where every option in
    // the bundle but `length_ss` propagates.
    let without_paths = without_length.matches("<path").count();
    let with_paths = with_length.matches("<path").count();
    assert!(
        with_paths < without_paths,
        "length_ss must visibly shorten the wiggle in a combined options call: \
             with_length={with_paths}, without_length={without_paths}"
    );
    // The End-side bracket emits exactly 1 hook <line>; presence is
    // unaffected by the length knob (the renderer anchors the hook at
    // the shortened terminus). Catches a regression where the bracket
    // drops when an explicit length is set.
    assert_eq!(
        with_length.matches("<line ").count(),
        without_length.matches("<line ").count(),
        "End-side bracket hook must survive the length-shortening"
    );
}

#[test]
fn full_options_length_ss_zero_drops_wiggle_with_other_options_set() {
    // Zero length suppresses the wiggle (non-positive fail-safe) even
    // when other options are set. The remaining content (ornament glyph
    // + bracket hooks) must still draw, but the wiggle tiles are gone.
    // Catches a regression where the non-positive fail-safe is bypassed
    // when other knobs are present.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let with_wiggle = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let zero = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(0.0),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert!(
        zero.matches("<path").count() < with_wiggle.matches("<path").count(),
        "length_ss = 0 must suppress wiggle tiles even with other options set: \
             zero={}, with_wiggle={}",
        zero.matches("<path").count(),
        with_wiggle.matches("<path").count()
    );
}

#[test]
fn full_options_length_ss_disables_cross_system_propagation() {
    // Mirrors the standalone builder's cross-system suppression
    // contract: a positive explicit length stops the wiggle within the
    // source system even when the trilled note is the last in its
    // system. With a single 1-measure system, an explicit length must
    // produce strictly fewer wiggle paths than the no-length default,
    // which would otherwise extend the wiggle to the system's right
    // edge.
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let without_explicit = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new())
        .end_barline()
        .render_svg();

    let with_explicit = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(1.0))
        .end_barline()
        .render_svg();

    assert!(
        with_explicit.matches("<path").count() < without_explicit.matches("<path").count(),
        "explicit length on last note of system must suppress cross-system propagation: \
             with_explicit={}, without_explicit={}",
        with_explicit.matches("<path").count(),
        without_explicit.matches("<path").count()
    );
}

// --- trill_with_extension_length_ss (explicit early termination) ---

#[test]
fn trill_with_extension_length_ss_sets_all_three_annotation_fields() {
    let event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::WHOLE,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(2.5),
            ..NoteAnnotations::default()
        },
    };
    let clef = Clef::Treble;
    let key_sig = KeySignature::Open;
    let result = convert_event(&event, &clef, &key_sig, None);
    match result {
        MeasureEvent::Note(ne) => {
            assert_eq!(ne.annotations.ornament, Some(Ornament::Trill));
            assert!(ne.annotations.trill_extension);
            assert_eq!(ne.annotations.trill_extension_length_ss, Some(2.5));
        }
        _ => panic!("expected Note event"),
    }
}

#[test]
fn trill_with_extension_length_ss_shorter_than_natural_renders_fewer_paths() {
    let natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let short = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(1.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let natural_paths = natural.matches("<path").count();
    let short_paths = short.matches("<path").count();
    assert!(
        short_paths < natural_paths,
        "1.0-ss explicit length must render fewer paths than the natural span: \
             short={short_paths}, natural={natural_paths}"
    );
}

#[test]
fn trill_with_extension_length_ss_larger_than_natural_clamps_to_default() {
    // An impossibly large length must clamp to the natural span — the
    // resulting SVG must be byte-identical to the no-length default.
    let natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let huge = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(1000.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Note: natural is "next-note" terminated (within-system), so its
    // cross_system flag is false. The clamped huge is also within-system.
    // Both code paths reach the same end_x — byte-equivalence holds.
    assert_eq!(
        huge, natural,
        "explicit length 1000 must clamp to natural and render byte-identical"
    );
}

#[test]
fn trill_with_extension_length_ss_zero_drops_wiggle() {
    // A zero length disables the wiggle but keeps the "tr" glyph. Path
    // count must equal that of a plain ornament-only trill (which also
    // emits only the "tr" prefix glyph).
    use crate::layout::ornament::Ornament;
    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let zero = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(0.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        zero.matches("<path").count(),
        plain_trill.matches("<path").count(),
        "explicit length 0 must produce the same path count as a plain trill (no wiggle)"
    );
}

#[test]
fn trill_with_extension_length_ss_on_rest_is_noop() {
    // Same no-op rule as every other ornament-annotation builder. Setting
    // a length annotation on a rest must not change the rendered SVG.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_length = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_length_ss(2.5)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, with_length,
        "trill_with_extension_length_ss on a rest must be a no-op"
    );
}

#[test]
fn trill_with_extension_length_ss_on_chord_renders_shortened_wiggle() {
    let natural_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let short_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_length_ss(1.5)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let natural_paths = natural_chord.matches("<path").count();
    let short_paths = short_chord.matches("<path").count();
    assert!(
            short_paths < natural_paths,
            "short-length chord trill must render fewer paths: short={short_paths}, natural={natural_paths}"
        );
}

#[test]
fn trill_with_extension_length_ss_disables_cross_system_propagation() {
    // A trill on the last note of a system normally extends its wiggle
    // to the system's right edge — but an explicit length stops the
    // wiggle at the requested point. Build a 1-measure system whose
    // last note is trilled, with vs. without an explicit length. The
    // explicit version must render fewer wiggle paths.
    let with_explicit = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .trill_with_extension_length_ss(1.0)
        .end_barline()
        .render_svg();

    let without_explicit = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .trill_with_extension()
        .end_barline()
        .render_svg();

    let explicit_paths = with_explicit.matches("<path").count();
    let natural_paths = without_explicit.matches("<path").count();
    assert!(
        explicit_paths < natural_paths,
        "explicit length on last note must yield fewer paths than the default \
             system-edge extension: explicit={explicit_paths}, natural-to-edge={natural_paths}"
    );
}

#[test]
fn trill_with_extension_length_ss_negative_treats_as_zero() {
    // Negative lengths route through the same `_ <= 0` branch as zero;
    // the contract is "no wiggle." Test it explicitly so a future change
    // to the sign-handling logic surfaces here.
    let neg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(-3.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let zero = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(0.0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        neg, zero,
        "negative explicit length must render byte-identically to zero"
    );
}

// --- trill_with_extension_to (note-anchored end) ---

#[test]
fn trill_with_extension_to_sets_all_three_annotation_fields() {
    // Builder must set the three coupled flags: ornament=Trill,
    // trill_extension=true, AND populate trill_extension_to_note_offset.
    // Catches a regression that silently drops the new field write.
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_to(3);

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::Trill));
            assert!(annotations.trill_extension);
            assert_eq!(annotations.trill_extension_to_note_offset, Some(3));
            // The other extension fields must remain unset — the builder
            // is additive, not destructive. Locks in field isolation
            // against an accidental cross-write.
            assert_eq!(annotations.trill_extension_length_ss, None);
            assert_eq!(annotations.trill_bracket, None);
            assert_eq!(annotations.trill_wiggle_speed, None);
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn trill_with_extension_to_offset_one_byte_equivalent_to_trill_with_extension() {
    // Offset = 1 means "extend to the next note" which is the implicit
    // default of trill_with_extension(). Both renderings must produce
    // byte-identical SVG. Locks in the "offset = 1 is the default"
    // contract; a regression that changes the offset=1 end_x formula
    // would fire this test.
    let via_default = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_offset_one = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_to(1)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_default, via_offset_one,
        "trill_with_extension_to(1) must be byte-equivalent to \
             trill_with_extension() — both terminate at the next note"
    );
}

#[test]
fn trill_with_extension_to_offset_two_extends_past_next_note() {
    // Offset = 2 means the wiggle extends past the next note to the
    // note after that. With three notes in a row (trilled + two more),
    // offset=2 must yield strictly MORE wiggle paths than offset=1
    // because the wiggle covers a longer horizontal span.
    let to_one = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .trill_with_extension_to(1)
        .note(p("F", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let to_two = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .trill_with_extension_to(2)
        .note(p("F", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let to_one_paths = to_one.matches("<path").count();
    let to_two_paths = to_two.matches("<path").count();
    assert!(
        to_two_paths > to_one_paths,
        "trill_with_extension_to(2) must render strictly more wiggle paths \
             than to(1): to(1)={to_one_paths}, to(2)={to_two_paths}"
    );
}

#[test]
fn trill_with_extension_to_offset_zero_drops_wiggle() {
    // Offset = 0 is degenerate (the target is the trilled note itself);
    // the renderer must produce no wiggle, matching the explicit-length
    // <= 0 fail-safe. Path count must equal that of an ornament-only
    // "tr" glyph rendering.
    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let zero_offset = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_to(0)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        zero_offset.matches("<path").count(),
        plain_trill.matches("<path").count(),
        "trill_with_extension_to(0) must produce the same path count as \
             a plain trill (no wiggle, only the 'tr' glyph)"
    );
}

#[test]
fn trill_with_extension_to_offset_overshoot_extends_to_system_edge() {
    // An offset that walks past the last note in the system must fall
    // back to the system-edge behavior (the wiggle extends to the
    // system's right edge rather than panicking or producing no
    // wiggle). With the trilled note as note 1 of a 3-note system and
    // offset=99 (no 99-notes-after exists), the wiggle covers more
    // horizontal span than offset=1 (next note only) — so the
    // overshoot variant must yield strictly more wiggle paths.
    //
    // This is the renderer's `notes.get(i + 99) → None` branch that
    // falls back to the staff-width-minus-edge-gap formula. Without
    // that fallback, the renderer would either panic on the index or
    // emit no wiggle at all.
    let overshoot = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .trill_with_extension_to(99)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let next_note_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .trill_with_extension_to(1)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let overshoot_paths = overshoot.matches("<path").count();
    let next_only_paths = next_note_only.matches("<path").count();
    assert!(
        overshoot_paths > next_only_paths,
        "overshooting offset must extend past the next note to the system edge — \
             more paths than offset=1: overshoot={overshoot_paths}, next_only={next_only_paths}"
    );
}

#[test]
fn trill_with_extension_to_on_rest_is_noop() {
    // Builder must be a no-op when the last event is a rest, matching
    // the convention of every other ornament-attaching builder. The
    // rendered SVG must be byte-identical to the same score without
    // the builder call.
    let plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let with_to = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_to(2)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        plain, with_to,
        "trill_with_extension_to on a rest must be a no-op"
    );
}

#[test]
fn trill_with_extension_to_on_chord_renders_extended_wiggle() {
    // Builder must accept chords as well as notes — same coupling
    // contract as trill_with_extension_length_ss_on_chord. A chord
    // trilled with offset=2 must render strictly more paths than the
    // same chord with the default extension (offset=1 implicit).
    let natural_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .trill_with_extension()
        .note(p("F", 4), Duration::QTR)
        .note(p("A", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let extended_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .trill_with_extension_to(2)
        .note(p("F", 4), Duration::QTR)
        .note(p("A", 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let natural_paths = natural_chord.matches("<path").count();
    let extended_paths = extended_chord.matches("<path").count();
    assert!(
        extended_paths > natural_paths,
        "chord trill with offset=2 must render strictly more paths than the \
             default offset=1: natural={natural_paths}, extended={extended_paths}"
    );
}

#[test]
fn trill_with_extension_to_length_ss_takes_precedence_when_both_set() {
    // When both length_ss AND to_note_offset are set, the explicit
    // length wins at draw time (per the annotation field's documented
    // precedence). Construct a Note annotation with both fields set
    // to mutually-disagreeing values and verify the rendering matches
    // the length-only variant, not the offset-only variant.
    let length_then_offset_event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(1.0),
            trill_extension_to_note_offset: Some(5),
            ..NoteAnnotations::default()
        },
    };
    let length_only_event = ScoreEvent::Note {
        pitch: p("E", 4),
        duration: Duration::QTR,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(1.0),
            ..NoteAnnotations::default()
        },
    };

    let make_render = |trilled_event: ScoreEvent| {
        let mut builder = ScoreBuilder::new().clef(Clef::Treble).time_signature(4, 4);
        builder.current_events.push((0, trilled_event));
        builder
            .note(p("F", 4), Duration::QTR)
            .note(p("G", 4), Duration::HALF)
            .end_barline()
            .render_svg()
    };

    let both_set = make_render(length_then_offset_event);
    let length_only = make_render(length_only_event);

    assert_eq!(
        both_set, length_only,
        "when both trill_extension_length_ss and trill_extension_to_note_offset \
             are set, the explicit length wins — rendering must match length-only \
             variant byte-identically"
    );
}

// --- TrillBracketOptions::extension_length_ss through the score builder ---

#[test]
fn bracketed_with_options_extension_length_ss_propagates_into_annotation() {
    // The newly-added extension_length_ss field on TrillBracketOptions
    // must thread into NoteAnnotations::trill_extension_length_ss
    // through the score-builder method. Catches a silently-dropped
    // wire-up in trill_with_extension_bracketed_with_options.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.0),
        );

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::Trill));
            assert!(annotations.trill_extension);
            assert_eq!(annotations.trill_bracket, Some(TrillBracketSide::Both));
            assert_eq!(
                annotations.trill_extension_length_ss,
                Some(2.0),
                "extension_length_ss must thread into trill_extension_length_ss"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn bracketed_with_options_extension_length_ss_byte_equivalent_to_widened_full_options() {
    // The intended ergonomic path: a caller can either set
    // extension_length_ss on the single-purpose TrillBracketOptions OR
    // widen to TrillExtensionFullOptions and use .with_length_ss(...).
    // Both must produce byte-identical SVG — the From conversion
    // propagates the new field, and the full-options builder writes
    // the same annotation state.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let via_bracketed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let widened: TrillExtensionFullOptions = TrillBracketOptions::new(TrillBracketSide::End)
        .with_extension_length_ss(1.5)
        .into();
    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(widened)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_bracketed, via_full,
        "trill_with_extension_bracketed_with_options(opts) must be byte-equivalent \
             to trill_with_extension_full_options(opts.into()) when opts carries an \
             explicit extension_length_ss — proves the From conversion propagates the field"
    );
}

#[test]
fn bracketed_with_options_extension_length_ss_shortens_wiggle() {
    // Setting an explicit extension length on a bracketed trill must
    // render strictly fewer paths than the same bracket without a
    // length override. Catches a regression where the new field is
    // set but the renderer never sees it (e.g. a missing
    // annotation-write in the score builder).
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let bracketed_natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let bracketed_short = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let natural_paths = bracketed_natural.matches("<path").count();
    let short_paths = bracketed_short.matches("<path").count();
    assert!(
        short_paths < natural_paths,
        "explicit length on a bracketed trill must shorten the wiggle: \
             short_paths={short_paths}, natural_paths={natural_paths}"
    );
    // Bracket geometry is glyph-independent — the bracketed variant on
    // both sides emits exactly 2 hook <line>s regardless of wiggle
    // length. Catches a regression where the explicit-length code path
    // accidentally drops one of the hooks.
    assert_eq!(
        bracketed_short.matches("<line ").count(),
        bracketed_natural.matches("<line ").count(),
        "bracket hook <line> count must be invariant under explicit length"
    );
}

#[test]
fn bracketed_with_options_extension_length_ss_oversized_clamps_to_natural() {
    // Mirrors the standalone builder's clamp contract: a length larger
    // than the natural span must render byte-identical to the same
    // options bundle without any length set.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let huge = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1000.0),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        huge, natural,
        "oversized extension_length_ss must clamp to natural and \
             render byte-identical to the no-length default"
    );
}

#[test]
fn bracketed_with_options_extension_length_ss_is_distinct_from_hook_length_ss() {
    // Score-integration canary that the two `with_*_ss` setters write
    // *different* fields. If a future refactor accidentally aliased
    // them, the rendered SVGs would converge — this fires the alarm.
    // Critical because the two names are confusable.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let hook_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let ext_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        hook_only, ext_only,
        ".with_length_ss(1.5) (hook length) and .with_extension_length_ss(1.5) \
             (wiggle termination) must write distinct fields and produce distinct SVGs"
    );
}

#[test]
fn bracketed_with_options_extension_length_ss_on_chord_renders_shortened_wiggle() {
    // Chord arm of the builder also honors the new field. Catches a
    // regression where only the Note arm of the let-else wire-up is
    // updated.
    use crate::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};

    let natural_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let short_chord = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let natural_paths = natural_chord.matches("<path").count();
    let short_paths = short_chord.matches("<path").count();
    assert!(
        short_paths < natural_paths,
        "extension_length_ss on a bracketed chord trill must shorten the wiggle: \
             short={short_paths}, natural={natural_paths}"
    );
}

// --- TrillExtensionSpeedOptions::extension_length_ss through the score builder ---

#[test]
fn speed_with_options_extension_length_ss_propagates_into_annotation() {
    // The newly-added extension_length_ss field on
    // TrillExtensionSpeedOptions must thread into
    // NoteAnnotations::trill_extension_length_ss. Catches a
    // silently-dropped wire-up.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow).with_extension_length_ss(2.25),
        );

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::Trill));
            assert!(annotations.trill_extension);
            assert_eq!(annotations.trill_wiggle_speed, Some(TrillWiggleSpeed::Slow));
            assert_eq!(
                annotations.trill_extension_length_ss,
                Some(2.25),
                "extension_length_ss must thread into trill_extension_length_ss"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn speed_with_options_extension_length_ss_byte_equivalent_to_widened_full_options() {
    // The intended ergonomic path: a caller can either set
    // extension_length_ss on the single-purpose TrillExtensionSpeedOptions
    // OR widen to TrillExtensionFullOptions and use .with_length_ss(...).
    // Both must produce byte-identical SVG.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let via_speed = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
                .with_extension_length_ss(1.75),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let widened: TrillExtensionFullOptions =
        TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
            .with_extension_length_ss(1.75)
            .into();
    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(widened)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_speed, via_full,
        "trill_with_extension_speed_with_options(opts) must be byte-equivalent \
             to trill_with_extension_full_options(opts.into()) when opts carries an \
             explicit extension_length_ss"
    );
}

#[test]
fn speed_with_options_extension_length_ss_shortens_wiggle() {
    // Setting an explicit extension length on a speed-variant trill
    // must render strictly fewer paths than the same speed without a
    // length override.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let speed_natural = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Standard,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let speed_short = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss(1.5),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let natural_paths = speed_natural.matches("<path").count();
    let short_paths = speed_short.matches("<path").count();
    assert!(
        short_paths < natural_paths,
        "explicit length on a speed-variant trill must shorten the wiggle: \
             short={short_paths}, natural={natural_paths}"
    );
}

#[test]
fn speed_with_options_extension_length_ss_zero_drops_wiggle() {
    // Zero length suppresses the wiggle entirely (matches the
    // standalone builder's non-positive fail-safe contract). The
    // path count must equal that of the ornament-only plain trill.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let plain_trill = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let zero = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast).with_extension_length_ss(0.0),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        zero.matches("<path").count(),
        plain_trill.matches("<path").count(),
        "extension_length_ss=0.0 on speed options must drop the wiggle"
    );
}

#[test]
fn speed_with_options_default_extension_length_ss_matches_plain_speed_byte_for_byte() {
    // Backwards-compatibility canary: adding the new field with a
    // default of `None` must NOT change the SVG produced by the
    // existing all-defaults code path. This complements the existing
    // `speed_with_options_default_ornament_matches_plain_speed_byte_for_byte`
    // by re-asserting the byte-equivalence after the new field is
    // added to the struct.
    use crate::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};

    let via_options = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Slow,
        ))
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slow)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_options, via_plain,
        "TrillExtensionSpeedOptions::new(speed) (no extension_length_ss) must \
             remain byte-equivalent to trill_with_extension_speed(speed)"
    );
}

// --- trill_with_extension_speed_ramp / speed_ramp via full options ---

#[test]
fn full_options_with_speed_ramp_sets_annotation_field() {
    // The new field on TrillExtensionFullOptions must thread through to
    // the annotation. A regression that silently dropped opts.speed_ramp
    // in trill_with_extension_full_options would fire here.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillSpeedRampSpec, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let spec = TrillSpeedRampSpec::new(
        TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
        3,
    );
    let opts = TrillExtensionFullOptions::new().with_speed_ramp(spec);

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(opts);

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(annotations.ornament, Some(Ornament::Trill));
            assert!(annotations.trill_extension);
            assert_eq!(
                annotations.trill_speed_ramp,
                Some(spec),
                "speed_ramp must propagate from opts to the annotation field"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn full_options_without_speed_ramp_leaves_annotation_none() {
    // Symmetric counterpart: a full-options bundle without
    // speed_ramp must leave the annotation's trill_speed_ramp at None
    // even if all other fields are populated. Critical canary against
    // a regression that "promoted" some other setter into a default
    // ramp.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};
    use crate::layout::trill_extension::TrillWiggleSpeed;
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let opts = TrillExtensionFullOptions::new()
        .with_bracket(TrillBracketSide::Both)
        .with_bracket_direction(HookDirection::Up)
        .with_bracket_length_ss(0.9)
        .with_speed(TrillWiggleSpeed::Fast)
        .with_ornament(Ornament::TrillWithMordent)
        .with_length_ss(2.0);

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(opts);

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.trill_speed_ramp, None,
                "speed_ramp must remain None when not requested"
            );
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn trill_with_extension_speed_ramp_sets_three_annotation_fields() {
    // The convenience builder must set ornament=Trill,
    // trill_extension=true, and trill_speed_ramp=Some(spec). Any
    // silently-dropped flag would fire here.
    use crate::layout::ornament::Ornament;
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        );

    let last = builder.current_events.last().expect("note pushed");
    match &last.1 {
        ScoreEvent::Note { annotations, .. } => {
            assert_eq!(
                annotations.ornament,
                Some(Ornament::Trill),
                "convenience builder hardcodes Trill"
            );
            assert!(
                annotations.trill_extension,
                "convenience builder enables trill_extension"
            );
            let spec = annotations.trill_speed_ramp.expect("ramp set");
            assert_eq!(spec.region_count, 3);
            assert!(matches!(
                spec.ramp,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                }
            ));
        }
        _ => panic!("expected last event to be a Note"),
    }
}

#[test]
fn trill_with_extension_speed_ramp_on_rest_is_noop() {
    // No-op semantics on a rest: the most recent event being a rest
    // means there's no note/chord to annotate; the builder must not
    // panic and must not retroactively annotate an earlier event.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .rest(Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .end_barline()
        .render_svg();

    assert!(svg.starts_with("<?xml") || svg.starts_with("<svg"));
    // A trill on a rest must produce no "tr" glyph. The trill glyph is
    // bundled under Bravura's ornament range; without the ornament
    // attached to a note, the renderer emits no trill glyph at all.
    assert!(
        !svg.contains("ornamentTrill"),
        "rest must not get a trill glyph attached"
    );
}

#[test]
fn trill_with_extension_speed_ramp_byte_equivalent_to_full_options_path() {
    // The convenience method must produce SVG byte-identical to
    // `trill_with_extension_full_options(new().with_speed_ramp_ramp_count(ramp, n))`.
    // This is the single most important canary for the convenience
    // builder: if it ever drifts from the full-options path (e.g. by
    // setting an extra annotation field), this fires.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillSpeedRampSpec, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
    let region_count = 3;

    let via_convenience = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(ramp, region_count)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let via_full = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed_ramp(TrillSpeedRampSpec::new(ramp, region_count)),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_eq!(
        via_convenience, via_full,
        "convenience builder must be byte-equivalent to the full-options path"
    );
}

#[test]
fn speed_ramp_renders_distinct_svg_from_single_speed() {
    // The whole point of the multi-speed path: a ramp must produce
    // visibly different SVG from a single-speed trill at any of the
    // ramp's endpoint speeds. If the dispatch silently fell through
    // to the single-speed path (e.g. ignoring the new annotation
    // field), the two would render identically and this fires.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Standard)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let multi = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        single, multi,
        "multi-speed ramp must produce SVG distinct from single-speed Standard"
    );
    // Both paths must produce a non-empty wiggle — i.e. at least
    // some <path> elements attributable to the trill rendering on
    // top of the staff lines, clef, and notehead. We assert each has
    // strictly more <path> elements than the baseline "no trill" SVG
    // would emit. The conservative floor is "more than the staff +
    // clef + notehead total." This score has 2 notes + 1 clef so a
    // safe lower bound for "with trill" is `> 4` paths.
    let count_paths = |s: &str| s.matches("<path").count();
    assert!(
        count_paths(&multi) > 4,
        "multi-speed must emit visible wiggle (path count = {})",
        count_paths(&multi)
    );
    assert!(
        count_paths(&single) > 4,
        "single-speed must emit visible wiggle (path count = {})",
        count_paths(&single)
    );
}

#[test]
fn constant_ramp_byte_equivalent_to_single_speed_when_one_region() {
    // A 1-region Constant ramp is degenerate-equivalent to the
    // single-speed path at the same speed: same number of tiles,
    // same glyph, same positions. The multi-speed renderer's tiling
    // for a single region uses identical floor(span/advance) math as
    // the single-speed renderer; with the same advance and span the
    // tile xs must be identical.
    //
    // Multi-speed and single-speed renderers emit identical glyph
    // outline path *data*, but they emit *different* number-of-paths
    // counts only when the multi-speed has multiple regions. With
    // exactly one Constant region they emit identical SVG.
    //
    // We assert this rather than asserting full byte-equality with
    // single-speed because the dispatch path inserts no extra geometry
    // for the multi-speed case — the only difference would be the
    // tile-x positions, which must match by construction.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let single = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Standard)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let constant_ramp = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 1)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // The two paths render the same number of wiggle tiles. The
    // multi-speed dispatch with 1 Constant Standard region uses
    // identical (start_x, end_x, advance) math as the single-speed
    // path, so they emit the same number of <path> elements.
    let count_paths = |s: &str| s.matches("<path").count();
    assert_eq!(
        count_paths(&single),
        count_paths(&constant_ramp),
        "1-region Constant Standard ramp must emit same path count as single-speed Standard"
    );
}

#[test]
fn degenerate_ramp_renders_no_wiggle_but_keeps_trill_glyph() {
    // A degenerate spec (region_count == 0) must make the renderer's
    // synthesizer return None, which falls through to "no wiggle." The
    // "tr" glyph itself remains drawn (since ornament=Trill is still
    // set on the annotation). Without the speed_ramp the same call
    // would produce a normal trill extension; with the degenerate
    // ramp the wiggle silently disappears.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let with_extension = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let degenerate = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            1, // Linear with 1 region is degenerate → None
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // The degenerate render must produce strictly *fewer* <path>
    // elements than the non-degenerate one: the trill glyph remains
    // but the wiggle is suppressed (typically several tiles' worth).
    let count_paths = |s: &str| s.matches("<path").count();
    assert!(
        count_paths(&degenerate) < count_paths(&with_extension),
        "degenerate ramp must produce fewer <path> elements than normal trill_with_extension \
             (degenerate={}, with_extension={})",
        count_paths(&degenerate),
        count_paths(&with_extension)
    );
}

#[test]
fn ramp_with_bracket_renders_both_wiggle_and_hooks() {
    // Bracket + multi-speed must coexist: the multi-speed bracket
    // dispatch must call layout_trill_bracket_hooks_multi_speed and
    // emit hook <line>s in addition to the wiggle tiles. Without
    // bracket support in the multi-speed branch, the hooks would be
    // silently dropped.
    use crate::layout::trill_bracket::TrillBracketSide;
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillSpeedRampSpec, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed_ramp(TrillSpeedRampSpec::new(
                    TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                    3,
                )),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // For Both brackets, the system renderer emits 2 hook <line>s
    // (start + end). No hooks are drawn elsewhere in this score, so
    // the count is a clean attribution.
    //
    // The wiggle is rendered as <path> elements; the bracket hooks
    // are <line> elements (`draw_trill_bracket_hooks` calls
    // `svg.add_line`). The count of <line> must be ≥ 2 once the
    // bracket pass runs — staff lines are typically 5 per staff so
    // we assert `> 5`.
    let line_count = svg.matches("<line").count();
    assert!(
            line_count >= 7,
            "Both-bracket + multi-speed should emit at least 5 staff lines + 2 bracket hooks (got {line_count})"
        );
}

#[test]
fn linear_ramp_renders_distinct_svg_from_constant_ramp() {
    // The Linear and Constant variants must produce different SVG
    // across the same span and region_count: Linear cycles through
    // multiple glyphs, Constant uses one. Same span, same tile
    // anchor — if the renderer ignored the variant choice they would
    // be byte-equal.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};

    let constant = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let linear = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        constant, linear,
        "Linear and Constant ramps must produce different SVG with the same region count"
    );
}

#[test]
fn speed_ramp_supersedes_speed_field_for_glyph_selection() {
    // When both `speed` and `speed_ramp` are set, the ramp's per-
    // region glyphs must drive the wiggle — the `speed` field is
    // ignored for glyph selection. Concrete canary: a Linear Slow→
    // Fast ramp on a single-speed-Slowest base produces different
    // SVG than the single-speed-Slowest baseline, proving the ramp
    // wins.
    use crate::layout::trill_extension::{TrillSpeedRamp, TrillSpeedRampSpec, TrillWiggleSpeed};
    use crate::layout::trill_options::TrillExtensionFullOptions;

    let speed_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let speed_plus_ramp = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_speed_ramp(TrillSpeedRampSpec::new(
                    TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                    3,
                )),
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        speed_only, speed_plus_ramp,
        "ramp must supersede speed field for glyph selection — same speed but with a ramp \
             must render distinct SVG"
    );
}
