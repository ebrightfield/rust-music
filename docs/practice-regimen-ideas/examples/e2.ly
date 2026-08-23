\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% D Lydian, tuple 1,[3,-1]. Fingerstyle (p-i-m-a). 21 notes = 7 fragments of 3, all degrees.
lineA = {
  \key d \major \time 3/4 \set fingeringOrientations = #'(up)
  d8\5-2 gis8\4-3 fis8\4-1 e8\5-2 a8\4-2 gis8\4-1 |
  fis8\4-1 b8\3-1 a8\4-4 gis8\4-3 cis'8\3-3 b8\3-1 |
  a8\4-2 d'8\3-2 cis'8\3-1 b8\3-1 e'8\2-2 d'8\3-4 |
  cis'8\3-2 fis'8\2-3 e'8\2-1 d'4.\3-2 |
  \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineA }
            \new TabStaff { \lineA } >> \layout {} }
