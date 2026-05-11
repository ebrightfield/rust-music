//! Verifies that `ScaleShapeSearchResult::from_raw_search_result(...).simple`
//! contains the five CAGED-shaped footprints for every key, for both the
//! major scale and the major pentatonic scale.
//!
//! Ground-truth shapes are defined in C major (library convention:
//! string 0 = low E, string 5 = high E; fret 0 = nut). Shapes for other
//! keys are derived by transposing up by N semitones. If the transposed
//! shape's minimum fret is >= 12, the whole shape is shifted down one octave.
//!
//! Major-pentatonic shapes are derived by dropping the 4th and 7th scale
//! degrees (by pitch class) from the major-scale CAGED shapes.

use std::collections::HashSet;

use music::fretboard::fretboard_shape::melodic_shape_search::ScaleShapeSearchResult;
use music::fretboard::STD_6STR_GTR;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::PcShape;

/// (string, fret) positions for a single shape.
type PositionSet = HashSet<(u8, u8)>;

#[derive(Clone, Copy, Debug)]
struct ShapeName(&'static str);

/// Ground-truth C-major CAGED shapes. Each inner array is a string's frets,
/// indexed by the library's string numbering (0 = low E, 5 = high E).
const CAGED_C_MAJOR: &[(ShapeName, [&[u8]; 6])] = &[
    (
        ShapeName("C-form"),
        [
            &[0, 1, 3], // low E
            &[0, 2, 3], // A
            &[0, 2, 3], // D
            &[0, 2],    // G
            &[0, 1, 3], // B
            &[0, 1, 3], // high E
        ],
    ),
    (
        ShapeName("A-form"),
        [
            &[3, 5],
            &[2, 3, 5],
            &[2, 3, 5],
            &[2, 4, 5],
            &[3, 5, 6],
            &[3, 5],
        ],
    ),
    (
        ShapeName("G-form"),
        [
            &[5, 7, 8],
            &[5, 7, 8],
            &[5, 7],
            &[4, 5, 7],
            &[5, 6, 8],
            &[5, 7, 8],
        ],
    ),
    (
        ShapeName("E-form"),
        [
            &[7, 8, 10],
            &[7, 8, 10],
            &[7, 9, 10],
            &[7, 9, 10],
            &[8, 10],
            &[7, 8, 10],
        ],
    ),
    (
        ShapeName("D-form"),
        [
            &[10, 12, 13],
            &[10, 12],
            &[9, 10, 12],
            &[9, 10, 12],
            &[10, 12, 13],
            &[10, 12, 13],
        ],
    ),
];

fn c_major_shape_positions(frets_per_string: &[&[u8]; 6]) -> PositionSet {
    let mut set = HashSet::new();
    for (string_idx, frets) in frets_per_string.iter().enumerate() {
        for &fret in frets.iter() {
            set.insert((string_idx as u8, fret));
        }
    }
    set
}

/// Transpose every fret by `semitones`. If the resulting minimum fret is >= 12,
/// shift the whole shape down by an octave so it lives in the playable range.
fn transpose_shape(shape: &PositionSet, semitones: u8) -> PositionSet {
    let shifted: PositionSet = shape
        .iter()
        .map(|(s, f)| (*s, *f + semitones))
        .collect();
    let min_fret = shifted.iter().map(|(_, f)| *f).min().unwrap();
    if min_fret >= 12 {
        shifted.iter().map(|(s, f)| (*s, f - 12)).collect()
    } else {
        shifted
    }
}

/// For each position in `shape`, look up its pitch class on the standard
/// guitar tuning, and drop it if it matches the 4th or 7th scale degree
/// (relative to `root_pc`). Used to derive pentatonic shapes from CAGED.
fn drop_4_and_7(shape: &PositionSet, root_pc: Pc) -> PositionSet {
    let fretboard = &*STD_6STR_GTR;
    let root_u8 = u8::from(&root_pc);
    let pc_of_4 = Pc::from(&((root_u8 + 5) % 12));
    let pc_of_7 = Pc::from(&((root_u8 + 11) % 12));
    shape
        .iter()
        .filter(|(s, f)| {
            let sounded = fretboard.sounded_note(*s, *f).unwrap();
            let pc = Pc::from(&sounded.pitch.note);
            pc != pc_of_4 && pc != pc_of_7
        })
        .copied()
        .collect()
}

fn shape_to_positions(shape: &music::fretboard::fretboard_shape::melodic_shape_search::MelodicFretboardShape) -> PositionSet {
    shape
        .shape
        .iter()
        .map(|n| (n.string, n.fret))
        .collect()
}

/// All 12 pitch classes, paired with a default spelling of the root.
const ROOTS: &[(u8, Note)] = &[
    (0,  Note::C),
    (1,  Note::Des),
    (2,  Note::D),
    (3,  Note::Ees),
    (4,  Note::E),
    (5,  Note::F),
    (6,  Note::Fis),
    (7,  Note::G),
    (8,  Note::Aes),
    (9,  Note::A),
    (10, Note::Bes),
    (11, Note::B),
];

/// Major-scale intervals from root (zeroed pitch-class representation). PcShape
/// stores pitch classes relative to the root, and `try_spell` interprets them
/// as intervals above the given root — so the set is the same for every key.
const MAJOR_INTERVALS: &[Pc] = &[
    Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc5, Pc::Pc7, Pc::Pc9, Pc::Pc11,
];

/// Major-pentatonic intervals from root (drop the 4th and 7th).
const MAJOR_PENTATONIC_INTERVALS: &[Pc] = &[
    Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc7, Pc::Pc9,
];

fn assert_shape_present(
    simple: &[music::fretboard::fretboard_shape::melodic_shape_search::MelodicFretboardShape],
    expected: &PositionSet,
    context: &str,
) {
    let found = simple.iter().any(|s| shape_to_positions(s) == *expected);
    if !found {
        // Build a diagnostic: expected + list of actual shapes' positions.
        let mut msg = format!("\n{}\n  expected shape (as (string, fret) pairs):\n    ", context);
        let mut expected_sorted: Vec<_> = expected.iter().collect();
        expected_sorted.sort();
        for (s, f) in expected_sorted {
            msg.push_str(&format!("({},{}) ", s, f));
        }
        msg.push_str("\n  but .simple contained shapes:\n");
        for (i, s) in simple.iter().enumerate() {
            let mut positions: Vec<_> = shape_to_positions(s).into_iter().collect();
            positions.sort();
            msg.push_str(&format!("    [{}] ", i));
            for (s, f) in positions {
                msg.push_str(&format!("({},{}) ", s, f));
            }
            msg.push('\n');
        }
        panic!("{}", msg);
    }
}

#[test]
fn all_caged_shapes_present_for_major_scales() {
    let fretboard = &*STD_6STR_GTR;
    let mut failures: Vec<String> = vec![];
    let pc_set = PcShape::new(MAJOR_INTERVALS.to_vec());

    for (root_pc, root_note) in ROOTS {
        let spelled = match pc_set.try_spell(root_note) {
            Ok(n) => n,
            Err(e) => {
                failures.push(format!("{} major: try_spell failed: {:?}", root_note, e));
                continue;
            }
        };
        let result =
            match ScaleShapeSearchResult::from_raw_search_result(&spelled, fretboard) {
                Ok(r) => r,
                Err(e) => {
                    failures.push(format!(
                        "{} major: from_raw_search_result failed: {:?}",
                        root_note, e
                    ));
                    continue;
                }
            };

        for (name, frets_per_string) in CAGED_C_MAJOR {
            let c_shape = c_major_shape_positions(frets_per_string);
            let expected = transpose_shape(&c_shape, *root_pc);
            let context = format!("{} major / {}", root_note, name.0);
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert_shape_present(&result.simple, &expected, &context);
            }));
            if res.is_err() {
                failures.push(context);
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} CAGED-shape assertions failed for major scales:\n  {}",
            failures.len(),
            failures.join("\n  ")
        );
    }
}

#[test]
fn all_caged_shapes_present_for_major_pentatonic_scales() {
    let fretboard = &*STD_6STR_GTR;
    let mut failures: Vec<String> = vec![];
    let pc_set = PcShape::new(MAJOR_PENTATONIC_INTERVALS.to_vec());

    for (root_pc, root_note) in ROOTS {
        let spelled = match pc_set.try_spell(root_note) {
            Ok(n) => n,
            Err(e) => {
                failures.push(format!(
                    "{} major pentatonic: try_spell failed: {:?}",
                    root_note, e
                ));
                continue;
            }
        };
        let result =
            match ScaleShapeSearchResult::from_raw_search_result(&spelled, fretboard) {
                Ok(r) => r,
                Err(e) => {
                    failures.push(format!(
                        "{} major pentatonic: from_raw_search_result failed: {:?}",
                        root_note, e
                    ));
                    continue;
                }
            };

        for (name, frets_per_string) in CAGED_C_MAJOR {
            let c_major_shape = c_major_shape_positions(frets_per_string);
            let c_pent_shape = drop_4_and_7(&c_major_shape, Pc::Pc0);
            let expected = transpose_shape(&c_pent_shape, *root_pc);
            let context = format!("{} major pentatonic / {}", root_note, name.0);
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert_shape_present(&result.simple, &expected, &context);
            }));
            if res.is_err() {
                failures.push(context);
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} CAGED-shape assertions failed for major pentatonic scales:\n  {}",
            failures.len(),
            failures.join("\n  ")
        );
    }
}

// Silence unused import if Pitch only referenced above.
#[allow(dead_code)]
fn _force_use(_p: Pitch) {}
