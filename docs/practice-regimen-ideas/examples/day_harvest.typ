#import "sheet.typ": sheet, blk, pair

#let elements = block(width: 100%, inset: 5pt, radius: 3pt, fill: luma(96%))[
  #set text(size: 8pt)
  #grid(columns: (1fr, 1fr, 1fr), gutter: 4pt,
    [*key* D · *position* open–VII · *set* D Lydian \
     *pattern* `1,[3,-1]`],
    [*rhythm* 3/4, 8ths · *artic.* let-ring, sustained voices \
     *RH* fingerstyle p-i-m-a],
    [*voicing* open-string wide spacings · *v-lead* pedal opens \
     *dyn.* *ppp*→*mp*],
  )
  #v(1pt)
  #text(size: 8pt)[
    *chain* D Lyd (7) ⊃ D△add9 (4) ⊃ ⟨D,G♯⟩ (♯11) #h(4pt)
    #text(fill: luma(40%))[· 5 of 6 open strings are diatonic to D Lydian — that's why this key]
  ]
  #v(-1pt)
  #text(size: 7.5pt, style: "italic", fill: luma(35%))[
    harvest day: Towner (open-string voicings) + Lage (held voice, quiet practice).
  ]
]

#sheet(
  date: "2026-08-29 — example E (harvest day)",
  sub: [Week 7 · Day 1 · *fingerstyle* · 30 min],
)[
#elements
#v(3pt)

#blk("1", "Warmup", "5 min", [♩=60],
  [*Reset chord first* — one voicing, *ppp*, let it die away. Then 3-per-string across *all six*, fingers 1-2-4: up legato, down staccato.],
  image("e1.cropped.png", width: 100%),
  [quietly. let the guitar do the work. — Lage],
  breakable: true)

#blk("2", "Hands + Vocabulary", "6 min", [♩=66],
  [D Lydian, tuple *`1,[3,-1]`*, all 7 degrees. Fingerstyle: *p-i-m-a*, one finger per string, no repeats.],
  image("e2.cropped.png", width: 100%),
  [the G♯ is the whole point of Lydian. don't soften it.])

#blk("3", "Ear → Hand", "5 min", [♩=66],
  [*Sing first,* then play. The ♯4 (G♯) is the ear target — sing it against an open D drone.],
  image("e3.cropped.png", width: 100%),
  [hear the ♯4 as bright, not wrong.])

#blk("4", "Harmony + Voicings", "7 min", [♩=60],
  [*Towner:* open strings woven into fretted voicings — wide spacing, sparse low register. Arpeggiate each, let everything ring.],
  image("e4.cropped.png", width: 100%),
  [open strings are colour, not convenience. listen to what rings on.])

#blk("5", "Two Voices", "7 min", [♩=60],
  [*Lage:* hold the top voice while the lower line walks. One guitar, two apparent parts — the sustained note must not die.],
  image("e5.cropped.png", width: 100%),
  [thumb keeps time, top voice keeps singing. two people.],
  breakable: true)
]
