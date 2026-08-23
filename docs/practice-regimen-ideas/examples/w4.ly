\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% REVISED: the point is NOT "slide C+ up two frets to D+" (obvious, nothing to practise).
% C+ = C E Ab. That ONE grip is the altered dominant in THREE keys a major 3rd apart:
%   C7#5 -> F      E7#5 -> A      Ab7#5 -> Db
% So the same shape does three harmonic jobs. Keep the grip, change what surrounds it.
comp = {
  \key c \major \time 4/4
  % ii - V(same grip) - I  in F
  <f\5 bes\4 d'\3 g'\2>2 <c'\4 e'\3 aes'\2>2 | <e\5 a\4 c'\3 f'\2>1 |
  % ii - V(SAME grip) - I  in A
  <fis\5 b\4 d'\3 a'\2>2 <c'\4 e'\3 aes'\2>2 | <aes\5 cis'\4 e'\3 a'\2>1 |
  % ii - V(SAME grip) - I  in Db
  <cis\5 fis\4 bes\3 ees'\2>2 <c'\4 e'\3 aes'\2>2 | <cis\5 aes\4 c'\3 f'\2>1\fermata \bar "|."
}
\score { <<
  \new ChordNames \chordmode { g2:m7 c2:7.5+ | f1:maj7 | b2:m7 c2:7.5+ | a1:maj7 | ees2:m7 c2:7.5+ | des1:maj7 }
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \comp }
  \new TabStaff { \comp } >> \layout {} }
