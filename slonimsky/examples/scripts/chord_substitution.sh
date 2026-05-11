#!/usr/bin/env bash
#
# Chord Substitution Explorer
#
# Demonstrates a practical reharmonization workflow: given a target chord
# (G dominant 7th, the V chord in C major), systematically discover
# substitute chords using set-theoretic and commonality tools. This is how
# a jazz arranger might explore reharmonization options.
#
# Musical scenarios covered:
#   1. Identify the target chord's set class and symmetry properties
#   2. Find the closest related chords by symmetric difference
#   3. Find scales that contain the target chord (for modal interchange)
#   4. Extract all triadic subchords (rootless voicing candidates)
#   5. Tritone substitution: compare G7 and Db7 via common-tones
#   6. Pitch-circle and interval-matrix diagrams for visual comparison
#
# Output:
#   - Text report: examples/output/chord_substitution_report.txt
#   - SVG diagrams: examples/output/sub_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/chord_substitution.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="cargo run -p slonimsky --"
fi

REPORT="$OUT_DIR/chord_substitution_report.txt"

{
echo "================================================="
echo "  Chord Substitution Explorer: G7 (V in C major)"
echo "================================================="
echo ""
echo "Target chord: G7 = G B D F (PCs: 7 11 2 5)"
echo ""

# --- 1. Target chord analysis ---
echo "================================================="
echo "  1. Set-Class Identification"
echo "================================================="
echo ""
echo "--- Prime form and Forte number ---"
$S forte G B D F
echo ""
echo "--- Symmetry properties ---"
$S orbits G B D F
echo ""
echo "--- Interval vector ---"
$S interval-vector G B D F
echo ""

# --- 2. Closest related chords ---
echo "================================================="
echo "  2. Closest Chords (by symmetric difference)"
echo "================================================="
echo ""
echo "The nearest chords share maximum common tones with G7."
echo "Each is a potential substitution or chromatic alteration."
echo ""
$S closest G B D F --limit 10
echo ""

# --- 3. Scales containing G7 ---
echo "================================================="
echo "  3. Scales Containing G7"
echo "================================================="
echo ""
echo "These scales support G7 as a diatonic chord. Each suggests"
echo "a different modal color for reharmonization."
echo ""
$S contains G B D F --limit 10
echo ""

# --- 4. Triadic subchords (rootless voicings) ---
echo "================================================="
echo "  4. Triadic Subchords of G7"
echo "================================================="
echo ""
echo "3-note subsets of G7 reveal the inner triads. Rootless"
echo "voicings (omitting the root G) are common in jazz piano."
echo ""
$S subchords G B D F --size 3 --name
echo ""

# --- 5. Tritone substitution: G7 vs Db7 ---
echo "================================================="
echo "  5. Tritone Substitution: G7 vs Db7"
echo "================================================="
echo ""
echo "The tritone sub replaces V7 with bII7 (a tritone away)."
echo "G7 = {G,B,D,F}   Db7 = {Db,F,Ab,Cb/B}"
echo ""
echo "--- Common tones (the shared tritone B-F) ---"
$S common-tones G,B,D,F Db,F,Ab,B
echo ""
echo "--- G7 prime form ---"
$S prime-form G B D F
echo ""
echo "--- Db7 prime form ---"
$S prime-form Db F Ab B
echo ""
echo "(Both are Forte 4-27 [0,2,5,8] — identical interval content"
echo " is why the tritone substitution works so smoothly.)"
echo ""

# --- 6. Extended dominant: subchords of G9 ---
echo "================================================="
echo "  6. Extended Dominant: Subchords of G9"
echo "================================================="
echo ""
echo "G9 = G B D F A. Its subchords include triads that can"
echo "function as rootless dominant voicings."
echo ""
echo "--- 3-note subchords ---"
$S subchords G B D F A --size 3 --name
echo ""
echo "--- 4-note subchords ---"
$S subchords G B D F A --size 4 --name
echo ""

# --- 7. Comparison: G7 vs common substitutes ---
echo "================================================="
echo "  7. Common-Tone Analysis: G7 vs Substitutes"
echo "================================================="
echo ""
echo "--- G7 ∩ Bdim7 (vii°7, shares 3 tones) ---"
$S common-tones G,B,D,F B,D,F,Ab
echo ""
echo "--- G7 ∩ Dm7 (ii7, pre-dominant partner) ---"
$S common-tones G,B,D,F D,F,A,C
echo ""
echo "--- G7 ∩ Em7 (iii7, relative sub) ---"
$S common-tones G,B,D,F E,G,B,D
echo ""
echo "--- G7 ∩ Bb7 (bIII7, backdoor dominant) ---"
$S common-tones G,B,D,F Bb,D,F,Ab
echo ""

# --- 8. Interval vector comparison ---
echo "================================================="
echo "  8. Interval Vector Comparison"
echo "================================================="
echo ""
echo "--- G7 (dominant 7th) ---"
$S interval-vector G B D F
echo ""
echo "--- Bdim (diminished triad from G7) ---"
$S interval-vector B D F
echo ""
echo "--- Dm (ii chord, pre-dominant partner) ---"
$S interval-vector D F A
echo ""

echo "================================================="
echo "  END OF REPORT"
echo "================================================="
} > "$REPORT" 2>&1

echo "Report written to: $REPORT"

# --- SVG diagrams ---
$S pitch-circle G B D F --title "G7 (original V7)" -o "$OUT_DIR/sub_g7_circle.svg"
echo "SVG: sub_g7_circle.svg"

$S pitch-circle Db F Ab B --title "Db7 (tritone sub)" --theme dark -o "$OUT_DIR/sub_db7_circle.svg"
echo "SVG: sub_db7_circle.svg"

$S pitch-circle B D F --title "Bdim (shared tritone)" --show-intervals -o "$OUT_DIR/sub_bdim_circle.svg"
echo "SVG: sub_bdim_circle.svg"

$S pitch-circle G B D F A --title "G9 (extended dom)" --theme colorful -o "$OUT_DIR/sub_g9_circle.svg"
echo "SVG: sub_g9_circle.svg"

$S interval-matrix G B D F --title "G7 interval matrix" -o "$OUT_DIR/sub_g7_matrix.svg"
echo "SVG: sub_g7_matrix.svg"

$S interval-matrix Db F Ab B --title "Db7 interval matrix" --theme dark -o "$OUT_DIR/sub_db7_matrix.svg"
echo "SVG: sub_db7_matrix.svg"

echo ""
echo "Done. 6 SVGs + 1 text report in $OUT_DIR"
