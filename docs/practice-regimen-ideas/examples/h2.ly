\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% Vocabulary block, example D. Pattern 1,[4,-2,3] over C Lydian b9 b13, ALL SEVEN degrees,
% ascending then descending, then the same tuple transposed down a m3 (A Lyd b9b13).
% The transposition is NOTATED, not described -- Rule 0 (silver platter) + Rule 0b (volume).
lineA = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #4
  % --- ASCEND: C Lyd b9b13, tuple 1,[4,-2,3], all 7 degrees ---
  c'8\4-3 g'8\2-1 e'8\3-2 aes'8\2-2 des'8\4-3 aes'8\2-1 ges'8\3-3 b'8\2-4 |
  e'8\3-2 b'8\2-4 g'8\2-1 c''8\1-1 ges'8\3-4 c''8\1-1 aes'8\2-2 des''8\1-2 |
  g'8\2-1 des''8\1-2 b'8\2-4 e''8\1-4 aes'8\2-2 e''8\1-4 c''8\1-1 ges''8\1-4 |
  b'8\2-4 ges''8\1-4 des''8\1-1 g''8\1-4 r2 |
  % --- DESCEND: tuple negated 1,[-4,2,-3] ---
  b'8\1-2 e'8\3-4 g'8\2-3 des'8\3-1 aes'8\2-4 des'8\3-1 ges'8\2-2 c'8\4-4 |
  g'8\2-1 c'8\4-3 e'8\3-2 b8\4-2 ges'8\2-2 b8\4-4 des'8\3-1 aes8\4-1 |
  e'8\3-4 aes8\4-1 c'8\4-4 g8\5-4 des'8\3-1 g8\5-4 b8\4-4 ges8\5-4 |
  c'8\4-4 ges8\5-4 aes8\4-1 e8\5-2 r2 |
  % --- SAME TUPLE DOWN A MINOR 3RD: A Lyd b9b13 (notated, not described) ---
  a8\4-3 e'8\2-1 des'8\3-2 f'8\2-2 bes8\4-3 f'8\2-1 ees'8\3-3 aes'8\2-4 |
  des'8\3-2 aes'8\2-4 e'8\2-1 a'8\1-1 ees'8\3-4 a'8\1-1 f'8\2-2 bes'8\1-2 |
  e'8\2-1 bes'8\1-2 aes'8\2-4 des''8\1-4 f'8\2-2 des''8\1-4 a'8\1-1 ees''8\1-4 |
  aes'8\2-4 ees''8\1-4 bes'8\1-1 e''8\1-4 r2 |
  \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \lineA }
            \new TabStaff { \lineA } >> \layout {} }
