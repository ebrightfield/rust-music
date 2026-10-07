#![cfg(feature = "png")]

use music::notation::rhythm::duration::Duration;
use music::note::{note::Note, pitch::Pitch};
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::render::png::PngRenderer;
use music_engraver::score::ScoreBuilder;
use resvg::tiny_skia::Pixmap;

const BASELINE: &[u8] = include_bytes!("fixtures/engraving_perceptual.png");

fn corpus() -> ScoreBuilder {
    let pitch = |note, octave| Pitch::new(note, octave);
    ScoreBuilder::new()
        .measures_per_system(1)
        .note(pitch(Note::C, 3), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .cresc()
        .note(pitch(Note::D, 3), Duration::QTR)
        .note(pitch(Note::C, 3), Duration::HALF)
        .hairpin_end()
        .barline()
        .beam_group(vec![
            (pitch(Note::E, 4), Duration::SIXTEENTH),
            (pitch(Note::C, 5), Duration::SIXTEENTH),
            (pitch(Note::G, 4), Duration::EIGHTH),
            (pitch(Note::E, 4), Duration::SIXTEENTH),
            (pitch(Note::G, 4), Duration::EIGHTH),
        ])
        .note(pitch(Note::Fis, 4), Duration::QTR)
        .end_barline()
}

fn actual_png() -> Vec<u8> {
    PngRenderer::new(1.0)
        .render_png(&corpus().render_svg())
        .unwrap()
}

fn ink(pixels: &[u8], width: usize, height: usize, x: usize, y: usize) -> bool {
    x < width && y < height && pixels[(y * width + x) * 4 + 3] >= 128
}

fn nearby_ink(pixels: &[u8], width: usize, height: usize, x: usize, y: usize) -> bool {
    let x_min = x.saturating_sub(1);
    let y_min = y.saturating_sub(1);
    (y_min..=(y + 1).min(height - 1))
        .any(|row| (x_min..=(x + 1).min(width - 1)).any(|col| ink(pixels, width, height, col, row)))
}

#[test]
fn bundled_font_score_matches_perceptual_png_baseline() {
    let expected = Pixmap::decode_png(BASELINE).unwrap();
    let actual = Pixmap::decode_png(&actual_png()).unwrap();
    assert_eq!(
        (actual.width(), actual.height()),
        (expected.width(), expected.height())
    );

    // Alpha silhouettes ignore encoder bytes and antialiasing intensity. A
    // one-pixel neighborhood tolerates small rasterizer edge differences,
    // while local tiles catch a missing beam, ledger, or hairpin even if the
    // rest of this two-system score still occupies the same page.
    let width = actual.width() as usize;
    let height = actual.height() as usize;
    let columns = 32;
    let rows = 16;
    let mut ink_by_tile = vec![0usize; columns * rows];
    let mut mismatches = vec![0usize; columns * rows];
    let expected_pixels = expected.data();
    let actual_pixels = actual.data();
    for y in 0..height {
        for x in 0..width {
            let tile = (y * rows / height) * columns + (x * columns / width);
            let in_expected = ink(expected_pixels, width, height, x, y);
            let in_actual = ink(actual_pixels, width, height, x, y);
            ink_by_tile[tile] += usize::from(in_expected || in_actual);
            if (in_expected && !nearby_ink(actual_pixels, width, height, x, y))
                || (in_actual && !nearby_ink(expected_pixels, width, height, x, y))
            {
                mismatches[tile] += 1;
            }
        }
    }
    let total_ink: usize = ink_by_tile.iter().sum();
    let total_mismatch: usize = mismatches.iter().sum();
    assert!(
        total_mismatch * 100 <= total_ink * 3,
        "{total_mismatch} of {total_ink} ink pixels differ"
    );
    for (tile, (&mismatch, &tile_ink)) in mismatches.iter().zip(&ink_by_tile).enumerate() {
        assert!(
            mismatch <= 12 || mismatch * 4 <= tile_ink,
            "tile {tile}: {mismatch} unmatched ink pixels among {tile_ink}"
        );
    }
}

/// Explicit baseline refresh after reviewing the generated image.
/// Run `cargo test -p music-engraver --features png --test perceptual_png refresh_baseline -- --ignored`.
#[test]
#[ignore]
fn refresh_baseline() {
    std::fs::write("tests/fixtures/engraving_perceptual.png", actual_png()).unwrap();
}
