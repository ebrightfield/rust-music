\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
mech = {
  \key g \major \time 4/4 \set fingeringOrientations = #'(up)
  % Climbing on ③: 1st finger walks 2-3-4 toward the 4th finger anchored at 5, then shift up one.
  a8\3-1\mp c'\3-4 bes\3-1 c'\3-4 b\3-1 c'\3-4 b\3-1 c'\3-4 |
  bes8\3-1 cis'\3-4 b\3-1 cis'\3-4 c'\3-1 cis'\3-4 c'4\3-4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \mech }
            \new TabStaff { \mech } >> \layout {} }
