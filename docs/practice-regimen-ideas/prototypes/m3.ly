\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
spider = {
  \key c \major \time 4/4 \set fingeringOrientations = #'(up)
  % anchors: 1st finger str4 fr5, 3rd finger str4 fr7. 2nd/4th reach to str5 and str3.
  g8\4-1\mp ees\5-2 a\4-3 cis'\3-4 g\4-1 ees\5-2 a\4-3 cis'\3-4 |
  aes\4-1 e\5-2 bes\4-3 d'\3-4 aes\4-1 e\5-2 bes\4-3 d'4\3-4 \bar "|."
}
\score { << \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f \spider }
            \new TabStaff { \spider } >> \layout {} }
