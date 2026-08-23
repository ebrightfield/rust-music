\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
vari = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #5
  r16 c''\1-4 b'\1-3 aes'\1-2 \tuplet 3/2 { g'\2-4 ges'\2-3 e'\2-1 } des'8\3-2\glissando e'\2-1 g'4\2-4 |
  \tuplet 3/2 { b'8\1-3 aes'\1-2 g'\2-4 } ges'8\2-3( e'\2-1) des'4\3-2 r4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \vari }
            \new TabStaff { \vari } >> \layout {} }
