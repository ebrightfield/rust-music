// REQ-O22, O23, O25: convert_tab with convention-aware string translation.

use crate::ast::tab::OwnedTab;
use crate::error::MusicRonError;
use music::fretboard::{Fretboard, StringConvention};
use music::notation::rhythm::RhythmicNotatedEvent;

/// Convert a parsed `OwnedTab` into resolved `RhythmicNotatedEvent`s using the
/// given fretboard for pitch lookup.
///
/// String indices from the document are translated through the tab's
/// `string_convention` (default: `OneIndexedFromHigh`) into the internal
/// 0-indexed-from-low representation that `Fretboard::sounded_note` expects.
pub fn convert_tab<'a>(
    owned: &OwnedTab,
    fretboard: &'a Fretboard,
) -> Result<Vec<RhythmicNotatedEvent<'a>>, MusicRonError> {
    let convention = owned
        .string_convention
        .unwrap_or(StringConvention::OneIndexedFromHigh);

    let num_strings = fretboard.open_strings.len() as u8;

    let mut out = Vec::with_capacity(owned.events.len());
    for (idx, ev) in owned.events.iter().enumerate() {
        let path = format!("events[{idx}]");

        let duration =
            crate::convert::duration::resolve(&ev.duration, &format!("{path}.duration"))?;

        // Translate user-facing string number to 0-indexed-from-low for sounded_note.
        let zero_indexed = convention_to_zero_indexed(convention, ev.string, num_strings)
            .ok_or_else(|| MusicRonError::InvalidPitch {
                input: format!(
                    "string {} out of range for {num_strings}-string instrument",
                    ev.string
                ),
                path: format!("{path}.string"),
            })?;

        let sounded = fretboard.sounded_note(zero_indexed, ev.fret).map_err(|e| {
            MusicRonError::InvalidPitch {
                input: e.to_string(),
                path: path.clone(),
            }
        })?;

        out.push(RhythmicNotatedEvent::fretted(sounded, duration));
    }
    Ok(out)
}

/// Translate a user-facing string number in the given convention to the
/// 0-indexed-from-low value that `Fretboard::sounded_note` expects.
/// Returns `None` if the input is out of range.
fn convention_to_zero_indexed(
    convention: StringConvention,
    user_string: u8,
    num_strings: u8,
) -> Option<u8> {
    match convention {
        StringConvention::ZeroIndexedFromLow => {
            if user_string < num_strings {
                Some(user_string)
            } else {
                None
            }
        }
        StringConvention::OneIndexedFromHigh => {
            // user_string 1 = highest (index num_strings-1), user_string num_strings = lowest (index 0)
            if user_string >= 1 && user_string <= num_strings {
                Some(num_strings - user_string)
            } else {
                None
            }
        }
        StringConvention::OneIndexedFromLow => {
            if user_string >= 1 && user_string <= num_strings {
                Some(user_string - 1)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::common::OwnedDuration;
    use crate::ast::common::OwnedTuning;
    use crate::ast::tab::{OwnedTab, OwnedTabEvent};
    use music::fretboard::STD_6STR_GTR;
    use music::note::note::Note;

    fn make_tab(events: Vec<OwnedTabEvent>, convention: Option<StringConvention>) -> OwnedTab {
        OwnedTab {
            meta: None,
            version: None,
            tuning: OwnedTuning::Named("standard".into()),
            string_convention: convention,
            events,
        }
    }

    fn make_event(string: u8, fret: u8) -> OwnedTabEvent {
        OwnedTabEvent {
            string,
            fret,
            duration: OwnedDuration::Shorthand("4".into()),
        }
    }

    #[test]
    fn default_convention_is_one_indexed_from_high() {
        // String 6 in OneIndexedFromHigh = lowest string (low E)
        let tab = make_tab(vec![make_event(6, 0)], None);
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn one_indexed_from_high_string_1_is_high_e() {
        // String 1 = highest string = high E (index 5)
        let tab = make_tab(vec![make_event(1, 0)], None);
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 1);
        // High E open = E5
        match &result[0].event {
            music::notation::rhythm::NotatedEvent::SingleEvent(
                music::notation::rhythm::SingleEvent::Fretted(sn),
                _,
            ) => {
                assert_eq!(sn.pitch.note, Note::E);
                assert_eq!(sn.pitch.octave, 5);
            }
            _ => panic!("expected Fretted"),
        }
    }

    #[test]
    fn zero_indexed_from_low() {
        // String 0 = lowest = low E
        let tab = make_tab(
            vec![make_event(0, 0)],
            Some(StringConvention::ZeroIndexedFromLow),
        );
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].event {
            music::notation::rhythm::NotatedEvent::SingleEvent(
                music::notation::rhythm::SingleEvent::Fretted(sn),
                _,
            ) => {
                assert_eq!(sn.pitch.note, Note::E);
                assert_eq!(sn.pitch.octave, 3);
            }
            _ => panic!("expected Fretted"),
        }
    }

    #[test]
    fn one_indexed_from_low() {
        // String 1 = lowest = low E
        let tab = make_tab(
            vec![make_event(1, 0)],
            Some(StringConvention::OneIndexedFromLow),
        );
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].event {
            music::notation::rhythm::NotatedEvent::SingleEvent(
                music::notation::rhythm::SingleEvent::Fretted(sn),
                _,
            ) => {
                assert_eq!(sn.pitch.note, Note::E);
                assert_eq!(sn.pitch.octave, 3);
            }
            _ => panic!("expected Fretted"),
        }
    }

    #[test]
    fn fret_3_on_a_string() {
        // String 5 in OneIndexedFromHigh = A string (index 1), fret 3 = C4
        let tab = make_tab(vec![make_event(5, 3)], None);
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].event {
            music::notation::rhythm::NotatedEvent::SingleEvent(
                music::notation::rhythm::SingleEvent::Fretted(sn),
                _,
            ) => {
                assert_eq!(sn.pitch.note, Note::C);
                assert_eq!(sn.pitch.octave, 4);
            }
            _ => panic!("expected Fretted"),
        }
    }

    #[test]
    fn string_out_of_range_is_error() {
        // String 7 on a 6-string instrument (OneIndexedFromHigh, max is 6)
        let tab = make_tab(vec![make_event(7, 0)], None);
        match convert_tab(&tab, &STD_6STR_GTR) {
            Err(MusicRonError::InvalidPitch { path, .. }) => {
                assert!(path.contains("string"));
            }
            Err(_) => panic!("expected InvalidPitch"),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn string_zero_in_one_indexed_is_error() {
        // String 0 is invalid in OneIndexedFromHigh
        let tab = make_tab(vec![make_event(0, 0)], None);
        assert!(matches!(
            convert_tab(&tab, &STD_6STR_GTR),
            Err(MusicRonError::InvalidPitch { .. })
        ));
    }

    #[test]
    fn multiple_events() {
        let tab = make_tab(
            vec![make_event(1, 0), make_event(6, 3), make_event(3, 5)],
            None,
        );
        let result = convert_tab(&tab, &STD_6STR_GTR).unwrap();
        assert_eq!(result.len(), 3);
    }

    // convention_to_zero_indexed unit tests
    #[test]
    fn convention_translation_boundaries() {
        // ZeroIndexedFromLow: valid 0..5, invalid 6
        assert_eq!(
            convention_to_zero_indexed(StringConvention::ZeroIndexedFromLow, 0, 6),
            Some(0)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::ZeroIndexedFromLow, 5, 6),
            Some(5)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::ZeroIndexedFromLow, 6, 6),
            None
        );

        // OneIndexedFromHigh: 1→5, 6→0, 0→None, 7→None
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromHigh, 1, 6),
            Some(5)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromHigh, 6, 6),
            Some(0)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromHigh, 0, 6),
            None
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromHigh, 7, 6),
            None
        );

        // OneIndexedFromLow: 1→0, 6→5, 0→None, 7→None
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromLow, 1, 6),
            Some(0)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromLow, 6, 6),
            Some(5)
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromLow, 0, 6),
            None
        );
        assert_eq!(
            convention_to_zero_indexed(StringConvention::OneIndexedFromLow, 7, 6),
            None
        );
    }
}
