#!/usr/bin/env bash
#
# Superchord Lattice: Triad Expansion Explorer
#
# Starting from a C major triad {C,E,G}, systematically explores all known
# superchords at each cardinality from 4 to 7 notes, then examines the
# reverse direction — subchords of larger collections back down to triads.
# Produces a text report showing the lattice of containment relationships
# and SVG pitch-circle diagrams for selected superchords at each level.
#
# Musical use case: understanding how a simple triad relates to larger
# harmonic structures (7th chords, 9th chords, scales/modes), and how
# those larger structures decompose back into familiar triads.
#
# Output:
#   - Text report:  examples/output/superchord_lattice_report.txt
#   - SVG diagrams: examples/output/lattice_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/superchord_lattice.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="cargo run -p slonimsky --"
fi

REPORT="$OUT_DIR/superchord_lattice_report.txt"
: > "$REPORT"

header() {
  echo "" >> "$REPORT"
  echo "================================================================" >> "$REPORT"
  echo "$1" >> "$REPORT"
  echo "================================================================" >> "$REPORT"
}

# --- Root triad ---
header "ROOT: C Major Triad {C, E, G}"

echo "Spelling:" >> "$REPORT"
$S spell Cmaj --format all >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Interval vector:" >> "$REPORT"
$S interval-vector C E G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Prime form:" >> "$REPORT"
$S prime-form C E G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Forte class:" >> "$REPORT"
$S forte C E G >> "$REPORT" 2>/dev/null

# SVG of root triad
$S pitch-circle C E G --title "C Major Triad" \
  -o "$OUT_DIR/lattice_root_cmaj.svg" 2>/dev/null

# --- Layer 1: 4-note superchords ---
header "LAYER 1: 4-note superchords of {C,E,G}"

$S superchords C E G --min-size 4 --max-size 4 >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Selected 4-note superchords — interval vectors:" >> "$REPORT"

for SET_LABEL in "C,E,G,B:Cmaj7" "C,E,G,Bb:C7" "C,E,G,A:Am7_inv"; do
  NOTES="${SET_LABEL%%:*}"
  LABEL="${SET_LABEL##*:}"
  IFS=',' read -ra N <<< "$NOTES"
  echo "" >> "$REPORT"
  echo "  $LABEL {$NOTES}:" >> "$REPORT"
  echo -n "    IV: " >> "$REPORT"
  $S interval-vector "${N[@]}" >> "$REPORT" 2>/dev/null
  echo -n "    Forte: " >> "$REPORT"
  $S forte "${N[@]}" >> "$REPORT" 2>/dev/null
done

# SVGs for key 4-note superchords
$S pitch-circle C E G B --title "Cmaj7" \
  -o "$OUT_DIR/lattice_l1_cmaj7.svg" 2>/dev/null
$S pitch-circle C E G Bb --title "C7" \
  -o "$OUT_DIR/lattice_l1_c7.svg" 2>/dev/null

# --- Layer 2: 5-note superchords ---
header "LAYER 2: 5-note superchords of {C,E,G}"

$S superchords C E G --min-size 5 --max-size 5 >> "$REPORT" 2>/dev/null

# --- Layer 3: 6-note superchords ---
header "LAYER 3: 6-note superchords of {C,E,G}"

$S superchords C E G --min-size 6 --max-size 6 >> "$REPORT" 2>/dev/null

# --- Layer 4: 7-note superchords (scales/modes) ---
header "LAYER 4: 7-note superchords (scales/modes) of {C,E,G}"

$S superchords C E G --min-size 7 --max-size 7 >> "$REPORT" 2>/dev/null

# SVG for a 7-note scale containing C major
$S pitch-circle C D E F G A B --title "C Ionian (contains Cmaj)" \
  -o "$OUT_DIR/lattice_l4_c_ionian.svg" 2>/dev/null

# --- Reverse: subchords of C Ionian ---
header "REVERSE: 3-note subchords (triads) of C Ionian {C,D,E,F,G,A,B}"

echo "All triadic subsets of the C major scale:" >> "$REPORT"
$S subchords C D E F G A B --size 3 --name >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "4-note subsets (7th chords) of C Ionian:" >> "$REPORT"
$S subchords C D E F G A B --size 4 --name >> "$REPORT" 2>/dev/null

# --- Cross-reference: common tones between layer-1 superchords ---
header "CROSS-REFERENCE: Common tones between 4-note superchords"

echo "Cmaj7 vs C7 (major 7th vs dominant 7th):" >> "$REPORT"
$S common-tones C,E,G,B C,E,G,Bb >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Cmaj7 vs Am7 (relative minor relationship):" >> "$REPORT"
$S common-tones C,E,G,B A,C,E,G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "C7 vs Am7:" >> "$REPORT"
$S common-tones C,E,G,Bb A,C,E,G >> "$REPORT" 2>/dev/null

# --- Summary ---
header "SUMMARY"

echo "Superchord lattice for C major triad {C, E, G}:" >> "$REPORT"
echo "" >> "$REPORT"
echo "  Layer 0 (root):   C major triad — 3 notes" >> "$REPORT"

# Count superchords at each layer
for SIZE in 4 5 6 7; do
  COUNT=$($S superchords C E G --min-size "$SIZE" --max-size "$SIZE" 2>/dev/null \
    | grep "^Total:" | sed 's/[^0-9]//g')
  echo "  Layer $((SIZE-3)) (${SIZE}-note): $COUNT superchords" >> "$REPORT"
done

echo "" >> "$REPORT"
echo "The lattice shows how a single triad fans out into increasingly" >> "$REPORT"
echo "complex harmonic structures, each layer adding one pitch class." >> "$REPORT"
echo "Reversing the lattice (subchords) reveals triadic content of scales." >> "$REPORT"
echo "" >> "$REPORT"

echo "=== superchord_lattice.sh completed ==="
