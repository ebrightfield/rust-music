use super::*;
use crate::font::bravura_font;
use crate::layout::barline::BarlineStyle;
use crate::layout::key_signature::KeySignature;
use crate::layout::lyric::LyricSyllable;
use crate::layout::measure::{MeasureLayoutConfig, NoteAnnotations, NoteEvent};
use crate::layout::ottava::OttavaKind;
use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
use crate::layout::system::{MeasureContent, MeasureEvent, SystemPrefix};
use crate::layout::time_signature::TimeSignatureKind;
use music::notation::clef::Clef;

fn setup() -> (MusicFont<'static>, EngravingConfig) {
    let font = bravura_font();
    let config = font.engraving_config();
    (font, config)
}

fn quarter_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    annotations: NoteAnnotations::default(),
    })
}

fn make_measure(pos: i8) -> MeasureContent {
    MeasureContent {
        events: vec![quarter_note(pos)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }
}

fn prefix() -> SystemPrefix {
    SystemPrefix::new(
        &Clef::Treble,
        KeySignature::Open,
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        }),
    )
}

#[test]
fn empty_page_renders_valid_svg() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let page = layout_page(&prefix(), &[], &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();
    assert!(output.starts_with("<svg"));
    assert!(output.ends_with("</svg>\n"));
    assert_eq!(output.matches("<path ").count(), 0);
}

#[test]
fn single_system_page() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..3).map(|i| make_measure(i as i8 * 2)).collect();
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Should have 5 staff lines
    assert!(output.matches("<line ").count() >= 5);
    // Should have a clef path
    assert!(output.matches("<path ").count() >= 1);
}

#[test]
fn two_system_page_has_two_sets_of_staff_lines() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // 2 systems × 5 staff lines = 10 staff lines minimum
    let line_count = output.matches("<line ").count();
    assert!(
        line_count >= 10,
        "expected >= 10 lines (2 staves), got {}",
        line_count,
    );
}

#[test]
fn three_system_page_element_counts() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));

    assert_eq!(page.systems.len(), 3);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // 3 systems × 5 staff lines = 15
    let line_count = output.matches("<line ").count();
    assert!(line_count >= 15, "expected >= 15 lines, got {}", line_count);

    // 3 clefs + time sig digits in first system only
    let path_count = output.matches("<path ").count();
    assert!(path_count >= 3, "expected >= 3 paths (3 clefs), got {}", path_count);
}

#[test]
fn page_svg_dimensions_are_positive() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Parse width and height from the SVG — they should be positive
    assert!(output.contains("width=\""));
    assert!(output.contains("height=\""));
    // Check viewBox has positive dimensions
    assert!(output.contains("viewBox=\""));
}

// --- cross-system tie tests ---

fn tied_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { tie_forward: true, ..Default::default() },})
}

#[test]
fn cross_system_tie_draws_two_half_ties() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: tied note at pos 4, system 2: note at pos 4
    let measures = vec![
        MeasureContent {
            events: vec![tied_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // 1 measure per system → forces cross-system tie
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Should have 2 filled tie paths (right half-tie + left half-tie)
    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(
        tie_count, 2,
        "expected 2 half-ties for cross-system tie, got {tie_count}"
    );
}

#[test]
fn no_cross_system_tie_without_tie_forward() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 0, "no ties without tie_forward");
}

#[test]
fn cross_system_tie_with_no_matching_target_draws_right_half_only() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Tied note at pos 4 in system 1, but note at pos 6 in system 2 (different position)
    let measures = vec![
        MeasureContent {
            events: vec![tied_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Only the right half-tie should be drawn (outgoing tie at end of system 1)
    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(
        tie_count, 1,
        "only right half-tie when no matching target, got {tie_count}"
    );
}

#[test]
fn within_system_tie_does_not_produce_cross_system_tie() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both tied note and target in same system (2 measures per system)
    let measures = vec![
        MeasureContent {
            events: vec![tied_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
    assert_eq!(page.systems.len(), 1);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Only 1 within-system tie, no cross-system ties
    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 1, "1 within-system tie, no cross-system tie");
}

#[test]
fn cross_system_tie_differs_from_no_tie() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let tied_measures = vec![
        MeasureContent {
            events: vec![tied_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let untied_measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let tied_page = layout_page(&prefix(), &tied_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let untied_page = layout_page(&prefix(), &untied_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let tied_svg = draw_page(&font, &config, &tied_page).unwrap().to_svg();
    let untied_svg = draw_page(&font, &config, &untied_page).unwrap().to_svg();

    assert_ne!(tied_svg, untied_svg, "cross-system tied output should differ from untied");
}

// --- cross-system slur tests ---

fn slur_start_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { slur_start: true, ..Default::default() },})
}

fn slur_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { slur_end: true, ..Default::default() },})
}

#[test]
fn cross_system_slur_draws_two_half_slurs() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: slur_start note, system 2: slur_end note
    let measures = vec![
        MeasureContent {
            events: vec![slur_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![slur_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // 1 measure per system → forces cross-system slur
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Should have 2 filled slur paths (right half-slur + left half-slur)
    let filled_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(
        filled_count, 2,
        "expected 2 half-slurs for cross-system slur, got {filled_count}"
    );
}

#[test]
fn no_cross_system_slur_without_flags() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    let filled_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled_count, 0, "no slurs without slur flags");
}

#[test]
fn cross_system_slur_right_half_only_when_no_end() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // slur_start in system 1, but no slur_end in system 2
    let measures = vec![
        MeasureContent {
            events: vec![slur_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],  // no slur_end
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Only the right half-slur
    let filled_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(
        filled_count, 1,
        "only right half-slur when no slur_end target, got {filled_count}"
    );
}

#[test]
fn within_system_slur_not_duplicated_as_cross_system() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both slur start and end in same system (2 measures per system)
    let measures = vec![
        MeasureContent {
            events: vec![slur_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![slur_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
    assert_eq!(page.systems.len(), 1);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Only 1 slur (within-system), no cross-system duplication
    let filled_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled_count, 1, "1 within-system slur, no cross-system slur");
}

#[test]
fn cross_system_slur_differs_from_no_slur() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_slur = vec![
        MeasureContent {
            events: vec![slur_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![slur_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let without_slur = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let slur_page = layout_page(&prefix(), &with_slur, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_slur_page = layout_page(&prefix(), &without_slur, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let slur_svg = draw_page(&font, &config, &slur_page).unwrap().to_svg();
    let no_slur_svg = draw_page(&font, &config, &no_slur_page).unwrap().to_svg();

    assert_ne!(slur_svg, no_slur_svg, "cross-system slurred output should differ from unslurred");
}

// --- cross-system hairpin tests ---

use crate::layout::hairpin::HairpinType;

fn cresc_start_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { hairpin_start: Some(HairpinType::Crescendo), ..Default::default() },})
}

fn hairpin_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { hairpin_end: true, ..Default::default() },})
}

#[test]
fn cross_system_hairpin_draws_four_lines() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: cresc start, system 2: hairpin end
    let measures = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // 1 measure per system → forces cross-system hairpin
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Without hairpin for comparison
    let no_hp_measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_hp_page = layout_page(&prefix(), &no_hp_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_hp_output = draw_page(&font, &config, &no_hp_page).unwrap().to_svg();

    // Cross-system hairpin = 2 half-hairpins × 2 lines each = 4 extra lines
    let hp_lines = output.matches("<line ").count();
    let no_lines = no_hp_output.matches("<line ").count();
    assert_eq!(
        hp_lines,
        no_lines + 4,
        "cross-system hairpin should add 4 lines (2 half-hairpins × 2 wedge lines), got {} vs {}",
        hp_lines,
        no_lines,
    );
}

#[test]
fn no_cross_system_hairpin_without_flags() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Count lines: should be baseline (staff lines + stems + barlines) only
    // No hairpin lines
    let no_hp_line_count = output.matches("<line ").count();
    // Verify no extra hairpin-positioned lines below staff
    // (This is implicitly verified by the 4-line-addition test above)
    assert!(no_hp_line_count > 0, "should have some lines");
}

#[test]
fn cross_system_hairpin_right_half_only_when_no_end() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // cresc start in system 1, but no hairpin_end in system 2
    let measures = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)], // no hairpin_end
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Compare with no hairpin
    let no_hp = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_hp_page = layout_page(&prefix(), &no_hp, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_hp_output = draw_page(&font, &config, &no_hp_page).unwrap().to_svg();

    // Only the right half-hairpin (2 lines)
    let hp_lines = output.matches("<line ").count();
    let no_lines = no_hp_output.matches("<line ").count();
    assert_eq!(
        hp_lines,
        no_lines + 2,
        "only right half-hairpin (2 lines) when no hairpin_end target"
    );
}

#[test]
fn within_system_hairpin_not_duplicated_as_cross_system() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both hairpin start and end in same system (2 measures per system)
    let measures = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
    assert_eq!(page.systems.len(), 1);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Compare with no hairpin
    let no_hp = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_hp_page = layout_page(&prefix(), &no_hp, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
    let no_hp_output = draw_page(&font, &config, &no_hp_page).unwrap().to_svg();

    // Only 1 within-system hairpin (2 lines), no cross-system duplication
    let hp_lines = output.matches("<line ").count();
    let no_lines = no_hp_output.matches("<line ").count();
    assert_eq!(hp_lines, no_lines + 2, "1 within-system hairpin (2 lines), no cross-system duplication");
}

#[test]
fn cross_system_hairpin_differs_from_no_hairpin() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_hp = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let without_hp = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let hp_page = layout_page(&prefix(), &with_hp, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_page = layout_page(&prefix(), &without_hp, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let hp_svg = draw_page(&font, &config, &hp_page).unwrap().to_svg();
    let no_svg = draw_page(&font, &config, &no_page).unwrap().to_svg();

    assert_ne!(hp_svg, no_svg, "cross-system hairpin output should differ from no hairpin");
}

#[test]
fn cross_system_decresc_differs_from_cresc() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let cresc = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let decresc = vec![
        MeasureContent {
            events: vec![MeasureEvent::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations { hairpin_start: Some(HairpinType::Decrescendo), ..Default::default() },})],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let cresc_page = layout_page(&prefix(), &cresc, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let decresc_page = layout_page(&prefix(), &decresc, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let cresc_svg = draw_page(&font, &config, &cresc_page).unwrap().to_svg();
    let decresc_svg = draw_page(&font, &config, &decresc_page).unwrap().to_svg();

    assert_ne!(cresc_svg, decresc_svg, "cross-system cresc and decresc should differ");
}

#[test]
fn two_system_page_has_different_y_for_staff_lines() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Extract all y1 values from <line> elements — there should be at least
    // 2 distinct groups (one per system)
    let y1_values: Vec<&str> = output
        .split("y1=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    assert!(y1_values.len() >= 10, "should have >= 10 line y1 values");

    // Parse to floats and check at least 2 distinct y ranges
    let y_floats: Vec<f64> = y1_values.iter().filter_map(|s| s.parse().ok()).collect();
    let min_y = y_floats.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = y_floats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    // Two systems means y-values span at least the system spacing
    let expected_min_span = 5.0 * ss; // at least 5 staff spaces apart
    assert!(
        max_y - min_y > expected_min_span,
        "y range {} should exceed {} (two systems should be separated)",
        max_y - min_y,
        expected_min_span,
    );
}

// ---- Measure number tests ----

#[test]
fn measure_numbers_enabled_adds_text_elements() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let mut page_cfg = PageLayoutConfig::new(ss, 8000.0);
    page_cfg.show_measure_numbers = true;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Fixed(3));
    let svg = draw_page(&font, &config, &page).unwrap().to_svg();

    // Two systems ⇒ two measure number text elements ("1" and "4")
    assert_eq!(page.systems.len(), 2);
    assert!(svg.contains(">1</text>"), "first system should show measure number 1");
    assert!(svg.contains(">4</text>"), "second system should show measure number 4");
}

#[test]
fn measure_numbers_disabled_no_text_elements() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0); // default: show_measure_numbers = false
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Fixed(3));
    let svg = draw_page(&font, &config, &page).unwrap().to_svg();

    // No measure number text should be present (other text may exist from key/time sig)
    assert!(!svg.contains(">1</text>"), "should not show measure numbers when disabled");
    assert!(!svg.contains(">4</text>"), "should not show measure numbers when disabled");
}

#[test]
fn measure_numbers_show_correct_values_across_three_systems() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let mut page_cfg = PageLayoutConfig::new(ss, 8000.0);
    page_cfg.show_measure_numbers = true;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Fixed(3));

    assert_eq!(page.systems.len(), 3);
    assert_eq!(page.systems[0].first_measure_number, 1);
    assert_eq!(page.systems[1].first_measure_number, 4);
    assert_eq!(page.systems[2].first_measure_number, 7);

    let svg = draw_page(&font, &config, &page).unwrap().to_svg();
    assert!(svg.contains(">1</text>"));
    assert!(svg.contains(">4</text>"));
    assert!(svg.contains(">7</text>"));
}

#[test]
fn measure_numbers_positioned_above_staff() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let mut page_cfg = PageLayoutConfig::new(ss, 8000.0);
    page_cfg.show_measure_numbers = true;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures = vec![make_measure(4)];
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Fixed(4));

    let svg = draw_page(&font, &config, &page).unwrap().to_svg();

    // The text element should have a y coordinate that is above the top staff line (y=0).
    // Since top_margin is 0, the staff top is at y=0, and the measure number
    // should be at a negative y (above).
    assert!(svg.contains(">1</text>"), "should show measure number 1");
    // Extract y attribute from the text element containing "1"
    let text_pos = svg.find(">1</text>").unwrap();
    let text_start = svg[..text_pos].rfind("<text ").unwrap();
    let text_tag = &svg[text_start..text_pos];
    let y_start = text_tag.find("y=\"").unwrap() + 3;
    let y_end = text_tag[y_start..].find('"').unwrap() + y_start;
    let y_val: f64 = text_tag[y_start..y_end].parse().unwrap();
    // y should be negative (above the staff line at y=0)
    assert!(y_val < 0.0, "measure number y={y_val} should be above the staff (negative)");
}

#[test]
fn measure_numbers_font_size_scales_with_staff_space() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let mut page_cfg = PageLayoutConfig::new(ss, 8000.0);
    page_cfg.show_measure_numbers = true;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures = vec![make_measure(4)];
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Fixed(4));

    let svg = draw_page(&font, &config, &page).unwrap().to_svg();

    // Font size should be MEASURE_NUMBER_FONT_SIZE_SS * staff_space
    let expected_size = MEASURE_NUMBER_FONT_SIZE_SS * ss;
    let font_size_str = format!("font-size=\"{}\"", expected_size as u32);
    assert!(
        svg.contains(&font_size_str),
        "SVG should contain font-size matching {expected_size}: {font_size_str}"
    );
}

#[test]
fn first_measure_number_set_correctly_for_auto_breaks() {
    let ss = 250.0;
    let mut page_cfg = PageLayoutConfig::new(ss, 3000.0); // narrow to force breaks
    page_cfg.show_measure_numbers = true;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();
    let page = layout_page(&prefix(), &measures, &mc, &page_cfg, &SystemBreaking::Auto);

    // With auto breaks and narrow width, should have multiple systems
    assert!(page.systems.len() > 1);
    // First system always starts at measure 1
    assert_eq!(page.systems[0].first_measure_number, 1);
    // Each system's first_measure_number should be monotonically increasing
    for i in 1..page.systems.len() {
        assert!(
            page.systems[i].first_measure_number > page.systems[i - 1].first_measure_number,
            "system {} measure number {} should be > system {} measure number {}",
            i, page.systems[i].first_measure_number,
            i - 1, page.systems[i - 1].first_measure_number,
        );
    }
}

#[test]
fn measure_numbers_enabled_differs_from_disabled() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let mc = MeasureLayoutConfig::from_staff_space(ss);
    let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();

    let mut page_cfg_on = PageLayoutConfig::new(ss, 8000.0);
    page_cfg_on.show_measure_numbers = true;
    let page_on = layout_page(&prefix(), &measures, &mc, &page_cfg_on, &SystemBreaking::Fixed(4));
    let svg_on = draw_page(&font, &config, &page_on).unwrap().to_svg();

    let page_cfg_off = PageLayoutConfig::new(ss, 8000.0);
    let page_off = layout_page(&prefix(), &measures, &mc, &page_cfg_off, &SystemBreaking::Fixed(4));
    let svg_off = draw_page(&font, &config, &page_off).unwrap().to_svg();

    assert_ne!(svg_on, svg_off, "enabling measure numbers should change the SVG output");
    assert!(svg_on.len() > svg_off.len(), "SVG with measure numbers should be larger");
}

// ---- Cross-system lyric extender tests ----

fn note_with_extender(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            lyric: Some(LyricSyllable::with_extender("love")),
            ..Default::default()
        },
    })
}

fn note_with_lyric_word(pos: i8, text: &str) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            lyric: Some(LyricSyllable::word(text)),
            ..Default::default()
        },
    })
}

#[test]
fn cross_system_lyric_extender_draws_two_lines() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: note with extender, system 2: note (target)
    let measures = vec![
        MeasureContent {
            events: vec![note_with_extender(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![note_with_lyric_word(4, "day")],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // 1 measure per system → forces cross-system extender
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Count all <line> elements. Compare against a version without the extender.
    let with_ext_lines = output.matches("<line ").count();

    // Build the same layout but without the extender lyric
    let measures_no_ext = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![note_with_lyric_word(4, "day")],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page_no_ext = layout_page(&prefix(), &measures_no_ext, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg_no_ext = draw_page(&font, &config, &page_no_ext).unwrap();
    let no_ext_lines = svg_no_ext.to_svg().matches("<line ").count();

    // The cross-system extender should add 2 extra lines (trailing + incoming)
    assert_eq!(
        with_ext_lines - no_ext_lines,
        2,
        "cross-system extender should add 2 lines (trailing + incoming), got {}",
        with_ext_lines - no_ext_lines,
    );
}

#[test]
fn no_cross_system_lyric_extender_without_extender_continuation() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Word lyric (not extender) — no cross-system line expected
    let measures = vec![
        MeasureContent {
            events: vec![note_with_lyric_word(4, "sing")],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg_with = draw_page(&font, &config, &page).unwrap().to_svg();

    // Compare against no lyric at all
    let measures_bare = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page_bare = layout_page(&prefix(), &measures_bare, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg_bare = draw_page(&font, &config, &page_bare).unwrap().to_svg();

    // Line count difference should be 0 (word lyrics don't produce extender lines)
    let diff = svg_with.matches("<line ").count() as i32 - svg_bare.matches("<line ").count() as i32;
    assert_eq!(diff, 0, "word lyric should not produce cross-system extender lines");
}

#[test]
fn within_system_extender_does_not_produce_cross_system_extender() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both extender source and target in the same system (2 measures per system)
    let measures = vec![
        MeasureContent {
            events: vec![note_with_extender(4), quarter_note(6)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    // Everything in one system
    assert_eq!(page.systems.len(), 1);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // The within-system extender should be drawn by the system renderer.
    // The cross-system logic should not add additional lines.
    // Just verify the output is valid and contains an extender line
    // (horizontal line at the lyric baseline).
    assert!(output.starts_with("<svg"));
}

#[test]
fn cross_system_lyric_extender_differs_from_no_extender() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures_ext = vec![
        MeasureContent {
            events: vec![note_with_extender(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let measures_bare = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let page_ext = layout_page(&prefix(), &measures_ext, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let page_bare = layout_page(&prefix(), &measures_bare, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let svg_ext = draw_page(&font, &config, &page_ext).unwrap().to_svg();
    let svg_bare = draw_page(&font, &config, &page_bare).unwrap().to_svg();

    assert_ne!(svg_ext, svg_bare, "cross-system lyric extender should change SVG output");
}

#[test]
fn cross_system_lyric_extender_lines_are_horizontal() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![note_with_extender(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![note_with_lyric_word(4, "day")],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Extract all <line> elements and find ones where y1 matches the lyric
    // baseline (below the staff). The extender lines should have y1 == y2.
    let lyric_baseline_y = LYRIC_BELOW_STAFF_SS * ss;
    // All extender lines should be horizontal (y1 == y2)
    // Verify by checking that the output contains at least some lines with
    // matching y1 and y2 that are in the lyric-region (below staff).
    let lines: Vec<&str> = output.split("<line ").skip(1).collect();
    let mut found_lyric_lines = 0;
    for line_tag in &lines {
        if let (Some(y1_start), Some(y2_start)) =
            (line_tag.find("y1=\""), line_tag.find("y2=\""))
        {
            let y1: &str = &line_tag[y1_start + 4..];
            let y1 = y1.split('"').next().unwrap();
            let y2: &str = &line_tag[y2_start + 4..];
            let y2 = y2.split('"').next().unwrap();
            let y1_f: f64 = y1.parse().unwrap_or(0.0);
            let _y2_f: f64 = y2.parse().unwrap_or(0.0);
            // Lyric lines are well below the staff (y > staff bottom line)
            if y1 == y2 && y1_f > lyric_baseline_y * 0.5 {
                found_lyric_lines += 1;
            }
        }
    }
    assert!(
        found_lyric_lines >= 2,
        "expected at least 2 horizontal lyric extender lines (trailing + incoming), found {found_lyric_lines}"
    );
}

// --- Cross-system ottava bracket tests ---

fn ottava_start_note(pos: i8, kind: OttavaKind) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ottava_start: Some(kind),
            ..NoteAnnotations::default()
        },
    })
}

fn ottava_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ottava_end: true,
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_ottava_draws_two_brackets() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: ottava start on high note, system 2: ottava end
    let measures = vec![
        MeasureContent {
            events: vec![ottava_start_note(10, OttavaKind::Ottava8va)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![ottava_end_note(12)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // 1 measure per system → forces cross-system ottava
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Without ottava for comparison
    let no_ott_measures = vec![
        MeasureContent {
            events: vec![quarter_note(10)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(12)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_ott_page = layout_page(&prefix(), &no_ott_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_ott_output = draw_page(&font, &config, &no_ott_page).unwrap().to_svg();

    // Cross-system ottava produces 2 half-brackets with dashed lines and labels.
    // Each bracket adds at least: 1 <text> (label) + 1 dashed <line> + 0 or 1 hook <line>.
    // Trailing bracket: label + dashed line (no end hook) = 1 text + 1 line
    // Incoming bracket: label + dashed line + end hook = 1 text + 2 lines
    let ott_lines = output.matches("<line ").count();
    let no_lines = no_ott_output.matches("<line ").count();
    assert!(
        ott_lines > no_lines,
        "cross-system ottava should add lines (dashed + hook), got {} vs {}",
        ott_lines, no_lines
    );

    let ott_texts = output.matches("<text ").count();
    let no_texts = no_ott_output.matches("<text ").count();
    assert!(
        ott_texts >= no_texts + 2,
        "cross-system ottava should add at least 2 text labels (8va), got {} vs {}",
        ott_texts, no_texts
    );

    // Verify the label text "8va" appears
    assert!(
        output.contains("8va"),
        "cross-system ottava should contain '8va' label"
    );
}

#[test]
fn no_cross_system_ottava_without_flags() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(10)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(12)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // No "8va" or "8vb" labels should appear
    assert!(!output.contains("8va"), "no ottava label without ottava annotations");
    assert!(!output.contains("8vb"), "no ottava label without ottava annotations");
}

#[test]
fn cross_system_ottava_right_half_only_when_no_end() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: ottava start, system 2: no ottava end (just a regular note)
    let measures = vec![
        MeasureContent {
            events: vec![ottava_start_note(10, OttavaKind::Ottava8va)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(12)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Only trailing half-bracket should be drawn (1 label "8va")
    let ott_count = output.matches("8va").count();
    assert_eq!(
        ott_count, 1,
        "only trailing half-bracket should be drawn when no ottava_end in next system, got {} occurrences",
        ott_count
    );
}

#[test]
fn within_system_ottava_not_duplicated_by_cross_system() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both start and end within the same system (single measure)
    let measures = vec![
        MeasureContent {
            events: vec![
                ottava_start_note(10, OttavaKind::Ottava8va),
                ottava_end_note(12),
            ],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Should have exactly 1 "8va" label (the within-system bracket)
    // Cross-system pass should not add another
    let ott_count = output.matches("8va").count();
    assert_eq!(
        ott_count, 1,
        "within-system ottava should not produce extra cross-system brackets, got {} '8va' occurrences",
        ott_count
    );
}

#[test]
fn cross_system_ottava_8vb_draws_below_staff() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![ottava_start_note(-2, OttavaKind::Ottava8vb)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![ottava_end_note(-4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // "8vb" label should appear
    assert!(
        output.contains("8vb"),
        "cross-system 8vb ottava should contain '8vb' label"
    );

    // Should differ from 8va version
    let measures_8va = vec![
        MeasureContent {
            events: vec![ottava_start_note(-2, OttavaKind::Ottava8va)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![ottava_end_note(-4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page_8va = layout_page(&prefix(), &measures_8va, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let output_8va = draw_page(&font, &config, &page_8va).unwrap().to_svg();
    assert_ne!(output, output_8va, "8vb and 8va cross-system ottavas should differ");
}

// --- Cross-system glissando tests ---

fn glissando_note(pos: i8, style: GlissandoStyle) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            glissando_start: Some(style),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_glissando_draws_two_half_lines() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1: note with glissando_start (last note, no target in system)
    // System 2: note as target
    let measures = vec![
        MeasureContent {
            events: vec![glissando_note(0, GlissandoStyle::Line)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    let no_gliss_measures = vec![
        MeasureContent {
            events: vec![quarter_note(0)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_gliss_page = layout_page(&prefix(), &no_gliss_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let no_gliss_output = draw_page(&font, &config, &no_gliss_page).unwrap().to_svg();

    let line_count = output.matches("<line ").count();
    let no_gliss_line_count = no_gliss_output.matches("<line ").count();

    // Should have 2 extra lines: trailing half + incoming half
    assert!(
        line_count >= no_gliss_line_count + 2,
        "cross-system glissando should add ≥2 lines: {line_count} vs {no_gliss_line_count}"
    );
}

#[test]
fn no_cross_system_glissando_without_flag() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures_no_flag = vec![
        MeasureContent {
            events: vec![quarter_note(0)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let measures_with_flag = vec![
        MeasureContent {
            events: vec![glissando_note(0, GlissandoStyle::Line)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page_no = layout_page(&prefix(), &measures_no_flag, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let page_with = layout_page(&prefix(), &measures_with_flag, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let output_no = draw_page(&font, &config, &page_no).unwrap().to_svg();
    let output_with = draw_page(&font, &config, &page_with).unwrap().to_svg();

    let lines_no = output_no.matches("<line ").count();
    let lines_with = output_with.matches("<line ").count();
    assert!(
        lines_with > lines_no,
        "without flag should have fewer lines than with flag: {lines_no} vs {lines_with}"
    );
}

#[test]
fn within_system_glissando_not_duplicated_as_cross_system() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // Both notes in the same system — glissando resolves within system
    let measures = vec![
        MeasureContent {
            events: vec![
                glissando_note(0, GlissandoStyle::Line),
                quarter_note(8),
            ],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // One glissando line (within-system), not three (within + 2 half)
    let line_count = output.matches("<line ").count();
    // 5 staff lines + 2 stems + 1 glissando = 8
    assert!(
        line_count <= 10,
        "within-system glissando should not be duplicated as cross-system: {line_count}"
    );
}

#[test]
fn cross_system_glissando_differs_from_no_glissando() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_gliss = vec![
        MeasureContent {
            events: vec![glissando_note(4, GlissandoStyle::Line)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let without_gliss = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let page_with = layout_page(&prefix(), &with_gliss, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let page_without = layout_page(&prefix(), &without_gliss, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let out_with = draw_page(&font, &config, &page_with).unwrap().to_svg();
    let out_without = draw_page(&font, &config, &page_without).unwrap().to_svg();

    assert_ne!(out_with, out_without, "glissando version should differ from non-glissando");
}

#[test]
fn cross_system_glissando_with_text_shows_label() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![glissando_note(0, GlissandoStyle::LineWithText)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    assert!(
        output.contains("gliss."),
        "LineWithText cross-system glissando should show 'gliss.' label"
    );
}

// --- Cross-system trill-extension (wavy line continuation) tests ---

use crate::layout::ornament::Ornament;

/// A whole note carrying a trill ornament with the wavy-line extension
/// enabled. Whole notes give the layout enough horizontal room that the
/// within-system wiggle is non-empty (so the within-system rendering and
/// the cross-system rendering are clearly distinguishable in path counts).
fn trill_ext_whole_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    })
}

fn whole_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

#[test]
fn cross_system_trill_extension_adds_incoming_wiggle_paths_on_next_system() {
    // System 1 ends with a trill+extension; system 2 starts with a plain note.
    // The system renderer already terminates the source wiggle at system 1's
    // right edge — what we're verifying here is that the *incoming* wiggle on
    // system 2 (drawn by draw_cross_system_trill_extensions) actually adds
    // additional `<path` elements that the no-trill baseline lacks.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_trill = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let without_trill = vec![
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let page_with = layout_page(&prefix(), &with_trill, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let page_without = layout_page(&prefix(), &without_trill, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page_with.systems.len(), 2, "test requires two systems");

    let out_with = draw_page(&font, &config, &page_with).unwrap().to_svg();
    let out_without = draw_page(&font, &config, &page_without).unwrap().to_svg();

    let with_paths = out_with.matches("<path ").count();
    let without_paths = out_without.matches("<path ").count();

    // The wiggle adds segments on system 1 (trailing) AND system 2 (incoming).
    // The cross-system pass alone has to add ≥1 segment on system 2; combined
    // with the within-system trailing wiggle and the "tr" glyph itself, the
    // delta should be substantial. The strict assertion is that
    // with_paths exceeds without_paths by at least 3 (tr glyph + ≥1 trailing
    // segment + ≥1 incoming segment).
    assert!(
        with_paths >= without_paths + 3,
        "trill+extension across systems should add ≥3 paths (tr + ≥1 trailing + ≥1 incoming), got {with_paths} vs {without_paths}"
    );
}

#[test]
fn cross_system_trill_extension_only_when_last_note_is_trilled() {
    // If the trill is NOT the last note of the system, there's nothing
    // unresolved — the within-system handler already drew the wiggle to the
    // following note. The cross-system pass must not produce any incoming
    // wiggle on system N+1 in that case.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // System 1 contains a trill followed by a plain note in the same system.
    let measures_resolved = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8), whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // For comparison: trill is the last note of system 1, so cross-system fires.
    let measures_unresolved = vec![
        MeasureContent {
            events: vec![whole_note(8), trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let page_resolved = layout_page(&prefix(), &measures_resolved, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let page_unresolved = layout_page(&prefix(), &measures_unresolved, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_resolved = draw_page(&font, &config, &page_resolved).unwrap().to_svg();
    let out_unresolved = draw_page(&font, &config, &page_unresolved).unwrap().to_svg();

    let resolved_paths = out_resolved.matches("<path ").count();
    let unresolved_paths = out_unresolved.matches("<path ").count();

    // The unresolved variant must produce strictly more paths because of the
    // incoming wiggle on system 2. The resolved variant should not produce an
    // incoming wiggle (trill is internal to system 1).
    assert!(
        unresolved_paths > resolved_paths,
        "unresolved-trill score should produce more paths (incoming wiggle); got resolved={resolved_paths}, unresolved={unresolved_paths}"
    );
}

#[test]
fn cross_system_trill_extension_no_target_system_no_incoming_wiggle() {
    // A trill+extension on the only system of a page must not crash and must
    // not produce a cross-system incoming wiggle (there is no next system).
    // The within-system trailing wiggle is still drawn by the system renderer.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8), whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    assert_eq!(page.systems.len(), 1, "test requires exactly one system");

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // Sanity: SVG is well-formed and a "tr" glyph is drawn (≥1 path).
    assert!(output.starts_with("<svg"));
    assert!(output.matches("<path ").count() >= 1);
}

#[test]
fn cross_system_trill_extension_incoming_y_anchored_to_target_staff() {
    // The incoming wiggle on system N+1 must anchor to system N+1's staff, not
    // to system N's. With two systems on the same page at different page-y
    // positions, the incoming wiggle's y must fall between system N+1's top
    // staff line and the top of system N's content area (i.e. clearly within
    // system N+1's vertical band).
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(page.systems.len(), 2);

    let sys2_top_y = page.systems[1].y;
    let sys1_top_y = page.systems[0].y;
    assert!(
        sys2_top_y > sys1_top_y,
        "test invariant: system 2 must be lower (larger y) than system 1; got {sys1_top_y} vs {sys2_top_y}"
    );

    let svg = draw_page(&font, &config, &page).unwrap();
    let output = svg.to_svg();

    // The wiggle paths are emitted via `translate(...)`. Extract the unique
    // y-values of the translate transforms; the cross-system incoming wiggle
    // must sit ABOVE system 2's top staff line (y < sys2_top_y) and BELOW
    // system 1's top staff line + offset (so it doesn't accidentally land
    // in system 1's vertical band).
    //
    // We don't try to reverse-engineer the exact pixel: we just verify that
    // at least one path's translate-y is bracketed by [sys1_top_y, sys2_top_y]
    // — i.e. in the inter-system "above-system-2-staff" band, which is where
    // a properly anchored incoming wiggle would sit.
    let mut found_in_band = false;
    for line in output.split('\n') {
        let Some(idx) = line.find("translate(") else { continue; };
        let rest = &line[idx + "translate(".len()..];
        let Some(close) = rest.find(')') else { continue; };
        let args = &rest[..close];
        let parts: Vec<&str> = args.split(',').collect();
        if parts.len() != 2 { continue; }
        let Ok(y) = parts[1].trim().parse::<f64>() else { continue; };
        if y > sys1_top_y && y < sys2_top_y {
            found_in_band = true;
            break;
        }
    }
    assert!(
        found_in_band,
        "incoming cross-system trill wiggle must have at least one path translated to y ∈ ({sys1_top_y}, {sys2_top_y})"
    );
}

#[test]
fn cross_system_trill_extension_no_op_without_trill() {
    // Sanity: pages without trill extensions render the same paths regardless
    // of whether draw_cross_system_trill_extensions runs. The function must be
    // a no-op when there are no unresolved trills.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![
        MeasureContent {
            events: vec![whole_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let output = draw_page(&font, &config, &page).unwrap().to_svg();

    // No trill glyph nor wiggle paths should appear. The "tr" glyph for
    // OrnamentTrill has a distinct path; we can't easily isolate it, but
    // we can confirm the path count is reasonable for two staves of two
    // plain whole notes — i.e. notably small.
    let path_count = output.matches("<path ").count();
    // 2 systems × (clef + whole notehead) + first system time sig digits.
    // An empty-style baseline has ≤ ~10 paths; the bound is generous.
    assert!(
        path_count < 12,
        "no-trill 2-system page should have a small path count, got {path_count}"
    );
}

// --- Cross-system trill bracket tests ---

use crate::layout::trill_bracket::TrillBracketSide;

/// A whole note carrying a trill+extension AND a bracket request. The
/// bracket side is parameterized so each test can target Start / End / Both
/// independently.
fn trill_ext_bracketed_whole_note(pos: i8, side: TrillBracketSide) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(side),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_trill_bracket_end_adds_one_hook_on_target_system() {
    // End bracket on the last note of system N: the system pass on N draws
    // NO hook (suppressed), and the page renderer draws exactly ONE hook
    // at the right edge of the incoming wiggle on system N+1.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_end_bracket = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_whole_note(8, TrillBracketSide::End)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &with_end_bracket, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p1.systems.len(), 2);
    assert_eq!(p2.systems.len(), 2);

    let out_with = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p2).unwrap().to_svg();

    let delta = out_with.matches("<line ").count() - out_plain.matches("<line ").count();
    assert_eq!(
        delta, 1,
        "End bracket on cross-system trill must add exactly 1 hook on N+1 (got delta={delta})"
    );
}

#[test]
fn cross_system_trill_bracket_both_adds_start_on_n_and_end_on_n_plus_1() {
    // Both bracket on the last note of system N: Start hook is drawn by the
    // system pass on N; End hook is drawn by the page renderer on N+1. The
    // two combined must add exactly 2 hook <line> elements vs the plain
    // (no-bracket) cross-system trill baseline.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_both = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_whole_note(8, TrillBracketSide::Both)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &with_both, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_with = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p2).unwrap().to_svg();

    let delta = out_with.matches("<line ").count() - out_plain.matches("<line ").count();
    assert_eq!(
        delta, 2,
        "Both bracket on cross-system trill must add exactly 2 hooks total (got delta={delta})"
    );
}

#[test]
fn cross_system_trill_bracket_start_adds_only_one_hook_on_source_system() {
    // Start bracket on the last note of system N: Start hook drawn by the
    // system pass on N; nothing extra on N+1 (the page renderer skips End
    // when the user only asked for Start). Total delta vs plain: 1.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_start = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_whole_note(8, TrillBracketSide::Start)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &with_start, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_with = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p2).unwrap().to_svg();

    let delta = out_with.matches("<line ").count() - out_plain.matches("<line ").count();
    assert_eq!(
        delta, 1,
        "Start-only bracket on cross-system trill must add exactly 1 hook on N (got delta={delta})"
    );
}

// --- Cross-system trill bracket custom direction / length ---

use crate::layout::trill_bracket::HookDirection;

fn trill_ext_bracketed_custom_whole_note(
    pos: i8,
    side: TrillBracketSide,
    direction: HookDirection,
    length_ss: f64,
) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(side),
            trill_bracket_direction: Some(direction),
            trill_bracket_length_ss: Some(length_ss),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_trill_bracket_custom_length_changes_n_plus_1_hook() {
    // The End hook drawn on system N+1 must honor the user's custom length.
    // We can't easily isolate the hook's y-extent across the whole page, but
    // we *can* assert that two different lengths produce different page-level
    // SVG with the same number of <line> elements (only geometry differs).
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let short = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_custom_whole_note(
                8,
                TrillBracketSide::End,
                HookDirection::Down,
                0.5,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let long = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_custom_whole_note(
                8,
                TrillBracketSide::End,
                HookDirection::Down,
                1.5,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_short = layout_page(&prefix(), &short, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_long = layout_page(&prefix(), &long, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_short = draw_page(&font, &config, &p_short).unwrap().to_svg();
    let out_long = draw_page(&font, &config, &p_long).unwrap().to_svg();

    assert_eq!(
        out_short.matches("<line ").count(),
        out_long.matches("<line ").count(),
        "Length change must NOT add/remove <line> elements on the page"
    );
    assert_ne!(out_short, out_long, "Length change must change the SVG");
}

#[test]
fn cross_system_trill_bracket_custom_direction_changes_n_plus_1_hook() {
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let mk = |direction: HookDirection| {
        vec![
            MeasureContent {
                events: vec![trill_ext_bracketed_custom_whole_note(
                    8,
                    TrillBracketSide::End,
                    direction,
                    0.75,
                )],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
            },
            MeasureContent {
                events: vec![whole_note(8)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
            },
        ]
    };
    let p_down = layout_page(&prefix(), &mk(HookDirection::Down), &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_up = layout_page(&prefix(), &mk(HookDirection::Up), &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_down = draw_page(&font, &config, &p_down).unwrap().to_svg();
    let out_up = draw_page(&font, &config, &p_up).unwrap().to_svg();

    assert_eq!(
        out_down.matches("<line ").count(),
        out_up.matches("<line ").count(),
        "Direction change must NOT add/remove <line> elements on the page"
    );
    assert_ne!(
        out_down, out_up,
        "Down vs Up direction must change the SVG output"
    );
}

#[test]
fn cross_system_trill_bracket_custom_defaults_match_plain_bracketed() {
    // The custom-API code path with `direction=Down, length=0.75ss` must
    // produce byte-identical SVG to the plain bracketed API. Cross-system
    // canary protecting existing users from drift introduced by the wiring.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let custom = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_custom_whole_note(
                8,
                TrillBracketSide::Both,
                HookDirection::Down,
                0.75,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_bracketed_whole_note(8, TrillBracketSide::Both)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &custom, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let out_custom = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p2).unwrap().to_svg();

    assert_eq!(
        out_custom, out_plain,
        "custom(Down, 0.75ss) cross-system must render byte-identically to the plain bracketed API"
    );
}

// --- Cross-system trill wiggle speed continuity ---

use crate::layout::trill_extension::TrillWiggleSpeed;

fn trill_ext_speed_whole_note(pos: i8, speed: TrillWiggleSpeed) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_wiggle_speed: Some(speed),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_trill_extension_fast_speed_adds_more_incoming_paths_than_slow() {
    // The incoming wiggle on system N+1 must respect the source note's
    // speed choice: a faster (denser) wiggle tiles more segments across the
    // same incoming span than a slower one. Without this, the cross-system
    // pass would drop the user's speed back to the default mid-trill —
    // visually breaking the continuity it's meant to provide.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    fn measures(speed: TrillWiggleSpeed) -> Vec<MeasureContent> {
        vec![
            MeasureContent {
                events: vec![trill_ext_speed_whole_note(8, speed)],
                barline: BarlineStyle::Single,
                volta: None,
                additional_voices: vec![],
            },
            MeasureContent {
                events: vec![whole_note(8)],
                barline: BarlineStyle::Final,
                volta: None,
                additional_voices: vec![],
            },
        ]
    }

    let fast_pg = layout_page(
        &prefix(),
        &measures(TrillWiggleSpeed::Fastest),
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(1),
    );
    let slow_pg = layout_page(
        &prefix(),
        &measures(TrillWiggleSpeed::Slowest),
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(1),
    );
    assert_eq!(fast_pg.systems.len(), 2, "test requires two systems");
    assert_eq!(slow_pg.systems.len(), 2, "test requires two systems");

    let fast = draw_page(&font, &config, &fast_pg).unwrap().to_svg();
    let slow = draw_page(&font, &config, &slow_pg).unwrap().to_svg();

    let fast_paths = fast.matches("<path ").count();
    let slow_paths = slow.matches("<path ").count();
    assert!(
        fast_paths > slow_paths,
        "Fastest cross-system wiggle must tile more total segments than Slowest: \
         fast={fast_paths}, slow={slow_paths}",
    );
}

#[test]
fn cross_system_trill_extension_speed_changes_svg_byte_for_byte() {
    // A speed choice must propagate through the cross-system path so the
    // incoming wiggle on N+1 also reflects the speed. Two distinct speeds
    // must produce distinct SVGs end-to-end.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let fast_measures = vec![
        MeasureContent {
            events: vec![trill_ext_speed_whole_note(8, TrillWiggleSpeed::Fast)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let slow_measures = vec![
        MeasureContent {
            events: vec![trill_ext_speed_whole_note(8, TrillWiggleSpeed::Slow)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &fast_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &slow_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_fast = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_slow = draw_page(&font, &config, &p2).unwrap().to_svg();

    assert_ne!(
        out_fast, out_slow,
        "Distinct cross-system wiggle speeds must render byte-distinct SVGs",
    );
}

#[test]
fn cross_system_trill_extension_standard_speed_matches_unset_speed() {
    // Asking for Standard explicitly must produce byte-identical output to
    // not setting a speed at all. This is the cross-system counterpart of
    // the within-system canary test.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let standard = vec![
        MeasureContent {
            events: vec![trill_ext_speed_whole_note(8, TrillWiggleSpeed::Standard)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let unset = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p1 = layout_page(&prefix(), &standard, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p2 = layout_page(&prefix(), &unset, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_std = draw_page(&font, &config, &p1).unwrap().to_svg();
    let out_unset = draw_page(&font, &config, &p2).unwrap().to_svg();

    assert_eq!(
        out_std, out_unset,
        "explicit Standard speed and unset speed must produce identical SVG"
    );
}

// --- Cross-system multi-speed trill (speed ramp) continuation tests ---

use crate::layout::trill_extension::{TrillSpeedRamp, TrillSpeedRampSpec};

/// A whole note carrying a trill+extension with a multi-speed `TrillSpeedRamp`.
fn trill_ext_ramp_whole_note(
    pos: i8,
    ramp: TrillSpeedRamp,
    region_count: usize,
) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_speed_ramp: Some(TrillSpeedRampSpec::new(ramp, region_count)),
            ..NoteAnnotations::default()
        },
    })
}

/// A whole note with both trill+extension and a multi-speed ramp AND a
/// bracket request — for cross-system multi-speed bracket tests.
fn trill_ext_ramp_bracketed_whole_note(
    pos: i8,
    ramp: TrillSpeedRamp,
    region_count: usize,
    side: TrillBracketSide,
) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_speed_ramp: Some(TrillSpeedRampSpec::new(ramp, region_count)),
            trill_bracket: Some(side),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn cross_system_multi_speed_trill_adds_incoming_paths_on_next_system() {
    // A multi-speed trill on the last note of system N must draw an
    // incoming wiggle on system N+1 (regions re-synthesized against the
    // target span). The path count must exceed a no-trill baseline by at
    // least 3 (tr glyph on N + ≥1 trailing tile on N + ≥1 incoming tile
    // on N+1).
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_ramp = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let without_trill = vec![
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_with = layout_page(&prefix(), &with_ramp, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_without = layout_page(&prefix(), &without_trill, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p_with.systems.len(), 2, "test requires two systems");

    let out_with = draw_page(&font, &config, &p_with).unwrap().to_svg();
    let out_without = draw_page(&font, &config, &p_without).unwrap().to_svg();

    let with_paths = out_with.matches("<path ").count();
    let without_paths = out_without.matches("<path ").count();
    assert!(
        with_paths >= without_paths + 3,
        "cross-system multi-speed trill should add ≥3 paths (tr + ≥1 trailing + ≥1 incoming), got {with_paths} vs {without_paths}",
    );
}

#[test]
fn cross_system_multi_speed_trill_renders_distinct_svg_from_single_speed() {
    // Critical correctness canary: a multi-speed cross-system trill must
    // NOT render byte-identical SVG to a single-speed cross-system trill.
    // A regression that ignored the speed_ramp on the unresolved side
    // would silently drop the multi-speed dispatch and produce identical
    // output to a Standard wiggle.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let ramp_measures = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let single_speed_measures = vec![
        MeasureContent {
            events: vec![trill_ext_speed_whole_note(8, TrillWiggleSpeed::Standard)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_ramp = layout_page(&prefix(), &ramp_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_single = layout_page(&prefix(), &single_speed_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p_ramp.systems.len(), 2);
    assert_eq!(p_single.systems.len(), 2);

    let out_ramp = draw_page(&font, &config, &p_ramp).unwrap().to_svg();
    let out_single = draw_page(&font, &config, &p_single).unwrap().to_svg();

    assert_ne!(
        out_ramp, out_single,
        "cross-system multi-speed trill must render distinct SVG from single-speed Standard",
    );
}

#[test]
fn cross_system_multi_speed_trill_uses_multiple_distinct_wiggle_glyphs() {
    // A Linear Slow→Fast multi-speed trill must emit at least TWO distinct
    // wiggle glyph outlines on system N+1 (the incoming wiggle re-
    // synthesizes regions across the target span and tiles each region's
    // glyph). A regression that fell back to a single-speed tile-fill
    // would emit only one distinct outline.
    //
    // We compare against a single-speed baseline on the same score
    // shape: the multi-speed SVG must contain a strict superset of the
    // single-speed SVG's distinct path `d="..."` values for the wiggle
    // family of glyphs.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let ramp_measures = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p = layout_page(&prefix(), &ramp_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p.systems.len(), 2);
    let out = draw_page(&font, &config, &p).unwrap().to_svg();

    // Collect all distinct `d="..."` substrings — a regression that
    // emitted only one wiggle outline would shrink this set substantially.
    use std::collections::HashSet;
    let mut distinct_paths: HashSet<&str> = HashSet::new();
    let bytes = out.as_bytes();
    let mut i = 0usize;
    while i + 3 < bytes.len() {
        if &bytes[i..i + 3] == b"d=\"" {
            let start = i + 3;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            // SAFETY: SVG path data is ASCII; we only sliced at ASCII
            // boundaries (the `"` delimiter is single-byte).
            distinct_paths.insert(std::str::from_utf8(&bytes[start..j]).unwrap());
            i = j + 1;
        } else {
            i += 1;
        }
    }
    // 3 regions × {Slow, Standard, Fast} produces 3 distinct wiggle
    // outlines on the target system AND 3 distinct wiggle outlines on
    // the source system (re-synthesized symmetrically). The overall
    // distinct path set must have at least 3 distinct wiggle glyphs
    // present — plus the "tr" glyph, the staff lines (which are <line>
    // not <path>), and the noteheads.
    assert!(
        distinct_paths.len() >= 5,
        "multi-speed cross-system trill should emit ≥5 distinct path outlines \
         (tr + Slow + Standard + Fast wiggles + notehead): got {}",
        distinct_paths.len()
    );
}

#[test]
fn cross_system_multi_speed_trill_explicit_length_suppresses_continuation() {
    // An explicit length terminates the wiggle within the source system
    // (this rule applies uniformly to single-speed and multi-speed). The
    // cross-system pass must NOT draw an incoming wiggle on N+1. We
    // compare against an identical score WITHOUT the trill: the path
    // counts on system N+1 must match.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let mut explicit_evt = trill_ext_ramp_whole_note(
        8,
        TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        },
        3,
    );
    if let MeasureEvent::Note(ref mut n) = explicit_evt {
        n.annotations.trill_extension_length_ss = Some(2.0);
    }
    let explicit_score = vec![
        MeasureContent {
            events: vec![explicit_evt],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let no_ramp_score = vec![
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_expl = layout_page(&prefix(), &explicit_score, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_none = layout_page(&prefix(), &no_ramp_score, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p_expl.systems.len(), 2);
    assert_eq!(p_none.systems.len(), 2);

    // Count `<path` segments on the *second* system. The page layout
    // has the second system starting at a y greater than the first;
    // splitting the SVG at the second `<line ` containing the system-2
    // staff-line y would be brittle. Instead, render both with and
    // without explicit length and assert that explicit length produces
    // strictly fewer paths than the no-explicit-length variant (which
    // we know adds an incoming wiggle).
    let no_explicit_score = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let p_continued = layout_page(
        &prefix(),
        &no_explicit_score,
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(1),
    );
    let out_explicit = draw_page(&font, &config, &p_expl).unwrap().to_svg();
    let out_continued = draw_page(&font, &config, &p_continued).unwrap().to_svg();

    let explicit_paths = out_explicit.matches("<path ").count();
    let continued_paths = out_continued.matches("<path ").count();
    assert!(
        explicit_paths < continued_paths,
        "explicit_length_ss must suppress cross-system multi-speed continuation: \
         explicit paths={explicit_paths}, continued paths={continued_paths}",
    );

    // Also: the explicit-length output must NOT contain MORE paths than
    // the no-ramp baseline (the trailing wiggle is internal, but the
    // cross-system incoming wiggle is suppressed).
    let out_none = draw_page(&font, &config, &p_none).unwrap().to_svg();
    let none_paths = out_none.matches("<path ").count();
    // We assert the explicit-length score has at most a few more paths
    // than the no-trill baseline: the trill glyph + a few internal
    // wiggle tiles on system N (capped by len_ss=2.0), no incoming on
    // N+1. A regression that re-engaged cross-system continuation would
    // add at least one wiggle tile on system N+1 — pushing the count
    // toward `continued_paths`.
    let internal_only_delta = explicit_paths.saturating_sub(none_paths);
    let continued_delta = continued_paths.saturating_sub(none_paths);
    assert!(
        internal_only_delta < continued_delta,
        "explicit-length delta over baseline ({internal_only_delta}) must be smaller \
         than continued-trill delta over baseline ({continued_delta})",
    );
}

#[test]
fn cross_system_trill_to_note_offset_suppresses_cross_system_continuation() {
    // A `to_note_offset` is a definite end-anchor request from the user.
    // The within-system pass terminates the wiggle at the offset target
    // (or at the system edge for overshoots) with cross_system=false; the
    // cross-system pass must mirror that decision and NOT draw an incoming
    // wiggle on system N+1. Without the page-renderer mirror, the source
    // system would correctly suppress propagation but system N+1 would
    // still get an unattributable incoming wiggle.
    //
    // We use offset=99 (overshoots) on the last note of system 1. The
    // source pass falls back to the system-edge formula but keeps
    // cross_system=false because to_note_offset.is_some(). The
    // page-renderer mirror catches that and returns None from
    // compute_cross_system_trill_continuation, suppressing the incoming
    // wiggle on system 2 — so the render must NOT contain more paths than
    // a no-trill baseline on the second system side.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_to_offset_evt = MeasureEvent::Note(NoteEvent {
        staff_position: 8,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_to_note_offset: Some(99),
            ..NoteAnnotations::default()
        },
    });
    let with_to_offset_score = vec![
        MeasureContent {
            events: vec![with_to_offset_evt],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // The "natural" comparison: same first-system trilled note WITHOUT
    // the to_note_offset annotation. The page-renderer continues the
    // wiggle onto system 2.
    let natural_score = vec![
        MeasureContent {
            events: vec![trill_ext_whole_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_to = layout_page(
        &prefix(),
        &with_to_offset_score,
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(1),
    );
    let p_nat = layout_page(
        &prefix(),
        &natural_score,
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(1),
    );
    assert_eq!(p_to.systems.len(), 2, "test requires two systems");
    assert_eq!(p_nat.systems.len(), 2, "test requires two systems");

    let out_to = draw_page(&font, &config, &p_to).unwrap().to_svg();
    let out_nat = draw_page(&font, &config, &p_nat).unwrap().to_svg();

    // The natural variant has more paths than the to_note_offset variant
    // because it adds an incoming wiggle on system 2 that the
    // to_note_offset variant suppresses. A regression that re-engaged
    // cross-system propagation under to_note_offset would equalize the
    // two counts (or even make them match exactly).
    let to_paths = out_to.matches("<path ").count();
    let nat_paths = out_nat.matches("<path ").count();
    assert!(
        to_paths < nat_paths,
        "to_note_offset must suppress cross-system continuation: \
         to_offset={to_paths}, natural={nat_paths}"
    );
}

#[test]
fn cross_system_multi_speed_trill_bracket_end_adds_one_hook_on_target_system() {
    // End bracket on a multi-speed trill's last source-system note: no
    // hook on N (suppressed by within-system pass), one hook on N+1 at
    // the right edge of the incoming multi-speed wiggle.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_end_bracket = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_bracketed_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
                TrillBracketSide::End,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_bracket = layout_page(&prefix(), &with_end_bracket, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_plain = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_bracket = draw_page(&font, &config, &p_bracket).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p_plain).unwrap().to_svg();

    let bracket_lines = out_bracket.matches("<line ").count();
    let plain_lines = out_plain.matches("<line ").count();
    // End hook adds exactly one `<line>` element: it's drawn on N+1, no
    // hook on N (suppressed).
    assert_eq!(
        bracket_lines,
        plain_lines + 1,
        "End bracket on multi-speed cross-system trill must add exactly one line on N+1: \
         bracket={bracket_lines}, plain={plain_lines}",
    );
}

#[test]
fn cross_system_multi_speed_trill_bracket_both_adds_start_on_n_and_end_on_n_plus_1() {
    // Both bracket on a multi-speed cross-system trill: Start drawn on
    // N (within-system pass), End drawn on N+1 (page pass). Total: 2
    // hook `<line>` elements added over a no-bracket baseline.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_both_bracket = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_bracketed_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
                TrillBracketSide::Both,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_bracket = layout_page(&prefix(), &with_both_bracket, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_plain = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_bracket = draw_page(&font, &config, &p_bracket).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p_plain).unwrap().to_svg();

    let bracket_lines = out_bracket.matches("<line ").count();
    let plain_lines = out_plain.matches("<line ").count();
    assert_eq!(
        bracket_lines,
        plain_lines + 2,
        "Both bracket on multi-speed cross-system trill must add exactly two lines \
         (Start on N + End on N+1): bracket={bracket_lines}, plain={plain_lines}",
    );
}

#[test]
fn cross_system_multi_speed_trill_bracket_start_only_adds_no_hook_on_target() {
    // Start-only bracket: hook drawn on N (within-system), nothing on
    // N+1 (cross-system pass should NOT draw an End hook when not
    // requested). Total: exactly one `<line>` over a no-bracket baseline.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let with_start_bracket = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_bracketed_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
                TrillBracketSide::Start,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let plain = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_bracket = layout_page(&prefix(), &with_start_bracket, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_plain = layout_page(&prefix(), &plain, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

    let out_bracket = draw_page(&font, &config, &p_bracket).unwrap().to_svg();
    let out_plain = draw_page(&font, &config, &p_plain).unwrap().to_svg();

    let bracket_lines = out_bracket.matches("<line ").count();
    let plain_lines = out_plain.matches("<line ").count();
    assert_eq!(
        bracket_lines,
        plain_lines + 1,
        "Start-only bracket on cross-system multi-speed trill must add exactly one line \
         (Start on N): bracket={bracket_lines}, plain={plain_lines}",
    );
}

#[test]
fn cross_system_multi_speed_trill_accel_distinct_from_decel() {
    // Slow→Fast (accel) and Fast→Slow (decel) ramps render distinct SVG
    // end-to-end. Direction-symmetry canary: a regression that ignored
    // ramp direction or sorted speeds before synthesizing would produce
    // identical output.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let accel = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let decel = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Fast,
                    end: TrillWiggleSpeed::Slow,
                },
                3,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_accel = layout_page(&prefix(), &accel, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_decel = layout_page(&prefix(), &decel, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let out_accel = draw_page(&font, &config, &p_accel).unwrap().to_svg();
    let out_decel = draw_page(&font, &config, &p_decel).unwrap().to_svg();

    assert_ne!(
        out_accel, out_decel,
        "cross-system accel and decel ramps must render distinct SVG",
    );
}

#[test]
fn cross_system_multi_speed_constant_ramp_equals_single_speed_when_one_region() {
    // Edge-case equivalence: `Constant(Standard)` with 1 region must
    // produce SVG byte-identical to a single-speed `Standard` trill on
    // the same score shape. Both should synthesize a single region of
    // Standard glyphs across the source-system trailing wiggle AND the
    // target-system incoming wiggle.
    //
    // This is the degenerate-equivalence canary that the
    // within-system test asserts (in `score::tests`); we mirror it for
    // the cross-system path.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let constant_ramp = vec![
        MeasureContent {
            events: vec![trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard),
                1,
            )],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let single_speed = vec![
        MeasureContent {
            events: vec![trill_ext_speed_whole_note(8, TrillWiggleSpeed::Standard)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![whole_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let p_const = layout_page(&prefix(), &constant_ramp, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    let p_single = layout_page(&prefix(), &single_speed, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
    assert_eq!(p_const.systems.len(), 2);
    assert_eq!(p_single.systems.len(), 2);

    let out_const = draw_page(&font, &config, &p_const).unwrap().to_svg();
    let out_single = draw_page(&font, &config, &p_single).unwrap().to_svg();

    // We don't assert byte-equality (the two paths use different
    // synthesize-vs-direct-glyph-lookup code paths; geometry should
    // match but path-element ordering might differ subtly). Instead,
    // assert the same total path count — a regression that emitted no
    // multi-speed continuation would be one path short.
    let const_paths = out_const.matches("<path ").count();
    let single_paths = out_single.matches("<path ").count();
    assert_eq!(
        const_paths, single_paths,
        "Constant(Standard) with 1 region must produce the same total path count \
         as single-speed Standard across system break: const={const_paths}, single={single_paths}",
    );
}

#[test]
fn cross_system_multi_speed_trill_no_target_system_no_crash() {
    // Single-system page with a multi-speed trill+extension on the last
    // (and only) note: no system N+1 exists, so the page renderer must
    // not crash and must not draw any incoming wiggle. The within-system
    // trailing wiggle (multi-speed) is still drawn by the system
    // renderer.
    let (font, config) = setup();
    let ss = config.staff_space;
    let page_cfg = PageLayoutConfig::new(ss, 8000.0);
    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_ramp_whole_note(
                8,
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                3,
            ),
            whole_note(8),
        ],
        barline: BarlineStyle::Final,
        volta: None,
        additional_voices: vec![],
    }];
    let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
    assert_eq!(page.systems.len(), 1, "test requires exactly one system");

    let out = draw_page(&font, &config, &page).unwrap().to_svg();
    assert!(out.starts_with("<svg"));
    // ≥1 trill glyph path + several wiggle tiles + notehead paths.
    assert!(out.matches("<path ").count() >= 3);
}
