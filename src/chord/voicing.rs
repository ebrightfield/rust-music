use crate::chord::pc_set::PcSet;
use crate::chord::spelling::spell_pc_set;
use crate::pitch::Pitch;

/// Returns a vector of increasing midi note values, based on a series of
/// vertically stacked intervals and a starting pitch.
fn stack_midi_from_intervals(pitch: &Pitch, intervals: &Vec<u8>) -> Vec<u8> {
    let mut midi_notes = vec![pitch.midi_note];
    intervals.iter()
        .for_each(|i| midi_notes.push(midi_notes.last().unwrap() + i));
    midi_notes
}

/// A collection of [Pitch]. [Note] duplicates are allowed.
pub struct Voicing(pub Vec<Pitch>);

impl Voicing {
    pub fn from_intervals(root: &Pitch, intervals: &Vec<u8>) -> anyhow::Result<Self> {
        let midi_notes = stack_midi_from_intervals(root, intervals);
        let pc_set = PcSet::from_midi_notes(&midi_notes);
        let spelling = spell_pc_set(&root.note, &pc_set)?;
        let pitches = midi_notes.iter()
            .map(|m| Pitch::spelled_as_in(*m, &spelling).unwrap())
            .collect();
        Ok(Self(pitches))
    }
}
