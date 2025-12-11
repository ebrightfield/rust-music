# Complete Bug Registry

This document provides an exhaustive list of all bugs found in the Python `pitch_set_lib` library, with exact line numbers, code snippets, and suggested Rust implementations.

## Critical Bugs (Severity: HIGH)

### Bug #1: Seven-Note Scale Matching Fails with Doubled Extensions

**File:** `name_gen.py`
**Lines:** 21-46
**Status:** Unfixed in Python

**Problem:** The scale and alteration matching code fails when a chord has two versions of an extension (e.g., both natural and sharp 11) but is missing another extension (e.g., no 9th).

**Documented Examples in Source:**
```python
# Lines 37-46 in name_gen.py
#'Maj6/b9': {Chord(C, pitch_set=(0, 1, 2, 4, 5, 8, 9)),
#        Chord(C, pitch_set=(0, 1, 2, 4, 6, 8, 9)),
#        Chord(C, pitch_set=(0, 1, 3, 4, 6, 7, 9))}
# The ones above are very wrong, lots of Maj6 ones are whack,
# they aren't getting multiple 9ths correctly
# 'Maj6(b9,11)': {Chord(C, pitch_set=(0, 1, 3, 4, 5, 7, 9))
# This one is also wrong:
# Eb mel min b9 b11 (Eb E F# G Bb C D)
# Should be Eb Maj7(b9,#9,13)
```

**Root Cause:** The `seven_note_scale_heuristics` matching in `seven_note_scale_processing()` (lines 247-251) doesn't verify that required scale degrees are present before matching.

**Rust Fix:**

```rust
// In naming_heuristics/scale_qualities.rs

/// Required and optional pitch classes for scale matching
pub struct ScalePattern {
    pub quality: ScaleQuality,
    pub required: Vec<Pc>,    // MUST all be present
    pub defining: Vec<Pc>,    // At least one must be present (characteristic tones)
    pub optional: Vec<Pc>,    // May be present without triggering alterations
}

pub fn match_seven_note_scale(
    pc_set: &PcSet,
    root: &Note
) -> Option<(ScaleQuality, Vec<AltChoice>)> {
    let pcs: HashSet<Pc> = pc_set.0.iter().cloned().collect();

    for pattern in SCALE_PATTERNS.iter() {
        // 1. ALL required PCs must be present
        let required: HashSet<Pc> = pattern.required.iter().cloned().collect();
        if !required.is_subset(&pcs) {
            continue;
        }

        // 2. At least ONE defining PC must be present
        let has_defining = pattern.defining.iter().any(|pc| pcs.contains(pc));
        if !has_defining {
            continue;
        }

        // 3. Calculate alterations (PCs not in expected set)
        let expected: HashSet<Pc> = pattern.required.iter()
            .chain(pattern.optional.iter())
            .chain(pattern.defining.iter())
            .cloned()
            .collect();

        let alterations: Vec<Pc> = pcs.difference(&expected).cloned().collect();

        // 4. Maximum 2 alterations allowed for scale match
        if alterations.len() <= 2 {
            let alt_choices = alterations.iter()
                .filter_map(|pc| pc_to_alt_choice(pc, root))
                .collect();
            return Some((pattern.quality.clone(), alt_choices));
        }
    }

    None
}

const SCALE_PATTERNS: &[ScalePattern] = &[
    ScalePattern {
        quality: ScaleQuality::HarmonicMinor,
        required: vec![Pc::Pc0, Pc::Pc3, Pc::Pc7],     // 1, b3, 5
        defining: vec![Pc::Pc11],                      // maj7 (characteristic)
        optional: vec![Pc::Pc2, Pc::Pc5, Pc::Pc8],     // 2, 4, b6
    },
    ScalePattern {
        quality: ScaleQuality::MelodicMinor,
        required: vec![Pc::Pc0, Pc::Pc3, Pc::Pc7],     // 1, b3, 5
        defining: vec![Pc::Pc9, Pc::Pc11],             // 6, maj7 (both characteristic)
        optional: vec![Pc::Pc2, Pc::Pc5],              // 2, 4
    },
    // ... more patterns
];
```

---

### Bug #2: Syntax Error in name_parser.py

**File:** `name_parser.py`
**Line:** 335
**Status:** Unfixed in Python (SYNTAX BUG)

**Problem:** Float literal `8.9` used instead of list `[8, 9]`.

**Original Code:**
```python
# Line 335 - WRONG!
elif pc in [8.9]:  # This is the float 8.9, not [8, 9]!
    redundant_thirteenth_err = True
```

**Impact:** Pitch classes 8 and 9 are never matched for redundant thirteenth detection.

**Rust Fix:**

```rust
// In chord_name/naming_heuristics/alts_and_extensions.rs

fn check_redundant_thirteenth(pc: &Pc) -> bool {
    matches!(pc, Pc::Pc8 | Pc::Pc9)  // Correctly matches both
}
```

---

### Bug #3: Voiceleading Enharmonic Crash

**File:** `voiceleading.py`
**Lines:** 19, 28-29, 57
**Status:** Unfixed in Python

**Problem:** The `.index()` method fails when enharmonic spellings differ between source and target.

**Original Code:**
```python
# Line 57
for p in v1.pitches:
    # The paths should be ordered according to chord spelling
    index = v1.chord.spelling.index(p)  # CRASH if p is enharmonic!
    new_spelling.append(p + paths[index])
```

**Failure Case:**
```python
# Source voicing has C# but target chord spelling has Db
# C# and Db are enharmonically equivalent but list.index() doesn't know that
v1.chord.spelling.index(Pitch("C#4"))  # Raises ValueError if spelling has Db
```

**Rust Fix:**

```rust
// In geometry/symmetry/voiceleading.rs

impl Voiceleading {
    /// Find voice index using pitch class matching (enharmonic-safe)
    fn find_voice_index(&self, pitch: &Pitch) -> Option<usize> {
        let target_pc = Pc::from(&pitch.note);

        self.departures.0.iter().position(|p| {
            Pc::from(&p.note) == target_pc
        })
    }

    /// Get path by pitch (enharmonic-safe lookup)
    pub fn path_for_pitch(&self, pitch: &Pitch) -> Option<i8> {
        self.find_voice_index(pitch)
            .and_then(|idx| self.paths.get(idx).copied())
    }

    /// Apply voiceleading paths to create destination voicing
    pub fn apply(&self) -> Result<Voicing, MusicSemanticsError> {
        let mut dest_pitches = Vec::with_capacity(self.departures.len());

        for (i, departure) in self.departures.0.iter().enumerate() {
            let path = self.paths.get(i)
                .ok_or(MusicSemanticsError::MismatchedCollectionSize)?;

            let dest = departure.at_distance_from(*path as isize)?;
            dest_pitches.push(dest);
        }

        Ok(Voicing::new(dest_pitches))
    }
}
```

---

### Bug #4: Voiceleading Equality Check Wrong Attribute

**File:** `voiceleading.py`
**Line:** 33
**Status:** Unfixed in Python

**Problem:** Equality comparison uses `other.v2` when it should use `other.ch2`.

**Original Code:**
```python
def __eq__(self, other):
    return (self.v1 == other.v1 and\
            self.ch2 == other.v2 and\  # WRONG! Should be other.ch2
            self.paths == other.paths)
```

**Impact:** Two voiceleadings that go to different chords may incorrectly compare as equal.

**Rust Fix:**

```rust
impl PartialEq for Voiceleading {
    fn eq(&self, other: &Self) -> bool {
        self.departures == other.departures
            && self.destinations == other.destinations  // Correct comparison
            && self.paths == other.paths
    }
}
```

---

### Bug #5: CMaj/9 and High-Fret Shapes Not Found

**File:** `gtr_shapes.py`
**Lines:** 12-14, 203
**Status:** Documented but unfixed in Python

**Problem:** The shape search algorithm filters out shapes where all frets are >= 12.

**Original Code:**
```python
# Lines 12-14 - Documented bug
# TODO I THINK I FOUND A GTR CHORD THAT ISNT INDEXED VIA THIS ALGORITHM!  CMaj/9:  12 x 12 0 13 x
# The logic that misses it I think has to do with everything being 12th fret or higher.

# Line 203 - The filter that causes the bug
possible_combinations_of_frets = [list(val) for val in product(*frets)
                                  if any(f.fret < 12 for f in val)]  # EXCLUDES all-high shapes!
```

**Impact:** Valid high-position shapes like `12-x-12-0-13-x` are never discovered.

**Rust Fix:**

```rust
// In fretboard/fretboard_shape/chord_shape_search.rs

impl<'a> FretboardShape<'a> {
    pub fn search_chord_shapes(
        notes: &[Note],
        fretboard: &'a Fretboard,
        options: &SearchOptions,
    ) -> Vec<Self> {
        let mut results = Vec::new();

        // Find all positions for each note
        let positions: Vec<Vec<(u8, u8)>> = notes.iter()
            .map(|note| Self::find_all_positions_for_note(note, fretboard))
            .collect();

        // Generate all combinations (no filter on fret height!)
        Self::search_recursive(
            fretboard,
            &positions,
            0,
            Vec::new(),
            &mut results,
            options,
        );

        results
    }

    fn find_all_positions_for_note(
        note: &Note,
        fretboard: &Fretboard
    ) -> Vec<(u8, u8)> {
        let mut positions = Vec::new();
        let target_pc = Pc::from(note);

        for string in 0..fretboard.num_strings() {
            let open = fretboard.get_string(string).unwrap();
            let open_pc = Pc::from(&open.note);
            let base_fret = open_pc.distance_up_to(&target_pc);

            // Include fret 0 (open string) when applicable
            if base_fret <= fretboard::MAX {
                positions.push((string, base_fret));
            }

            // Include octave higher (fret + 12)
            let high_fret = base_fret + 12;
            if high_fret <= fretboard::MAX {
                positions.push((string, high_fret));
            }

            // Include octave lower when base > 12
            if base_fret > 12 {
                positions.push((string, base_fret - 12));
            }
        }

        positions
    }
}

pub struct SearchOptions {
    pub include_high_position: bool,  // Default: true (don't filter >= 12)
    pub max_span: u8,                 // Default: 4
    pub include_open_strings: bool,   // Default: true
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            include_high_position: true,  // FIX: Include high-fret shapes
            max_span: 4,
            include_open_strings: true,
        }
    }
}
```

---

## Medium Bugs (Severity: MEDIUM)

### Bug #6: `only_one()` Function Fails with Pitch Class 0

**File:** `name_gen.py`
**Lines:** 50-67
**Status:** Documented but unfixed in Python

**Problem:** Function uses `False` as sentinel, but pitch class 0 is falsy in Python.

**Original Code:**
```python
def only_one(collection, should_have_one):
    match = False  # Sentinel value
    for i in collection:
        if i in should_have_one:
            if not match:  # BUG: `if not 0` is True!
                match = i
            else:
                return False
    return match
```

**Failure Case:**
```python
only_one([0, 4, 7], {0, 4})  # Returns False instead of detecting conflict
# Because match = 0, and `if not 0` evaluates to True
```

**Rust Fix:**

```rust
/// Check if exactly one element from `candidates` appears in `collection`
/// Returns Some(element) if exactly one, None if zero or multiple
pub fn only_one<T: Eq + Clone>(
    collection: &[T],
    candidates: &HashSet<T>
) -> Option<T> {
    let mut found: Option<T> = None;

    for item in collection {
        if candidates.contains(item) {
            match &found {
                None => found = Some(item.clone()),
                Some(_) => return None,  // Multiple matches
            }
        }
    }

    found
}
```

---

### Bug #7: Multiple Alterations Only Returns First

**File:** `name_gen.py`
**Line:** 204
**Status:** Unfixed in Python

**Problem:** Early return in loop only outputs first alteration set.

**Original Code:**
```python
if len(alt_strs) == 1:
    return sep + alt_strings[alt_strs[0]]
else:
    for s in alt_strs:
        return '(' + ','.join(alt_strings[i] for i in alt_strs) + ')'  # Returns on FIRST iteration!
```

**Should Be:**
```python
else:
    return '(' + ','.join(alt_strings[i] for i in alt_strs) + ')'  # No loop needed
```

**Rust Fix:**

```rust
pub fn format_alterations(alts: &[AltChoice]) -> String {
    if alts.is_empty() {
        return String::new();
    }

    let alt_strs: Vec<&str> = alts.iter()
        .map(|alt| alt.to_str())
        .collect();

    if alt_strs.len() == 1 {
        alt_strs[0].to_string()
    } else {
        format!("({})", alt_strs.join(","))
    }
}
```

---

### Bug #8: Scale Degree Filling Incomplete

**File:** `name_parser.py`
**Lines:** 372-377
**Status:** Partial implementation in Python

**Problem:** Default scale degree filling doesn't handle all cases correctly.

**Original Code:**
```python
# Lines 372-377 - Fills in missing degrees
if not any(pc in inferred_pitches for pc in [1, 2]):
    inferred_pitches.set(2, True)   # Add natural 9th
if not any(pc in inferred_pitches for pc in [5, 6]):
    inferred_pitches.set(5, True)   # Add natural 11th
if not any(pc in inferred_pitches for pc in [8, 9]):
    inferred_pitches.set(9, True)   # Add natural 13th
```

**Issue:** This logic doesn't account for:
- Chords that explicitly omit certain degrees
- Context where natural degree would conflict with chord quality
- 6th vs 13th distinction (6th doesn't imply 9th/11th)

**Rust Fix:**

```rust
pub fn fill_default_scale_degrees(
    pc_set: &mut HashSet<Pc>,
    base_quality: &BaseQuality,
    has_seventh: bool,
) {
    // Only fill degrees for extended chords (7th+)
    if !has_seventh {
        return;
    }

    // 9th: Fill with natural 9 if no 9th variant present
    let has_ninth = pc_set.contains(&Pc::Pc1)
        || pc_set.contains(&Pc::Pc2)
        || pc_set.contains(&Pc::Pc3);
    if !has_ninth {
        pc_set.insert(Pc::Pc2);  // Natural 9
    }

    // 11th: Fill based on quality (avoid contradictions)
    let has_eleventh = pc_set.contains(&Pc::Pc5) || pc_set.contains(&Pc::Pc6);
    if !has_eleventh {
        match base_quality {
            BaseQuality::Major | BaseQuality::Dominant => {
                // Major: #11 is common, natural 11 conflicts with 3rd
                // Don't auto-fill - let it be omitted
            }
            _ => {
                pc_set.insert(Pc::Pc5);  // Natural 11
            }
        }
    }

    // 13th: Fill with natural 13 if no 13th variant present
    let has_thirteenth = pc_set.contains(&Pc::Pc8) || pc_set.contains(&Pc::Pc9);
    if !has_thirteenth {
        pc_set.insert(Pc::Pc9);  // Natural 13
    }
}
```

---

### Bug #9: Contradictory Eleventh Exception Too Narrow

**File:** `name_parser.py`
**Lines:** 362-365
**Status:** Inflexible implementation in Python

**Problem:** Special case for #11 only applies to major chords, but dominant chords also commonly have #11.

**Original Code:**
```python
if contradictory_eleventh_err:
    if not (6 in alt_pcs and 'major' in quality):  # Only major!
        raise ValueError(...)
    scale.remove(5)  # Mutates input list
```

**Rust Fix:**

```rust
pub fn handle_contradictory_eleventh(
    pc_set: &PcSet,
    base_quality: &BaseQuality,
) -> Result<Vec<Pc>, MusicSemanticsError> {
    let has_natural_11 = pc_set.0.contains(&Pc::Pc5);
    let has_sharp_11 = pc_set.0.contains(&Pc::Pc6);

    if has_natural_11 && has_sharp_11 {
        // Both present - this is ambiguous
        return Err(MusicSemanticsError::AmbiguousChordQuality(
            "Both natural and sharp 11 present".to_string()
        ));
    }

    if has_sharp_11 && !has_natural_11 {
        // #11 is acceptable in Major, Dominant, and some other qualities
        match base_quality {
            BaseQuality::Major
            | BaseQuality::Dominant
            | BaseQuality::Lydian => {
                // #11 is characteristic - no conflict
                Ok(pc_set.0.clone())
            }
            BaseQuality::Minor | BaseQuality::MinorMajor => {
                // Less common but valid (melodic minor derivatives)
                Ok(pc_set.0.clone())
            }
            _ => {
                Err(MusicSemanticsError::InvalidChordConstruction(
                    format!("#11 unusual in {:?} quality", base_quality)
                ))
            }
        }
    } else {
        Ok(pc_set.0.clone())
    }
}
```

---

## Low Bugs (Severity: LOW)

### Bug #10: Empty Shape Error Message Confusing

**File:** `gtr_shapes.py`
**Lines:** 37-38
**Status:** Minor issue in Python

**Problem:** Error message says "non-empty" when it means "all-muted."

**Original Code:**
```python
if all(f == 'x' for f in self.shape):
    raise ValueError("Cannot instantiate GtrShape with a non-empty shape")
    # Should say: "Cannot instantiate GtrShape with all strings muted"
```

**Rust Fix:**

```rust
impl<'a> FretboardShape<'a> {
    pub fn new(
        fretted_notes: Vec<FrettedNote<'a>>,
        fretboard: &'a Fretboard
    ) -> Result<Self, MusicSemanticsError> {
        // Check for at least one sounded note
        let has_sounded = fretted_notes.iter()
            .any(|f| matches!(f, FrettedNote::Sounded(_)));

        if !has_sounded {
            return Err(MusicSemanticsError::InvalidChordConstruction(
                "FretboardShape must have at least one sounded note".to_string()
            ));
        }

        Ok(Self { fretted_notes, fretboard })
    }
}
```

---

### Bug #11: Debug Print Statements Left in Production Code

**File:** `rhythm.py`
**Lines:** 62, 69, 75-76, 81-82, 89-90, 313
**Status:** Unprofessional but harmless

**Problem:** Print statements scattered throughout production code.

**Original Code:**
```python
# Line 62
print(f"big_beat_durations: {big_beat_durations}")
# Line 69
print(f"Running total: {running_total}")
# etc.
```

**Rust Approach:** Use the `log` crate with appropriate log levels:

```rust
use log::{debug, trace};

fn calculate_big_beats(meter: &Meter) -> Vec<Duration> {
    debug!("Calculating big beats for meter: {:?}", meter);

    let durations = // ... calculation

    trace!("big_beat_durations: {:?}", durations);
    durations
}
```

---

## Summary Table

| # | File | Line(s) | Severity | Description | Rust Status |
|---|------|---------|----------|-------------|-------------|
| 1 | name_gen.py | 21-46 | HIGH | Seven-note scale matching fails | Needs implementation |
| 2 | name_parser.py | 335 | HIGH | Syntax error: `[8.9]` → `[8, 9]` | N/A (design avoids) |
| 3 | voiceleading.py | 57 | HIGH | Enharmonic crash in `.index()` | Fixed by design |
| 4 | voiceleading.py | 33 | MEDIUM | Wrong attribute in `__eq__` | Fixed by design |
| 5 | gtr_shapes.py | 203 | HIGH | High-fret shapes filtered out | Needs fix |
| 6 | name_gen.py | 50-67 | MEDIUM | `only_one()` fails on PC 0 | Suggested fix above |
| 7 | name_gen.py | 204 | MEDIUM | Multiple alts returns first only | Suggested fix above |
| 8 | name_parser.py | 372-377 | MEDIUM | Incomplete scale degree filling | Suggested fix above |
| 9 | name_parser.py | 362-365 | MEDIUM | #11 exception too narrow | Suggested fix above |
| 10 | gtr_shapes.py | 37-38 | LOW | Confusing error message | Suggested fix above |
| 11 | rhythm.py | multiple | LOW | Debug print statements | Use `log` crate |
