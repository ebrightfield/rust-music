use std::fmt::Debug;
use itertools::Itertools;
use crate::error::MusicSemanticsError;
use crate::note::{Note, Pitch};
use crate::note_collections::geometry::contour::Movement;
use crate::note_collections::Voicing;

/// A mapping between two Voicings defined by paths
/// from one Voicing's pitches to that of the other.
#[derive(Debug, Clone)]
pub struct Voiceleading {
    /// The starting [Voicing].
    pub from: Voicing,
    /// The ending [Voicing] after applying the `paths`.
    pub to: Voicing,
    /// `self.paths[i]` describes the number of semitones moved up or down
    /// by the pitch at `self.from.0[i]`.
    pub paths: Vec<i8>,
}

impl Voiceleading {
    /// If `to` is provided, this constructor tries to spell `self.to` in accordance.
    /// This can be used to assert that the paths applied conform to a given target chord.
    pub fn new(
        from: Voicing,
        paths: Vec<i8>,
        to: Option<&Vec<Note>>,
        rules: &Option<&Vec<Box<dyn VoiceleadingRule>>>,
    ) -> Result<Self, MusicSemanticsError> {
        let paths_applied = from.apply_paths(&paths, to)?;
        if let Some(rules) = rules {
            let mut violations = vec![];
            for rule in *rules {
                if !rule.apply(&from, &paths, &paths_applied) {
                    violations.push(rule.name());
                }
            }
            if !violations.is_empty() {
                return Err(MusicSemanticsError::VoiceleadingViolation(violations));
            }
        }
        Ok(Self {
            from: from,
            to: Voicing::new(paths_applied),
            paths,
        })
    }

    /// Returns a Vec of 2-tuples of `(score, voiceleading)`, sorted by the score.
    /// Scoring is currently done by naive distance metric, plans are to add the means
    /// to change this once the rest of the API stabilizes.
    pub fn find_all(
        from: &Voicing,
        to: &Vec<Note>,
        rules: Option<&Vec<Box<dyn VoiceleadingRule>>>,
    ) -> Result<Vec<(usize, Voiceleading)>, MusicSemanticsError> {
        let mut voiceleadings = vec![];
        for ordering in (0..from.len()).permutations(from.len()) {
            for contour_combo in vec![Movement::Ascending, Movement::Descending]
                .iter().combinations_with_replacement(from.len()) {
                let mut paths = vec![];
                for (i, j) in ordering.iter().enumerate() {
                    let departure = from.get(i).unwrap();
                    let destination = departure.up_to_note(&to[*j])?;
                    let mut path = i8::try_from(destination.midi_note - departure.midi_note).unwrap();
                    if *contour_combo[i] == Movement::Descending && path != 0 {
                        path = path - 12;
                    }
                    paths.push(path);
                }
                let maybe_valid = Voiceleading::new(
                    from.clone(),
                    paths,
                    Some(to),
                    &rules,
                );
                if let Ok(voiceleading) = maybe_valid {
                    let score = naive_distance(&voiceleading);
                    voiceleadings.push((score, voiceleading));
                }
            }
        }
        voiceleadings.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        Ok(voiceleadings)
    }
}

/// A distance metric where we simply sum the absolute values of all the paths of a voiceleading.
pub fn naive_distance(v: &Voiceleading) -> usize {
    v.paths.iter().map(|p| usize::try_from(p.abs()).unwrap()).sum()
}

/// Describes a restriction on voiceleading.
pub trait VoiceleadingRule: Debug {
    fn apply(&self, from: &Voicing, paths: &Vec<i8>, to: &Vec<Pitch>) -> bool;
    
    fn name(&self) -> String;
}

#[derive(Debug)]
pub struct NoVoxCrossings;
impl VoiceleadingRule for NoVoxCrossings {
    fn apply(&self, _from: &Voicing, _paths: &Vec<i8>, to: &Vec<Pitch>) -> bool {
        to.clone() == *Voicing::new(to.clone())
    }

    fn name(&self) -> String {
        "No Voice Crossings".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_voiceleadings() {
        let v1 = Voicing::new(vec![
                Pitch::new(Note::C, 4).unwrap(),
                Pitch::new(Note::G, 4).unwrap(),
                Pitch::new(Note::E, 5).unwrap(),
            ]);
        let ch2 = vec![Note::F, Note::A, Note::C];
        let voiceleadings = Voiceleading::find_all(&v1, &ch2,
        Some(&vec![Box::new(NoVoxCrossings)])
        ).unwrap();
        println!("{:#?}", voiceleadings[2]);
    }
}