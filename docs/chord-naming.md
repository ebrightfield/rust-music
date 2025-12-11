# Chord Naming System

The chord naming module provides types and algorithms for describing pitch class sets using standard chord nomenclature.

**Location:** `music::note_collections::chord_name`

## Overview

The naming system consists of:
1. **ChordName** - The complete description of a named chord
2. **ChordQuality** - The tonal flavor (major, minor, augmented, etc.)
3. **TonalSpecification** - Root note and bass information
4. **Naming Heuristics** - Algorithms to infer names from PcSets

## ChordName

A ChordName combines a quality with tonal specification and the underlying PcSet:

```rust
use music::note_collections::chord_name::{ChordName, ChordNameDisplayConfig};

// Given a ChordName instance
let name_string = chord_name.to_string(None);  // Use default config

// Or with custom configuration
let config = ChordNameDisplayConfig::default();
let name_string = chord_name.to_string(Some(&config));
```

## ChordQuality

**Location:** `music::note_collections::chord_name::quality::chord`

ChordQuality is the main enum describing chord types:

```rust
pub enum ChordQuality {
    Major(MajorSubtype),
    Minor(MinorSubtype),
    Aug(AugSubtype),
    Dim(DimSubtype),
    Sus(SusSubtype),
    Interval(IntervalClass),  // Two-note chords
    SingleNote,               // Single pitch class
}
```

### Major Subtypes

```rust
pub enum MajorSubtype {
    Maj(Alt),                    // Major triad
    Maj6(Alt),                   // Major 6th
    MajN(Vec<Extension>, Alt),   // Major 7th, 9th, etc.
    N(Vec<Extension>, Alt),      // Dominant 7th, 9th, etc.
}
```

Examples:
- `Maj` → C (major triad)
- `Maj6` → C6
- `MajN([Seventh])` → CMaj7
- `N([Seventh])` → C7 (dominant)
- `MajN([Seventh, Ninth])` → CMaj9

### Minor Subtypes

```rust
pub enum MinorSubtype {
    Min(Alt),                       // Minor triad
    Min6(Alt),                      // Minor 6th
    MinMajN(Vec<Extension>, Alt),   // Minor-Major 7th
    MinN(Vec<Extension>, Alt),      // Minor 7th, 9th, etc.
}
```

Examples:
- `Min` → Cm
- `Min6` → Cm6
- `MinMajN([Seventh])` → CmMaj7
- `MinN([Seventh])` → Cm7

### Augmented Subtypes

```rust
pub enum AugSubtype {
    Aug(Alt),                      // Augmented triad
    AugMajN(Vec<Extension>, Alt),  // Aug-Major 7th
    AugN(Vec<Extension>, Alt),     // Aug 7th
}
```

Examples:
- `Aug` → C+
- `AugMajN([Seventh])` → C+Maj7
- `AugN([Seventh])` → C+7

### Diminished Subtypes

```rust
pub enum DimSubtype {
    Dim(Alt),                     // Diminished triad
    MinNb5(Vec<Extension>, Alt),  // Half-diminished (m7b5)
    DimN(Vec<Extension>, Alt),    // Fully diminished 7th
    DimMajN(Vec<Extension>, Alt), // Diminished-Major 7th (rare)
}
```

Examples:
- `Dim` → Cdim
- `MinNb5([Seventh])` → Cm7b5
- `DimN([Seventh])` → Cdim7

### Sus Subtypes

```rust
pub enum SusSubtype {
    Sus2(Alt),                     // Suspended 2nd
    Sus4(Alt),                     // Suspended 4th
    DomNSus(Vec<Extension>, Alt),  // 7sus, 9sus, etc.
    MajNSus(Vec<Extension>, Alt),  // Maj7sus, etc.
    SixNineSus(Alt),               // 6/9sus
}
```

## Extensions and Alterations

### Extension

Extensions are the 7th, 9th, 11th, and 13th:

```rust
pub enum Extension {
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
}
```

### Alt (Alterations)

Alterations modify extensions:

```rust
pub enum AltChoice {
    FlatNine,
    Nine,
    SharpNine,
    FlatEleven,
    Eleven,
    SharpEleven,
    FlatThirteenth,
    Thirteenth,
    SharpThirteenth,
}
```

### Extension Styles

The `ExtensionStyle` enum controls how extensions are displayed:

```rust
pub enum ExtensionStyle {
    None,              // Always show as 7th + alterations
    Strict,            // 13th requires 11th and 9th present
    Highest,           // Use highest extension present
    HighestUnlessOne,  // Use highest unless only one extra extension
}
```

Example with a chord containing 7th, 9th, and 13th:
- `None` → "7(9, 13)"
- `Strict` → "7(9, 13)" (strict requires 11th for 13th label)
- `Highest` → "13"
- `HighestUnlessOne` → "13"

## TonalSpecification

Specifies the root and bass relationship:

```rust
pub enum TonalSpecification {
    SlashChord { bass: Note, root: Note },  // C/E (C over E bass)
    RootPosition(Note),                      // Standard root
    None(Option<Pc>),                        // No tonal center specified
}
```

## Display Configuration

`ChordNameDisplayConfig` provides fine-grained control:

```rust
pub struct ChordNameDisplayConfig {
    pub explicit_sus4: bool,                    // Show "sus4" vs "sus"
    pub utf8_accidentals: bool,                 // Use ♯/♭ vs #/b
    pub space_between_root_and_quality: usize,  // "C Maj7" vs "CMaj7"
    pub space_between_quality_and_slash: usize, // "C /E" vs "C/E"
    pub space_after_slash: usize,               // "C/ E" vs "C/E"
    pub extension_style: ExtensionStyle,        // How to format extensions
}
```

## Naming Heuristics

**Location:** `music::note_collections::chord_name::naming_heuristics`

The library includes algorithms to infer chord names from PcSets. These are organized by chord family:

- `alts_and_extensions` - Handle extensions and alterations
- `aug_qualities` - Augmented chord detection
- `dim_qualities` - Diminished chord detection
- `inferred_third_qualities` - Chords with implied thirds
- `maj_and_min_qualities` - Major/minor chord detection
- `scale_qualities` - Scale-like structures
- `sus_qualities` - Suspended chord detection

## musical-combinatorics Crate

The companion `musical-combinatorics` crate provides exhaustive enum types for chord qualities:

### ThreeNoteChordQuality

All possible three-note chord types (triads).

### FourNoteChordQuality

Exhaustive enumeration of all four-note chord types:

```rust
pub enum FourNoteChordQuality {
    // Seventh Chords
    Maj7, Dom7, Min7, MinMaj7, Dim7, Min7Flat5, Aug7, AugMaj7, Dom7Flat5,

    // Triads + 9th
    Maj9, MinFlat9, MajFlat9, MajSharp9, Min9, Dim9, DimFlat9,

    // Triads + 11th
    Maj11, MajSharp11, Min11, MinSharp11, Dim11, DimFlat11,

    // Stacked Fourths
    PPP, APP, PAP, PPA,

    // Clusters
    WWW, HWW, WHW, WWH, HAH, AHH, HHA, HWH, WHH, HHW, HHM, MHH, HAW, WAH, HHH,

    // Perfect Fourths + Half steps
    PHP, PPH,
}
```

Usage:

```rust
use musical_combinatorics::FourNoteChordQuality;

// Identify a chord quality with its inversion
let (inversion, quality) = FourNoteChordQuality::identify(&pc_set).unwrap();
```

### SevenNoteScaleQuality

Exhaustive enumeration of seven-note scales (modes).

## Predefined PcSets

The `musical-combinatorics` crate exports predefined PcSets for common chords:

```rust
pub const MAJ7_PCS: &[Pc] = &[Pc0, Pc4, Pc7, Pc11];
pub const DOM7_PCS: &[Pc] = &[Pc0, Pc4, Pc7, Pc10];
pub const MIN7_PCS: &[Pc] = &[Pc0, Pc3, Pc7, Pc10];
// ... many more
```
