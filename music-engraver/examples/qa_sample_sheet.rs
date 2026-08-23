//! QA sample sheet: renders a broad corpus of representative + edge-case scores
//! to individual SVG files under `docs/engraver-qa/` for human visual review.
//!
//! Run with:
//! ```sh
//! CARGO_HOME=$HOME/.cargo cargo run -q -p music-engraver --example qa_sample_sheet
//! ```
//!
//! With the `png` feature, a handful of PNGs are also emitted:
//! ```sh
//! CARGO_HOME=$HOME/.cargo cargo run -q -p music-engraver --features png --example qa_sample_sheet
//! ```
//!
//! Each sample is rendered under `catch_unwind`; a panicking sample is reported
//! as a FAILURE rather than aborting the whole run, so the sheet surfaces
//! crash-bugs as part of the QA output.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::hairpin::HairpinType;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::tab::TabScoreBuilder;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, oct: i8) -> Pitch {
    Pitch::new(note, oct)
}

fn dotted(kind: DurationKind, dots: u8) -> Duration {
    Duration::new(kind, dots)
}

fn out_dir() -> PathBuf {
    // Workspace root is the parent of CARGO_MANIFEST_DIR (music-engraver/).
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("workspace root").to_path_buf();
    root.join("docs/engraver-qa")
}

/// A single named sample producing an SVG string. Returns the SVG via a closure
/// so we can wrap evaluation in `catch_unwind`.
struct Sample {
    name: &'static str,
    category: &'static str,
    render: Box<dyn Fn() -> String>,
}

fn samples() -> Vec<Sample> {
    let mut v: Vec<Sample> = Vec::new();

    macro_rules! sample {
        ($name:expr, $cat:expr, $body:expr) => {
            v.push(Sample {
                name: $name,
                category: $cat,
                render: Box::new(move || $body),
            });
        };
    }

    // 1. Diatonic C-major scale, two octaves ascending then descending (treble).
    sample!("01_diatonic_scale_two_octaves", "scales", {
        let mut s = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Open)
            .time_signature(4, 4);
        let scale = [Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B];
        // ascending C4..C6
        for oct in 4..=5 {
            for n in scale {
                s = s.note(p(n, oct), Duration::EIGHTH);
            }
        }
        s = s.note(p(Note::C, 6), Duration::QTR).barline();
        // descending C6..C4
        for oct in (4..=5).rev() {
            for n in scale.iter().rev() {
                s = s.note(p(*n, oct), Duration::EIGHTH);
            }
        }
        s = s.note(p(Note::C, 4), Duration::QTR);
        s.end_barline().render_svg()
    });

    // 2. Wide register span: lots of ledger lines above and below.
    sample!("02_wide_register_ledger_lines", "register", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 3), Duration::QTR)   // well below staff
            .note(p(Note::A, 3), Duration::QTR)
            .note(p(Note::C, 6), Duration::QTR)   // well above staff
            .note(p(Note::E, 6), Duration::QTR)
            .barline()
            .note(p(Note::G, 6), Duration::HALF)  // very high
            .note(p(Note::F, 3), Duration::HALF)  // low
            .barline()
            .note(p(Note::C, 7), Duration::WHOLE) // extreme high
            .barline()
            .note(p(Note::C, 2), Duration::WHOLE) // extreme low
            .end_barline()
            .render_svg()
    });

    // 3. Chromatic run (dense single accidentals).
    sample!("03_chromatic_run", "accidentals", {
        let mut s = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4);
        let chromatic = [
            Note::C, Note::Cis, Note::D, Note::Dis, Note::E, Note::F,
            Note::Fis, Note::G, Note::Gis, Note::A, Note::Ais, Note::B,
        ];
        for (i, n) in chromatic.iter().enumerate() {
            s = s.note(p(*n, 4), Duration::EIGHTH);
            if (i + 1) % 4 == 0 {
                s = s.barline();
            }
        }
        s.end_barline().render_svg()
    });

    // 4. Double sharps and double flats.
    sample!("04_double_accidentals", "accidentals", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::Cisis, 4), Duration::QTR)  // C double-sharp
            .note(p(Note::Disis, 4), Duration::QTR)
            .note(p(Note::Fisis, 4), Duration::QTR)
            .note(p(Note::Gisis, 4), Duration::QTR)
            .barline()
            .note(p(Note::Deses, 5), Duration::QTR)  // D double-flat
            .note(p(Note::Eeses, 5), Duration::QTR)
            .note(p(Note::Geses, 5), Duration::QTR)
            .note(p(Note::Aeses, 5), Duration::QTR)
            .end_barline()
            .render_svg()
    });

    // 5. Triads — close-position root, 1st, 2nd inversion.
    sample!("05_triads", "chords", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)], Duration::QTR)
            .chord(vec![p(Note::E, 4), p(Note::G, 4), p(Note::C, 5)], Duration::QTR)
            .chord(vec![p(Note::G, 4), p(Note::C, 5), p(Note::E, 5)], Duration::QTR)
            .chord(vec![p(Note::C, 5), p(Note::E, 5), p(Note::G, 5)], Duration::QTR)
            .end_barline()
            .render_svg()
    });

    // 6. Seventh chords + spread voicing.
    sample!("06_seventh_chords", "chords", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            // close Cmaj7
            .chord(vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4), p(Note::B, 4)], Duration::QTR)
            // G7
            .chord(vec![p(Note::G, 3), p(Note::B, 3), p(Note::D, 4), p(Note::F, 4)], Duration::QTR)
            // spread voicing (wide)
            .chord(vec![p(Note::C, 3), p(Note::G, 4), p(Note::E, 5), p(Note::B, 5)], Duration::HALF)
            .end_barline()
            .render_svg()
    });

    // 7. Clusters (seconds → forced notehead offset).
    sample!("07_clusters", "chords", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .chord(vec![p(Note::C, 4), p(Note::D, 4), p(Note::E, 4)], Duration::QTR)
            .chord(vec![p(Note::C, 4), p(Note::D, 4), p(Note::E, 4), p(Note::F, 4), p(Note::G, 4)], Duration::QTR)
            .chord(vec![p(Note::B, 4), p(Note::C, 5), p(Note::D, 5)], Duration::HALF)
            .end_barline()
            .render_svg()
    });

    // 8. Beamed eighths and sixteenths, mixed beaming.
    sample!("08_beamed_groups", "beaming", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .beam_group(vec![
                (p(Note::C, 4), Duration::EIGHTH),
                (p(Note::E, 4), Duration::EIGHTH),
                (p(Note::G, 4), Duration::EIGHTH),
                (p(Note::C, 5), Duration::EIGHTH),
            ])
            .beam_group(vec![
                (p(Note::C, 5), Duration::SIXTEENTH),
                (p(Note::B, 4), Duration::SIXTEENTH),
                (p(Note::A, 4), Duration::SIXTEENTH),
                (p(Note::G, 4), Duration::SIXTEENTH),
            ])
            .barline()
            // mixed eighths + sixteenths in one beam group
            .beam_group(vec![
                (p(Note::C, 4), Duration::EIGHTH),
                (p(Note::D, 4), Duration::SIXTEENTH),
                (p(Note::E, 4), Duration::SIXTEENTH),
                (p(Note::F, 4), Duration::EIGHTH),
                (p(Note::G, 4), Duration::SIXTEENTH),
                (p(Note::A, 4), Duration::SIXTEENTH),
            ])
            .end_barline()
            .render_svg()
    });

    // 9. Tuplets: triplet and quintuplet.
    sample!("09_tuplets", "tuplets", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .tuplet(3, vec![
                (p(Note::C, 4), Duration::EIGHTH),
                (p(Note::D, 4), Duration::EIGHTH),
                (p(Note::E, 4), Duration::EIGHTH),
            ])
            .tuplet(3, vec![
                (p(Note::E, 4), Duration::EIGHTH),
                (p(Note::F, 4), Duration::EIGHTH),
                (p(Note::G, 4), Duration::EIGHTH),
            ])
            .barline()
            .tuplet(5, vec![
                (p(Note::C, 5), Duration::SIXTEENTH),
                (p(Note::B, 4), Duration::SIXTEENTH),
                (p(Note::A, 4), Duration::SIXTEENTH),
                (p(Note::G, 4), Duration::SIXTEENTH),
                (p(Note::F, 4), Duration::SIXTEENTH),
            ])
            .end_barline()
            .render_svg()
    });

    // 10. All rest durations.
    sample!("10_all_rests", "rests", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .rest(Duration::WHOLE)
            .barline()
            .rest(Duration::HALF)
            .rest(Duration::HALF)
            .barline()
            .rest(Duration::QTR)
            .rest(Duration::EIGHTH)
            .rest(Duration::SIXTEENTH)
            .rest(dotted(DurationKind::Qtr, 1))
            .barline()
            .rest(Duration::new(DurationKind::ThirtySecond, 0))
            .rest(Duration::new(DurationKind::SixtyFourth, 0))
            .end_barline()
            .render_svg()
    });

    // 11. Dynamics across a phrase.
    sample!("11_dynamics", "dynamics", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 4), Duration::QTR).dynamic(Dynamic::Ppp)
            .note(p(Note::E, 4), Duration::QTR).dynamic(Dynamic::Mp)
            .note(p(Note::G, 4), Duration::QTR).dynamic(Dynamic::Mf)
            .note(p(Note::C, 5), Duration::QTR).dynamic(Dynamic::Fff)
            .barline()
            .note(p(Note::C, 5), Duration::QTR).dynamic(Dynamic::Sfz)
            .note(p(Note::G, 4), Duration::QTR).dynamic(Dynamic::Fp)
            .note(p(Note::E, 4), Duration::HALF).dynamic(Dynamic::Pp)
            .end_barline()
            .render_svg()
    });

    // 12. Hairpins (crescendo + decrescendo).
    sample!("12_hairpins", "dynamics", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 4), Duration::QTR).dynamic(Dynamic::Piano).hairpin_start(HairpinType::Crescendo)
            .note(p(Note::E, 4), Duration::QTR)
            .note(p(Note::G, 4), Duration::QTR)
            .note(p(Note::C, 5), Duration::QTR).hairpin_end().dynamic(Dynamic::Forte)
            .barline()
            .note(p(Note::C, 5), Duration::QTR).hairpin_start(HairpinType::Decrescendo)
            .note(p(Note::G, 4), Duration::QTR)
            .note(p(Note::E, 4), Duration::QTR)
            .note(p(Note::C, 4), Duration::QTR).hairpin_end().dynamic(Dynamic::Piano)
            .end_barline()
            .render_svg()
    });

    // 13. Slurs and ties.
    sample!("13_slurs_and_ties", "phrasing", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 4), Duration::QTR).slur_start()
            .note(p(Note::E, 4), Duration::QTR)
            .note(p(Note::G, 4), Duration::QTR)
            .note(p(Note::C, 5), Duration::QTR).slur_end()
            .barline()
            .note(p(Note::C, 5), Duration::HALF).tie()
            .note(p(Note::C, 5), Duration::HALF)
            .end_barline()
            .render_svg()
    });

    // 14. Articulations.
    sample!("14_articulations", "articulations", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 5), Duration::QTR).articulation(Articulation::Staccato)
            .note(p(Note::C, 5), Duration::QTR).articulation(Articulation::Accent)
            .note(p(Note::C, 5), Duration::QTR).articulation(Articulation::Tenuto)
            .note(p(Note::C, 5), Duration::QTR).articulation(Articulation::Marcato)
            .barline()
            .note(p(Note::C, 5), Duration::HALF).articulation(Articulation::Fermata)
            .note(p(Note::C, 5), Duration::HALF).articulation(Articulation::Staccatissimo)
            .end_barline()
            .render_svg()
    });

    // 15. Multi-measure phrase with automatic line breaks (several systems).
    sample!("15_multi_system_phrase", "layout", {
        let mut s = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(1))
            .time_signature(4, 4)
            .auto_line_breaks()
            .system_width_fu(8000.0)
            .show_measure_numbers();
        let melody = [Note::G, Note::A, Note::B, Note::C, Note::D, Note::E, Note::Fis];
        for m in 0..8 {
            for n in melody.iter() {
                let oct = if *n == Note::C || *n == Note::D || *n == Note::E || *n == Note::Fis { 5 } else { 4 };
                s = s.note(p(*n, oct), Duration::EIGHTH);
            }
            // 8th note 8 completes the measure: 7 melody eighths + this one
            // = 8 eighths = exactly 4/4.
            s = s.note(p(Note::G, 4), Duration::EIGHTH);
            s = if m == 7 { s.end_barline() } else { s.barline() };
        }
        s.render_svg()
    });

    // 16. Bass clef sample.
    sample!("16_bass_clef", "clefs", {
        ScoreBuilder::new()
            .clef(Clef::Bass)
            .key_signature(KeySignature::Flats(2))
            .time_signature(4, 4)
            .note(p(Note::C, 2), Duration::QTR)
            .note(p(Note::E, 2), Duration::QTR)
            .note(p(Note::G, 2), Duration::QTR)
            .note(p(Note::C, 3), Duration::QTR)
            .barline()
            .chord(vec![p(Note::C, 2), p(Note::G, 2), p(Note::E, 3)], Duration::HALF)
            .note(p(Note::C, 3), Duration::HALF)
            .end_barline()
            .render_svg()
    });

    // 17. Treble-8 (8va / 8vb) clefs.
    sample!("17_treble_octave_clefs", "clefs", {
        ScoreBuilder::new()
            .clef(Clef::Treble8ba)   // guitar / tenor octave clef
            .time_signature(4, 4)
            .note(p(Note::E, 4), Duration::QTR)
            .note(p(Note::G, 4), Duration::QTR)
            .note(p(Note::B, 4), Duration::QTR)
            .note(p(Note::E, 5), Duration::QTR)
            .end_barline()
            .render_svg()
    });

    // 18. Barline styles (double, repeats).
    sample!("18_barline_styles", "layout", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 4), Duration::WHOLE)
            .barline_style(BarlineStyle::StartRepeat)
            .note(p(Note::E, 4), Duration::WHOLE)
            .barline_style(BarlineStyle::Double)
            .note(p(Note::G, 4), Duration::WHOLE)
            .barline_style(BarlineStyle::EndRepeat)
            .note(p(Note::C, 5), Duration::WHOLE)
            .end_barline()
            .render_svg()
    });

    // 19. Dotted-note rhythms.
    sample!("19_dotted_rhythms", "rhythm", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(p(Note::C, 4), dotted(DurationKind::Qtr, 1))
            .note(p(Note::D, 4), Duration::EIGHTH)
            .note(p(Note::E, 4), dotted(DurationKind::Half, 1))
            .barline()
            .note(p(Note::F, 4), dotted(DurationKind::Eighth, 1))
            .note(p(Note::G, 4), Duration::SIXTEENTH)
            .note(p(Note::A, 4), dotted(DurationKind::Qtr, 2))   // double-dotted
            .note(p(Note::B, 4), Duration::SIXTEENTH)
            .end_barline()
            .render_svg()
    });

    // 20. Key signatures sweep (many sharps / many flats).
    sample!("20_key_signatures", "layout", {
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Sharps(7))
            .time_signature(4, 4)
            .note(p(Note::C, 5), Duration::WHOLE)
            .barline()
            .note(p(Note::C, 5), Duration::WHOLE)
            .end_barline()
            .render_svg()
    });

    v
}

/// Guitar tab (multi-staff: notation over tab) — separate because it returns
/// via MultiStaffScore rather than ScoreBuilder.
fn render_tab() -> String {
    let notation = ScoreBuilder::new()
        .clef(Clef::Treble8ba)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::HALF)
        .end_barline();

    let tab = TabScoreBuilder::guitar()
        .quarter().fret(6, 0).next()
        .quarter().fret(4, 0).next()
        .quarter().fret(3, 0).next()
        .quarter().fret(1, 0)
        .barline()
        .quarter().fret(1, 0).next()
        .quarter().fret(3, 0).next()
        .half().fret(4, 0)
        .end_barline();

    MultiStaffScore::guitar_tab(notation, tab)
        .system_width_fu(10000.0)
        .render_svg()
}

fn main() {
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("create output dir");

    let mut ok: Vec<String> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    // PNG emission for a representative subset (only with the png feature).
    let png_subset = [
        "01_diatonic_scale_two_octaves",
        "06_seventh_chords",
        "12_hairpins",
        "15_multi_system_phrase",
        "16_bass_clef",
    ];

    for s in samples() {
        let render = s.render;
        let result = catch_unwind(AssertUnwindSafe(render));
        match result {
            Ok(svg) => {
                let path = dir.join(format!("{}.svg", s.name));
                std::fs::write(&path, &svg).expect("write SVG");
                let valid = svg.starts_with("<svg") && svg.contains("</svg>");
                println!(
                    "OK   [{:>13}] {} ({} bytes){}",
                    s.category,
                    s.name,
                    svg.len(),
                    if valid { "" } else { "  <-- WARNING: malformed SVG" }
                );
                ok.push(s.name.to_string());

                #[cfg(feature = "png")]
                if png_subset.contains(&s.name) {
                    // Re-render under png; svg already produced so this only
                    // exercises the rasterizer.
                    emit_png(&dir, s.name, &svg);
                }
                #[cfg(not(feature = "png"))]
                let _ = &png_subset;
            }
            Err(e) => {
                let msg = panic_message(&e);
                println!("FAIL [{:>13}] {} -- PANICKED: {}", s.category, s.name, msg);
                failed.push((s.name.to_string(), msg));
            }
        }
    }

    // Tab sample (separate code path).
    {
        let result = catch_unwind(AssertUnwindSafe(render_tab));
        match result {
            Ok(svg) => {
                let path = dir.join("21_guitar_tab.svg");
                std::fs::write(&path, &svg).expect("write SVG");
                println!("OK   [          tab] 21_guitar_tab ({} bytes)", svg.len());
                ok.push("21_guitar_tab".to_string());
            }
            Err(e) => {
                let msg = panic_message(&e);
                println!("FAIL [          tab] 21_guitar_tab -- PANICKED: {}", msg);
                failed.push(("21_guitar_tab".to_string(), msg));
            }
        }
    }

    println!();
    println!(
        "Sample sheet written to: {}",
        dir.display()
    );
    println!("Rendered OK: {}/{}", ok.len(), ok.len() + failed.len());
    if !failed.is_empty() {
        println!("FAILURES ({}):", failed.len());
        for (name, msg) in &failed {
            println!("  - {name}: {msg}");
        }
    }

    #[cfg(not(feature = "png"))]
    println!("PNG: skipped (build without --features png).");
}

#[cfg(feature = "png")]
fn emit_png(dir: &std::path::Path, name: &str, svg: &str) {
    use music_engraver::render::png::PngRenderer;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut renderer = PngRenderer::new(2.0);
        renderer.load_system_fonts();
        renderer.render_png(svg).expect("png render")
    }));
    match result {
        Ok(bytes) => {
            let path = dir.join(format!("{name}.png"));
            std::fs::write(&path, &bytes).expect("write PNG");
            println!("     -> PNG {name}.png ({} bytes)", bytes.len());
        }
        Err(e) => {
            println!("     -> PNG {name} FAILED: {}", panic_message(&e));
        }
    }
}

fn panic_message(e: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic>".to_string()
    }
}
