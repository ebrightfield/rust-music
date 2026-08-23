\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
\score { \new Staff { \clef "treble_8" \relative c' {
  \key bes \major \time 4/4
  bes4\mf d8 c bes4 r8 f' | ees4 d8 c d2\fermata \bar "|."
} } \layout {} }
