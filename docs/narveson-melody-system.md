# A Narveson-based melody-generation system for `rust-music`

**Status:** Design proposal
**Source theory:** Paul Narveson, *Theory of Melody* (modern epoch, c. 1600–present)
**Target codebase:** `music`, `music-midi`, `music-engraver`, `musical-combinatorics` (workspace at `/home/eric/Documents/rust-music`)

This document proposes how Narveson's classifications and laws can become a Rust subsystem that **generates, analyzes, and revises melodies** — producing artifacts that flow naturally into the workspace's existing notation (LilyPond, VexTab, the in-progress engraver) and audio (`music-midi`) pipelines.

It is written to be implementable in incremental layers. The first three layers give us a usable end-to-end generator; the remaining layers add the long-range design, style modeling, and revision pipeline that make the output recognizably "Narvesonian."

---

## 1. Why Narveson maps cleanly to this codebase

Narveson's theory is unusually well-typed:

- **Six melodic elements**, each defined by precise structural constraints (direction, step/skip, layout).
- **Four phrase shapes** in two families (direct/concealed), each with explicit curvature counts.
- **Eight phrase types** (3 accompanying + 5 melody-line) with explicit role / ordering rules.
- **Style factors** as a tagged matrix (minor/major × baroque/classical/romantic/contemporary × element/background/MTC).
- **Interest features** with laws of compensation/subordination.

These are sum types and rules — exactly what Rust's enums + a constraint pass do well. They also map onto things we already have:

| Narveson concept | Existing type (or near-neighbor) |
|---|---|
| Phrase, pitch sequence | `Vec<MelodicEvent>` in `music::melody::sequencer` |
| Tonal context | `ChordProgression` / `TimedChord` / `NoteSet` |
| Rhythmic placement | `notation::rhythm::Duration`, beat grid |
| Stepwise / skipwise motion | `NoteSet::pitch_n_steps_from`, `Pc` / `Pitch` |
| Range, pitch level, span | `PitchBounds` |
| Chord/scale identity | `PcSet`, `NoteSet`, `OctavePartition` |
| Output formats | LilyPond (`notation::lilypond`), VexTab, MIDI (`music-midi`), SVG (`svg`) |

So the new subsystem doesn't reinvent these; it composes them.

---

## 2. Where it lives

Add a new crate to the workspace:

```
music-narveson/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── element/        # 1. melodic elements (Layer 1)
    ├── shape/          # 2. phrase shapes (Layer 2)
    ├── phrase/         # 3. phrase types, phrase IR (Layer 3)
    ├── interest/       # 4. interest features + laws (Layer 4)
    ├── style/          # 5. style profile matrix (Layer 5)
    ├── design/         # 6. core / reuses / subsurface design (Layer 6)
    ├── compose/        # 7. paragraph & long-range composition (Layer 7)
    ├── revise/         # 8. five-pass revision pipeline (Layer 8)
    ├── config/         # configuration surface + presets
    ├── analyze/        # inverse pass — classify an existing phrase
    └── render/         # serializers to LilyPond, MIDI, SVG, RON
```

`music-narveson` depends on `music` (always) and feature-flags everything else (`lilypond`, `midi`, `engraver`, `svg`). The existing `music::melody` module is a building block we will reuse — not a replacement target. (We integrate by emitting `Vec<MelodicEvent>` at the end of the pipeline.)

A new crate (rather than a `music::narveson` module) is justified because:
1. Narveson's vocabulary is opinionated; some users of `music` will want the primitives without it.
2. The crate carries its own preset library and (eventually) RNG/seed strategy, neither of which belongs in the core types.
3. It can mature independently and keep the `music` API small.

---

## 3. Core data model (Layer 1–3)

### 3.1 Elements — the atomic vocabulary

```rust
/// Narveson's six melodic elements. Each is either in normal/direct form
/// (a continuous succession of pitches) or in background form (the same
/// abstract pattern realized non-consecutively). The pedal element has
/// no background form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MelodicElement {
    Scale(ScaleElement),
    Appoggiatura(AppoggiaturaElement),
    Chord(ChordElement),
    Skip(SkipElement),
    Neighbor(NeighborElement),
    Pedal(PedalElement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementForm {
    /// Pitches lie consecutively in the surface phrase.
    Direct,
    /// Pitches are derived from contextual reduction (skeletal / cross-wave).
    Background,
}

/// A `ScaleElement`: three or more pitches moving stepwise in one direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleElement {
    pub direction: Direction,       // re-uses music::melody::Direction
    pub pitches: Vec<Pitch>,        // length >= 3
    pub form: ElementForm,
    /// True when the underlying motion is along a non-fully-stepped scale
    /// (e.g. pentatonic, or upper harmonic-minor A2 gap).
    pub gap_tolerated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppoggiaturaElement { pub pitches: [Pitch; 2], pub form: ElementForm }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordElement {
    pub pitches: Vec<Pitch>,        // >= 3; direction may freely mix
    pub guise: ChordGuise,
    pub form: ElementForm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordGuise {
    FamiliarChord, ModernChord, Polychordal, IncompleteChord,
    ShortNonchord, ExtendedNonchord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipElement {
    /// 2 direct pitches a skip apart.
    Plain { pitches: [Pitch; 2], form: ElementForm },
    /// 3+ pitches in a skipwise-isolated shake pattern.
    Shake { pitches: Vec<Pitch>,   form: ElementForm },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborElement {
    /// 3+ pitches alternating across a single m2/M2 (rarely d3) interval.
    pub pitches: Vec<Pitch>,
    pub basic_interval: BasicInterval, // m2 | M2 | d3
    pub form: ElementForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PedalElement {
    pub pitch: Pitch,
    /// Beat-grid positions where the pitch appears.
    pub positions: Vec<BeatPosition>,
    pub layout: PedalLayout,
    /// Pedal elements always have ElementForm::Direct.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PedalLayout {
    Regular,
    Agogic,
    Asorhythmic,
    AgogicWithRegular,
    Combined,   // rare combinations
}
```

### 3.2 The chain-of-elements rule

Narveson's combination rule ("each non-final direct element joins the next on one common note, the **linking note**; if it ends with repeated notes, the *last* repeated note is the linking note") becomes a single invariant on a phrase's element vector:

```rust
pub struct PhraseElements {
    pub direct: Vec<MelodicElement>,    // never Pedal here
    pub pedals: Vec<PedalElement>,      // possibly empty
}

impl PhraseElements {
    /// Verify the chain-of-elements rule and pedal coincidence rule.
    pub fn validate(&self) -> Result<(), ElementChainError> { /* ... */ }
}
```

Validation is a function, not an unsafe construction barrier — Narveson sometimes considers "false melodic chord elements" worth diagnosing, not just rejecting.

### 3.3 Phrase shapes

Curvature is half-integer (½, 1, 1½, 2, 2½, …). Encode with an integer count of *half-curves*:

```rust
/// Half-curves: 1 == ½ curve. 2 == 1 curve, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HalfCurves(pub u16);

#[derive(Debug, Clone)]
pub enum PhraseShape {
    Direct(DirectShape),
    Concealed(ConcealedShape),
}

#[derive(Debug, Clone)]
pub enum DirectShape {
    Simple(SimpleShape),     // 1 curve, or floor ½ curve
    Compound(CompoundShape), // 2 curves, or 1½ / 2½
}

#[derive(Debug, Clone)]
pub struct SimpleShape {
    pub curvature: HalfCurves,           // 1 or 2 (i.e. ½ or 1)
    pub range:   Interval,
    pub pitch_level: Pitch,
    pub proportions: Option<Ratio>,
    pub extraneants: Vec<Extraneant>,    // empty == nonexceptional
}
#[derive(Debug, Clone)]
pub struct CompoundShape { /* curvature in {3,4,5}; anchor_curve : remaining */ }

#[derive(Debug, Clone)]
pub enum ConcealedShape {
    Skeletal(SkeletalShape),     // pitches on principal accents
    CrossWave(CrossWaveShape),   // pitches on a consistent pulse weight
}

#[derive(Debug, Clone)]
pub enum Extraneant {
    DanglingReturn(DanglingReturnPattern),
    InterruptingRepetitive(InterruptingRepetitivePattern),
    DeviatingPitch(DeviatingPitch),
}
```

The four shape rules from §02.02 translate directly into constructor functions (`SimpleShape::try_new(...)`, etc.) that return `Result<_, ShapeError>` and check explicit invariants (e.g. *deviating pitch must occur off an effective beat and be preceded by ≥3 prior notes, ≥2 distinct in pitch*).

### 3.4 Phrase types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseType {
    // Accompanying (essential regularity)
    Elaborate, SemiMelodious, Rudimentary,
    // Melody-line (essential irregularity)
    Introducing, Stating, Extending, Transisting, Concluding,
}

/// Substitutions like "stating-concluding phrase" or "transisting-introducing phrase".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhraseRole { pub primary: PhraseType, pub substituting_for: Option<PhraseType> }
```

### 3.5 The `Phrase` IR — single point of truth

This is the IR all layers read and write:

```rust
pub struct Phrase {
    /// The realized note stream (this is what gets rendered to LilyPond/MIDI).
    pub events: Vec<MelodicEvent>,         // music::melody::MelodicEvent
    /// Beat grid / meter (re-uses notation::rhythm).
    pub meter: Meter,
    /// Harmonic context this phrase was generated against.
    pub harmony: ChordProgression,
    /// Narveson-level structural annotations.
    pub analysis: PhraseAnalysis,
}

pub struct PhraseAnalysis {
    pub elements: PhraseElements,
    pub primary_shape: PhraseShape,
    pub secondary_shapes: Vec<PhraseShape>,    // concealed + (optionally) one secondary direct
    pub role: PhraseRole,
    pub melodic_tonal_center: Option<MtcAnalysis>,
    pub interest: InterestReport,             // populated by Layer 4
    pub core_reuses: CoreReuseMap,            // populated by Layer 6
}
```

This dual representation (concrete events + structural analysis) is intentional. It lets us:

- **Render** any phrase with the existing notation crates (events drive output).
- **Revise** any phrase by editing the analysis and re-deriving events.
- **Analyze** an externally-supplied phrase by running classification *into* this IR — supporting the "inverse" use case (read a LilyPond fragment, get back its Narveson description).

---

## 4. Interest features and laws (Layer 4)

Narveson splits "melodic interest" into:

- **Revised-form features** — `Variety`, `Irregularity`, `Subtlety`.
- **Basic features** — `DelayedCompletion`, `SurpriseCompletion`.
- **Two laws** — *compensation* and *subordination*.

Encode each feature as a scored detector trait, then aggregate:

```rust
pub trait InterestFeature {
    fn id(&self) -> InterestKind;
    fn detect(&self, phrase: &Phrase) -> Vec<InterestInstance>;
}

pub struct InterestInstance {
    pub kind: InterestKind,
    pub span: EventSpan,
    pub magnitude: f32,    // 0..1
}

pub enum Quality {
    Inferior, Imperfect, Standard, Strong, Superior,
}

pub struct InterestReport {
    pub instances: Vec<InterestInstance>,
    pub law_of_compensation: ComplianceLevel,
    pub law_of_subordination: ComplianceLevel,
    pub overall_quality: Quality,
}
```

`overall_quality` is computed by a *single* scoring function that combines:

- the count and magnitude of detected instances,
- compliance with the two laws (penalize uncompensated tension, penalize features so prominent they upstage primary material),
- weighting drawn from the active `StyleProfile`.

This is the natural decision boundary the generator and the revision pipeline both consult.

---

## 5. Style profile (Layer 5)

The minor + major factor matrix becomes a single configuration record. Defaults reproduce one of Narveson's named styles; users can override any axis:

```rust
pub struct StyleProfile {
    pub era: EraPreset,                 // Baroque | Classical | Romantic | Contemporary | Custom
    pub element_ranges: ElementRangePolicy,         // minor baroque/classical/romantic/contemporary
    pub chord_element_guise_bias: ChordGuiseBias,
    pub background: BackgroundPolicy,               // rbs-series, scale variety, tendency tones
    pub tendency_tone: TendencyTonePolicy,
    pub prominent_scale_tones: Vec<ScaleDegree>,    // baroque: do/sol; romantic: mi/sol; ...
    pub overall_background_scale: OverallBgScalePolicy,
    pub mtc: MtcPolicy,                             // central-tone vs. two-indicative-tones
    pub composer_quirk: Option<ComposerQuirk>,      // Hindemith P4, Brahms 3rds, Bartok P4+tritone, Tchaikovsky bg-scale themes
}

impl StyleProfile {
    pub fn baroque()      -> Self { /* ... */ }
    pub fn classical()    -> Self { /* ... */ }
    pub fn romantic()     -> Self { /* ... */ }
    pub fn contemporary() -> Self { /* ... */ }
}
```

Style profiles are *the* primary user-facing knob. Every other generator function takes `&StyleProfile` by reference and consults it for thresholds, biases, and weights — keeping the rest of the API "sane defaults" for users who don't want to hand-tune.

---

## 6. Short-range design (Layer 6) — the heart of generation

Narveson's §03.02 ("short-ranged melodic design") gives us the **core** concept and four subsurface designs:

```rust
/// A "melodic core" — the small, recurring kernel the rest of the phrase is built around.
pub struct Core {
    /// Either a pitch sub-pattern, a rhythm sub-pattern, or both.
    pub kernel: CoreKernel,
    /// Reuses (transpositions, variants) detected or planned within the phrase.
    pub reuses: Vec<CoreReuse>,
}

pub enum CoreKernel {
    Pitch(Vec<Pc>),
    Rhythm(Vec<Duration>),
    Combined(Vec<(Pc, Duration)>),
}

pub enum SubsurfaceDesign {
    // Part-to-whole
    Flexible(FlexibleDesign),
    Attachment(AttachmentDesign),
    // Whole-to-part
    ShapeElaborated(ShapeElaboratedDesign),
    BackgroundElaborated(BackgroundElaboratedDesign),
}
```

**Pipeline for generating a single phrase:**

```text
StyleProfile + HarmonicContext + PhraseRole + RNG
        │
        ▼
1. Choose primary PhraseShape consistent with PhraseRole and StyleProfile.
2. Pick a SubsurfaceDesign appropriate to the shape:
   - direct shapes => flexible / attachment / shape-elaborated
   - concealed shapes => background-elaborated typically
3. Generate a Core (kernel) sized to the design.
4. Lay the core into the shape skeleton; place reuses along the curvature contour.
5. Fill the surface with melodic elements obeying:
   - chain-of-elements rule
   - element-range policy (StyleProfile)
   - tendency-tone treatment (StyleProfile)
   - chord membership (HarmonicContext)
6. Apply chosen extraneants (if direct shape; counts/positions per Narveson rules).
7. Snap to beat grid (Meter); emit Vec<MelodicEvent>.
8. Run the Layer-4 interest detectors; record InterestReport.
9. Return a fully-annotated Phrase.
```

Steps 1–4 are deterministic given the seed; steps 5–7 use the RNG with style-weighted choices. The seed lives in `GenerationConfig::seed` so any output is reproducible.

`music::melody::MelodicSequencer` is *one tool used inside step 5*: when a section of the surface is best described as "step through these chord tones with this interval pattern," we instantiate a sequencer with the right `IntervalPattern`. The new system orchestrates which sequencer (or which non-sequencer routine) to use where.

---

## 7. Long-range composition (Layer 7)

A `MelodicParagraph` is a vector of phrases playing the roles introducing → stating → transisting → concluding (with extending phrases joining onto theses, and optional fragmentary additions):

```rust
pub struct MelodicParagraph {
    pub phrases: Vec<Phrase>,
    pub master_subsurface_design: MasterSubsurfaceDesign,
    pub submaster_subsurface_design: Option<SubmasterDesign>,
    pub cross_phrase_master_core_reuses: Vec<CrossPhraseReuse>,
    pub ordering: ParagraphOrdering, // Normal | PartialNormal | Substituted | Reversed
}
```

Composition order at the paragraph level:

1. Choose ordering (default Normal; configurable / probabilistic).
2. Decide a **master subsurface design**: a single core that's the "DNA" of the whole paragraph.
3. Allocate phrase roles + lengths (Narveson notes paragraphs of 3–9 phrases).
4. For each phrase: generate (Layer 6) with the master core seeded into its short-range design.
5. Plant **cross-phrase master core reuses** at structurally-important positions (downbeats of stating area, return points, etc.).
6. Apply submaster design if requested.

---

## 8. Revision pipeline (Layer 8) — the five-pass loop

Narveson is explicit: melody is *not* "compose then stop"; it is **rough draft → intensive revision** (warmup → design → interest → hidden-core → final-touch). Encode this as a strategy chain:

```rust
pub trait RevisionPass {
    fn name(&self) -> &'static str;
    fn revise(&self, phrase: &mut Phrase, style: &StyleProfile, rng: &mut Rng);
}

pub struct WarmupRevision;
pub struct DesignRevision;
pub struct InterestRevision;
pub struct HiddenCoreRevision;
pub struct FinalTouchRevision;

pub struct RevisionPipeline {
    pub passes: Vec<Box<dyn RevisionPass>>, // default = the five above in order
    pub max_iterations: u8,
    pub stop_on_quality: Quality,           // e.g. Standard
}
```

Each pass takes the phrase, evaluates it against the relevant detector subset (warmup: gross errors; design: shape integrity; interest: instances + laws; hidden-core: core reuse density; final-touch: contour smoothing), and **mutates** it via small local edits.

This pipeline is what distinguishes Narvesonian output from raw stochastic melody: the *quality scoring is in-loop*, not post hoc.

---

## 9. Configuration surface

The top-level user API is two records plus a default:

```rust
pub struct GenerationConfig {
    pub style: StyleProfile,
    pub harmony: ChordProgression,
    pub bounds: PitchBounds,
    pub meter: Meter,
    pub paragraph: ParagraphSpec,
    pub revision: RevisionPipeline,
    pub seed: u64,
}

pub struct ParagraphSpec {
    pub phrase_count: RangeInclusive<u8>,    // default 3..=9 per Narveson
    pub ordering: OrderingPolicy,            // default Normal w/ small substitution probability
    pub include_introducing: Probability,    // default low ("basically a luxury")
    pub include_transisting: Probability,    // default ~0.5 ("about half")
    pub include_concluding:  Probability,    // default ~0.5
    pub fragmentary_additions: Probability,  // default low
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            style: StyleProfile::classical(),
            harmony: ChordProgression::static_chord(NoteSet::new(vec![Note::C, Note::E, Note::G])),
            bounds: PitchBounds::try_new(Pitch::new(Note::C, 3), Pitch::new(Note::C, 6)).unwrap(),
            meter: Meter::common_time(),
            paragraph: ParagraphSpec::default(),
            revision: RevisionPipeline::standard_five_pass(),
            seed: 0xC0FFEE,
        }
    }
}
```

Anything more sophisticated (custom interest weights, custom shape repertoire, custom extraneant frequencies) is reachable through builder methods but not required.

### 9.1 Public entry points

```rust
// One-shot
pub fn compose(cfg: &GenerationConfig) -> Result<MelodicParagraph, NarvesonError>;

// Streaming / phrase-by-phrase (for interactive use)
pub struct Composer { /* ... */ }
impl Composer {
    pub fn new(cfg: GenerationConfig) -> Self;
    pub fn next_phrase(&mut self, role: PhraseRole) -> Result<Phrase, NarvesonError>;
}

// Inverse — classify external input
pub fn analyze(events: &[MelodicEvent], harmony: &ChordProgression, meter: &Meter)
    -> Result<PhraseAnalysis, NarvesonError>;
```

`Composer` gives concert-style use: "give me one stating phrase in C major, classical, then a transisting phrase modulating to G."

---

## 10. Output integration

The `Phrase` and `MelodicParagraph` types are the *only* surface the rest of the workspace needs to consume. Renderers translate them into the formats already in this repo:

| Target | Module | Notes |
|---|---|---|
| LilyPond source | `music-narveson::render::lilypond` (gated by `lilypond` feature) | Builds on `music::notation::lilypond` templates. Adds **annotation overlays** (element brackets, shape labels, core highlights) via LilyPond `\once \override` for analytical PDFs. |
| VexTab source | `music-narveson::render::vextab` | For web frontends already targeted by `music::notation::vextab`. |
| MIDI / playback | `music-narveson::render::midi` (depends on `music-midi`) | `Phrase` → `Vec<MidiEvent>` → existing SMF writer / synth path. Articulation rules driven by style profile (legato in romantic; detached in baroque). |
| SVG analytical diagrams | `music-narveson::render::svg` | Renders **annotated phrase diagrams** — pitches on a staff-like grid with element brackets, shape arcs, core highlights — using `music::svg`. Mirrors the `pitch_circle` / `interval` builders' style so output looks like the rest of the project. |
| RON (round-trippable) | `music-narveson::render::ron` (depends on `music-ron`) | The whole `MelodicParagraph` plus its analysis serialized for tests, fixtures, and the `lilypond-parser` round-trip story. |

This is why a unified IR matters: every renderer reads `Phrase` and never has to re-derive structural information.

### 10.1 Example artifact pipelines

**"Generate a Baroque stating phrase and play it":**

```rust
let cfg = GenerationConfig {
    style: StyleProfile::baroque(),
    harmony: ChordProgression::new(vec![
        TimedChord::new(NoteSet::new(vec![Note::C, Note::E, Note::G]), Duration::HALF),
        TimedChord::new(NoteSet::new(vec![Note::G, Note::B, Note::D]), Duration::HALF),
    ]),
    ..Default::default()
};
let mut composer = Composer::new(cfg);
let phrase = composer.next_phrase(PhraseRole::stating())?;
music_narveson::render::midi::play(&phrase)?;          // music-midi path
let ly = music_narveson::render::lilypond::phrase(&phrase, AnnotationLevel::None)?;
std::fs::write("out.ly", ly)?;
```

**"Analyze and visualize an existing LilyPond fragment":**

```rust
let events = lilypond_parser::parse_to_events("c4 d e f g2")?;
let analysis = music_narveson::analyze(&events, &harmony, &meter)?;
let svg = music_narveson::render::svg::annotated_phrase(&events, &analysis, &SvgTheme::print());
std::fs::write("analysis.svg", svg.to_string())?;
```

---

## 11. Testing strategy

This is the area where the structure pays off most. We get three test layers for free:

1. **Rule unit tests.** Each Narveson rule (chain-of-elements; deviating-pitch position; extraneant counts; etc.) becomes a single function with `#[test]` cases drawn from the source's examples. The book repeatedly gives `†`-marked author examples — those are gold inputs.
2. **Round-trip tests.** `Phrase` → LilyPond → `lilypond-parser` → events → `analyze` → comparable `PhraseAnalysis`. This catches both serialization bugs and classification drift.
3. **Statistical / property tests.** With a fixed seed and a style preset, the *distribution* of detected element kinds, shape types, and interest features should fall within style-specific bounds (proptest-style ranges). For example: `StyleProfile::baroque()` should yield element ranges exceeding the octave with probability < 0.1; `StyleProfile::classical()` should yield it significantly often. These tests double as a regression check on the style matrix.

---

## 12. Phasing (concrete milestone plan)

Each phase ends with a runnable example artifact. None of them require a later phase to be useful.

| Phase | Deliverable | Output artifact |
|---|---|---|
| **P1** | Layer 1–3: elements, shapes, phrase IR, validators. No generation yet. | `analyze` works on hand-written `Vec<MelodicEvent>`s; passes Narveson's textbook examples. |
| **P2** | Single-phrase generation: direct shapes only, simple cores, no extraneants, default style. | `cargo run --example narveson-phrase` prints a LilyPond stating phrase. |
| **P3** | Extraneants + concealed shapes + secondary shape combinations. | `cargo run --example narveson-shape-gallery` renders one phrase per shape kind to SVG. |
| **P4** | Interest detectors + laws + quality scoring. | Quality score reported alongside each generated phrase; can be inspected. |
| **P5** | Style profile matrix (all four eras + composer quirks). | `cargo run --example narveson-styles` produces four MIDIs side by side, same harmony, different styles. |
| **P6** | Short-range design (cores, reuses, four subsurface designs). | Phrases are recognizably motivically coherent; reuse map serializable to RON. |
| **P7** | Paragraph composition + master subsurface design. | Multi-phrase compositions playable end-to-end via `music-midi`. |
| **P8** | Five-pass revision pipeline. | Quality scores demonstrably improve over passes (statistical test). |
| **P9** | Annotated rendering: shape arcs, element brackets, core highlights in LilyPond + SVG. | Analytical PDFs suitable for the project docs. |

The phasing is intentionally aligned with Narveson's book structure: Phase 1–3 cover Part A–C (mechanics), Phase 4–5 cover Part D (interest and style), Phase 6–9 cover the dynamics section (design, paragraph, revision).

---

## 13. Open questions to settle before P1

These are decisions the design leaves explicit so they can be chosen up front rather than discovered in code:

1. **RNG library** — `rand` with a fixed `SeedableRng`, or roll a small deterministic PRNG to avoid adding a workspace dep? Recommendation: `rand` (already standard) + `rand_chacha` for reproducibility.
2. **Microtonality** — the underlying `Pitch` is 12-EDO. Narveson is explicitly modern-epoch Western, so this is consistent. If 19/31/JI extensions are wanted later, the IR is unaffected because intervals are computed via `NoteSet` step methods.
3. **Tendency-tone resolution policy granularity** — Narveson distinguishes immediate, delayed, and two non-resolving treatments. Encode as an enum with four variants and let the style profile weight them.
4. **Where to enforce the chain-of-elements rule** — at construction (strict) or only at validation (permissive)? Recommendation: permissive for `analyze`, strict for `compose`. Implemented by separate constructors (`PhraseElements::checked` vs `PhraseElements::raw`).
5. **Should the analyzer attempt to classify every input as one of the eight phrase types?** — Recommendation: yes, but return a confidence score; some hybrid phrases will reasonably score across two types.

---

## 14. Why this design

The shape of the proposal follows three principles:

- **Narveson's vocabulary becomes the type system.** Enums where he enumerates; structs where he names parts; trait objects only for *behaviors* (interest detectors, revision passes) that the user might extend. No reflection, no string-keyed dispatch.
- **One IR, multiple readers.** `Phrase` carries both the realized events and the structural analysis; every renderer and every revision pass touches the same data. This is what lets us add LilyPond annotation, SVG visualization, and MIDI playback without parallel state.
- **Sane defaults all the way down.** A user who wants "make me a melody" calls `compose(&GenerationConfig::default())` and gets a respectable classical stating phrase. A user who wants Bartók-flavoured contemporary phrases sets one field. A user who wants to override the deviating-pitch frequency in modified-simple shapes builds their own `StyleProfile`. None of those users pay for capability they don't use.

The result fits the existing workspace conventions: a new crate alongside `music-midi` and `music-ron`; consumes `music` primitives; emits artifacts in formats `lilypond-parser`, `music-engraver`, and `music-midi` already speak.

---

## 15. Extensions

These extensions slot into the existing architecture without altering its core. Each is described in terms of how it changes the `Phrase` IR, the `StyleProfile`, and the generation pipeline.

### 15.1 Learning style profiles from transcribed input

The `StyleProfile` is a record of numeric biases, thresholds, and weights — so it can be **fit** from a corpus rather than authored by hand. The work splits into three pieces, each leaning on machinery we already have:

```rust
pub struct CorpusItem {
    pub events: Vec<MelodicEvent>,
    pub harmony: ChordProgression,
    pub meter:   Meter,
    pub label:   Option<String>,     // e.g. "Bach WTC I Fugue 2", optional
}

pub struct StyleFitOptions {
    pub smoothing:    f32,           // Laplace-like add-α smoothing for sparse counts
    pub min_evidence: usize,         // require N phrases before fitting a given factor
    pub weight_by_paragraph: bool,   // upweight phrases nearer paragraph centers
}

pub fn fit_style(corpus: &[CorpusItem], opts: &StyleFitOptions) -> StyleProfile;

/// Blend two profiles — useful for "70% Bach, 30% me".
pub fn blend(a: &StyleProfile, b: &StyleProfile, t: f32) -> StyleProfile;

/// How well does this phrase look like that style? (For corpus diagnostics.)
pub fn style_likeness(p: &Phrase, s: &StyleProfile) -> f32;
```

How each factor is fit:

| Factor | Estimator |
|---|---|
| `element_ranges` | Histogram of scale/chord/skip element ranges; choose the policy variant whose distribution best matches (KL-divergence to canonical baroque/classical/romantic/contemporary curves). |
| `chord_element_guise_bias` | Counts over `ChordGuise` classifications produced by `analyze`. |
| `background.rbs_series` | Frequency of scale-belonging across consecutive rbs segments. |
| `tendency_tone` | Conditional distribution of treatments observed on `ti`, `fa`, etc. |
| `prominent_scale_tones` | Top-2 scale degrees by occupancy of concealed-shape pitches. |
| `overall_background_scale` | Mode of the do→sol / mi→do / sol→mi spans across phrases. |
| `mtc` | Fraction of phrases whose first/last-pitch relationship matches the central-tone vs. two-indicative-tones chart. |
| `composer_quirk` | Threshold-triggered: e.g. if predominant skip == P4 with prevalence > τ, set `ComposerQuirk::Hindemith`. |

**Pipeline:** run `analyze` over every item → aggregate counters into a `StyleStatistics` intermediate → derive a `StyleProfile` by mapping each statistic to its policy enum or weight. The intermediate is itself useful (serializable, comparable, plottable) and falls out naturally.

This composes with the existing system unchanged: `fit_style` returns a `StyleProfile` that the generator already knows how to consume. Train once, generate forever.

A natural pairing: a `LearnedStyleProfile` newtype that wraps a `StyleProfile` plus the `StyleStatistics` it was fit from, so the generator can fall back to sampled distributions for factors with strong empirical signal but no canonical Narveson policy.

### 15.2 Pitch-set constraints (strict and soft)

Two questions are entangled here: *which pitch classes are allowed at all* and *how strictly*. Model them as a single layer in the generation pipeline:

```rust
pub struct PitchSetConstraint {
    pub allowed: PcSet,                       // music::note_collections::PcSet
    pub mode: PitchSetMode,
    /// Optional: certain pitches "cost more" than others, even within the allowed set.
    pub weights: Option<HashMap<Pc, f32>>,
}

pub enum PitchSetMode {
    /// Reject any pitch outside `allowed`.
    Strict,
    /// Allow deviations with budget `deviation_budget` per phrase; out-of-set pitches
    /// are penalized in scoring but not forbidden.
    Soft { deviation_budget: f32, penalty_per_note: f32 },
    /// As Soft, but deviating pitches must be tendency tones resolving to set members
    /// (this is exactly Narveson's chromatic-tone-as-tendency-tone rule).
    SoftWithResolution { deviation_budget: f32, resolution_window: u8 },
}
```

`PitchSetConstraint` plugs into step 5 of the per-phrase pipeline (§6) as a *filter on candidate pitches*: when the sequencer or generator is choosing the next note,

1. Compute the candidate set normally (chord tones + scale + tendency tones).
2. Intersect with `allowed` in `Strict` mode, or score with penalty in `Soft` modes.
3. The revision pipeline's *interest revision* pass treats `Soft` deviations as **deliberate** if and only if they fit the style profile's tendency-tone policy.

This lets users say:
- "Stay strictly in C minor pentatonic" (Strict).
- "Stay mostly in C dorian, but you may borrow up to 2 notes per phrase as long as they resolve" (SoftWithResolution).
- "Use the 8-note Bartók scale, but the tritone is twice as expensive as a perfect fourth" (Strict + weights).

Internally this is a single trait `PitchPolicy` consulted at note-choice time; the constraint mode just selects which implementation. `PcSet` already exists in your `note_collections` so the input ergonomics are clean.

### 15.3 Harmonic function and chord-progression as generative substrate

The current `ChordProgression` is timing-only ("C major for 2 beats, G major for 2 beats"). Harmonic *function* is richer — and Narveson explicitly relies on it (rbs series, tonal center, indicative tones). Extend the harmonic input in two complementary ways:

```rust
/// Augments a TimedChord with functional context.
pub struct FunctionalChord {
    pub timed:        TimedChord,            // existing music::melody type
    pub function:     HarmonicFunction,      // Tonic | Dominant | Subdominant | Predominant | Secondary{ of: ScaleDegree } | Modal { mode: Mode } | NonFunctional
    pub key:          KeyContext,            // root pitch class + Major/Minor/ChurchMode
    pub stability:    Stability,             // Tonic | Stable | Tensed | UnstableExpectingResolution
    pub avoid_tones:  Vec<Pc>,               // explicit "do not land here"
    pub color_tones:  Vec<Pc>,               // explicit "lean into these"
}

pub struct FunctionalProgression {
    pub chords: Vec<FunctionalChord>,
    /// Optional macro-form: e.g. AABA, sonata exposition, blues 12-bar. Lets the
    /// paragraph-level composer align stating/transisting/concluding areas with
    /// harmonic structural points.
    pub form:   Option<MacroForm>,
}
```

Three integration points:

1. **Per-note pitch choice.** The generator's candidate-set construction already consults the current chord's `NoteSet`. With functional information available, the candidate-set is built as `chord_tones ∪ scale_tones(key) \ avoid_tones`, with `color_tones` boosted in weight.
2. **Phrase-role placement.** A `FunctionalProgression` with `form: AABA` automatically aligns: stating phrase on the A section's tonic establishment; transisting phrase across A→B; concluding phrase on the final A's cadence.
3. **Tendency-tone resolution.** Functional context disambiguates which non-chord tones are tendencies: `ti` over a V chord is the leading tone (strong resolution to *do*); `ti` over a I chord is the maj7 (color, no resolution required). The existing `TendencyTonePolicy` consults `FunctionalChord::function` rather than just the key signature.

Two convenient builders:

```rust
FunctionalProgression::from_roman_numerals("I vi ii V | I", &KeyContext::c_major());
FunctionalProgression::from_chord_symbols("Cmaj7 | Am7 D7 | Gmaj7", &KeyContext::c_major());
```

The Roman-numeral parser is a thin wrapper over `note_collections::chord_name::parsing` (which already exists). The chord-symbol path piggybacks on the same parser and infers function from key context.

If the user supplies *only* `ChordProgression` (no function), the system reasonably infers function from chord identity + key — but explicit input dominates inference.

### 15.4 Replacing or augmenting the RNG with a learned model

The pipeline already isolates randomness behind decision points: "choose a phrase shape," "choose an extraneant," "choose the next pitch among candidates." Those decision points become the **interface** between the generator and any model — RNG today, neural model tomorrow.

```rust
pub trait DecisionOracle {
    fn choose_shape(&mut self, ctx: &ShapeContext) -> PhraseShapeKind;
    fn choose_element(&mut self, ctx: &ElementContext) -> ElementKind;
    fn choose_pitch(&mut self, candidates: &[PitchCandidate], ctx: &PitchContext) -> Pc;
    fn choose_duration(&mut self, candidates: &[Duration], ctx: &RhythmContext) -> Duration;
    fn choose_extraneant(&mut self, ctx: &ExtraneantContext) -> Option<ExtraneantKind>;
    // ... one method per decision point
}
```

Each `*Context` carries the **full local view**: current `Phrase` so far, active `StyleProfile`, active `FunctionalChord`, pitch-set constraint, beat position, distance to phrase end. That is enough state for any reasonable decision model.

Three concrete oracle implementations:

| Implementation | Description | When to use |
|---|---|---|
| `RngOracle` | Style-weighted weighted choice over `candidates` using `rand_chacha`. The default. | Reproducible procedural generation; tests; offline batch composition. |
| `MarkovOracle` | Fit per-decision Markov tables from a corpus (using the §15.1 statistics). Deterministic given seed. | Imitating a small corpus tightly; very fast; runs offline. |
| `LlmOracle` | Calls out to an LLM with a structured prompt: serialize the context as a compact JSON-ish blob, request a single token from `candidates`, parse. | Capturing higher-order musical knowledge the rule system doesn't model. |

`LlmOracle` design notes:

- The model never sees raw pitches outside `candidates`. The oracle's *contract* is "pick one of these"; the rule layer continues to enforce shape, extraneant counts, tendency-tone resolution, pitch-set membership. The model contributes taste; the rules enforce correctness. (This is the same separation that makes constrained-decoding tractable for code models.)
- Each call is independent and cacheable on `(context_hash, candidates_hash)` — cache the choice keyed on the abstract context, not on call order. This makes generation deterministic for a fixed model + prompt and amortizes cost across re-generations.
- Provide a `BatchedLlmOracle` that requests several upcoming decisions in one call when the rule layer can pre-compute candidate sets for, e.g., the next 4 notes. Latency dominates cost here, so batching matters.
- The Anthropic SDK pattern from the workspace's existing `claude-api` skill applies: prompt caching on the system prompt + style profile description; only the running phrase context changes per call.

**Hybrid oracle.** The most useful variant in practice is `WeightedOracle::new(vec![ (0.7, llm), (0.3, rng) ])` — fall back to RNG when the model returns out-of-set values or hits a budget, and route low-stakes decisions (which of two equivalent rhythms) directly to RNG to save tokens. The decision-point abstraction makes this a one-liner.

**Training data path.** `analyze` produces `PhraseAnalysis` for any input corpus; project that into (context, decision) pairs and you have a fine-tuning dataset. The exact same `*Context` types serialized for inference are serialized for training.

### 15.5 Rhythmic templates

Rhythm is currently a `Vec<Duration>` cycled by the sequencer. That's too thin to express the kinds of constraints that matter for stylistic melody (motive rhythm, syncopation budgets, hemiola, anacrustic emphasis). Add a rhythmic template layer:

```rust
pub struct RhythmTemplate {
    pub meter: Meter,
    pub events: Vec<RhythmSlot>,
    pub repeats: RepeatPolicy,
    pub flex: RhythmFlex,
}

pub enum RhythmSlot {
    /// Fixed: duration `d`, note attack required.
    Note { duration: Duration, accent: Accent },
    /// Fixed: duration `d`, rest.
    Rest { duration: Duration },
    /// Free: any duration from `choices` (RNG / oracle picks).
    Choice { choices: Vec<Duration>, weights: Option<Vec<f32>> },
    /// Tied continuation of previous slot.
    Tie,
    /// Wildcard: oracle fills a span of total duration `budget` with any rhythm.
    Fill { budget: Duration },
}

pub struct RhythmFlex {
    pub allow_subdivision: bool,        // may split a Note slot into two halves
    pub allow_tying:       bool,        // may tie across slots
    pub syncopation_budget: u8,         // off-beat attacks allowed per measure
    pub tuplet_policy:     TupletPolicy,
}
```

How it composes:

- The template lives in `GenerationConfig::rhythm` alongside the style profile. Default: a one-measure template of `Choice` slots weighted by style (baroque = mostly eighths/sixteenths; romantic = wider spread).
- The phrase generator places melodic elements **over** the template: when an element wants more notes than the current template region offers, it triggers subdivision (if `allow_subdivision`); when fewer, it triggers tying (if `allow_tying`).
- The skeletal-shape generator places principal-accent pitches directly on slots marked with `Accent::Principal`. This is how Narveson's rule "skeletal pitches lie on principal accents" becomes a constraint and not just a coincidence.
- The pedal element's layout (regular / agogic / asorhythmic) drives template generation when no template is supplied: agogic layout produces a template emphasizing agogic-accent slots; asorhythmic produces a template with mixed meters.

**Template libraries.** Ship a presets module: `Templates::sicilienne_6_8()`, `Templates::common_time_anacrustic()`, `Templates::bebop_eighths()`, `Templates::three_against_two_hemiola()`. These are just `RhythmTemplate` constants and are easy to grow.

**Learnt templates.** `fit_rhythm_templates(&corpus, k)` clusters the corpus's measure-rhythms into `k` templates (k-medoids over rhythm-string edit distance). Style profiles can then carry an inferred template repertoire instead of policy enums.

### 15.6 How these extensions interact

The five extensions above are designed to be **orthogonal**: each one plugs into a single, named seam in the existing pipeline.

| Extension | Seam |
|---|---|
| 15.1 Learned profiles | Produces `StyleProfile` — no other changes. |
| 15.2 Pitch-set constraints | A pitch-policy filter in step 5 of the per-phrase pipeline. |
| 15.3 Functional harmony | Replaces / augments `ChordProgression` input; consulted at note-choice and phrase-role placement. |
| 15.4 Oracle | Replaces the RNG behind decision points; everything else unchanged. |
| 15.5 Rhythm templates | Replaces `rhythm_pattern: Vec<Duration>` with a richer structure that the generator consults during template-aware note placement. |

A worked example combining all five — *"a Bach-fit profile, restricted to C natural minor with chromatic tendency tones allowed, over a ii–V–i progression in 6/8 with a sicilienne template, choices made by an LLM oracle with RNG fallback"*:

```rust
let profile = fit_style(&bach_corpus, &StyleFitOptions::default());
let cfg = GenerationConfig {
    style: profile,
    harmony: FunctionalProgression::from_roman_numerals(
        "ii° V7 i", &KeyContext::c_minor()).into(),
    pitch_set: Some(PitchSetConstraint {
        allowed: PcSet::c_natural_minor(),
        mode: PitchSetMode::SoftWithResolution {
            deviation_budget: 1.5,
            resolution_window: 2,
        },
        weights: None,
    }),
    rhythm: Templates::sicilienne_6_8(),
    oracle: Box::new(WeightedOracle::new(vec![
        (0.7, Box::new(LlmOracle::claude_sonnet())),
        (0.3, Box::new(RngOracle::seeded(0xBACH))),
    ])),
    ..Default::default()
};
let paragraph = compose(&cfg)?;
```

No new layers are introduced; each field is a knob already present in the design with a richer type.
