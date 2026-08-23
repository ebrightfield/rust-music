\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
bendFull = ^\markup { \teeny "full" }
bendHalf = ^\markup { \teeny "½" }
lineB = {
  \key g \major \time 4/4 \set fingeringOrientations = #'(up)
  d'8\3-1\mf e'\3-3 g'\2-2 a'\2-4 g'\2-2\bendFull( a'\2-4) e'\3-3 d'\3-1 |
  d'4\3-1 r8 g'\2-2 a'8\2-4\bendHalf( bes'\2-4) g'2\2-2\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineB }
            \new TabStaff { \lineB } >> \layout {} }
