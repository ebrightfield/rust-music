#!/usr/bin/env bash
# Ear-training demo: generate interval-identification MIDI quizzes
# and JSON answer keys for practice / study use.
#
# Produces:
#   - ear_training_beginner.mid   — 5-item quiz (seed=100, reproducible)
#   - ear_training_beginner.json  — JSON answer key for the beginner quiz
#   - ear_training_advanced.mid   — 15-item quiz (seed=200, reproducible)
#   - ear_training_advanced.txt   — text answer key (captured from stdout)
#
# Requires: cargo build -p slonimsky --features midi

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
mkdir -p "$OUT_DIR"

# Locate binary (consistent with other scripts: SLONIMSKY env var or auto-detect)
if [ -n "${SLONIMSKY:-}" ]; then
    BIN="$SLONIMSKY"
else
    BIN="$SCRIPT_DIR/../../../target/debug/slonimsky"
    if [ ! -x "$BIN" ]; then
        BIN="$SCRIPT_DIR/../../../target/release/slonimsky"
    fi
    if [ ! -x "$BIN" ]; then
        echo "Binary not found. Build first: cargo build -p slonimsky --features midi" >&2
        echo "Or set SLONIMSKY=/path/to/binary" >&2
        exit 1
    fi
fi

# Verify ear-training subcommand is available (requires --features midi)
if ! "$BIN" ear-training --help &>/dev/null; then
    echo "ear-training subcommand not available. Rebuild with: cargo build -p slonimsky --features midi" >&2
    exit 0
fi

echo "=== Ear Training Demo ==="
echo ""

# --- Beginner quiz: 5 items, seed 100 ---
echo "1) Generating beginner quiz (5 intervals, seed=100)..."
"$BIN" ear-training --type intervals --count 5 --seed 100 \
    -o "$OUT_DIR/ear_training_beginner.mid" -v \
    > "$OUT_DIR/ear_training_beginner_answers.txt"
echo "   MIDI: $OUT_DIR/ear_training_beginner.mid"
echo "   Answers: $OUT_DIR/ear_training_beginner_answers.txt"
echo ""

# --- Beginner quiz: JSON answer key ---
echo "2) Generating JSON answer key for beginner quiz (same seed)..."
"$BIN" ear-training --type intervals --count 5 --seed 100 \
    -o "$OUT_DIR/ear_training_beginner.json"
echo "   JSON: $OUT_DIR/ear_training_beginner.json"
echo ""

# --- Advanced quiz: 15 items, seed 200 ---
echo "3) Generating advanced quiz (15 intervals, seed=200)..."
"$BIN" ear-training --type intervals --count 15 --seed 200 \
    -o "$OUT_DIR/ear_training_advanced.mid" -v \
    > "$OUT_DIR/ear_training_advanced_answers.txt"
echo "   MIDI: $OUT_DIR/ear_training_advanced.mid"
echo "   Answers: $OUT_DIR/ear_training_advanced_answers.txt"
echo ""

# --- Validation ---
echo "=== Validation ==="
PASS=0
FAIL=0

for f in ear_training_beginner.mid ear_training_beginner.json \
         ear_training_beginner_answers.txt \
         ear_training_advanced.mid ear_training_advanced_answers.txt; do
    path="$OUT_DIR/$f"
    if [ -s "$path" ]; then
        size=$(wc -c < "$path")
        echo "  ✓ $f ($size bytes)"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $f MISSING or EMPTY"
        FAIL=$((FAIL + 1))
    fi
done

# Check MIDI magic bytes (MThd = 0x4d546864)
for midi in ear_training_beginner.mid ear_training_advanced.mid; do
    header=$(head -c 4 "$OUT_DIR/$midi")
    if [ "$header" = "MThd" ]; then
        echo "  ✓ $midi has valid MThd header"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $midi bad header"
        FAIL=$((FAIL + 1))
    fi
done

# Check JSON validity
if command -v python3 &>/dev/null; then
    if python3 -c "import json; json.load(open('$OUT_DIR/ear_training_beginner.json'))" 2>/dev/null; then
        items=$(python3 -c "import json; print(len(json.load(open('$OUT_DIR/ear_training_beginner.json'))))")
        echo "  ✓ beginner JSON is valid ($items items)"
        PASS=$((PASS + 1))
    else
        echo "  ✗ beginner JSON is invalid"
        FAIL=$((FAIL + 1))
    fi
fi

# Check text answer keys contain interval names
for txt in ear_training_beginner_answers.txt ear_training_advanced_answers.txt; do
    if grep -q "Interval Identification" "$OUT_DIR/$txt"; then
        echo "  ✓ $txt has correct header"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $txt missing header"
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && echo "All checks passed!" || exit 1
