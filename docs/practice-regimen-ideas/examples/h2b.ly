\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineX = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #4
  b'8\1-2 e'8\3-4 g'8\2-3 des'8\3-1 aes'8\2-4 des'8\3-1 ges'8\2-2 c'8\4-4 |
  g'8\2-1 c'8\4-3 e'8\3-2 b8\4-2 ges'8\2-2 b8\4-4 des'8\3-1 aes8\4-1 |
  e'8\3-4 aes8\4-1 c'8\4-4 g8\5-4 des'8\3-1 g8\5-4 b8\4-4 ges8\5-4 |
  c'8\4-4 ges8\5-4 aes8\4-1 e8\5-2 r2 |
  \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineX }
            \new TabStaff { \lineX } >> \layout {} }
