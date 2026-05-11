#!/usr/bin/env bash
#
# Arpeggio Dictionary Demo
#
# Generates fretboard arpeggio/chord shape sheets for common chord types
# across multiple keys and tunings. Demonstrates:
#   - Major triad shapes across circle-of-fifths keys
#   - Minor 7th shapes in selected keys with dark theme
#   - Dominant 7th shapes for jazz ii-V-I keys
#   - Drop-D tuning shapes for power chords
#   - 7-string extended range for major 7th arpeggios
#
# Output: text reference sheets + SVG fretboard grids in examples/output/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

# Use SLONIMSKY env var if set; otherwise find pre-built binary or fall back to cargo run
if [ -n "${SLONIMSKY:-}" ]; then
  : # already set
elif [ -x "$SCRIPT_DIR/../../../target/debug/slonimsky" ]; then
  SLONIMSKY="$SCRIPT_DIR/../../../target/debug/slonimsky"
elif [ -x "$SCRIPT_DIR/../../../target/release/slonimsky" ]; then
  SLONIMSKY="$SCRIPT_DIR/../../../target/release/slonimsky"
else
  SLONIMSKY="cargo run -p slonimsky --"
fi

echo "=== Arpeggio Dictionary Demo ==="
echo ""

# 1. Major triad — all 12 keys, text reference
echo "1. Major triad (C E G) — all 12 keys, text output..."
$SLONIMSKY arpeggio-dictionary C E G --positions 4 \
  > "$OUT_DIR/arpeggio_dict_major_triad_all_keys.txt"
echo "   → $(wc -l < "$OUT_DIR/arpeggio_dict_major_triad_all_keys.txt") lines"

# 2. Major triad — circle-of-fifths keys (C, G, D, A, E, B), SVG grid
echo "2. Major triad — circle-of-fifths keys, SVG fretboard grid..."
$SLONIMSKY arpeggio-dictionary C E G --keys C,G,D,A,E,B --positions 3 \
  -o "$OUT_DIR/arpeggio_dict_major_triad_cof.svg"
echo "   → $(wc -c < "$OUT_DIR/arpeggio_dict_major_triad_cof.svg") bytes"

# 3. Minor 7th (C Eb G Bb) — selected keys, dark theme SVG
echo "3. Minor 7th (C Eb G Bb) — keys A, D, E, dark theme SVG..."
$SLONIMSKY arpeggio-dictionary C Eb G Bb --keys A,D,E --positions 4 \
  --theme dark -o "$OUT_DIR/arpeggio_dict_minor7_dark.svg"
echo "   → $(wc -c < "$OUT_DIR/arpeggio_dict_minor7_dark.svg") bytes"

# 4. Dominant 7th (C E G Bb) — jazz ii-V-I keys, text reference
echo "4. Dominant 7th (C E G Bb) — keys C, F, Bb, Eb, text..."
$SLONIMSKY arpeggio-dictionary C E G Bb --keys C,F,Bb,Eb --positions 3 \
  > "$OUT_DIR/arpeggio_dict_dom7_jazz_keys.txt"
echo "   → $(wc -l < "$OUT_DIR/arpeggio_dict_dom7_jazz_keys.txt") lines"

# 5. Power chord (C G) — drop-D tuning, all keys, SVG
echo "5. Power chord (C G) — drop-D tuning, keys D, A, E, SVG..."
$SLONIMSKY arpeggio-dictionary C G --tuning drop-d --keys D,A,E --positions 4 \
  -o "$OUT_DIR/arpeggio_dict_power_chord_dropd.svg"
echo "   → $(wc -c < "$OUT_DIR/arpeggio_dict_power_chord_dropd.svg") bytes"

# 6. Major 7th (C E G B) — 7-string tuning, keys C, G, text
echo "6. Major 7th (C E G B) — 7-string tuning, keys C, G..."
$SLONIMSKY arpeggio-dictionary C E G B --tuning 7-string --keys C,G --positions 3 \
  > "$OUT_DIR/arpeggio_dict_maj7_7string.txt"
echo "   → $(wc -l < "$OUT_DIR/arpeggio_dict_maj7_7string.txt") lines"

# 7. Diminished triad (C Eb Gb) — print theme SVG for practice sheets
echo "7. Diminished triad (C Eb Gb) — keys C, F#, print theme SVG..."
$SLONIMSKY arpeggio-dictionary C Eb Gb --keys C,Gb --positions 4 \
  --theme print -o "$OUT_DIR/arpeggio_dict_dim_print.svg"
echo "   → $(wc -c < "$OUT_DIR/arpeggio_dict_dim_print.svg") bytes"

echo ""
echo "=== Summary ==="
echo "Generated 7 output files in $OUT_DIR:"
ls -la "$OUT_DIR"/arpeggio_dict_* 2>/dev/null | awk '{print "  " $NF " (" $5 " bytes)"}'
echo ""
echo "Done."
