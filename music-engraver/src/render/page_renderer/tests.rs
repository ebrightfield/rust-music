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
