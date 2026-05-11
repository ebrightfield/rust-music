#!/usr/bin/env bash
#
# Set-Class Analysis Report
#
# Demonstrates a real music-theory workflow: take several important pitch-class
# sets (major triad, dominant 7th, diminished 7th, whole-tone fragment, Forte
# 6-Z44) and produce a combined analysis report in text plus SVG diagrams.
#
# For each set, the script generates:
#   - Chord name (via `name`)
#   - Interval matrix (text + SVG)
#   - Interval vector (text + SVG)
#   - Pitch-circle diagram (SVG)
#   - Subchord enumeration with names
#
# Output: text report to stdout AND to examples/output/analysis_report.txt,
#         SVG diagrams to examples/output/analysis_*.svg
#
# Usage:
#   bash slonimsky/examples/scripts/set_class_analysis.sh
#   # Or with a pre-built binary:
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/set_class_analysis.sh

set -euo pipefail

SLONIMSKY="${SLONIMSKY:-cargo run -p slonimsky --}"
OUT_DIR="slonimsky/examples/output"
REPORT="$OUT_DIR/analysis_report.txt"

mkdir -p "$OUT_DIR"

# Truncate the report file
> "$REPORT"

log() {
    echo "$@" | tee -a "$REPORT"
}

analyze_set() {
    local label="$1"
    local file_tag="$2"
    shift 2
    local pcs=("$@")

    log "================================================================"
    log "  $label: ${pcs[*]}"
    log "================================================================"
    log ""

    # Chord name (may fail for unrecognizable sets — that's OK)
    log "--- Chord Name ---"
    $SLONIMSKY name "${pcs[@]}" 2>/dev/null | tee -a "$REPORT" || log "(no recognized chord name)"
    log ""

    # Interval matrix (text)
    log "--- Interval Matrix ---"
    $SLONIMSKY interval-matrix "${pcs[@]}" 2>/dev/null | tee -a "$REPORT"
    log ""

    # Interval vector (text with breakdown)
    log "--- Interval Vector ---"
    $SLONIMSKY interval-vector "${pcs[@]}" 2>/dev/null | tee -a "$REPORT"
    log ""

    # Subchords (size 3, named)
    if [ ${#pcs[@]} -ge 4 ]; then
        log "--- Subchords (size 3) ---"
        $SLONIMSKY subchords "${pcs[@]}" --size 3 --name 2>/dev/null | tee -a "$REPORT"
        log ""
    fi

    # SVG diagrams
    $SLONIMSKY pitch-circle "${pcs[@]}" \
        --title "$label" \
        -o "$OUT_DIR/analysis_${file_tag}_circle.svg" 2>/dev/null

    $SLONIMSKY interval-matrix "${pcs[@]}" \
        --title "$label" \
        -o "$OUT_DIR/analysis_${file_tag}_matrix.svg" 2>/dev/null

    $SLONIMSKY interval-vector "${pcs[@]}" \
        --title "$label" \
        -o "$OUT_DIR/analysis_${file_tag}_vector.svg" 2>/dev/null

    log "(SVG diagrams written: analysis_${file_tag}_{circle,matrix,vector}.svg)"
    log ""
}

log "╔══════════════════════════════════════════════════════════════════╗"
log "║           Set-Class Analysis Report — slonimsky CLI            ║"
log "╚══════════════════════════════════════════════════════════════════╝"
log ""
log "Generated: $(date -u '+%Y-%m-%d %H:%M UTC')"
log ""

# 1. Major triad — the most fundamental consonance
analyze_set "Major Triad (C)" "major_triad" C E G

# 2. Dominant 7th — tonal music's workhorse dissonance
analyze_set "Dominant 7th (C7)" "dom7" C E G Bb

# 3. Diminished 7th — maximal transpositional symmetry (order 4)
analyze_set "Diminished 7th" "dim7" C Eb Gb A

# 4. Augmented triad — 3-fold transpositional symmetry
analyze_set "Augmented Triad (C+)" "aug" C E Ab

# 5. Whole-tone hexachord — 6-fold symmetry, Forte 6-35
analyze_set "Whole-Tone Scale (6-35)" "wholetone" C D E Gb Ab Bb

# 6. Chromatic tetrachord — maximally compact cluster
analyze_set "Chromatic Tetrachord" "chrom4" C Db D Eb

log "================================================================"
log "  Summary"
log "================================================================"
log ""
log "Sets analyzed: 6"
log "SVG artifacts: 18 (3 per set: circle, matrix, vector)"
log "Text report:   $REPORT"
log ""

# Count and list SVG outputs
svg_count=$(find "$OUT_DIR" -name "analysis_*.svg" | wc -l)
log "Total SVG files produced: $svg_count"
log ""
log "Done."
