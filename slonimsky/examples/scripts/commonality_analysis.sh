#!/usr/bin/env bash
#
# Commonality & Symmetry Analysis
#
# Demonstrates a comparative analysis workflow: given pairs and groups
# of pitch-class sets (scales, chords, modes), compute their shared
# pitch content via `common-tones`, explore their symmetry properties
# via `orbits`, and classify them via `prime-form`. This is the kind
# of analysis a theory student or composer uses to understand
# relationships between harmonic materials.
#
# Musical scenarios covered:
#   1. Modal interchange: C major vs C natural minor — which notes differ?
#   2. Related modes: D dorian vs G mixolydian vs C ionian — shared tones
#   3. Tritone substitution: G7 vs Db7 — common tritone, different shells
#   4. Symmetric structures: dim7, aug triad, whole-tone — orbit analysis
#   5. Chord-scale overlap: Cmaj7 tones within C major pentatonic
#
# Output:
#   - Text report: examples/output/commonality_report.txt
#   - SVG diagrams: examples/output/common_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/commonality_analysis.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/commonality_analysis.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -z "${SLONIMSKY:-}" ]; then
    SLONIMSKY="$(CARGO_HOME=/tmp/cargo-home cargo build -p slonimsky --message-format=short 2>&1 | tail -1)"
    SLONIMSKY="$SCRIPT_DIR/../../../target/debug/slonimsky"
fi

REPORT="$OUT_DIR/commonality_report.txt"
> "$REPORT"

echo "=== Commonality & Symmetry Analysis ===" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 1. Modal interchange: C major vs C natural minor
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "1. Modal Interchange: C Major vs C Natural Minor" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# C major = C D E F G A B = 0 2 4 5 7 9 11
# C natural minor = C D Eb F G Ab Bb = 0 2 3 5 7 8 10
echo "C Major:         {0, 2, 4, 5, 7, 9, 11}" | tee -a "$REPORT"
echo "C Natural Minor: {0, 2, 3, 5, 7, 8, 10}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Common tones:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 0,2,4,5,7,9,11 0,2,3,5,7,8,10 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Prime form of C major scale:" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 0 2 4 5 7 9 11 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Pitch circles showing both scales side by side
"$SLONIMSKY" pitch-circle 0 2 4 5 7 9 11 --title "C Major" \
    -o "$OUT_DIR/common_c_major_scale.svg"
"$SLONIMSKY" pitch-circle 0 2 3 5 7 8 10 --title "C Natural Minor" \
    -o "$OUT_DIR/common_c_minor_scale.svg"

# ─────────────────────────────────────────────────────────────────────
# 2. Related modes: D dorian / G mixolydian / C ionian
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "2. Related Modes: D Dorian / G Mixolydian / C Ionian" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# All three are modes of C major — they share ALL 7 notes
# D dorian = D E F G A B C = 2 4 5 7 9 11 0
# G mixolydian = G A B C D E F = 7 9 11 0 2 4 5
# C ionian = C D E F G A B = 0 2 4 5 7 9 11
echo "D Dorian:      {0, 2, 4, 5, 7, 9, 11} (same PCs as C major)" | tee -a "$REPORT"
echo "G Mixolydian:  {0, 2, 4, 5, 7, 9, 11} (same PCs as C major)" | tee -a "$REPORT"
echo "C Ionian:      {0, 2, 4, 5, 7, 9, 11}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Common tones (all three modes):" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 0,2,4,5,7,9,11 2,4,5,7,9,11,0 7,9,11,0,2,4,5 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ All 7 tones are shared: modes of the same parent scale." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Now compare with D dorian vs D melodic minor (different!)
# D melodic minor ascending = D E F G A B C# = 2 4 5 7 9 11 1
echo "Now: D Dorian vs D Melodic Minor (ascending):" | tee -a "$REPORT"
echo "D Melodic Minor: {1, 2, 4, 5, 7, 9, 11}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
echo "Common tones:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 0,2,4,5,7,9,11 1,2,4,5,7,9,11 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ 6 of 7 tones shared — only the 7th degree differs (C vs C#)." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 3. Tritone substitution: G7 vs Db7
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "3. Tritone Substitution: G7 vs Db7" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# G7 = G B D F = 7 11 2 5
# Db7 = Db F Ab Cb = 1 5 8 11
echo "G7:  {2, 5, 7, 11}  (G B D F)" | tee -a "$REPORT"
echo "Db7: {1, 5, 8, 11}  (Db F Ab Cb)" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Common tones:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 2,5,7,11 1,5,8,11 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ Tritone sub shares the tritone (F & B = PCs 5 & 11)." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Prime form of G7:" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 7 11 2 5 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Prime form of Db7:" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 1 5 8 11 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ Same set class — tritone subs are T6-related dom7 chords." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Pitch circles for both
"$SLONIMSKY" pitch-circle 7 11 2 5 --title "G7" --show-intervals \
    -o "$OUT_DIR/common_g7_circle.svg"
"$SLONIMSKY" pitch-circle 1 5 8 11 --title "Db7 (tritone sub)" --show-intervals \
    -o "$OUT_DIR/common_db7_circle.svg"

# ─────────────────────────────────────────────────────────────────────
# 4. Symmetric structures: orbits analysis
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "4. Symmetric Structures: Orbit Analysis" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Diminished 7th — maximally symmetric (T3, T6)
echo "── Diminished 7th (C Eb Gb Bbb = {0, 3, 6, 9}) ──" | tee -a "$REPORT"
"$SLONIMSKY" orbits 0 3 6 9 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 0 3 6 9 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 0 3 6 9 --title "Dim7 — T3/T6 Symmetric" --show-intervals \
    -o "$OUT_DIR/common_dim7_orbits.svg"

# Augmented triad — T4 symmetric
echo "── Augmented Triad (C E G# = {0, 4, 8}) ──" | tee -a "$REPORT"
"$SLONIMSKY" orbits 0 4 8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 0 4 8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 0 4 8 --title "Aug Triad — T4 Symmetric" --show-intervals \
    -o "$OUT_DIR/common_aug_orbits.svg"

# Whole-tone scale — T2, T4, T6
echo "── Whole-Tone Scale ({0, 2, 4, 6, 8, 10}) ──" | tee -a "$REPORT"
"$SLONIMSKY" orbits 0 2 4 6 8 10 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" prime-form 0 2 4 6 8 10 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 0 2 4 6 8 10 --title "Whole-Tone — T2/T4/T6" --show-intervals \
    -o "$OUT_DIR/common_wholetone_orbits.svg"

# Major triad — no symmetry (control case)
echo "── Major Triad (C E G = {0, 4, 7}) — no symmetry ──" | tee -a "$REPORT"
"$SLONIMSKY" orbits 0 4 7 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 5. Chord-scale overlap: Cmaj7 within pentatonic
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "5. Chord-Scale Overlap: Cmaj7 in C Major Pentatonic" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Cmaj7 = C E G B = 0 4 7 11
# C major pentatonic = C D E G A = 0 2 4 7 9
echo "Cmaj7:             {0, 4, 7, 11}" | tee -a "$REPORT"
echo "C Major Pentatonic: {0, 2, 4, 7, 9}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Common tones:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 0,4,7,11 0,2,4,7,9 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ 3 of 4 chord tones in the pentatonic (C, E, G); only B is outside." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Subchords of C major pentatonic (triads):" | tee -a "$REPORT"
"$SLONIMSKY" subchords 0 2 4 7 9 --size 3 --name 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

"$SLONIMSKY" pitch-circle 0 4 7 11 --title "Cmaj7" --show-intervals \
    -o "$OUT_DIR/common_cmaj7_circle.svg"
"$SLONIMSKY" pitch-circle 0 2 4 7 9 --title "C Major Pentatonic" --show-intervals \
    -o "$OUT_DIR/common_c_pent_circle.svg"

echo "" | tee -a "$REPORT"
echo "=== Analysis Complete ===" | tee -a "$REPORT"

# Summary
SVG_COUNT=$(ls "$OUT_DIR"/common_*.svg 2>/dev/null | wc -l)
REPORT_LINES=$(wc -l < "$REPORT")
echo ""
echo "Generated $SVG_COUNT SVG diagrams and ${REPORT_LINES}-line text report."
echo "Report: $REPORT"
echo "SVGs:   $OUT_DIR/common_*.svg"
