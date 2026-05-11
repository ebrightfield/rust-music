#!/usr/bin/env bash
# scale_book_demo.sh — Generate scale-book outputs for multiple scale families.
#
# Use case: practice-material generation. Produces text reference sheets
# and SVG pitch-circle grids for all modes of major, melodic-minor,
# harmonic-minor, and harmonic-major scales across selected keys.
#
# Produces:
#   slonimsky/examples/output/scale_book_major_text.txt
#   slonimsky/examples/output/scale_book_major_C_G.svg
#   slonimsky/examples/output/scale_book_melodic_minor_text.txt
#   slonimsky/examples/output/scale_book_melodic_minor_A.svg
#   slonimsky/examples/output/scale_book_harmonic_minor_text.txt
#   slonimsky/examples/output/scale_book_harmonic_minor_E.svg
#   slonimsky/examples/output/scale_book_harmonic_major_text.txt
#   slonimsky/examples/output/scale_book_all_families_report.txt
#
# Invoke: bash slonimsky/examples/scripts/scale_book_demo.sh
#   or:   SLONIMSKY=/path/to/binary bash slonimsky/examples/scripts/scale_book_demo.sh

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

echo "=== Scale Book Demo ==="

# 1. Major scale — text reference for all 12 keys
echo "  Major scale (all 12 keys, text)..."
$SLON scale-book major > "$OUT_DIR/scale_book_major_text.txt"

# 2. Major scale — SVG grid for C and G (14 diagrams: 7 modes × 2 keys)
echo "  Major scale (C, G keys, SVG)..."
$SLON scale-book major --keys C,G -o "$OUT_DIR/scale_book_major_C_G.svg"

# 3. Melodic minor — text reference for keys A, D, G
echo "  Melodic minor (A, D, G keys, text)..."
$SLON scale-book melodic-minor --keys A,D,G > "$OUT_DIR/scale_book_melodic_minor_text.txt"

# 4. Melodic minor — SVG for key of A (dark theme, 7 diagrams)
echo "  Melodic minor (A, SVG dark)..."
$SLON scale-book melodic-minor --keys A -t dark -o "$OUT_DIR/scale_book_melodic_minor_A.svg"

# 5. Harmonic minor — text reference for E, B keys
echo "  Harmonic minor (E, B keys, text)..."
$SLON scale-book harmonic-minor --keys E,B > "$OUT_DIR/scale_book_harmonic_minor_text.txt"

# 6. Harmonic minor — SVG for key of E (print theme)
echo "  Harmonic minor (E, SVG print)..."
$SLON scale-book harmonic-minor --keys E -t print -o "$OUT_DIR/scale_book_harmonic_minor_E.svg"

# 7. Harmonic major — text reference for C key only
echo "  Harmonic major (C, text)..."
$SLON scale-book harmonic-major --keys C > "$OUT_DIR/scale_book_harmonic_major_text.txt"

# 8. Combined report: all four families for key of C, concatenated
echo "  Combined report (all families, key of C)..."
{
  echo "============================================"
  echo "SCALE BOOK: Combined report — Key of C"
  echo "============================================"
  echo ""
  for family in major melodic-minor harmonic-minor harmonic-major; do
    $SLON scale-book "$family" --keys C
    echo ""
  done
} > "$OUT_DIR/scale_book_all_families_report.txt"

echo ""
echo "=== Done. Outputs: ==="
ls -lh "$OUT_DIR"/scale_book_*
echo ""
echo "Text files contain mode-by-mode note spellings."
echo "SVG files contain pitch-circle grids (one circle per mode)."
