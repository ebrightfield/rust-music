\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
npsThree = {
  \key g \dorian \time 6/8 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #3
  g,8\6-1 a,\6-3 bes,\6-4 c\5-1 d\5-3 e\5-4 | f\4-1 g\4-3 a\4-4 bes\3-1 c'\3-3 d'\3-4 |
  e'\2-1 f'\2-2 g'\2-4 a'\1-1 bes'\1-2 c''\1-4 \bar "||"
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \npsThree }
            \new TabStaff { \npsThree } >> \layout {} }
