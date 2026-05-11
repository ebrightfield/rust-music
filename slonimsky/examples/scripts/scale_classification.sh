#!/usr/bin/env bash
# Scale Classification — Forte set-class analysis of common 7-note scales
#
# Demonstrates: forte, prime-form, orbits, interval-vector, common-tones,
#               pitch-circle (SVG output)
#
# Classifies 8 important scale types by their Forte number, compares their
# interval vectors, finds which scales share a set class (TnI equivalence),
# and generates pitch-circle SVGs for each.
#
# Usage:
#   SLONIMSKY=target/debug/slonimsky bash slonimsky/examples/scripts/scale_classification.sh
#
# Output:
#   slonimsky/examples/output/scale_classification_report.txt
#   slonimsky/examples/output/scale_*.svg

set -euo pipefail

SLONIMSKY="${SLONIMSKY:-cargo run -p slonimsky --}"
OUTDIR="slonimsky/examples/output"
REPORT="$OUTDIR/scale_classification_report.txt"
mkdir -p "$OUTDIR"

# Scales: name, PCs (space-separated integers)
declare -a SCALE_NAMES=(
  "C Major (Ionian)"
  "C Natural Minor (Aeolian)"
  "C Harmonic Minor"
  "C Melodic Minor (ascending)"
  "C Whole-Tone"
  "C Octatonic (half-whole)"
  "C Harmonic Major"
  "C Hungarian Minor"
)
declare -a SCALE_PCS=(
  "0 2 4 5 7 9 11"
  "0 2 3 5 7 8 10"
  "0 2 3 5 7 8 11"
  "0 2 3 5 7 9 11"
  "0 2 4 6 8 10"
  "0 1 3 4 6 7 9 10"
  "0 2 4 5 7 8 11"
  "0 2 3 6 7 8 11"
)
declare -a SCALE_TAGS=(
  "major"
  "nat_minor"
  "harm_minor"
  "mel_minor"
  "whole_tone"
  "octatonic"
  "harm_major"
  "hungarian"
)

{
echo "============================================"
echo "  SCALE CLASSIFICATION — Forte Set-Class Analysis"
echo "============================================"
echo ""

# --- Section 1: Classify each scale ---
echo "=== 1. Forte Classification of Common Scales ==="
echo ""

for i in "${!SCALE_NAMES[@]}"; do
  name="${SCALE_NAMES[$i]}"
  pcs="${SCALE_PCS[$i]}"
  tag="${SCALE_TAGS[$i]}"

  echo "--- $name ---"
  echo "PCs: {$pcs}"
  $SLONIMSKY forte $pcs -v 2>/dev/null || true
  echo ""

  # Generate pitch-circle SVG with intervals shown
  $SLONIMSKY pitch-circle $pcs \
    --show-intervals \
    --title "$name" \
    -o "$OUTDIR/scale_${tag}_circle.svg" 2>/dev/null
done

# --- Section 2: Set-class equivalences ---
echo ""
echo "=== 2. Set-Class Equivalences (TnI) ==="
echo ""
echo "Scales sharing a Forte number are related by transposition and/or inversion."
echo ""

echo "Major vs Natural Minor:"
echo "  Major:   $($SLONIMSKY forte 0 2 4 5 7 9 11 2>/dev/null | grep 'Forte:')"
echo "  N.Minor: $($SLONIMSKY forte 0 2 3 5 7 8 10 2>/dev/null | grep 'Forte:')"
echo "  → Same set class? Both are 7-35 (the diatonic collection)."
echo ""

echo "Harmonic Minor vs Harmonic Major:"
echo "  H.Minor: $($SLONIMSKY forte 0 2 3 5 7 8 11 2>/dev/null | grep 'Forte:')"
echo "  H.Major: $($SLONIMSKY forte 0 2 4 5 7 8 11 2>/dev/null | grep 'Forte:')"
echo "  → Compare: same or different set class?"
echo ""

echo "Melodic Minor vs Hungarian Minor:"
echo "  M.Minor: $($SLONIMSKY forte 0 2 3 5 7 9 11 2>/dev/null | grep 'Forte:')"
echo "  Hung:    $($SLONIMSKY forte 0 2 3 6 7 8 11 2>/dev/null | grep 'Forte:')"
echo ""

# --- Section 3: Interval content comparison ---
echo ""
echo "=== 3. Interval Vector Comparison ==="
echo ""
echo "The interval vector (IC1..IC6) reveals the intervallic DNA of each scale."
echo "Notation: m2/M7  M2/m7  m3/M6  M3/m6  P4/P5  TT"
echo ""

for i in "${!SCALE_NAMES[@]}"; do
  name="${SCALE_NAMES[$i]}"
  pcs="${SCALE_PCS[$i]}"
  iv_line=$($SLONIMSKY interval-vector $pcs 2>/dev/null | grep 'Interval vector:')
  printf "  %-35s %s\n" "$name" "$iv_line"
done

echo ""
echo "Notes:"
echo "  - Diatonic (7-35) has the unique property of maximal evenness."
echo "  - Whole-tone (6-35) has no m2, m3, P4, or TT — only M2, M3, and TT."
echo "  - Octatonic (8-28) has high symmetry and equal distribution."

# --- Section 4: Symmetry analysis ---
echo ""
echo ""
echo "=== 4. Symmetry Properties ==="
echo ""

echo "Whole-tone scale (transpositionally symmetric):"
$SLONIMSKY orbits 0 2 4 6 8 10 2>/dev/null
echo ""

echo "Octatonic scale (transpositionally symmetric):"
$SLONIMSKY orbits 0 1 3 4 6 7 9 10 2>/dev/null
echo ""

echo "Major scale (no transpositional symmetry):"
$SLONIMSKY orbits 0 2 4 5 7 9 11 2>/dev/null
echo ""

# --- Section 5: Common tones between parallel scales ---
echo ""
echo "=== 5. Common Tones Between Parallel Scales ==="
echo ""

echo "C Major vs C Natural Minor (parallel major/minor):"
$SLONIMSKY common-tones 0,2,4,5,7,9,11 0,2,3,5,7,8,10 2>/dev/null
echo ""

echo "C Major vs C Harmonic Minor (raised 7th):"
$SLONIMSKY common-tones 0,2,4,5,7,9,11 0,2,3,5,7,8,11 2>/dev/null
echo ""

echo "C Major vs C Melodic Minor (raised 6th and 7th):"
$SLONIMSKY common-tones 0,2,4,5,7,9,11 0,2,3,5,7,9,11 2>/dev/null
echo ""

echo "C Harmonic Minor vs C Melodic Minor (differ by one note):"
$SLONIMSKY common-tones 0,2,3,5,7,8,11 0,2,3,5,7,9,11 2>/dev/null
echo ""

echo "C Major vs C Harmonic Major (lowered 6th):"
$SLONIMSKY common-tones 0,2,4,5,7,9,11 0,2,4,5,7,8,11 2>/dev/null
echo ""

# --- Section 6: Subchord content ---
echo ""
echo "=== 6. Triad Content of Selected Scales ==="
echo ""

echo "Major scale — named triads (C(7,3) = 35 subsets, showing first 15):"
$SLONIMSKY subchords 0 2 4 5 7 9 11 --size 3 --name 2>/dev/null || true
echo ""

echo "Harmonic minor — named triads:"
$SLONIMSKY subchords 0 2 3 5 7 8 11 --size 3 --name 2>/dev/null || true
echo ""

echo "============================================"
echo "  End of Scale Classification Report"
echo "============================================"

} > "$REPORT" 2>&1

echo "Report: $REPORT ($(wc -l < "$REPORT") lines)"
echo "SVGs generated:"
ls -1 "$OUTDIR"/scale_*_circle.svg 2>/dev/null | while read f; do
  echo "  $f ($(wc -c < "$f") bytes)"
done
echo "Done."
