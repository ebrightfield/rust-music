use std::{fmt, str::FromStr};

use num_rational::Ratio;
use thiserror::Error;

/// Exact musical time measured in quarter notes from the beginning of a piece.
///
/// This is deliberately separate from `music::notation::rhythm::Duration`:
/// corpus positions may contain tuplets or other rational values that are not
/// representable as a conventional notated duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ScoreTime(Ratio<i64>);

impl ScoreTime {
    pub const ZERO: Self = Self(Ratio::new_raw(0, 1));

    pub fn new(numerator: i64, denominator: i64) -> Result<Self, TimeError> {
        if denominator == 0 {
            return Err(TimeError::ZeroDenominator);
        }
        Ok(Self(Ratio::new(numerator, denominator)))
    }

    pub fn from_integer(quarter_notes: i64) -> Self {
        Self(Ratio::from_integer(quarter_notes))
    }

    pub fn numerator(self) -> i64 {
        *self.0.numer()
    }

    pub fn denominator(self) -> i64 {
        *self.0.denom()
    }

    pub fn checked_span_to(self, end: Self) -> Result<ExactScoreSpan, TimeError> {
        ExactScoreSpan::new(self, end)
    }
}

impl FromStr for ScoreTime {
    type Err = TimeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err(TimeError::Empty);
        }
        if let Some((numerator, denominator)) = value.split_once('/') {
            return Self::new(
                numerator
                    .trim()
                    .parse()
                    .map_err(|_| TimeError::Invalid(value.into()))?,
                denominator
                    .trim()
                    .parse()
                    .map_err(|_| TimeError::Invalid(value.into()))?,
            );
        }
        if let Some((whole, fraction)) = value.split_once('.') {
            let negative = whole.starts_with('-');
            let whole: i64 = whole
                .parse()
                .map_err(|_| TimeError::Invalid(value.into()))?;
            let scale = 10_i64
                .checked_pow(fraction.len() as u32)
                .ok_or_else(|| TimeError::Invalid(value.into()))?;
            let fraction: i64 = fraction
                .parse()
                .map_err(|_| TimeError::Invalid(value.into()))?;
            let numerator = whole
                .checked_mul(scale)
                .and_then(|base| base.checked_add(if negative { -fraction } else { fraction }))
                .ok_or_else(|| TimeError::Invalid(value.into()))?;
            return Self::new(numerator, scale);
        }
        Ok(Self::from_integer(
            value
                .parse()
                .map_err(|_| TimeError::Invalid(value.into()))?,
        ))
    }
}

impl fmt::Display for ScoreTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator() == 1 {
            write!(formatter, "{}", self.numerator())
        } else {
            write!(formatter, "{}/{}", self.numerator(), self.denominator())
        }
    }
}

impl std::ops::Add for ScoreTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for ScoreTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul<i64> for ScoreTime {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TimeError {
    #[error("time value is empty")]
    Empty,
    #[error("time denominator cannot be zero")]
    ZeroDenominator,
    #[error("invalid time value {0:?}")]
    Invalid(String),
    #[error("score span ends before it begins: {start}..{end}")]
    ReversedSpan { start: ScoreTime, end: ScoreTime },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExactScoreSpan {
    pub start: ScoreTime,
    pub end: ScoreTime,
}

impl ExactScoreSpan {
    pub fn new(start: ScoreTime, end: ScoreTime) -> Result<Self, TimeError> {
        if end < start {
            return Err(TimeError::ReversedSpan { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn duration(self) -> ScoreTime {
        self.end - self.start
    }

    pub fn contains(self, time: ScoreTime) -> bool {
        self.start <= time && time < self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceSpan {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    pub corpus: String,
    pub piece: String,
    pub facet: &'static str,
    pub rows: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteEvent {
    pub source: SourceRef,
    pub score_span: ExactScoreSpan,
    pub midi_key: u8,
    pub tonal_pitch_class: Option<i32>,
    pub staff: Option<u16>,
    pub voice: Option<u8>,
    /// True for grace-note rows, which DCML represents with zero duration.
    pub is_grace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    pub source: SourceRef,
    pub count: i64,
    pub number: String,
    pub score_span: ExactScoreSpan,
    pub time_signature: Option<TimeSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub numerator: u16,
    pub denominator: u16,
}

impl FromStr for TimeSignature {
    type Err = TimeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (numerator, denominator) = value
            .trim()
            .split_once('/')
            .ok_or_else(|| TimeError::Invalid(value.into()))?;
        let numerator = numerator
            .parse()
            .map_err(|_| TimeError::Invalid(value.into()))?;
        let denominator = denominator
            .parse()
            .map_err(|_| TimeError::Invalid(value.into()))?;
        if numerator == 0 || denominator == 0 {
            return Err(TimeError::Invalid(value.into()));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarmonyAnnotation {
    pub source: SourceRef,
    pub score_span: ExactScoreSpan,
    pub label: String,
    pub global_key: Option<String>,
    pub local_key: Option<String>,
    pub numeral: Option<String>,
    pub chord_type: Option<String>,
    pub figured_bass: Option<String>,
    pub changes: Option<String>,
    pub root_fifths: Option<i32>,
    pub bass_fifths: Option<i32>,
    pub cadence: Option<String>,
    pub phrase_end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusPiece {
    pub corpus: String,
    pub piece: String,
    pub notes: Vec<NoteEvent>,
    pub measures: Vec<Measure>,
    pub harmonies: Vec<HarmonyAnnotation>,
}

impl CorpusPiece {
    pub fn end(&self) -> ScoreTime {
        self.notes
            .iter()
            .map(|note| note.score_span.end)
            .chain(self.measures.iter().map(|measure| measure.score_span.end))
            .max()
            .unwrap_or(ScoreTime::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_reduces_exact_times() {
        assert_eq!(
            "6/8".parse::<ScoreTime>().unwrap(),
            ScoreTime::new(3, 4).unwrap()
        );
        assert_eq!(
            "1.25".parse::<ScoreTime>().unwrap(),
            ScoreTime::new(5, 4).unwrap()
        );
        assert_eq!(
            "-0.5".parse::<ScoreTime>().unwrap(),
            ScoreTime::new(-1, 2).unwrap()
        );
    }

    #[test]
    fn spans_are_half_open() {
        let span = ExactScoreSpan::new(ScoreTime::ZERO, ScoreTime::from_integer(1)).unwrap();
        assert!(span.contains(ScoreTime::ZERO));
        assert!(!span.contains(ScoreTime::from_integer(1)));
    }
}
