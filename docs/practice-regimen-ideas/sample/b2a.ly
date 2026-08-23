\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
lineA = {
  \key bes \major \time 4/4 \set fingeringOrientations = #'(up)
  bes8\3-1 d'\2-3 c'\2-1 ees'\2-4 d'\2-3 g'\1-3 ees'\2-4 a'\1-5 |
  bes8\3-1 d'\2-3 c'\2-1 ees'\2-4 d'\2-3 g'\1-3 ees'\2-4 a'\1-5 \bar "||"
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineA }
            \new TabStaff { \lineA } >> \layout {} }
