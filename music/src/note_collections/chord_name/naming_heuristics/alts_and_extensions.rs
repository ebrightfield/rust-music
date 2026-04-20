use std::collections::HashSet;
use crate::note_collections::chord_name::quality::chord::{Alt, AltChoice, Extension};
use crate::note::pitch_class::Pc;

/// This controls how we search for potential chord alterations,
/// as the presence of some notes in certain contexts is an alteration,
/// but in other contexts is not.
#[derive(Debug, Clone, PartialEq)]
pub enum TriadContext {
    Major,
    Minor,
    Aug,
    Dim,
    Sus,
}

/// Generate an [Alt] to describe what alterations should be added to a chord name.
pub fn generate_alt(pcs: &HashSet<Pc>, triad_context: TriadContext) -> Alt {
    let mut alterations = vec![];
    // Based on some starting values and a [TriadContext],
    // we can modify the [possible_alts] local to something tailored to each context.
    //
    // Each `usize` keys into `AltChoice::try_from(usize)`. Omissions below are
    // intentional because the corresponding pitch-class is already a defining
    // member of the triad for that context, and re-emitting it as an alteration
    // would duplicate a structural tone.
    let possible_alts: Vec<usize> = match triad_context {
        // Major triad: Pc4 (M3) and Pc7 (P5) are structural.
        //   4 → FlatEleven (Pc4) is the M3 itself; `CMaj7♭11` is not a real chord.
        //  10 → SharpThirteenth (Pc10) is the ♭7 — emitted as the chord's own
        //       7th via [Extension::Seventh] instead of as an alt.
        TriadContext::Major => vec![1,2,3,5,6,8,9],
        // Minor triad: Pc3 (m3) and Pc7 (P5) are structural.
        //   3 → SharpNine (Pc3) is the m3 itself; already asserted by the
        //       "min" in the chord name.
        //  10 → SharpThirteenth (Pc10) is the ♭7 — emitted via Extension.
        TriadContext::Minor => vec![1,2,4,5,6,8,9],
        // Augmented triad: Pc4 (M3) and Pc8 (#5) are structural.
        //   4 → FlatEleven (Pc4) is the M3.
        //   8 → FlatThirteenth (Pc8) is the #5 — already in the "+" label.
        //  10 → SharpThirteenth (Pc10) is the ♭7 — emitted via Extension.
        TriadContext::Aug => vec![1,2,3,5,6,9],
        // Diminished triad: Pc3 (m3) and Pc6 (♭5) are structural.
        //   3 → SharpNine (Pc3) is the m3.
        //   6 → SharpEleven (Pc6) is the ♭5 — already in the "dim" label.
        //   9 → Thirteenth (Pc9) is the dim7 — emitted via Extension.
        //  10 → SharpThirteenth (Pc10) is the ♭7 — emitted via Extension.
        TriadContext::Dim => vec![1,2,4,5,8],
        // Sus chord: Pc2 or Pc5 is the suspension; Pc7 is structural.
        //   2 → Nine (Pc2) and 5 → Eleven (Pc5) are the sus member.
        //   3 → SharpNine (Pc3) clashes with the suspension; treat in own path.
        //   4 → FlatEleven (Pc4) would sound the 3rd we're suspending.
        //  10 → SharpThirteenth (Pc10) is the ♭7 — emitted via Extension.
        TriadContext::Sus => vec![1,6,8,9]
    };
    for alt_num in possible_alts {
        // We can use unwraps in this block because we only use hardcoded numbers that we
        // know are going to be valid for the type conversions.
        let alt_as_u8 = u8::try_from(alt_num).unwrap();
        if pcs.contains(&Pc::from(&alt_as_u8)) {
            alterations.push(AltChoice::try_from(alt_num).unwrap())
        }
    }

    // Tier 1.1: ♭5 vs ♯11 disambiguation.
    //
    // Over a Major or Minor triad, Pc6 is emitted as SharpEleven by default —
    // but that's only correct when the natural P5 (Pc7) is *also* present.
    // When Pc7 is absent, Pc6 is functioning as ♭5, not ♯11: the chord has no
    // "natural 5 + altered 11" reading because the 5 isn't there to be
    // altered past. Swap the SharpEleven for FlatFive in that case.
    //
    // This does not apply to Dim (Pc6 is the ♭5 structurally and is not
    // emitted from the Dim table at all) or Aug (Pc6 is not in the Aug table).
    // Sus chords with Pc6 remain as SharpEleven for now; Pc5 (the suspension)
    // fills the same "fifth slot" role there and the case is rarer.
    let wants_flat_five = matches!(triad_context, TriadContext::Major | TriadContext::Minor)
        && pcs.contains(&Pc::Pc6)
        && !pcs.contains(&Pc::Pc7);
    if wants_flat_five {
        for alt in alterations.iter_mut() {
            if *alt == AltChoice::SharpEleven {
                *alt = AltChoice::FlatFive;
            }
        }
    }

    // Tier 1.2: suppress Thirteenth when no 7th is present.
    //
    // A chord like {Pc0, Pc4, Pc7, Pc9} is a 6-chord, not a 13-chord: the
    // "13" label implies a stacked-thirds reading, which requires a 7th for
    // the extension to be meaningful. Without Pc10 or Pc11, the 6 is the top
    // tone and the chord is rendered via the Maj6/Min6 subtype (which holds
    // only the residual Alt). Drop the Thirteenth so the subtype renders as
    // a plain 6-chord instead of `Maj (13)`.
    //
    // Dim and Aug contexts don't emit Thirteenth from their tables anyway
    // (Dim reserves Pc9 for the diminished 7th; Aug drops Pc9 because #5 is
    // structural). Sus context similarly keeps Pc9 for a 6/9-sus reading.
    let has_seventh = pcs.contains(&Pc::Pc10) || pcs.contains(&Pc::Pc11);
    let is_maj_min = matches!(triad_context, TriadContext::Major | TriadContext::Minor);
    if is_maj_min && !has_seventh {
        alterations.retain(|a| *a != AltChoice::Thirteenth);
    }

    alterations.into()
}

/// Generate an [Alt] to describe what alterations should be added to a chord name.
/// Also generate a `Vec<Extension>`.
/// Used for "xxxN" qualities, i.e. Maj7, dom7, min7, min7b5, etc.
pub fn generate_alt_and_extensions(pcs: &HashSet<Pc>, triad_context: TriadContext) -> (Alt, Vec<Extension>) {
    let mut extensions = vec![Extension::Seventh];
    let mut alts = generate_alt(pcs, triad_context.clone());
    // Generate extensions
    if alts.contains(&AltChoice::Nine) {
        alts.retain(|x| *x != AltChoice::Nine);
        extensions.push(Extension::Ninth);
    }
    if alts.contains(&AltChoice::Eleven) {
        alts.retain(|x| *x != AltChoice::Eleven);
        // 4ths/11ths on Sus chords are the Sus part!
        if triad_context != TriadContext::Sus {
            extensions.push(Extension::Eleventh);
        }
    }
    if alts.contains(&AltChoice::Thirteenth) {
        alts.retain(|x| *x != AltChoice::Thirteenth);
        // 6ths/13ths on a diminished chord are diminished 7ths.
        if triad_context != TriadContext::Dim {
            extensions.push(Extension::Thirteenth);
        }
    }
    (alts, extensions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    /// For each context, every `AltChoice` listed in `possible_alts` must be
    /// reachable from some pc-set. This locks in the table as an intentional
    /// whitelist: adding a new `AltChoice` without updating the tables will
    /// silently drop it, and this test prevents that.
    #[test]
    fn each_context_can_emit_every_listed_alt() {
        // (pc used to trigger, expected AltChoice)
        let cases = [
            (Pc1, AltChoice::FlatNine),
            (Pc2, AltChoice::Nine),
            (Pc3, AltChoice::SharpNine),
            (Pc4, AltChoice::FlatEleven),
            (Pc5, AltChoice::Eleven),
            (Pc6, AltChoice::SharpEleven),
            (Pc8, AltChoice::FlatThirteenth),
            (Pc9, AltChoice::Thirteenth),
            (Pc10, AltChoice::SharpThirteenth),
        ];

        let check = |ctx: TriadContext, table: &[usize]| {
            for (pc, expected) in cases.iter() {
                let trigger_num = u8::from(pc) as usize;
                let present = table.contains(&trigger_num);
                // When testing Pc6 on Major/Minor in isolation, the Tier 1.1
                // swap kicks in and SharpEleven becomes FlatFive. Provide a
                // natural P5 so we isolate the base table behavior.
                // When testing Pc9 on Major/Minor in isolation, Tier 1.2
                // suppresses Thirteenth in the absence of a 7th. Provide Pc10
                // (the ♭7) so the base-table emission survives.
                let mut triggers = vec![Pc0, *pc];
                if *pc == Pc6
                    && matches!(ctx, TriadContext::Major | TriadContext::Minor)
                {
                    triggers.push(Pc7);
                }
                if *pc == Pc9
                    && matches!(ctx, TriadContext::Major | TriadContext::Minor)
                {
                    triggers.push(Pc10);
                }
                let pcs: HashSet<Pc> = triggers.iter().copied().collect();
                let alt = generate_alt(&pcs, ctx.clone());
                if present {
                    assert!(
                        alt.contains(expected),
                        "{:?} table lists {} but did not emit {:?} (alt = {:?})",
                        ctx, trigger_num, expected, alt,
                    );
                } else {
                    assert!(
                        !alt.contains(expected),
                        "{:?} table omits {} but still emitted {:?}",
                        ctx, trigger_num, expected,
                    );
                }
            }
        };

        check(TriadContext::Major, &[1,2,3,5,6,8,9]);
        check(TriadContext::Minor, &[1,2,4,5,6,8,9]);
        check(TriadContext::Aug,   &[1,2,3,5,6,9]);
        check(TriadContext::Dim,   &[1,2,4,5,8]);
        check(TriadContext::Sus,   &[1,6,8,9]);
    }
}
