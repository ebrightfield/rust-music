use crate::font::EngravingConfig;
use crate::layout::staff::StaffLayout;

/// Visual style of a barline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarlineStyle {
    /// Single thin barline (most common, end of each measure).
    Single,
    /// Double thin barline (section boundary, key/time change).
    Double,
    /// Final barline: thin then thick (end of piece).
    Final,
    /// Start repeat: thick then thin, with dots.
    StartRepeat,
    /// End repeat: thin then thick, with dots.
    EndRepeat,
}

/// Describes one vertical stroke in a barline group.
#[derive(Clone, Debug, PartialEq)]
pub struct BarlineStroke {
    /// X-coordinate (centre of stroke).
    pub x: f64,
    /// Thickness of this stroke in font design units.
    pub thickness: f64,
    /// Y-coordinate of top of stroke.
    pub y_top: f64,
    /// Y-coordinate of bottom of stroke.
    pub y_bottom: f64,
}

/// Describes a repeat dot pair (two dots stacked around middle line).
#[derive(Clone, Debug, PartialEq)]
pub struct RepeatDots {
    /// X-coordinate of centre of dots.
    pub x: f64,
    /// Y-coordinate of upper dot (space above middle line).
    pub y_upper: f64,
    /// Y-coordinate of lower dot (space below middle line).
    pub y_lower: f64,
}

/// Full description of a barline's visual components.
#[derive(Clone, Debug, PartialEq)]
pub struct BarlineLayout {
    pub strokes: Vec<BarlineStroke>,
    pub dots: Option<RepeatDots>,
    /// Total advance width of the barline group.
    pub width: f64,
}

/// Compute the layout of a barline at the given x position.
///
/// Returns strokes (thin/thick lines) and optional repeat dots,
/// all in font design units.
pub fn barline_layout(
    style: BarlineStyle,
    x: f64,
    staff: &StaffLayout,
    config: &EngravingConfig,
) -> BarlineLayout {
    let y_top = staff.y_of(crate::layout::TOP_LINE);
    let y_bottom = staff.y_of(crate::layout::BOTTOM_LINE);
    let thin = config.to_font_units(config.thin_barline_thickness);
    let thick = config.to_font_units(config.thick_barline_thickness);
    let sep = config.to_font_units(config.barline_separation);
    let dot_sep = config.to_font_units(config.repeat_barline_dot_separation);

    match style {
        BarlineStyle::Single => {
            let stroke = BarlineStroke {
                x,
                thickness: thin,
                y_top,
                y_bottom,
            };
            BarlineLayout {
                strokes: vec![stroke],
                dots: None,
                width: thin,
            }
        }
        BarlineStyle::Double => {
            let s1 = BarlineStroke {
                x,
                thickness: thin,
                y_top,
                y_bottom,
            };
            let s2 = BarlineStroke {
                x: x + thin / 2.0 + sep + thin / 2.0,
                thickness: thin,
                y_top,
                y_bottom,
            };
            let width = thin + sep + thin;
            BarlineLayout {
                strokes: vec![s1, s2],
                dots: None,
                width,
            }
        }
        BarlineStyle::Final => {
            // Thin line then thick line
            let s1 = BarlineStroke {
                x,
                thickness: thin,
                y_top,
                y_bottom,
            };
            let s2 = BarlineStroke {
                x: x + thin / 2.0 + sep + thick / 2.0,
                thickness: thick,
                y_top,
                y_bottom,
            };
            let width = thin + sep + thick;
            BarlineLayout {
                strokes: vec![s1, s2],
                dots: None,
                width,
            }
        }
        BarlineStyle::EndRepeat => {
            // Dots, then thin, then thick
            let dot_x = x;
            let thin_x = x + dot_sep + thin / 2.0;
            let thick_x = thin_x + thin / 2.0 + sep + thick / 2.0;
            let dots = Some(repeat_dots(dot_x, staff));
            let s1 = BarlineStroke {
                x: thin_x,
                thickness: thin,
                y_top,
                y_bottom,
            };
            let s2 = BarlineStroke {
                x: thick_x,
                thickness: thick,
                y_top,
                y_bottom,
            };
            let width = dot_sep + thin + sep + thick;
            BarlineLayout {
                strokes: vec![s1, s2],
                dots,
                width,
            }
        }
        BarlineStyle::StartRepeat => {
            // Thick, then thin, then dots
            let s1 = BarlineStroke {
                x,
                thickness: thick,
                y_top,
                y_bottom,
            };
            let thin_x = x + thick / 2.0 + sep + thin / 2.0;
            let s2 = BarlineStroke {
                x: thin_x,
                thickness: thin,
                y_top,
                y_bottom,
            };
            let dot_x = thin_x + thin / 2.0 + dot_sep;
            let dots = Some(repeat_dots(dot_x, staff));
            let width = thick + sep + thin + dot_sep;
            BarlineLayout {
                strokes: vec![s1, s2],
                dots,
                width,
            }
        }
    }
}

/// Compute repeat dot positions: two dots in the spaces above and below
/// the middle staff line (positions 3 and 5 in standard 5-line staff).
fn repeat_dots(x: f64, staff: &StaffLayout) -> RepeatDots {
    RepeatDots {
        x,
        y_upper: staff.y_of(5),
        y_lower: staff.y_of(3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (StaffLayout, EngravingConfig) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (staff, config)
    }

    #[test]
    fn single_barline_has_one_stroke() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Single, 100.0, &staff, &config);
        assert_eq!(bl.strokes.len(), 1);
        assert!(bl.dots.is_none());
        assert_eq!(bl.strokes[0].x, 100.0);
    }

    #[test]
    fn single_barline_thickness_matches_config() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Single, 0.0, &staff, &config);
        let expected = config.to_font_units(config.thin_barline_thickness);
        assert!((bl.strokes[0].thickness - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn single_barline_spans_full_staff() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Single, 0.0, &staff, &config);
        assert!((bl.strokes[0].y_top - staff.y_of(8)).abs() < f64::EPSILON);
        assert!((bl.strokes[0].y_bottom - staff.y_of(0)).abs() < f64::EPSILON);
    }

    #[test]
    fn double_barline_has_two_thin_strokes() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Double, 0.0, &staff, &config);
        assert_eq!(bl.strokes.len(), 2);
        assert!(bl.dots.is_none());
        let thin = config.to_font_units(config.thin_barline_thickness);
        assert!((bl.strokes[0].thickness - thin).abs() < f64::EPSILON);
        assert!((bl.strokes[1].thickness - thin).abs() < f64::EPSILON);
    }

    #[test]
    fn double_barline_second_stroke_is_right_of_first() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Double, 0.0, &staff, &config);
        assert!(bl.strokes[1].x > bl.strokes[0].x);
    }

    #[test]
    fn final_barline_thin_then_thick() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Final, 0.0, &staff, &config);
        assert_eq!(bl.strokes.len(), 2);
        let thin = config.to_font_units(config.thin_barline_thickness);
        let thick = config.to_font_units(config.thick_barline_thickness);
        assert!((bl.strokes[0].thickness - thin).abs() < f64::EPSILON);
        assert!((bl.strokes[1].thickness - thick).abs() < f64::EPSILON);
    }

    #[test]
    fn final_barline_width_is_thin_plus_sep_plus_thick() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Final, 0.0, &staff, &config);
        let thin = config.to_font_units(config.thin_barline_thickness);
        let thick = config.to_font_units(config.thick_barline_thickness);
        let sep = config.to_font_units(config.barline_separation);
        assert!((bl.width - (thin + sep + thick)).abs() < f64::EPSILON);
    }

    #[test]
    fn end_repeat_has_dots() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::EndRepeat, 0.0, &staff, &config);
        assert!(bl.dots.is_some());
        assert_eq!(bl.strokes.len(), 2);
    }

    #[test]
    fn start_repeat_has_dots() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::StartRepeat, 0.0, &staff, &config);
        assert!(bl.dots.is_some());
        assert_eq!(bl.strokes.len(), 2);
    }

    #[test]
    fn repeat_dots_straddle_middle_line() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::EndRepeat, 0.0, &staff, &config);
        let dots = bl.dots.unwrap();
        // Middle line is position 4; dots at positions 5 and 3
        let y_middle = staff.y_of(4);
        assert!(dots.y_upper < y_middle, "upper dot should be above middle");
        assert!(dots.y_lower > y_middle, "lower dot should be below middle");
    }

    #[test]
    fn start_and_end_repeat_have_different_stroke_order() {
        let (staff, config) = setup();
        let start = barline_layout(BarlineStyle::StartRepeat, 0.0, &staff, &config);
        let end = barline_layout(BarlineStyle::EndRepeat, 0.0, &staff, &config);
        let thick = config.to_font_units(config.thick_barline_thickness);
        // Start repeat: first stroke is thick
        assert!((start.strokes[0].thickness - thick).abs() < f64::EPSILON);
        // End repeat: second stroke is thick
        assert!((end.strokes[1].thickness - thick).abs() < f64::EPSILON);
    }

    #[test]
    fn barline_at_offset_x_shifts_all_strokes() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Double, 500.0, &staff, &config);
        assert!(bl.strokes[0].x >= 500.0);
        assert!(bl.strokes[1].x > 500.0);
    }

    #[test]
    fn single_barline_width_equals_thin_thickness() {
        let (staff, config) = setup();
        let bl = barline_layout(BarlineStyle::Single, 0.0, &staff, &config);
        let thin = config.to_font_units(config.thin_barline_thickness);
        assert!((bl.width - thin).abs() < f64::EPSILON);
    }
}
