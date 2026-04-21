//! SVG rendering for all notation elements.
//!
//! Each renderer module converts layout structs from [`crate::layout`] into SVG
//! elements via [`SvgWriter`]. Higher-level renderers ([`measure_renderer`],
//! [`system_renderer`], [`page_renderer`]) compose element renderers into
//! complete SVG documents.

pub mod accidental_renderer;
pub mod articulation_renderer;
pub mod barline_renderer;
pub mod beam_renderer;
pub mod chord_symbol_renderer;
pub mod dot_renderer;
pub mod dynamics_renderer;
pub mod expression_renderer;
pub mod flag_renderer;
pub mod grace_renderer;
pub mod hairpin_renderer;
pub mod lyric_renderer;
pub mod key_sig_renderer;
pub mod measure_renderer;
pub mod multi_staff_renderer;
pub mod note_renderer;
pub mod ornament_renderer;
pub mod page_renderer;
pub mod rehearsal_renderer;
pub mod rest_renderer;
pub mod slur_renderer;
pub mod staff_renderer;
pub mod stem_renderer;
mod svg_writer;
pub mod system_renderer;
pub mod tab_renderer;
pub mod tab_beam_renderer;
pub mod tab_rhythm_renderer;
pub mod tempo_renderer;
pub mod tie_renderer;
pub mod time_sig_renderer;
pub mod tuplet_renderer;

pub use accidental_renderer::draw_accidental;
pub use articulation_renderer::draw_articulation;
pub use chord_symbol_renderer::draw_chord_symbol;
pub use barline_renderer::draw_barline;
pub use beam_renderer::draw_beam_group;
pub use dot_renderer::draw_dots;
pub use dynamics_renderer::draw_dynamic;
pub use expression_renderer::draw_expression;
pub use hairpin_renderer::draw_hairpin;
pub use lyric_renderer::draw_lyric;
pub use flag_renderer::draw_flag;
pub use grace_renderer::draw_grace_note;
pub use key_sig_renderer::draw_key_signature;
pub use measure_renderer::draw_measure;
pub use note_renderer::{draw_ledger_lines, draw_note, draw_notehead, draw_stemmed_note, NoteheadKind};
pub use multi_staff_renderer::{draw_brace, draw_bracket, draw_joined_barline, draw_multi_staff_connectors};
pub use ornament_renderer::draw_ornament;
pub use page_renderer::draw_page;
pub use rehearsal_renderer::draw_rehearsal_mark;
pub use rest_renderer::draw_rest;
pub use staff_renderer::{draw_clef, draw_staff_lines};
pub use stem_renderer::draw_stem;
pub use svg_writer::{RectStyle, SvgWriter, TextStyle};
pub use system_renderer::draw_system;
pub use slur_renderer::draw_slur;
pub use tempo_renderer::draw_tempo_mark;
pub use tie_renderer::draw_tie;
pub use time_sig_renderer::draw_time_signature;
pub use tab_renderer::{draw_fret_number, draw_fret_number_at, draw_tab_clef, draw_tab_staff_lines};
pub use tab_beam_renderer::draw_tab_beam_group;
pub use tab_rhythm_renderer::draw_tab_rhythm;
pub use tuplet_renderer::draw_tuplet_bracket;

#[cfg(feature = "png")]
pub mod png;
