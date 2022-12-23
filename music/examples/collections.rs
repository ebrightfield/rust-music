use music::{Pc, Note, Pitch, PcSet, NoteSet, Voicing, pcs, pc, pitch, voicing, StackedIntervals};
use music::geometry::symmetry::transpositional::{Modes, Transpose};
use music::notation::clef::Clef;

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
        let vii_chord = min7b5.transpose(11);
        assert_eq!(vii_chord, vec![pc!(11), pc!(2), pc!(5), pc!(9)]);
    }

    // A [NoteSet] contains collection of [Note] objects,
    // sorted (where C = Pc0 = the "lowest" note), then deduplicated.
    // The sorting can be overridden to treat a different note as Pc0.
    {
        let notes = NoteSet::new(vec![Note::Ees, Note::G, Note::Ees, Note::C], None);
        assert_eq!(notes.to_vec(), vec![Note::C, Note::Ees, Note::G]);

        // It's easy to step through them.
        let mut note = notes[0];
        note = notes.up_n_steps(&note, 1).unwrap();
        assert_eq!(note, Note::Ees);
        note = notes.up_n_steps(&note, 1).unwrap();
        assert_eq!(note, Note::G);
        note = notes.up_n_steps(&note, 1).unwrap();
        assert_eq!(note, Note::C);
        note = notes.up_n_steps(&note, 2).unwrap();
        assert_eq!(note, Note::G);
        note = notes.down_n_steps(&note, 5).unwrap();
        assert_eq!(note, Note::C);
    }

    // Voicings are collections of pitches.
    {
        let v = Voicing::new(vec![
           pitch!(c, 4),
           pitch!(g, 4),
           pitch!(e, 5),
        ]);

        // Sometimes, we might want to think of voicings purely in terms of stacking
        // intervals on top of one another, low to high
        let stacked_intervals = StackedIntervals::new(vec![7,4,5]);
        let v2 = Voicing::from_intervals(
            &pitch!(c,4),
            &stacked_intervals
        ).unwrap();
        assert_eq!(
            v2,
            voicing!(
                pitch!(c,4),
                pitch!(g,4),
                pitch!(b,4),
                pitch!(e,5)
            )
        );

        // We can easily find out a voicing's lowest and highest notes
        // Returns an Option because an empty voicing has no lowest or highest notes.
        let (lowest, highest) = v.span().unwrap();
        assert_eq!(lowest, pitch!(c, 4));
        assert_eq!(highest, pitch!(e, 5));

        // And they're easy to transpose to a new register
        assert_eq!(
            v.move_by_octaves(2).unwrap(),
            voicing!(pitch!(c, 6), pitch!(g, 6), pitch!(e, 7)),
        );

        // Or to normalize their register to be best centered in a clef
        let normalized = voicing!(
            pitch!(c,6), pitch!(g,6), pitch!(e,7)
        ).normalize_register_to_clef(Clef::Treble).unwrap();
        assert_eq!(
            normalized,
            voicing!(pitch!(c, 4), pitch!(g, 4), pitch!(e, 5)),
        );

        // We can also apply voiceleading paths to them.
        // Since chromatic paths do not imply spelling, we can also optionally
        // provide a spelling context as below.
        let f_major = v.apply_paths(
            &vec![0, 2, 1], Some(&vec![Note::F, Note::A, Note::C])
        ).unwrap();
        assert_eq!(
            f_major,
            vec![
                pitch!(c, 4),
                pitch!(a, 4),
                pitch!(f, 5),
            ]
        );
    }

    // These collections all dereference to their inner collection,
    // making iteration easy
    {
        let major_triad = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);
        major_triad.iter().for_each(|_| {});
        let v = Voicing::new(vec![
            pitch!(c, 4),
            pitch!(g, 4),
            pitch!(e, 5),
        ]);
        v.iter().for_each(|_| {});
    }
}