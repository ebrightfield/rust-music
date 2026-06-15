# Image Preprocessing for Sheet Music Transcription

Claude Code reads images directly via the Read tool, but reliability degrades on very large, very small, or low-contrast scans. ImageMagick (`magick` command, available at `/usr/bin/magick`) bridges the gap.

**First principle**: try reading the raw image before preprocessing. Only preprocess if you can't reliably identify noteheads, accidentals, or clef.

## Decision table

| Symptom in the raw image | Recipe |
|--------------------------|--------|
| File > ~5 MB or longest side > 3000 px | **Downscale** to ~2000 px longest side |
| Pale / faded scan, low contrast | **Normalize** (auto-stretch histogram) |
| Slight blur, fuzzy noteheads | **Unsharp mask**, light |
| Bleed-through from reverse page | **Threshold** to pure black/white |
| Color scan (sepia, blue paper) | **Greyscale** then normalize |
| Multi-page PDF | **Per-page extract** at 300 DPI |
| Very wide single image with many systems | **Crop into horizontal strips**, one system per file |
| Rotated / skewed | **Deskew** |

These compose: a typical "clean it up" pass is `-resize -normalize -unsharp`.

## Common recipes

All recipes write to `/tmp/` to keep the working tree clean.

### Standard cleanup (most common)

```bash
magick input.png \
  -resize 'x2000>' \
  -normalize \
  -unsharp 0x1 \
  /tmp/score-clean.png
```

- `x2000>` resizes to 2000px tall, only if larger (`>`); preserves aspect ratio.
- `-normalize` stretches the histogram so the darkest pixel becomes black and lightest becomes white.
- `-unsharp 0x1` is a light sharpen that helps thin staff lines and ledger lines without ringing.

### PDF → page PNGs

```bash
magick -density 300 score.pdf /tmp/score-page-%02d.png
```

`-density` is set **before** the input file. 300 DPI is the sweet spot for music; 150 is too coarse for accidentals, 600 wastes memory.

### Greyscale + threshold (eliminate bleed-through)

```bash
magick input.png \
  -colorspace Gray \
  -normalize \
  -threshold 60% \
  /tmp/score-bw.png
```

Tune `-threshold` between 50% and 70%. Higher values keep more detail (and more noise); lower values are cleaner but may drop thin lines.

### Crop into systems

For a single image containing many systems (staff lines + the music on them), splitting by horizontal strip often improves transcription because Claude can focus on one band at a time:

```bash
# First, find the image height
identify -format '%h\n' input.png
# Then crop into strips; pick a strip height that contains roughly one system + margin
magick input.png -crop x400 +repage /tmp/system-%02d.png
```

`+repage` is critical — without it the cropped pieces retain the original canvas offset and tools may misalign.

### Deskew

```bash
magick input.png -deskew 40% /tmp/score-straight.png
```

The percentage is the threshold for detection; 40% works for most scans. Apply before any cropping.

### Crop to ROI

If only one measure or system matters, crop tightly:

```bash
# magick input.png -crop WxH+X+Y +repage output.png
magick input.png -crop 1200x400+200+1500 +repage /tmp/measure-roi.png
```

Use `identify input.png` to find dimensions first.

## Sizing rules of thumb

| Element to read | Min on-image size |
|-----------------|-------------------|
| Notehead | ~12 px diameter |
| Accidental (sharp/flat) | ~20 px tall |
| Time signature digit | ~25 px tall |
| Tempo marking text | ~15 px tall |

If a downscale would drop a critical element below these thresholds, **don't downscale that far**; crop instead.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `magick: unable to open module` for PDF | Install Ghostscript (`pacman -S ghostscript`) |
| Output file is huge | Add `-quality 85` for JPG, or `-depth 8` |
| Staff lines disappear after threshold | Threshold is too low; try 65–70% |
| Noteheads merged after sharpening | Reduce unsharp radius (`-unsharp 0x0.5`) |
| Cropped pieces have weird positions | Forgot `+repage` after `-crop` |

## When to stop preprocessing and ask the user

If after a standard cleanup pass you still can't reliably tell:

- Whether a notehead is on a line or a space
- Sharp vs natural vs courtesy accidental
- Quarter vs eighth (filled head ambiguity)
- Which clef is in use

…then the source is genuinely too poor. Tell the user what's unreadable and ask for a higher-resolution scan, the original PDF, or a different page. **Don't guess.**
