\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
vari = {
  \key c \major \time 5/4 \set fingeringOrientations = #'(up)
  r4 c''8\1-4( bes'\1-3) aes'\1-1 fis'\2-3 e'\2-1( d'\3-4) c'4\3-2 |
  e'8\2-1 fis'\2-3 aes'4\1-1 r4 c''8\1-4 bes'\1-3 aes'4\1-1 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \vari }
            \new TabStaff { \vari } >> \layout {} }
