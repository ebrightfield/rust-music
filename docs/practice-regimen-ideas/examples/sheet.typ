// Reusable daily-practice sheet template.
//
// Usage from a per-day file:
//   #import "sheet.typ": sheet, blk
//   #sheet(date: "2026-08-01", sub: "Week 1 · Day 1 …")[
//     #blk("1", "Warmup", "4 min", [♩=63], [what to play], image("b1.png"), [intention])
//     ...
//   ]
//
// Design rules this template enforces (see ../exercise-suite.md §1, §5):
//  - one italic "what to play" line per block, one "intention" line, nothing else
//  - per-block done/hard checkboxes (drives the weekly `hard` carry-over)
//  - a 3-field log strip, short enough to actually fill in

// `breakable` defaults to false so a block stays visually intact. Pass breakable: true for
// tall blocks (many bars of notation) — otherwise the block jumps wholesale to the next page
// and leaves half a page blank, which reads as padding rather than as a deliberate page break.
#let blk(n, name, mins, tempo, what, body, intent, breakable: false) = block(
  width: 100%, breakable: breakable, inset: 6pt, radius: 3pt,
  stroke: 0.6pt + luma(60%),
)[
  #grid(columns: (1fr, auto),
    text(weight: "bold", size: 10pt)[#n · #upper(name)],
    text(size: 9pt, fill: luma(30%))[#mins · #tempo],
  )
  #v(-2pt)
  #text(size: 9pt, style: "italic")[#what]
  #v(1pt)
  #body
  #v(-1pt)
  #grid(columns: (1fr, auto),
    text(size: 8.5pt, fill: luma(35%))[intention: #intent],
    text(size: 8.5pt)[☐ done  ☐ hard],
  )
]

// Two stacked, labelled notation images — for "notate both, play both" (a-b-a)
// and for model/variation pairs.
#let pair(label_a, img_a, label_b, img_b, tail: none) = [
  #text(size: 8.5pt, weight: "bold")[#label_a]
  #img_a
  #v(1pt)
  #text(size: 8.5pt, weight: "bold")[#label_b]
  #img_b
  #if tail != none [ #v(1pt) #text(size: 9pt, style: "italic")[#tail] ]
]

// Font fallback list. Liberation Sans lacks ♭ (U+266D), △ (U+25B3), ⊓/⊔ (U+2293/4) — those
// glyphs render as BLANK with no warning, silently dropping accidentals from chord/scale names.
// Noto Sans Symbols covers them; Typst falls through the list per-glyph.
#let BODY_FONT = ("Liberation Sans", "Noto Sans Symbols", "Noto Music")

#let sheet(date: "", sub: "", body) = [
  #set page(paper: "us-letter", margin: (x: 1.3cm, y: 1.1cm))
  #set text(font: BODY_FONT, size: 9.5pt)
  #align(center)[
    #text(size: 14pt, weight: "bold")[Daily Practice — #date]
    #v(-4pt)
    #text(size: 9pt, fill: luma(30%))[#sub]
  ]
  #v(3pt)
  #body
  #v(4pt)
  #block(width: 100%, inset: 6pt,
    stroke: (dash: "dashed", paint: luma(55%), thickness: 0.6pt), radius: 3pt)[
    #text(size: 9pt, weight: "bold")[Log] #h(6pt)
    #text(size: 9pt)[
      felt free: #box(width: 4.6cm, repeat[.]) #h(3pt)
      felt stuck: #box(width: 4.6cm, repeat[.]) #h(3pt)
      keep: #box(width: 3.4cm, repeat[.])
    ]
  ]
]
