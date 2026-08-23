\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
mech = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  \set TabStaff.minimumFret = #5 \set TabStaff.restrainOpenStrings = ##t
  % ASCEND pos V: 4 notes per string, fingers 1-2-3-4, all six strings. Slurred (legato).
  a,16\6-1\mp( bes,\6-2 b,\6-3 c\6-4) d\5-1( ees\5-2 e\5-3 f\5-4)
  g\4-1( aes\4-2 a\4-3 bes\4-4) c'\3-1( cis'\3-2 d'\3-3 ees'\3-4) |
  e'\2-1( f'\2-2 fis'\2-3 g'\2-4) a'\1-1( bes'\1-2 b'\1-3 c''\1-4)
  \set TabStaff.minimumFret = #8
  % DESCEND pos VIII: fingers 4-3-2-1, staccato, back down all six strings.
  ees''\1-4-. d''\1-3-. cis''\1-2-. c''\1-1-. bes'\2-4-. a'\2-3-. aes'\2-2-. g'\2-1-. |
  fis'\3-4-. f'\3-3-. e'\3-2-. ees'\3-1-. cis'\4-4-. c'\4-3-. b\4-2-. bes\4-1-.
  aes\5-4-. g\5-3-. fis\5-2-. f\5-1-. ees\6-4-. d\6-3-. cis\6-2-. c\6-1-. \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \mech }
            \new TabStaff { \mech } >> \layout {} }
