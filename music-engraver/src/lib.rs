//! Native Rust music engraver — renders notation to SVG using SMuFL fonts.
//!
//! This crate consumes types from the [`music`] crate (`Pitch`, `Duration`, `Clef`)
//! and produces publication-quality SVG output using the Bravura music font (bundled).
//!
//! # Quick start
//!
//! Use [`score::ScoreBuilder`] for the simplest path from music types to SVG:
//!
//! ```no_run
//! use music::notation::clef::Clef;
//! use music::notation::rhythm::duration::Duration;
//! use music::note::pitch::Pitch;
//! use music::note::note::Note;
//! use music_engraver::layout::key_signature::KeySignature;
//! use music_engraver::score::ScoreBuilder;
//!
//! let svg = ScoreBuilder::new()
//!     .clef(Clef::Treble)
//!     .key_signature(KeySignature::Sharps(2))
//!     .time_signature(4, 4)
//!     .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::Fis, 4).unwrap(), Duration::QTR)
//!     .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
//!     .barline()
//!     .rest(Duration::WHOLE)
//!     .end_barline()
//!     .render_svg();
//! ```
//!
//! # Architecture
//!
//! - **[`font`]** — Font loading, glyph outline extraction, engraving configuration.
//!   Bravura OTF is bundled; glyph lookup is by SMuFL canonical name (`smufl::Glyph`).
//! - **[`layout`]** — Geometry and positioning: staff, clef, note placement, stems,
//!   beams, accidentals, dots, flags, rests, barlines, key/time signatures,
//!   measure/system/page layout.
//! - **[`render`]** — SVG generation: converts layout structs into SVG elements
//!   via [`render::SvgWriter`].
//! - **[`score`]** — High-level builder API bridging `music` crate types to the
//!   layout/render pipeline.

pub mod font;
pub mod layout;
pub mod render;
pub mod score;
