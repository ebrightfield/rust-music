//! Gourlay spring-rod spacing calibration sweep (implementation plan Phase 4).
//!
//! Sweeps `spacing_exponent` (c) and `spring_constant` (k) over a corpus of
//! representative measures and reports, per configuration:
//!
//! - **natural width** — the `s = 1` measure width, compared against the
//!   legacy power-of-ratio model's width for the same measure. The legacy
//!   model is reproduced here (`legacy_width`) purely as a calibration
//!   reference; it is not used by the engraver any more.
//! - **ink clearance under compression** — the smallest gap between adjacent
//!   rendered notehead/dot ink and the next notehead/accidental ink.
//! - **rod share** and **realized advance ratio** — the incompressible
//!   fraction of natural width, and the perceived long:short spacing contrast.
//!
//! Glyph-specific rods now follow the bundled font's actual advance and ink
//! bounds. The historic legacy-width comparison is informative, not a target.
//!
//! Run with:
//! ```text
//! cargo run --example spacing_calibration -p music-engraver
//! ```

use music_engraver::layout::accidental::ResolvedAccidental;
use music_engraver::layout::measure::{
    layout_measure, MeasureElement, MeasureLayoutConfig, NoteAnnotations, NoteEvent, NoteheadStyle,
};

/// Staff space in font design units (Bravura: 1000 upem / 4 spaces = 250).
const SS: f64 = 250.0;

/// Legacy power-of-ratio spacing, reproduced for width comparison only.
///
/// The pre-Gourlay model (verified against `be1bc93~1`): every event's
/// *entire* width was `min_note_spacing · ratio^steps`, where `steps` is how
/// many duration doublings separate the event from the shortest note in the
/// measure. Defaults were `min_note_spacing = 1.5·ss`, `ratio = 1.6`.
/// Notably it added **no** dot or accidental allowance — dots and accidentals
/// got no extra room, which is precisely the crowding the rod model fixes.
fn legacy_width(durations: &[i8]) -> f64 {
    const RATIO: f64 = 1.6;
    const MIN_NOTE_SPACING: f64 = 1.5 * SS;
    let shortest = durations.iter().copied().max().unwrap_or(2);
    durations
        .iter()
        .map(|&d| {
            let steps = shortest as f64 - d as f64;
            MIN_NOTE_SPACING * RATIO.powf(steps)
        })
        .sum()
}

/// One corpus entry: a bare sequence of rhythmic events.
struct Case {
    name: &'static str,
    /// `duration_log2` per event (-1=breve, 0=whole, 1=half, 2=quarter, 3=eighth, 4=16th).
    durations: Vec<i8>,
    dots: Vec<u8>,
    accidentals: Vec<bool>,
}

fn corpus() -> Vec<Case> {
    let n = |durs: Vec<i8>| {
        let len = durs.len();
        (durs, vec![0; len], vec![false; len])
    };
    vec![
        {
            // C-major scale: 8 straight quarters.
            let (durations, dots, accidentals) = n(vec![2; 8]);
            Case {
                name: "scale-8-quarters",
                durations,
                dots,
                accidentals,
            }
        },
        {
            // Twinkle: quarters ending in a half.
            let (durations, dots, accidentals) = n(vec![2, 2, 2, 2, 2, 2, 1]);
            Case {
                name: "twinkle-phrase",
                durations,
                dots,
                accidentals,
            }
        },
        {
            // Dotted rhythms: dotted-quarter + eighth pairs.
            let durations = vec![2, 3, 2, 3];
            let dots = vec![1, 0, 1, 0];
            let accidentals = vec![false; 4];
            Case {
                name: "dotted-quarter-eighth",
                durations,
                dots,
                accidentals,
            }
        },
        {
            // Dense 16ths — the case the old model could crush.
            let (durations, dots, accidentals) = n(vec![4; 16]);
            Case {
                name: "dense-16ths",
                durations,
                dots,
                accidentals,
            }
        },
        {
            // Mixed whole + eighths: the widest duration ratio in one measure.
            let (durations, dots, accidentals) = n(vec![0, 3, 3, 3, 3]);
            Case {
                name: "mixed-whole-eighth",
                durations,
                dots,
                accidentals,
            }
        },
        {
            // Chromatic run: every note carries an accidental (rod-heavy).
            let durations = vec![3; 8];
            let dots = vec![0; 8];
            let accidentals = vec![true; 8];
            Case {
                name: "chromatic-accidentals",
                durations,
                dots,
                accidentals,
            }
        },
    ]
}

fn build_elements(case: &Case) -> Vec<MeasureElement> {
    case.durations
        .iter()
        .zip(&case.dots)
        .zip(&case.accidentals)
        .enumerate()
        .map(|(i, ((&duration_log2, &dots), &acc))| {
            MeasureElement::Note(NoteEvent {
                staff_position: (i % 8) as i8,
                duration_log2,
                dots,
                accidental: if acc {
                    Some(ResolvedAccidental::plain(smufl::Glyph::AccidentalSharp))
                } else {
                    None
                },
                stem_direction: None,
                annotations: NoteAnnotations::default(),
            })
        })
        .collect()
}

/// Smallest actual horizontal ink gap between adjacent events at `target`,
/// including accidental leading gaps and dot ink. In staff spaces.
fn min_gap_under_compression(case: &Case, cfg: &MeasureLayoutConfig, target: f64) -> f64 {
    use music_engraver::font::BUNDLED_BRAVURA;
    use music_engraver::layout::accidental::ACCIDENTAL_NOTEHEAD_PADDING_SS;
    use music_engraver::layout::dot::{DOT_INTER_DOT_SPACING_SS, DOT_NOTEHEAD_PADDING_SS};
    use smufl::Glyph;

    let font = &*BUNDLED_BRAVURA;
    let layout = layout_measure(&build_elements(case), cfg);
    let scale = if layout.total_spring > 0.0 {
        ((target - layout.total_rod) / layout.total_spring).max(0.0)
    } else {
        1.0
    };
    let mut x = 0.0;
    let mut previous_end = 0.0;
    let mut previous_right: Option<f64> = None;
    let mut min_gap = f64::INFINITY;
    for positioned in &layout.elements {
        x += positioned.x - previous_end;
        previous_end = positioned.x + positioned.width;
        let MeasureElement::Note(note) = &positioned.element else {
            continue;
        };
        let glyph = NoteheadStyle::Normal.glyph(note.duration_log2);
        let bbox = font.glyph_bbox_design_units(glyph).unwrap();
        let advance = f64::from(font.glyph_advance(glyph).unwrap());
        let mut left = x + bbox.x_left.min(0.0);
        if note.accidental.is_some() {
            let sharp = Glyph::AccidentalSharp;
            left = left.min(
                x - f64::from(font.glyph_advance(sharp).unwrap())
                    - ACCIDENTAL_NOTEHEAD_PADDING_SS * SS
                    + font.glyph_bbox_design_units(sharp).unwrap().x_left,
            );
        }
        let mut right = x + bbox.x_right.max(advance);
        if note.dots > 0 {
            right = right.max(
                x + advance
                    + DOT_NOTEHEAD_PADDING_SS * SS
                    + (f64::from(note.dots) - 1.0) * DOT_INTER_DOT_SPACING_SS * SS
                    + font
                        .glyph_bbox_design_units(Glyph::AugmentationDot)
                        .unwrap()
                        .x_right
                        .max(f64::from(
                            font.glyph_advance(Glyph::AugmentationDot).unwrap(),
                        )),
            );
        }
        if let Some(previous) = previous_right {
            min_gap = min_gap.min(left - previous);
        }
        previous_right = Some(right);
        x += positioned.rod + scale * positioned.spring;
    }
    min_gap / SS
}

fn main() {
    let exponents = [0.5, 0.6, 0.7];
    let constants = [0.8, 1.0, 1.2];
    let cases = corpus();

    println!("Gourlay spacing calibration sweep");
    println!("staff space = {SS} font units; widths in staff spaces\n");

    // Natural-width comparison against the legacy model.
    println!("== Natural width (s = 1) vs legacy power-of-ratio model ==");
    print!("{:<24}{:>10}", "case", "legacy");
    for c in exponents {
        for k in constants {
            print!("{:>12}", format!("c{c} k{k}"));
        }
    }
    println!();

    for case in &cases {
        let legacy = legacy_width(&case.durations) / SS;
        print!("{:<24}{:>10.2}", case.name, legacy);
        for c in exponents {
            for k in constants {
                let mut cfg = MeasureLayoutConfig::from_staff_space(SS);
                cfg.spacing_exponent = c;
                cfg.spring_constant = k * SS;
                let layout = layout_measure(&build_elements(case), &cfg);
                print!("{:>12.2}", layout.total_width / SS);
            }
        }
        println!();
    }

    // The hard rods reserve every printed glyph's ink, even when the springs
    // reach zero. Report the actual ink gap at 50% of natural width; unlike an
    // estimated black-notehead threshold this remains valid for whole notes,
    // dotted notes and accidental-heavy passages.
    println!("\n== Minimum adjacent ink clearance at 50% natural width (ss) ==");
    print!("{:<24}", "case");
    for c in exponents {
        for k in constants {
            print!("{:>12}", format!("c{c} k{k}"));
        }
    }
    println!();

    for case in &cases {
        print!("{:<24}", case.name);
        for c in exponents {
            for k in constants {
                let mut cfg = MeasureLayoutConfig::from_staff_space(SS);
                cfg.spacing_exponent = c;
                cfg.spring_constant = k * SS;
                let natural = layout_measure(&build_elements(case), &cfg).total_width;

                let gap = min_gap_under_compression(case, &cfg, natural * 0.5);
                print!("{gap:>12.2}");
            }
        }
        println!();
    }

    // Hard floor: all springs collapsed. Every entry should retain at least
    // the configured minimum rod padding between adjacent ink.
    println!("\n== Minimum adjacent ink clearance at spring floor (s = 0) ==");
    for case in &cases {
        let cfg = MeasureLayoutConfig::from_staff_space(SS);
        let gap = min_gap_under_compression(case, &cfg, 0.0);
        let flag = if gap * SS + 1e-9 < cfg.min_rod_padding {
            " *UNDER PAD"
        } else {
            ""
        };
        println!("  {:<24}{gap:.2}ss{flag}", case.name);
    }

    // Rod share: the hard floor on compression, independent of target.
    println!("\n== Incompressible rod share of natural width ==");
    print!("{:<24}", "case");
    for c in exponents {
        for k in constants {
            print!("{:>12}", format!("c{c} k{k}"));
        }
    }
    println!();
    for case in &cases {
        print!("{:<24}", case.name);
        for c in exponents {
            for k in constants {
                let mut cfg = MeasureLayoutConfig::from_staff_space(SS);
                cfg.spacing_exponent = c;
                cfg.spring_constant = k * SS;
                let l = layout_measure(&build_elements(case), &cfg);
                print!("{:>12.2}", l.total_rod / l.total_width);
            }
        }
        println!();
    }

    // Solve for the k that reproduces the legacy natural width per case.
    //
    // For a uniform-rhythm measure of n events every spring is exactly k, so
    // total = n·(rod + k) and the matching k is legacy/n - rod. This is the
    // churn-minimizing value the plan asks for, computed rather than guessed.
    println!("\n== k that reproduces legacy natural width (c-independent for uniform rhythm) ==");
    for case in &cases {
        let cfg = MeasureLayoutConfig::from_staff_space(SS);
        let n = case.durations.len() as f64;
        let legacy = legacy_width(&case.durations);
        let uniform = case.durations.iter().all(|&d| d == case.durations[0]);
        let layout = layout_measure(&build_elements(case), &cfg);
        let implied_k = (legacy - layout.total_rod) / n / SS;
        println!(
            "  {:<24}legacy {:>6.2}ss  rod {:>6.2}ss  => k = {:>6.2}ss{}",
            case.name,
            legacy / SS,
            layout.total_rod / SS,
            implied_k,
            if uniform {
                ""
            } else {
                "  (mixed rhythm: approximate)"
            }
        );
    }

    // Long/short contrast: how much wider is a whole note's spring than a 16th's?
    println!("\n== Long:short spring ratio (whole vs 16th, 4 doublings) ==");
    for c in exponents {
        let ratio = 2.0_f64.powf(4.0).powf(c);
        println!("  c = {c}: whole-note spring is {ratio:.2}x the 16th's");
    }

    // Total-width proportionality: the quantity a reader actually perceives.
    //
    // Engraving practice (Gould, "Behind Bars", ch. on horizontal spacing) wants
    // a note twice as long to occupy noticeably-but-sublinearly more space —
    // strict 2x proportionality wastes paper and reads badly. This reports the
    // realized center-to-center advance ratio between a half and an eighth in
    // one measure, which is what c controls end to end.
    println!("\n== Realized advance ratio, half vs eighth (same measure) ==");
    println!("(strict proportional would be 4.00; engraving practice favors ~1.5-2.5)");
    for c in exponents {
        for k in constants {
            let mut cfg = MeasureLayoutConfig::from_staff_space(SS);
            cfg.spacing_exponent = c;
            cfg.spring_constant = k * SS;
            // One half note then one eighth: eighth is shortest (duration 1).
            let elements = vec![
                MeasureElement::Note(NoteEvent {
                    staff_position: 0,
                    duration_log2: 1,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureElement::Note(NoteEvent {
                    staff_position: 2,
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
            ];
            let l = layout_measure(&elements, &cfg);
            let ratio = l.elements[0].width / l.elements[1].width;
            println!("  c = {c}, k = {k}: half advance is {ratio:.2}x the eighth's");
        }
    }
}
