#!/usr/bin/env bash
# pitch_circle_gallery.sh — Generate a gallery of pitch-circle SVGs
# demonstrating common chord/scale structures across themes.
#
# Use case: quick visual reference for pitch-class geometry of triads,
# seventh chords, symmetric scales, and chromatic clusters.
#
# Produces: slonimsky/examples/output/pc_*.svg (one per diagram)
#
# Invoke: bash slonimsky/examples/scripts/pitch_circle_gallery.sh
#   or:   SLONIMSKY=/path/to/binary bash slonimsky/examples/scripts/pitch_circle_gallery.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

# Use pre-built binary if SLONIMSKY env var is set, otherwise cargo run
if [ -n "${SLONIMSKY:-}" ]; then
  SLON="$SLONIMSKY"
else
  cargo build -p slonimsky --quiet 2>/dev/null || cargo build -p slonimsky
  SLON="cargo run -p slonimsky --quiet --"
fi

echo "=== Pitch-circle gallery ==="

# 1. Major triad (default theme)
echo "  C major triad..."
$SLON pitch-circle C E G \
  --title "C Major Triad" --show-intervals \
  -o "$OUT_DIR/pc_c_major_triad.svg"

# 2. Minor seventh chord (dark theme)
echo "  A minor 7..."
$SLON pitch-circle A C E G \
  --title "Am7" --show-intervals --theme dark \
  -o "$OUT_DIR/pc_am7_dark.svg"

# 3. Diminished seventh — fully symmetric (print theme)
echo "  Diminished 7 (symmetric)..."
$SLON pitch-circle 0 3 6 9 \
  --title "Dim7 (0,3,6,9)" --show-intervals --theme print \
  -o "$OUT_DIR/pc_dim7_print.svg"

# 4. Augmented triad — another symmetric structure
echo "  Augmented triad..."
$SLON pitch-circle 0 4 8 \
  --title "Aug (0,4,8)" --show-intervals \
  -o "$OUT_DIR/pc_aug_triad.svg"

# 5. Whole-tone scale (colorful theme)
echo "  Whole-tone scale..."
$SLON pitch-circle 0 2 4 6 8 10 \
  --title "Whole-Tone Scale" --show-intervals --theme colorful \
  -o "$OUT_DIR/pc_whole_tone_colorful.svg"

# 6. Chromatic cluster (first 5 pcs) — dense region
echo "  Chromatic cluster 0-4..."
$SLON pitch-circle 0 1 2 3 4 \
  --title "Chromatic Cluster (0–4)" \
  -o "$OUT_DIR/pc_chromatic_cluster.svg"

# 7. Pentatonic scale (major pentatonic from C)
echo "  C major pentatonic..."
$SLON pitch-circle C D E G A \
  --title "C Major Pentatonic" --show-intervals \
  -o "$OUT_DIR/pc_c_pentatonic.svg"

# 8. Dominant 7#9 — the Hendrix chord
echo "  Dominant 7#9..."
$SLON pitch-circle 0 4 7 10 3 \
  --title "Dom7#9 (0,3,4,7,10)" --show-intervals --theme dark \
  -o "$OUT_DIR/pc_dom7sharp9_dark.svg"

echo ""
echo "Done. Output in: $OUT_DIR/"
ls -1 "$OUT_DIR"/pc_*.svg
