\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% LAGE harvest: hold one voice while another moves -> two apparent parts, one guitar.
% Upper voice sustains on ① (F#4 fr2 / E4 fr0). Lower line walks on ④③②, frets 2-7.
upper = {
  \key d \major \time 4/4 \voiceOne
  fis'4\1~ fis'2. | e'1~ | e'2 fis'2~ | fis'1 \bar "|."
}
lower = {
  \key d \major \time 4/4 \voiceTwo
  d8\4 fis\4 a\3 b\3 a\3 fis\4 d\4 e\4 |
  fis8\4 a\3 b\3 cis'\3 b\3 a\3 fis\4 e\4 |
  d8\4 e\4 fis\4 gis\4 a\3 b\3 a\3 fis\4 |
  e8\4 fis\4 gis\4 a\3 b\3 a\3 fis\4 d\4 \bar "|."
}
\score {
  <<
    \new Staff \with { \override StringNumber.stencil = ##f } {
      \clef "treble_8" << \upper \\ \lower >>
    }
    \new TabStaff { << \upper \\ \lower >> }
  >>
  \layout {}
}
