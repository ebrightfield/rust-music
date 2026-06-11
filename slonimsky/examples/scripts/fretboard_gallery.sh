#!/usr/bin/env bash
# fretboard_gallery.sh — Generate a gallery of fretboard chord-shape SVGs
# demonstrating common open and barre chords across tunings and themes.
#
# Use case: visual chord-shape reference sheet for guitar and bass players,
# showing standard shapes, alternative tunings, and display orientations.
#
# Produces: slonimsky/examples/output/fb_*.svg (one per diagram)
#
# Invoke: bash slonimsky/examples/scripts/fretboard_gallery.sh
#   or:   SLONIMSKY=/path/to/binary bash slonimsky/examples/scripts/fretboard_gallery.sh

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

echo "=== Fretboard gallery ==="

# --- Standard tuning open chords ---

echo "  C major (open)..."
$SLON fretboard x-3-2-0-1-0 \
  --title "C Major" \
  -o "$OUT_DIR/fb_c_major_open.svg"

echo "  A minor (open)..."
$SLON fretboard x-0-2-2-1-0 \
  --title "A Minor" \
  -o "$OUT_DIR/fb_a_minor_open.svg"

echo "  G major (open)..."
$SLON fretboard 3-2-0-0-0-3 \
  --title "G Major" \
  -o "$OUT_DIR/fb_g_major_open.svg"

echo "  E minor (open)..."
$SLON fretboard 0-2-2-0-0-0 \
  --title "E Minor" \
  -o "$OUT_DIR/fb_e_minor_open.svg"

echo "  D major (open)..."
$SLON fretboard x-x-0-2-3-2 \
  --title "D Major" \
  -o "$OUT_DIR/fb_d_major_open.svg"

# --- Barre chords ---

echo "  F major barre (dark theme)..."
$SLON fretboard 1-3-3-2-1-1 \
  --title "F Major (Barre)" --theme dark \
  -o "$OUT_DIR/fb_f_major_barre_dark.svg"

echo "  Bb major barre (dark theme)..."
$SLON fretboard x-1-3-3-3-1 \
  --title "Bb Major (Barre)" --theme dark \
  -o "$OUT_DIR/fb_bb_major_barre_dark.svg"

# --- Themes ---

echo "  Am7 (print theme)..."
$SLON fretboard x-0-2-0-1-0 \
  --title "Am7" --theme print \
  -o "$OUT_DIR/fb_am7_print.svg"

echo "  Em pentatonic box (colorful theme)..."
$SLON fretboard 0-3-0-0-0-3 \
  --title "Em Pentatonic (Open)" --theme colorful \
  -o "$OUT_DIR/fb_em_pent_colorful.svg"

# --- Alternative tunings ---

echo "  Drop-D power chord..."
$SLON fretboard 0-0-x-x-x-x \
  --title "D5 (Drop-D)" --tuning drop-d \
  -o "$OUT_DIR/fb_d5_drop_d.svg"

echo "  DADGAD open chord..."
$SLON fretboard 0-0-0-0-0-0 \
  --title "Dsus4 (DADGAD Open)" --tuning dadgad \
  -o "$OUT_DIR/fb_dsus4_dadgad.svg"

# --- Orientation ---

echo "  E major horizontal..."
$SLON fretboard 0-2-2-1-0-0 \
  --title "E Major (Horizontal)" --orientation horizontal \
  -o "$OUT_DIR/fb_e_major_horizontal.svg"

echo ""
echo "Done. Output in: $OUT_DIR/"
ls -1 "$OUT_DIR"/fb_*.svg
