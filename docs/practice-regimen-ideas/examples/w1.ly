\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
mech = {
  \key c \major \time 5/4
  \set fingeringOrientations = #'(up)
  % p-i-m-a on Cm(add9) shape: thumb=str5, i=str4, m=str3, a=str2
  c8\5\mp g\4 bes\3 d'\2 bes\3 g\4 c\5 g\4 bes\3 d'\2 |
  c8\5\< g\4 bes\3 d'\2 bes\3 g\4 c\5 g\4 bes\3 d'\2\! |
  c4\5\mf\> g\4 bes\3 d'2\2\! \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \mech }
            \new TabStaff { \mech } >> \layout {} }
