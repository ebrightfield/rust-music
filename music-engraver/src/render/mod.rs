//! SVG rendering for all notation elements.
//!
//! Each renderer module converts layout structs from [`crate::layout`] into SVG
//! elements via [`SvgWriter`]. Higher-level renderers ([`measure_renderer`],
//! [`system_renderer`], [`page_renderer`]) compose element renderers into
//! complete SVG documents.

pub mod accidental_renderer;
pub mod arpeggio_renderer;
pub mod articulation_renderer;
pub mod barline_renderer;
pub mod beam_renderer;
pub(crate) mod bend_gesture_renderer;
pub mod breath_renderer;
pub mod chord_symbol_renderer;
pub mod church_rest_renderer;
pub mod dot_renderer;
pub mod dynamics_renderer;
pub mod flag_renderer;
pub mod glissando_renderer;
pub mod grace_renderer;
pub mod hairpin_renderer;
pub mod key_sig_renderer;
pub mod lyric_renderer;
pub mod measure_renderer;
pub mod multi_measure_rest_renderer;
pub mod multi_staff_renderer;
pub mod navigation_renderer;
pub mod note_renderer;
pub mod ornament_renderer;
pub mod ottava_renderer;
pub mod page_renderer;
pub mod pedal_renderer;
pub mod rehearsal_renderer;
pub mod rest_renderer;
pub mod slur_renderer;
pub mod staff_renderer;
pub mod stem_renderer;
mod svg_writer;
pub mod system_renderer;
pub mod tab_beam_renderer;
pub mod tab_hammer_renderer;
pub mod tab_harmonic_renderer;
pub mod tab_let_ring_renderer;
pub mod tab_palm_mute_renderer;
pub mod tab_renderer;
pub mod tab_rhythm_renderer;
pub mod tab_slide_renderer;
pub mod tab_vibrato_renderer;
pub mod tempo_renderer;
pub mod text_line_renderer;
pub mod text_spanner_renderer;
pub mod tie_renderer;
pub mod time_sig_renderer;
pub mod tremolo_renderer;
pub mod trill_bracket_renderer;
pub mod trill_extension_renderer;
pub mod tuplet_renderer;
pub mod volta_renderer;

pub use accidental_renderer::draw_accidental;
pub use arpeggio_renderer::draw_arpeggio;
pub use articulation_renderer::draw_articulation;
pub use barline_renderer::draw_barline;
pub use beam_renderer::draw_beam_group;
pub use breath_renderer::draw_breath_mark;
pub use chord_symbol_renderer::{draw_chord_symbol, draw_chord_symbol_composite};
pub use church_rest_renderer::draw_church_rest;
pub use dot_renderer::draw_dots;
pub use dynamics_renderer::draw_dynamic;
pub use flag_renderer::draw_flag;
pub use glissando_renderer::draw_glissando;
pub use grace_renderer::draw_grace_note;
pub use hairpin_renderer::draw_hairpin;
pub use key_sig_renderer::draw_key_signature;
pub use lyric_renderer::draw_lyric;
pub use measure_renderer::draw_measure;
pub use multi_measure_rest_renderer::draw_multi_measure_rest;
pub use multi_staff_renderer::{
    draw_brace, draw_bracket, draw_joined_barline, draw_multi_staff_connectors,
};
pub use navigation_renderer::draw_navigation_sign;
pub use note_renderer::{
    draw_ledger_lines, draw_note, draw_notehead, draw_stemmed_note, draw_styled_notehead,
    notehead_advance, NoteheadKind,
};
pub use ornament_renderer::draw_ornament;
pub use ottava_renderer::draw_ottava_bracket;
pub use page_renderer::draw_page;
pub use pedal_renderer::draw_pedal;
pub use rehearsal_renderer::draw_rehearsal_mark;
pub use rest_renderer::draw_rest;
pub use slur_renderer::draw_slur;
pub use staff_renderer::{draw_clef, draw_staff_lines};
pub use stem_renderer::draw_stem;
pub use svg_writer::{RectStyle, SvgWriter, TextStyle};
pub use system_renderer::draw_system;
pub use tab_beam_renderer::draw_tab_beam_group;
pub use tab_hammer_renderer::draw_tab_legato;
pub use tab_harmonic_renderer::draw_tab_harmonic;
pub use tab_let_ring_renderer::{draw_tab_let_ring, draw_tab_let_ring_dash};
pub use tab_palm_mute_renderer::{draw_tab_palm_mute, draw_tab_palm_mute_dash};
pub use tab_renderer::{
    draw_fret_number, draw_fret_number_at, draw_tab_clef, draw_tab_staff_lines,
};
pub use tab_rhythm_renderer::draw_tab_rhythm;
pub use tab_slide_renderer::draw_tab_slide;
pub use tab_vibrato_renderer::draw_tab_vibrato;
pub use tempo_renderer::draw_tempo_mark;
pub use text_line_renderer::draw_text_line;
pub use text_spanner_renderer::draw_text_spanner;
pub use tie_renderer::draw_tie;
pub use time_sig_renderer::draw_time_signature;
pub use tremolo_renderer::draw_tremolo;
pub use trill_bracket_renderer::{draw_trill_bracket_hook, draw_trill_bracket_hooks};
pub use trill_extension_renderer::draw_trill_extension;
pub use tuplet_renderer::draw_tuplet_bracket;
pub use volta_renderer::draw_volta_bracket;

#[cfg(feature = "png")]
pub mod png;
