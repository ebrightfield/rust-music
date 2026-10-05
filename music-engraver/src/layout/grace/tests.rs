use super::*;

fn group(kind: GraceNoteKind, accidentals: bool) -> GraceGroup {
    GraceGroup {
        kind,
        slur: false,
        notes: vec![
            GraceNoteEvent {
                staff_position: 0,
                duration_log2: 3,
                dots: 0,
                accidental: accidentals
                    .then_some(ResolvedAccidental::plain(smufl::Glyph::AccidentalFlat)),
            },
            GraceNoteEvent {
                staff_position: -2,
                duration_log2: 3,
                dots: 0,
                accidental: None,
            },
        ],
    }
}

#[test]
fn c12_r009_beamed_graces_reserve_accidental_width_before_principal() {
    let font = crate::font::bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 6000.0, &config);
    let plain = group(GraceNoteKind::Appoggiatura, false);
    let altered = group(GraceNoteKind::Appoggiatura, true);
    let principal_x = 1200.0;
    let extent = grace_group_extent(&altered, StemDirection::Up, staff.staff_space);
    assert!(extent > grace_group_extent(&plain, StemDirection::Up, staff.staff_space));
    let layout = layout_grace_group(&altered, principal_x, StemDirection::Up, &staff, &config);
    assert_eq!(layout.left_x, principal_x - extent);
    assert_eq!(layout.notes.len(), 2);
    assert_eq!(layout.beams.len(), 1, "two eighths share one beam");
    assert!(layout
        .notes
        .iter()
        .all(|note| note.flags == 0 && note.stem.is_some()));
    let slashed = layout_grace_group(
        &group(GraceNoteKind::Acciaccatura, false),
        principal_x,
        StemDirection::Up,
        &staff,
        &config,
    );
    assert!(slashed.slashed);
    assert!(layout_grace_slur(
        &slashed,
        principal_x + 100.0,
        2,
        StemDirection::Up,
        &staff,
        &config
    )
    .is_some());
}
