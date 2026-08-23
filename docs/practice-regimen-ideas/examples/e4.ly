\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% TOWNER harvest: open strings woven into fretted voicings. Wide spacing, sparse low register.
% Open strings drawn grey. Arpeggiate p-i-m-a, let everything ring.
opn = \once \override NoteHead.color = #(rgb-color 0.45 0.45 0.45)
towner = {
  \key d \major \time 4/4
  % Dmaj: D(open4) G#(3.1) B(open2) F#(1.2)  -- spread 16st, 2 opens
  <d\4 gis\3 b\2 fis'\1>1 |
  % A: A(open5) D(open4) B(open2) F#(1.2) -- spread 21st, 3 opens
  <a,\5 d\4 b\2 fis'\1>1 |
  % F#m: F#(6.2) D(open4) G#(3.1) E(open1) -- spread 22st
  <fis,\6 d\4 gis\3 e'\1>1 |
  % E: E(open6) D(open4) B(open2) F#(1.2) -- spread 26st, 3 opens
  <e,\6 d\4 b\2 fis'\1>1 \bar "|."
}
\score { <<
  \new ChordNames \chordmode { d1:maj9^5 | a1:sus4.9 | fis1:m11 | e1:9sus4 }
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \towner }
  \new TabStaff { \towner } >> \layout {} }
