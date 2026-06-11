#!/usr/bin/env bash
#
# Jazz Progression Voice-Leading: Common Patterns
#
# Demonstrates the `progression` subcommand through four canonical jazz
# chord sequences, showing how the greedy voice-leading optimizer
# distributes motion across voices. Each progression is run in both
# unconstrained and no-crossings modes, with verbose output showing
# per-voice semitone paths.
#
# Progressions covered:
#   1. ii-V-I in C major (Dm7 → G7 → Cmaj7) — the most common jazz cadence
#   2. I-vi-ii-V turnaround (Cmaj7 → Am7 → Dm7 → G7) — rhythm changes bridge
#   3. iii-vi-ii-V (Em7 → Am7 → Dm7 → G7) — full cycle of fifths descent
#   4. Tritone substitution ii-V (Dm7 → Db7 → Cmaj7) — chromatic approach
#
# For each progression, pitch-circle diagrams of the constituent chords
# are also generated, giving a visual complement to the text analysis.
#
# Output:
#   - Text report: examples/output/progression_jazz_report.txt
#   - SVG diagrams: examples/output/prog_*.svg
#   - MIDI files: examples/output/prog_*.mid (requires --features midi)
#
# Usage:
#   bash slonimsky/examples/scripts/progression_jazz.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/progression_jazz.sh
#
# MIDI support:
#   Set SLONIMSKY_MIDI=1 to build with midi feature and emit MIDI files.
#   Without it, only text + SVG output is produced.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
elif [ "${SLONIMSKY_MIDI:-}" = "1" ]; then
  S="cargo run -p slonimsky --features midi --"
else
  S="cargo run -p slonimsky --"
fi

# MIDI-enabled runner: uses midi feature for -o .mid output
if [ -n "${SLONIMSKY:-}" ]; then
  SM="$SLONIMSKY"
elif [ "${SLONIMSKY_MIDI:-}" = "1" ]; then
  SM="cargo run -p slonimsky --features midi --"
else
  SM=""
fi

REPORT="$OUT_DIR/progression_jazz_report.txt"

{
echo "============================================"
echo "  Jazz Progression Voice-Leading Analysis"
echo "============================================"
echo ""

# --- 1. ii-V-I in C major ---
echo "============================================"
echo "  1. ii-V-I in C major"
echo "     Dm7 → G7 → Cmaj7"
echo "============================================"
echo ""
echo "--- Unconstrained (3 voices: triad reduction) ---"
$S progression D,F,A G,B,D C,E,G
echo ""
echo "--- 4 voices, no crossings, verbose ---"
$S progression D,F,A,C G,B,D,F C,E,G,B --no-crossings -v
echo ""
echo "--- Common tones across the chain ---"
echo "  Dm7 ∩ G7:"
$S common-tones D,F,A,C G,B,D,F
echo "  G7 ∩ Cmaj7:"
$S common-tones G,B,D,F C,E,G,B
echo "  All three:"
$S common-tones D,F,A,C G,B,D,F C,E,G,B
echo ""

# --- 2. I-vi-ii-V turnaround ---
echo "============================================"
echo "  2. I-vi-ii-V Turnaround"
echo "     Cmaj7 → Am7 → Dm7 → G7"
echo "============================================"
echo ""
echo "--- 4 voices, no crossings, verbose ---"
$S progression C,E,G,B A,C,E,G D,F,A,C G,B,D,F --no-crossings -v
echo ""
echo "--- Unconstrained ---"
$S progression C,E,G,B A,C,E,G D,F,A,C G,B,D,F
echo ""

# --- 3. iii-vi-ii-V cycle of fifths ---
echo "============================================"
echo "  3. iii-vi-ii-V (Cycle of Fifths)"
echo "     Em7 → Am7 → Dm7 → G7"
echo "============================================"
echo ""
echo "--- 4 voices, no crossings, verbose ---"
$S progression E,G,B,D A,C,E,G D,F,A,C G,B,D,F --no-crossings -v
echo ""
echo "--- Set-class analysis of the chain ---"
echo "  Em7:"
$S prime-form E G B D
echo "  Am7:"
$S prime-form A C E G
echo "  Dm7:"
$S prime-form D F A C
echo "  G7:"
$S prime-form G B D F
echo ""
echo "(All diatonic 7th chords are either 4-20 [maj7], 4-26 [min7], or 4-27 [dom7].)"
echo ""

# --- 4. Tritone substitution ---
echo "============================================"
echo "  4. Tritone Substitution ii-V"
echo "     Dm7 → Db7 → Cmaj7"
echo "============================================"
echo ""
echo "--- 4 voices, no crossings, verbose ---"
$S progression D,F,A,C Db,F,Ab,B C,E,G,B --no-crossings -v
echo ""
echo "--- Unconstrained ---"
$S progression D,F,A,C Db,F,Ab,B C,E,G,B
echo ""
echo "--- Common tones: Db7 ∩ G7 (tritone pair shares the tritone F-B) ---"
$S common-tones Db,F,Ab,B G,B,D,F
echo ""

echo "============================================"
echo "  Summary"
echo "============================================"
echo ""
echo "Progression                   | Voices | Cost (L1) | Avg/step"
echo "------------------------------|--------|-----------|----------"
echo "(See individual sections above for exact costs.)"
echo ""
echo "Key observations:"
echo "  - ii-V-I is remarkably smooth: each step can move by only 2-3 semitones total"
echo "  - The turnaround (I-vi-ii-V) achieves similar smoothness via shared tones"
echo "  - Cycle-of-fifths (iii-vi-ii-V) has identical set-class content to the turnaround"
echo "  - Tritone substitution (Db7 for G7) preserves the guide tones F and B(Cb)"
echo ""
echo "============================================"
echo "  END OF REPORT"
echo "============================================"
} > "$REPORT" 2>&1

echo "Report written to: $REPORT"

# --- SVG diagrams: pitch circles for the constituent chords ---
$S pitch-circle D F A C --title "Dm7 (ii)" --theme print -o "$OUT_DIR/prog_dm7_circle.svg"
echo "SVG: prog_dm7_circle.svg"

$S pitch-circle G B D F --title "G7 (V)" --theme dark -o "$OUT_DIR/prog_g7_circle.svg"
echo "SVG: prog_g7_circle.svg"

$S pitch-circle C E G B --title "Cmaj7 (I)" -o "$OUT_DIR/prog_cmaj7_circle.svg"
echo "SVG: prog_cmaj7_circle.svg"

$S pitch-circle A C E G --title "Am7 (vi)" --theme print -o "$OUT_DIR/prog_am7_circle.svg"
echo "SVG: prog_am7_circle.svg"

$S pitch-circle E G B D --title "Em7 (iii)" --theme colorful -o "$OUT_DIR/prog_em7_circle.svg"
echo "SVG: prog_em7_circle.svg"

$S pitch-circle Db F Ab B --title "Db7 (subV)" --theme dark -o "$OUT_DIR/prog_db7_circle.svg"
echo "SVG: prog_db7_circle.svg"

# --- MIDI files: one per progression (requires midi feature) ---
MIDI_COUNT=0
if [ -n "$SM" ]; then
  echo ""
  echo "Generating MIDI files..."

  $SM progression D,F,A,C G,B,D,F C,E,G,B --no-crossings -o "$OUT_DIR/prog_ii_V_I.mid"
  echo "MIDI: prog_ii_V_I.mid (ii-V-I)"
  MIDI_COUNT=$((MIDI_COUNT + 1))

  $SM progression C,E,G,B A,C,E,G D,F,A,C G,B,D,F --no-crossings -o "$OUT_DIR/prog_turnaround.mid"
  echo "MIDI: prog_turnaround.mid (I-vi-ii-V)"
  MIDI_COUNT=$((MIDI_COUNT + 1))

  $SM progression E,G,B,D A,C,E,G D,F,A,C G,B,D,F --no-crossings -o "$OUT_DIR/prog_cycle_of_fifths.mid"
  echo "MIDI: prog_cycle_of_fifths.mid (iii-vi-ii-V)"
  MIDI_COUNT=$((MIDI_COUNT + 1))

  $SM progression D,F,A,C Db,F,Ab,B C,E,G,B --no-crossings -o "$OUT_DIR/prog_tritone_sub.mid"
  echo "MIDI: prog_tritone_sub.mid (tritone sub ii-V)"
  MIDI_COUNT=$((MIDI_COUNT + 1))

  echo ""
  echo "Done. 6 SVGs + $MIDI_COUNT MIDIs + 1 text report in $OUT_DIR"
else
  echo ""
  echo "Done. 6 SVGs + 1 text report in $OUT_DIR"
  echo "(Set SLONIMSKY_MIDI=1 to also generate MIDI files.)"
fi
