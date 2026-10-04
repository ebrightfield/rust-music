//! Geometry and positioning for all notation elements.
//!
//! Each submodule handles one element type (staff, clef, notehead, stem, beam,
//! accidental, dot, flag, rest, barline, key/time signature) and produces
//! layout structs consumed by the corresponding renderer in [`crate::render`].
//! Higher-level modules ([`measure`], [`system`], [`page`]) compose elements
//! into complete scored layouts.

pub mod accidental;
pub mod arpeggio;
pub mod articulation;
pub mod barline;
pub mod beam;
pub(crate) mod bend_gesture;
pub mod breath;
pub mod chord;
pub mod chord_symbol;
pub mod clef;
pub mod cresc_text;
pub mod dot;
pub mod dynamics;
pub mod expression;
pub mod flag;
pub mod glissando;
pub mod grace;
pub mod hairpin;
pub mod key_signature;
pub mod lyric;
pub mod measure;
pub mod multi_measure_rest;
pub mod multi_staff;
pub mod navigation;
pub mod note_placement;
pub mod ornament;
pub mod ottava;
pub mod page;
pub mod pedal;
pub mod rehearsal;
pub mod rest;
pub mod slur;
pub mod staff;
pub mod stem;
pub mod system;
pub mod tab;
pub mod tab_beam;
pub mod tab_hammer;
pub mod tab_harmonic;
pub mod tab_let_ring;
pub mod tab_palm_mute;
pub mod tab_rhythm;
pub mod tab_slide;
pub mod tab_vibrato;
pub mod tempo;
pub mod tie;
pub mod time_signature;
pub mod tremolo;
pub mod trill_bracket;
pub mod trill_extension;
pub mod trill_options;
pub mod tuplet;
pub mod voice_collision;
pub mod volta;

pub use accidental::{AccidentalDisplay, ResolvedAccidental};
pub use arpeggio::{layout_arpeggio, ArpeggioDirection, ArpeggioLayout, ARPEGGIO_PADDING_SS};
pub use articulation::{
    layout_articulation, layout_articulation_stack, Articulation, ArticulationLayout,
    ArticulationPlacement,
};
pub use barline::{BarlineLayout, BarlineStyle};
pub use beam::{
    beam_group_stem_direction, compute_beam_counts, layout_beam_group, BeamGroupLayout, BeamedNote,
};
pub use breath::{
    layout_breath_mark, BreathMark, BreathMarkLayout, BREATH_MARK_ABOVE_STAFF_SS,
    BREATH_MARK_RIGHT_PADDING_SS,
};
pub use chord::{
    chord_extent, chord_has_offsets, chord_left_notehead_offset, layout_chord_noteheads,
    notehead_x_offset, ChordNote, ChordNoteLayout,
};
pub use chord_symbol::{
    layout_chord_symbol, layout_chord_symbol_composite, parse_chord_symbol_segments,
    ChordSymbolCompositeLayout, ChordSymbolLayout, ChordSymbolSegment, ChordSymbolSegmentBox,
    ACCIDENTAL_BASELINE_RAISE_FACTOR, ACCIDENTAL_SIDE_BEARING_FACTOR, ACCIDENTAL_SIZE_FACTOR,
    CHORD_SYMBOL_ABOVE_STAFF_SS, CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR,
};
pub use clef::ClefLayout;
pub use dot::{dot_staff_position, dot_xs};
pub use dynamics::{layout_dynamic, Dynamic, DynamicLayout, DYNAMICS_BELOW_STAFF_SS};
pub use expression::{layout_expression, ExpressionLayout};
pub use flag::flag_glyph;
pub use glissando::{
    layout_glissando, layout_half_glissando_left, layout_half_glissando_right, GlissandoLayout,
    GlissandoStyle, GLISSANDO_H_PADDING_SS,
};
pub use grace::{
    grace_note_glyph, grace_note_x_reservation, layout_grace_note, GraceNoteKind, GraceNoteLayout,
    GRACE_NOTE_SCALE, GRACE_NOTE_SPACING_SS,
};
pub use hairpin::{
    layout_hairpin, layout_hairpin_dashed, layout_hairpin_styled, layout_hairpin_with_niente,
    layout_hairpin_with_niente_at_open_end, HairpinDashStyle, HairpinLayout, HairpinType,
    NienteCircleLayout, NientePlacement, HAIRPIN_BELOW_STAFF_SS, HAIRPIN_DASH_LENGTH_SS,
    HAIRPIN_GAP_LENGTH_SS, HAIRPIN_NIENTE_RADIUS_SS,
};
pub use key_signature::{KeySignature, KeySignatureLayout};
pub use lyric::{
    layout_lyric, LyricContinuation, LyricLayout, LyricSyllable, LYRIC_BELOW_STAFF_SS,
};
pub use measure::{
    layout_measure, MeasureElement, MeasureLayout, MeasureLayoutConfig, NoteEvent,
    PositionedElement, RestEvent,
};
pub use multi_measure_rest::{
    church_rest_supported, layout_church_rest, layout_multi_measure_rest, ChurchRestGlyph,
    ChurchRestLayout, MultiMeasureRestLayout, MultiMeasureRestStyle,
    CHURCH_REST_COUNT_ABOVE_STAFF_SS, CHURCH_REST_GLYPH_SPACING_SS, CHURCH_REST_MAX_COUNT,
    COUNT_ABOVE_HBAR_SS, COUNT_FONT_SIZE_SS, HBAR_BAR_THICKNESS_SS, HBAR_HALF_HEIGHT_SS,
    HBAR_HORIZONTAL_PADDING_SS, HBAR_SERIF_THICKNESS_SS,
};
pub use multi_staff::{
    layout_multi_staff, staff_layouts_from_multi, BraceLayout, BracketLayout, ConnectorKind,
    MultiStaffLayout, StaffGroup, SubBracket, SubBracketLayout,
};
pub use navigation::{layout_navigation_sign, NavigationSign, NavigationSignLayout};
pub use note_placement::pitch_to_staff_position;
pub use ornament::{layout_ornament, Ornament, OrnamentLayout};
pub use ottava::{layout_ottava_bracket, OttavaBracketLayout, OttavaKind, OTTAVA_ABOVE_STAFF_SS};
pub use page::{layout_page, PageLayout, PageLayoutConfig, PageSystem, SystemBreaking};
pub use pedal::{layout_pedal, PedalLayout, PedalMark, PEDAL_BELOW_STAFF_SS};
pub use rehearsal::{layout_rehearsal_mark, RehearsalMarkLayout, RehearsalStyle};
pub use slur::{layout_slur, slur_direction_from_stem, SlurDirection, SlurLayout};
pub use staff::{StaffLayout, StaffPosition, BOTTOM_LINE, STANDARD_LINE_COUNT, TOP_LINE};
pub use stem::{auto_stem_direction, auto_stem_direction_chord, StemDirection};
pub use system::{
    layout_system, ClefKind, MeasureContent, MeasureEvent, SystemLayout, SystemMeasure,
    SystemPrefix,
};
pub use tab::{
    layout_fret_number, tab_clef_glyph, FretNumberLayout, TabStaffLayout,
    FRET_NUMBER_FONT_SIZE_RATIO, TAB_4_STRING_LINE_COUNT, TAB_LINE_COUNT,
};
pub use tab_beam::{
    compute_tab_beam_counts, layout_tab_beam_group, TabBeamGroupLayout, TabBeamedNote,
};
pub use tab_hammer::{layout_tab_legato, LegatoKind, TabLegatoLayout};
pub use tab_harmonic::{
    layout_tab_harmonic, TabHarmonicLayout, HARMONIC_ABOVE_FRET_SS, HARMONIC_GLYPH_SCALE,
};
pub use tab_let_ring::{
    layout_tab_let_ring, layout_tab_let_ring_dash, TabLetRingDashLayout, TabLetRingLayout,
    LET_RING_ABOVE_STAFF_SS, LET_RING_DASH_OFFSET_SS, LET_RING_FONT_SIZE_RATIO,
};
pub use tab_palm_mute::{
    layout_tab_palm_mute, layout_tab_palm_mute_dash, TabPalmMuteDashLayout, TabPalmMuteLayout,
    PALM_MUTE_ABOVE_STAFF_SS, PALM_MUTE_DASH_OFFSET_SS, PALM_MUTE_FONT_SIZE_RATIO,
};
pub use tab_rhythm::{layout_tab_rhythm, needs_stem, tab_flag_count, TabRhythmLayout};
pub use tab_slide::{layout_tab_slide, TabSlideLayout};
pub use tab_vibrato::{layout_tab_vibrato, TabVibratoLayout, VibratoKind};
pub use tempo::{layout_tempo_mark, MetronomeNoteKind, TempoMark, TempoMarkLayout};
pub use tie::{
    layout_half_tie_left, layout_half_tie_right, layout_tie, tie_direction_from_stem, TieDirection,
    TieLayout,
};
pub use time_signature::{TimeSignatureKind, TimeSignatureLayout};
pub use tremolo::{layout_tremolo, TremoloCount, TremoloLayout};
pub use trill_bracket::{
    layout_trill_bracket_hook, layout_trill_bracket_hooks, layout_trill_bracket_hooks_multi_speed,
    HookDirection, TrillBracketHookLayout, TrillBracketOptions, TrillBracketSide,
};
pub use trill_extension::{
    layout_trill_extension, layout_trill_extension_multi_speed, layout_trill_extension_with_glyph,
    multi_speed_trill_extension_right_edge, trill_extension_right_edge,
    MultiSpeedTrillExtensionLayout, TrillExtensionLayout, TrillExtensionSpeedOptions,
    TrillExtensionTile, TrillSpeedRamp, TrillSpeedRampSpec, TrillSpeedRegion, TrillWiggleSpeed,
};
pub use trill_options::TrillExtensionFullOptions;
pub use tuplet::{
    layout_tuplet_bracket, tuplet_number_glyphs, tuplet_placement_from_stem, TupletBracketLayout,
    TupletPlacement,
};
pub use voice_collision::{compute_voice_collision_offsets, VoiceCollisionOffset};
pub use volta::{
    layout_volta_bracket, VoltaAnnotation, VoltaBracketLayout, VoltaHooks, VOLTA_ABOVE_STAFF_SS,
};
