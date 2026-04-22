//! Geometry and positioning for all notation elements.
//!
//! Each submodule handles one element type (staff, clef, notehead, stem, beam,
//! accidental, dot, flag, rest, barline, key/time signature) and produces
//! layout structs consumed by the corresponding renderer in [`crate::render`].
//! Higher-level modules ([`measure`], [`system`], [`page`]) compose elements
//! into complete scored layouts.

pub mod accidental;
pub mod articulation;
pub mod barline;
pub mod beam;
pub mod chord;
pub mod chord_symbol;
pub mod clef;
pub mod dot;
pub mod dynamics;
pub mod expression;
pub mod flag;
pub mod grace;
pub mod hairpin;
pub mod lyric;
pub mod key_signature;
pub mod measure;
pub mod multi_staff;
pub mod navigation;
pub mod note_placement;
pub mod ottava;
pub mod ornament;
pub mod page;
pub mod pedal;
pub mod rehearsal;
pub mod rest;
pub mod slur;
pub mod staff;
pub mod stem;
pub mod tab;
pub mod tab_beam;
pub mod tab_rhythm;
pub mod tab_bend;
pub mod tab_hammer;
pub mod tab_slide;
pub mod tab_harmonic;
pub mod tab_vibrato;
pub mod system;
pub mod tempo;
pub mod tie;
pub mod tremolo;
pub mod time_signature;
pub mod tuplet;
pub mod volta;

pub use barline::{BarlineLayout, BarlineStyle};
pub use beam::{
    beam_group_stem_direction, compute_beam_counts, layout_beam_group, BeamGroupLayout, BeamedNote,
};
pub use chord::{
    chord_extent, chord_has_offsets, layout_chord_noteheads, notehead_x_offset, ChordNote,
    ChordNoteLayout,
};
pub use clef::ClefLayout;
pub use dot::{dot_staff_position, dot_xs};
pub use flag::flag_glyph;
pub use key_signature::{KeySignature, KeySignatureLayout};
pub use measure::{
    layout_measure, MeasureElement, MeasureLayout, MeasureLayoutConfig, NoteEvent, PositionedElement,
    RestEvent,
};
pub use note_placement::pitch_to_staff_position;
pub use page::{layout_page, PageLayout, PageLayoutConfig, PageSystem, SystemBreaking};
pub use staff::{StaffLayout, StaffPosition, BOTTOM_LINE, STANDARD_LINE_COUNT, TOP_LINE};
pub use stem::{auto_stem_direction, auto_stem_direction_chord, StemDirection};
pub use system::{
    layout_system, ClefKind, MeasureContent, MeasureEvent, SystemLayout, SystemMeasure, SystemPrefix,
};
pub use tie::{
    layout_half_tie_left, layout_half_tie_right, layout_tie, tie_direction_from_stem, TieDirection,
    TieLayout,
};
pub use multi_staff::{
    layout_multi_staff, staff_layouts_from_multi, BraceLayout, BracketLayout, ConnectorKind,
    MultiStaffLayout, StaffGroup,
};
pub use articulation::{
    layout_articulation, Articulation, ArticulationLayout, ArticulationPlacement,
};
pub use dynamics::{layout_dynamic, Dynamic, DynamicLayout, DYNAMICS_BELOW_STAFF_SS};
pub use expression::{layout_expression, ExpressionLayout};
pub use grace::{
    grace_note_glyph, grace_note_x_reservation, layout_grace_note, GraceNoteKind, GraceNoteLayout,
    GRACE_NOTE_SCALE, GRACE_NOTE_SPACING_SS,
};
pub use hairpin::{layout_hairpin, HairpinLayout, HairpinType, HAIRPIN_BELOW_STAFF_SS};
pub use lyric::{layout_lyric, LyricContinuation, LyricLayout, LyricSyllable, LYRIC_BELOW_STAFF_SS};
pub use slur::{layout_slur, slur_direction_from_stem, SlurDirection, SlurLayout};
pub use time_signature::{TimeSignatureKind, TimeSignatureLayout};
pub use rehearsal::{layout_rehearsal_mark, RehearsalMarkLayout, RehearsalStyle};
pub use tempo::{layout_tempo_mark, MetronomeNoteKind, TempoMark, TempoMarkLayout};
pub use chord_symbol::{layout_chord_symbol, ChordSymbolLayout, CHORD_SYMBOL_ABOVE_STAFF_SS};
pub use navigation::{layout_navigation_sign, NavigationSign, NavigationSignLayout};
pub use ornament::{layout_ornament, Ornament, OrnamentLayout};
pub use ottava::{layout_ottava_bracket, OttavaBracketLayout, OttavaKind, OTTAVA_ABOVE_STAFF_SS};
pub use pedal::{layout_pedal, PedalLayout, PedalMark, PEDAL_BELOW_STAFF_SS};
pub use tab_beam::{
    compute_tab_beam_counts, layout_tab_beam_group, TabBeamGroupLayout, TabBeamedNote,
};
pub use tab_rhythm::{layout_tab_rhythm, needs_stem, tab_flag_count, TabRhythmLayout};
pub use tab::{
    layout_fret_number, tab_clef_glyph, FretNumberLayout, TabStaffLayout, FRET_NUMBER_FONT_SIZE_RATIO,
    TAB_4_STRING_LINE_COUNT, TAB_LINE_COUNT,
};
pub use tab_bend::{layout_tab_bend, BendAmount, TabBendLayout};
pub use tab_hammer::{layout_tab_legato, LegatoKind, TabLegatoLayout};
pub use tab_slide::{layout_tab_slide, TabSlideLayout};
pub use tab_harmonic::{layout_tab_harmonic, TabHarmonicLayout, HARMONIC_ABOVE_FRET_SS, HARMONIC_GLYPH_SCALE};
pub use tab_vibrato::{layout_tab_vibrato, TabVibratoLayout, VibratoKind};
pub use volta::{
    layout_volta_bracket, VoltaAnnotation, VoltaBracketLayout, VoltaHooks, VOLTA_ABOVE_STAFF_SS,
};
pub use tremolo::{layout_tremolo, TremoloCount, TremoloLayout};
pub use tuplet::{
    layout_tuplet_bracket, tuplet_number_glyphs, tuplet_placement_from_stem, TupletBracketLayout,
    TupletPlacement,
};
