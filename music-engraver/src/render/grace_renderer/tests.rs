use super::*;

#[test]
fn c12_r009_group_engravings_include_grace_flat_beam_slash_and_slur() {
    use crate::layout::accidental::ResolvedAccidental;
    use crate::layout::grace::{
        layout_grace_group, GraceGroup, GraceNoteEvent, GraceNoteKind,
    };
    let font = crate::font::bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 6000.0, &config);
    let make_layout = |kind| {
        layout_grace_group(
            &GraceGroup {
                kind,
                slur: true,
                notes: vec![
                    GraceNoteEvent {
                        staff_position: 0,
                        duration_log2: 3,
                        dots: 0,
                        accidental: Some(ResolvedAccidental::plain(Glyph::AccidentalFlat)),
                    },
                    GraceNoteEvent {
                        staff_position: 2,
                        duration_log2: 3,
                        dots: 0,
                        accidental: None,
                    },
                ],
            },
            1200.0,
            StemDirection::Up,
            &staff,
            &config,
        )
    };
    let svg = |kind| {
        let mut writer = SvgWriter::new(800.0, 200.0, -300.0, -500.0, 6000.0, 2500.0);
        draw_grace_group(&mut writer, &staff, &font, &config, &make_layout(kind)).unwrap();
        writer.to_svg()
    };
    let beamed = svg(GraceNoteKind::Appoggiatura);
    let slashed = svg(GraceNoteKind::Acciaccatura);
    let flat = font.glyph_outline(Glyph::AccidentalFlat).unwrap().path_data;
    assert!(beamed.contains(&flat));
    assert!(beamed.contains("<polygon"), "two graces require a beam");
    assert!(
        slashed.matches("<line ").count() > beamed.matches("<line ").count(),
        "acciaccatura slash must cross its first stem"
    );
}
