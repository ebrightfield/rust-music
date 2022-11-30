use std::collections::HashMap;
use crate::note_collections::NoteSet;
use crate::fretboard::Fretboard;
use crate::fretboard::fretboard_shape::FretboardShape;
use crate::fretboard::fretted_note::SoundedNote;
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

fn tally_new_violations(frets: &Vec<SoundedNote>) -> (usize, usize) {
    let mut same_str_violations = 0;
    let mut str_xing_violations = 0;
    // Four or more frets up on the same string.
    // (more than four should be screened out upstream)
    if frets.len() > 1 && {
        let last = frets.last().unwrap();
        let second_to_last = &frets[frets.len() - 2];
        second_to_last.string == last.string && last.fret - second_to_last.fret >= 4
    } {
            same_str_violations += 1;
    }
    // Four frets in three notes.
    // e.g. Fingering patterns on a string such as 1-4-5 or 1-2-5
    if frets.len() > 2 && {
        let last = frets.last().unwrap();
        let second_to_last = &frets[frets.len() - 2];
        let third_to_last = &frets[frets.len() - 3];
        third_to_last.string == second_to_last.string &&
            second_to_last.string == last.string &&
            last.fret - third_to_last.fret == 4
    } {
        same_str_violations += 1;
    }
    if frets.len() > 1 && {
        let last = frets.last().unwrap();
        let second_to_last = &frets[frets.len() - 2];
        second_to_last.string + 1 == last.string && second_to_last.fret as isize - last.fret as isize == 4
    } {
        str_xing_violations += 1;
    }

    //
    if frets.len() > 2 && {
        let last = frets.last().unwrap();
        let second_to_last = &frets[frets.len() - 2];
        let third_to_last = &frets[frets.len() - 3];
        let did_xing_twice = third_to_last.string + 2 == second_to_last.string + 1 &&
            second_to_last.string + 1 == last.string;
        let first_xing = third_to_last.fret as isize - second_to_last.fret as isize;
        let second_xing = second_to_last.fret as isize - last.fret as isize;
        let xings = (first_xing, second_xing);
        let is_bad_xing = !vec![(1,1), (1,2), (2,1), (2,2)].contains(&xings);
        did_xing_twice && is_bad_xing
    } {
        str_xing_violations += 1;
    }
    (same_str_violations, str_xing_violations)
}

#[cfg(test)]
mod tests {
    use crate::fretboard::STD_6STR_GTR;
    use super::*;

    #[test]
    fn test_violations() {
        // One note should have no violations.
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let frets = vec![s1];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (0,0));
        // These pairs notes should have no violations.
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let s2 = (*STD_6STR_GTR).sounded_note(0, 2).unwrap();
        let frets = vec![s1, s2];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (0,0));
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let s2 = (*STD_6STR_GTR).sounded_note(0, 3).unwrap();
        let frets = vec![s1, s2];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (0,0));
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let s2 = (*STD_6STR_GTR).sounded_note(0, 4).unwrap();
        let frets = vec![s1, s2];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (0,0));

        // Violation -- Four frets across the same string
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let s2 = (*STD_6STR_GTR).sounded_note(0, 5).unwrap();
        let frets = vec![s1, s2];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (1,0));

        // Violation -- Three notes over four notes on same string
        let s1 = (*STD_6STR_GTR).sounded_note(0, 1).unwrap();
        let s2 = (*STD_6STR_GTR).sounded_note(0, 2).unwrap();
        let s3 = (*STD_6STR_GTR).sounded_note(0, 5).unwrap();
        let frets = vec![s1, s2, s3];
        let violations = tally_new_violations(&frets);
        assert_eq!(violations, (1,0));
    }
}