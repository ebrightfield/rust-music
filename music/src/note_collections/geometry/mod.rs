pub mod contour;
pub mod sets;
pub mod symmetry;

use crate::note::pitch_class::Pc;
use crate::note_collections::interval_class::IntervalClass;
use crate::note_collections::pc_set::PcShape;

/// A matrix showing the interval between every pair of pitch classes in a set.
///
/// The matrix is square with dimensions equal to the cardinality of the PcShape.
/// Entry (i, j) contains the interval from pc[i] to pc[j], calculated as
/// (pc[j] - pc[i]) mod 12.
///
/// This is useful for:
/// - Computing interval vectors
/// - Finding pitch class pairs with specific intervals
/// - Set-theoretical analysis of pitch collections
// REQ-O15
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalMatrix {
    /// The pitch class shape this matrix represents
    pcs: PcShape,
    /// The interval matrix, where matrix[i][j] = interval from pc[i] to pc[j]
    matrix: Vec<Vec<IntervalClass>>,
}

impl IntervalMatrix {
    /// Creates a new IntervalMatrix from a PcShape.
    ///
    /// The matrix is computed by calculating the interval (mod 12) between
    /// every ordered pair of pitch classes in the shape.
    // REQ-O15
    pub fn new(pc_shape: &PcShape) -> Self {
        let pcs_vec: &[Pc] = pc_shape; // Uses Deref
        let n = pcs_vec.len();
        let mut matrix = Vec::with_capacity(n);

        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            let pc_i: i32 = (&pcs_vec[i]).into();
            for pc in pcs_vec.iter() {
                let pc_j: i32 = pc.into();
                let interval = (pc_j - pc_i).rem_euclid(12);
                row.push(IntervalClass::from(&interval));
            }
            matrix.push(row);
        }

        Self {
            pcs: pc_shape.clone(),
            matrix,
        }
    }

    /// Returns the interval at position (row, col) in the matrix.
    /// Returns None if the indices are out of bounds.
    pub fn get(&self, row: usize, col: usize) -> Option<&IntervalClass> {
        self.matrix.get(row).and_then(|r| r.get(col))
    }

    /// Returns the pitch class shape this matrix represents.
    pub fn pcs(&self) -> &PcShape {
        &self.pcs
    }

    /// Returns the dimension of the matrix (equal to the cardinality of the PcShape).
    pub fn dimension(&self) -> usize {
        self.matrix.len()
    }

    /// Computes the interval vector for this pitch class set.
    ///
    /// The interval vector counts how many times each interval class appears
    /// in the set. Index 0 counts IC0 (unisons), index 1 counts IC1 (minor seconds), etc.
    ///
    /// Note: This counts ordered intervals (0-11), not the reduced interval classes
    /// (0-6) used in some set theory contexts.
    ///
    /// The diagonal (IC0) is excluded since it represents each note with itself.
    pub fn interval_vector(&self) -> [usize; 12] {
        let mut vector = [0usize; 12];
        let n = self.matrix.len();

        for i in 0..n {
            for j in 0..n {
                if i != j {
                    // Exclude diagonal (interval with self)
                    let ic: u8 = (&self.matrix[i][j]).into();
                    vector[ic as usize] += 1;
                }
            }
        }

        vector
    }

    /// Computes the reduced interval vector (IC 1-6 only).
    ///
    /// This is the traditional interval vector used in set theory, where
    /// intervals are reduced to their "smaller" form (IC7 becomes IC5, IC8 becomes IC4, etc.)
    /// IC0 is excluded (self-intervals), and IC6 is the tritone (its own inverse).
    ///
    /// Returns an array of 6 elements: [ic1_count, ic2_count, ic3_count, ic4_count, ic5_count, ic6_count]
    pub fn reduced_interval_vector(&self) -> [usize; 6] {
        let full_vector = self.interval_vector();
        // The full vector counts ordered pairs (both directions), so each
        // unordered pair is counted twice. Divide by 2 to get the standard
        // music-theory interval-class vector.
        [
            (full_vector[1] + full_vector[11]) / 2, // IC1 + IC11 -> ic1
            (full_vector[2] + full_vector[10]) / 2, // IC2 + IC10 -> ic2
            (full_vector[3] + full_vector[9]) / 2,  // IC3 + IC9 -> ic3
            (full_vector[4] + full_vector[8]) / 2,  // IC4 + IC8 -> ic4
            (full_vector[5] + full_vector[7]) / 2,  // IC5 + IC7 -> ic5
            full_vector[6] / 2,                     // IC6 (tritone, its own inverse)
        ]
    }

    /// Finds all pairs of pitch classes that have the given interval.
    /// Returns a vector of (from_pc, to_pc) tuples.
    pub fn find_interval(&self, ic: IntervalClass) -> Vec<(Pc, Pc)> {
        let pcs_vec: &[Pc] = &self.pcs; // Uses Deref
        let n = pcs_vec.len();
        let mut pairs = Vec::new();

        for i in 0..n {
            for j in 0..n {
                if self.matrix[i][j] == ic {
                    pairs.push((pcs_vec[i], pcs_vec[j]));
                }
            }
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn test_interval_matrix_creation() {
        // C major triad: C, E, G (Pc0, Pc4, Pc7)
        let pc_set = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let matrix = IntervalMatrix::new(&pc_set);

        assert_eq!(matrix.dimension(), 3);

        // Check diagonal (self-intervals are 0)
        assert_eq!(matrix.get(0, 0), Some(&IntervalClass::Ic0));
        assert_eq!(matrix.get(1, 1), Some(&IntervalClass::Ic0));
        assert_eq!(matrix.get(2, 2), Some(&IntervalClass::Ic0));

        // C to E (0 to 4) = 4 semitones (major third)
        assert_eq!(matrix.get(0, 1), Some(&IntervalClass::Ic4));
        // C to G (0 to 7) = 7 semitones (perfect fifth)
        assert_eq!(matrix.get(0, 2), Some(&IntervalClass::Ic7));
        // E to G (4 to 7) = 3 semitones (minor third)
        assert_eq!(matrix.get(1, 2), Some(&IntervalClass::Ic3));
        // E to C (4 to 0) = 8 semitones (minor sixth, going up)
        assert_eq!(matrix.get(1, 0), Some(&IntervalClass::Ic8));
    }

    #[test]
    fn test_interval_vector() {
        // C major triad: C, E, G
        let pc_set = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let matrix = IntervalMatrix::new(&pc_set);
        let vector = matrix.interval_vector();

        // We have 6 ordered pairs (excluding diagonal):
        // C->E: IC4, E->C: IC8
        // C->G: IC7, G->C: IC5
        // E->G: IC3, G->E: IC9
        assert_eq!(vector[3], 1); // One IC3
        assert_eq!(vector[4], 1); // One IC4
        assert_eq!(vector[5], 1); // One IC5
        assert_eq!(vector[7], 1); // One IC7
        assert_eq!(vector[8], 1); // One IC8
        assert_eq!(vector[9], 1); // One IC9
    }

    #[test]
    fn test_reduced_interval_vector() {
        // C major triad: C, E, G
        let pc_set = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let matrix = IntervalMatrix::new(&pc_set);
        let reduced = matrix.reduced_interval_vector();

        // In traditional set theory notation: <001110>
        // ic1: 0, ic2: 0, ic3: 1, ic4: 1, ic5: 1, ic6: 0
        assert_eq!(reduced, [0, 0, 1, 1, 1, 0]);
    }

    #[test]
    fn test_find_interval() {
        // C major triad: C, E, G
        let pc_set = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let matrix = IntervalMatrix::new(&pc_set);

        // Find all major thirds (IC4)
        let major_thirds = matrix.find_interval(IntervalClass::Ic4);
        assert_eq!(major_thirds.len(), 1);
        assert_eq!(major_thirds[0], (Pc0, Pc4)); // C to E

        // Find all perfect fifths (IC7)
        let fifths = matrix.find_interval(IntervalClass::Ic7);
        assert_eq!(fifths.len(), 1);
        assert_eq!(fifths[0], (Pc0, Pc7)); // C to G

        // Find all unisons (IC0) - includes diagonal
        let unisons = matrix.find_interval(IntervalClass::Ic0);
        assert_eq!(unisons.len(), 3); // C-C, E-E, G-G
    }

    #[test]
    fn test_diminished_seventh_chord() {
        // Diminished seventh: C, Eb, Gb, A (Pc0, Pc3, Pc6, Pc9)
        // This is a symmetrical structure with interval vector <004002>
        let pc_set = PcShape::new(vec![Pc0, Pc3, Pc6, Pc9]);
        let matrix = IntervalMatrix::new(&pc_set);
        let reduced = matrix.reduced_interval_vector();

        // Diminished seventh: ic3 appears 4 times (each note is a minor third from 2 others)
        // ic6 appears 2 times (two tritone pairs: C-Gb, Eb-A)
        assert_eq!(reduced[2], 4); // ic3: 4 unordered pairs
        assert_eq!(reduced[5], 2); // ic6: 2 unordered tritone pairs
    }
}
