//! SVG rendering for all notation elements.
//!
//! Each renderer module converts layout structs from [`crate::layout`] into SVG
//! elements via [`SvgWriter`]. Higher-level renderers ([`measure_renderer`],
//! [`system_renderer`], [`page_renderer`]) compose element renderers into
//! complete SVG documents.

pub mod accidental_renderer;
pub mod barline_renderer;
pub mod beam_renderer;
pub mod dot_renderer;
pub mod flag_renderer;
pub mod key_sig_renderer;
pub mod measure_renderer;
pub mod note_renderer;
pub mod page_renderer;
pub mod rest_renderer;
pub mod staff_renderer;
pub mod stem_renderer;
mod svg_writer;
pub mod system_renderer;
pub mod tie_renderer;
pub mod time_sig_renderer;

pub use accidental_renderer::draw_accidental;
pub use barline_renderer::draw_barline;
pub use beam_renderer::draw_beam_group;
pub use dot_renderer::draw_dots;
pub use flag_renderer::draw_flag;
pub use key_sig_renderer::draw_key_signature;
pub use measure_renderer::draw_measure;
pub use note_renderer::{draw_ledger_lines, draw_note, draw_notehead, draw_stemmed_note, NoteheadKind};
pub use page_renderer::draw_page;
pub use rest_renderer::draw_rest;
pub use staff_renderer::{draw_clef, draw_staff_lines};
pub use stem_renderer::draw_stem;
pub use svg_writer::SvgWriter;
pub use system_renderer::draw_system;
pub use tie_renderer::draw_tie;
pub use time_sig_renderer::draw_time_signature;
