\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineX = {
  \key a \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #4
  a8\4-3 e'8\2-1 des'8\3-2 f'8\2-2 bes8\4-3 f'8\2-1 ees'8\3-3 aes'8\2-4 |
  des'8\3-2 aes'8\2-4 e'8\2-1 a'8\1-1 ees'8\3-4 a'8\1-1 f'8\2-2 bes'8\1-2 |
  e'8\2-1 bes'8\1-2 aes'8\2-4 des''8\1-4 f'8\2-2 des''8\1-4 a'8\1-1 ees''8\1-4 |
  aes'8\2-4 ees''8\1-4 bes'8\1-1 e''8\1-4 r2 |
  \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineX }
            \new TabStaff { \lineX } >> \layout {} }
