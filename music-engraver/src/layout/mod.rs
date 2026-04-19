//! Geometry and positioning for all notation elements.
//!
//! Each submodule handles one element type (staff, clef, notehead, stem, beam,
//! accidental, dot, flag, rest, barline, key/time signature) and produces
//! layout structs consumed by the corresponding renderer in [`crate::render`].
//! Higher-level modules ([`measure`], [`system`], [`page`]) compose elements
//! into complete scored layouts.

pub mod accidental;
pub mod barline;
pub mod beam;
pub mod chord;
pub mod clef;
pub mod dot;
pub mod dynamics;
pub mod flag;
pub mod hairpin;
pub mod key_signature;
pub mod measure;
pub mod note_placement;
pub mod page;
pub mod rest;
pub mod slur;
pub mod staff;
pub mod stem;
pub mod system;
pub mod tie;
pub mod time_signature;
pub mod tuplet;

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
pub use dynamics::{layout_dynamic, Dynamic, DynamicLayout, DYNAMICS_BELOW_STAFF_SS};
pub use hairpin::{layout_hairpin, HairpinLayout, HairpinType, HAIRPIN_BELOW_STAFF_SS};
pub use slur::{layout_slur, slur_direction_from_stem, SlurDirection, SlurLayout};
pub use time_signature::{TimeSignatureKind, TimeSignatureLayout};
pub use tuplet::{
    layout_tuplet_bracket, tuplet_number_glyphs, tuplet_placement_from_stem, TupletBracketLayout,
    TupletPlacement,
};
