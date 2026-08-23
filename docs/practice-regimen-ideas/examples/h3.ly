\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
\score { \new Staff { \clef "treble_8" \relative c' {
  \time 5/8 c8\mf des e ges4 | \time 7/8 g8 aes b c des4 |
  \time 5/8 b8 aes ges e4 | \time 7/8 des8 c b, c des4\fermata \bar "|."
} } \layout {} }
