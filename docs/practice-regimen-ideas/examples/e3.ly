\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
\score { \new Staff { \clef "treble_8" \relative c' {
  \key d \major \time 3/4
  d4\mf fis8 gis a4 | b8 a gis4 fis8 e | d4 gis8 fis e4 | d2.\fermata \bar "|."
} } \layout {} }
