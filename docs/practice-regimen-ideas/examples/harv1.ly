\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% HARVEST: McLaughlin — chromatic tetrachords (1-2-3-4), strict alternate picking,
% OPEN STRING used as the connector between groupings (his device for fast tempi / odd meters).
mcl = {
  \key d \minor \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #5 \set TabStaff.restrainOpenStrings = ##f
  g16\4-1\mf^\markup{\teeny "D"} aes\4-2^\markup{\teeny "U"} a\4-3^\markup{\teeny "D"} bes\4-4^\markup{\teeny "U"}
  \once \override NoteHead.color = #(rgb-color 0.45 0.45 0.45) g8\3^\markup{\teeny "open"}
  c'16\3-1 cis'\3-2 d'\3-3 ees'\3-4 |
  \once \override NoteHead.color = #(rgb-color 0.45 0.45 0.45) b8\2^\markup{\teeny "open"}
  e'16\2-1 f'\2-2 fis'\2-3 g'\2-4
  \once \override NoteHead.color = #(rgb-color 0.45 0.45 0.45) e'8\1^\markup{\teeny "open"}
  a'16\1-1 bes'\1-2 b'\1-3 c''\1-4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \mcl }
            \new TabStaff { \mcl } >> \layout {} }
