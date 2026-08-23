\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
npsTwo = {
  \key g \dorian \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #3
  g,8\6-1 bes,\6-4 c\5-1 e\5-4 f\4-1 a\4-4 bes\3-1 d'\3-4 |
  e'\2-1 g'\2-4 a'\1-1 c''\1-4 r2 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \npsTwo }
            \new TabStaff { \npsTwo } >> \layout {} }
