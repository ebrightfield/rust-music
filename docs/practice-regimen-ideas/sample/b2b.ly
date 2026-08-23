\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineB = {
  \key bes \major \time 4/4 \set fingeringOrientations = #'(up)
  bes8\4-4 d'\3-3 c'\3-1 ees'\3-4 d'\3-3 g'\2-4 ees'\3-4 a'\1-1 |
  bes8\4-4 d'\3-3 c'\3-1 ees'\3-4 d'\3-3 g'\2-4 ees'\3-4 a'\1-1 \bar "||"
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineB }
            \new TabStaff { \lineB } >> \layout {} }
