## An As-of-yet Unnamed Music Theory and Rendering Library

The intention of this library is to describe musical pitch sets
and other music theoretic constructs with their revelant information
that is entailed from the particular instances being constructed.

With this library, one can perform:
- Tonal analysis -- answering questions like which chords or scales which into which others,
ways to voicelead between chords, names, modes, etc.
- Musical Combinatorics -- Go through all of the chord types, all the ways of voicing those
chord types, and all the ways to realize those voicings on a particular fretboard.
- Automated Scoring -- Output lilypond source code for document generation, or VexTab for
integration with JS frontends.

### Applications
- What sub- and super- chords exist within a given chord or scale?
- What are all the exhaustive ways to play a given chord on a given fretted instrument?
- What should I name a given set of notes? Are there other plausible names, assuming various other root notes?


### Philosophy
This crate takes most of its influence from the following things:
- Dmitri Tymoczko's work most well-fleshed-out in his book Geometry of Music.
- My own manual explorations of the combinatoric space of pitch sets, their chord qualities, ways of voicing them, etc.
- The mathematical structures that providing meaningful summations of possibilities for various
musical choices that one often finds while composing/analyzing a piece, or even while improvising.
- My conviction that the combinatoric space _is not too big_ for thorough and complete overview, at the very least.
- With the right tools, generating teaching tools for theory, instrument practice, references, etc.

### TODO
- Chord types as enums -- FiveNoteChordQuality, SixNoteChordQuality, SevenNoteChordQuality, IrregularScaleQuality
- Library Error Type
- Various geometric calculations of sub-/super-chords, voiceleadings, etc.
- Lilypond and VexTab features
- Chord name parsing
- Melodic Sequencer