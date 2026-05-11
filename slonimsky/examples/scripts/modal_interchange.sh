#!/usr/bin/env bash
#
# Modal Interchange Explorer
#
# Explores modal interchange (mode mixture / borrowed chords) in a practical
# composition context. Starting from C major, this script:
#
#   1. Lists the diatonic triads of C major and C natural minor (Aeolian)
#   2. Finds common tones between the two parallel scales
#   3. Identifies the "borrowed" chords — triads from C minor that differ
#      from C major's diatonic set
#   4. Analyzes a classic pop/rock progression that uses a borrowed iv chord
#      (I - IV - iv - I, the "Creep" / "Space Oddity" pattern)
#   5. Compares voice-leading cost of the borrowed-chord progression vs.
#      a purely diatonic alternative (I - IV - vi - I)
#   6. Generates pitch-circle diagrams of borrowed chords for visual
#      comparison against their diatonic counterparts
#   7. Checks which scales contain each borrowed chord (containment query)
#
# This demonstrates how slonimsky's set-theoretic tools illuminate
# the harmonic vocabulary available through modal interchange.
#
# Usage:
#   cargo build -p slonimsky && ./slonimsky/examples/scripts/modal_interchange.sh
#   SLONIMSKY=/path/to/slonimsky bash slonimsky/examples/scripts/modal_interchange.sh
#
# Outputs:
#   slonimsky/examples/output/modal_interchange_report.txt
#   slonimsky/examples/output/mi_c_major_scale_circle.svg
#   slonimsky/examples/output/mi_c_minor_scale_circle.svg
#   slonimsky/examples/output/mi_borrowed_iv_circle.svg
#   slonimsky/examples/output/mi_borrowed_bVI_circle.svg
#   slonimsky/examples/output/mi_borrowed_bVII_circle.svg
#   slonimsky/examples/output/mi_borrowed_prog_analysis.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../output"
REPORT="$OUT_DIR/modal_interchange_report.txt"
mkdir -p "$OUT_DIR"

if [ -n "${SLONIMSKY:-}" ]; then
  S="$SLONIMSKY"
else
  S="$SCRIPT_DIR/../../../target/debug/slonimsky"
  if [ ! -x "$S" ]; then
    S="$SCRIPT_DIR/../../../target/release/slonimsky"
  fi
  if [ ! -x "$S" ]; then
    echo "Binary not found. Build first: cargo build -p slonimsky" >&2
    echo "Or set SLONIMSKY=/path/to/binary" >&2
    exit 1
  fi
fi

{
  echo "Modal Interchange Explorer"
  echo "=========================="
  echo ""
  echo "Key center: C"
  echo "Primary scale: C major (Ionian)"
  echo "Parallel scale: C natural minor (Aeolian)"
  echo ""

  # --- Section 1: Diatonic triads ---
  echo "1. DIATONIC TRIADS"
  echo "-------------------"
  echo ""
  echo "C Major diatonic triads:"
  # I=C,E,G  ii=D,F,A  iii=E,G,B  IV=F,A,C  V=G,B,D  vi=A,C,E  vii°=B,D,F
  echo "  I   = C E G   (C major)"
  echo "  ii  = D F A   (D minor)"
  echo "  iii = E G B   (E minor)"
  echo "  IV  = F A C   (F major)"
  echo "  V   = G B D   (G major)"
  echo "  vi  = A C E   (A minor)"
  echo "  vii°= B D F   (B diminished)"
  echo ""
  echo "C Natural Minor diatonic triads:"
  # i=C,Eb,G  ii°=D,F,Ab  III=Eb,G,Bb  iv=F,Ab,C  v=G,Bb,D  VI=Ab,C,Eb  VII=Bb,D,F
  echo "  i   = C Eb G  (C minor)"
  echo "  ii° = D F Ab  (D diminished)"
  echo "  III = Eb G Bb (Eb major)"
  echo "  iv  = F Ab C  (F minor)"
  echo "  v   = G Bb D  (G minor)"
  echo "  VI  = Ab C Eb (Ab major)"
  echo "  VII = Bb D F  (Bb major)"
  echo ""

  # --- Section 2: Scale common tones ---
  echo "2. COMMON TONES BETWEEN PARALLEL SCALES"
  echo "-----------------------------------------"
  echo ""
  $S common-tones C,D,E,F,G,A,B C,D,Eb,F,G,Ab,Bb
  echo ""

  # --- Section 3: Borrowed chords analysis ---
  echo "3. BORROWED CHORDS (from C minor, not in C major)"
  echo "---------------------------------------------------"
  echo ""
  echo "Chords unique to C minor (not found as diatonic triads in C major):"
  echo ""

  echo "  iv (F minor: F Ab C) — the classic borrowed chord:"
  $S common-tones F,A,C F,Ab,C
  echo ""

  echo "  bIII (Eb major: Eb G Bb):"
  $S common-tones E,G,B Eb,G,Bb
  echo ""

  echo "  bVI (Ab major: Ab C Eb):"
  $S common-tones A,C,E Ab,C,Eb
  echo ""

  echo "  bVII (Bb major: Bb D F):"
  $S common-tones B,D,F Bb,D,F
  echo ""

  # --- Section 4: The borrowed iv progression ---
  echo "4. PROGRESSION ANALYSIS: I - IV - iv - I (borrowed iv)"
  echo "-------------------------------------------------------"
  echo ""
  echo "This progression appears in 'Creep' (Radiohead), 'Space Oddity'"
  echo "(Bowie), and countless other songs. The iv chord borrows Ab from"
  echo "the parallel minor, creating a chromatic descent: A→Ab→G."
  echo ""
  echo "Analysis:"
  $S analyze C,E,G F,A,C F,Ab,C C,E,G --key C
  echo ""

  echo "Voice-leading path (greedy):"
  $S progression C,E,G F,A,C F,Ab,C C,E,G
  echo ""

  # --- Section 5: Comparison with diatonic alternative ---
  echo "5. COMPARISON: borrowed iv vs. diatonic vi"
  echo "--------------------------------------------"
  echo ""
  echo "Diatonic alternative: I - IV - vi - I"
  echo ""
  echo "Analysis:"
  $S analyze C,E,G F,A,C A,C,E C,E,G --key C
  echo ""

  echo "Voice-leading path (greedy):"
  $S progression C,E,G F,A,C A,C,E C,E,G
  echo ""

  echo "The borrowed iv creates a stronger pull back to I because the"
  echo "chromatic motion Ab→G (a half step) has greater directional"
  echo "gravity than the diatonic vi→I motion."
  echo ""

  # --- Section 6: Scale containment of borrowed chords ---
  echo "6. SCALE CONTAINMENT OF BORROWED CHORDS"
  echo "-----------------------------------------"
  echo ""
  echo "Which scales contain the borrowed iv (F Ab C)?"
  $S contains F,Ab,C --limit 8
  echo ""

  echo "Which scales contain the borrowed bVI (Ab C Eb)?"
  $S contains Ab,C,Eb --limit 8
  echo ""

  echo "Which scales contain the borrowed bVII (Bb D F)?"
  $S contains Bb,D,F --limit 8
  echo ""

  # --- Section 7: Extended borrowed-chord progression ---
  echo "7. EXTENDED PROGRESSION: I - bVI - bVII - I (Aeolian cadence)"
  echo "---------------------------------------------------------------"
  echo ""
  echo "The bVI-bVII-I cadence is a hallmark of rock and film music."
  echo ""
  echo "Analysis:"
  $S analyze C,E,G Ab,C,Eb Bb,D,F C,E,G --key C
  echo ""

  echo "Voice-leading path:"
  $S progression C,E,G Ab,C,Eb Bb,D,F C,E,G
  echo ""

  echo "Common tones through the chain:"
  $S common-tones C,E,G Ab,C,Eb
  $S common-tones Ab,C,Eb Bb,D,F
  $S common-tones Bb,D,F C,E,G
  echo ""

  echo "=== End of Modal Interchange Report ==="

} > "$REPORT" 2>&1

echo "Report written: $REPORT"

# --- SVG diagrams ---

# C major scale circle
$S pitch-circle C D E F G A B \
  --title "C Major Scale" --show-intervals \
  -o "$OUT_DIR/mi_c_major_scale_circle.svg"
echo "SVG: mi_c_major_scale_circle.svg"

# C natural minor scale circle
$S pitch-circle C D Eb F G Ab Bb \
  --title "C Natural Minor Scale" --show-intervals \
  -o "$OUT_DIR/mi_c_minor_scale_circle.svg"
echo "SVG: mi_c_minor_scale_circle.svg"

# Borrowed iv chord (F minor)
$S pitch-circle F Ab C \
  --title "Borrowed iv: F minor" \
  -o "$OUT_DIR/mi_borrowed_iv_circle.svg"
echo "SVG: mi_borrowed_iv_circle.svg"

# Borrowed bVI chord (Ab major)
$S pitch-circle Ab C Eb \
  --title "Borrowed bVI: Ab major" \
  -o "$OUT_DIR/mi_borrowed_bVI_circle.svg"
echo "SVG: mi_borrowed_bVI_circle.svg"

# Borrowed bVII chord (Bb major)
$S pitch-circle Bb D F \
  --title "Borrowed bVII: Bb major" \
  -o "$OUT_DIR/mi_borrowed_bVII_circle.svg"
echo "SVG: mi_borrowed_bVII_circle.svg"

# JSON analysis of the borrowed-chord progression
$S analyze C,E,G F,A,C F,Ab,C C,E,G --key C --format json \
  -o "$OUT_DIR/mi_borrowed_prog_analysis.json"
echo "JSON: mi_borrowed_prog_analysis.json"

echo ""
echo "All outputs written to $OUT_DIR/mi_*"
