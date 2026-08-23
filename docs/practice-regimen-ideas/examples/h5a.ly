\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
model = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #5
  c'8\3-1 e'\2-1( ges'\2-3) b'\1-3 \tuplet 3/2 { aes'\1-2 g'\2-4 ges'\2-3 } e'4\2-1 |
  des'8\3-2\glissando e'\2-1 g'\2-4 \parenthesize ges'\2-3 b'4\1-3 r8 aes'8\1-2 |
  \tuplet 3/2 { c''\1-4 b'\1-3 aes'\1-2 } \tuplet 3/2 { g'\2-4 ges'\2-3 e'\2-1 } des'4\3-2 r4 |
  e'8\2-1 ges'\2-3 aes'\1-2 b'\1-3 c''2\1-4\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \model }
            \new TabStaff { \model } >> \layout {} }
