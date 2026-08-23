\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineA = {
  \key c \major \time 5/4 \set fingeringOrientations = #'(up)
  fis8\4-1( aes\4-3) c'\3-2( d'\3-4) e'\2-1( fis'\2-3) aes'\1-1( bes'\1-3) c''\1-4 r8 |
  bes'8\1-3( aes'\1-1) fis'\2-3( e'\2-1) d'\3-4( c'\3-2) aes\4-3( fis\4-1) fis2\4-1 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineA }
            \new TabStaff { \lineA } >> \layout {} }
