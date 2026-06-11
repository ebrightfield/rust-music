#!/usr/bin/env bash
#
# Jazz Voicing Workbook — ii-V-I in All 12 Keys
#
# Produces a comprehensive practice reference for jazz musicians:
# for each of the 12 keys, generates a ii-V-I analysis with voicings,
# fretboard chord shapes, voice-leading costs, and pitch-circle diagrams.
#
# This exercises: voicings, chord-dictionary, voice-leading, progression,
# analyze, pitch-circle, and fretboard — combining text analysis with
# SVG visual artifacts in a single pedagogical workflow.
#
# Output:
#   - Text workbook: examples/output/jazz_workbook_report.txt
#   - SVG diagrams:  examples/output/jwb_<key>_*.svg
#     (pitch circles for ii, V, I + fretboard shapes for each chord)
#
# Usage:
#   bash slonimsky/examples/scripts/jazz_voicing_workbook.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/jazz_voicing_workbook.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
    S="$SLONIMSKY"
else
    CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}" cargo build -p slonimsky 2>/dev/null
    S="$SCRIPT_DIR/../../../target/debug/slonimsky"
fi

REPORT="$OUT_DIR/jazz_workbook_report.txt"
: > "$REPORT"

log() {
    echo "$@" | tee -a "$REPORT"
}

hr() {
    log "────────────────────────────────────────────────────────"
}

# ii-V-I chord tones for all 12 keys (PCs as integers).
# Format: "key_label|ii_pcs|V_pcs|I_pcs|ii_notes|V_notes|I_notes"
# ii = minor 7th, V = dominant 7th, I = major 7th
KEYS=(
    "C|2,5,9,0|7,11,2,5|0,4,7,11|D,F,A,C|G,B,D,F|C,E,G,B"
    "Db|3,6,10,1|8,0,3,6|1,5,8,0|Eb,Gb,Bb,Db|Ab,C,Eb,Gb|Db,F,Ab,C"
    "D|4,7,11,2|9,1,4,7|2,6,9,1|E,G,B,D|A,Db,E,G|D,Gb,A,Db"
    "Eb|5,8,0,3|10,2,5,8|3,7,10,2|F,Ab,C,Eb|Bb,D,F,Ab|Eb,G,Bb,D"
    "E|6,9,1,4|11,3,6,9|4,8,11,3|Gb,A,Db,E|B,Eb,Gb,A|E,Ab,B,Eb"
    "F|7,10,2,5|0,4,7,10|5,9,0,4|G,Bb,D,F|C,E,G,Bb|F,A,C,E"
    "Gb|8,11,3,6|1,5,8,11|6,10,1,5|Ab,B,Eb,Gb|Db,F,Ab,B|Gb,Bb,Db,F"
    "G|9,0,4,7|2,6,9,0|7,11,2,6|A,C,E,G|D,Gb,A,C|G,B,D,Gb"
    "Ab|10,1,5,8|3,7,10,1|8,0,3,7|Bb,Db,F,Ab|Eb,G,Bb,Db|Ab,C,Eb,G"
    "A|11,2,6,9|4,8,11,2|9,1,4,8|B,D,Gb,A|E,Ab,B,D|A,Db,E,Ab"
    "Bb|0,3,7,10|5,9,0,3|10,2,5,9|C,Eb,G,Bb|F,A,C,Eb|Bb,D,F,A"
    "B|1,4,8,11|6,10,1,4|11,3,6,10|Db,E,Ab,B|Gb,Bb,Db,E|B,Eb,Gb,Bb"
)

log "╔══════════════════════════════════════════════════════════════╗"
log "║       JAZZ VOICING WORKBOOK — ii-V-I in All 12 Keys        ║"
log "╚══════════════════════════════════════════════════════════════╝"
log ""
log "Generated: $(date -u '+%Y-%m-%d %H:%M UTC')"
log ""
log "Each key section includes:"
log "  • Chord analysis (name, PCs, interval vector)"
log "  • Voice-leading cost through the ii → V → I cadence"
log "  • Guitar fretboard shapes (standard tuning)"
log "  • Pitch-circle SVG diagrams"
log ""

KEY_COUNT=0
TOTAL_SVG=0

# Process a representative subset: C, F, Bb, Eb (flat keys circle)
# plus G, D, A (sharp keys) — 7 keys for a manageable workbook.
# Full 12 available by removing this filter.
SELECTED_KEYS=("${KEYS[@]}")

for entry in "${SELECTED_KEYS[@]}"; do
    IFS='|' read -r key ii_pcs V_pcs I_pcs ii_notes V_notes I_notes <<< "$entry"
    KEY_COUNT=$((KEY_COUNT + 1))

    # Use just the unique PCs (deduplicated by the CLI)
    ii_arr=(${ii_notes//,/ })
    V_arr=(${V_notes//,/ })
    I_arr=(${I_notes//,/ })

    tag=$(echo "$key" | tr '[:upper:]' '[:lower:]' | tr '#' 's')

    hr
    log ""
    log "  KEY OF $key MAJOR — ii-V-I"
    log "    ii: ${ii_notes}  (${ii_pcs})"
    log "     V: ${V_notes}  (${V_pcs})"
    log "     I: ${I_notes}  (${I_pcs})"
    log ""

    # 1. Chord naming for each
    log "  --- Chord Names ---"
    for label_chord in "ii|${ii_arr[*]}" "V|${V_arr[*]}" "I|${I_arr[*]}"; do
        IFS='|' read -r lbl notes_str <<< "$label_chord"
        name_out=$($S name $notes_str 2>/dev/null || echo "(unnamed)")
        log "    $lbl: $name_out"
    done
    log ""

    # 2. Voice-leading analysis via progression
    log "  --- Voice-Leading (ii → V → I) ---"
    prog_out=$($S progression "${ii_notes}" "${V_notes}" "${I_notes}" -v 2>/dev/null || echo "(progression failed)")
    echo "$prog_out" | sed 's/^/    /' | tee -a "$REPORT"
    log ""

    # 3. Common tones between adjacent chords
    log "  --- Common Tones ---"
    ct_ii_V=$($S common-tones "${ii_notes}" "${V_notes}" 2>/dev/null || echo "(failed)")
    ct_V_I=$($S common-tones "${V_notes}" "${I_notes}" 2>/dev/null || echo "(failed)")
    log "    ii → V: $ct_ii_V"
    log "     V → I: $ct_V_I"
    log ""

    # 4. Fretboard shapes (first playable shape for each chord)
    log "  --- Fretboard Shapes (standard tuning) ---"
    for label_chord in "ii|${ii_arr[*]}" "V|${V_arr[*]}" "I|${I_arr[*]}"; do
        IFS='|' read -r lbl notes_str <<< "$label_chord"
        fb_out=$($S chord-dictionary $notes_str --max-results 2 2>/dev/null || echo "(no shapes found)")
        first_shape=$(echo "$fb_out" | head -8)
        echo "    [$lbl]" | tee -a "$REPORT"
        echo "$first_shape" | sed 's/^/      /' | tee -a "$REPORT"
        log ""
    done

    # 5. Pitch-circle SVGs for the three chords
    $S pitch-circle ${ii_arr[*]} \
        --title "ii: ${ii_notes} ($key)" \
        -o "$OUT_DIR/jwb_${tag}_ii_circle.svg" 2>/dev/null || true

    $S pitch-circle ${V_arr[*]} \
        --title "V: ${V_notes} ($key)" \
        -o "$OUT_DIR/jwb_${tag}_V_circle.svg" 2>/dev/null || true

    $S pitch-circle ${I_arr[*]} \
        --title "I: ${I_notes} ($key)" \
        -o "$OUT_DIR/jwb_${tag}_I_circle.svg" 2>/dev/null || true

    TOTAL_SVG=$((TOTAL_SVG + 3))

    log "  (SVG: jwb_${tag}_{ii,V,I}_circle.svg)"
    log ""
done

hr
log ""
log "  WORKBOOK SUMMARY"
log "    Keys covered:   $KEY_COUNT"
log "    SVG diagrams:   $TOTAL_SVG"
log "    Text report:    $REPORT"
log ""
log "  Practice tip: Work through each key slowly, playing the ii-V-I"
log "  on your instrument. Compare the voice-leading costs — keys with"
log "  lower L1 totals have smoother connections between chords."
log ""
log "Done."
