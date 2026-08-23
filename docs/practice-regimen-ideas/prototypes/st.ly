\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% strum pattern x.xx.xx. on a G6 voicing, stroke marks from strumpat.py
comp = {
  \key g \major \time 4/4
  <g, d b e'>8\mf-\downbow ~ q8 q8-\downbow q8-\upbow ~ q8 q8-\upbow q8-\downbow ~ q8 \bar "|."
}
\score { << \new ChordNames \chordmode { g1:6 }
            \new Staff { \clef "treble_8" \comp }
            \new TabStaff { \comp } >> \layout {} }
