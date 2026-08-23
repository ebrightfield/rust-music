#import "sheet.typ": sheet, blk, pair

#sheet(
  date: "2026-08-15 — example C",
  sub: [Week 3 · Day 2 (flatpick, *swing 8ths*) · G · 30 min #h(6pt)|#h(6pt) core: G major ⊃ G pent ⊃ G6 → bends → G6·Em7·A7·D7],
)[
#blk("1", "Warmup", "4 min", [♩=58 swing],
  [*Climbing 1–4* on ③: the 1st finger walks up while the 4th holds the top. Swing the 8ths. *mp*.],
  image("g1.cropped.png", width: 100%),
  [the stretch closes as you climb — let it, don't fight it.])

#blk("2", "Hands + Vocabulary", "7 min", [♩=63 swing],
  [G pentatonic (⊂ G major). *Bends written in:* "full" = whole step, "½" = half. *Play a, then b, then a.*],
  pair(
    [a) lower box — frets 5–9], image("g2a.cropped.png", width: 100%),
    [b) upper box — frets 7–10], image("g2b.cropped.png", width: 100%),
  ),
  [push the bend from the wrist; check it against the target pitch.])

#blk("3", "Ear → Hand", "6 min", [♩=63 swing],
  [*Sing first,* then play. G pentatonic only — no F\#, no C.],
  image("g3.cropped.png", width: 100%),
  [the missing 4th and 7th are what makes it sound open. Hear that.])

#blk("4", "Harmony + Groove", "7 min", [♩=72 swing],
  [G6 · Em7 · A7 · D7 — drop-2, frets 4–9. *Anticipate:* chords land on the & of 4 and tie over. Fm7 is a chromatic approach into A7.],
  image("g4.cropped.png", width: 100%),
  [push the chord ahead of the beat. comp for someone else — leave them room.])

#blk("5", "Improv on a Model", "6 min", [♩=72 swing],
  [Over the changes above. Model ×2, variation ×2, then continue.],
  pair(
    [model — play ×2], image("g5a.cropped.png", width: 100%),
    [variation of bars 1–2 — then ×2], image("g5b.cropped.png", width: 100%),
    tail: [then: keep going — G pentatonic, one bend per phrase, your notes.],
  ),
  [a bend is a note with an approach. Arrive in tune.])
]
