/// Tempo marking layout — metronome marks and tempo text above the staff.
///
/// Standard engraving convention: tempo markings appear above the staff,
/// left-aligned with the beat they apply to. Two common forms:
/// - Text only: "Allegro", "Andante", etc. (bold)
/// - Metronome mark: "♩ = 120" or combined "Allegro ♩ = 120"
///
/// For metronome marks, the note symbol is rendered as a SMuFL glyph
/// (noteheadBlack + stem) at the appropriate size, with " = BPM" as text.
use crate::layout::staff::StaffLayout;

/// The kind of note value used in a metronome mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetronomeNoteKind {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

/// A tempo marking specification.
#[derive(Debug, Clone, PartialEq)]
pub enum TempoMark {
    /// Text-only tempo indication (e.g. "Allegro").
    Text(String),
    /// Metronome mark only (e.g. ♩ = 120).
    Metronome {
        note_kind: MetronomeNoteKind,
        /// Whether the note value is dotted.
        dotted: bool,
        bpm: u16,
    },
    /// Combined text and metronome (e.g. "Allegro ♩ = 120").
    TextWithMetronome {
        text: String,
        note_kind: MetronomeNoteKind,
        dotted: bool,
        bpm: u16,
    },
}

/// Result of laying out a tempo marking.
#[derive(Debug, Clone)]
pub struct TempoMarkLayout {
    /// Left x-position of the tempo marking.
    pub x_left: f64,
    /// Baseline y-position (above the staff).
    pub y_baseline: f64,
    /// Font size for the text, in font design units.
    pub font_size: f64,
    /// The text to render (e.g. "Allegro", or "= 120" when a glyph precedes it).
    pub text: String,
    /// Whether to render bold text.
    pub bold: bool,
    /// Optional metronome glyph info: (notehead_glyph, has_stem, has_flag, dotted, x_offset, text_after).
    /// The note symbol is drawn at the left edge, then the "= BPM" text follows.
    pub metronome: Option<MetronomeInfo>,
}

/// Info for rendering a metronome note symbol.
#[derive(Debug, Clone)]
pub struct MetronomeInfo {
    /// The SMuFL glyph for the notehead.
    pub notehead_glyph: smufl::Glyph,
    /// Whether to draw a stem on the notehead.
    pub has_stem: bool,
    /// Number of flags (0 for quarter/half/whole, 1 for eighth, 2 for sixteenth).
    pub flag_count: u8,
    /// Whether the note symbol is dotted.
    pub dotted: bool,
    /// X-position of the note symbol.
    pub note_x: f64,
    /// The "= BPM" text that follows the note symbol.
    pub eq_text: String,
    /// X-position for the "= BPM" text (after the note symbol).
    pub eq_text_x: f64,
}

/// Distance above the top staff line for tempo marking placement, in staff spaces.
const TEMPO_ABOVE_STAFF_SS: f64 = 2.8;

/// Font size for tempo text, in staff spaces.
const TEMPO_FONT_SIZE_SS: f64 = 1.6;

/// Approximate width of one character in the tempo font, as a fraction of font_size.
const CHAR_WIDTH_RATIO: f64 = 0.55;

/// Width of the note symbol area (notehead + stem + optional flag), in staff spaces.
const NOTE_SYMBOL_WIDTH_SS: f64 = 1.4;

/// Spacing between text and metronome symbol, in staff spaces.
const TEXT_METRONOME_GAP_SS: f64 = 0.4;

impl MetronomeNoteKind {
    /// SMuFL glyph for the notehead at this duration.
    pub fn notehead_glyph(self) -> smufl::Glyph {
        match self {
            MetronomeNoteKind::Whole => smufl::Glyph::MetNoteWhole,
            MetronomeNoteKind::Half => smufl::Glyph::MetNoteHalfUp,
            MetronomeNoteKind::Quarter => smufl::Glyph::MetNoteQuarterUp,
            MetronomeNoteKind::Eighth => smufl::Glyph::MetNote8thUp,
            MetronomeNoteKind::Sixteenth => smufl::Glyph::MetNote16thUp,
        }
    }
}

/// Lay out a tempo marking above the staff.
///
/// `x_left` is the left-edge horizontal position (aligned with the note/beat it applies to).
/// `staff` provides vertical reference for placement above the top staff line.
/// `staff_space` is the staff space size in font design units.
pub fn layout_tempo_mark(
    mark: &TempoMark,
    x_left: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> TempoMarkLayout {
    let font_size = TEMPO_FONT_SIZE_SS * staff_space;
    let above_offset = TEMPO_ABOVE_STAFF_SS * staff_space;
    let top_line_y = staff.y_of(8);
    let y_baseline = top_line_y - above_offset;

    match mark {
        TempoMark::Text(text) => TempoMarkLayout {
            x_left,
            y_baseline,
            font_size,
            text: text.clone(),
            bold: true,
            metronome: None,
        },
        TempoMark::Metronome {
            note_kind,
            dotted,
            bpm,
        } => {
            let note_width = NOTE_SYMBOL_WIDTH_SS * staff_space;
            let eq_text = format!(" = {bpm}");
            let eq_text_x = x_left + note_width;

            TempoMarkLayout {
                x_left,
                y_baseline,
                font_size,
                text: String::new(),
                bold: true,
                metronome: Some(MetronomeInfo {
                    notehead_glyph: note_kind.notehead_glyph(),
                    has_stem: !matches!(note_kind, MetronomeNoteKind::Whole),
                    flag_count: match note_kind {
                        MetronomeNoteKind::Eighth => 1,
                        MetronomeNoteKind::Sixteenth => 2,
                        _ => 0,
                    },
                    dotted: *dotted,
                    note_x: x_left,
                    eq_text,
                    eq_text_x,
                }),
            }
        }
        TempoMark::TextWithMetronome {
            text,
            note_kind,
            dotted,
            bpm,
        } => {
            let text_width = text.len() as f64 * font_size * CHAR_WIDTH_RATIO;
            let gap = TEXT_METRONOME_GAP_SS * staff_space;
            let note_x = x_left + text_width + gap;
            let note_width = NOTE_SYMBOL_WIDTH_SS * staff_space;
            let eq_text = format!(" = {bpm}");
            let eq_text_x = note_x + note_width;

            TempoMarkLayout {
                x_left,
                y_baseline,
                font_size,
                text: text.clone(),
                bold: true,
                metronome: Some(MetronomeInfo {
                    notehead_glyph: note_kind.notehead_glyph(),
                    has_stem: !matches!(note_kind, MetronomeNoteKind::Whole),
                    flag_count: match note_kind {
                        MetronomeNoteKind::Eighth => 1,
                        MetronomeNoteKind::Sixteenth => 2,
                        _ => 0,
                    },
                    dotted: *dotted,
                    note_x,
                    eq_text,
                    eq_text_x,
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    #[test]
    fn text_only_has_no_metronome() {
        let layout = layout_tempo_mark(
            &TempoMark::Text("Allegro".into()),
            100.0,
            &test_staff(),
            250.0,
        );
        assert!(layout.metronome.is_none());
        assert_eq!(layout.text, "Allegro");
        assert!(layout.bold);
    }

    #[test]
    fn metronome_only_has_empty_text() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            100.0,
            &test_staff(),
            250.0,
        );
        assert!(layout.text.is_empty());
        assert!(layout.metronome.is_some());
        let m = layout.metronome.unwrap();
        assert_eq!(m.eq_text, " = 120");
        assert!(m.has_stem);
        assert_eq!(m.flag_count, 0);
        assert!(!m.dotted);
    }

    #[test]
    fn combined_text_and_metronome() {
        let layout = layout_tempo_mark(
            &TempoMark::TextWithMetronome {
                text: "Allegro".into(),
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 132,
            },
            50.0,
            &test_staff(),
            250.0,
        );
        assert_eq!(layout.text, "Allegro");
        assert!(layout.metronome.is_some());
        let m = layout.metronome.unwrap();
        assert_eq!(m.eq_text, " = 132");
        // Note symbol should be to the right of the text
        assert!(
            m.note_x > 50.0,
            "note_x {} should be right of x_left 50",
            m.note_x
        );
    }

    #[test]
    fn baseline_above_top_staff_line() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout = layout_tempo_mark(&TempoMark::Text("Allegro".into()), 0.0, &staff, 250.0);
        assert!(
            layout.y_baseline < top_y,
            "baseline {} should be above top line {}",
            layout.y_baseline,
            top_y
        );
    }

    #[test]
    fn x_left_preserved() {
        let layout = layout_tempo_mark(
            &TempoMark::Text("Presto".into()),
            777.0,
            &test_staff(),
            250.0,
        );
        assert_eq!(layout.x_left, 777.0);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_tempo_mark(&TempoMark::Text("A".into()), 0.0, &test_staff(), 125.0);
        let large = layout_tempo_mark(&TempoMark::Text("A".into()), 0.0, &test_staff(), 250.0);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn eighth_note_has_one_flag() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Eighth,
                dotted: false,
                bpm: 88,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let m = layout.metronome.unwrap();
        assert_eq!(m.flag_count, 1);
        assert!(m.has_stem);
    }

    #[test]
    fn sixteenth_note_has_two_flags() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Sixteenth,
                dotted: false,
                bpm: 100,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let m = layout.metronome.unwrap();
        assert_eq!(m.flag_count, 2);
    }

    #[test]
    fn whole_note_has_no_stem() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Whole,
                dotted: false,
                bpm: 60,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let m = layout.metronome.unwrap();
        assert!(!m.has_stem);
        assert_eq!(m.flag_count, 0);
    }

    #[test]
    fn dotted_flag_preserved() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: true,
                bpm: 72,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let m = layout.metronome.unwrap();
        assert!(m.dotted);
    }

    #[test]
    fn eq_text_x_is_right_of_note_x() {
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            100.0,
            &test_staff(),
            250.0,
        );
        let m = layout.metronome.unwrap();
        assert!(
            m.eq_text_x > m.note_x,
            "eq_text_x {} should be right of note_x {}",
            m.eq_text_x,
            m.note_x
        );
    }

    #[test]
    fn different_bpm_produces_different_eq_text() {
        let a = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 60,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let b = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        assert_ne!(a.metronome.unwrap().eq_text, b.metronome.unwrap().eq_text);
    }

    #[test]
    fn different_note_kinds_produce_different_glyphs() {
        let q = MetronomeNoteKind::Quarter.notehead_glyph();
        let h = MetronomeNoteKind::Half.notehead_glyph();
        let e = MetronomeNoteKind::Eighth.notehead_glyph();
        assert_ne!(q, h);
        assert_ne!(q, e);
        assert_ne!(h, e);
    }

    #[test]
    fn combined_note_x_shifts_with_longer_text() {
        let short = layout_tempo_mark(
            &TempoMark::TextWithMetronome {
                text: "A".into(),
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let long = layout_tempo_mark(
            &TempoMark::TextWithMetronome {
                text: "Allegro molto".into(),
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        let short_x = short.metronome.unwrap().note_x;
        let long_x = long.metronome.unwrap().note_x;
        assert!(
            long_x > short_x,
            "longer text should push note symbol further right: {} vs {}",
            long_x,
            short_x
        );
    }
}
