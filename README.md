## An As-of-yet Unnamed Music Theory and Rendering Library

The intention of this library is to describe musical pitch sets
and other music theoretic constructs with their revelant information
that is entailed from the particular instances being constructed.

This library also intends to render objects into Lilypond source code,
for purposes of automated instructional document creation.

### Applications
- What sub- and super- chords exist within a given chord or scale?
- What are all the exhaustive ways to play a given chord on a given fretted instrument?
- What should I name a given set of notes? Are there other plausible names, assuming various other root notes?


### TODO
- Pitch struct, note + octave information
- Chord types as enums
- Library Error Type
- Various calculations of sub-/super-chords, whether something is intervallically symmetrical.


#### Finding Guitar Shapes
0. Find all combinations of n strings, where n = chord.len().
1. Find all voicings for a given chord.
2. Find all ways to order n notes.
3. You have:
   a. Regular, valid shapes
   b. Wider Interval shapes (shapes that have an consecutive  note pairing wider than an octave)
   c. Non Transposables
And can possibly "separate octaves", that is, do not equivocate across octaves with respect to shapes.
4. You do this by:
   a. Grouping output by Voicings, and having a set of GtrShapes in each.
   b. For Each Grouping:
      i. For Each Ordering:
        * Gather all frets (f_i, f_i+12) where f_i is the first fret found containing note relevant to a given ordering and string grouping.
        * Only add an f+12 if f < 6
        * Just attempt to make a gtrshape out of it, and if it's valid, add it in, making sure to distinguish its category