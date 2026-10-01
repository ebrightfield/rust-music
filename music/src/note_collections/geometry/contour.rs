//! Melodic contour analysis and comparison.
//!
//! This module provides tools for analyzing the shape of melodies,
//! comparing contours, and performing contour transformations.

use crate::note::pitch::Pitch;

/// Direction of melodic movement between two adjacent pitches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Ascending,
    Descending,
}

/// Single step in a contour: movement up, down, or repeated note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contour {
    Movement(Movement),
    Repeat,
}

impl Contour {
    /// Returns the inverted contour (up becomes down, down becomes up).
    pub fn invert(&self) -> Self {
        match self {
            Contour::Movement(Movement::Ascending) => Contour::Movement(Movement::Descending),
            Contour::Movement(Movement::Descending) => Contour::Movement(Movement::Ascending),
            Contour::Repeat => Contour::Repeat,
        }
    }

    /// Returns the numeric representation: +1 for up, -1 for down, 0 for repeat.
    pub fn to_numeric(&self) -> i8 {
        match self {
            Contour::Movement(Movement::Ascending) => 1,
            Contour::Movement(Movement::Descending) => -1,
            Contour::Repeat => 0,
        }
    }

    /// Creates a Contour from a numeric value.
    pub fn from_numeric(n: i8) -> Self {
        match n.signum() {
            1 => Contour::Movement(Movement::Ascending),
            -1 => Contour::Movement(Movement::Descending),
            _ => Contour::Repeat,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntervalChange {
    Expanding,
    Contracting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositeMovement {
    Parallel(Movement),
    Similar(Movement, IntervalChange),
    Oblique(Movement, IntervalChange),
    Contrary(IntervalChange),
    Repeat,
}

/// A sequence of contour movements representing the overall shape of a melody.
///
/// The contour sequence captures the ups and downs of a melody without specific
/// pitch or interval information. It's useful for comparing melodic shapes
/// across different transpositions and contexts.
#[derive(Debug, Clone, PartialEq)]
pub struct ContourSequence(Vec<Contour>);

impl ContourSequence {
    /// Create a new contour sequence from a vector of contours.
    pub fn new(contours: Vec<Contour>) -> Self {
        Self(contours)
    }

    /// Create a contour sequence from a series of pitches.
    pub fn from_pitches(pitches: &[Pitch]) -> Self {
        if pitches.len() < 2 {
            return Self(vec![]);
        }

        let contours = pitches
            .windows(2)
            .map(|pair| {
                let diff = pair[1].midi_note as i16 - pair[0].midi_note as i16;
                Contour::from_numeric(diff as i8)
            })
            .collect();

        Self(contours)
    }

    /// Returns the length of the contour sequence.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the contours as a slice.
    pub fn contours(&self) -> &[Contour] {
        &self.0
    }

    /// Returns the retrograde (reverse) of the contour sequence.
    ///
    /// The retrograde is the sequence played backwards.
    pub fn retrograde(&self) -> Self {
        let mut reversed = self.0.clone();
        reversed.reverse();
        Self(reversed)
    }

    /// Returns the inversion of the contour sequence.
    ///
    /// The inversion flips all movements: ascending becomes descending and vice versa.
    pub fn inversion(&self) -> Self {
        let inverted = self.0.iter().map(|c| c.invert()).collect();
        Self(inverted)
    }

    /// Returns the retrograde-inversion of the contour sequence.
    pub fn retrograde_inversion(&self) -> Self {
        self.retrograde().inversion()
    }

    /// Compute the similarity between two contour sequences.
    ///
    /// Returns a value between 0.0 and 1.0, where 1.0 means identical contours.
    /// Uses the proportion of matching contour elements.
    ///
    /// If the sequences are different lengths, only compares up to the shorter length.
    pub fn similarity(&self, other: &ContourSequence) -> f64 {
        if self.is_empty() && other.is_empty() {
            return 1.0;
        }
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }

        let min_len = self.len().min(other.len());
        let max_len = self.len().max(other.len());

        let matches = self.0[..min_len]
            .iter()
            .zip(&other.0[..min_len])
            .filter(|(a, b)| a == b)
            .count();

        // Normalize by the maximum length to penalize length differences
        matches as f64 / max_len as f64
    }

    /// Check if this contour is equivalent to another under transformations.
    ///
    /// Two contours are equivalent if one can be transformed into the other
    /// through retrograde, inversion, or retrograde-inversion.
    pub fn is_equivalent(&self, other: &ContourSequence) -> bool {
        if self == other {
            return true;
        }
        if &self.retrograde() == other {
            return true;
        }
        if &self.inversion() == other {
            return true;
        }
        if &self.retrograde_inversion() == other {
            return true;
        }
        false
    }

    /// Compute weighted similarity considering all transformations.
    ///
    /// Returns the maximum similarity across identity, retrograde, inversion,
    /// and retrograde-inversion transformations.
    pub fn max_similarity(&self, other: &ContourSequence) -> f64 {
        let identity_sim = self.similarity(other);
        let retro_sim = self.retrograde().similarity(other);
        let inv_sim = self.inversion().similarity(other);
        let ri_sim = self.retrograde_inversion().similarity(other);

        identity_sim.max(retro_sim).max(inv_sim).max(ri_sim)
    }

    /// Convert to a numeric vector (+1, -1, 0).
    pub fn to_numeric_vec(&self) -> Vec<i8> {
        self.0.iter().map(|c| c.to_numeric()).collect()
    }
}

impl std::ops::Deref for ContourSequence {
    type Target = Vec<Contour>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    #[test]
    fn test_contour_invert() {
        assert_eq!(
            Contour::Movement(Movement::Ascending).invert(),
            Contour::Movement(Movement::Descending)
        );
        assert_eq!(
            Contour::Movement(Movement::Descending).invert(),
            Contour::Movement(Movement::Ascending)
        );
        assert_eq!(Contour::Repeat.invert(), Contour::Repeat);
    }

    #[test]
    fn test_contour_from_pitches() {
        let pitches = vec![
            Pitch::new(Note::C, 4),
            Pitch::new(Note::E, 4), // up
            Pitch::new(Note::D, 4), // down
            Pitch::new(Note::D, 4), // repeat
            Pitch::new(Note::G, 4), // up
        ];

        let contour = ContourSequence::from_pitches(&pitches);
        assert_eq!(contour.len(), 4);
        assert_eq!(contour[0], Contour::Movement(Movement::Ascending));
        assert_eq!(contour[1], Contour::Movement(Movement::Descending));
        assert_eq!(contour[2], Contour::Repeat);
        assert_eq!(contour[3], Contour::Movement(Movement::Ascending));
    }

    #[test]
    fn test_retrograde() {
        let contour = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
            Contour::Repeat,
        ]);

        let retrograde = contour.retrograde();
        assert_eq!(retrograde[0], Contour::Repeat);
        assert_eq!(retrograde[1], Contour::Movement(Movement::Descending));
        assert_eq!(retrograde[2], Contour::Movement(Movement::Ascending));
    }

    #[test]
    fn test_inversion() {
        let contour = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
            Contour::Repeat,
        ]);

        let inverted = contour.inversion();
        assert_eq!(inverted[0], Contour::Movement(Movement::Descending));
        assert_eq!(inverted[1], Contour::Movement(Movement::Ascending));
        assert_eq!(inverted[2], Contour::Repeat);
    }

    #[test]
    fn test_similarity_identical() {
        let contour1 = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
        ]);
        let contour2 = contour1.clone();

        assert_eq!(contour1.similarity(&contour2), 1.0);
    }

    #[test]
    fn test_similarity_opposite() {
        let contour1 = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Ascending),
        ]);
        let contour2 = ContourSequence::new(vec![
            Contour::Movement(Movement::Descending),
            Contour::Movement(Movement::Descending),
        ]);

        assert_eq!(contour1.similarity(&contour2), 0.0);
    }

    #[test]
    fn test_similarity_partial() {
        let contour1 = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
        ]);
        let contour2 = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
            Contour::Movement(Movement::Descending), // Different
            Contour::Movement(Movement::Ascending),  // Different
        ]);

        assert_eq!(contour1.similarity(&contour2), 0.5); // 2 out of 4 match
    }

    #[test]
    fn test_is_equivalent() {
        let contour1 = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
        ]);

        // Inversion should be equivalent
        let inversion = contour1.inversion();
        assert!(contour1.is_equivalent(&inversion));

        // Retrograde should be equivalent
        let retrograde = contour1.retrograde();
        assert!(contour1.is_equivalent(&retrograde));
    }

    #[test]
    fn test_empty_contours() {
        let empty1 = ContourSequence::new(vec![]);
        let empty2 = ContourSequence::new(vec![]);

        assert_eq!(empty1.similarity(&empty2), 1.0);
        assert!(empty1.is_equivalent(&empty2));
    }

    #[test]
    fn test_max_similarity() {
        let ascending = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Ascending),
        ]);
        let descending = ContourSequence::new(vec![
            Contour::Movement(Movement::Descending),
            Contour::Movement(Movement::Descending),
        ]);

        // Direct similarity is 0, but they are inversions
        assert_eq!(ascending.similarity(&descending), 0.0);
        assert_eq!(ascending.max_similarity(&descending), 1.0);
    }

    #[test]
    fn test_to_numeric_vec() {
        let contour = ContourSequence::new(vec![
            Contour::Movement(Movement::Ascending),
            Contour::Movement(Movement::Descending),
            Contour::Repeat,
        ]);

        assert_eq!(contour.to_numeric_vec(), vec![1, -1, 0]);
    }
}
