#!/usr/bin/env bash
# annotate_demo.sh — Demonstrate the `annotate` subcommand with voiced
# chord progressions, producing text and JSON voice-leading analysis.
#
# Use case: voice-leading quality assessment — per-step L1/L∞ costs,
# motion labels (common tone / step / skip / leap), crossing detection,
# and smoothness ratings for concrete voicings with octave placement.
#
# Invocation:
#   cargo build -p slonimsky && ./slonimsky/examples/scripts/annotate_demo.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/annotate_demo.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$SCRIPT_DIR/../output"
mkdir -p "$OUT"

if [ -n "${SLONIMSKY:-}" ]; then
  BIN="$SLONIMSKY"
else
  BIN="$SCRIPT_DIR/../../../target/debug/slonimsky"
  if [ ! -x "$BIN" ]; then
    BIN="$SCRIPT_DIR/../../../target/release/slonimsky"
  fi
  if [ ! -x "$BIN" ]; then
    echo "Binary not found. Build first: cargo build -p slonimsky" >&2
    echo "Or set SLONIMSKY=/path/to/binary" >&2
    exit 1
  fi
fi

echo "=== Annotate Demo ==="

# 1. Smooth I-vi-ii-V in close position (text)
echo ""
echo "--- 1. I-vi-ii-V close position (text) ---"
"$BIN" annotate C4,E4,G4 C4,E4,A4 D4,F4,A4 D4,F4,B4 \
  | tee "$OUT/annotate_close_position.txt"

# 2. Same progression with --key for context (text)
echo ""
echo "--- 2. I-vi-ii-V with key context (text) ---"
"$BIN" annotate C4,E4,G4 C4,E4,A4 D4,F4,A4 D4,F4,B4 --key C \
  | tee "$OUT/annotate_with_key.txt"

# 3. Wide-leap progression showing poor smoothness (text)
echo ""
echo "--- 3. Wide leaps — poor smoothness (text) ---"
"$BIN" annotate C3,E3,G3 A4,C5,E5 D3,F3,A3 G4,B4,D5 \
  | tee "$OUT/annotate_wide_leaps.txt"

# 4. Voice crossing example with --no-crossings warning (text)
echo ""
echo "--- 4. Voice crossings detected (text) ---"
"$BIN" annotate C4,E4,G4 E4,C4,G4 --no-crossings \
  | tee "$OUT/annotate_crossings.txt"

# 5. Four-voice chorale-style progression (JSON)
echo ""
echo "--- 5. Four-voice chorale (JSON) ---"
"$BIN" annotate C3,G3,E4,C5 C3,A3,F4,C5 D3,A3,F4,D5 G2,G3,D4,B4 \
  --format json \
  | tee "$OUT/annotate_chorale.json"

# 6. Jazz ii-V-I in Bb with close voicings (JSON file output)
echo ""
echo "--- 6. Jazz ii-V-I in Bb (JSON file) ---"
"$BIN" annotate C4,Eb4,F4,A4 Bb3,D4,F4,Ab4 Bb3,D4,F4,A4 \
  --key Bb --format json \
  -o "$OUT/annotate_jazz_iiVI_Bb.json"
echo "  Written to $OUT/annotate_jazz_iiVI_Bb.json"

# 7. Verbose mode on a two-chord step (text file output)
echo ""
echo "--- 7. Verbose two-chord step ---"
"$BIN" annotate E3,G3,B3,D4 F3,A3,C4,E4 -v \
  > "$OUT/annotate_verbose.txt" 2>&1
echo "  Written to $OUT/annotate_verbose.txt"

# 8. Text file output for archiving
echo ""
echo "--- 8. Descending bass line (text file) ---"
"$BIN" annotate C4,E4,G4 B3,D4,G4 Bb3,D4,F4 A3,C4,F4 \
  -o "$OUT/annotate_descending_bass.txt"
echo "  Written to $OUT/annotate_descending_bass.txt"

echo ""
echo "=== All annotate demo outputs written to $OUT/annotate_*.{txt,json} ==="
ls -la "$OUT"/annotate_*
echo "=== Done ==="
