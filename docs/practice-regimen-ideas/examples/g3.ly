\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
\score { \new Staff { \clef "treble_8" \relative c' {
  \key g \major \time 4/4
  g4\mf b8 d e4 d8 b | a4 g8 e d4 g8 a | b2 r4 g4\fermata \bar "|."
} } \layout {} }
