#import "sheet.typ": sheet, blk, pair

#let elements = block(width: 100%, inset: 5pt, radius: 3pt, fill: luma(96%))[
  #set text(size: 8pt)
  #grid(columns: (1fr, 1fr, 1fr), gutter: 4pt,
    [*key* C · *position* V–X · *set* Lydian ♭9♭13 \
     *pattern* `1,[4,-2,3]` (wide leaps)],
    [*rhythm* sextuplets; 5/8+7/8 · *artic.* slides + ghost notes \
     *RH* string skipping, economy],
    [*voicing* C△add♯11 on ④③②① · *v-lead* chromatic pedal C→C♯→C \
     *dyn.* terraced *mp*→*f*],
  )
  #v(1pt)
  #text(size: 8pt)[
    *chain* C Lyd♭9♭13 (7) ⊃ C△add♯11 (4) ⊃ C△♭5 (3) ⊃ ⟨C,G♭⟩ (♯11) #h(4pt)
    #text(fill: luma(40%))[· each block plays one rung of this chain]
  ]
  #v(-1pt)
  #text(size: 7.5pt, style: "italic", fill: luma(35%))[
    same set, position, and voicing throughout — one decision, propagated.
  ]
]

#sheet(
  date: "2026-08-22 — example D (harder / wider)",
  sub: [Week 6 · Day 3 · flatpick · 30 min],
)[
#elements
#v(3pt)

#blk("1", "Warmup", "5 min", [♩=72],
  [4-per-string, *all six strings*: pos V ascending legato (1-2-3-4) → pos VIII descending staccato (4-3-2-1). Then reverse.],
  image("h1.cropped.png", width: 100%),
  [loose. no squeeze — check your thumb every 4 bars.])

#blk("2", "Hands + Vocabulary", "8 min", [♩=76],
  [C Lyd♭9♭13, tuple *`1,[4,-2,3]`* — 5ths/6ths, not steps. All 7 degrees up, negated tuple down, then the same tuple in *A Lyd♭9♭13*. All written out.],
  [
    #text(size: 8.5pt, weight: "bold")[a) up — all 7 degrees, C Lyd♭9♭13]
    #image("h2a.cropped.png", width: 100%)
    #v(1pt)
    #text(size: 8.5pt, weight: "bold")[b) down — tuple negated `1,[-4,2,-3]`]
    #image("h2b.cropped.png", width: 100%)
    #v(1pt)
    #text(size: 8.5pt, weight: "bold")[c) same tuple, down a m3 — A Lyd♭9♭13]
    #image("h2c.cropped.png", width: 100%)
  ],
  [leaps, not scales. let the hand fall, don't place it.],
  breakable: true)

#blk("3", "Ear → Hand", "4 min", [♩=76],
  [*Sing first,* then play. *5/8 + 7/8* alternating. Chromatic — sing intervals, not scale degrees.],
  image("h3.cropped.png", width: 100%),
  [hear it before you play it. breathe on the barline.])

#blk("4", "Harmony + Voice-leading", "7 min", [♩=80],
  [*C△add♯11* on the top four strings (④③②①) — all three chords ⊂ C Lyd♭9♭13. Top voice is a chromatic pedal C→C♯→C.],
  image("h4.cropped.png", width: 100%),
  [inner voices still. hear only the top move.])

#blk("5", "Improv on a Model", "6 min", [♩=80],
  [Over the voicings above. Model ×2, variation ×2, then continue. Parenthesized note = ghost, strike it dead.],
  pair(
    [model — play ×2], image("h5a.cropped.png", width: 100%),
    [variation of bars 1–2 — then ×2], image("h5b.cropped.png", width: 100%),
    tail: [then: keep going — same set, ♯11 and ♭9 on strong beats, one rest per bar.],
  ),
  [♭9/♯11 = the colour. land on purpose. yes-and.])
]
