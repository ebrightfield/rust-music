//! Per-measure metadata carried by every [`MeasureContent`]: logical bar
//! numbering, meter and length semantics, and line-break permission.
//!
//! A *logical measure* is the span between two real measure boundaries. It is
//! usually one [`MeasureContent`]; when a system break falls on an inline
//! barline inside it, the later visual pieces are separate contents flagged
//! [`MeasureMeta::continuation`] that copy the first piece's number and
//! lengths.
//!
//! [`MeasureContent`]: crate::layout::system::MeasureContent

use std::cmp::Ordering;
use std::ops::Add;

use music::notation::rhythm::duration::Duration;

use crate::layout::time_signature::TimeSignatureKind;

/// An exact musical length, as a reduced fraction of a whole note.
///
/// Tuplet members make lengths non-integral in any fixed tick unit, so
/// measure lengths are kept exact: `3/4` is three quarter notes, `9/8` nine
/// eighths, `11/16` eleven sixteenths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasureLength {
    numerator: u64,
    denominator: u64,
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl MeasureLength {
    /// The zero length (`0/1`).
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// `numerator/denominator` of a whole note, reduced.
    ///
    /// # Panics
    ///
    /// Panics if `denominator` is zero.
    pub fn new(numerator: u64, denominator: u64) -> Self {
        assert!(
            denominator != 0,
            "measure length denominator must be non-zero"
        );
        let divisor = gcd(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }

    /// Reduced numerator.
    pub fn numerator(self) -> u64 {
        self.numerator
    }

    /// Reduced denominator.
    pub fn denominator(self) -> u64 {
        self.denominator
    }

    /// The length of a written duration performed at `tuplet_number :
    /// in_time_of` (`1:1` outside tuplets; a zero ratio term leaves the
    /// written length unscaled, as measure layout does).
    pub fn of_duration_in_ratio(duration: &Duration, tuplet_number: u32, in_time_of: u32) -> Self {
        let written = Self::from(*duration);
        if tuplet_number == 0 || in_time_of == 0 {
            written
        } else {
            Self::new(
                written.numerator * u64::from(in_time_of),
                written.denominator * u64::from(tuplet_number),
            )
        }
    }

    /// Length as a floating-point number of whole notes.
    pub fn as_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl Default for MeasureLength {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Duration> for MeasureLength {
    /// A written duration's length (128th-note ticks over 128).
    fn from(duration: Duration) -> Self {
        Self::new(duration.ticks() as u64, 128)
    }
}

impl Add for MeasureLength {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Ord for MeasureLength {
    fn cmp(&self, other: &Self) -> Ordering {
        (u128::from(self.numerator) * u128::from(other.denominator))
            .cmp(&(u128::from(other.numerator) * u128::from(self.denominator)))
    }
}

impl PartialOrd for MeasureLength {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Line-break permission at the end of a [`MeasureContent`].
///
/// [`MeasureContent`]: crate::layout::system::MeasureContent
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineBreak {
    /// The line breaker decides.
    #[default]
    Auto,
    /// A system break must follow this content.
    Force,
    /// No system break may follow this content.
    Forbid,
}

/// Metadata of one [`MeasureContent`].
///
/// `Default` describes an unnumbered-by-intent, unmetered, empty measure
/// (number 0, no meter, no nominal length); the score builders fill every
/// field.
///
/// [`MeasureContent`]: crate::layout::system::MeasureContent
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeasureMeta {
    /// Logical bar number. Measures count up from the score's start number
    /// (1 by default); an anacrusis does not advance the count and carries
    /// the number before it, so a score-initial pickup is bar 0.
    pub number: i32,
    /// Meter in force for this measure, printed or not (`None` before any
    /// meter is declared).
    pub meter: Option<TimeSignatureKind>,
    /// Whether the declaration of [`Self::meter`] was printed. A hidden meter
    /// keeps its metric semantics (nominal length) but draws nothing.
    pub meter_visible: bool,
    /// Expected length: the meter's, an explicit measure-length override, or
    /// an anacrusis's partial length; multiplied by the number of bars in a
    /// compressed multi-measure rest. `None` for unmetered and cadenza
    /// measures, which carry no length expectation.
    pub nominal_length: Option<MeasureLength>,
    /// Performed length of the longest voice (tuplets at their performed
    /// ratio; multi-measure rests occupy the represented number of bars).
    pub actual_length: MeasureLength,
    /// Pickup measure declared with a partial length.
    pub anacrusis: bool,
    /// A later visual piece of a logical measure that was split at an inline
    /// barline. It shares the logical measure's number and lengths.
    pub continuation: bool,
    /// Exact onset of this visual piece in its logical measure. Non-split
    /// measures begin at zero; pieces after a forced mid-measure break retain
    /// the break's original performed onset on every stave.
    pub visual_start: MeasureLength,
    /// At a split, each voice's exact performed cursor at its first original
    /// event in this piece (primary first). This preserves a note that spans
    /// the break: its next event begins after `visual_start`, not at the break.
    /// Unsplit measures leave this empty.
    pub visual_voice_onsets: Vec<MeasureLength>,
    /// Line-break permission after this content.
    pub line_break: LineBreak,
}

impl MeasureMeta {
    /// Whether the measure is written shorter than its nominal length (an
    /// incomplete bar). Unmetered measures are never incomplete.
    pub fn is_incomplete(&self) -> bool {
        self.nominal_length
            .is_some_and(|nominal| self.actual_length < nominal)
    }
}
