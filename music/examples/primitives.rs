///
/// There are three single-pitch primitives: [Pc], [Note], and [Pitch].
///
/// - [Pc] -- Represents spelling and octave agnostic notes. You can only
/// do chromatic interval math with these, and only in a mod-12 space where "down 7"
/// the same as "up 5".
/// - [Note] -- Represents a note with spelling information but no octave. You can
/// do all of the same chromatic interval math that you can with [Pc], but also
/// can make diatonic calculations, tonal implications, and start to spell chords and scales.
/// - [Pitch] -- Represents a note with both spelling and octave information.
/// This value only needs a duration to be notated, and is the most "specific" we need to be
/// with for twelve-tone equal temperament systems or MIDI.
use music::{Pc, pc, Note, Pitch, pitch, Spelling};
use music::geometry::symmetry::transpositional::{Transpose, TryTranspose};
use music::note::spelling::{Accidental, Letter};

fn main() {

    // The Pc is a mod-12 enum that can be converted to and from numbers,
    // and has some built-in methods for distance calculations and manipulation.
    {
        let p = Pc::Pc0;
        assert_eq!(p.next(), Pc::Pc1);
        assert_eq!(p.previous(), Pc::Pc11);

        // There are macros to shorthand all of these primitives
        assert_eq!(pc!(11).next(), pc!(0));

        // You can do distance calculations
        assert_eq!(pc!(3).distance_up_to(&pc!(9)), 6);
        assert_eq!(pc!(0).distance_down_to(&pc!(9)), 3);
        // And create new instances from distances
        assert_eq!(pc!(7).transpose(7), pc!(2));
        assert_eq!(pc!(7).transpose(-7), pc!(0));
    }

    // The Note is an enum of all spellings deemed acceptable. See documentation for more details.
    {
        let n = Note::C;
        // Diatonic distance calculations
        assert_eq!(Note::C.diatonic_distance_up(&Note::E), 2);

        // Enharmonic calculations
        assert_eq!(Note::Cis.enharmonic(), Note::Des);
        assert_eq!(Note::C.enharmonic_flip_bcef(), Note::Bis);
        assert_eq!(Note::Fisis.enharmonic(), Note::G);
        assert!(n.is_enharmonic(&Note::Bis));

        // You can get a Pc from a note (Pc0 = C)
        assert_eq!(Pc::Pc0, Pc::from(Note::C));
        assert_eq!(Pc::Pc7, Pc::from(Note::G));

        // But more than one note maps to any given Pc
        // (more on how to resolve spellings later)
        assert_eq!(Pc::Pc3.notes(), vec![Note::Dis, Note::Ees]);
        assert_eq!(Pc::from(Note::Cis), Pc::from(Note::Des));

        // We have ways of controlling spellings at a higher level,
        // but transposing a note will return simply a sharp or a natural.
        assert_eq!(n.transpose(6), Note::Fis);
        assert_eq!(Note::A.transpose(5), Note::D);

        // You can introspect on note spellings if need be:
        let spelling = Spelling::from(n);
        assert_eq!(spelling.letter, Letter::C);
        assert_eq!(spelling.acc, Accidental::Natural);
    }

    // Pitches are just notes with octave information
    // They can't be higher than what the MIDI protocol allows (128)
    // Middle C = C4 = midi-note 60
    {
        let p = Pitch::new(Note::C, 4).unwrap();
        assert_eq!(
            p,
            Pitch::from_midi(60).unwrap(),
        );
        assert_eq!(p.midi_note, 60);

        // There are convenience methods for finding the pitch for the next note up/down
        assert_eq!(p.up_to_note(&Note::B).unwrap(), pitch!(b, 4));
        assert_eq!(p.down_to_note(&Note::B).unwrap(), pitch!(b, 3));

        // There is still enharmonic calculation, but it cares about octaves.
        assert!(pitch!(bes, 5).is_same_frequency(&pitch!(ais, 5)));
        assert!(!pitch!(bes, 5).is_same_frequency(&pitch!(ais, 4)));
        // For the same enharmonic calculation as the [Note] struct, the pitch stores a Note.
        assert!(pitch!(bes, 5).note.is_enharmonic(&Note::Ais));

        // Diatonic distance for pitches accounts for octaves
        assert_eq!(p.diatonic_distance(&pitch!(g, 5)), 11);
        assert_eq!(p.diatonic_distance(&pitch!(g, 2)), -10);

        // There's a macro to condense that constructor,
        // but beware, it's going to unwrap it!
        assert_eq!(
            pitch!(fis, 4),
            Pitch::new(Note::Fis, 4).unwrap(),
        );

        // Since pitches are bounded, we have to try_transpose instead.
        let transposed = pitch!(fis, 4).try_transpose(13).unwrap();
        assert_eq!(
            transposed,
            Pitch::new(Note::G, 5).unwrap(),
        );
        assert_eq!(
            p.raise_octaves(2).unwrap(),
            pitch!(c, 6)
        );
    }
}
