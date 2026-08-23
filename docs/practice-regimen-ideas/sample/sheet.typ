#set page(paper: "us-letter", margin: (x: 1.3cm, y: 1.1cm))
#set text(font: "Liberation Sans", size: 9.5pt)

#let block_(n, name, mins, tempo, what, body, intent) = block(
  width: 100%, breakable: false, inset: 6pt, radius: 3pt,
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

#align(center)[
  #text(size: 14pt, weight: "bold")[Daily Practice — 2026-08-01]
  #v(-4pt)
  #text(size: 9pt, fill: luma(30%))[
    Week 1 · Day 1 (position V, straight 8ths, flatpick) · Bb · 30 min
    #h(6pt)|#h(6pt) core: Bb major → 1,\[2\] → slides → Cm7·F7·Bb△
  ]
]
#v(3pt)

#block_("1", "Warmup", "4 min", [♩=63],
  [Perm *1-3-2-4*, 5th position, one string per bar. Bar-half 1 slurred, bar-half 2 staccato. *mp*.],
  image("b1.cropped.png", width: 100%),
  [relaxed hand; the staccato half is shorter, not harder.])

#block_("2", "Hands + Vocabulary", "7 min", [♩=72],
  [Bb major, pattern *1,\[2\]* (up 1 degree, take 3rds). *Play a, then b, then a* — same notes, two boxes.],
  [
    #text(size: 8.5pt, weight: "bold")[a) position V]
    #image("b2a.cropped.png", width: 100%)
    #v(1pt)
    #text(size: 8.5pt, weight: "bold")[b) position VII]
    #image("b2b.cropped.png", width: 100%)
  ],
  [both boxes, same notes. Let the hand feel the difference.])

#block_("3", "Ear → Hand", "6 min", [♩=66],
  [*Sing it first* (solfège or numbers), then play it. Two bars, Bb. Find it without hunting.],
  image("b3.cropped.png", width: 100%),
  [sing the whole phrase before you touch the string.])

#block_("4", "Harmony + Groove", "7 min", [♩=76],
  [Cm7 · F7 · Bb△ — guide-tone voicings. Strum as written: ⊓ down, ⊔ up. *The rests are the exercise.*],
  image("b4.cropped.png", width: 100%),
  [let the rests breathe; top voice moves by step.])

#block_("5", "Improv on a Model", "6 min", [♩=76],
  [Over the changes above. Play the model ×2, then the variation ×2, then keep going.],
  [
    #text(size: 8.5pt, weight: "bold")[model — play ×2]
    #image("b5a.cropped.png", width: 100%)
    #v(1pt)
    #text(size: 8.5pt, weight: "bold")[variation of bars 1–2 — then play ×2]
    #image("b5b.cropped.png", width: 100%)
    #v(1pt)
    #text(size: 9pt, style: "italic")[then: keep going — same rhythm, your notes. Rest at least once per bar.]
  ],
  [end phrases on purpose. Silence is a note you chose.])

#v(4pt)
#block(width: 100%, inset: 6pt, stroke: (dash: "dashed", paint: luma(55%), thickness: 0.6pt), radius: 3pt)[
  #text(size: 9pt, weight: "bold")[Log] #h(6pt)
  #text(size: 9pt)[felt free: #box(width: 4.6cm, repeat[.]) #h(3pt) felt stuck: #box(width: 4.6cm, repeat[.]) #h(3pt) keep: #box(width: 3.4cm, repeat[.])]
]
