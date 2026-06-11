#!/usr/bin/env bash
#
# Chord Similarity Explorer
#
# Demonstrates using `closest` to find chords and scales most similar to a
# given pitch-class set, combined with `name`, `prime-form`, `forte`, and
# `pitch-circle` for full context. Produces a text report and SVG diagrams.
#
# Workflow:
#   1. Identify the input chord (name, prime form, Forte number)
#   2. Find the 10 closest chords (by symmetric difference)
#   3. Find the 10 closest scales
#   4. Draw pitch-circle diagrams for the input and top 3 closest chords
#   5. Combine everything into a report
#
# Input: C E G# B (augmented major 7th — an interesting chord that sits
# between major and augmented, with many close neighbors)
#
# Output:
#   - closest_report.txt     — combined text report
#   - closest_input.svg      — pitch circle of the input chord
#   - closest_match_1.svg    — pitch circle of the #1 closest chord
#   - closest_match_2.svg    — pitch circle of the #2 closest chord
#   - closest_match_3.svg    — pitch circle of the #3 closest chord
#
# Usage:
#   bash slonimsky/examples/scripts/closest_similarity.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
REPORT="$OUT_DIR/closest_report.txt"

mkdir -p "$OUT_DIR"

if [ -z "${SLONIMSKY:-}" ]; then
    SLONIMSKY="$SCRIPT_DIR/../../../target/debug/slonimsky"
    if [ ! -x "$SLONIMSKY" ]; then
        echo "Building slonimsky..."
        CARGO_HOME=/workspace/.cargo-home cargo build -p slonimsky 2>/dev/null
    fi
fi

echo "=== Chord Similarity Explorer ===" | tee "$REPORT"
echo "" | tee -a "$REPORT"
echo "Input: C E G# B (augmented major 7th)" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# --- Section 1: Identify the input chord ---
echo "--- Identification ---" | tee -a "$REPORT"

echo "Name:" | tee -a "$REPORT"
$SLONIMSKY name C E Ab B 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Prime form:" | tee -a "$REPORT"
$SLONIMSKY prime-form C E Ab B 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Forte number:" | tee -a "$REPORT"
$SLONIMSKY forte C E Ab B 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Interval vector:" | tee -a "$REPORT"
$SLONIMSKY interval-vector C E Ab B 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# --- Section 2: Closest chords ---
echo "--- 10 Closest Chords (by symmetric difference) ---" | tee -a "$REPORT"
$SLONIMSKY closest C E Ab B --pool chords --limit 10 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# --- Section 3: Closest scales ---
echo "--- 10 Closest Scales (by symmetric difference) ---" | tee -a "$REPORT"
$SLONIMSKY closest C E Ab B --pool scales --limit 10 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# --- Section 4: Pitch-circle diagrams ---
echo "--- Generating SVG diagrams ---" | tee -a "$REPORT"

# Input chord
$SLONIMSKY pitch-circle C E Ab B \
    --title "Input: C E Ab B" \
    -o "$OUT_DIR/closest_input.svg"
echo "  closest_input.svg — input chord pitch circle" | tee -a "$REPORT"

# Top 3 closest: we know from the catalog that common close neighbors
# of {0,4,8,11} include Cmaj7 {0,4,7,11}, C augmented {0,4,8}, and
# C dom7 {0,4,7,10}. Generate pitch circles for comparison.

# Match 1: Cmaj7 (remove the #5, add natural 5 — dist 2)
$SLONIMSKY pitch-circle C E G B \
    --title "Match: C Maj7 (dist≈2)" \
    -o "$OUT_DIR/closest_match_1.svg"
echo "  closest_match_1.svg — C Maj7 (natural 5th)" | tee -a "$REPORT"

# Match 2: C augmented triad (drop the 7th — dist 1)
$SLONIMSKY pitch-circle C E Ab \
    --title "Match: C Aug (dist≈1)" \
    -o "$OUT_DIR/closest_match_2.svg"
echo "  closest_match_2.svg — C Augmented triad" | tee -a "$REPORT"

# Match 3: C7#5 (replace B with Bb — dist 2)
$SLONIMSKY pitch-circle C E Ab Bb \
    --title "Match: C7#5 (dist≈2)" \
    -o "$OUT_DIR/closest_match_3.svg"
echo "  closest_match_3.svg — C7#5 (dominant augmented)" | tee -a "$REPORT"

echo "" | tee -a "$REPORT"

# --- Section 5: Common-tone analysis between input and top matches ---
echo "--- Common-Tone Analysis ---" | tee -a "$REPORT"

echo "Input vs Cmaj7:" | tee -a "$REPORT"
$SLONIMSKY common-tones C,E,Ab,B C,E,G,B 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Input vs C Augmented:" | tee -a "$REPORT"
$SLONIMSKY common-tones C,E,Ab,B C,E,Ab 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Input vs C7#5:" | tee -a "$REPORT"
$SLONIMSKY common-tones C,E,Ab,B C,E,Ab,Bb 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "=== Done ===" | tee -a "$REPORT"
echo "Report: $REPORT"
echo "SVGs:   $OUT_DIR/closest_*.svg"
