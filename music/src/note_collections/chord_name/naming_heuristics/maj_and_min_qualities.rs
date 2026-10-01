use crate::note::pitch_class::Pc;
use crate::note::pitch_class::Pc::*;
use crate::note_collections::chord_name::naming_heuristics::alts_and_extensions::{
    generate_alt, generate_alt_and_extensions, TriadContext,
};
use crate::note_collections::chord_name::naming_heuristics::NamingHeuristic;
use crate::note_collections::chord_name::quality::chord::{
    AddMember, ChordQuality, MajorSubtype, MinorSubtype,
};
use crate::note_collections::chord_name::NamingConfig;
use std::collections::HashSet;

/// Backward-compat wrapper — equivalent to `common_prefix_with` under
/// `NamingConfig::default()`.
pub fn common_prefix(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    common_prefix_with(pcs, &NamingConfig::default())
}

/// Core major/minor dispatch. Considers `NamingConfig` flags:
/// - `prefer_add_notation` routes triads with 9/11/13 and no 7th through the
///   `Add` subtype.
/// - `distinguish_sixth_from_thirteenth` (plan Tier 2.6) controls whether
///   Pc9 over a triad, without a 7th, becomes `Maj6`/`Min6` (true) or stays
///   as a plain triad with `(13)` in the alt list (false).
pub fn common_prefix_with(pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
    let has_seventh = pcs.contains(&Pc10) || pcs.contains(&Pc11);
    // `prefer_add_notation` only fires when no 7th is present *and* at least
    // one extension (Pc2/Pc5/Pc9) is present over the triad. Otherwise fall
    // through to the baseline branches.
    let add_members_opt = if cfg.prefer_add_notation && !has_seventh {
        let mut members: Vec<AddMember> = Vec::new();
        if pcs.contains(&Pc2) {
            members.push(AddMember::Add9);
        }
        if pcs.contains(&Pc5) {
            members.push(AddMember::Add11);
        }
        if pcs.contains(&Pc9) {
            members.push(AddMember::Add6);
        }
        if members.is_empty() {
            None
        } else {
            Some(members)
        }
    } else {
        None
    };

    if pcs.contains(&Pc4) {
        if pcs.contains(&Pc11) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::MajN(ext, alt)));
        }
        if pcs.contains(&Pc10) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::N(ext, alt)));
        }
        if let Some(members) = add_members_opt {
            // Triad + extensions, no 7, add-notation opted in.
            let alt = generate_alt_add_only(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::Add(members, alt)));
        }
        if pcs.contains(&Pc9) && cfg.distinguish_sixth_from_thirteenth {
            let alt = generate_alt(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::Maj6(alt)));
        }
        // Baseline major triad. When `distinguish_sixth_from_thirteenth` is
        // off and Pc9 is present, `generate_alt_legacy_thirteenth` restores
        // the pre-Tier-1.2 behavior of emitting Thirteenth as an alt.
        let alt = if !cfg.distinguish_sixth_from_thirteenth && pcs.contains(&Pc9) && !has_seventh {
            generate_alt_legacy_thirteenth(pcs, TriadContext::Major)
        } else {
            generate_alt(pcs, TriadContext::Major)
        };
        return Some(ChordQuality::Major(MajorSubtype::Maj(alt)));
    }
    if pcs.contains(&Pc3) {
        if pcs.contains(&Pc11) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::MinMajN(ext, alt)));
        }
        if pcs.contains(&Pc10) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::MinN(ext, alt)));
        }
        if let Some(members) = add_members_opt {
            let alt = generate_alt_add_only(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::Add(members, alt)));
        }
        if pcs.contains(&Pc9) && cfg.distinguish_sixth_from_thirteenth {
            let alt = generate_alt(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::Min6(alt)));
        }
        let alt = if !cfg.distinguish_sixth_from_thirteenth && pcs.contains(&Pc9) && !has_seventh {
            generate_alt_legacy_thirteenth(pcs, TriadContext::Minor)
        } else {
            generate_alt(pcs, TriadContext::Minor)
        };
        return Some(ChordQuality::Minor(MinorSubtype::Min(alt)));
    }
    None
}

/// Variant of `generate_alt` used in the `Add` code path: strips the
/// extension pcs (Pc2/Pc5/Pc9) that have already been promoted into
/// `AddMember`s so they don't re-surface as `(9, 11, 13)` alts.
fn generate_alt_add_only(
    pcs: &HashSet<Pc>,
    ctx: TriadContext,
) -> crate::note_collections::chord_name::quality::chord::Alt {
    use crate::note_collections::chord_name::quality::chord::AltChoice;
    let mut alt = generate_alt(pcs, ctx);
    // Remove extension-derived alts that are represented by AddMember.
    alt.retain(|a| {
        !matches!(
            a,
            AltChoice::Nine | AltChoice::Eleven | AltChoice::Thirteenth
        )
    });
    alt
}

/// Legacy pre-Tier-1.2 alt generation: re-inject the Thirteenth that
/// `generate_alt` suppresses for Pc9-without-7th. Used when
/// `NamingConfig::distinguish_sixth_from_thirteenth` is `false`.
fn generate_alt_legacy_thirteenth(
    pcs: &HashSet<Pc>,
    ctx: TriadContext,
) -> crate::note_collections::chord_name::quality::chord::Alt {
    use crate::note_collections::chord_name::quality::chord::{Alt, AltChoice};
    let mut alt: Vec<AltChoice> = generate_alt(pcs, ctx).to_vec();
    if !alt.contains(&AltChoice::Thirteenth) {
        alt.push(AltChoice::Thirteenth);
    }
    Alt::from(alt)
}

/// Common Logic across all heuristics based on diminished chords.
pub fn search_for_maj_min_quality(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    search_for_maj_min_quality_with(pcs, &NamingConfig::default())
}

pub fn search_for_maj_min_quality_with(
    pcs: &HashSet<Pc>,
    cfg: &NamingConfig,
) -> Option<ChordQuality> {
    if pcs.len() == 7 {
        // Do the strategy for scale names instead
    }
    common_prefix_with(pcs, cfg)
}

/// Triads, sans 5th, with a 6th, and possibly also with a 9th.
/// Triads with only 9ths are covered in [MajOrMinN].
#[derive(Debug)]
pub struct MajOrMin69;
impl NamingHeuristic for MajOrMin69 {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3, Pc4]), HashSet::from([Pc9])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1, Pc2])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

/// When both the third and sharp ninth are in the chord.
/// We assume a P5th here. It's not tonally important in this case, and is covered elsewhere.
#[derive(Debug)]
pub struct MajSharpNine;
impl NamingHeuristic for MajSharpNine {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3]), HashSet::from([Pc4])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1]),
            HashSet::from([Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

/// This covers everything from an explicit Major or Minor Triad all the way to
/// any combination of extensions, up through to a seven note scale.
#[derive(Debug)]
pub struct MajOrMinN;
impl NamingHeuristic for MajOrMinN {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3, Pc4]), HashSet::from([Pc7])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5, Pc6]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

/// When both the third and sharp ninth are in the chord.
/// We assume a P5th here. It's not tonally important in this case, and is covered elsewhere.
#[derive(Debug)]
pub struct MajNSharpNine;
impl NamingHeuristic for MajNSharpNine {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3]),
            HashSet::from([Pc4]),
            HashSet::from([Pc7]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1]),
            HashSet::from([Pc5, Pc6]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

/// Chord shell (no P5) with a 7th over a major 3rd. Used primarily for
/// add-notation variants; the `show_omissions` path (Tier 2.3) may pick this
/// up when a P5 is absent.
#[derive(Debug)]
pub struct MajChordShell;
impl NamingHeuristic for MajChordShell {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc4]), HashSet::from([Pc10, Pc11])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc9]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        maj_min_with_possible_no_fifth(pcs, cfg, TriadContext::Major)
    }
}

#[derive(Debug)]
pub struct MinChordShell;
impl NamingHeuristic for MinChordShell {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3]), HashSet::from([Pc10, Pc11])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc8, Pc9]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        maj_min_with_possible_no_fifth(pcs, cfg, TriadContext::Minor)
    }
}

/// When the P5 is absent but the set matches a maj/min + 7 shell, route
/// through the config-aware `show_omissions` path. Otherwise fall back to the
/// default common_prefix handling.
fn maj_min_with_possible_no_fifth(
    pcs: &HashSet<Pc>,
    cfg: &NamingConfig,
    ctx: TriadContext,
) -> Option<ChordQuality> {
    if cfg.show_omissions && !pcs.contains(&Pc7) {
        let (alt, ext) = generate_alt_and_extensions(pcs, ctx.clone());
        return Some(match ctx {
            TriadContext::Major => {
                // Decide dominant vs. major by which seventh is present.
                if pcs.contains(&Pc11) {
                    ChordQuality::Major(MajorSubtype::NoFifth(ext, alt))
                } else {
                    ChordQuality::Major(MajorSubtype::DomNoFifth(ext, alt))
                }
            }
            TriadContext::Minor => ChordQuality::Minor(MinorSubtype::NoFifth(ext, alt)),
            _ => unreachable!(),
        });
    }
    common_prefix_with(pcs, cfg)
}

#[derive(Debug)]
pub struct RootToThirdCluster;
impl NamingHeuristic for RootToThirdCluster {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1, Pc2]), HashSet::from([Pc3, Pc4])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

#[derive(Debug)]
pub struct ThirdAndFourth;
impl NamingHeuristic for ThirdAndFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3, Pc4]), HashSet::from([Pc5])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}

#[derive(Debug)]
pub struct ThirdAndSharpFourth;
impl NamingHeuristic for ThirdAndSharpFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc4]), HashSet::from([Pc6])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
    fn generate_name_with(&self, pcs: &HashSet<Pc>, cfg: &NamingConfig) -> Option<ChordQuality> {
        search_for_maj_min_quality_with(pcs, cfg)
    }
}
