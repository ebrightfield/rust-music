## Rust Music Semantics
A music theory library focused on precision, correctness, and expressiveness.

This library provides:

1. A powerful music theory engine, with precise types appropriate for all manner of situations.
2. Functions for inferring chord/scale spellings and names, as well as parsing
chord/scale names into sets of notes.
3. A means of converting Rust types to Lilypond or VexTab source code,
allowing for automated document generation or application development.

#### General Use Cases for this Library:
- Tonal analysis and general music theory computation -- answering questions like which chords or scales fit into which others, finding voiceleading paths between chords, finding symmetries, iterating through
the various possible voicings of a chord, parsing or generating chord/scale names, etc. Functions
for answering all of these questions exist out-of-the-box.
- Automated scoring -- Output lilypond source code for document generation, or VexTab for
integration with JS frontends. 
- Application development -- VexTab is a powerful way to generate music notation in a UI.
Using Web Assembly, Rust is easy to incorporate into browser-based frontend frameworks.

### Code Examples

See the `music/examples` directory for some demonstrations on the basic types.

### Philosophy
This crate takes much influence from Dmitri Tymoczko's work, which you can find in his book Geometry of Music.

My conviction that the combinatoric space that falls out of the twelve notes _is not too big_ for thorough and complete overview, at the very least.

I also believe that having a solid foundation in "thinking through musical space" is a key
part of musicianship, no matter whether performing a memorized piece or improvising, composing or analyzing, reading or transcribing.

Good musicianship requires _much_ more than music theory.
But with the right tools, protocols, and practice, it should be no burden to acquire the kind of
music theory understanding that we usually only expect of professional musicians.

### TODO
- Chord types as enums -- FiveNoteChordQuality, SixNoteChordQuality, IrregularScaleQuality
- Various geometric calculations of sub-/super-chords, etc.
- A few more Lilypond code-gen feature
- A few more Vextab code-gen feature
- Chord name parsing (regex)
- Melodic Sequencer
- PcSet vs PcMultiset
- Do a pass on ergonomics
- Common scale check -- Check which common scales a chord fits into.

### Lilypond TODO
- Repeat blocks
- Breaks, barlines,
- Modifiers like `\hide`, `\stemUp`, `\stemDown`, etc
- Context blocks (create a lazy static `Slashes` context for a useful example)
- Chord name customization
- Cleaner `.ly` output formatting, particularly with tabs and line breaks.