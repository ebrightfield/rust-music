\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
mech = {
  \key bes \major \time 4/4
  a,8\6\mp( b,\6 ais,\6 c\6) a,\6-. b,\6-. ais,\6-. c\6-. |
  d\5( e\5 dis\5 f\5) d\5-. e\5-. dis\5-. f\5-. |
  g\4( a\4 gis\4 bes\4) g\4-. a\4-. gis\4-. bes\4-. |
  c'\3( d'\3 cis'\3 ees'\3) c'\3-. d'\3-. cis'\3-. ees'\3-. \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \mech }
            \new TabStaff { \mech } >> \layout {} }
