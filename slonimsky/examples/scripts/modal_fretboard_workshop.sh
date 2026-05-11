#!/usr/bin/env bash
#
# Modal Fretboard Workshop: Harmonic Minor Modes on Guitar
#
# A guitar-practice-oriented workflow that combines scale-book generation,
# chord-dictionary shapes, containment analysis, and fretboard diagrams
# to produce a study sheet for the harmonic minor scale family.
#
# This is the kind of material a jazz guitar student would prepare when
# learning to navigate harmonic minor modes across the fretboard:
#   1. Generate the scale-book for harmonic minor (all 7 modes, key of A)
#   2. Show fretboard shapes for characteristic chords of each mode
#   3. Use containment analysis to find which modes contain specific chords
#   4. Generate pitch-circle diagrams for the parent scale and two
#      signature modes (Phrygian Dominant, Ultralocrian)
#   5. Show diatonic triads and seventh chords via subchords + practice-sheet
#
# Output:
#   - Text report:   examples/output/modal_workshop_report.txt
#   - SVG diagrams:  examples/output/modal_*.svg
#   - Scale book:    examples/output/modal_harm_minor_book.svg
#   - Practice sheet: examples/output/modal_harm_minor_practice.txt
#
# Usage:
#   bash slonimsky/examples/scripts/modal_fretboard_workshop.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/modal_fretboard_workshop.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -z "${SLONIMSKY:-}" ]; then
    SLONIMSKY="$SCRIPT_DIR/../../../target/debug/slonimsky"
fi

REPORT="$OUT_DIR/modal_workshop_report.txt"
> "$REPORT"

echo "=== Modal Fretboard Workshop: A Harmonic Minor ===" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 1. Scale-book: all 7 modes of harmonic minor in the key of A
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "1. Scale Book: A Harmonic Minor — All 7 Modes" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Text output of all modes in key of A
"$SLONIMSKY" scale-book harmonic-minor --keys A \
    2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# SVG pitch-circle grid for all modes × key of A
"$SLONIMSKY" scale-book harmonic-minor --keys A \
    -o "$OUT_DIR/modal_harm_minor_book.svg"

echo "(Scale-book SVG saved to modal_harm_minor_book.svg)" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 2. Pitch circles for the parent scale and signature modes
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "2. Pitch Circles: Parent Scale & Signature Modes" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# A harmonic minor = A B C D E F G# = 9 11 0 2 4 5 8
echo "A Harmonic Minor: {9, 11, 0, 2, 4, 5, 8} = A B C D E F G#" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 9 11 0 2 4 5 8 --title "A Harmonic Minor" --show-intervals \
    -o "$OUT_DIR/modal_a_harm_minor_circle.svg"

# E Phrygian Dominant (5th mode) = E F G# A B C D = 4 5 8 9 11 0 2
echo "E Phrygian Dominant: {4, 5, 8, 9, 11, 0, 2} = E F G# A B C D" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 4 5 8 9 11 0 2 --title "E Phrygian Dominant" --show-intervals \
    -o "$OUT_DIR/modal_e_phrygian_dom_circle.svg"

# G# Ultralocrian (7th mode) = G# A B C D E F = 8 9 11 0 2 4 5
echo "G# Ultralocrian: {8, 9, 11, 0, 2, 4, 5} = G# A B C D E F" | tee -a "$REPORT"
"$SLONIMSKY" pitch-circle 8 9 11 0 2 4 5 --title "G# Ultralocrian" --show-intervals \
    -o "$OUT_DIR/modal_gs_ultralocrian_circle.svg"

echo "" | tee -a "$REPORT"
echo "→ All three modes share the same pitch content (modes of one parent scale)." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 3. Fretboard shapes for characteristic chords
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "3. Fretboard Shapes: Characteristic Chords" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# Am(maj7) — i chord of A harmonic minor = A C E G# = 9 0 4 8
echo "── Am(maj7): {9, 0, 4, 8} — the i chord ──" | tee -a "$REPORT"
"$SLONIMSKY" chord-dictionary 9 0 4 8 --max-results 4 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
# Open position shape: x-0-2-1-1-0
"$SLONIMSKY" fretboard x-0-2-1-1-0 --title "Am(maj7) — open" \
    -o "$OUT_DIR/modal_fret_am_maj7.svg"

# E7 — V chord (Phrygian Dominant tonic) = E G# B D = 4 8 11 2
echo "── E7: {4, 8, 11, 2} — the V7 chord ──" | tee -a "$REPORT"
"$SLONIMSKY" chord-dictionary 4 8 11 2 --max-results 4 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
# Open position shape: 0-2-0-1-0-0
"$SLONIMSKY" fretboard 0-2-0-1-0-0 --title "E7 — open" \
    -o "$OUT_DIR/modal_fret_e7.svg"

# Bdim7 — vii° chord = B D F G# = 11 2 5 8
echo "── Bdim7: {11, 2, 5, 8} — the vii° chord ──" | tee -a "$REPORT"
"$SLONIMSKY" chord-dictionary 11 2 5 8 --max-results 4 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"
# Common voicing: x-2-3-1-3-x
"$SLONIMSKY" fretboard x-2-3-1-3-x --title "Bdim7" \
    -o "$OUT_DIR/modal_fret_bdim7.svg"

# ─────────────────────────────────────────────────────────────────────
# 4. Containment analysis: which modes contain E7?
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "4. Containment: Which Scales Contain E7?" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "E7 = {4, 8, 11, 2} = E G# B D" | tee -a "$REPORT"
"$SLONIMSKY" contains 4 8 11 2 --limit 10 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Closest sets to E7:" | tee -a "$REPORT"
"$SLONIMSKY" closest 4 8 11 2 --limit 8 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 5. Diatonic chord analysis via subchords
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "5. Diatonic Triads of A Harmonic Minor" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "A Harmonic Minor = {9, 11, 0, 2, 4, 5, 8}" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "All triadic subsets (size 3) with names:" | tee -a "$REPORT"
"$SLONIMSKY" subchords 9 11 0 2 4 5 8 --size 3 --name 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "All tetrachord subsets (size 4) with names:" | tee -a "$REPORT"
"$SLONIMSKY" subchords 9 11 0 2 4 5 8 --size 4 --name 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 6. Common tones between adjacent diatonic chords
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "6. Common Tones: Am(maj7) ↔ E7 ↔ Bdim7" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Am(maj7) ↔ E7:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 9,0,4,8 4,8,11,2 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "E7 ↔ Bdim7:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 4,8,11,2 11,2,5,8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "All three chords:" | tee -a "$REPORT"
"$SLONIMSKY" common-tones 9,0,4,8 4,8,11,2 11,2,5,8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 7. Practice sheet for A harmonic minor
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "7. Practice Sheet: A Harmonic Minor" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

"$SLONIMSKY" practice-sheet --key A --scale harmonic-minor \
    2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

"$SLONIMSKY" practice-sheet --key A --scale harmonic-minor \
    -o "$OUT_DIR/modal_harm_minor_practice.svg"
echo "(Practice sheet SVG saved)" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# 8. Set-class analysis: prime form + forte numbers
# ─────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "8. Set-Class Identity of Key Chords" | tee -a "$REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Am(maj7) prime form:" | tee -a "$REPORT"
"$SLONIMSKY" forte 9 0 4 8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "E7 prime form:" | tee -a "$REPORT"
"$SLONIMSKY" forte 4 8 11 2 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "Bdim7 prime form:" | tee -a "$REPORT"
"$SLONIMSKY" forte 11 2 5 8 -v 2>&1 | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

echo "→ Note: both E7 and Am(maj7) are set class 4-27; Bdim7 is 4-28 (T3/T6 symmetric)." | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

# ─────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────
echo "=== Workshop Complete ===" | tee -a "$REPORT"

SVG_COUNT=$(ls "$OUT_DIR"/modal_*.svg 2>/dev/null | wc -l)
REPORT_LINES=$(wc -l < "$REPORT")
echo ""
echo "Generated $SVG_COUNT SVG diagrams and ${REPORT_LINES}-line text report."
echo "Report: $REPORT"
echo "SVGs:   $OUT_DIR/modal_*.svg"
