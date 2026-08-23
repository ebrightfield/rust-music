\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
model = {
  \key c \major \time 5/4 \set fingeringOrientations = #'(up)
  c'8\3-2( d'\3-4) e'\2-1 fis'\2-3 r4 aes'\1-1( bes'\1-3) c''4\1-4 |
  bes'8\1-3 aes'\1-1 fis'\2-3( e'\2-1) r4 d'8\3-4 c'\3-2 c'4\3-2\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \model }
            \new TabStaff { \model } >> \layout {} }
