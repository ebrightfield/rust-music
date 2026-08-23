\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% REVISED: drop-2 voicings frets 4-9, no open-string chords. Comping is anticipated and
% syncopated (chords land on the & of 4 into the next bar), with a chromatic approach chord.
% Was: open-position G/Em/A/D with a Charleston stab — too easy, and 3-x-0-0-0-x is not a
% voicing a jazz player reaches for.
comp = {
  \key g \major \time 4/4
  \set TabStaff.minimumFret = #4
  % G6 : anticipate beat 3, leave beat 1 of the answer open
  <d\5 g\4 b\3 e'\2>4.\mf-> r8 <d\5 g\4 b\3 e'\2>8 ~ q4 <e\5 b\4 d'\3 g'\2>8 ~ |
  % Em7 -> chromatic approach (Fm7-ish shape a fret below A7) -> A7
  q4 r8 <e\5 b\4 d'\3 g'\2>8 <f\5 bes\4 d'\3 aes'\2>8-. <e\5 a\4 cis'\3 g'\2>4.-> |
  % A7 held, then D7 anticipated on the & of 4
  <e\5 a\4 cis'\3 g'\2>2 r8 <d\5 a\4 c'\3 fis'\2>8 ~ q4 |
  % D7 -> G6 resolution, syncopated
  <d\5 a\4 c'\3 fis'\2>8 r8 q4 <d\5 g\4 b\3 e'\2>2\fermata \bar "|."
}
\score { <<
  \new ChordNames \chordmode { g1:6 | e2:m7 f8:m7 a4.:7 | a2:7 d2:7 | d2:7 g2:6 }
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \comp }
  \new TabStaff { \comp } >> \layout {} }
