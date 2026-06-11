#!/usr/bin/env bash
# Practice Sheet Demo
#
# Generates practice sheets for 4 musically useful key+scale combinations:
#   1. C Major — the foundational reference (text + SVG)
#   2. G Major — sharp-key reference, shows transposition (text)
#   3. A Melodic Minor — jazz staple, shows altered modes (text + SVG dark)
#   4. E Harmonic Minor — classical/metal staple (text + SVG print)
#
# Exercises: practice-sheet text output, SVG output with themes,
#            multiple scale families, key transposition.
#
# Usage:
#   SLONIMSKY=target/debug/slonimsky bash slonimsky/examples/scripts/practice_sheet_demo.sh
#
# Output: 7 files in slonimsky/examples/output/

set -euo pipefail

SLONIMSKY="${SLONIMSKY:-cargo run -p slonimsky --}"
OUT="slonimsky/examples/output"
mkdir -p "$OUT"

echo "=== Practice Sheet Demo ==="
echo ""

# --- 1. C Major (text + SVG) ---
echo "1. C Major practice sheet (text)..."
$SLONIMSKY practice-sheet --key C --scale major > "$OUT/practice_sheet_c_major.txt"
echo "   → $(wc -l < "$OUT/practice_sheet_c_major.txt") lines"

echo "   C Major practice sheet (SVG, default theme)..."
$SLONIMSKY practice-sheet --key C --scale major -o "$OUT/practice_sheet_c_major.svg"
echo "   → $(wc -c < "$OUT/practice_sheet_c_major.svg") bytes"

# --- 2. G Major (text only — shows sharp-key transposition) ---
echo "2. G Major practice sheet (text)..."
$SLONIMSKY practice-sheet --key G --scale major > "$OUT/practice_sheet_g_major.txt"
echo "   → $(wc -l < "$OUT/practice_sheet_g_major.txt") lines"

# --- 3. A Melodic Minor (text + SVG dark) ---
echo "3. A Melodic Minor practice sheet (text)..."
$SLONIMSKY practice-sheet --key A --scale melodic-minor > "$OUT/practice_sheet_a_mel_minor.txt"
echo "   → $(wc -l < "$OUT/practice_sheet_a_mel_minor.txt") lines"

echo "   A Melodic Minor practice sheet (SVG, dark theme)..."
$SLONIMSKY practice-sheet --key A --scale melodic-minor --theme dark \
  -o "$OUT/practice_sheet_a_mel_minor_dark.svg"
echo "   → $(wc -c < "$OUT/practice_sheet_a_mel_minor_dark.svg") bytes"

# --- 4. E Harmonic Minor (text + SVG print) ---
echo "4. E Harmonic Minor practice sheet (text)..."
$SLONIMSKY practice-sheet --key E --scale harmonic-minor > "$OUT/practice_sheet_e_harm_minor.txt"
echo "   → $(wc -l < "$OUT/practice_sheet_e_harm_minor.txt") lines"

echo "   E Harmonic Minor practice sheet (SVG, print theme)..."
$SLONIMSKY practice-sheet --key E --scale harmonic-minor --theme print \
  -o "$OUT/practice_sheet_e_harm_minor_print.svg"
echo "   → $(wc -c < "$OUT/practice_sheet_e_harm_minor_print.svg") bytes"

# --- Summary ---
echo ""
echo "=== Summary ==="
echo "Generated 7 practice sheet files:"
echo "  Text:  practice_sheet_c_major.txt"
echo "         practice_sheet_g_major.txt"
echo "         practice_sheet_a_mel_minor.txt"
echo "         practice_sheet_e_harm_minor.txt"
echo "  SVG:   practice_sheet_c_major.svg"
echo "         practice_sheet_a_mel_minor_dark.svg"
echo "         practice_sheet_e_harm_minor_print.svg"
echo ""
echo "Done."
