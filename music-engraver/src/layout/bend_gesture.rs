//! Geometry for timed semantic bend-gesture segments.

/// The rendered staff participating in a coordinated bend gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BendView {
    Standard,
    Tab,
}

impl BendView {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Tab => "tab",
        }
    }
}

/// The musical phase represented by one visual segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BendSegmentPhase {
    Rise,
    Hold,
    Release,
}

impl BendSegmentPhase {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Rise => "rise",
            Self::Hold => "hold",
            Self::Release => "release",
        }
    }
}

/// A segment's role when a phase crosses system boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BendFragment {
    Complete,
    Start,
    Middle,
    End,
}

impl BendFragment {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Start => "start",
            Self::Middle => "middle",
            Self::End => "end",
        }
    }
}

/// Fully resolved geometry for one phase fragment on one staff.
#[derive(Clone, Debug)]
pub(crate) struct BendSegmentLayout {
    pub(crate) gesture_index: usize,
    pub(crate) string: u8,
    pub(crate) view: BendView,
    pub(crate) phase: BendSegmentPhase,
    pub(crate) fragment: BendFragment,
    pub(crate) x_start: f64,
    pub(crate) y_start: f64,
    pub(crate) x_end: f64,
    pub(crate) y_end: f64,
    pub(crate) stroke_width: f64,
    pub(crate) staff_space: f64,
    pub(crate) arrow_at_end: bool,
    pub(crate) amount_label: Option<String>,
    pub(crate) target_label: Option<String>,
}

impl BendSegmentLayout {
    pub(crate) fn is_level(&self) -> bool {
        (self.y_start - self.y_end).abs() < f64::EPSILON
    }
}
