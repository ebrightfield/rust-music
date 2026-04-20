use std::collections::HashSet;
use crate::note_collections::chord_name::NamingConfig;
use crate::note_collections::chord_name::quality::chord::ChordQuality;
use crate::note_collections::chord_name::quality::scale::ScaleQuality;
use crate::note_collections::interval_class::IntervalClass;
use crate::note::pitch_class::Pc;
use crate::note::pitch_class::Pc::*;

pub mod maj_and_min_qualities;
pub mod alts_and_extensions;
pub mod altered_dom;
pub mod aug_qualities;
pub mod dim_qualities;
pub mod sus_qualities;
pub mod inferred_third_qualities;
pub mod scale_qualities;

/// A Chord Naming Heuristic contains two sets:
/// - Required Pcs -- Vec of subsets of Pcs, the input must contain only one element in each subset.
/// - Optional Pcs -- Vec of subsets of Pcs, the "only one" requirement likewise applies.
///
/// In order to "match" a naming heuristic's requirements, all elements of `pcs` must match,
/// and all `HashSet`s in `self.required` should intersect on only one element of `pcs`.
///
/// Many naming heuristics are built, which can then be iterated over. When a call to
/// [NamingHeuristic::validate] returns true, we can then call [NamingHeuristic::generate_name].
///
/// It is not required that a heuristic generate a name.
pub trait NamingHeuristic: std::fmt::Debug {
    /// For our purposes, either a [ChordQuality] or a [ScaleQuality].
    /// In principle, one could build their own naming system and put anything here,
    /// even a simple string.
    type T;

    /// We want to our chord in question to have only _one_ element in common with each `HashSet`.
    /// This property must hold true for each element.
    fn required(&self) -> Vec<HashSet<Pc>> { vec! [] }
    /// We want to our chord in question to have only _one_ element in common with each `HashSet`.
    /// These properties are optional, all or none of them could match.
    fn optional(&self) -> Vec<HashSet<Pc>> { vec! [] }

    /// One-shot execution of an attempt at applying this heuristic to naming a chord.
    /// If the heuristic simply doesn't apply, it returns `None`.
    /// Likewise, `self.generate_name` can sometimes return `None`.
    fn apply(&self, pcs: &HashSet<Pc>) -> Option<Self::T> {
        if self.validate(pcs) {
            return self.generate_name(pcs);
        }
        None
    }

    /// Try to generate a name based on the content of `pcs`.
    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<Self::T>;

    /// Config-sensitive variant of [`Self::generate_name`]. The default
    /// implementation ignores the config and delegates to `generate_name`.
    /// Heuristics that need access to config (e.g. for `prefer_add_notation`
    /// or `show_omissions`) override this. Inference dispatch calls this
    /// variant, not `generate_name`, so heuristic impls must keep the two in
    /// sync if they override one of them.
    fn generate_name_with(&self, pcs: &HashSet<Pc>, _cfg: &NamingConfig) -> Option<Self::T> {
        self.generate_name(pcs)
    }

    /// Does a given `Vec<Pc>` satisfy the following:
    /// 1. All intersections with required `HashSet`s have only one element.
    /// 2. All elements in `pcs` are matched,
    ///    whether through required or optional `HashSet` intersections.
    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        // Entry points (`infer_chord_quality`, `infer_scale_quality`) guarantee
        // Pc0 is present. Heuristic `required`/`optional` sets never mention
        // Pc0, so we strip it here so the total-cover check
        // (`matched.len() == pcs.len()`) succeeds.
        let mut pcs = pcs.clone();
        pcs.remove(&Pc0);
        let mut matched = vec![];
        for subset in self.required().iter() {
            let intersection: Vec<Pc> = subset
                .intersection(&pcs)
                .map(|pc| pc.clone())
                .collect();
            if intersection.len() == 1 {
                matched.extend(intersection);
            } else {
                // Failed a subset requirement
                return false;
            }
        }
        for subset in self.optional().iter() {
            let intersection: Vec<Pc> = subset
                .intersection(&pcs)
                .map(|pc| pc.clone())
                .collect();
            if intersection.len() == 1 {
                matched.extend(intersection);
            }
        }
        matched.len() == pcs.len()
    }
}

/// A naming heuristic that produces a [ChordQuality].
type ChordHeuristic = Box<dyn NamingHeuristic<T=ChordQuality>>;

/// Identifier for which chord naming heuristic matched an input.
///
/// **Order is semantically load-bearing.** The variants are listed in the
/// same precedence as `chord_heuristics()`. The ordered-dispatch test
/// (below, `chord_heuristic_kind_order_matches_dispatch`) pins the order at
/// compile time — reordering the variants without updating
/// `chord_heuristics` and the dispatch table produces wrong chord names.
///
/// Sentinel variants (`DegenerateSingleton`, `DegenerateInterval`) never
/// appear in ordered dispatch; they're surfaced by the fast-path in
/// [`infer_chord_quality_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordHeuristicKind {
    // Fast-path sentinels
    DegenerateSingleton,
    DegenerateInterval,
    // Ordered dispatch — do NOT reorder without updating chord_heuristics()
    // AND the dispatch test.
    AlteredDominant,
    MajOrMin69,
    MajSharpNine,
    MajOrMinN,
    MajNSharpNine,
    MajChordShell,
    MinChordShell,
    RootToThirdCluster,
    ThirdAndFourth,
    ThirdAndSharpFourth,
    FifthAndUpperNotes,
    NinthAndSixthNoThird,
    TritoneAndSeventh,
    NinthAndSeventh,
    AugChordQualities,
    DimNChords,
    NotMin6Chord,
    TritoneAndDimSeventh,
    SusNChords,
    BothSecondAndFourth,
    Altered13Sus,
    FourthAndSeventh,
    FlatSecondAndFourth,
}

impl ChordHeuristicKind {
    /// Ordered dispatch list — matches the order returned by
    /// [`chord_heuristics()`] exactly. The `DegenerateSingleton` /
    /// `DegenerateInterval` sentinels are omitted because they bypass
    /// ordered dispatch.
    pub const ORDERED: &'static [ChordHeuristicKind] = &[
        ChordHeuristicKind::AlteredDominant,
        ChordHeuristicKind::MajOrMin69,
        ChordHeuristicKind::MajSharpNine,
        ChordHeuristicKind::MajOrMinN,
        ChordHeuristicKind::MajNSharpNine,
        ChordHeuristicKind::MajChordShell,
        ChordHeuristicKind::MinChordShell,
        ChordHeuristicKind::RootToThirdCluster,
        ChordHeuristicKind::ThirdAndFourth,
        ChordHeuristicKind::ThirdAndSharpFourth,
        ChordHeuristicKind::FifthAndUpperNotes,
        ChordHeuristicKind::NinthAndSixthNoThird,
        ChordHeuristicKind::TritoneAndSeventh,
        ChordHeuristicKind::NinthAndSeventh,
        ChordHeuristicKind::AugChordQualities,
        ChordHeuristicKind::DimNChords,
        ChordHeuristicKind::NotMin6Chord,
        ChordHeuristicKind::TritoneAndDimSeventh,
        ChordHeuristicKind::SusNChords,
        ChordHeuristicKind::BothSecondAndFourth,
        ChordHeuristicKind::Altered13Sus,
        ChordHeuristicKind::FourthAndSeventh,
        ChordHeuristicKind::FlatSecondAndFourth,
    ];

    /// A debug-friendly human-readable label for the heuristic. Used by the
    /// combinatoric snapshot dump (§7) to flag which heuristic fired for
    /// each input set.
    pub fn label(&self) -> &'static str {
        match self {
            ChordHeuristicKind::DegenerateSingleton => "DegenerateSingleton",
            ChordHeuristicKind::DegenerateInterval => "DegenerateInterval",
            ChordHeuristicKind::AlteredDominant => "AlteredDominant",
            ChordHeuristicKind::MajOrMin69 => "MajOrMin69",
            ChordHeuristicKind::MajSharpNine => "MajSharpNine",
            ChordHeuristicKind::MajOrMinN => "MajOrMinN",
            ChordHeuristicKind::MajNSharpNine => "MajNSharpNine",
            ChordHeuristicKind::MajChordShell => "MajChordShell",
            ChordHeuristicKind::MinChordShell => "MinChordShell",
            ChordHeuristicKind::RootToThirdCluster => "RootToThirdCluster",
            ChordHeuristicKind::ThirdAndFourth => "ThirdAndFourth",
            ChordHeuristicKind::ThirdAndSharpFourth => "ThirdAndSharpFourth",
            ChordHeuristicKind::FifthAndUpperNotes => "FifthAndUpperNotes",
            ChordHeuristicKind::NinthAndSixthNoThird => "NinthAndSixthNoThird",
            ChordHeuristicKind::TritoneAndSeventh => "TritoneAndSeventh",
            ChordHeuristicKind::NinthAndSeventh => "NinthAndSeventh",
            ChordHeuristicKind::AugChordQualities => "AugChordQualities",
            ChordHeuristicKind::DimNChords => "DimNChords",
            ChordHeuristicKind::NotMin6Chord => "NotMin6Chord",
            ChordHeuristicKind::TritoneAndDimSeventh => "TritoneAndDimSeventh",
            ChordHeuristicKind::SusNChords => "SusNChords",
            ChordHeuristicKind::BothSecondAndFourth => "BothSecondAndFourth",
            ChordHeuristicKind::Altered13Sus => "Altered13Sus",
            ChordHeuristicKind::FourthAndSeventh => "FourthAndSeventh",
            ChordHeuristicKind::FlatSecondAndFourth => "FlatSecondAndFourth",
        }
    }

    /// Build the corresponding boxed heuristic. Used by the trait-object
    /// entry point so callers can still use the old return shape while the
    /// enum handles identification.
    fn to_heuristic(&self) -> ChordHeuristic {
        match self {
            ChordHeuristicKind::DegenerateSingleton => Box::new(DegenerateSingleton),
            ChordHeuristicKind::DegenerateInterval => Box::new(DegenerateInterval),
            ChordHeuristicKind::AlteredDominant => Box::new(altered_dom::AlteredDominant),
            ChordHeuristicKind::MajOrMin69 => Box::new(maj_and_min_qualities::MajOrMin69),
            ChordHeuristicKind::MajSharpNine => Box::new(maj_and_min_qualities::MajSharpNine),
            ChordHeuristicKind::MajOrMinN => Box::new(maj_and_min_qualities::MajOrMinN),
            ChordHeuristicKind::MajNSharpNine => Box::new(maj_and_min_qualities::MajNSharpNine),
            ChordHeuristicKind::MajChordShell => Box::new(maj_and_min_qualities::MajChordShell),
            ChordHeuristicKind::MinChordShell => Box::new(maj_and_min_qualities::MinChordShell),
            ChordHeuristicKind::RootToThirdCluster => Box::new(maj_and_min_qualities::RootToThirdCluster),
            ChordHeuristicKind::ThirdAndFourth => Box::new(maj_and_min_qualities::ThirdAndFourth),
            ChordHeuristicKind::ThirdAndSharpFourth => Box::new(maj_and_min_qualities::ThirdAndSharpFourth),
            ChordHeuristicKind::FifthAndUpperNotes => Box::new(inferred_third_qualities::FifthAndUpperNotes),
            ChordHeuristicKind::NinthAndSixthNoThird => Box::new(inferred_third_qualities::NinthAndSixthNoThird),
            ChordHeuristicKind::TritoneAndSeventh => Box::new(inferred_third_qualities::TritoneAndSeventh),
            ChordHeuristicKind::NinthAndSeventh => Box::new(inferred_third_qualities::NinthAndSeventh),
            ChordHeuristicKind::AugChordQualities => Box::new(aug_qualities::AugChordQualities),
            ChordHeuristicKind::DimNChords => Box::new(dim_qualities::DimNChords),
            ChordHeuristicKind::NotMin6Chord => Box::new(dim_qualities::NotMin6Chord),
            ChordHeuristicKind::TritoneAndDimSeventh => Box::new(dim_qualities::TritoneAndDimSeventh),
            ChordHeuristicKind::SusNChords => Box::new(sus_qualities::SusNChords),
            ChordHeuristicKind::BothSecondAndFourth => Box::new(sus_qualities::BothSecondAndFourth),
            ChordHeuristicKind::Altered13Sus => Box::new(sus_qualities::Altered13Sus),
            ChordHeuristicKind::FourthAndSeventh => Box::new(sus_qualities::FourthAndSeventh),
            ChordHeuristicKind::FlatSecondAndFourth => Box::new(sus_qualities::FlatSecondAndFourth),
        }
    }
}

/// An order-sensitive list of all the various naming heuristics.
/// The first heuristic to match on the content is applied to generating a name.
///
/// Kept around as the trait-object surface. The ordered dispatch list is
/// defined in [`ChordHeuristicKind::ORDERED`]; this function instantiates
/// the same sequence as boxed trait objects.
pub fn chord_heuristics() -> Vec<ChordHeuristic> {
    ChordHeuristicKind::ORDERED
        .iter()
        .map(|k| k.to_heuristic())
        .collect()
}

/// Infer a [ChordQuality] from a `HashSet<Pc>` using [`NamingConfig::default`].
///
/// Convenience wrapper around [`infer_chord_quality_with`]. See that function
/// for the detailed contract.
pub fn infer_chord_quality(pcs: &HashSet<Pc>) -> Option<(ChordHeuristic, Option<ChordQuality>)> {
    infer_chord_quality_with(pcs, &NamingConfig::default())
}

/// Infer a [ChordQuality] from a `HashSet<Pc>` under a [`NamingConfig`].
///
/// Pc0 is treated as implicit: inputs without Pc0 are semantically equivalent
/// to the same set with Pc0 inserted. The root is always Pc0; a set like
/// `{Pc3, Pc7}` names a minor triad with an implicit root.
///
/// Fast-paths for degenerate cases (after Pc0 insertion):
/// - `{Pc0}` (singleton) returns `ChordQuality::SingleNote`.
/// - Two-pc sets return `ChordQuality::Interval(_)` naming the interval above
///   Pc0. This covers `{Pc0, Pc7}` → `P5` (power chord), etc.
///
/// Three-or-more pc sets dispatch to the heuristic list in precedence order.
/// Returns `None` only when no heuristic matches.
///
/// The [`NamingConfig`] controls config-sensitive choices such as:
/// - `prefer_add_notation`: route 9/11/13 over a triad (no 7th) to an `add`
///   subtype instead of an `(N)` alt.
/// - `show_omissions`: record missing 3rd/5th when a triad quality was
///   inferred despite the defining tone being absent.
/// - `distinguish_sixth_from_thirteenth`: when `false`, the plan-era
///   "Thirteenth without 7th" behavior is preserved for regression compat.
pub fn infer_chord_quality_with(
    pcs: &HashSet<Pc>,
    cfg: &NamingConfig,
) -> Option<(ChordHeuristic, Option<ChordQuality>)> {
    // Normalize Pc0 at the entry point. Any input is treated as if Pc0 were
    // present; this gives all downstream heuristics a uniform assumption that
    // the root is in the set.
    let mut pcs = pcs.clone();
    pcs.insert(Pc0);

    // Fast-path for degenerate cases (size measured post-normalization).
    if pcs.len() == 1 {
        return Some((
            Box::new(DegenerateSingleton),
            Some(ChordQuality::SingleNote),
        ));
    }
    if pcs.len() == 2 {
        // Pick the non-Pc0 member and render its interval class.
        let other = pcs
            .iter()
            .find(|pc| **pc != Pc0)
            .copied()
            .expect("size-2 set with Pc0 must contain a second pc");
        let ic = IntervalClass::from(&u8::from(&other));
        return Some((
            Box::new(DegenerateInterval),
            Some(ChordQuality::Interval(ic)),
        ));
    }

    for heuristic in chord_heuristics() {
        if heuristic.validate(&pcs) {
            let name = heuristic.generate_name_with(&pcs, cfg);
            return Some((heuristic, name));
        }
    }
    None
}

/// Inference variant that returns the [`ChordHeuristicKind`] that fired
/// alongside the quality. Useful for snapshot diffing and debugging.
/// Same Pc0 normalization and fast-path as [`infer_chord_quality_with`].
pub fn infer_chord_quality_kind(
    pcs: &HashSet<Pc>,
    cfg: &NamingConfig,
) -> Option<(ChordHeuristicKind, Option<ChordQuality>)> {
    let mut pcs = pcs.clone();
    pcs.insert(Pc0);

    if pcs.len() == 1 {
        return Some((ChordHeuristicKind::DegenerateSingleton, Some(ChordQuality::SingleNote)));
    }
    if pcs.len() == 2 {
        let other = pcs.iter().find(|pc| **pc != Pc0).copied().unwrap();
        let ic = IntervalClass::from(&u8::from(&other));
        return Some((ChordHeuristicKind::DegenerateInterval, Some(ChordQuality::Interval(ic))));
    }

    for kind in ChordHeuristicKind::ORDERED {
        let heuristic = kind.to_heuristic();
        if heuristic.validate(&pcs) {
            let name = heuristic.generate_name_with(&pcs, cfg);
            return Some((*kind, name));
        }
    }
    None
}

/// Detailed inference entry point that also returns the list of quality
/// ambiguities detected for the input pcset. See [`QualityAmbiguity`] for the
/// reportable kinds.
///
/// When `cfg.report_ambiguities` is `false`, the returned `Vec` is always
/// empty — callers interested in ambiguity data must opt in through the
/// config. When the config opts in, the detector runs unconditionally; it
/// does not gate on whether a chord quality was inferred successfully.
pub fn infer_chord_quality_detailed(
    pcs: &HashSet<Pc>,
    cfg: &NamingConfig,
) -> (Option<ChordQuality>, Vec<crate::note_collections::chord_name::quality::chord::QualityAmbiguity>) {
    let quality = infer_chord_quality_with(pcs, cfg).and_then(|(_, q)| q);
    if !cfg.report_ambiguities {
        return (quality, vec![]);
    }
    let mut pcs_norm = pcs.clone();
    pcs_norm.insert(Pc0);
    let intervals: Vec<u8> = pcs_norm.iter().map(|pc| u8::from(pc)).collect();
    let mut ambiguities =
        crate::note_collections::chord_name::quality::chord::QualityAmbiguity::find_duplicate_degrees(&intervals);
    // SixthVsThirteenth — Pc9 present, Pc10/Pc11 may or may not be.
    if pcs_norm.contains(&Pc9) {
        let has_seventh = pcs_norm.contains(&Pc10) || pcs_norm.contains(&Pc11);
        ambiguities.push(
            crate::note_collections::chord_name::quality::chord::QualityAmbiguity::SixthVsThirteenth { has_seventh },
        );
    }
    (quality, ambiguities)
}

/// Sentinel heuristic surfaced when the fast-path matches a singleton set.
/// Never participates in ordered dispatch; its `validate` always returns
/// `false`.
#[derive(Debug)]
pub struct DegenerateSingleton;
impl NamingHeuristic for DegenerateSingleton {
    type T = ChordQuality;
    fn validate(&self, _pcs: &HashSet<Pc>) -> bool { false }
    fn generate_name(&self, _pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(ChordQuality::SingleNote)
    }
}

/// Sentinel heuristic surfaced when the fast-path matches a two-pc set.
/// Never participates in ordered dispatch; its `validate` always returns
/// `false`.
#[derive(Debug)]
pub struct DegenerateInterval;
impl NamingHeuristic for DegenerateInterval {
    type T = ChordQuality;
    fn validate(&self, _pcs: &HashSet<Pc>) -> bool { false }
    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        let other = pcs.iter().find(|pc| **pc != Pc0).copied()?;
        let ic = IntervalClass::from(&u8::from(&other));
        Some(ChordQuality::Interval(ic))
    }
}

/// A naming heuristic that produces a [ScaleQuality].
type ScaleHeuristic = Box<dyn NamingHeuristic<T=ScaleQuality>>;

pub fn scale_heuristics() -> Vec<ScaleHeuristic> {
    // Order matters here! The first match will be dispatched to name generation.
    vec![
        // Literal equivalence checks
        Box::new(scale_qualities::WholetoneScale),
        Box::new(scale_qualities::AugAHScale),
        Box::new(scale_qualities::AugHAScale),
        Box::new(scale_qualities::DimHWScale),
        Box::new(scale_qualities::DimWHScale),
        Box::new(scale_qualities::HarmonicMinor),
        Box::new(scale_qualities::HarmonicMajor),
        Box::new(scale_qualities::AlteredScale),
        // Pentatonic scales (check before 7-note modes, as they're exact matches)
        Box::new(scale_qualities::MajorPentatonic),
        Box::new(scale_qualities::MinorPentatonic),
        Box::new(scale_qualities::BluesMajor),
        Box::new(scale_qualities::BluesMinor),
        // Scales with possible alterations
        Box::new(scale_qualities::MajorScale),
        Box::new(scale_qualities::IonianAug),
        Box::new(scale_qualities::Dorian),
        Box::new(scale_qualities::Phrygian),
        Box::new(scale_qualities::Lydian),
        Box::new(scale_qualities::LydianAug),
        Box::new(scale_qualities::Mixolydian),
        Box::new(scale_qualities::MixolydianAug),
        Box::new(scale_qualities::NaturalMinor),
        Box::new(scale_qualities::MelodicMinor),
        Box::new(scale_qualities::Locrian),
    ]
}

/// Infer a [ScaleQuality] from a `HashSet<Pc>`.
///
/// The nested `Option` in the return type mirrors
/// [`infer_chord_quality`]: the outer `Option` distinguishes "no heuristic
/// matched" from "a heuristic matched but produced no name"; the inner
/// `Option` is what `NamingHeuristic::generate_name` returns.
///
/// NOTE on Pc0: scale heuristics in [scale_qualities] override `validate` with
/// hardcoded equality checks against Pc0-absent reference sets. To preserve
/// compatibility with those overrides, this entry point strips Pc0 rather
/// than inserting it. (Chord heuristics rely on the default `validate` which
/// already strips Pc0 — see [`infer_chord_quality`].)
pub fn infer_scale_quality(pcs: &HashSet<Pc>) -> Option<(ScaleHeuristic, Option<ScaleQuality>)> {
    let mut pcs = pcs.clone();
    pcs.remove(&Pc0);
    for heuristic in scale_heuristics() {
        if heuristic.validate(&pcs) {
            let name = heuristic.generate_name(&pcs);
            return Some((heuristic, name));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::note_collections::chord_name::quality::chord::{Alt, Extension, MajorSubtype};
    use crate::note_collections::PcSet;
    use super::*;

    /// Lock the enum-dispatch order to the trait-object list. If
    /// `chord_heuristics()` and `ChordHeuristicKind::ORDERED` ever drift out
    /// of sync, this test fails, catching the order regression before any
    /// snapshot diff does.
    #[test]
    fn chord_heuristic_kind_order_matches_dispatch() {
        let heuristics = chord_heuristics();
        assert_eq!(
            heuristics.len(),
            ChordHeuristicKind::ORDERED.len(),
            "chord_heuristics() and ChordHeuristicKind::ORDERED must agree on length"
        );
        for (i, (k, h)) in ChordHeuristicKind::ORDERED.iter().zip(heuristics.iter()).enumerate() {
            // Debug format of the struct always starts with its type name.
            let debug_label = format!("{:?}", h);
            assert!(
                debug_label.starts_with(k.label()),
                "position {}: ORDERED has {:?} ({:?}) but chord_heuristics() produced {:?}",
                i, k, k.label(), debug_label,
            );
        }
    }

    #[test]
    fn chord_names() {
        let notes = vec![Pc0, Pc4, Pc5, Pc7, Pc11];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_chord_quality(&notes);
        let quality = quality.unwrap();
        let quality = quality.1.unwrap();
        assert_eq!(quality.to_string(&Default::default()), "Δ7 (11)");
        assert_eq!(
            quality,
             ChordQuality::Major(MajorSubtype::MajN(
                vec![Extension::Seventh, Extension::Eleventh], Alt(vec![])
            ))
        );
        let notes = vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert_eq!(quality.unwrap().1,
            Some(
             ScaleQuality::Major(vec![], vec![])
            )
        );
        // Lydian scale with b2 (0,1,4,5,7,9,11)
        let notes = vec![Pc0, Pc1, Pc4, Pc5, Pc7, Pc9, Pc11];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify Lydian b2 scale");

        // Altered scale / Superlocrian (0,1,3,4,6,8,10)
        let notes = vec![Pc0, Pc1, Pc4, Pc6, Pc7, Pc8, Pc10];
        let notes: HashSet<Pc> = PcSet::from(&notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify altered/superlocrian variant");

        // Locrian scale (0,1,3,5,6,8,10)
        let notes = vec![Pc0, Pc1, Pc3, Pc5, Pc6, Pc8, Pc10];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify Locrian scale");

        // Diminished whole-half scale variant (0,2,3,5,6,8,10)
        let notes = vec![Pc0, Pc2, Pc3, Pc5, Pc6, Pc8, Pc10];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify dim WH scale variant");
    }

    #[test]
    fn pentatonic_scales() {
        // Major pentatonic: 1 2 3 5 6 -> Pc0, Pc2, Pc4, Pc7, Pc9
        let notes = vec![Pc0, Pc2, Pc4, Pc7, Pc9];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify major pentatonic");
        assert_eq!(quality.unwrap().1, Some(ScaleQuality::MajorPentatonic));

        // Minor pentatonic: 1 b3 4 5 b7 -> Pc0, Pc3, Pc5, Pc7, Pc10
        let notes = vec![Pc0, Pc3, Pc5, Pc7, Pc10];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify minor pentatonic");
        assert_eq!(quality.unwrap().1, Some(ScaleQuality::MinorPentatonic));

        // Blues major: 1 2 b3 3 5 6 -> Pc0, Pc2, Pc3, Pc4, Pc7, Pc9
        let notes = vec![Pc0, Pc2, Pc3, Pc4, Pc7, Pc9];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify blues major");
        assert_eq!(quality.unwrap().1, Some(ScaleQuality::BluesMajor));

        // Blues minor: 1 b3 4 b5 5 b7 -> Pc0, Pc3, Pc5, Pc6, Pc7, Pc10
        let notes = vec![Pc0, Pc3, Pc5, Pc6, Pc7, Pc10];
        let notes: HashSet<Pc> = PcSet::from(notes).into();
        let quality = infer_scale_quality(&notes);
        assert!(quality.is_some(), "Should identify blues minor");
        assert_eq!(quality.unwrap().1, Some(ScaleQuality::BluesMinor));
    }
}

