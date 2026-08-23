#import "sheet.typ": sheet, blk, pair

#sheet(
  date: "2026-08-08 — example B",
  sub: [Week 2 · Day 4 (fingerstyle, 5/4) · C whole-tone · 30 min #h(6pt)|#h(6pt) core: whole-tone → 2-per-string → hammer-ons → C+ · D+],
)[
#blk("1", "Warmup", "4 min", [♩=56],
  [*p-i-m-a* on one shape, 5/4. Thumb on ⑤, i-m-a on ④③②. Follow the hairpins.],
  image("w1.cropped.png", width: 100%),
  [thumb stays planted; fingers pull from the knuckle, not the tip.])

#blk("2", "Hands + Vocabulary", "7 min", [♩=60],
  [C whole-tone, *2 notes per string*, ascending then descending. Slurs are hammer-ons going up, pull-offs coming down.],
  image("w2a.cropped.png", width: 100%),
  [the 2-fret gaps are a stretch, not a grab. Let the thumb slide behind the neck.])

#blk("3", "Ear → Hand", "6 min", [♩=60],
  [*Sing first,* then play. 5/4 — count 3+2. The rest in bar 1 lands on beat 3.],
  image("w3.cropped.png", width: 100%),
  [feel the 3+2 grouping before the first note.])

#blk("4", "Harmony + Groove", "7 min", [♩=66],
  [*One grip, three keys.* C+ (C E A♭) is the altered V in F, A *and* D♭ — a major 3rd apart. The ♯5 shape never moves; everything around it does.],
  image("w4.cropped.png", width: 100%),
  [same fingers, three functions. hear the resolution change under a fixed grip.])

#blk("5", "Improv on a Model", "6 min", [♩=66],
  [Over C+ · D+. Model ×2, variation ×2, then continue.],
  pair(
    [model — play ×2], image("w5a.cropped.png", width: 100%),
    [variation of bars 1–2 — then ×2], image("w5b.cropped.png", width: 100%),
    tail: [then: keep going — 5/4, one rest per bar minimum, your notes.],
  ),
  [whole-tone has no home note. Land somewhere anyway, on purpose.])
]
