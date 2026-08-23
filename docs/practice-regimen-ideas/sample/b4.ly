\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% Cm7 (3-5-3-4) -> F7 (3-3-2-4) -> Bbmaj7 (1-3-2-3). Notated strum: rests are written in.
comp = {
  \key bes \major \time 4/4
  <c\5 g\4 bes\3 ees'\2>4\mf-\downbow r8 <c\5 g\4 bes\3 ees'\2>8-\upbow ~ q4 r4 |
  <c\5 f\4 a\3 ees'\2>4-\downbow r8 <c\5 f\4 a\3 ees'\2>8-\upbow ~ q4 r4 |
  <bes,\5 f\4 a\3 d'\2>4-\downbow r8 <bes,\5 f\4 a\3 d'\2>8-\upbow ~ q2 |
  <bes,\5 f\4 a\3 d'\2>1\fermata \bar "|."
}
\score { <<
  \new ChordNames \chordmode { c1:m7 | f1:7 | bes1:maj7 | bes1:maj7 }
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \comp }
  \new TabStaff { \comp } >> \layout {} }
