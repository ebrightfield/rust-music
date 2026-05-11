#!/usr/bin/env bash
#
# Chord Exploration Workflow
#
# Demonstrates a guitarist-focused chord exploration workflow: given a
# starting chord, find its fretboard shapes, discover related chords by
# distance, check which scales contain it, and examine common tones with
# nearby chords. This is the kind of workflow a jazz guitarist uses when
# exploring harmonic substitutions and reharmonization options.
#
# Musical scenarios covered:
#   1. Chord dictionary: all playable shapes for Cmaj7 on standard guitar
#   2. Closest relatives: chords nearest to Cmaj7 by symmetric difference
#   3. Scale containment: which 7-note scales contain Cmaj7?
#   4. Common tones: shared notes between Cmaj7 and its closest relatives
#   5. Substitution chains: Am7 → Cmaj7 → Em7 (relative minor/iii relationships)
#   6. Drop-D exploration: chord shapes for D7 in drop-D tuning
#
# Output:
#   - Text report: examples/output/chord_exploration_report.txt
#   - SVG diagrams: examples/output/chordex_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/chord_exploration.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/chord_exploration.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="cargo run -p slonimsky --"
fi

REPORT="$OUT_DIR/chord_exploration_report.txt"
: > "$REPORT"

header() {
  echo "" >> "$REPORT"
  echo "========================================" >> "$REPORT"
  echo "$1" >> "$REPORT"
  echo "========================================" >> "$REPORT"
}

# --- 1. Chord dictionary: Cmaj7 shapes on standard guitar ---
header "1. CHORD DICTIONARY: Cmaj7 on Standard Guitar"

echo "All playable Cmaj7 shapes (max span 4 frets, standard tuning):" >> "$REPORT"
$S chord-dictionary C E G B --max-span 4 --max-results 15 >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Compact shapes only (max span 2 frets):" >> "$REPORT"
$S chord-dictionary C E G B --max-span 2 --max-results 10 >> "$REPORT" 2>/dev/null

# Pitch circle for Cmaj7
$S pitch-circle C E G B --title "Cmaj7" -o "$OUT_DIR/chordex_cmaj7_circle.svg" 2>/dev/null

# --- 2. Closest relatives: what's near Cmaj7? ---
header "2. CLOSEST RELATIVES: Chords near Cmaj7"

echo "Closest chords to Cmaj7 {0,4,7,11} by symmetric difference:" >> "$REPORT"
$S closest C E G B --pool chords --limit 15 >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Closest scales to Cmaj7:" >> "$REPORT"
$S closest C E G B --pool scales --limit 10 >> "$REPORT" 2>/dev/null

# --- 3. Scale containment: which scales hold Cmaj7? ---
header "3. SCALE CONTAINMENT: Scales containing Cmaj7"

echo "7-note scales containing all of {C, E, G, B}:" >> "$REPORT"
$S contains C E G B --in scales >> "$REPORT" 2>/dev/null

# --- 4. Common tones: Cmaj7 vs relatives ---
header "4. COMMON TONES: Cmaj7 vs Nearby Chords"

echo "Cmaj7 {C,E,G,B} vs Am7 {A,C,E,G}:" >> "$REPORT"
$S common-tones C,E,G,B A,C,E,G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Cmaj7 {C,E,G,B} vs Em7 {E,G,B,D}:" >> "$REPORT"
$S common-tones C,E,G,B E,G,B,D >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Cmaj7 {C,E,G,B} vs Dm7 {D,F,A,C}:" >> "$REPORT"
$S common-tones C,E,G,B D,F,A,C >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Cmaj7 {C,E,G,B} vs Fmaj7 {F,A,C,E}:" >> "$REPORT"
$S common-tones C,E,G,B F,A,C,E >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "All four diatonic 7th chords together:" >> "$REPORT"
$S common-tones C,E,G,B A,C,E,G E,G,B,D D,F,A,C -v >> "$REPORT" 2>&1

# Pitch circles for the comparison chords
$S pitch-circle A C E G --title "Am7" -o "$OUT_DIR/chordex_am7_circle.svg" 2>/dev/null
$S pitch-circle E G B D --title "Em7" -o "$OUT_DIR/chordex_em7_circle.svg" 2>/dev/null
$S pitch-circle D F A C --title "Dm7" -o "$OUT_DIR/chordex_dm7_circle.svg" 2>/dev/null

# --- 5. Substitution chain: Am7 → Cmaj7 → Em7 ---
header "5. SUBSTITUTION CHAIN: Am7 → Cmaj7 → Em7"

echo "These three chords share the relative minor / mediant relationship." >> "$REPORT"
echo "Am7 and Cmaj7 share 3 of 4 tones; Cmaj7 and Em7 share 3 of 4 tones." >> "$REPORT"
echo "" >> "$REPORT"

echo "Prime form of Am7 {A,C,E,G}:" >> "$REPORT"
$S prime-form A C E G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Prime form of Cmaj7 {C,E,G,B}:" >> "$REPORT"
$S prime-form C E G B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Prime form of Em7 {E,G,B,D}:" >> "$REPORT"
$S prime-form E G B D >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Forte classification of Am7:" >> "$REPORT"
$S forte A C E G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Forte classification of Cmaj7:" >> "$REPORT"
$S forte C E G B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Interval vectors compared:" >> "$REPORT"
echo "  Am7:  $($S interval-vector A C E G 2>/dev/null | grep '<')" >> "$REPORT"
echo "  Cmaj7: $($S interval-vector C E G B 2>/dev/null | grep '<')" >> "$REPORT"
echo "  Em7:  $($S interval-vector E G B D 2>/dev/null | grep '<')" >> "$REPORT"

# --- 6. Drop-D exploration: D7 shapes ---
header "6. DROP-D EXPLORATION: D7 Shapes"

echo "D7 chord shapes in drop-D tuning (max span 4):" >> "$REPORT"
$S chord-dictionary D Gb A C --tuning drop-d --max-span 4 --max-results 10 >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "D7 chord shapes in standard tuning for comparison:" >> "$REPORT"
$S chord-dictionary D Gb A C --tuning standard --max-span 4 --max-results 10 >> "$REPORT" 2>/dev/null

# Fretboard shapes for visual reference
$S fretboard x-x-0-2-1-2 --title "D7 open (std)" -o "$OUT_DIR/chordex_d7_open.svg" 2>/dev/null
$S fretboard x-x-0-2-1-2 --title "D7 open (std) dark" --theme dark -o "$OUT_DIR/chordex_d7_dark.svg" 2>/dev/null

# Summary
header "SUMMARY"
echo "Cmaj7 exploration complete. Key findings:" >> "$REPORT"
echo "  - Cmaj7 has multiple playable fretboard shapes (see section 1)" >> "$REPORT"
echo "  - Am7, Em7 are the closest 4-note relatives (3 shared tones each)" >> "$REPORT"
echo "  - Cmaj7 appears in many 7-note scales (C Major, G Major, etc.)" >> "$REPORT"
echo "  - Am7, Cmaj7, Em7 form a substitution chain (all Forte class 4-20 or 4-26)" >> "$REPORT"
echo "  - Drop-D tuning opens different voicing possibilities for D7" >> "$REPORT"
echo "" >> "$REPORT"

echo "=== chord_exploration.sh completed ==="
