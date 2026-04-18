#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

OUT_DIR="target/pitch_set_svgs"
PDF_PATH="$OUT_DIR/all_shapes.pdf"
TMP_DIR="$OUT_DIR/_pdf_pages"

# 1. Generate the SVGs (per-shape tiles + per-page A4 SVGs + combined).
cargo run --example pitch_set_svgs "$@"

# 2. Check for the tools needed to produce a PDF.
missing=()
command -v rsvg-convert >/dev/null 2>&1 || missing+=("rsvg-convert (pkg: librsvg)")
command -v pdfunite    >/dev/null 2>&1 || missing+=("pdfunite (pkg: poppler)")

if [ ${#missing[@]} -gt 0 ]; then
    echo
    echo "Skipping PDF generation. Missing tools:"
    for m in "${missing[@]}"; do
        echo "  - $m"
    done
    echo "SVGs are still available under $OUT_DIR/."
    exit 0
fi

# 3. Convert each page_NN.svg to a single-page PDF. rsvg-convert does not
#    paginate or scale across pages; each input SVG is already sized A4, so
#    the output PDF page is one-to-one.
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR"

shopt -s nullglob
pages=("$OUT_DIR"/page_*.svg)
if [ ${#pages[@]} -eq 0 ]; then
    echo "No page_*.svg files found under $OUT_DIR; nothing to convert."
    exit 1
fi

echo
echo "Converting ${#pages[@]} pages to PDF..."
for svg in "${pages[@]}"; do
    base=$(basename "$svg" .svg)
    rsvg-convert \
        --format=pdf \
        --keep-aspect-ratio \
        --output "$TMP_DIR/$base.pdf" \
        "$svg"
done

# 4. Merge into one PDF.
pdfunite "$TMP_DIR"/page_*.pdf "$PDF_PATH"
rm -rf "$TMP_DIR"

echo "Wrote $PDF_PATH"
