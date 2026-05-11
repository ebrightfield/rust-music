#!/usr/bin/env bash
#
# Coltrane Changes — Giant Steps Harmonic Analysis
#
# Explores the harmonic structure of John Coltrane's "Giant Steps"
# substitution pattern, which divides the octave into three equal parts
# (major thirds) and moves through three tonal centers: C, E, Ab.
#
# This script demonstrates how slonimsky's set-theoretic and
# voice-leading tools illuminate the geometry behind Coltrane's
# innovation:
#
#   1. The augmented triad as the skeleton — {0,4,8} has T4 symmetry
#   2. Major triads built on each node: C, E, Ab
#   3. Common-tone analysis between adjacent key centers
#   4. Voice-leading costs through the cycle
#   5. The dominant approach pattern: each key center is preceded by
#      its V chord (G7→C, B7→E, Eb7→Ab)
#   6. Interval vectors and Forte classification of the combined
#      pitch content
#   7. Progression analysis with Roman numerals
#   8. Pitch-circle visualizations of the three key centers and the
#      combined augmented-triad skeleton
#
# Musical context: "Giant Steps" (1960) uses a ii-V-I pattern that
# cycles through three keys a major third apart, creating a sense of
# constant modulation while maintaining an underlying symmetry that
# the ear can follow.
#
# Usage:
#   cargo build -p slonimsky && bash slonimsky/examples/scripts/coltrane_changes.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/coltrane_changes.sh
#
# Outputs:
#   slonimsky/examples/output/coltrane_report.txt
#   slonimsky/examples/output/coltrane_skeleton_circle.svg
#   slonimsky/examples/output/coltrane_c_circle.svg
#   slonimsky/examples/output/coltrane_e_circle.svg
#   slonimsky/examples/output/coltrane_ab_circle.svg
#   slonimsky/examples/output/coltrane_combined_circle.svg

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

# Ensure writable cargo home for compilation
if [ ! -w "${CARGO_HOME:-/opt/rust/cargo}" ]; then
  export CARGO_HOME=/workspace/.cargo
fi

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="cargo run -p slonimsky --"
fi

# Pre-build so cargo compilation output doesn't leak into reports
cargo build -p slonimsky 2>/dev/null

REPORT="$OUT_DIR/coltrane_report.txt"
: > "$REPORT"

header() {
  echo "" >> "$REPORT"
  echo "========================================" >> "$REPORT"
  echo "$1" >> "$REPORT"
  echo "========================================" >> "$REPORT"
}

# Run a command, merging stderr into stdout but filtering cargo noise
run_verbose() {
  "$@" 2>&1 | grep -v '^\s*\(Finished\|Running\|Compiling\|Downloaded\) ' || true
}

header "COLTRANE CHANGES — Giant Steps Harmonic Analysis"
echo "" >> "$REPORT"
echo "The 'Giant Steps' substitution divides the octave into three" >> "$REPORT"
echo "equal parts (major thirds), cycling through key centers:" >> "$REPORT"
echo "  C → E → Ab → (C)" >> "$REPORT"

# --- 1. Augmented triad skeleton ---
header "1. THE AUGMENTED TRIAD SKELETON: {0, 4, 8}"

echo "The three tonal centers form an augmented triad:" >> "$REPORT"
echo "" >> "$REPORT"
$S orbits 0 4 8 >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Prime form:" >> "$REPORT"
$S prime-form C E Ab >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Forte classification:" >> "$REPORT"
$S forte C E Ab >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Interval vector:" >> "$REPORT"
$S interval-vector C E Ab >> "$REPORT" 2>/dev/null

# Pitch circle of the skeleton
$S pitch-circle C E Ab --title "Coltrane Skeleton (C,E,Ab)" \
  -o "$OUT_DIR/coltrane_skeleton_circle.svg" 2>/dev/null

# --- 2. Major triads at each node ---
header "2. MAJOR TRIADS AT EACH NODE"

echo "C major triad:" >> "$REPORT"
$S name C E G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "E major triad:" >> "$REPORT"
$S name E Ab B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Ab major triad:" >> "$REPORT"
$S name Ab C Eb >> "$REPORT" 2>/dev/null

# Pitch circles for each key center
$S pitch-circle C E G --title "C major" \
  -o "$OUT_DIR/coltrane_c_circle.svg" 2>/dev/null
$S pitch-circle E Ab B --title "E major" \
  -o "$OUT_DIR/coltrane_e_circle.svg" 2>/dev/null
$S pitch-circle Ab C Eb --title "Ab major" \
  -o "$OUT_DIR/coltrane_ab_circle.svg" 2>/dev/null

# --- 3. Common tones between adjacent key centers ---
header "3. COMMON TONES BETWEEN KEY CENTERS"

echo "C major vs E major:" >> "$REPORT"
$S common-tones C,E,G E,Ab,B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "E major vs Ab major:" >> "$REPORT"
$S common-tones E,Ab,B Ab,C,Eb >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Ab major vs C major (completing the cycle):" >> "$REPORT"
$S common-tones Ab,C,Eb C,E,G >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "All three triads together:" >> "$REPORT"
run_verbose $S common-tones C,E,G E,Ab,B Ab,C,Eb -v >> "$REPORT"

# --- 4. Voice-leading costs ---
header "4. VOICE-LEADING THROUGH THE CYCLE"

echo "C major → E major:" >> "$REPORT"
run_verbose $S voice-leading --from C4,E4,G4 --to E,Ab,B --limit 3 -v >> "$REPORT"

echo "" >> "$REPORT"
echo "E major → Ab major:" >> "$REPORT"
run_verbose $S voice-leading --from E4,Ab4,B4 --to Ab,C,Eb --limit 3 -v >> "$REPORT"

echo "" >> "$REPORT"
echo "Ab major → C major (completing the cycle):" >> "$REPORT"
run_verbose $S voice-leading --from Ab3,C4,Eb4 --to C,E,G --limit 3 -v >> "$REPORT"

# --- 5. The dominant approach pattern ---
header "5. V-I APPROACHES (THE GIANT STEPS PATTERN)"

echo "Each key center is approached by its dominant triad:" >> "$REPORT"
echo "  G → C,  B → E,  Eb → Ab" >> "$REPORT"
echo "" >> "$REPORT"

echo "Dominant triad → Tonic voice-leading (G → C):" >> "$REPORT"
run_verbose $S voice-leading --from G3,B3,D4 --to C,E,G --limit 3 -v >> "$REPORT"

echo "" >> "$REPORT"
echo "Dominant triad → Tonic voice-leading (B → E):" >> "$REPORT"
run_verbose $S voice-leading --from B3,Eb4,Gb4 --to E,Ab,B --limit 3 -v >> "$REPORT"

echo "" >> "$REPORT"
echo "Dominant triad → Tonic voice-leading (Eb → Ab):" >> "$REPORT"
run_verbose $S voice-leading --from Eb3,G3,Bb3 --to Ab,C,Eb --limit 3 -v >> "$REPORT"

# --- 6. Combined pitch content ---
header "6. COMBINED PITCH CONTENT"

echo "Union of all three major triads: {C, Eb, E, G, Ab, B}" >> "$REPORT"
echo "This is a 6-note collection (hexatonic scale):" >> "$REPORT"
echo "" >> "$REPORT"

echo "Prime form of the hexatonic collection:" >> "$REPORT"
$S prime-form C Eb E G Ab B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Forte classification:" >> "$REPORT"
$S forte C Eb E G Ab B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Interval vector:" >> "$REPORT"
$S interval-vector C Eb E G Ab B >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "Orbits (symmetry properties):" >> "$REPORT"
$S orbits C Eb E G Ab B >> "$REPORT" 2>/dev/null

# Pitch circle of the combined hexatonic collection
$S pitch-circle C Eb E G Ab B \
  --title "Hexatonic (C+E+Ab triads)" \
  -o "$OUT_DIR/coltrane_combined_circle.svg" 2>/dev/null

# --- 7. Harmonic analysis ---
header "7. HARMONIC ANALYSIS: C → E → Ab → C"

echo "Progression analyzed in C major:" >> "$REPORT"
$S analyze C,E,G E,Ab,B Ab,C,Eb C,E,G --key C >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "The III and bVI chords are chromatic — outside C major's" >> "$REPORT"
echo "diatonic set. This is the hallmark of Coltrane's approach:" >> "$REPORT"
echo "symmetric root motion (by major thirds) producing chords" >> "$REPORT"
echo "that relate to each other by geometry, not by key." >> "$REPORT"

# --- 8. Subchord analysis ---
header "8. SUBCHORDS OF THE HEXATONIC COLLECTION"

echo "3-note subsets (triads) within {C, Eb, E, G, Ab, B}:" >> "$REPORT"
$S subchords C Eb E G Ab B --size 3 --name >> "$REPORT" 2>/dev/null

echo "" >> "$REPORT"
echo "4-note subsets (seventh chords) within the hexatonic:" >> "$REPORT"
$S subchords C Eb E G Ab B --size 4 --name >> "$REPORT" 2>/dev/null

header "SUMMARY"
echo "Coltrane Changes exploit the augmented triad's T4 symmetry:" >> "$REPORT"
echo "  - Three key centers (C, E, Ab) form {0, 4, 8}" >> "$REPORT"
echo "  - Each adjacent pair shares exactly 1 common tone" >> "$REPORT"
echo "  - The union forms a hexatonic scale with rich subchord content" >> "$REPORT"
echo "  - V-I dominant approaches connect the key centers melodically" >> "$REPORT"
echo "  - The progression sounds 'outside' in any single key but is" >> "$REPORT"
echo "    internally coherent through geometric symmetry" >> "$REPORT"
echo "" >> "$REPORT"

echo "=== coltrane_changes.sh completed ==="
