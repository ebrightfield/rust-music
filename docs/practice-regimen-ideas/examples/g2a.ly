\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% Bends notated chart-style: slur + "full"/"1/2" markup above. Reads unambiguously on paper.
bendFull = ^\markup { \teeny "full" }
bendHalf = ^\markup { \teeny "½" }
lineA = {
  \key g \major \time 4/4 \set fingeringOrientations = #'(up)
  g8\4-1\mf a\4-3 d'\3-3 e'\3-1 g'\2-4\bendFull( a'\2-4) e'\3-1 d'\3-3 |
  g4\4-1 r8 d'\3-3 e'8\3-1\bendHalf( f'\3-1) g'2\2-4\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineA }
            \new TabStaff { \lineA } >> \layout {} }
