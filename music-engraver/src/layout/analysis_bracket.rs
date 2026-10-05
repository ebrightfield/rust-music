//! Horizontal analytical brackets spanning standard-notation events.

use crate::layout::placement::Placement;

/// Stroke of the horizontal part of an analysis bracket.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnalysisBracketStyle {
    #[default]
    Solid,
    Dashed,
}

/// Appearance of a bracket opened at one note, chord or rest and closed at another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisBracketSpec {
    pub style: AnalysisBracketStyle,
    pub placement: Placement,
    /// Whether a short perpendicular hook is drawn at the first event.
    pub start_hook: bool,
    /// Whether a short perpendicular hook is drawn at the last event.
    pub end_hook: bool,
    /// Optional text printed just outside the bracket at its start.
    pub label: Option<String>,
}

impl AnalysisBracketSpec {
    pub fn new(style: AnalysisBracketStyle, placement: Placement) -> Self {
        Self {
            style,
            placement,
            start_hook: true,
            end_hook: true,
            label: None,
        }
    }

    pub fn hooks(mut self, start: bool, end: bool) -> Self {
        self.start_hook = start;
        self.end_hook = end;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Positioned bracket geometry; coordinates use the same space as its staff.
#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisBracketLayout {
    pub x_start: f64,
    pub x_end: f64,
    pub y: f64,
    pub stroke_width: f64,
    pub hook_length: f64,
    pub start_hook: bool,
    pub end_hook: bool,
    pub style: AnalysisBracketStyle,
    pub placement: Placement,
    pub label: Option<String>,
}

/// Position a segment outside the staff and its participating notes/stems.
/// `top` and `bottom` are vertical extents of the events this segment crosses.
#[allow(clippy::too_many_arguments)]
pub fn layout_analysis_bracket(
    spec: &AnalysisBracketSpec,
    x_start: f64,
    x_end: f64,
    staff_top: f64,
    staff_bottom: f64,
    top: f64,
    bottom: f64,
    staff_space: f64,
    start_hook: bool,
    end_hook: bool,
    show_label: bool,
) -> Option<AnalysisBracketLayout> {
    if x_end <= x_start {
        return None;
    }
    let y = match spec.placement {
        Placement::Above => staff_top.min(top) - staff_space * 0.95,
        Placement::Below => staff_bottom.max(bottom) + staff_space * 0.95,
    };
    Some(AnalysisBracketLayout {
        x_start,
        x_end,
        y,
        stroke_width: staff_space * 0.08,
        hook_length: staff_space * 0.45,
        start_hook: start_hook && spec.start_hook,
        end_hook: end_hook && spec.end_hook,
        style: spec.style,
        placement: spec.placement,
        label: if show_label { spec.label.clone() } else { None },
    })
}
