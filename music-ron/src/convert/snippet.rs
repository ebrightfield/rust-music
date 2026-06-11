use crate::ast::common::OwnedEvent;
use crate::ast::OwnedSnippet;
use crate::error::MusicRonError;
use music::notation::clef::Clef;
use music::notation::rhythm::RhythmicNotatedEvent;
use music::notation::rhythm::Tuplet;

/// Resolved clef + events from an `OwnedSnippet`.
pub struct ResolvedSnippet {
    pub clef: Clef,
    pub events: Vec<RhythmicNotatedEvent<'static>>,
}

impl std::fmt::Debug for ResolvedSnippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedSnippet")
            .field("clef", &self.clef)
            .field("events_len", &self.events.len())
            .finish()
    }
}

pub fn convert_snippet(owned: &OwnedSnippet) -> Result<ResolvedSnippet, MusicRonError> {
    let clef = resolve_clef(&owned.clef, "clef")?;
    let mut out = Vec::with_capacity(owned.events.len());
    convert_events(&owned.events, "events", &mut out)?;
    Ok(ResolvedSnippet { clef, events: out })
}

fn resolve_clef(name: &str, path: &str) -> Result<Clef, MusicRonError> {
    match name {
        "treble" => Ok(Clef::Treble),
        "treble8va" => Ok(Clef::Treble8va),
        "treble8ba" => Ok(Clef::Treble8ba),
        "bass" => Ok(Clef::Bass),
        other => Err(MusicRonError::UnknownClef {
            name: other.into(),
            path: path.into(),
        }),
    }
}

fn convert_events(
    evs: &[OwnedEvent],
    path_prefix: &str,
    out: &mut Vec<RhythmicNotatedEvent<'static>>,
) -> Result<(), MusicRonError> {
    let mut i = 0;
    while i < evs.len() {
        let path = format!("{path_prefix}[{i}]");
        match &evs[i] {
            OwnedEvent::Note { pitch, duration } => {
                let p = crate::convert::pitch::resolve(pitch, &format!("{path}.pitch"))?;
                let d = crate::convert::duration::resolve(duration, &format!("{path}.duration"))?;
                out.push(RhythmicNotatedEvent::pitch(p, d));
            }
            OwnedEvent::Chord { pitches, duration } => {
                let voicing =
                    crate::convert::pitch::resolve_voicing(pitches, &format!("{path}.pitches"))?;
                let d = crate::convert::duration::resolve(duration, &format!("{path}.duration"))?;
                out.push(RhythmicNotatedEvent::voicing(voicing, d));
            }
            OwnedEvent::Rest { duration } => {
                let d = crate::convert::duration::resolve(duration, &format!("{path}.duration"))?;
                out.push(RhythmicNotatedEvent::rest(d));
            }
            OwnedEvent::Tie => {
                // REQ-O19: a Tie must bridge two pitched events.
                let prev_ok = i > 0
                    && matches!(
                        evs[i - 1],
                        OwnedEvent::Note { .. } | OwnedEvent::Chord { .. }
                    );
                let next_ok = i + 1 < evs.len()
                    && matches!(
                        evs[i + 1],
                        OwnedEvent::Note { .. } | OwnedEvent::Chord { .. }
                    );
                if !(prev_ok && next_ok) {
                    return Err(MusicRonError::InvalidTie {
                        path,
                        reason: "tie must bridge two pitched events".into(),
                    });
                }
                // Fold tie into the following event (emit with tied=true).
                i += 1;
                let next_path = format!("{path_prefix}[{i}]");
                match &evs[i] {
                    OwnedEvent::Note { pitch, duration } => {
                        let p = crate::convert::pitch::resolve(
                            pitch,
                            &format!("{next_path}.pitch"),
                        )?;
                        let d = crate::convert::duration::resolve(
                            duration,
                            &format!("{next_path}.duration"),
                        )?;
                        out.push(RhythmicNotatedEvent::pitch_tied(p, d));
                    }
                    OwnedEvent::Chord { pitches, duration } => {
                        let v = crate::convert::pitch::resolve_voicing(
                            pitches,
                            &format!("{next_path}.pitches"),
                        )?;
                        let d = crate::convert::duration::resolve(
                            duration,
                            &format!("{next_path}.duration"),
                        )?;
                        out.push(RhythmicNotatedEvent::voicing_tied(v, d));
                    }
                    _ => unreachable!("guarded by next_ok check"),
                }
            }
            OwnedEvent::Tuplet {
                numerator,
                denominator,
                base,
                children,
            } => {
                // REQ-O18: recursively convert children, then validate.
                let mut child_events = Vec::with_capacity(children.len());
                convert_events(children, &format!("{path}.children"), &mut child_events)?;
                let tuplet = Tuplet::new(
                    child_events,
                    *numerator as usize,
                    *denominator as usize,
                    *base,
                );
                tuplet.validate().map_err(|e| MusicRonError::InvalidTuplet {
                    path: path.clone(),
                    reason: format!("{e}"),
                })?;
                out.push(tuplet.into());
            }
        }
        i += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::common::{OwnedDuration, OwnedPitch};
    use music::notation::rhythm::duration::DurationKind;

    fn note(pitch: &str, dur: &str) -> OwnedEvent {
        OwnedEvent::Note {
            pitch: OwnedPitch::Shorthand(pitch.into()),
            duration: OwnedDuration::Shorthand(dur.into()),
        }
    }

    fn rest(dur: &str) -> OwnedEvent {
        OwnedEvent::Rest {
            duration: OwnedDuration::Shorthand(dur.into()),
        }
    }

    #[test]
    fn resolve_clef_treble() {
        assert_eq!(resolve_clef("treble", "x").unwrap(), Clef::Treble);
    }

    #[test]
    fn resolve_clef_bass() {
        assert_eq!(resolve_clef("bass", "x").unwrap(), Clef::Bass);
    }

    #[test]
    fn resolve_clef_unknown() {
        let err = resolve_clef("soprano", "clef").unwrap_err();
        assert!(matches!(err, MusicRonError::UnknownClef { name, .. } if name == "soprano"));
    }

    #[test]
    fn convert_note_and_rest() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "treble".into(),
            events: vec![note("c4", "4"), rest("8")],
        };
        let resolved = convert_snippet(&snippet).unwrap();
        assert_eq!(resolved.events.len(), 2);
        assert_eq!(resolved.clef, Clef::Treble);
    }

    #[test]
    fn tie_between_notes() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "treble".into(),
            events: vec![note("c4", "4"), OwnedEvent::Tie, note("c4", "4")],
        };
        let resolved = convert_snippet(&snippet).unwrap();
        assert_eq!(resolved.events.len(), 2);
        assert!(resolved.events[1].tied);
    }

    #[test]
    fn tie_at_end_errors() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "treble".into(),
            events: vec![note("c4", "4"), OwnedEvent::Tie],
        };
        let err = convert_snippet(&snippet).unwrap_err();
        assert!(matches!(err, MusicRonError::InvalidTie { .. }));
    }

    #[test]
    fn tie_after_rest_errors() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "treble".into(),
            events: vec![rest("4"), OwnedEvent::Tie, note("c4", "4")],
        };
        let err = convert_snippet(&snippet).unwrap_err();
        assert!(matches!(err, MusicRonError::InvalidTie { .. }));
    }

    #[test]
    fn tuplet_triplet() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "bass".into(),
            events: vec![OwnedEvent::Tuplet {
                numerator: 3,
                denominator: 2,
                base: DurationKind::Eighth,
                children: vec![note("c3", "8"), note("e3", "8"), note("g3", "8")],
            }],
        };
        let resolved = convert_snippet(&snippet).unwrap();
        assert_eq!(resolved.events.len(), 1);
    }

    #[test]
    fn chord_event() {
        let snippet = OwnedSnippet {
            meta: None,
            version: None,
            clef: "treble".into(),
            events: vec![OwnedEvent::Chord {
                pitches: vec![
                    OwnedPitch::Shorthand("c4".into()),
                    OwnedPitch::Shorthand("e4".into()),
                    OwnedPitch::Shorthand("g4".into()),
                ],
                duration: OwnedDuration::Shorthand("2".into()),
            }],
        };
        let resolved = convert_snippet(&snippet).unwrap();
        assert_eq!(resolved.events.len(), 1);
    }
}
