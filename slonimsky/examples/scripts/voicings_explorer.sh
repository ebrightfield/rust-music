#!/usr/bin/env bash
#
# Voicings Explorer — canonical voicing enumeration for jazz/arranging study
#
# Demonstrates how to use the `voicings` subcommand to explore all canonical
# voicings of common chord types (triads and seventh chords), combining the
# output with `forte` and `orbits` for set-theoretic context. Useful for
# arrangers studying drop-voicing families and keyboard/guitar players
# exploring inversion options.
#
# Produces:
#   voicings_major_triad.txt       — all 6 voicings of C major triad
#   voicings_minor_triad.txt       — all 6 voicings of C minor triad
#   voicings_dom7.txt              — all 24 voicings of C7
#   voicings_maj7.txt              — all 24 voicings of Cmaj7
#   voicings_min7.txt              — all 24 voicings of Cm7 (0 3 7 10)
#   voicings_dim7.txt              — all voicings of Cdim7 (0 3 6 9)
#   voicings_comparison_report.txt — summary comparing all chord types
#
# Usage:
#   ./voicings_explorer.sh
#   SLONIMSKY=/path/to/slonimsky ./voicings_explorer.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

# Locate binary
if [ -n "${SLONIMSKY:-}" ]; then
    BIN="$SLONIMSKY"
elif [ -f "$SCRIPT_DIR/../../../target/release/slonimsky" ]; then
    BIN="$SCRIPT_DIR/../../../target/release/slonimsky"
elif [ -f "$SCRIPT_DIR/../../../target/debug/slonimsky" ]; then
    BIN="$SCRIPT_DIR/../../../target/debug/slonimsky"
else
    echo "Building slonimsky..."
    (cd "$SCRIPT_DIR/../../.." && cargo build -p slonimsky)
    BIN="$SCRIPT_DIR/../../../target/debug/slonimsky"
fi

echo "Using binary: $BIN"
echo "Output dir:   $OUT_DIR"
echo

# ── Helper: generate voicing report for one chord ──────────────────────
generate_voicing_report() {
    local label="$1"
    local outfile="$2"
    shift 2
    local args=("$@")

    {
        echo "═══════════════════════════════════════════════════"
        echo "  VOICINGS: $label"
        echo "═══════════════════════════════════════════════════"
        echo
        echo "── All canonical voicings (verbose) ──"
        echo
        $BIN voicings "${args[@]}" -v
        echo
        echo "── Set-theoretic context ──"
        echo
        $BIN forte "${args[@]}"
        echo
        $BIN orbits "${args[@]}"
    } > "$outfile"

    local count
    count=$(grep -c '^\s\+[0-9]\+\.' "$outfile" || echo 0)
    echo "  $label → $outfile ($count voicings)"
}

# ── Generate voicing reports ───────────────────────────────────────────

echo "Generating voicing reports..."
echo

# Triads
generate_voicing_report "C major triad (C E G)" \
    "$OUT_DIR/voicings_major_triad.txt" \
    C E G

generate_voicing_report "C minor triad (C Eb G)" \
    "$OUT_DIR/voicings_minor_triad.txt" \
    0 3 7

# Seventh chords
generate_voicing_report "C dominant 7 (C E G Bb)" \
    "$OUT_DIR/voicings_dom7.txt" \
    0 4 7 10

generate_voicing_report "C major 7 (C E G B)" \
    "$OUT_DIR/voicings_maj7.txt" \
    C E G B

generate_voicing_report "C minor 7 (C Eb G Bb)" \
    "$OUT_DIR/voicings_min7.txt" \
    0 3 7 10

generate_voicing_report "C diminished 7 (C Eb Gb A)" \
    "$OUT_DIR/voicings_dim7.txt" \
    0 3 6 9

# ── Comparison report ──────────────────────────────────────────────────

echo
echo "Generating comparison report..."

REPORT="$OUT_DIR/voicings_comparison_report.txt"
{
    echo "═══════════════════════════════════════════════════════════"
    echo "  VOICING COMPARISON REPORT"
    echo "  Canonical voicings of common jazz chord types"
    echo "═══════════════════════════════════════════════════════════"
    echo
    echo "Chord types compared:"
    echo "  1. Major triad     (C E G)       — PcSet {0, 4, 7}"
    echo "  2. Minor triad     (C Eb G)      — PcSet {0, 3, 7}"
    echo "  3. Dominant 7      (C E G Bb)    — PcSet {0, 4, 7, 10}"
    echo "  4. Major 7         (C E G B)     — PcSet {0, 4, 7, 11}"
    echo "  5. Minor 7         (C Eb G Bb)   — PcSet {0, 3, 7, 10}"
    echo "  6. Diminished 7    (C Eb Gb A)   — PcSet {0, 3, 6, 9}"
    echo
    echo "─── Triads: close vs open position ───────────────────────"
    echo
    echo "Major triad — close position (Family 1):"
    $BIN voicings C E G --limit 3
    echo
    echo "Minor triad — close position (Family 1):"
    $BIN voicings 0 3 7 --limit 3
    echo
    echo "─── Seventh chords: first family (close) ─────────────────"
    echo
    echo "Dominant 7 — close position:"
    $BIN voicings 0 4 7 10 --limit 4
    echo
    echo "Major 7 — close position:"
    $BIN voicings C E G B --limit 4
    echo
    echo "Minor 7 — close position:"
    $BIN voicings 0 3 7 10 --limit 4
    echo
    echo "Diminished 7 — close position:"
    $BIN voicings 0 3 6 9 --limit 4
    echo
    echo "─── Symmetry comparison ──────────────────────────────────"
    echo
    echo "Dim7 (0 3 6 9) — highly symmetric:"
    $BIN orbits 0 3 6 9
    echo
    echo "Dom7 (0 4 7 10) — no transpositional symmetry:"
    $BIN orbits 0 4 7 10
    echo
    echo "─── Forte classification ─────────────────────────────────"
    echo
    for pcs in "C E G" "0 3 7" "0 4 7 10" "C E G B" "0 3 7 10" "0 3 6 9"; do
        # shellcheck disable=SC2086
        $BIN forte $pcs
        echo
    done
    echo "═══════════════════════════════════════════════════════════"
    echo "  End of comparison report"
    echo "═══════════════════════════════════════════════════════════"
} > "$REPORT"

echo "  Comparison → $REPORT"
echo
echo "Done! All output files:"
ls -la "$OUT_DIR"/voicings_*.txt
