\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineX = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #4
  c'8\4-3 g'8\2-1 e'8\3-2 aes'8\2-2 des'8\4-3 aes'8\2-1 ges'8\3-3 b'8\2-4 |
  e'8\3-2 b'8\2-4 g'8\2-1 c''8\1-1 ges'8\3-4 c''8\1-1 aes'8\2-2 des''8\1-2 |
  g'8\2-1 des''8\1-2 b'8\2-4 e''8\1-4 aes'8\2-2 e''8\1-4 c''8\1-1 ges''8\1-4 |
  b'8\2-4 ges''8\1-4 des''8\1-1 g''8\1-4 r2 |
  \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineX }
            \new TabStaff { \lineX } >> \layout {} }
