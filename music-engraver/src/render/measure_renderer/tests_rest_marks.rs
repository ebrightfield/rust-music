//! Marks on rests, text scripts, custom dynamics and barline marks, asserted
//! at numeric positions relative to the event they attach to.

use music::notation::clef::Clef;
use smufl::Glyph;

use super::draw_measure;
use crate::font::{bravura_font, EngravingConfig, MusicFont};
use crate::layout::articulation::Articulation;
use crate::layout::barline::{barline_layout, BarlineStyle};
use crate::layout::dynamics::{CustomDynamic, Dynamic, DYNAMIC_TEXT_GAP_SS};
use crate::layout::measure::{
    layout_measure, MeasureElement, MeasureLayout, MeasureLayoutConfig, NoteAnnotations,
    NoteEvent, RestEvent,
};
use crate::layout::placement::Placement;
use crate::layout::staff::StaffLayout;
use crate::layout::tempo::TempoMark;
use crate::layout::text_script::{
    TextScript, TEXT_ASCENT_RATIO, TEXT_DESCENT_RATIO, TEXT_SCRIPT_FONT_SIZE_SS,
    TEXT_SCRIPT_PADDING_SS, TEXT_SCRIPT_STACK_GAP_SS,
};
use crate::render::SvgWriter;
use crate::svg_probe::{assert_close, glyph, glyphs, text};

struct Rendered {
    svg: String,
    layout: MeasureLayout,
    staff: StaffLayout,
    config: EngravingConfig,
    font: MusicFont<'static>,
}

impl Rendered {
    fn ss(&self) -> f64 {
        self.config.staff_space
    }
}

fn render(elements: Vec<MeasureElement>) -> Rendered {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
    let layout = layout_measure(&elements, &MeasureLayoutConfig::from_staff_space(config.staff_space));
    let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -2000.0, 8000.0, 5000.0);
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    Rendered {
        svg: svg.to_svg(),
        layout,
        staff,
        config,
        font,
    }
}

fn quarter_rest(annotations: NoteAnnotations) -> MeasureElement {
    MeasureElement::Rest(RestEvent {
        duration_log2: 2,
        dots: 0,
        annotations,
    })
}

fn quarter_note(position: i8, annotations: NoteAnnotations) -> MeasureElement {
    MeasureElement::Note(NoteEvent {
        staff_position: position,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations,
    })
}

fn advance(font: &MusicFont, g: Glyph) -> f64 {
    f64::from(font.glyph_advance(g).unwrap())
}

const TEXT_DESCENT_SS: f64 = TEXT_DESCENT_RATIO * TEXT_SCRIPT_FONT_SIZE_SS;
const TEXT_ASCENT_SS: f64 = TEXT_ASCENT_RATIO * TEXT_SCRIPT_FONT_SIZE_SS;

#[test]
fn dynamic_on_rest_is_centered_on_the_rest_below_or_above() {
    for (placement, expected_y_ss) in [(Placement::Below, None), (Placement::Above, Some(()))] {
        let r = render(vec![quarter_rest(NoteAnnotations {
            dynamic: Some(Dynamic::Piano.into()),
            dynamics_placement: placement,
            ..NoteAnnotations::default()
        })]);
        let rest = glyph(&r.svg, &r.font, Glyph::RestQuarter);
        let rest_x = r.layout.elements[0].x;
        assert_close(rest.x, rest_x);
        let p = glyph(&r.svg, &r.font, Glyph::DynamicPiano);
        let rest_center = rest_x + advance(&r.font, Glyph::RestQuarter) / 2.0;
        assert_close(p.x, rest_center - advance(&r.font, Glyph::DynamicPiano) / 2.0);
        let expected_y = match expected_y_ss {
            None => r.staff.bottom_y() + 2.5 * r.ss(),
            Some(()) => r.staff.y_of(8) - 1.3 * r.ss(),
        };
        assert_close(p.y, expected_y);
    }
}

#[test]
fn fermata_on_rest_sits_above_the_staff_over_the_rest() {
    let r = render(vec![quarter_rest(NoteAnnotations {
        articulations: vec![Articulation::Fermata],
        ..NoteAnnotations::default()
    })]);
    let rest_x = r.layout.elements[0].x;
    let fermata = glyph(&r.svg, &r.font, Glyph::FermataAbove);
    assert_close(fermata.x, rest_x + advance(&r.font, Glyph::RestQuarter) / 2.0);
    let bbox = r.font.glyph_bbox_design_units(Glyph::FermataAbove).unwrap();
    assert!(
        fermata.y + bbox.y_bottom <= r.staff.y_of(8),
        "fermata must clear the top line: {} vs {}",
        fermata.y + bbox.y_bottom,
        r.staff.y_of(8)
    );
}

#[test]
fn text_label_on_rest_starts_at_the_rest_above_the_staff() {
    let r = render(vec![quarter_rest(NoteAnnotations {
        text_scripts: vec![TextScript::above("a)")],
        ..NoteAnnotations::default()
    })]);
    let label = text(&r.svg, "a)");
    assert_close(label.x, r.layout.elements[0].x);
    assert_eq!(label.anchor, "start");
    assert_eq!(label.style, "normal");
    assert_close(
        label.y,
        r.staff.y_of(8) - (TEXT_SCRIPT_PADDING_SS + TEXT_DESCENT_SS) * r.ss(),
    );
}

#[test]
fn tempo_on_rest_is_left_aligned_with_the_rest() {
    let r = render(vec![quarter_rest(NoteAnnotations {
        tempo_mark: Some(TempoMark::text("Andante")),
        ..NoteAnnotations::default()
    })]);
    let tempo = text(&r.svg, "Andante");
    assert_close(tempo.x, r.layout.elements[0].x);
    assert_eq!(tempo.weight, "bold");
    assert_close(tempo.y, r.staff.y_of(8) - 2.8 * r.ss());
}

#[test]
fn dotted_rest_draws_its_dot_right_of_the_rest() {
    let r = render(vec![MeasureElement::Rest(RestEvent {
        duration_log2: 2,
        dots: 1,
        annotations: NoteAnnotations::default(),
    })]);
    let dot = glyph(&r.svg, &r.font, Glyph::AugmentationDot);
    assert!(dot.x > r.layout.elements[0].x + advance(&r.font, Glyph::RestQuarter));
    // A quarter rest is centered on the middle line, so its dot goes in the
    // space above it.
    assert_close(dot.y, r.staff.y_of(5));
}

#[test]
fn text_scripts_stack_outward_on_each_side() {
    let r = render(vec![quarter_note(
        4,
        NoteAnnotations {
            text_scripts: vec![
                TextScript::above("Sing"),
                TextScript::above("Imagine").italic(),
                TextScript::below("gliss.").italic(),
                TextScript::below("[ed.]").tiny(),
            ],
            dynamic: Some(Dynamic::Forte.into()),
            ..NoteAnnotations::default()
        },
    )]);
    let ss = r.ss();
    let note_x = r.layout.elements[0].x;
    let sing = text(&r.svg, "Sing");
    let imagine = text(&r.svg, "Imagine");
    assert_close(sing.x, note_x);
    assert_close(imagine.x, note_x);
    assert_eq!(imagine.style, "italic");
    let first = r.staff.y_of(8) - (TEXT_SCRIPT_PADDING_SS + TEXT_DESCENT_SS) * ss;
    assert_close(sing.y, first);
    assert_close(
        imagine.y,
        first - (TEXT_ASCENT_SS + TEXT_SCRIPT_STACK_GAP_SS + TEXT_DESCENT_SS) * ss,
    );

    // Below: the forte holds its band; the first below script moves out
    // past it only if needed, the second stacks under the first.
    let forte = glyph(&r.svg, &r.font, Glyph::DynamicForte);
    assert_close(forte.y, r.staff.bottom_y() + 2.5 * ss);
    let forte_bottom = forte.y + r.font.glyph_bbox_design_units(Glyph::DynamicForte).unwrap().y_bottom;
    let gliss = text(&r.svg, "gliss.");
    assert_eq!(gliss.style, "italic");
    let gliss_y = (r.staff.bottom_y() + 4.0 * ss)
        .max(forte_bottom + TEXT_SCRIPT_STACK_GAP_SS * ss + TEXT_ASCENT_SS * ss);
    assert_close(gliss.y, gliss_y);
    let ed = text(&r.svg, "[ed.]");
    let tiny = 2f64.powf(-2.0 / 6.0);
    assert_close(ed.size, TEXT_SCRIPT_FONT_SIZE_SS * ss * tiny);
    assert_close(
        ed.y,
        gliss_y
            + (TEXT_DESCENT_SS + TEXT_SCRIPT_STACK_GAP_SS) * ss
            + TEXT_ASCENT_RATIO * TEXT_SCRIPT_FONT_SIZE_SS * tiny * ss,
    );
}

#[test]
fn ledger_line_note_pushes_its_dynamic_clear_of_the_notehead() {
    // Middle C hangs 1.5 staff spaces below the bottom line; the p glyph's
    // top must clear its notehead by the script padding.
    let r = render(vec![quarter_note(
        -2,
        NoteAnnotations {
            dynamic: Some(Dynamic::Piano.into()),
            ..NoteAnnotations::default()
        },
    )]);
    let ss = r.ss();
    let p = glyph(&r.svg, &r.font, Glyph::DynamicPiano);
    let top = r.font.glyph_bbox_design_units(Glyph::DynamicPiano).unwrap().y_top;
    assert_close(
        p.y + top,
        r.staff.y_of(-2) + 0.5 * ss + TEXT_SCRIPT_PADDING_SS * ss,
    );
}

#[test]
fn piu_p_centers_the_dynamic_glyph_and_sets_the_words_to_its_left() {
    let r = render(vec![quarter_rest(NoteAnnotations {
        dynamic: Some(CustomDynamic::new().text("più").mark(Dynamic::Piano).into()),
        dynamics_placement: Placement::Above,
        ..NoteAnnotations::default()
    })]);
    let rest_center = r.layout.elements[0].x + advance(&r.font, Glyph::RestQuarter) / 2.0;
    let p = glyph(&r.svg, &r.font, Glyph::DynamicPiano);
    assert_close(p.x, rest_center - advance(&r.font, Glyph::DynamicPiano) / 2.0);
    let words = text(&r.svg, "più");
    assert_eq!(words.style, "italic");
    assert_eq!(words.anchor, "end");
    assert_close(words.x, p.x - DYNAMIC_TEXT_GAP_SS * r.ss());
    assert_close(words.y, p.y);
}

#[test]
fn barline_mark_is_centered_on_the_following_barline() {
    let r = render(vec![
        quarter_note(
            4,
            NoteAnnotations {
                text_marks: vec![TextScript::glyph(Glyph::FermataAbove, Placement::Above)],
                ..NoteAnnotations::default()
            },
        ),
        MeasureElement::Barline(BarlineStyle::Final),
    ]);
    let barline_x = r.layout.elements[1].x;
    let width = barline_layout(BarlineStyle::Final, barline_x, &r.staff, &r.config).width;
    let fermata = glyph(&r.svg, &r.font, Glyph::FermataAbove);
    assert_close(
        fermata.x,
        barline_x + width / 2.0 - advance(&r.font, Glyph::FermataAbove) / 2.0,
    );
    let bbox = r.font.glyph_bbox_design_units(Glyph::FermataAbove).unwrap();
    assert_close(
        fermata.y + bbox.y_bottom,
        r.staff.y_of(8) - TEXT_SCRIPT_PADDING_SS * r.ss(),
    );
}

#[test]
fn barline_text_mark_without_a_barline_lands_after_its_event() {
    let r = render(vec![quarter_note(
        4,
        NoteAnnotations {
            text_marks: vec![TextScript::above("etc.")],
            ..NoteAnnotations::default()
        },
    )]);
    let etc = text(&r.svg, "etc.");
    assert_close(etc.x, r.layout.elements[0].x + r.layout.elements[0].rod);
    assert!(glyphs(&r.svg, &r.font, Glyph::FermataAbove).is_empty());
}
