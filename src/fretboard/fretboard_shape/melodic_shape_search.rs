use std::collections::HashMap;
use crate::chord::NoteSet;
use crate::fretboard::Fretboard;
use crate::fretboard::fretboard_shape::FretboardShape;
use crate::note::note::Note;

/// A struct intended to wrap a [FretboardShape], and add some scoring metrics.
pub struct MelodicFretboardShape<'a> {
    shape: FretboardShape<'a>,
    /// Fitness / playability score. Higher is worse -- it's a cost / penalty metric.
    /// We do not define our scoring metric inherent in this struct.
    /// Users are free to design their own scoring means.
    score: usize,
}

impl<'a> MelodicFretboardShape<'a> {
    /// We do not define our metric for "completion" in this struct.
    /// Users are free to design their own criteria to define "completion".
    pub fn is_complete(&self) -> bool {
        let span = self.shape.span();
        false
    }
}

/// Broken down by various classifications
pub struct ScaleShapeSearchResult<'a> {
    simple: HashMap<Note, Vec<MelodicFretboardShape<'a>>>
}

impl<'a> ScaleShapeSearchResult<'a> {
    pub fn new() -> Self {
        Self {
            simple: HashMap::new(),
        }
    }
}

fn recursive_scale_search<'a>(
    chord: &Vec<Note>,
    fretboard: &'a Fretboard,
) -> anyhow::Result<Vec<MelodicFretboardShape<'a>>> {
    Ok(vec![])
}

/// Searches over the space of possible arrangements of fretboard shapes.
pub fn find_scale_shapes<'a>(
    chord: &Vec<Note>,
    starting_note: &Note,
    fretboard: &'a Fretboard,
) -> anyhow::Result<ScaleShapeSearchResult<'a>> {
    //  4. Perform a recursive search.
    //  5. Only keep the complete shapes.
    //  6. Sort into sublists by score.
    //  7. Sort each sublist by span, removing anything with a span >= 6, flattening to one list.
    //  8. Sort them into their respective categories.
    // TODO We're normalizing the spelling because this is done in the Python, is this necessary?
    let starting_note = starting_note.spelled_as_in(chord)?;
    let chord = NoteSet::new(chord.clone(), Some(&starting_note));
    // Initialize the recursive search
    let mut scale_shapes: Vec<MelodicFretboardShape> = vec![];
    let mut span: usize = 0;
    let mut notes_on_curr_str = 1;
    let mut score: usize = 0;
    let mut first_fretted_note = fretboard.note_on_string(&starting_note, 0)?;
    // Giving ourselves headroom such that even if our shape progressed completely downward from the start,
    // we would not run into the edge of the fretboard, thus killing off a search into shapes
    // that could have been explored and which are *perhaps* playable up twelve frets.
    if first_fretted_note.fret < 7 {
        first_fretted_note = first_fretted_note.up_n_frets(12).unwrap();
    }
    let frets = vec![first_fretted_note];
    Ok(ScaleShapeSearchResult::new())
}
