//! Tier 1.4 — altered dominant (`7alt`) heuristic.
//!
//! The "altered" label is reserved for dominant chords that alter the 5th
//! and 9th in both directions — a rare but iconic jazz sonority whose
//! canonical voicing is the altered scale `{0, 1, 3, 4, 6, 8, 10}`.
//!
//! Trigger rule (per `docs/chord-naming-correction-plan.md` Tier 1.4):
//!   - Pc4 (M3) and Pc10 (♭7) must be present.
//!   - At least one alteration from the 9-axis: `{Pc1, Pc3}`.
//!   - At least one alteration from the 5-axis: `{Pc6, Pc8}`.
//!   - Total alteration count (across both axes) ≥ 3.
//!   - Pc7 must NOT be present — a natural P5 disqualifies the altered
//!     reading (those chords are labeled specifically, e.g. `7♭9♯9`).
//!
//! Chords with fewer alterations, or alterations confined to a single axis,
//! fall through to other heuristics and produce specific labels.

use std::collections::HashSet;

use crate::note::pitch_class::Pc;
use crate::note::pitch_class::Pc::*;
use crate::note_collections::chord_name::naming_heuristics::NamingHeuristic;
use crate::note_collections::chord_name::quality::chord::{ChordQuality, MajorSubtype};

const NINE_AXIS: &[Pc] = &[Pc1, Pc3];
const FIVE_AXIS: &[Pc] = &[Pc6, Pc8];

#[derive(Debug)]
pub struct AlteredDominant;

impl NamingHeuristic for AlteredDominant {
    type T = ChordQuality;

    // Custom `validate` — the default required/optional cover check is too
    // permissive for the multi-axis count rule. We also disallow Pc7.
    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        if !pcs.contains(&Pc4) || !pcs.contains(&Pc10) {
            return false;
        }
        if pcs.contains(&Pc7) {
            return false;
        }

        let nine_axis_count = NINE_AXIS.iter().filter(|p| pcs.contains(p)).count();
        let five_axis_count = FIVE_AXIS.iter().filter(|p| pcs.contains(p)).count();
        let total = nine_axis_count + five_axis_count;

        if nine_axis_count == 0 || five_axis_count == 0 {
            return false;
        }
        if total < 3 {
            return false;
        }

        // Reject stray pcs that aren't part of the altered collection. The
        // altered dominant is built from exactly the following pcs (plus the
        // implicit root Pc0): Pc1, Pc3, Pc4, Pc6, Pc8, Pc10.
        let allowed: HashSet<Pc> = [Pc0, Pc1, Pc3, Pc4, Pc6, Pc8, Pc10]
            .iter()
            .copied()
            .collect();
        pcs.iter().all(|pc| allowed.contains(pc))
    }

    fn generate_name(&self, _pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(ChordQuality::Major(MajorSubtype::DomAlt))
    }
}
