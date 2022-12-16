

use music::{Pc, Note, Pitch, PcSet, NoteSet, Voicing, OctavePartition, pcs, pc, note};
use music::geometry::symmetry::transpositional::Modes;

fn main() {

    // A collection of Pc make up a PcSet.
    // They are deduplicated, ordered, and zeroed (in that order)
    {
        let major_triad = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);

        // You can get the modes / inversions
        let modes = major_triad.modes();
        assert!(modes.contains(&major_triad));
        assert!(modes.contains(&pcs!(0, 3, 8)));
        assert!(modes.contains(&pcs!(0, 5, 9)));
        assert_eq!(major_triad.rotate(1), pcs!(0, 3, 8));
        assert_eq!(major_triad.rotate(2), pcs!(0, 5, 9));

        // You can test whether something is a transposed version of something else
        assert!(major_triad.is_transposed_version_of(&vec![pc!(2), pc!(6), pc!(9)]));

        // Spelling is possible via heuristics and a given starting note
        let spelled = major_triad.try_spell(&Note::C).unwrap();
        assert_eq!(spelled, vec![Note::C, Note::E, Note::G]);
        let major7 = PcSet::new(vec![pc!(0), pc!(4), pc!(7), pc!(11)]);
        let spelled = major7.try_spell(&Note::Bes).unwrap();
        assert_eq!(spelled, vec![Note::Bes, Note::D, Note::F, Note::A]);
        let spelled = major7.try_spell(&Note::A).unwrap();
        assert_eq!(spelled, vec![Note::A, Note::Cis, Note::E, Note::Gis]);

        // Spelling should be able to handle strange things pretty well,
        // and even take into account the other notes as context
        let min7b5 = pcs!(0, 3, 6, 10);
        let spelled = min7b5.try_spell(&Note::A).unwrap();
        assert_eq!(spelled, vec![Note::A, Note::C, Note::Ees, Note::G]);
        let dom7sharp11 = pcs!(0, 4, 6, 7, 10);
        let spelled = dom7sharp11.try_spell(&Note::A).unwrap();
        assert_eq!(spelled, vec![Note::A, Note::Cis, Note::Dis, Note::E, Note::G]);

        // They are transposable into a nonzeroed vec of Pc.
        let vii_chord = min7b5.transpose_nonzeroed(11);
        assert_eq!(vii_chord, vec![pc!(11), pc!(2), pc!(5), pc!(9)]);
    }

    // These collections all dereference to their inner collection,
    // making iteration easy
    {
        let major_triad = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);
        major_triad.iter().for_each(|pc| {});
    }
}