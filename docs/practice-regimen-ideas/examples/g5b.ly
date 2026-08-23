\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
bendFull = ^\markup { \teeny "full" }
vari = {
  \key g \major \time 4/4 \set fingeringOrientations = #'(up)
  % displaced by an eighth, bend widened to a full step
  r8 d'8\3-1 e'\3-3 g'\2-2\bendFull( a'\2-4) g'8\2-2 e'4\3-3 |
  r8 e'8\3-3 d'\3-1 b\3-4 d'4\3-1 r4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \vari }
            \new TabStaff { \vari } >> \layout {} }
