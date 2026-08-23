\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% Voicings on the TOP FOUR strings, gradual adjacent-string deltas (max |2|).
% Revised: the earlier x-7-9-5-7-x had a -4 backwards jump mid-chord across str4->str3 —
% span 4 but a brutal reverse-stretch. See tools/fret2ly.py MAX_ADJ_BACK.
% Top voice is a chromatic pedal C -> C# -> C (what these shapes actually give).
comp = {
  \key c \major \time 4/4
  \set TabStaff.minimumFret = #6
  <b\4 e'\3 fis'\2 c''\1>2\mf-> <c'\4 e'\3 bes'\2 cis''\1>2 |
  <gis\4 ees'\3 g'\2 c''\1>2 -> <b\4 e'\3 fis'\2 c''\1>2 |
  <c'\4 e'\3 bes'\2 cis''\1>4\< <gis\4 ees'\3 g'\2 c''\1>4
  <b\4 e'\3 fis'\2 c''\1>4 <c'\4 e'\3 bes'\2 cis''\1>4\! |
  <b\4 e'\3 fis'\2 c''\1>1\fermata \bar "|."
}
\score { <<
  \new ChordNames \chordmode { c2:maj7.11+ c2:7.9- | aes2:maj7 c2:maj7.11+ | c4:7.9- aes4:maj7 c4:maj7.11+ c4:7.9- | c1:maj7.11+ }
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \comp }
  \new TabStaff { \comp } >> \layout {} }
