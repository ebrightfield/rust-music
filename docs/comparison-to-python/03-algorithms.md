# Algorithm Comparison

This document compares the core algorithms between Python `pitch_set_lib` and Rust `music`.

## Pitch Set Normalization

### Python Implementation

```python
# chord_transformations.py
def pitch_set_sanitize(pcs):
    """Normalize pitch set: mod-12, deduplicate, sort."""
    return tuple(sorted(set(pc % 12 for pc in pcs)))

def pitch_set_to_int_row(pitch_set):
    """Convert pitch set to interval row (intervals between adjacent pcs)."""
    pitch_set = pitch_set_sanitize(pitch_set)
    return tuple(
        (pitch_set[(i+1) % len(pitch_set)] - pitch_set[i]) % 12
        for i in range(len(pitch_set))
    )
```

**Issues:**
- Sanitization is optional - easy to forget
- No "zeroing" (transposing so lowest PC is 0)
- Returns tuple of integers, not typed

### Rust Implementation

```rust
// pc_set.rs
impl PcSet {
    pub fn new(pcs: Vec<Pc>) -> Self {
        // 1. Deduplicate
        let mut deduped: Vec<Pc> = pcs.into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 2. Sort
        deduped.sort();

        // 3. Zero (transpose so first element is Pc0)
        if let Some(first) = deduped.first() {
            let offset = 12 - u8::from(first);
            deduped = deduped.iter()
                .map(|pc| pc.transpose(offset as i8))
                .collect();
        }

        Self(deduped)
    }
}
```

**Rust Advantages:**
- Cannot create un-normalized PcSet
- Zeroing ensures canonical form for comparison
- Type-safe throughout

**Suggestion - Add `from_unzeroed` for cases where zeroing is undesirable:**

```rust
impl PcSet {
    /// Create PcSet without zeroing (for transposed analysis)
    pub fn from_unzeroed(pcs: Vec<Pc>) -> Self {
        let mut deduped: Vec<Pc> = pcs.into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        deduped.sort();
        Self(deduped)
    }
}
```

## Mode Generation

### Python Implementation

```python
def rotate_pitch_set(pitch_set, degrees):
    """Rotate pitch set by N degrees and re-zero."""
    pitch_set = pitch_set_sanitize(pitch_set)
    rotated = pitch_set[degrees:] + pitch_set[:degrees]
    # Re-zero to first element
    offset = rotated[0]
    return tuple((pc - offset) % 12 for pc in rotated)

def modes_from_pitch_set(pitch_set):
    """Generate all modes (rotations) of a pitch set."""
    pitch_set = pitch_set_sanitize(pitch_set)
    return [rotate_pitch_set(pitch_set, i) for i in range(len(pitch_set))]
```

### Rust Implementation

```rust
// geometry/symmetry/transpositional.rs
pub trait Modes {
    fn modes(&self) -> Vec<Self> where Self: Sized;
    fn is_mode(&self, other: &Self) -> Option<usize>;
}

impl Modes for PcSet {
    fn modes(&self) -> Vec<Self> {
        (0..self.len())
            .map(|i| self.rotate(i))
            .collect()
    }

    fn is_mode(&self, other: &Self) -> Option<usize> {
        self.modes()
            .iter()
            .position(|mode| mode == other)
    }
}

impl PcSet {
    pub fn rotate(&self, times: usize) -> Self {
        let n = self.len();
        if n == 0 { return self.clone(); }

        let times = times % n;
        let rotated: Vec<Pc> = self.0[times..]
            .iter()
            .chain(self.0[..times].iter())
            .cloned()
            .collect();

        PcSet::new(rotated)  // Re-zeros automatically
    }
}
```

**Rust Advantage:** Trait-based design enables generic mode operations.

## Spelling Algorithm

### Python Implementation

```python
# chord_transformations.py
def root_and_pcs_to_spelling(root, pcs, context=None):
    """
    Spell pitch classes given a root note.
    Uses DEFAULT_SPELLINGS + SPELL_RULE_DICT for context-aware correction.
    """
    root_name = root.name if hasattr(root, 'name') else root
    default = [DEFAULT_SPELLINGS[root_name][pc] for pc in pcs]

    # Apply spelling rules
    if root_name in SPELL_RULE_DICT:
        for rule in SPELL_RULE_DICT[root_name]:
            pc, incl, excl, replacement = rule
            if pc in pcs:
                if all(i in pcs for i in incl) and not any(e in pcs for e in excl):
                    idx = list(pcs).index(pc)
                    default[idx] = replacement

    return [Note(n) for n in default]
```

**Python Issues:**
1. Rules stored as tuples - hard to understand
2. No validation of rule correctness
3. Silent fallback if context missing

### Rust Implementation

```rust
// note_collections/spelling.rs
pub struct SpellingRule {
    pub pc: Pc,
    pub incl: Vec<Pc>,      // Required pitch classes
    pub excl: Vec<Pc>,      // Excluded pitch classes
    pub not_all: Vec<Pc>,   // Cannot have ALL of these
}

pub fn spell_pc_set(root: &Note, pcs: &[Pc]) -> Result<Vec<Note>, MusicSemanticsError> {
    // Get default spellings for this root
    let defaults = get_default_spellings(root);

    let mut result: Vec<Note> = pcs.iter()
        .map(|pc| defaults[pc].clone())
        .collect();

    // Apply context-aware rules
    let rules = get_spelling_rules(root);
    for rule in rules {
        if pcs.contains(&rule.pc) {
            let incl_satisfied = rule.incl.iter().all(|pc| pcs.contains(pc));
            let excl_satisfied = !rule.excl.iter().any(|pc| pcs.contains(pc));
            let not_all_satisfied = !rule.not_all.iter().all(|pc| pcs.contains(pc));

            if incl_satisfied && excl_satisfied && not_all_satisfied {
                if let Some(idx) = pcs.iter().position(|p| p == &rule.pc) {
                    result[idx] = result[idx].enharmonic();
                }
            }
        }
    }

    Ok(result)
}
```

**Rust Advantages:**
1. `SpellingRule` struct is self-documenting
2. Explicit `not_all` field (Python lacks this)
3. Returns `Result` for error handling

**Suggestion - Add rule validation:**

```rust
impl SpellingRule {
    pub fn validate(&self) -> Result<(), MusicSemanticsError> {
        // Ensure incl and excl don't overlap
        for pc in &self.incl {
            if self.excl.contains(pc) {
                return Err(MusicSemanticsError::InvalidSpellingRule(
                    format!("PC {:?} in both incl and excl", pc)
                ));
            }
        }
        Ok(())
    }
}
```

## Transpositional Symmetry Detection

### Python Implementation

```python
def is_symmetrical(pcs):
    """
    Detect rotational symmetry in pitch set.
    Returns the transposition interval if symmetric, else None.
    """
    pcs = pitch_set_sanitize(pcs)
    n = len(pcs)

    for interval in [6, 4, 3, 2]:  # Check T6, T4, T3, T2
        transposed = tuple((pc + interval) % 12 for pc in pcs)
        if pitch_set_sanitize(transposed) == pcs:
            return interval
    return None
```

**Python Issues:**
1. Only returns first symmetry found
2. Doesn't identify which PCs are symmetric
3. Doesn't handle T1 (chromatic cluster)

### Rust Implementation

```rust
// geometry/symmetry/transpositional.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TranspositionalSymmetry {
    T6,  // Tritone (e.g., augmented 4th)
    T4,  // Major 3rd (e.g., augmented triad)
    T3,  // Minor 3rd (e.g., diminished 7th)
    T2,  // Whole tone
    T1,  // Chromatic
}

impl PcSet {
    pub fn transpositional_symmetry(&self) -> HashMap<Pc, HashSet<TranspositionalSymmetry>> {
        let mut result: HashMap<Pc, HashSet<TranspositionalSymmetry>> = HashMap::new();

        for (interval, sym) in [
            (6, TranspositionalSymmetry::T6),
            (4, TranspositionalSymmetry::T4),
            (3, TranspositionalSymmetry::T3),
            (2, TranspositionalSymmetry::T2),
            (1, TranspositionalSymmetry::T1),
        ] {
            let transposed: Vec<Pc> = self.0.iter()
                .map(|pc| pc.transpose(interval))
                .collect();

            // Check if transposition maps set to itself
            if self.is_transposed_version_of(transposed.clone()) {
                // Record which PCs have this symmetry
                for pc in &self.0 {
                    result.entry(pc.clone())
                        .or_insert_with(HashSet::new)
                        .insert(sym.clone());
                }
            }
        }

        result
    }
}
```

**Rust Advantages:**
1. Returns ALL symmetries found
2. Maps symmetries to specific PCs
3. Enum type prevents invalid symmetry values
4. Includes T1 for completeness

## Voiceleading Search

### Python Implementation

```python
# voiceleading.py
class Voiceleading:
    @staticmethod
    def gen(v1, ch2, rules=[no_vox_crossings]):
        """Generate all voiceleadings from voicing v1 to chord ch2."""
        results = []
        source_pitches = v1.pitches
        target_notes = ch2.spelling

        # All bijective mappings
        for perm in permutations(range(len(target_notes))):
            # All contour combinations (up/down for each voice)
            for contour in product([1, -1], repeat=len(source_pitches)):
                paths = []
                for i, (src, direction) in enumerate(zip(source_pitches, contour)):
                    target = target_notes[perm[i]]
                    path = calculate_path(src, target, direction)
                    paths.append(path)

                # Validate against rules
                vl = Voiceleading(v1, ch2, paths)
                if all(rule(vl) for rule in rules):
                    results.append(vl)

        # Sort by distance
        results.sort(key=lambda vl: vl.naive_distance)
        return results
```

**Python Issues:**
1. `list.index()` fails with enharmonic equivalents (documented bug)
2. Rules applied post-hoc, not during generation
3. Memory-intensive for large voicings

### Rust Implementation

```rust
// geometry/symmetry/voiceleading.rs
pub struct Voiceleading {
    pub departures: Voicing,
    pub paths: Vec<i8>,
    pub destinations: Voicing,
}

impl Voiceleading {
    pub fn find_all<R: VoiceleadingRule>(
        departures: &Voicing,
        destination_notes: &[Note],
        rules: &[R],
    ) -> Result<Vec<(usize, Self)>, MusicSemanticsError> {
        let n = departures.len();
        let mut results = Vec::new();

        // All permutations of destination notes
        for perm in (0..n).permutations(n) {
            // All contour combinations
            for contour in (0..n).map(|_| vec![true, false]).multi_cartesian_product() {
                let mut paths = Vec::new();
                let mut valid = true;

                for (i, &ascending) in contour.iter().enumerate() {
                    let src = &departures.0[i];
                    let target = &destination_notes[perm[i]];

                    let path = if ascending {
                        src.note.distance_up_to_note(target) as i8
                    } else {
                        -(src.note.distance_down_to_note(target) as i8)
                    };
                    paths.push(path);
                }

                // Apply rules during generation
                let vl = Self::try_build(departures, &paths, destination_notes)?;
                if rules.iter().all(|r| r.apply(&vl)) {
                    let score = vl.naive_distance();
                    results.push((score, vl));
                }
            }
        }

        results.sort_by_key(|(score, _)| *score);
        Ok(results)
    }

    pub fn naive_distance(&self) -> usize {
        self.paths.iter().map(|p| p.abs() as usize).sum()
    }
}
```

**Rust Advantages:**
1. Type-safe path representation
2. Rules applied during generation (early exit possible)
3. Returns `Result` for error handling
4. Score included in return type

**Suggestion - Add parallel voiceleading search:**

```rust
use rayon::prelude::*;

impl Voiceleading {
    pub fn find_all_parallel<R: VoiceleadingRule + Sync>(
        departures: &Voicing,
        destination_notes: &[Note],
        rules: &[R],
    ) -> Result<Vec<(usize, Self)>, MusicSemanticsError> {
        let n = departures.len();
        let perms: Vec<_> = (0..n).permutations(n).collect();

        let results: Vec<_> = perms.par_iter()
            .flat_map(|perm| {
                // ... same logic as above, parallelized
            })
            .collect();

        // Sort results
        let mut sorted = results;
        sorted.sort_by_key(|(score, _)| *score);
        Ok(sorted)
    }
}
```

## Subchord Enumeration

### Python Implementation

```python
def is_subchord(pcs, subchord, strict=False):
    """Check if subchord is a subset of pcs (with mode consideration)."""
    pcs = pitch_set_sanitize(pcs)
    subchord = pitch_set_sanitize(subchord)

    if strict:
        return set(subchord).issubset(set(pcs))
    else:
        # Check against all modes
        for mode in modes_from_pitch_set(pcs):
            if set(subchord).issubset(set(mode)):
                return True
        return False
```

### Rust Implementation

```rust
// geometry/sets.rs
pub fn get_subchords(pc_set: &PcSet, size: usize) -> Result<Vec<PcSet>, MusicSemanticsError> {
    if size > pc_set.len() {
        return Err(MusicSemanticsError::SizeTooLargeForSubchords);
    }
    if size < 3 {
        return Err(MusicSemanticsError::SizeTooSmallForChords);
    }

    let subchords: Vec<PcSet> = pc_set.0
        .iter()
        .combinations(size)
        .map(|combo| PcSet::new(combo.into_iter().cloned().collect()))
        .collect();

    Ok(subchords)
}
```

**Suggestion - Add mode-aware subchord check:**

```rust
impl PcSet {
    /// Check if other is a subchord of any mode of self
    pub fn contains_subchord(&self, other: &PcSet) -> bool {
        self.modes().iter().any(|mode| {
            other.0.iter().all(|pc| mode.0.contains(pc))
        })
    }

    /// Find which modes contain the given subchord
    pub fn modes_containing(&self, subchord: &PcSet) -> Vec<(usize, PcSet)> {
        self.modes()
            .into_iter()
            .enumerate()
            .filter(|(_, mode)| {
                subchord.0.iter().all(|pc| mode.0.contains(pc))
            })
            .collect()
    }
}
```

## Interval Matrix (Incomplete in Both)

### Python

The Python library has `get_int_matrix()` but it's underutilized:

```python
def get_int_matrix(pitch_set):
    """Return interval matrix for all mode transpositions."""
    pitch_set = pitch_set_sanitize(pitch_set)
    return [pitch_set_to_int_row(rotate_pitch_set(pitch_set, i))
            for i in range(len(pitch_set))]
```

### Rust

Rust has a stub type in `geometry/mod.rs`:

```rust
// Currently just a placeholder
pub struct IntervalMatrix;
```

**Suggestion - Full implementation:**

```rust
/// Matrix of intervals between all pitch class pairs
pub struct IntervalMatrix {
    pcs: PcSet,
    matrix: Vec<Vec<IntervalClass>>,
}

impl IntervalMatrix {
    pub fn new(pc_set: &PcSet) -> Self {
        let n = pc_set.len();
        let mut matrix = vec![vec![IntervalClass::Ic0; n]; n];

        for i in 0..n {
            for j in 0..n {
                let from = &pc_set.0[i];
                let to = &pc_set.0[j];
                let interval = from.distance_up_to(to);
                matrix[i][j] = IntervalClass::try_from(interval).unwrap();
            }
        }

        Self { pcs: pc_set.clone(), matrix }
    }

    /// Get interval from row pitch to column pitch
    pub fn get(&self, row: usize, col: usize) -> Option<&IntervalClass> {
        self.matrix.get(row)?.get(col)
    }

    /// Find all instances of a specific interval
    pub fn find_interval(&self, ic: IntervalClass) -> Vec<(Pc, Pc)> {
        let mut results = Vec::new();
        for (i, row) in self.matrix.iter().enumerate() {
            for (j, &ref interval) in row.iter().enumerate() {
                if *interval == ic {
                    results.push((self.pcs.0[i].clone(), self.pcs.0[j].clone()));
                }
            }
        }
        results
    }

    /// Get interval class vector (counts of each interval)
    pub fn interval_vector(&self) -> [usize; 6] {
        let mut vector = [0usize; 6];
        for row in &self.matrix {
            for ic in row {
                let idx = match ic {
                    IntervalClass::Ic0 => continue,  // Skip unison
                    IntervalClass::Ic1 | IntervalClass::Ic11 => 0,
                    IntervalClass::Ic2 | IntervalClass::Ic10 => 1,
                    IntervalClass::Ic3 | IntervalClass::Ic9 => 2,
                    IntervalClass::Ic4 | IntervalClass::Ic8 => 3,
                    IntervalClass::Ic5 | IntervalClass::Ic7 => 4,
                    IntervalClass::Ic6 => 5,
                };
                vector[idx] += 1;
            }
        }
        // Divide by 2 (each interval counted twice)
        for v in &mut vector {
            *v /= 2;
        }
        vector
    }
}
```

## Melodic Contour Analysis

### Python

Python has `melodic_sequencer.py` but it's incomplete/stubbed.

### Rust

Rust has a full contour module:

```rust
// geometry/contour.rs
pub enum Movement {
    Up,
    Down,
    Same,
}

pub struct Contour(Vec<Movement>);

impl Contour {
    pub fn from_pitches(pitches: &[Pitch]) -> Self {
        let movements: Vec<Movement> = pitches.windows(2)
            .map(|pair| {
                match pair[1].midi_note.cmp(&pair[0].midi_note) {
                    Ordering::Greater => Movement::Up,
                    Ordering::Less => Movement::Down,
                    Ordering::Equal => Movement::Same,
                }
            })
            .collect();
        Self(movements)
    }
}

pub enum CompositeMovement {
    Ascent,      // All up
    Descent,     // All down
    Arch,        // Up then down
    Trough,      // Down then up
    Plateau,     // Same throughout
}
```

**Suggestion - Add contour similarity metric:**

```rust
impl Contour {
    /// Compare contours using CSIM (Contour Similarity)
    pub fn similarity(&self, other: &Contour) -> f64 {
        if self.0.len() != other.0.len() {
            return 0.0;
        }

        let matches = self.0.iter()
            .zip(other.0.iter())
            .filter(|(a, b)| a == b)
            .count();

        matches as f64 / self.0.len() as f64
    }

    /// Check if contours are equivalent (same shape)
    pub fn is_equivalent(&self, other: &Contour) -> bool {
        self.0 == other.0
    }

    /// Get the retrograde (reversed) contour
    pub fn retrograde(&self) -> Self {
        Self(self.0.iter().rev().cloned().collect())
    }

    /// Get the inversion (flipped) contour
    pub fn inversion(&self) -> Self {
        Self(self.0.iter().map(|m| match m {
            Movement::Up => Movement::Down,
            Movement::Down => Movement::Up,
            Movement::Same => Movement::Same,
        }).collect())
    }
}
```

## Summary: Algorithm Completeness

| Algorithm | Python | Rust | Notes |
|-----------|--------|------|-------|
| PC normalization | Partial (no zeroing) | Complete | Rust enforces at construction |
| Mode generation | Complete | Complete | Similar performance |
| Spelling rules | Complete | Complete | Rust adds `not_all` field |
| Symmetry detection | Partial | Complete | Rust finds all symmetries |
| Voiceleading | Buggy | Complete | Rust fixes enharmonic bug |
| Subchord enumeration | Basic | Complete | Rust adds mode-aware variants |
| Interval matrix | Basic | Stub | Needs implementation |
| Melodic contour | Incomplete | Complete | Rust has full analysis |
