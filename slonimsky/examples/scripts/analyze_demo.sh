#!/usr/bin/env bash
# analyze_demo.sh — Demonstrate the `analyze` subcommand with common
# chord progressions, producing both text and JSON output.
#
# Use case: harmonic analysis of standard progressions — key estimation,
# Roman numeral assignment, common-tone and voice-leading cost summaries.
#
# Invocation:
#   cargo build -p slonimsky && ./slonimsky/examples/scripts/analyze_demo.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/analyze_demo.sh

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

echo "=== Analyze Demo ==="

# 1. Classic I-vi-ii-V in C major (text)
echo ""
echo "--- 1. I-vi-ii-V in C major (text) ---"
"$BIN" analyze C,E,G A,C,E D,F,A G,B,D \
  | tee "$OUT/analyze_I_vi_ii_V.txt"

# 2. I-IV-V-I in G major with explicit key (text)
echo ""
echo "--- 2. I-IV-V-I in G major (explicit key, text) ---"
"$BIN" analyze G,B,D C,E,G D,A,B G,B,D --key G --scale major \
  | tee "$OUT/analyze_I_IV_V_I_G.txt"

# 3. ii-V-I in C major — jazz cadence (JSON)
echo ""
echo "--- 3. ii-V-I in C major (JSON) ---"
"$BIN" analyze D,F,A,C G,B,D,F C,E,G,B --format json \
  | tee "$OUT/analyze_iiVI_jazz.json"

# 4. I-bVII-IV-I — mixolydian vamp (text)
echo ""
echo "--- 4. I-bVII-IV-I rock vamp (text) ---"
"$BIN" analyze C,E,G Bb,D,F F,A,C C,E,G \
  | tee "$OUT/analyze_rock_vamp.txt"

# 5. vi-IV-I-V pop progression in C major (JSON)
echo ""
echo "--- 5. vi-IV-I-V pop progression (JSON) ---"
"$BIN" analyze A,C,E F,A,C C,E,G G,B,D --format json \
  | tee "$OUT/analyze_pop_viIVI_V.json"

# 6. Blues I-IV-I-V-IV-I in A (text, explicit key)
echo ""
echo "--- 6. Blues I-IV-I-V-IV-I in A (text) ---"
"$BIN" analyze A,Db,E D,Gb,A A,Db,E E,Ab,B D,Gb,A A,Db,E \
  --key A \
  | tee "$OUT/analyze_blues_A.txt"

# 7. Minor i-iv-V in A natural minor (text)
echo ""
echo "--- 7. i-iv-V in A natural minor (text) ---"
"$BIN" analyze A,C,E D,F,A E,Ab,B \
  --key A --scale natural-minor \
  | tee "$OUT/analyze_minor_A.txt"

# 8. Verbose mode on ii-V-I to show extra diagnostics
echo ""
echo "--- 8. ii-V-I verbose mode ---"
"$BIN" analyze D,F,A,C G,B,D,F C,E,G,B -v \
  > "$OUT/analyze_iiVI_verbose.txt" 2>&1

echo ""
echo "=== All analyze demo outputs written to $OUT/analyze_*.{txt,json} ==="
ls -la "$OUT"/analyze_*
echo "=== Done ==="
