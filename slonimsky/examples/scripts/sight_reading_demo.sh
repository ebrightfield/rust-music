#!/usr/bin/env bash
# Sight-Reading Exercise Demo
#
# Generates sight-reading exercises across different keys, scales,
# and difficulty levels. Demonstrates the `sight-reading` subcommand's
# ability to produce deterministic, varied melodic exercises for
# practice purposes.
#
# Exercises:
#   1. C Major, difficulty 1 (beginner — stepwise quarter notes)
#   2. G Major, difficulty 3 (intermediate — steps, skips, mixed rhythms)
#   3. A Melodic Minor, difficulty 5 (advanced — wide leaps, complex rhythms)
#   4. Bb Harmonic Minor, difficulty 2 (flat-key warm-up)
#   5. D Harmonic Major, difficulty 4 (syncopated, wider intervals)
#   6. Seed comparison: same key/scale/difficulty, two different seeds
#   7. Extended exercise: 8 measures at difficulty 3 in E major
#
# Usage:
#   SLONIMSKY=target/debug/slonimsky bash slonimsky/examples/scripts/sight_reading_demo.sh
#
# Output: 8 text files in slonimsky/examples/output/

set -euo pipefail

SLONIMSKY="${SLONIMSKY:-cargo run -p slonimsky --}"
OUT="slonimsky/examples/output"
mkdir -p "$OUT"

echo "=== Sight-Reading Exercise Demo ==="
echo ""

# --- 1. C Major beginner ---
echo "1. C Major, difficulty 1 (beginner — stepwise motion)..."
$SLONIMSKY sight-reading --key C --scale major --difficulty 1 --seed 100 \
  > "$OUT/sight_reading_c_major_d1.txt"
echo "   → $(wc -l < "$OUT/sight_reading_c_major_d1.txt") lines"

# --- 2. G Major intermediate ---
echo "2. G Major, difficulty 3 (intermediate — steps + skips)..."
$SLONIMSKY sight-reading --key G --scale major --difficulty 3 --seed 200 \
  > "$OUT/sight_reading_g_major_d3.txt"
echo "   → $(wc -l < "$OUT/sight_reading_g_major_d3.txt") lines"

# --- 3. A Melodic Minor advanced ---
echo "3. A Melodic Minor, difficulty 5 (advanced — wide leaps)..."
$SLONIMSKY sight-reading --key A --scale melodic-minor --difficulty 5 --seed 300 \
  > "$OUT/sight_reading_a_melminor_d5.txt"
echo "   → $(wc -l < "$OUT/sight_reading_a_melminor_d5.txt") lines"

# --- 4. Bb Harmonic Minor warm-up ---
echo "4. Bb Harmonic Minor, difficulty 2 (flat-key warm-up)..."
$SLONIMSKY sight-reading --key Bb --scale harmonic-minor --difficulty 2 --seed 400 \
  > "$OUT/sight_reading_bb_harmminor_d2.txt"
echo "   → $(wc -l < "$OUT/sight_reading_bb_harmminor_d2.txt") lines"

# --- 5. D Harmonic Major syncopated ---
echo "5. D Harmonic Major, difficulty 4 (syncopated)..."
$SLONIMSKY sight-reading --key D --scale harmonic-major --difficulty 4 --seed 500 \
  > "$OUT/sight_reading_d_harmmajor_d4.txt"
echo "   → $(wc -l < "$OUT/sight_reading_d_harmmajor_d4.txt") lines"

# --- 6. Seed comparison ---
echo "6. Seed comparison: F Major difficulty 3, seed 42 vs seed 99..."
$SLONIMSKY sight-reading --key F --scale major --difficulty 3 --seed 42 \
  > "$OUT/sight_reading_f_major_seed42.txt"
$SLONIMSKY sight-reading --key F --scale major --difficulty 3 --seed 99 \
  > "$OUT/sight_reading_f_major_seed99.txt"
if diff -q "$OUT/sight_reading_f_major_seed42.txt" "$OUT/sight_reading_f_major_seed99.txt" > /dev/null 2>&1; then
  echo "   ⚠ Same output (seeds should differ!)"
else
  echo "   ✓ Different seeds produce different exercises"
fi

# --- 7. Extended 8-measure exercise ---
echo "7. E Major, difficulty 3, 8 measures (extended exercise)..."
$SLONIMSKY sight-reading --key E --scale major --difficulty 3 --measures 8 --seed 700 \
  > "$OUT/sight_reading_e_major_8m.txt"
echo "   → $(wc -l < "$OUT/sight_reading_e_major_8m.txt") lines"

echo ""
echo "=== Summary ==="
echo "Generated 8 sight-reading exercise files in $OUT/"
ls -la "$OUT"/sight_reading_*.txt
echo ""
echo "Done."
