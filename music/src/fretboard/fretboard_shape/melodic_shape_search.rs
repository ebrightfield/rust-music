use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use itertools::Itertools;
use crate::note_collections::NoteSet;
use crate::fretboard::Fretboard;
use crate::fretboard::fretted_note::SoundedNote;
use crate::note::note::Note;
use crate::note::pitch::Pitch;

/// A struct intended to wrap a [FretboardShape], and add some scoring metrics.
#[derive(Debug, Clone)]
pub struct MelodicFretboardShape<'a> {
    shape: Vec<SoundedNote<'a>>,
    /// Fitness / playability score. Higher is worse -- it's a cost / penalty metric.
    /// We do not define our scoring metric inherent in this struct.
    /// Users are free to design their own scoring means.
    score: usize,
}

impl<'a> Display for MelodicFretboardShape<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.shape.iter()
            .map(|note| note.to_string())
            .join(" ");
        write!(f, "{}", s)
    }
}

impl<'a> MelodicFretboardShape<'a> {
    /// Two octave range
    pub fn is_complete(&self) -> bool {
        let (low, high) = self.range();
        high.midi_note - low.midi_note >= 24
    }

    pub fn range(&self) -> (Pitch, Pitch) {
        let mut pitches: Vec<Pitch> = self.shape
            .iter()
            .map(|p| p.pitch.clone())
            .collect();
        pitches.sort_by(|a, b| a.midi_note.partial_cmp(&b.midi_note).unwrap());
        (pitches.first().unwrap().clone(), pitches.last().unwrap().clone())
    }

    /// Minimum and maximum fret numbers, *including* open strings.
    pub fn span(&self) -> (u8, u8) {
        let mut lowest: u8 = u8::MAX;
        let mut highest: u8  = u8::MIN;
        for SoundedNote { fret, .. } in &self.shape {
            if *fret < lowest {
                lowest = *fret;
            }
            if *fret > highest {
                highest = *fret;
            }
        }
        (lowest, highest)
    }
}

const N_PER_STRING_TUPLES: &[(usize, usize)] = &[(2,2), (2,3), (3,2), (3,3)];

/// Broken down by various classifications
pub struct ScaleShapeSearchResult<'a> {
    pub simple: Vec<MelodicFretboardShape<'a>>,
    pub open: MelodicFretboardShape<'a>,
    pub n_per_string_2_2: HashMap<Note, Option<MelodicFretboardShape<'a>>>,
    pub n_per_string_2_3: HashMap<Note, Option<MelodicFretboardShape<'a>>>,
    pub n_per_string_3_2: HashMap<Note, Option<MelodicFretboardShape<'a>>>,
    pub n_per_string_3_3: HashMap<Note, Option<MelodicFretboardShape<'a>>>,
    pub other: HashMap<Note, Vec<MelodicFretboardShape<'a>>>,
}

impl<'a> ScaleShapeSearchResult<'a> {
    pub fn new() -> Self {
        Self {
            simple: vec![],
            open: MelodicFretboardShape {
                shape: vec![],
                score: 0
            },
            n_per_string_2_2: HashMap::new(),
            n_per_string_2_3: HashMap::new(),
            n_per_string_3_2: HashMap::new(),
            n_per_string_3_3: HashMap::new(),
            other: HashMap::new(),
        }
    }

    pub fn from_raw_search_result(
        chord: &Vec<Note>,
        fretboard: &'a Fretboard,
    ) -> anyhow::Result<Self> {
        let result = find_all_scale_shapes(chord, fretboard)?;
        // Calculate open shape
        let open_shape = find_open_scale_shape(
            chord,
            fretboard,
        )?;
        let mut instance = Self::new();
        instance.open = open_shape;
        // TODO add an is_playable check before appending
        for (note, _shapes) in result {
            // categorize into simple shapes, or other
            for n in N_PER_STRING_TUPLES {
                let shape = find_n_note_per_string_shapes(
                    n.clone(),
                    chord,
                    &note,
                    fretboard,
                ).ok();
                if *n == (2,2) {
                    instance.n_per_string_2_2.insert(note, shape);
                } else if *n == (2,3) {
                    instance.n_per_string_2_3.insert(note, shape);
                } else if *n == (3,2) {
                    instance.n_per_string_3_2.insert(note, shape);
                } else if *n == (3,3) {
                    instance.n_per_string_3_3.insert(note, shape);
                }
            }
        }
        todo!()
    }
}

pub fn find_open_scale_shape<'a>(
    chord: &Vec<Note>,
    fretboard: &'a Fretboard,
) -> anyhow::Result<MelodicFretboardShape<'a>> {
    if chord.is_empty() {
        return Err(anyhow!("Empty set of notes"));
    }
    let mut first_note = fretboard.sounded_note(0, 0)?;
    while !chord.contains(&first_note.pitch.note) {
        first_note = first_note.up_n_frets(1)?;
    }
    let chord = NoteSet::new(chord.clone(), Some(&first_note.pitch.note));
    let next_note = first_note.next_note_next_string(&chord)
        .unwrap_or(first_note.next_note_same_string(&chord).unwrap());
    // Manually add our next note.
    let mut notes = MelodicFretboardShape {
        shape: vec![first_note.clone(), next_note],
        score: 0
    };
    while !{
        let last_note = notes.shape.last().unwrap();
        let last_string = fretboard.num_strings() - 1;
        let on_last_string_past_5th = last_note.string == last_string &&
            last_note.fret >=5;
        on_last_string_past_5th
    } {
        let last_note = notes.shape.last().unwrap();
        let next_note = last_note.next_note_next_string(&chord)
            .unwrap_or(last_note.next_note_same_string(&chord).unwrap());
        notes.shape.push(next_note);
    }
    Ok(notes)
}

pub fn find_n_note_per_string_shapes<'a>(
    n: (usize, usize),
    chord: &Vec<Note>,
    starting_note: &Note,
    fretboard: &'a Fretboard,
) -> anyhow::Result<MelodicFretboardShape<'a>> {
    if chord.is_empty() {
        return Err(anyhow!("Empty set of notes"));
    }
    let (value_1, value_2) = n;
    if value_1 < 2 || value_2 < 2 {
        return Err(anyhow!("Invalid n ({}, {})", value_1, value_2));
    }
    let starting_note = starting_note.spelled_as_in(chord)?;
    let chord = NoteSet::new(chord.clone(), Some(&starting_note));
    let first_fretted_note = fretboard.note_on_string(&starting_note, 0)?;
    let mut using_value_1 = true;
    let mut shape = vec![];
    shape.push(first_fretted_note);
    for _ in 0..fretboard.num_strings() {
        let mut num_notes_on_curr_str = 1;
        while (using_value_1 && num_notes_on_curr_str < value_1) || num_notes_on_curr_str < value_2 {
            let last_note = shape.last().unwrap();
            shape.push(last_note.next_note_same_string(&chord)?);
            num_notes_on_curr_str += 1;
        }
        using_value_1 = !using_value_1;
        let last_note = shape.last().unwrap();
        shape.push(last_note.next_note_next_string(&chord)?)
    }
    Ok(MelodicFretboardShape { shape, score: 0 })
}

/// Finds scale shapes starting from each note.
pub fn find_all_scale_shapes<'a>(
    chord: &Vec<Note>,
    fretboard: &'a Fretboard,
    ) -> anyhow::Result<HashMap<Note, Vec<MelodicFretboardShape<'a>>>> {
    Ok(chord
        .iter()
        .map(|note| melodic_shapes_at_starting_note(chord, note, fretboard)
            .map(|ok| (note.clone(), ok)
        ))
        .into_iter()
        .flatten()
        .collect()
    )
}

/// Meant to be cloned across different branches of the recursive search tree.
#[derive(Debug, Clone)]
struct RecursiveSearchParams<'a> {
    frets: Vec<SoundedNote<'a>>,
    notes_on_curr_string: usize,
    span_on_curr_string: usize,
    score: usize,
    fretboard: &'a Fretboard,
}

fn normalize_octave_register(
    mut frets: Vec<SoundedNote>,
) -> Vec<SoundedNote> {
    while frets.iter().all(|note| note.fret > 12) {
        frets = frets.iter().map(|f| f.down_an_octave().unwrap()).collect();
    }
    frets
}

/// We never recurse many levels deep, because the anatomical restrictions of
/// the hand force recursion to terminate early and often. There are many
/// branches, but they are all shallow.
fn recursive_melodic_search<'a>(
    chord: &NoteSet,
    mut params: RecursiveSearchParams<'a>,
    shapes: &mut Vec<MelodicFretboardShape<'a>>,
) {
    let new_violations = tally_new_violations(&params.frets);
    params.score += new_violations.0 + new_violations.1;
    // If we've completed 2 octaves, we're done.
    if params.frets.len() > 2 * chord.len() {
        let frets = normalize_octave_register(params.frets);
        let shape = MelodicFretboardShape {
            shape: frets,
            score: params.score,
        };
        shapes.push(shape);
        return;
    }
    // The boolean flag below is marked true when we recurse deeper.
    // If we don't, then we've traversed as far as we can on this shape, so
    // we push it into the search results and return.
    let mut was_dead_end = true;

    let last_fret = params.frets.last().unwrap(); // We know it'll never be empty.
    let next_note_same_string = last_fret.next_note_same_string(chord).ok();
    if next_note_same_string.is_none() {
        // This could only happen if we are attempting to go past the 35th fret.
        // We can terminate the search in this case, as we're nowhere near it.
        // Practically speaking, this should be an impossible condition, but since we
        // aren't 100% sure, we're guarding against it here.
        return;
    }
    let next_note_same_string = next_note_same_string.unwrap();
    let distance_to_next_note: usize = (next_note_same_string.fret - last_fret.fret) as usize;
    let span: usize = params.span_on_curr_string + distance_to_next_note;
    if span < 5 && params.notes_on_curr_string < 4 {
        was_dead_end = false;
        let mut new_params = params.clone();
        new_params.span_on_curr_string = span;
        new_params.notes_on_curr_string += 1;
        new_params.frets.push(next_note_same_string.clone());
        recursive_melodic_search(chord, new_params, shapes);
    }
    if params.fretboard.num_strings() > last_fret.string + 1 {
        let next_string = &params.fretboard.open_strings[last_fret.string as usize + 1];
        let this_string = &params.fretboard.open_strings[last_fret.string as usize];
        let gap = next_string.midi_note - this_string.midi_note;
        let can_change_strings = next_note_same_string.fret >= gap;
        if can_change_strings && !{
            params.notes_on_curr_string == 1 && params.frets.len() > 1 &&
                {
                    let second_to_last = &params.frets[params.frets.len()-2];
                    let third_to_last = &params.frets[params.frets.len()-3];
                    (third_to_last.fret as isize - second_to_last.fret as isize > 1 &&
                        distance_to_next_note < 3
                    ) ||
                    (second_to_last.fret as isize - last_fret.fret as isize > 3 &&
                        distance_to_next_note < gap as usize
                    )
                }
        } {
            was_dead_end = false;
            let next_note_next_str = last_fret.next_note_next_string(chord)
                .unwrap();
            let mut new_params = params.clone();
            new_params.span_on_curr_string = 0;
            new_params.notes_on_curr_string = 1;
            new_params.frets.push(next_note_next_str);
            recursive_melodic_search(chord, new_params, shapes);
        }
    }
    if distance_to_next_note >= 7 && params.fretboard.num_strings() > last_fret.string + 2 {
        let next_string = &params.fretboard.open_strings[last_fret.string as usize + 2];
        let this_string = &params.fretboard.open_strings[last_fret.string as usize];
        let gap = next_string.midi_note - this_string.midi_note;
        let can_change_strings = next_note_same_string.fret >= gap;
        if can_change_strings && !{
            params.notes_on_curr_string == 1 && params.frets.len() > 1 &&
                {
                    let second_to_last = &params.frets[params.frets.len()-2];
                    second_to_last.fret as isize - last_fret.fret as isize > 1
                        || (
                        second_to_last.fret as isize - last_fret.fret as isize > 3 &&
                        distance_to_next_note < gap as usize
                        )
                }
        } {
            was_dead_end = false;
            // Add the note two strings up, and recurse.
            let fret = params.fretboard.which_fret(
                &next_note_same_string.pitch.note,
                last_fret.string + 2,
            ).unwrap();
            let mut next_note = params.fretboard.sounded_note(
                last_fret.string + 2,
                fret
            ).unwrap();
            while next_note.pitch.midi_note < last_fret.pitch.midi_note {
                next_note = next_note.up_an_octave().unwrap();
            }
            let mut new_params = params.clone();
            new_params.span_on_curr_string = 0;
            new_params.notes_on_curr_string = 1;
            new_params.frets.push(next_note);
            recursive_melodic_search(chord, new_params, shapes);
        }
    }
    if was_dead_end {
        let frets = normalize_octave_register(params.frets);
        let shape = MelodicFretboardShape {
            shape: frets,
            score: params.score,
        };
        shapes.push(shape);
    }
}

/// Searches over the space of possible arrangements of fretboard shapes.
pub fn melodic_shapes_at_starting_note<'a>(
    chord: &Vec<Note>,
    starting_note: &Note,
    fretboard: &'a Fretboard,
) -> anyhow::Result<Vec<MelodicFretboardShape<'a>>> {
    // TODO We're normalizing the spelling because this is done in the Python, is this necessary?
    let starting_note = starting_note.spelled_as_in(chord)?;
    let chord = NoteSet::new(chord.clone(), Some(&starting_note));
    // Initialize the recursive search
    let mut first_fretted_note = fretboard.note_on_string(&starting_note, 0)?;
    // Giving ourselves headroom such that even if our shape progressed completely downward from the start,
    // we would not run into the edge of the fretboard, thus killing off a search into shapes
    // that could have been explored and which are *perhaps* playable up twelve frets.
    if first_fretted_note.fret < 7 {
        first_fretted_note = first_fretted_note.up_n_frets(12).unwrap();
    }
    let mut notes_on_curr_string = 1;
    let new_fret_same_str = first_fretted_note
        .next_note_same_string(&chord).unwrap();
    let span = (new_fret_same_str.fret - first_fretted_note.fret) as usize;
    let frets = vec![first_fretted_note.clone(), new_fret_same_str.clone()];
    let mut shapes = vec![];
    if span < 5 {
        notes_on_curr_string += 1;
        let params = RecursiveSearchParams {
            frets: frets.clone(),
            notes_on_curr_string,
            span_on_curr_string: span,
            score: 0,
            fretboard,
        };
        recursive_melodic_search(&chord, params, &mut shapes);
    }
    if fretboard.num_strings() > 1 {
        let this_string = fretboard.open_strings[first_fretted_note.string as usize];
        let next_string = fretboard.open_strings[(first_fretted_note.string + 1) as usize];
        let gap = next_string.midi_note - this_string.midi_note;
        if new_fret_same_str.fret >= gap {
            let next_note_next_str = first_fretted_note
                .next_note_next_string(&chord)?;
            let frets = vec![first_fretted_note.clone(), next_note_next_str.clone()];
            let params = RecursiveSearchParams {
                frets,
                notes_on_curr_string: 2,
                span_on_curr_string: 0,
                score: 0,
                fretboard,
            };
            recursive_melodic_search(&chord, params, &mut shapes);
        }
        if span >= 7 && fretboard.num_strings() > first_fretted_note.string + 2 {
            let next_string = &fretboard.open_strings[first_fretted_note.string as usize + 2];
            let this_string = &fretboard.open_strings[first_fretted_note.string as usize];
            let gap = next_string.midi_note - this_string.midi_note;
            let can_change_strings = new_fret_same_str.fret >= gap;
            if can_change_strings && !{
                // commented this condition out because I think it's irrelevant here
                //notes_on_curr_string == 1 && frets.len() > 1 &&
                    {
                        let second_to_last = &frets[frets.len()-2];
                        second_to_last.fret as isize - first_fretted_note.fret as isize > 1
                            || (
                            second_to_last.fret as isize - first_fretted_note.fret as isize > 3 &&
                                span < gap as usize
                        )
                    }
            } {
                let fret = fretboard.which_fret(
                    &new_fret_same_str.pitch.note,
                    first_fretted_note.string + 2,
                ).unwrap();
                let mut next_note = fretboard.sounded_note(
                    first_fretted_note.string + 2,
                    fret
                ).unwrap();
                while next_note.pitch.midi_note < first_fretted_note.pitch.midi_note {
                    next_note = next_note.up_an_octave().unwrap();
                }
                let frets = vec![first_fretted_note.clone(), next_note.clone()];
                let params = RecursiveSearchParams {
                    frets,
                    notes_on_curr_string: 1,
                    span_on_curr_string: 0,
                    score: 0,
                    fretboard,
                };
                recursive_melodic_search(&chord, params, &mut shapes);
            }
        }
    }
    let mut by_score: HashMap<usize, Vec<MelodicFretboardShape>> = HashMap::new();
    shapes
        .into_iter()
        .filter(|shape| shape.is_complete())
        .for_each(|shape| by_score
            .entry(shape.score)
            .or_insert_with(|| vec![])
            .push(shape)
        );
    let mut shapes: Vec<MelodicFretboardShape> = vec![];
    by_score
        .into_iter()
        .sorted_by_key(|e| e.0)
        .for_each(|entry| {
            let mut more_shapes = entry.1.clone();
            more_shapes.sort_by(|a,b| {
                let (a_min, a_max) = a.span();
                let a_span = a_max - a_min;
                let (b_min, b_max) = b.span();
                let b_span = b_max - b_min;
                a_span.partial_cmp(&b_span).unwrap()
            });
            more_shapes.retain(|s| {
                let (min, max) = s.span();
                let span = max - min;
                span < 6
            });
            shapes.extend(more_shapes);
        });

    Ok(shapes)
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

    #[test]
    fn scale_search() {
        let chord = vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B];
        let _result = melodic_shapes_at_starting_note(
            &chord,
            &Note::C,
            &*STD_6STR_GTR,
        ).unwrap();
        // for shape in result {
        //     println!("{}", shape);
        // }
    }

    #[test]
    fn find_open_shape() {
        let chord = vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B];

        let shape = find_open_scale_shape(
            &chord,
            &*STD_6STR_GTR,
        ).unwrap();
        let should_be = "1:0(E) 1:1(F) 1:3(G) 2:0(A) 2:2(B) 2:3(C) \
        3:0(D) 3:2(E) 3:3(F) 4:0(G) 4:2(A) 5:0(B) 5:1(C) 5:3(D) 6:0(E) 6:1(F) 6:3(G) 6:5(A)";
        //println!("{}", shape);
        assert_eq!(format!("{}", shape), should_be);
    }
}