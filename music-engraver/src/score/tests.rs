    use super::*;
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
    fn log2_breve_is_0() {
        assert_eq!(duration_kind_to_log2(DurationKind::Breve), 0);
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
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn sharp_note_suppressed_when_in_sharp_key() {
        // F# in D major (2 sharps) — F is sharped in the key sig, so the
        // accidental is redundant and suppressed.
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, None);
    }

    #[test]
    fn natural_note_in_sharp_key_shows_natural() {
        // F natural in D major — F is sharped in key sig, so we show a natural
        let pitch = Pitch::new(Note::F, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalNatural));
    }

    #[test]
    fn natural_note_in_open_key_no_accidental() {
        let pitch = Pitch::new(Note::C, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, None);
    }

    #[test]
    fn flat_note_shows_flat() {
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn sharp_note_shown_in_flat_key() {
        // F# in Bb major (2 flats) — F is not flatted, so sharp is shown
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn sharp_note_shown_when_not_in_key() {
        // G# in D major (2 sharps: F#, C#) — G is not altered, so show the sharp
        let pitch = Pitch::new(Note::Gis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalSharp));
    }

    #[test]
    fn flat_note_suppressed_when_in_flat_key() {
        // Bb in Bb major (2 flats: Bb, Eb) — B is flatted in key sig, suppress
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, None);
    }

    #[test]
    fn flat_note_shown_in_sharp_key() {
        // Bb in G major (1 sharp) — B is not altered, show the flat
        let pitch = Pitch::new(Note::Bes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(1));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn flat_note_shown_when_not_in_key() {
        // Ab in Bb major (2 flats: Bb, Eb) — A is not flatted, show the flat
        let pitch = Pitch::new(Note::Aes, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Flats(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalFlat));
    }

    #[test]
    fn double_sharp_shows_double_sharp() {
        let pitch = Pitch::new(Note::Fisis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Open);
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
    }

    #[test]
    fn double_sharp_shown_even_in_sharp_key() {
        // F## in D major — double sharps are never in key signatures
        let pitch = Pitch::new(Note::Fisis, 4).unwrap();
        let glyph = should_show_accidental(&pitch, &KeySignature::Sharps(2));
        assert_eq!(glyph, Some(smufl::Glyph::AccidentalDoubleSharp));
    }

    #[test]
    fn double_flat_shown_even_in_flat_key() {
        let pitch = Pitch::new(Note::Beses, 4).unwrap();
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
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .barline()
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::WHOLE)
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
            .note(Pitch::new(Note::C, 3).unwrap(), Duration::QTR)
            .render_svg();
        let svg_bass = ScoreBuilder::new()
            .clef(Clef::Bass)
            .note(Pitch::new(Note::C, 3).unwrap(), Duration::QTR)
            .render_svg();
        // Different clef → different clef glyph path data
        assert_ne!(svg_treble, svg_bass);
    }

    #[test]
    fn dotted_note_produces_dot_glyph() {
        let dotted_qtr = Duration::new(DurationKind::Qtr, 1);
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), dotted_qtr)
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
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH)
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
                .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::D, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::E, 4).unwrap(), Duration::WHOLE)
                .barline()
                .note(Pitch::new(Note::F, 4).unwrap(), Duration::WHOLE)
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
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
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
        let pitch = Pitch::new(Note::B, 4).unwrap();
        let event = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
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
        let result = builder.convert_event(&event, &Clef::Treble);
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
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // Common time uses a single glyph (timeSigCommon), not two digit glyphs.
        // With numeric 4/4 we'd get 2 digit paths; with common we get 1 symbol path.
        let svg_numeric = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // The SVGs should differ because different glyphs are used
        assert_ne!(svg, svg_numeric);
    }

    #[test]
    fn cut_time_uses_cut_common_symbol() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .cut_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        let svg_common = ScoreBuilder::new()
            .clef(Clef::Treble)
            .common_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        // Cut time and common time produce different SVGs
        assert_ne!(svg, svg_common);
    }

    #[test]
    fn cut_time_differs_from_numeric_2_2() {
        let svg_cut = ScoreBuilder::new()
            .clef(Clef::Treble)
            .cut_time()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        let svg_numeric = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(2, 2)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
            .render_svg();
        assert_ne!(svg_cut, svg_numeric);
    }

    #[test]
    fn pending_events_auto_flushed_as_final_barline() {
        // Not calling barline() or end_barline() — events should still render
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .render_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path"));
    }

    #[test]
    fn try_render_svg_returns_ok_for_valid_input() {
        let result = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .barline()
            .rest(Duration::WHOLE)
            .end_barline()
            .try_render_svg()
            .unwrap();
        let svg_direct = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(2))
            .time_signature(4, 4)
            .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
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
        let pitch = Pitch::new(Note::Fis, 4).unwrap();

        // First occurrence: show sharp
        let first = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(first, Some(smufl::Glyph::AccidentalSharp));

        // Record it
        seen.insert(note_key(&pitch), Accidental::Sharp);

        // Second occurrence: suppress (same accidental already shown)
        let second = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(second, None, "repeated sharp should be suppressed");
    }

    #[test]
    fn resolve_accidental_with_tracking_suppresses_repeated_flat() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let pitch = Pitch::new(Note::Bes, 4).unwrap();

        let first = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(first, Some(smufl::Glyph::AccidentalFlat));
        seen.insert(note_key(&pitch), Accidental::Flat);

        let second = resolve_accidental(&pitch, &key, Some(&seen));
        assert_eq!(second, None, "repeated flat should be suppressed");
    }

    #[test]
    fn resolve_accidental_shows_courtesy_natural_after_sharp() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let sharp_pitch = Pitch::new(Note::Fis, 4).unwrap();
        let natural_pitch = Pitch::new(Note::F, 4).unwrap();

        // Show sharp on F#4
        seen.insert(note_key(&sharp_pitch), Accidental::Sharp);

        // F4 natural should show a courtesy natural
        let result = resolve_accidental(&natural_pitch, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalNatural),
            "natural should show after sharp on same letter+octave"
        );
    }

    #[test]
    fn resolve_accidental_shows_courtesy_natural_after_flat() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let flat_pitch = Pitch::new(Note::Bes, 4).unwrap();
        let natural_pitch = Pitch::new(Note::B, 4).unwrap();

        seen.insert(note_key(&flat_pitch), Accidental::Flat);

        let result = resolve_accidental(&natural_pitch, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalNatural),
            "natural should show after flat on same letter+octave"
        );
    }

    #[test]
    fn resolve_accidental_different_octaves_independent() {
        let key = KeySignature::Open;
        let mut seen: AccidentalTracker = HashMap::new();
        let fis4 = Pitch::new(Note::Fis, 4).unwrap();
        let fis5 = Pitch::new(Note::Fis, 5).unwrap();

        // Show sharp on F#4
        seen.insert(note_key(&fis4), Accidental::Sharp);

        // F#5 is a different octave — should still show sharp
        let result = resolve_accidental(&fis5, &key, Some(&seen));
        assert_eq!(
            result,
            Some(smufl::Glyph::AccidentalSharp),
            "sharp on different octave should not be suppressed"
        );
    }

    #[test]
    fn convert_event_tracked_suppresses_repeated_accidental() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let pitch = Pitch::new(Note::Fis, 4).unwrap();

        let ev1 = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let ev2 = ScoreEvent::Note {
            pitch,
            duration: Duration::QTR,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
        };

        let r1 = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);
        let r2 = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);

        match (&r1, &r2) {
            (MeasureEvent::Note(n1), MeasureEvent::Note(n2)) => {
                assert!(
                    n1.accidental.is_some(),
                    "first F#4 should show accidental"
                );
                assert!(
                    n2.accidental.is_none(),
                    "second F#4 should suppress accidental"
                );
            }
            _ => panic!("expected Note events"),
        }
    }

    #[test]
    fn convert_event_tracked_shows_natural_after_sharp() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        let ev_sharp = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let ev_natural = ScoreEvent::Note {
            pitch: Pitch::new(Note::F, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };

        let _ = builder.convert_event_tracked(&ev_sharp, &Clef::Treble, &mut seen);
        let r2 = builder.convert_event_tracked(&ev_natural, &Clef::Treble, &mut seen);

        match r2 {
            MeasureEvent::Note(n) => {
                assert_eq!(
                    n.accidental,
                    Some(smufl::Glyph::AccidentalNatural),
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
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR) // suppressed
            .barline()
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR) // should show (new measure)
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
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // One F#4 alone for comparison
        let svg_single = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
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
    fn effective_accidental_natural_on_altered() {
        let pitch = Pitch::new(Note::F, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Sharps(1));
        assert_eq!(eff, Some(Accidental::Natural));
    }

    #[test]
    fn effective_accidental_natural_on_unaltered() {
        let pitch = Pitch::new(Note::C, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Open);
        assert_eq!(eff, None);
    }

    #[test]
    fn effective_accidental_sharp() {
        let pitch = Pitch::new(Note::Fis, 4).unwrap();
        let eff = effective_accidental(&pitch, &KeySignature::Open);
        assert_eq!(eff, Some(Accidental::Sharp));
    }

    #[test]
    fn note_key_same_letter_different_octave() {
        let p1 = Pitch::new(Note::C, 3).unwrap();
        let p2 = Pitch::new(Note::C, 5).unwrap();
        assert_ne!(note_key(&p1), note_key(&p2));
    }

    #[test]
    fn note_key_same_note_same_octave() {
        let p1 = Pitch::new(Note::C, 4).unwrap();
        let p2 = Pitch::new(Note::C, 4).unwrap();
        assert_eq!(note_key(&p1), note_key(&p2));
    }

    #[test]
    fn note_key_enharmonic_different() {
        // C# and Db are different letters
        let p1 = Pitch::new(Note::Cis, 4).unwrap();
        let p2 = Pitch::new(Note::Des, 4).unwrap();
        assert_ne!(note_key(&p1), note_key(&p2));
    }

    // --- chord support in ScoreBuilder ---

    #[test]
    fn chord_produces_multiple_noteheads() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
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
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .render_svg();
        let svg_chord = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
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
                vec![
                    Pitch::new(Note::Fis, 4).unwrap(),
                    Pitch::new(Note::A, 4).unwrap(),
                ],
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
                vec![
                    Pitch::new(Note::F, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
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
            pitches: vec![
                Pitch::new(Note::E, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
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
    fn convert_event_tracked_chord_suppresses_repeated_accidental() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();

        // First: single F#4 note
        let ev1 = ScoreEvent::Note {
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then: chord with F#4 (should be suppressed) and A4
        let ev2 = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::Fis, 4).unwrap(),
                Pitch::new(Note::A, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Chord(c) => {
                // F#4's accidental should be suppressed (already shown)
                assert_eq!(c.accidentals[0], None, "F#4 accidental should be suppressed");
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
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
            ])
            .end_barline()
            .render_svg();

        assert!(svg.starts_with("<svg"));
        // 4 noteheads + clef + time sig digits
        let path_count = svg.matches("<path ").count();
        assert!(path_count >= 4, "at least 4 paths for noteheads, got {path_count}");
        // Beam polygons (1 primary for all eighth notes)
        let polygon_count = svg.matches("<polygon ").count();
        assert!(polygon_count >= 1, "at least 1 beam polygon, got {polygon_count}");
    }

    #[test]
    fn beam_group_differs_from_individual_eighth_notes() {
        let e4 = Pitch::new(Note::E, 4).unwrap();
        let f4 = Pitch::new(Note::F, 4).unwrap();

        let svg_beamed = ScoreBuilder::new()
            .clef(Clef::Treble)
            .beam_group(vec![
                (e4, Duration::EIGHTH),
                (f4, Duration::EIGHTH),
            ])
            .end_barline()
            .render_svg();

        let svg_flagged = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(e4, Duration::EIGHTH)
            .note(f4, Duration::EIGHTH)
            .end_barline()
            .render_svg();

        // Beamed version should have polygons, flagged should not
        assert!(svg_beamed.matches("<polygon ").count() >= 1, "beamed has polygons");
        assert_eq!(svg_flagged.matches("<polygon ").count(), 0, "flagged has no polygons");
    }

    #[test]
    fn beam_group_convert_event_produces_beam_group_measure_event() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::BeamGroup {
            notes: vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
            ],
        };
        let result = builder.convert_event(&event, &Clef::Treble);
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
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then beam group with F#4 again — should suppress repeated accidental
        let ev2 = ScoreEvent::BeamGroup {
            notes: vec![
                (Pitch::new(Note::Fis, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
            ],
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::BeamGroup(bg) => {
                assert!(bg.notes[0].accidental.is_none(), "F#4 accidental suppressed");
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .tie()
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        // Tie should produce a filled path with stroke="none"
        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert!(tie_count >= 1, "expected at least 1 tie path, got {tie_count}");
        // Should contain Bézier curves
        assert!(svg.contains(" C"), "tie should have cubic Bézier curves");
    }

    #[test]
    fn tie_across_barline_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .tie()
            .barline()
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert!(tie_count >= 1, "tie across barline should produce a tie path");
    }

    #[test]
    fn no_tie_without_tie_call() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "tie after rest should have no effect");
    }

    #[test]
    fn tie_differs_from_untied() {
        let svg_tied = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .tie()
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .end_barline()
            .render_svg();

        let svg_untied = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::HALF)
            .end_barline()
            .render_svg();

        assert_ne!(svg_tied, svg_untied, "tied and untied should produce different SVGs");
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
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.tie_forward, "tie_forward should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    // --- chord tie support ---

    #[test]
    fn tie_after_chord_sets_tie_forward() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.tie_forward, "chord tie_forward should be preserved");
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
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .tie()
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        // 3 notes in the chord → 3 ties
        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 3, "expected 3 ties (one per chord note), got {tie_count}");
    }

    #[test]
    fn chord_tie_differs_from_untied_chord() {
        let build = || {
            ScoreBuilder::new()
                .clef(Clef::Treble)
                .chord(
                    vec![
                        Pitch::new(Note::E, 4).unwrap(),
                        Pitch::new(Note::G, 4).unwrap(),
                    ],
                    Duration::HALF,
                )
        };

        let svg_tied = build()
            .tie()
            .chord(
                vec![
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        let svg_untied = build()
            .chord(
                vec![
                    Pitch::new(Note::E, 4).unwrap(),
                    Pitch::new(Note::G, 4).unwrap(),
                ],
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
                vec![Pitch::new(Note::C, 4).unwrap(), Pitch::new(Note::E, 4).unwrap()],
                Duration::HALF,
            )
            .chord(
                vec![Pitch::new(Note::C, 4).unwrap(), Pitch::new(Note::E, 4).unwrap()],
                Duration::HALF,
            )
            .end_barline()
            .render_svg();

        let tie_count = svg.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "no ties without .tie() call");
    }

    #[test]
    fn convert_event_tracked_chord_preserves_tie_forward() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.tie_forward, "tracked conversion should preserve tie_forward");
            }
            _ => panic!("expected Chord event"),
        }
    }

    // --- dynamics integration ---

    #[test]
    fn dynamic_on_note_produces_extra_path() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Piano)
            .render_svg();
        let svg_f = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .render_svg();
        assert_ne!(svg_p, svg_f, "p and f should produce different SVGs");
    }

    #[test]
    fn dynamic_on_chord_produces_extra_path() {
        let svg_with = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
                Duration::QTR,
            )
            .dynamic(Dynamic::Ff)
            .end_barline()
            .render_svg();
        let svg_without = ScoreBuilder::new()
            .clef(Clef::Treble)
            .chord(
                vec![
                    Pitch::new(Note::C, 4).unwrap(),
                    Pitch::new(Note::E, 4).unwrap(),
                ],
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

        assert_eq!(svg_with, svg_without, "dynamic on rest should have no effect");
    }

    #[test]
    fn convert_event_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Pp),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.dynamic, Some(Dynamic::Pp), "dynamic should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Fff),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.dynamic, Some(Dynamic::Fff), "tracked conversion preserves dynamic");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_chord_preserves_dynamic() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: Some(Dynamic::Sfz),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.dynamic, Some(Dynamic::Sfz), "chord dynamic should be preserved");
            }
            _ => panic!("expected Chord event"),
        }
    }

    #[test]
    fn multiple_dynamics_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Piano)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::B, 4).unwrap(), Duration::QTR)
            .dynamic(Dynamic::Forte)
            .note(Pitch::new(Note::D, 5).unwrap(), Duration::QTR)
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
            .tuplet(3, vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
            ])
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
        assert!(line_count > 10, "expected > 10 lines (staff + stems + bracket), got {line_count}");
    }

    #[test]
    fn tuplet_differs_from_beam_group() {
        let notes = vec![
            (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
            (Pitch::new(Note::F, 4).unwrap(), Duration::EIGHTH),
            (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
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
        assert_eq!(paths_tuplet, paths_beam + 1, "tuplet adds 1 path for number glyph");
    }

    #[test]
    fn tuplet_convert_event_produces_tuplet_group() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::TupletGroup {
            notes: vec![
                (Pitch::new(Note::E, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::B, 4).unwrap(), Duration::EIGHTH),
            ],
            tuplet_number: 3,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
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
            pitch: Pitch::new(Note::Fis, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let _ = builder.convert_event_tracked(&ev1, &Clef::Treble, &mut seen);

        // Then tuplet with F#4 again — should suppress repeated accidental
        let ev2 = ScoreEvent::TupletGroup {
            notes: vec![
                (Pitch::new(Note::Fis, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::A, 4).unwrap(), Duration::EIGHTH),
                (Pitch::new(Note::C, 5).unwrap(), Duration::EIGHTH),
            ],
            tuplet_number: 3,
        };
        let result = builder.convert_event_tracked(&ev2, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::TupletGroup(tg) => {
                assert!(tg.beam_group.notes[0].accidental.is_none(), "F#4 accidental suppressed");
                assert!(tg.beam_group.notes[1].accidental.is_none(), "A4 has no accidental");
            }
            _ => panic!("expected TupletGroup event"),
        }
    }

    #[test]
    fn tuplet_quintuplet_in_score() {
        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .tuplet(5, vec![
                (Pitch::new(Note::C, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::D, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::E, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::F, 4).unwrap(), Duration::SIXTEENTH),
                (Pitch::new(Note::G, 4).unwrap(), Duration::SIXTEENTH),
            ])
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .slur_start()
            .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
            .slur_end()
            .end_barline()
            .render_svg();

        // Should contain a filled slur path (Bézier curves)
        let filled_count = svg.matches(r#"stroke="none""#).count();
        assert!(filled_count >= 1, "expected at least 1 filled slur path, got {filled_count}");
        assert!(svg.contains(" C"), "slur should contain cubic Bézier commands");
    }

    #[test]
    fn slurred_differs_from_unslurred() {
        let slurred = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .slur_start()
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
            .slur_end()
            .end_barline()
            .render_svg();

        let unslurred = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
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
            .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
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
            pitch: Pitch::new(Note::E, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: true,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.slur_start, "slur_start should be preserved");
                assert!(!n.slur_end, "slur_end should be false");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_slur_flags() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let mut seen: AccidentalTracker = HashMap::new();
        let event = ScoreEvent::Note {
            pitch: Pitch::new(Note::G, 4).unwrap(),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: true,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(!n.slur_start, "slur_start should be false");
                assert!(n.slur_end, "slur_end should be preserved");
            }
            _ => panic!("expected Note event"),
        }
    }

    #[test]
    fn chord_slur_preserves_flags() {
        let builder = ScoreBuilder::new().clef(Clef::Treble);
        let event = ScoreEvent::Chord {
            pitches: vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::E, 4).unwrap(),
            ],
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: true,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert!(c.slur_start, "chord slur_start should be preserved");
                assert!(!c.slur_end);
            }
            _ => panic!("expected Chord event"),
        }
    }

    // --- hairpin tests ---

    use crate::layout::hairpin::HairpinType;

    /// Helper to create a Pitch from a note name string and octave.
    fn p(name: &str, octave: u8) -> Pitch {
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
        Pitch::new(note, octave).unwrap()
    }

    #[test]
    fn hairpin_adds_lines_to_svg() {
        let svg_hp = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p("C", 4), Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR)
            .note(p("G", 4), Duration::QTR).hairpin_end()
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
            .note(p("C", 4), Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
            .end_barline()
            .render_svg();

        let svg_d = ScoreBuilder::new()
            .clef(Clef::Treble)
            .note(p("C", 4), Duration::QTR).decresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
            .end_barline()
            .render_svg();

        assert_ne!(svg_c, svg_d, "cresc and decresc should differ");
    }

    #[test]
    fn hairpin_on_rest_is_noop() {
        let svg1 = ScoreBuilder::new()
            .clef(Clef::Treble)
            .rest(Duration::QTR).cresc()
            .note(p("E", 4), Duration::QTR).hairpin_end()
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: Some(HairpinType::Crescendo),
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.hairpin_start, Some(HairpinType::Crescendo));
                assert!(!n.hairpin_end);
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_hairpin_fields() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: true,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let mut seen = HashMap::new();
        let result = builder.convert_event_tracked(&event, &clef, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.hairpin_start.is_none());
                assert!(n.hairpin_end);
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: Some(HairpinType::Decrescendo),
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.hairpin_start, Some(HairpinType::Decrescendo));
                assert!(!c.hairpin_end);
            }
            _ => panic!("expected Chord"),
        }
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
        assert!(svg.contains("<text "), "rehearsal mark should produce a <text> element");
        assert!(svg.contains("<rect "), "boxed rehearsal mark should produce a <rect> element");
        assert!(svg.contains(">A<"), "text content 'A' should appear in the SVG");
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
        assert!(svg.contains(">B<"), "chord rehearsal mark text should appear in SVG");
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
        assert!(svg.contains(">C<"), "plain rehearsal mark text should appear");
        // Plain style should NOT have a rect (only boxed does)
        assert!(!svg.contains("<rect "), "plain rehearsal mark should not have a rect");
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
        assert_eq!(svg_with, svg_without, "rehearsal_mark on rest should have no effect");
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
        assert_ne!(svg_with, svg_without, "rehearsal mark should change the SVG output");
    }

    #[test]
    fn convert_event_preserves_rehearsal_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("A".to_string(), RehearsalStyle::Boxed)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.rehearsal_mark, Some(("A".to_string(), RehearsalStyle::Boxed)));
            }
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn convert_event_tracked_preserves_rehearsal_mark() {
        let builder = ScoreBuilder::new().key_signature(KeySignature::Open);
        let event = ScoreEvent::Note {
            pitch: p("C", 4),
            duration: Duration::QTR,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("B".to_string(), RehearsalStyle::Plain)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let mut seen = HashMap::new();
        let result = builder.convert_event_tracked(&event, &clef, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.rehearsal_mark, Some(("B".to_string(), RehearsalStyle::Plain)));
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: Some(("12".to_string(), RehearsalStyle::Boxed)),
            tempo_mark: None, expression: None,
        };
        let clef = Clef::Treble;
        let result = builder.convert_event(&event, &clef);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.rehearsal_mark, Some(("12".to_string(), RehearsalStyle::Boxed)));
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
        assert!(!svg_with.contains(">Andante<"), "tempo mark on rest should be ignored");
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Text("Largo".into())),
            expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.tempo_mark, Some(TempoMark::Text("Largo".into())));
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Metronome {
                note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 60,
            }),
            expression: None,
        };
        let result = builder.convert_event_tracked(&event, &Clef::Treble, &mut seen);
        match result {
            MeasureEvent::Note(n) => {
                assert!(n.tempo_mark.is_some());
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: Some(TempoMark::Text("Adagio".into())),
            expression: None,
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Chord(c) => {
                assert_eq!(c.tempo_mark, Some(TempoMark::Text("Adagio".into())));
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

        assert!(svg.contains(">dolce<"), "expression text should appear in SVG");
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

        assert!(!svg.contains(">legato<"), "expression on rest should be ignored");
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
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: Some("cantabile".into()),
        };
        let result = builder.convert_event(&event, &Clef::Treble);
        match result {
            MeasureEvent::Note(n) => {
                assert_eq!(n.expression, Some("cantabile".into()));
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

        assert!(svg.contains(">sostenuto<"), "chord expression should appear");
        assert!(svg.contains("italic"), "chord expression should be italic");
    }
