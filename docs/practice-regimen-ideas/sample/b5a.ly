\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
model = {
  \key bes \major \time 4/4 \set fingeringOrientations = #'(up)
  % m1 Cm7 (8 eighths): 5 notes + r4. (3 eighths)
  c'8\2-1 ees'\2-4 g'\1-3 bes'\1-4\glissando c''\1-4 r4. |
  % m2 F7: 6 eighths + r4 (2 eighths)
  a'8\1-3 g'\1-1 f'\2-4 ees'\2-2 d'\2-1 c'\2-1 r4 |
  % m3 Bb: 7 eighths + r8
  bes8\3-1 d'\2-3 f'\1-1 bes'\1-4 a'\1-3 g'\1-1 f'\2-4 r8 |
  % m4 Bb: 4 eighths + half note
  ees'8\2-4 d'\2-3 c'\2-1 d'\2-3 d'2\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \model }
            \new TabStaff { \model } >> \layout {} }
