# Chord Naming System & Bug Analysis

This document analyzes the chord naming systems in both libraries, documents known bugs in the Python implementation, and suggests fixes for the Rust implementation.

## Architecture Overview

### Python: Procedural with Global State

```python
# name_gen.py - Main naming function
def name(root, pitch_set, context=None):
    """
    Infer chord name from root and pitch classes.
    Returns (quality, name, omitted_notes, parent_scale).
    """
    # Complex conditional logic with regex
    # ~700 lines of heuristics
```

### Rust: Structured with Types

```rust
// chord_name/mod.rs
pub struct ChordName {
    pub tonality: TonalSpecification,
    pub quality: ChordQuality,
    pub pc_set: PcSet,
}

pub enum TonalSpecification {
    SlashChord { bass: Note, root: Note },
    RootPosition(Note),
    None(Option<Pc>),
}
```

## Known Python Bugs

### Bug 1: Harmonic Minor False Positives

**Location:** `name_gen.py` lines 21-46

**Description:** The harmonic minor detection is too lenient, matching chords that shouldn't be classified as harmonic minor derivatives.

```python
# Problematic code in name_gen.py
def seven_note_scale_match(pcs):
    """Match against 7-note scale patterns."""
    for scale_name, pattern in seven_note_scale_heuristics.items():
        if matches_with_alterations(pcs, pattern):
            return scale_name, get_alterations(pcs, pattern)
    return None, []

# The issue: matches_with_alterations is too permissive
def matches_with_alterations(pcs, pattern):
    """Check if pcs matches pattern allowing alterations."""
    # BUG: Doesn't check for required notes (e.g., needs 9th present)
    # BUG: Allows too many alterations
    matched = sum(1 for p in pcs if p in pattern)
    return matched >= len(pattern) - 2  # Too lenient!
```

**Example of incorrect behavior:**
```python
# Python incorrectly classifies:
# [0, 3, 7, 11] as "Harmonic Minor" derivative
# Should be: "mMaj7" (minor-major seventh)
```

**Rust Fix Suggestion:**

```rust
// In naming_heuristics/scale_qualities.rs
pub fn match_seven_note_scale(
    pc_set: &PcSet,
    root: &Note
) -> Option<(ScaleQuality, Vec<Pc>)> {
    let pcs: HashSet<Pc> = pc_set.0.iter().cloned().collect();

    for (quality, required, optional) in SCALE_PATTERNS.iter() {
        let required_set: HashSet<Pc> = required.iter().cloned().collect();

        // ALL required notes must be present
        if !required_set.is_subset(&pcs) {
            continue;
        }

        // Count alterations (notes not in required or optional)
        let all_expected: HashSet<Pc> = required.iter()
            .chain(optional.iter())
            .cloned()
            .collect();

        let alterations: Vec<Pc> = pcs.difference(&all_expected)
            .cloned()
            .collect();

        // Maximum 1 alteration for scale match
        if alterations.len() <= 1 {
            return Some((quality.clone(), alterations));
        }
    }

    None
}

const SCALE_PATTERNS: &[(ScaleQuality, &[Pc], &[Pc])] = &[
    // (Quality, Required PCs, Optional PCs)
    (ScaleQuality::HarmonicMinor,
     &[Pc::Pc0, Pc::Pc3, Pc::Pc7, Pc::Pc11],  // 1, b3, 5, 7
     &[Pc::Pc2, Pc::Pc5, Pc::Pc8]),            // 2, 4, b6
    // ... more patterns
];
```

### Bug 2: Scale Degree Duplication

**Location:** `name_gen.py`

**Description:** The naming system doesn't handle cases where a chord has two versions of the same scale degree (e.g., both natural 11 and #11).

```python
# Python fails on:
pcs = (0, 4, 7, 10, 5, 6)  # Contains both 11 (5) and #11 (6)
# Returns: "Dom7#11" (loses natural 11)
# Should: Indicate both or flag as ambiguous
```

**Rust Fix Suggestion:**

```rust
// In quality/chord.rs
#[derive(Debug, Clone)]
pub struct ChordQuality {
    pub base: BaseQuality,
    pub extensions: Vec<Extension>,
    pub alterations: Alt,
    pub ambiguities: Vec<QualityAmbiguity>,  // NEW
}

#[derive(Debug, Clone)]
pub enum QualityAmbiguity {
    DuplicateScaleDegree { degree: u8, versions: Vec<Pc> },
    MultipleInterpretations(Vec<ChordQuality>),
}

impl ChordQuality {
    pub fn from_pc_set(
        pc_set: &PcSet,
        root: &Note
    ) -> Result<Self, MusicSemanticsError> {
        let mut quality = Self::infer_base(pc_set, root)?;

        // Check for duplicate scale degrees
        let degree_map = Self::map_pcs_to_degrees(pc_set, root);
        for (degree, pcs) in degree_map.iter() {
            if pcs.len() > 1 {
                quality.ambiguities.push(
                    QualityAmbiguity::DuplicateScaleDegree {
                        degree: *degree,
                        versions: pcs.clone(),
                    }
                );
            }
        }

        Ok(quality)
    }

    fn map_pcs_to_degrees(
        pc_set: &PcSet,
        root: &Note
    ) -> HashMap<u8, Vec<Pc>> {
        let root_pc = Pc::from(root);
        let mut map: HashMap<u8, Vec<Pc>> = HashMap::new();

        for pc in &pc_set.0 {
            let interval = root_pc.distance_up_to(pc);
            let degree = Self::interval_to_degree(interval);
            map.entry(degree).or_default().push(pc.clone());
        }

        map
    }

    fn interval_to_degree(interval: u8) -> u8 {
        match interval {
            0 => 1,
            1 | 2 => 9,    // b9, 9
            3 | 4 => 3,    // b3, 3
            5 | 6 => 11,   // 11, #11
            7 => 5,
            8 | 9 => 13,   // b13, 13
            10 | 11 => 7,  // b7, 7
            _ => unreachable!()
        }
    }
}
```

### Bug 3: Maj6/b9 Misidentification

**Location:** `name_gen.py`

**Description:** Chords with 6th and b9 are incorrectly analyzed due to the 6th being confused with 13th.

```python
# Python incorrectly identifies:
# C E G A Db = [0, 4, 7, 9, 1]
# Returns: "Maj13(b9)" with omitted 11th
# Should: "Maj6(b9)" or "Maj6(addb9)"
```

**Analysis:** The issue is that 6th chords shouldn't be promoted to 13th chords unless a 7th is present.

**Rust Fix Suggestion:**

```rust
// In naming_heuristics/alts_and_extensions.rs
pub fn determine_extensions(
    pc_set: &PcSet,
    base: &BaseQuality
) -> (Vec<Extension>, Alt) {
    let has_seventh = pc_set.contains_any(&[Pc::Pc10, Pc::Pc11]);
    let has_sixth = pc_set.0.contains(&Pc::Pc9);
    let has_ninth = pc_set.contains_any(&[Pc::Pc1, Pc::Pc2, Pc::Pc3]);

    // KEY FIX: 6th without 7th should NOT become 13th
    if has_sixth && !has_seventh {
        // This is a 6th chord, not a 13th chord
        return classify_sixth_chord(pc_set, base);
    }

    // Standard extension logic for 7th chords
    if has_seventh {
        return classify_extended_chord(pc_set, base);
    }

    // Triad with added tones
    classify_add_chord(pc_set, base)
}

fn classify_sixth_chord(
    pc_set: &PcSet,
    base: &BaseQuality
) -> (Vec<Extension>, Alt) {
    let mut alts = Vec::new();

    // Check for added tensions over 6th chord
    if pc_set.0.contains(&Pc::Pc1) {
        alts.push(AltChoice::FlatNine);
    }
    if pc_set.0.contains(&Pc::Pc2) {
        alts.push(AltChoice::Nine);  // 6/9 chord
    }

    // Return 6th as special extension, not 13th
    (vec![Extension::Sixth], Alt(alts))
}
```

### Bug 4: Voiceleading Enharmonic Bug

**Location:** `voiceleading.py`

**Description:** The `path()` method uses `list.index()` which fails when enharmonic equivalents are involved.

```python
# voiceleading.py
def path(self, num):
    """Get the path for voice number."""
    # BUG: This fails if ch2 contains enharmonic equivalent
    idx = self.ch2.spelling.index(self.target_notes[num])
    return self.paths[idx]

# Fails when:
# v1 contains C# but ch2.spelling contains Db
# list.index(C#) raises ValueError because Db != C#
```

**Rust Fix:**

```rust
// In geometry/symmetry/voiceleading.rs
impl Voiceleading {
    /// Get path for voice by index, handling enharmonics
    pub fn path(&self, voice: usize) -> Option<i8> {
        self.paths.get(voice).copied()
    }

    /// Find path to a specific note, matching by pitch class
    pub fn path_to_note(&self, note: &Note) -> Option<i8> {
        let target_pc = Pc::from(note);

        for (i, dest_pitch) in self.destinations.0.iter().enumerate() {
            let dest_pc = Pc::from(&dest_pitch.note);
            if dest_pc == target_pc {
                return self.paths.get(i).copied();
            }
        }

        None
    }
}
```

### Bug 5: Fretboard Shape CMaj/9 Missing

**Location:** `gtr_shapes.py`

**Description:** Certain valid shapes are not found by the search algorithm.

```python
# Documented in comments:
# CMaj/9 shape ['x',2,3,0,0,'x'] not indexed by algorithm
# Shape: x-2-3-0-0-x (C shape with open D and G)
```

**Analysis:** The algorithm fails when searching for notes that could be at fret 0 but the algorithm only checks higher positions.

**Rust Fix:**

```rust
// In fretboard_shape/chord_shape_search.rs
fn find_positions_for_note(
    note: &Note,
    fretboard: &Fretboard
) -> Vec<(u8, u8)> {
    let mut positions = Vec::new();
    let target_pc = Pc::from(note);

    for string in 0..fretboard.num_strings() {
        let open = fretboard.get_string(string).unwrap();
        let open_pc = Pc::from(&open.note);
        let base_fret = open_pc.distance_up_to(&target_pc);

        // FIX: Always include base position (including fret 0)
        if base_fret <= fretboard::MAX {
            positions.push((string, base_fret));
        }

        // Also check octave above (fret 12+ relative)
        let high_fret = base_fret + 12;
        if high_fret <= fretboard::MAX {
            positions.push((string, high_fret));
        }

        // FIX: If base_fret > 12, also check octave below
        if base_fret > 12 {
            let low_fret = base_fret - 12;
            positions.push((string, low_fret));
        }
    }

    positions
}
```

## Naming Heuristic Comparison

### Python: Regex-Based

```python
# name_parser.py
def infer_maj(rest):
    """Parse major chord extensions."""
    match = re.match(r'^(maj|Maj|M|Δ)', rest)
    if match:
        # Set major quality
        return rest[match.end():]
    return rest

def infer_min(rest):
    """Parse minor chord extensions."""
    match = re.match(r'^(min|Min|m|-)', rest)
    # ...
```

### Rust: Pattern Matching

```rust
// naming_heuristics/maj_and_min_qualities.rs
pub fn infer_major_quality(
    pc_set: &PcSet,
    root: &Note
) -> Option<ChordQuality> {
    let has_major_third = pc_set.0.contains(&Pc::Pc4);
    let has_minor_third = pc_set.0.contains(&Pc::Pc3);
    let has_perfect_fifth = pc_set.0.contains(&Pc::Pc7);

    if has_major_third && !has_minor_third {
        if has_perfect_fifth {
            // Major triad base
            return Some(ChordQuality {
                base: BaseQuality::Major,
                extensions: infer_extensions(pc_set),
                alterations: infer_alterations(pc_set, BaseQuality::Major),
                ambiguities: Vec::new(),
            });
        }
    }

    None
}
```

## Extension Style System

The Rust implementation has a more sophisticated extension style system:

```rust
pub enum ExtensionStyle {
    None,               // Everything as alterations
    Strict,             // Must have all lower extensions
    Highest,            // Label by highest extension
    HighestUnlessOne,   // Like Highest unless only 1 extension
}

impl ChordName {
    pub fn with_extension_style(&self, style: ExtensionStyle) -> String {
        match style {
            ExtensionStyle::None => {
                // C7(9,11,13) instead of C13
                self.format_all_as_alterations()
            }
            ExtensionStyle::Strict => {
                // C13 only if 7, 9, 11 present
                self.format_strict_extensions()
            }
            ExtensionStyle::Highest => {
                // C13 if 13 present (most common)
                self.format_highest_extension()
            }
            ExtensionStyle::HighestUnlessOne => {
                // C13 if multiple extensions, else C7(add13)
                self.format_contextual()
            }
        }
    }
}
```

**Suggestion - Add style configuration:**

```rust
#[derive(Debug, Clone)]
pub struct NamingConfig {
    pub extension_style: ExtensionStyle,
    pub prefer_add_notation: bool,       // "add9" vs "9"
    pub show_omissions: bool,            // "no5" notation
    pub slash_chord_threshold: usize,    // Min notes for slash detection
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            extension_style: ExtensionStyle::Highest,
            prefer_add_notation: false,
            show_omissions: true,
            slash_chord_threshold: 4,
        }
    }
}

impl ChordName {
    pub fn format(&self, config: &NamingConfig) -> String {
        // Use config to control output format
    }
}
```

## Slash Chord Detection

### Python Implementation

```python
def detect_slash_chord(pcs, root):
    """Check if this is an inversion or slash chord."""
    bass_pc = pcs[0]  # Assuming first is bass

    if bass_pc == root.pc:
        return None  # Root position

    # Check if it's an inversion of a known chord
    for mode_idx, mode in enumerate(modes_from_pitch_set(pcs)):
        if mode_idx == 0:
            continue
        parent_name = lookup_chord_name(mode)
        if parent_name:
            parent_root = root.note_given_distance(-mode_idx)
            return SlashChord(parent_root, root)

    return None
```

### Rust Implementation

```rust
// chord_name/mod.rs
impl ChordName {
    pub fn detect_slash_chord(
        pc_set: &PcSet,
        bass: &Note,
        config: &NamingConfig
    ) -> Option<TonalSpecification> {
        let bass_pc = Pc::from(bass);

        // Try each mode as potential root position
        for (mode_idx, mode) in pc_set.modes().iter().enumerate() {
            if mode_idx == 0 {
                continue;  // Skip if bass is root
            }

            // Check if this mode matches a known chord type
            if let Some(quality) = Self::lookup_quality(&mode) {
                // Calculate the actual root
                let root_pc = bass_pc.transpose(-(mode_idx as i8));
                if let Some(root) = Self::pc_to_note(&root_pc, bass) {
                    return Some(TonalSpecification::SlashChord {
                        bass: bass.clone(),
                        root,
                    });
                }
            }
        }

        None
    }
}
```

## Summary: Bug Status

| Bug | Python Status | Rust Status | Severity |
|-----|---------------|-------------|----------|
| Harmonic minor false positive | Unfixed | Needs implementation | High |
| Scale degree duplication | Unfixed | Suggested fix above | Medium |
| Maj6/b9 misidentification | Unfixed | Suggested fix above | High |
| Voiceleading enharmonic | Unfixed | Fixed in design | High |
| CMaj/9 shape missing | Unfixed | Suggested fix above | Medium |
| Extension style rigidity | N/A | Has ExtensionStyle enum | Low |

## Recommendations for Rust Implementation

1. **Implement all naming heuristic modules** in `naming_heuristics/`:
   - Complete `maj_and_min_qualities.rs`
   - Complete `dim_qualities.rs`
   - Complete `aug_qualities.rs`
   - Complete `sus_qualities.rs`
   - Complete `scale_qualities.rs`
   - Complete `alts_and_extensions.rs`

2. **Add comprehensive tests** for edge cases:
   ```rust
   #[test]
   fn test_maj6_vs_maj13() {
       let maj6 = pcs!(0, 4, 7, 9);  // C E G A
       let maj13 = pcs!(0, 4, 7, 10, 2, 9);  // C E G Bb D A

       let name6 = ChordName::from_pc_set(&maj6, &Note::C);
       let name13 = ChordName::from_pc_set(&maj13, &Note::C);

       assert!(name6.quality.base == BaseQuality::Major);
       assert!(name6.quality.extensions.contains(&Extension::Sixth));

       assert!(name13.quality.base == BaseQuality::Dominant);
       assert!(name13.quality.extensions.contains(&Extension::Thirteenth));
   }
   ```

3. **Add ambiguity reporting** for chords that could have multiple valid interpretations.

4. **Document all naming rules** with musical examples.
