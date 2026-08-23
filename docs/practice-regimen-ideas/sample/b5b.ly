\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
vari = {
  \key bes \major \time 4/4 \set fingeringOrientations = #'(up)
  % m1 inverted: descend from top, displaced by an eighth
  r8 c''\1-4 bes'\1-4\glissando g'\1-3 ees'\2-4 c'\2-1 r4 |
  % m2 same notes, rhythm compressed then rest
  a'8\1-3 g'\1-1 f'\2-4 ees'\2-2 d'4\2-1 r4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \vari }
            \new TabStaff { \vari } >> \layout {} }
