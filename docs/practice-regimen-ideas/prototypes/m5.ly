\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% Octave Exercise (port of Octave Exercise.pdf): the SAME pitch found in two places,
% then up an octave. Note-rehearsal + neck mapping. Explicit frets via \minimumFret.
oct = {
  \key c \major \time 4/4
  \set TabStaff.minimumFret = #7 \set TabStaff.restrainOpenStrings = ##t
  c2\6 \set TabStaff.minimumFret = #3 c2\5 |
  \set TabStaff.minimumFret = #10 c'2\4 \set TabStaff.minimumFret = #5 c'2\3 |
  \set TabStaff.minimumFret = #13 c''2\2 \set TabStaff.minimumFret = #8 c''2\1 |
  \set TabStaff.minimumFret = #5 c'2\3 \set TabStaff.minimumFret = #3 c2\5 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \oct }
            \new TabStaff { \oct } >> \layout {} }
