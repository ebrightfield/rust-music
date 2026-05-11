use itertools::Itertools;
use crate::error::MusicSemanticsError;
use crate::note::Pc;
use crate::note_collections::pc_set::PcShape;
use crate::note_collections::geometry::symmetry::transpositional::Modes;

pub fn get_subchords(pcs: &PcShape, size: u8) -> Result<Vec<Vec<Pc>>, MusicSemanticsError> {
    if size < 3 {
        return Err(MusicSemanticsError::SizeTooSmallForChords(size as usize));
    }
    if size as usize > pcs.len() - 1 {
        return Err(MusicSemanticsError::SizeTooLargeForSubchords(size, pcs.clone()));
    }
    Ok((**pcs).clone()
        .into_iter()
        .combinations(size as usize)
        .collect()
    )
}

impl PcShape {
    /// Check if `other` is a subchord of any mode of self.
    /// This is useful for checking if a chord is "contained within" a scale
    /// regardless of which mode we're considering.
    ///
    /// # Example
    /// ```
    /// use music::note::pitch_class::Pc::*;
    /// use music::note_collections::pc_set::PcShape;
    /// use music::note_collections::geometry::sets::*;
    ///
    /// // C major scale
    /// let c_major = PcShape::new(vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11]);
    /// // A minor triad (mode 6 of C major)
    /// let a_minor = PcShape::new(vec![Pc0, Pc3, Pc7]);  // Zeroed from A=9
    ///
    /// assert!(c_major.contains_subchord(&a_minor));
    /// ```
    pub fn contains_subchord(&self, other: &PcShape) -> bool {
        // Get all modes of self
        let modes = self.modes();

        // Check if other (as a set of pitch classes) is contained in any mode
        for mode in &modes {
            // Check if all PCs from other are present in this mode
            let all_present = other.iter().all(|pc| mode.contains(pc));
            if all_present {
                return true;
            }
        }

        false
    }

    /// Find which modes of self contain the given subchord.
    /// Returns a vector of (mode_index, mode) pairs.
    ///
    /// # Example
    /// ```
    /// use music::note::pitch_class::Pc::*;
    /// use music::note_collections::pc_set::PcShape;
    /// use music::note_collections::geometry::sets::*;
    ///
    /// let major_scale = PcShape::new(vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11]);
    /// let major_triad = PcShape::new(vec![Pc0, Pc4, Pc7]);
    ///
    /// let modes = major_scale.modes_containing(&major_triad);
    /// // Major triads appear in modes 0 (Ionian), 3 (Lydian), and 4 (Mixolydian)
    /// assert!(modes.iter().any(|(idx, _)| *idx == 0));  // Ionian
    /// ```
    pub fn modes_containing(&self, subchord: &PcShape) -> Vec<(usize, PcShape)> {
        self.modes()
            .into_iter()
            .enumerate()
            .filter(|(_, mode)| {
                subchord.iter().all(|pc| mode.contains(pc))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn test_contains_subchord() {
        // C major scale: C D E F G A B = 0 2 4 5 7 9 11
        let c_major = PcShape::new(vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11]);

        // C major triad (zeroed): 0 4 7
        let c_maj_triad = PcShape::new(vec![Pc0, Pc4, Pc7]);
        assert!(c_major.contains_subchord(&c_maj_triad));

        // D minor triad in C major (D F A = 2 5 9, zeroed: 0 3 7)
        let d_min_triad = PcShape::new(vec![Pc0, Pc3, Pc7]);
        assert!(c_major.contains_subchord(&d_min_triad));

        // B diminished in C major (B D F = 11 2 5, zeroed: 0 3 6)
        let b_dim = PcShape::new(vec![Pc0, Pc3, Pc6]);
        assert!(c_major.contains_subchord(&b_dim));

        // Augmented triad (0 4 8) - not in major scale
        let aug_triad = PcShape::new(vec![Pc0, Pc4, Pc8]);
        assert!(!c_major.contains_subchord(&aug_triad));
    }

    #[test]
    fn test_modes_containing() {
        // Major scale
        let major_scale = PcShape::new(vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11]);

        // Major triad (0 4 7)
        let major_triad = PcShape::new(vec![Pc0, Pc4, Pc7]);

        let modes = major_scale.modes_containing(&major_triad);

        // Major triads should appear in exactly 3 modes of the major scale:
        // - Mode 0 (Ionian/Major): I chord
        // - Mode 3 (Lydian): I chord
        // - Mode 4 (Mixolydian): I chord
        assert_eq!(modes.len(), 3);

        // Verify mode indices
        let indices: Vec<usize> = modes.iter().map(|(idx, _)| *idx).collect();
        assert!(indices.contains(&0));  // Ionian
        assert!(indices.contains(&3));  // Lydian
        assert!(indices.contains(&4));  // Mixolydian
    }

    #[test]
    fn test_modes_containing_minor_triad() {
        let major_scale = PcShape::new(vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11]);

        // Minor triad (0 3 7)
        let minor_triad = PcShape::new(vec![Pc0, Pc3, Pc7]);

        let modes = major_scale.modes_containing(&minor_triad);

        // Minor triads appear in:
        // - Mode 1 (Dorian): i chord
        // - Mode 2 (Phrygian): i chord
        // - Mode 5 (Aeolian/Natural Minor): i chord
        assert_eq!(modes.len(), 3);
    }
}
