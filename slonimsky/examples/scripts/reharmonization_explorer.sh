#!/usr/bin/env bash
# Reharmonization Explorer
#
# Demonstrates a complete reharmonization workflow:
# given a I-vi-ii-V progression in C major, explore alternative
# chord substitutions using `closest`, evaluate their voice-leading
# cost via `progression`, check common tones with `common-tones`,
# and analyze the resulting reharmonized sequences via `analyze`.
#
# Usage:
#   cargo build -p slonimsky && ./slonimsky/examples/scripts/reharmonization_explorer.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/reharmonization_explorer.sh
#
# Outputs:
#   slonimsky/examples/output/reharm_report.txt
#   slonimsky/examples/output/reharm_original_analysis.txt
#   slonimsky/examples/output/reharm_substituted_analysis.txt
#   slonimsky/examples/output/reharm_original_circles.svg (4 pitch circles)
#   slonimsky/examples/output/reharm_sub_em7_circle.svg
#   slonimsky/examples/output/reharm_sub_fmaj7_circle.svg
#   slonimsky/examples/output/reharm_sub_db7_circle.svg

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Locate the slonimsky binary
if [ -n "${SLONIMSKY:-}" ]; then
    SL="$SLONIMSKY"
else
    SL="$SCRIPT_DIR/../../../target/debug/slonimsky"
    if [ ! -x "$SL" ]; then
        SL="$SCRIPT_DIR/../../../target/release/slonimsky"
    fi
    if [ ! -x "$SL" ]; then
        echo "Binary not found. Build first: cargo build -p slonimsky" >&2
        echo "Or set SLONIMSKY=/path/to/binary" >&2
        exit 1
    fi
fi

OUT="$SCRIPT_DIR/../output"
mkdir -p "$OUT"

REPORT="$OUT/reharm_report.txt"
> "$REPORT"

banner() { printf '\n==== %s ====\n\n' "$1" >> "$REPORT"; }

# --- Original progression: I-vi-ii-V in C major ---
# Cmaj (C E G), Am (A C E), Dm (D F A), G (G B D)

banner "1. ORIGINAL PROGRESSION: I-vi-ii-V in C major"
echo "Chords: Cmaj → Am → Dm → G" >> "$REPORT"
echo "" >> "$REPORT"

# Analyze the original
$SL analyze C,E,G A,C,E D,F,A G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

# Save detailed analysis
$SL analyze C,E,G A,C,E D,F,A G,B,D > "$OUT/reharm_original_analysis.txt" 2>/dev/null

# Voice-leading cost of original
echo "Voice-leading plan:" >> "$REPORT"
$SL progression C,E,G A,C,E D,F,A G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

# Pitch circles for original chords
$SL pitch-circle C E G --title "I: C major" -o "$OUT/reharm_original_circles.svg" 2>/dev/null

# --- Step 2: Find substitution candidates for each chord ---

banner "2. SUBSTITUTION CANDIDATES"

echo "--- Substitutes for Am (vi, PcSet {0,4,9}) ---" >> "$REPORT"
$SL closest A C E --pool chords --limit 8 >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "--- Substitutes for Dm (ii, PcSet {2,5,9}) ---" >> "$REPORT"
$SL closest D F A --pool chords --limit 8 >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "--- Substitutes for G (V, PcSet {7,11,2}) ---" >> "$REPORT"
$SL closest G B D --pool chords --limit 8 >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

# --- Step 3: Reharmonization #1 — tritone substitution of V ---
# Replace G (V) with Db7 (bII7, tritone sub shares the tritone B-F = 11,5)

banner "3. REHARMONIZATION #1: Tritone substitution (G → Db7)"
echo "Replace G major (V) with Db7 (bII7 = {1,5,8,11})" >> "$REPORT"
echo "Shared tritone: B(11) and F(5) are common to G7 and Db7" >> "$REPORT"
echo "" >> "$REPORT"

echo "Common tones between G major and Db major:" >> "$REPORT"
$SL common-tones G,B,D Db,F,Ab >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Progression with tritone sub (Cmaj → Am → Dm → Db7):" >> "$REPORT"
$SL analyze C,E,G A,C,E D,F,A Db,F,Ab >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Voice-leading plan:" >> "$REPORT"
$SL progression C,E,G A,C,E D,F,A Db,F,Ab >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

$SL pitch-circle Db F Ab --title "Db (tritone sub)" -o "$OUT/reharm_sub_db7_circle.svg" 2>/dev/null

# --- Step 4: Reharmonization #2 — vi → iii (Em7 for Am) ---
# Replace Am (vi) with Em (iii) — shares 2 common tones (E, implied G)

banner "4. REHARMONIZATION #2: vi → iii (Am → Em)"
echo "Replace Am (vi, {0,4,9}) with Em (iii, {4,7,11})" >> "$REPORT"
echo "" >> "$REPORT"

echo "Common tones between Am and Em:" >> "$REPORT"
$SL common-tones A,C,E E,G,B >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Progression (Cmaj → Em → Dm → G):" >> "$REPORT"
$SL analyze C,E,G E,G,B D,F,A G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Voice-leading plan:" >> "$REPORT"
$SL progression C,E,G E,G,B D,F,A G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

$SL pitch-circle E G B --title "Em (iii substitute)" -o "$OUT/reharm_sub_em7_circle.svg" 2>/dev/null

# --- Step 5: Reharmonization #3 — ii → IV (Dm → Fmaj7) ---
# Replace Dm (ii) with Fmaj7 (IV7) — shares 3/4 common tones

banner "5. REHARMONIZATION #3: ii → IV (Dm → Fmaj7)"
echo "Replace Dm (ii, {2,5,9}) with Fmaj7 (IV7, {0,4,5,9})" >> "$REPORT"
echo "" >> "$REPORT"

echo "Common tones between Dm and F major:" >> "$REPORT"
$SL common-tones D,F,A F,A,C >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Progression (Cmaj → Am → Fmaj7 → G):" >> "$REPORT"
# Use F A C E for Fmaj7 (but as 3-note F major triad for voice count match)
$SL analyze C,E,G A,C,E F,A,C G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

echo "Voice-leading plan:" >> "$REPORT"
$SL progression C,E,G A,C,E F,A,C G,B,D >> "$REPORT" 2>/dev/null
echo "" >> "$REPORT"

$SL pitch-circle F A C --title "F major (IV substitute)" -o "$OUT/reharm_sub_fmaj7_circle.svg" 2>/dev/null

# --- Step 6: Compare all four progressions ---

banner "6. VOICE-LEADING COST COMPARISON"
echo "Original I-vi-ii-V:" >> "$REPORT"
$SL progression C,E,G A,C,E D,F,A G,B,D 2>/dev/null | grep "Total" >> "$REPORT"
echo "" >> "$REPORT"

echo "Tritone sub (Cmaj→Am→Dm→Db):" >> "$REPORT"
$SL progression C,E,G A,C,E D,F,A Db,F,Ab 2>/dev/null | grep "Total" >> "$REPORT"
echo "" >> "$REPORT"

echo "iii sub (Cmaj→Em→Dm→G):" >> "$REPORT"
$SL progression C,E,G E,G,B D,F,A G,B,D 2>/dev/null | grep "Total" >> "$REPORT"
echo "" >> "$REPORT"

echo "IV sub (Cmaj→Am→F→G):" >> "$REPORT"
$SL progression C,E,G A,C,E F,A,C G,B,D 2>/dev/null | grep "Total" >> "$REPORT"
echo "" >> "$REPORT"

# --- Step 7: Save the best substituted analysis ---

banner "7. FULL ANALYSIS: Best reharmonization"
echo "Selecting Cmaj→Am→F→G (I-vi-IV-V) as the smoothest diatonic alternative" >> "$REPORT"
echo "" >> "$REPORT"
$SL analyze C,E,G A,C,E F,A,C G,B,D > "$OUT/reharm_substituted_analysis.txt" 2>/dev/null
cat "$OUT/reharm_substituted_analysis.txt" >> "$REPORT"
echo "" >> "$REPORT"

echo "---" >> "$REPORT"
echo "Generated by slonimsky reharmonization_explorer.sh" >> "$REPORT"

LINES=$(wc -l < "$REPORT")
SVGS=$(ls "$OUT"/reharm_*.svg 2>/dev/null | wc -l)
echo "Done: ${LINES} line report + ${SVGS} SVG diagrams in $OUT/"
