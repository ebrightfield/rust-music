\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
\score { \new Staff { \clef "treble_8" \relative c' {
  \key c \major \time 5/4
  c4\mf d8 e r8 fis4 gis8 fis | e4 d8 c r4 c2 \bar "|."
} } \layout {} }
