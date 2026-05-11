#!/usr/bin/env bash
# containment_explorer.sh — Chord↔scale containment queries and comparison.
#
# Exercises: contains, common-tones, superchords, subchords, pitch-circle
#
# Workflow: Given a chord, find which scales contain it. Given a scale,
# find which chords live inside it. Cross-reference with common-tone
# analysis and pitch-circle diagrams.
#
# Use case: A guitarist wants to know what scales work over a Cmaj7 chord,
# what triads are available in D dorian, and how two related scales
# compare in their chord content.
#
# Produces: slonimsky/examples/output/contain_*.svg + containment_report.txt
#
# Invoke: bash slonimsky/examples/scripts/containment_explorer.sh
#   or:   SLONIMSKY=/path/to/binary bash slonimsky/examples/scripts/containment_explorer.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  SLON="$SLONIMSKY"
else
  CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo build -p slonimsky --quiet 2>/dev/null \
    || CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo build -p slonimsky
  SLON="CARGO_HOME=${CARGO_HOME:-/tmp/cargo-home} cargo run -p slonimsky --quiet --"
fi

run() { eval "$SLON" "$@"; }

REPORT="$OUT_DIR/containment_report.txt"
: > "$REPORT"

banner() {
  echo "" >> "$REPORT"
  echo "================================================================" >> "$REPORT"
  echo "  $1" >> "$REPORT"
  echo "================================================================" >> "$REPORT"
  echo "" >> "$REPORT"
}

echo "=== Containment Explorer ==="

# ── 1. "Which scales contain Cmaj7?" ────────────────────────────────
banner "1. Scales containing Cmaj7 {0, 4, 7, 11}"
echo "  [1/6] Scales containing Cmaj7..."

echo "Query: contains C E G B --in scales" >> "$REPORT"
echo "" >> "$REPORT"
run contains C E G B --in scales >> "$REPORT" 2>/dev/null

run pitch-circle C E G B --title Cmaj7 -o "$OUT_DIR/contain_cmaj7_circle.svg"

# ── 2. "Which chords live in C major scale?" ────────────────────────
banner "2. Chords contained in C major scale {0, 2, 4, 5, 7, 9, 11}"
echo "  [2/6] Chords in C major scale..."

echo "Query: contains C D E F G A B --in chords" >> "$REPORT"
echo "" >> "$REPORT"
run contains C D E F G A B --in chords >> "$REPORT" 2>/dev/null

run pitch-circle C D E F G A B --title C-Major-Scale \
  -o "$OUT_DIR/contain_c_major_scale.svg"

# ── 3. "Triads in D dorian" ─────────────────────────────────────────
banner "3. Triads in D dorian {2, 4, 5, 7, 9, 11, 0}"
echo "  [3/6] Triads in D dorian..."

echo "Query: subchords D E F G A B C --size 3 --name" >> "$REPORT"
echo "" >> "$REPORT"
run subchords D E F G A B C --size 3 --name >> "$REPORT" 2>/dev/null

run pitch-circle D E F G A B C --title D-Dorian --root D \
  -o "$OUT_DIR/contain_d_dorian_circle.svg"

# ── 4. "Comparing chord content: D dorian vs D mixolydian" ──────────
banner "4. Chord content comparison: D dorian vs D mixolydian"
echo "  [4/6] D dorian vs D mixolydian..."

echo "--- D dorian triads ---" >> "$REPORT"
run subchords D E F G A B C --size 3 --name >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "--- D mixolydian triads {2, 4, 6, 7, 9, 11, 0} ---" >> "$REPORT"
run subchords D E Gb G A B C --size 3 --name >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "--- Common tones between D dorian and D mixolydian ---" >> "$REPORT"
run common-tones D,E,F,G,A,B,C D,E,Gb,G,A,B,C >> "$REPORT" 2>/dev/null

run pitch-circle D E Gb G A B C --title D-Mixolydian --root D \
  -o "$OUT_DIR/contain_d_mixolydian_circle.svg"

# ── 5. "What scales contain a diminished 7th?" ──────────────────────
banner "5. Scales containing diminished 7th {0, 3, 6, 9}"
echo "  [5/6] Scales containing dim7..."

echo "Query: contains C Eb Gb A --in scales" >> "$REPORT"
echo "" >> "$REPORT"
run contains C Eb Gb A --in scales >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Symmetry context:" >> "$REPORT"
run prime-form 0 3 6 9 -v >> "$REPORT" 2>/dev/null

run pitch-circle 0 3 6 9 --title Dim7 --show-intervals \
  -o "$OUT_DIR/contain_dim7_circle.svg"

# ── 6. "Superchords of a minor triad — what 4-note chords extend Am?" ─
banner "6. 4-note superchords of A minor {9, 0, 4}"
echo "  [6/6] Superchords of Am..."

echo "Query: superchords A C E --max-size 4" >> "$REPORT"
echo "" >> "$REPORT"
run superchords A C E --max-size 4 >> "$REPORT" 2>/dev/null

run pitch-circle A C E --title Am-Triad --root A \
  -o "$OUT_DIR/contain_am_triad_circle.svg"

# ── Summary ──────────────────────────────────────────────────────────
banner "Summary"
{
  echo "This report demonstrates containment queries:"
  echo "  1. contains --in scales: find scales for a chord (Cmaj7)"
  echo "  2. contains --in chords: find chords in a scale (C major)"
  echo "  3. subchords --name: enumerate triads in a mode (D dorian)"
  echo "  4. Comparative analysis: D dorian vs D mixolydian"
  echo "  5. Symmetric sets: scales containing dim7"
  echo "  6. superchords: extending A minor to 4-note chords"
} >> "$REPORT"

LINES=$(wc -l < "$REPORT")
SVGS=$(ls "$OUT_DIR"/contain_*.svg 2>/dev/null | wc -l)
echo ""
echo "Done! $LINES-line report + $SVGS SVGs in $OUT_DIR"
