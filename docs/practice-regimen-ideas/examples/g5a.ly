\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
bendHalf = ^\markup { \teeny "½" }
model = {
  \key g \major \time 4/4 \set fingeringOrientations = #'(up)
  % G6: pickup into a bend, rest on 4
  d'8\3-1 e'\3-3 g'\2-2\bendHalf( a'\2-4) g'8\2-2 e'\3-3 r4 |
  % Em7: descending, land on D
  e'8\3-3 d'\3-1 b\3-4 d'\3-1 e'4\3-3 r4 |
  % A7: chromatic approach to C#
  e'8\3-3 g'\2-2 a'\2-4 g'\2-2 e'8\3-3 d'\3-1 r4 |
  % D7 -> G6: resolve
  a'8\2-4 g'\2-2 e'\3-3 d'\3-1 g'2\2-2\fermata \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \model }
            \new TabStaff { \model } >> \layout {} }
