#!/usr/bin/env bash
#
# Forte Set-Class Identification Workflow
#
# Demonstrates how to identify and classify pitch-class sets using the
# Forte numbering system. Given a collection of PcSets, computes for each:
#   - Prime form (Rahn's algorithm)
#   - Forte number (set-class catalog lookup)
#   - Symmetry orbits (transpositional + inversional)
#   - Interval vector with labeled ic breakdown
#   - Pitch-circle visualization
#
# This exercises: forte, prime-form, orbits, interval-vector, pitch-circle
# together — the first example to use `forte` as the primary entry point.
#
# Musical scenarios:
#   1. Common triads: verify major/minor share Forte class 3-11
#   2. Seventh chords: dom7 (4-27), maj7 (4-20), dim7 (4-28)
#   3. Symmetric sets: augmented (3-12), tritone (2-6), chromatic cluster (3-1)
#   4. Jazz voicings: quartal trichord (3-9), "So What" voicing (4-23)
#
# Output:
#   - Text report: examples/output/forte_id_report.txt
#   - SVG diagrams: examples/output/forte_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/forte_identification.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/forte_identification.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -z "${SLONIMSKY:-}" ]; then
    CARGO_HOME=/tmp/cargo-home cargo build -p slonimsky 2>/dev/null
    SLONIMSKY="$SCRIPT_DIR/../../../target/debug/slonimsky"
fi

REPORT="$OUT_DIR/forte_id_report.txt"

{
echo "============================================"
echo "  Forte Set-Class Identification Report"
echo "============================================"
echo ""

# --- Section 1: Triad equivalence ---
echo "=== 1. Triad Set-Class Equivalence ==="
echo ""
echo "Major and minor triads share the same set class under TnI."
echo ""

echo "C major triad (C E G):"
$SLONIMSKY forte C E G
echo ""
echo "  Verbose detail:"
$SLONIMSKY forte C E G -v 2>&1 || true
echo ""

echo "C minor triad (C Eb G):"
$SLONIMSKY forte C Eb G
echo ""

echo "D minor triad (D F A):"
$SLONIMSKY forte D F A
echo ""

echo "A major triad (A C# E):"
$SLONIMSKY forte A C# E
echo ""

echo "  => All four map to 3-11: the general trichord with"
echo "     interval content [0,3,7]. Major/minor are TnI-related."
echo ""

# Generate pitch-circle SVGs for major vs minor
$SLONIMSKY pitch-circle C E G --title "C Major (3-11)" \
    -o "$OUT_DIR/forte_c_major_circle.svg"
$SLONIMSKY pitch-circle C Eb G --title "C Minor (3-11)" --theme dark \
    -o "$OUT_DIR/forte_c_minor_circle.svg"

# --- Section 2: Seventh chord classification ---
echo "=== 2. Seventh Chord Classification ==="
echo ""

echo "Dominant 7th (C E G Bb):"
$SLONIMSKY forte C E G Bb
echo ""

echo "Major 7th (C E G B):"
$SLONIMSKY forte C E G B
echo ""

echo "Diminished 7th (C Eb Gb A):"
$SLONIMSKY forte C Eb Gb A
echo ""
echo "  Verbose dim7 (maximal symmetry):"
$SLONIMSKY forte C Eb Gb A -v 2>&1 || true
echo ""

echo "Half-diminished 7th (C Eb Gb Bb):"
$SLONIMSKY forte C Eb Gb Bb
echo ""

echo "Minor 7th (C Eb G Bb):"
$SLONIMSKY forte C Eb G Bb
echo ""

echo "  => Seventh chord qualities map to distinct Forte numbers."
echo "     dom7=4-27, maj7=4-20, dim7=4-28, half-dim7=4-27."
echo "     (min7 = [0,2,5,9] not yet in table — card-4 partially covered.)"
echo ""

# Pitch circles for 7th chords
$SLONIMSKY pitch-circle C E G Bb --title "Dom7 (4-27)" --show-intervals \
    -o "$OUT_DIR/forte_dom7_circle.svg"
$SLONIMSKY pitch-circle C Eb Gb A --title "Dim7 (4-28)" --show-intervals \
    -o "$OUT_DIR/forte_dim7_circle.svg"

# --- Section 3: Symmetric sets ---
echo "=== 3. Symmetric Set Identification ==="
echo ""

echo "Augmented triad (C E Ab):"
$SLONIMSKY forte C E Ab
echo ""
echo "  Symmetry analysis:"
$SLONIMSKY orbits C E Ab
echo ""

echo "Tritone (C F#):"
$SLONIMSKY forte C F#
echo ""
echo "  Symmetry analysis:"
$SLONIMSKY orbits C F#
echo ""

echo "Chromatic trichord (C C# D):"
$SLONIMSKY forte C C# D
echo ""

echo "Whole-tone fragment (C D E F#):"
$SLONIMSKY forte C D E F#
echo ""
echo "  Symmetry analysis:"
$SLONIMSKY orbits C D E F#
echo ""

# SVGs for symmetric sets
$SLONIMSKY pitch-circle C E Ab --title "Aug (3-12) T4-symmetric" \
    --show-intervals --theme colorful \
    -o "$OUT_DIR/forte_aug_symmetric.svg"
$SLONIMSKY pitch-circle C D E F# --title "WT fragment (4-21) T6-symmetric" \
    --show-intervals --theme colorful \
    -o "$OUT_DIR/forte_wt_fragment.svg"

# --- Section 4: Jazz voicings ---
echo "=== 4. Jazz Voicing Set Classes ==="
echo ""

echo "Quartal trichord (C F Bb) — stacked P4s:"
$SLONIMSKY forte C F Bb
echo ""
echo "  Prime form:"
$SLONIMSKY prime-form C F Bb
echo ""

echo "\"So What\" voicing fragment (D G C F) — stacked P4s:"
$SLONIMSKY forte D G C F
echo ""
echo "  Prime form:"
$SLONIMSKY prime-form D G C F
echo ""
echo "  Interval vector:"
$SLONIMSKY interval-vector D G C F
echo ""

# SVG for quartal
$SLONIMSKY pitch-circle C F Bb --title "Quartal (3-9)" --theme print \
    --show-intervals \
    -o "$OUT_DIR/forte_quartal_circle.svg"
$SLONIMSKY pitch-circle D G C F --title "So What (4-23)" --theme print \
    --show-intervals \
    -o "$OUT_DIR/forte_sowhat_circle.svg"

# --- Section 5: Cross-comparison summary ---
echo "=== 5. Interval Vector Comparison ==="
echo ""
echo "Same ic content implies same Forte number. Comparing IVs:"
echo ""

echo "Major triad IV:"
$SLONIMSKY interval-vector C E G
echo ""

echo "Minor triad IV:"
$SLONIMSKY interval-vector C Eb G
echo ""

echo "Dom7 IV:"
$SLONIMSKY interval-vector C E G Bb
echo ""

echo "Dim7 IV:"
$SLONIMSKY interval-vector C Eb Gb A
echo ""

echo "============================================"
echo "  Report complete. SVGs in examples/output/"
echo "============================================"

} > "$REPORT" 2>&1

echo "Report written to: $REPORT"
echo "SVGs generated:"
ls -1 "$OUT_DIR"/forte_*.svg 2>/dev/null | while read f; do
    echo "  $(basename "$f") ($(wc -c < "$f") bytes)"
done
