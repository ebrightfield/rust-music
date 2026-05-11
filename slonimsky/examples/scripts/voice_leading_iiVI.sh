#!/usr/bin/env bash
#
# Voice-Leading Exploration: ii-V-I in C major
#
# Demonstrates the voice-leading and voicing subcommands through a jazz
# musician's lens: given the ubiquitous ii-V-I progression (Dm7-G7-Cmaj7),
# explore canonical voicings for each chord, find the smoothest
# voice-leadings between adjacent chords, and visualize the harmonic
# relationships with common-tone analysis and pitch-circle diagrams.
#
# Musical scenarios covered:
#   1. Voicing catalog: all canonical voicings for Dm7, G7, Cmaj7
#   2. Common tones: shared notes between adjacent chords in the chain
#   3. Voice-leading Dm7→G7: smoothest paths with no voice crossings
#   4. Voice-leading G7→Cmaj7: smoothest paths with no voice crossings
#   5. Full chain comparison: prime form + interval vector for each chord
#   6. Pitch-circle diagrams for all three chords
#
# Output:
#   - Text report: examples/output/voice_leading_iiVI_report.txt
#   - SVG diagrams: examples/output/vl_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/voice_leading_iiVI.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/voice_leading_iiVI.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="cargo run -p slonimsky --"
fi

REPORT="$OUT_DIR/voice_leading_iiVI_report.txt"

{
echo "=========================================="
echo "  Voice-Leading Exploration: ii-V-I in C"
echo "=========================================="
echo ""
echo "Progression: Dm7 (ii) → G7 (V) → Cmaj7 (I)"
echo ""

# --- 1. Voicing catalog ---
echo "=========================================="
echo "  1. Canonical Voicings"
echo "=========================================="
echo ""
echo "--- Dm7 voicings (first 6) ---"
$S voicings D F A C --limit 6
echo ""
echo "--- G7 voicings (first 6) ---"
$S voicings G B D F --limit 6
echo ""
echo "--- Cmaj7 voicings (first 6) ---"
$S voicings C E G B --limit 6
echo ""

# --- 2. Common tones between adjacent chords ---
echo "=========================================="
echo "  2. Common Tones"
echo "=========================================="
echo ""
echo "--- Dm7 ∩ G7 ---"
$S common-tones D,F,A,C G,B,D,F
echo ""
echo "--- G7 ∩ Cmaj7 ---"
$S common-tones G,B,D,F C,E,G,B
echo ""
echo "--- Dm7 ∩ Cmaj7 ---"
$S common-tones D,F,A,C C,E,G,B
echo ""
echo "--- All three (Dm7 ∩ G7 ∩ Cmaj7) ---"
$S common-tones D,F,A,C G,B,D,F C,E,G,B
echo ""

# --- 3. Voice-leading Dm7 → G7 ---
echo "=========================================="
echo "  3. Voice-Leading: Dm7 → G7"
echo "=========================================="
echo ""
echo "--- Smoothest (no crossings), starting from D4 F4 A4 C5 ---"
$S voice-leading --from D4,F4,A4,C5 --to G,B,D,F --no-crossings --limit 5 -v
echo ""
echo "--- All voice-leadings (unconstrained), top 5 by distance ---"
$S voice-leading --from D4,F4,A4,C5 --to G,B,D,F --limit 5
echo ""

# --- 4. Voice-leading G7 → Cmaj7 ---
echo "=========================================="
echo "  4. Voice-Leading: G7 → Cmaj7"
echo "=========================================="
echo ""
echo "--- Smoothest (no crossings), starting from G3 B3 D4 F4 ---"
$S voice-leading --from G3,B3,D4,F4 --to C,E,G,B --no-crossings --limit 5 -v
echo ""
echo "--- Smoothest (no crossings), starting from D4 F4 G4 B4 ---"
echo "    (This voicing is the smoothest Dm7→G7 result from section 3)"
$S voice-leading --from D4,F4,G4,B4 --to C,E,G,B --no-crossings --limit 5
echo ""

# --- 5. Set-class comparison ---
echo "=========================================="
echo "  5. Set-Class Comparison"
echo "=========================================="
echo ""
echo "--- Dm7 ---"
$S prime-form D F A C -v
echo ""
echo "--- G7 ---"
$S prime-form G B D F -v
echo ""
echo "--- Cmaj7 ---"
$S prime-form C E G B -v
echo ""
echo "(Note: Dm7 [0,3,5,8] = 4-26, G7 [0,2,5,8] = 4-27, Cmaj7 [0,1,5,8] = 4-20."
echo " All three are distinct set classes, but share ic3=2 and ic5≥1 content.)"
echo ""

# --- 6. Interval vectors ---
echo "--- Dm7 interval vector ---"
$S interval-vector D F A C
echo ""
echo "--- G7 interval vector ---"
$S interval-vector G B D F
echo ""
echo "--- Cmaj7 interval vector ---"
$S interval-vector C E G B
echo ""

echo "=========================================="
echo "  END OF REPORT"
echo "=========================================="
} > "$REPORT" 2>&1

echo "Report written to: $REPORT"

# --- SVG diagrams ---
$S pitch-circle D F A C --title "Dm7 (ii)" -o "$OUT_DIR/vl_dm7_circle.svg"
echo "SVG: vl_dm7_circle.svg"

$S pitch-circle G B D F --title "G7 (V)" --theme dark -o "$OUT_DIR/vl_g7_circle.svg"
echo "SVG: vl_g7_circle.svg"

$S pitch-circle C E G B --title "Cmaj7 (I)" -o "$OUT_DIR/vl_cmaj7_circle.svg"
echo "SVG: vl_cmaj7_circle.svg"

$S pitch-circle D F A C --show-intervals --title "Dm7 intervals" --theme print -o "$OUT_DIR/vl_dm7_intervals.svg"
echo "SVG: vl_dm7_intervals.svg"

echo ""
echo "Done. 4 SVGs + 1 text report in $OUT_DIR"
