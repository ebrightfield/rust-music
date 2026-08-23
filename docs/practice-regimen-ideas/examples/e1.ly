\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
% LAGE harvest: open with a "reset chord" — one voicing, fermata, listen. Then the warmup.
% WIDE by design: all six strings, fingers 1-2-4, ascending legato then descending staccato.
warm = {
  \key d \major \time 4/4
  % reset chord: let it ring, ppp, breathe
  <d\4 a\3 d'\2 fis'\1>1\ppp\fermata \bar "||"
  \time 12/8 \set TabStaff.minimumFret = #2
  \set fingeringOrientations = #'(up)
  fis,8\6-1( g,\6-2 a,\6-4) b,\5-1( c\5-2 d\5-4) e\4-1( f\4-2 g\4-4) a\3-1( bes\3-2 c'\3-4) |
  cis'\2-1( d'\2-2 e'\2-4) fis'\1-1( g'\1-2 a'\1-4)
  a'\1-4-. g'\1-2-. fis'\1-1-. e'\2-4-. d'\2-2-. cis'\2-1-. |
  c'\3-4-. bes\3-2-. a\3-1-. g\4-4-. f\4-2-. e\4-1-.
  d\5-4-. c\5-2-. b,\5-1-. a,\6-4-. g,\6-2-. fis,\6-1-. \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \warm }
            \new TabStaff { \warm } >> \layout {} }
